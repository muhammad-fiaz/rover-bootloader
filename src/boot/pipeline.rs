//! # Boot pipeline
//!
//! Complete, robust boot orchestration pipeline from UEFI firmware entry to kernel handoff.

use core::ffi::c_void;

use super::menu::run_boot_menu;
use crate::arch::current_arch;
use crate::config::parser::BootConfig;
use crate::error::BootError;
#[cfg(target_arch = "aarch64")]
use crate::formats::elf::EM_AARCH64;
#[cfg(target_arch = "riscv64")]
use crate::formats::elf::EM_RISCV;
#[cfg(target_arch = "x86_64")]
use crate::formats::elf::EM_X86_64;
use crate::formats::elf::ElfBinary;
use crate::fs::uefi_fs::{close_file, open_file, query_file_size, read_file};
use crate::graphics::console::ConsoleWriter;
use crate::graphics::gop::discover_framebuffer;
use crate::memory::allocator::{allocate_pages, allocate_pool, free_pool};
use crate::memory::exit_boot::exit_boot_services_safe;
use crate::memory::map::{normalize_memory_map, parse_and_convert_memory_map};
use crate::protocol::framebuffer::RoverFramebuffer;
use crate::protocol::info::{
    ROVER_BOOT_MAGIC, ROVER_BOOT_PROTOCOL_VERSION, ROVER_FLAG_ACPI, ROVER_FLAG_CMDLINE,
    ROVER_FLAG_FRAMEBUFFER, ROVER_FLAG_INITRD, ROVER_FLAG_SECURE_BOOT, ROVER_FLAG_SMBIOS,
    RoverBootInfo,
};
use crate::protocol::memory::RoverMemoryDescriptor;
use crate::protocol::module::{RoverCommandLine, RoverKernelImage, RoverModule, RoverSystemInfo};
use crate::security::secure_boot::is_secure_boot_enabled;
use crate::security::sha256::verify_sha256_hex;
use rover_uefi::base::{AllocationType, EfiHandle, MemoryType};
use rover_uefi::protocols::acpi::discovery::find_config_entry as find_acpi_entry;
use rover_uefi::protocols::acpi::guid::{ACPI_10_GUID, ACPI_20_GUID};
use rover_uefi::protocols::driver::guid::LOADED_IMAGE_GUID;
use rover_uefi::protocols::driver::loaded_image::EfiLoadedImageProtocol;
use rover_uefi::protocols::smbios::discovery::find_config_entry as find_smbios_entry;
use rover_uefi::protocols::smbios::guid::{SMBIOS_2X_GUID, SMBIOS_3X_GUID};
use rover_uefi::protocols::storage::filesystem::file::EfiFileProtocol;
use rover_uefi::protocols::storage::filesystem::protocol::EfiSimpleFileSystemProtocol;
use rover_uefi::protocols::storage::guid::SIMPLE_FILE_SYSTEM_GUID;
use rover_uefi::tables::system::EfiSystemTable;

/// Memory page size in bytes.
const PAGE_SIZE: usize = 4096;

/// Number of pages to allocate for kernel execution stack (64 KiB).
const STACK_PAGES: usize = 16;

/// Maximum number of memory map descriptors supported.
const MAX_MEMORY_DESCRIPTORS: usize = 512;

/// Orchestrates the entire boot sequence.
///
/// # Safety
///
/// `system_table` must be a valid, live pointer to the [`EfiSystemTable`] passed by firmware.
///
/// # Errors
///
/// Returns a [`BootError`] if any phase of the boot orchestration fails.
#[allow(clippy::too_many_lines)]
pub unsafe fn execute_boot_pipeline(
    image_handle: EfiHandle,
    system_table: *const EfiSystemTable,
) -> Result<(), BootError> {
    if system_table.is_null() {
        return Err(BootError::DeviceNotFound);
    }

    let st = unsafe { &*system_table };
    let bs = st.boot_services;
    let con_out = st.con_out;
    let con_in = st.con_in;
    let rt = st.runtime_services;

    if bs.is_null() {
        return Err(BootError::DeviceNotFound);
    }

    let mut writer = ConsoleWriter::new(con_out);
    writer.set_color(11, 0); // Cyan
    writer.write_str("Rover Bootloader starting...\n");

    // 1. Locate LoadedImageProtocol for our image.
    let mut loaded_image_ptr: *mut c_void = core::ptr::null_mut();
    let mut loaded_image_guid = LOADED_IMAGE_GUID;
    let status = unsafe {
        ((*bs).handle_protocol)(
            image_handle,
            &raw mut loaded_image_guid,
            &raw mut loaded_image_ptr,
        )
    };

    if !status.is_success() || loaded_image_ptr.is_null() {
        return Err(BootError::DeviceNotFound);
    }

    let loaded_image = unsafe { &*loaded_image_ptr.cast::<EfiLoadedImageProtocol>() };
    let boot_device_handle = loaded_image.device_handle;

    // 2. Open SimpleFileSystem on boot device.
    let mut fs_proto_ptr: *mut c_void = core::ptr::null_mut();
    let mut fs_guid = SIMPLE_FILE_SYSTEM_GUID;
    let status = unsafe {
        ((*bs).handle_protocol)(boot_device_handle, &raw mut fs_guid, &raw mut fs_proto_ptr)
    };

    if !status.is_success() || fs_proto_ptr.is_null() {
        return Err(BootError::FileSystemNotFound);
    }

    let fs = fs_proto_ptr.cast::<EfiSimpleFileSystemProtocol>();
    let mut root: *mut EfiFileProtocol = core::ptr::null_mut();
    let status = unsafe { ((*fs).open_volume)(fs, &raw mut root) };

    if !status.is_success() || root.is_null() {
        return Err(BootError::FileSystemNotFound);
    }

    // 3. Search for configuration file `rover.cfg`.
    let config_paths = ["\\rover.cfg", "\\boot\\rover.cfg", "\\EFI\\BOOT\\rover.cfg"];

    let mut config_buffer = [0u8; 4096];
    let mut config_text_len = 0;

    for path in &config_paths {
        if let Ok(file) = unsafe { open_file(root, path) } {
            if let Ok(read_bytes) = unsafe { read_file(file, &mut config_buffer) } {
                unsafe { close_file(file) };
                if read_bytes > 0 {
                    config_text_len = read_bytes;
                    break;
                }
            } else {
                unsafe { close_file(file) };
            }
        }
    }

    let config = if config_text_len > 0 {
        if let Ok(text) = core::str::from_utf8(&config_buffer[..config_text_len]) {
            BootConfig::parse(text).unwrap_or_else(|_| BootConfig::fallback())
        } else {
            BootConfig::fallback()
        }
    } else {
        BootConfig::fallback()
    };

    // 4. Interactive boot menu.
    let selected_idx = unsafe { run_boot_menu(&config, &mut writer, con_in, bs) };
    let selected_entry = config
        .entries()
        .get(selected_idx)
        .and_then(|opt| opt.as_ref())
        .ok_or(BootError::NoBootableEntry)?;

    writer.clear();
    writer.set_color(10, 0); // Green
    writer.write_str("Loading kernel: ");
    writer.write_str(selected_entry.kernel_path);
    writer.write_str("\n");

    // 5. Discover Framebuffer via GOP.
    let framebuffer = unsafe { discover_framebuffer(bs, selected_entry.resolution) }
        .unwrap_or_else(|_| RoverFramebuffer::empty());

    // 6. Discover ACPI RSDP.
    let acpi_ptr = unsafe {
        let rsdp2 = find_acpi_entry(st, &ACPI_20_GUID);
        if rsdp2.is_null() {
            let rsdp1 = find_acpi_entry(st, &ACPI_10_GUID);
            rsdp1 as u64
        } else {
            rsdp2 as u64
        }
    };

    // 7. Discover SMBIOS.
    let smbios_ptr = unsafe {
        let smb3 = find_smbios_entry(st, &SMBIOS_3X_GUID);
        if smb3.is_null() {
            let smb2 = find_smbios_entry(st, &SMBIOS_2X_GUID);
            smb2 as u64
        } else {
            smb3 as u64
        }
    };

    // 8. Detect Secure Boot status.
    let secure_boot = unsafe { is_secure_boot_enabled(rt) };

    // 9. Read kernel binary file.
    let kernel_file = unsafe { open_file(root, selected_entry.kernel_path)? };
    let kernel_file_size = unsafe { query_file_size(kernel_file)? };

    if kernel_file_size == 0 || kernel_file_size > 128 * 1024 * 1024 {
        unsafe { close_file(kernel_file) };
        return Err(BootError::ImageTruncated);
    }

    let kernel_size = usize::try_from(kernel_file_size).map_err(|_| BootError::ImageTruncated)?;
    let kernel_pool = unsafe { allocate_pool(bs, MemoryType::LOADER_DATA, kernel_size)? };
    let kernel_raw_slice = unsafe { core::slice::from_raw_parts_mut(kernel_pool, kernel_size) };

    let read_result = unsafe { read_file(kernel_file, kernel_raw_slice) };
    unsafe { close_file(kernel_file) };
    read_result?;

    // 10. Check SHA-256 integrity if specified in entry.
    if let Some(expected_hash) = selected_entry.expected_sha256 {
        writer.write_str("Verifying kernel integrity...\n");
        if !verify_sha256_hex(kernel_raw_slice, expected_hash) {
            let _ = unsafe { free_pool(bs, kernel_pool) };
            return Err(BootError::SecurityVerificationFailed);
        }
    }

    // 11. Parse ELF executable.
    let elf = ElfBinary::parse(kernel_raw_slice)?;

    // Validate machine architecture.
    #[cfg(target_arch = "x86_64")]
    elf.validate_architecture(EM_X86_64)?;
    #[cfg(target_arch = "aarch64")]
    elf.validate_architecture(EM_AARCH64)?;
    #[cfg(target_arch = "riscv64")]
    elf.validate_architecture(EM_RISCV)?;

    let (min_addr, max_addr) = elf.memory_bounds()?;
    let mem_span = max_addr
        .checked_sub(min_addr)
        .ok_or(BootError::SegmentOutOfBounds)?;
    let mem_span_usize = usize::try_from(mem_span).map_err(|_| BootError::SegmentOutOfBounds)?;
    let total_pages = mem_span_usize.div_ceil(PAGE_SIZE);

    // Allocate physical memory pages for kernel image.
    let kernel_phys_base = unsafe {
        allocate_pages(
            bs,
            AllocationType::ADDRESS,
            MemoryType::LOADER_DATA,
            total_pages,
            min_addr,
        )
        .or_else(|_| {
            allocate_pages(
                bs,
                AllocationType::ANY_PAGES,
                MemoryType::LOADER_DATA,
                total_pages,
                0,
            )
        })?
    };

    let dest_kernel_slice = unsafe {
        core::slice::from_raw_parts_mut(kernel_phys_base as *mut u8, total_pages * PAGE_SIZE)
    };

    elf.load_into_memory(dest_kernel_slice, min_addr)?;
    let kernel_entry_point = elf.entry_point();

    // Free the temporary kernel file buffer.
    let _ = unsafe { free_pool(bs, kernel_pool) };

    // 12. Load initrd if specified.
    let mut initrd_module = RoverModule::empty();
    if let Some(initrd_path) = selected_entry.initrd_path {
        writer.write_str("Loading initrd: ");
        writer.write_str(initrd_path);
        writer.write_str("\n");

        if let Ok(initrd_file) = unsafe { open_file(root, initrd_path) } {
            #[allow(clippy::collapsible_if)]
            if let Ok(size) = unsafe { query_file_size(initrd_file) } {
                if let Ok(size_usize) = usize::try_from(size) {
                    if size_usize > 0 {
                        let initrd_pages = size_usize.div_ceil(PAGE_SIZE);
                        if let Ok(initrd_phys) = unsafe {
                            allocate_pages(
                                bs,
                                AllocationType::ANY_PAGES,
                                MemoryType::LOADER_DATA,
                                initrd_pages,
                                0,
                            )
                        } {
                            let initrd_slice = unsafe {
                                core::slice::from_raw_parts_mut(initrd_phys as *mut u8, size_usize)
                            };
                            if unsafe { read_file(initrd_file, initrd_slice) }.is_ok() {
                                initrd_module.physical_start = initrd_phys;
                                initrd_module.size = size;
                            }
                        }
                    }
                }
            }
            unsafe { close_file(initrd_file) };
        }
    }

    // 13. Copy command line string to allocated buffer.
    let mut cmdline_desc = RoverCommandLine::empty();
    if let Some(cmd_str) = selected_entry.cmdline {
        let cmd_bytes = cmd_str.as_bytes();
        let cmd_pages = (cmd_bytes.len() + 1).div_ceil(PAGE_SIZE);
        if let Ok(cmd_phys) = unsafe {
            allocate_pages(
                bs,
                AllocationType::ANY_PAGES,
                MemoryType::LOADER_DATA,
                cmd_pages,
                0,
            )
        } {
            let cmd_slice = unsafe {
                core::slice::from_raw_parts_mut(cmd_phys as *mut u8, cmd_bytes.len() + 1)
            };
            cmd_slice[..cmd_bytes.len()].copy_from_slice(cmd_bytes);
            cmd_slice[cmd_bytes.len()] = 0; // Null terminator
            cmdline_desc.ptr = cmd_phys;
            cmdline_desc.length = cmd_bytes.len() as u64;
        }
    }

    // 14. Allocate stack for kernel (64 KiB).
    let stack_base = unsafe {
        allocate_pages(
            bs,
            AllocationType::ANY_PAGES,
            MemoryType::LOADER_DATA,
            STACK_PAGES,
            0,
        )
    }
    .map_err(|_| BootError::StackAllocationFailed)?;

    let stack_top = stack_base.saturating_add((STACK_PAGES * PAGE_SIZE) as u64);

    // 15. Allocate space for normalized RoverMemoryDescriptor array.
    let desc_array_bytes = MAX_MEMORY_DESCRIPTORS * core::mem::size_of::<RoverMemoryDescriptor>();
    let desc_pages = desc_array_bytes.div_ceil(PAGE_SIZE);
    let desc_array_phys = unsafe {
        allocate_pages(
            bs,
            AllocationType::ANY_PAGES,
            MemoryType::LOADER_DATA,
            desc_pages,
            0,
        )?
    };

    let normalized_descriptors = unsafe {
        core::slice::from_raw_parts_mut(
            desc_array_phys as *mut RoverMemoryDescriptor,
            MAX_MEMORY_DESCRIPTORS,
        )
    };

    // 16. Allocate page for master RoverBootInfo struct.
    let boot_info_phys =
        unsafe { allocate_pages(bs, AllocationType::ANY_PAGES, MemoryType::LOADER_DATA, 1, 0)? };

    // 17. Allocate memory map buffer (64 KiB) for exit_boot_services.
    let raw_map_pages = 16;
    let raw_map_phys = unsafe {
        allocate_pages(
            bs,
            AllocationType::ANY_PAGES,
            MemoryType::LOADER_DATA,
            raw_map_pages,
            0,
        )?
    };

    let raw_map_buffer = unsafe {
        core::slice::from_raw_parts_mut(raw_map_phys as *mut u8, raw_map_pages * PAGE_SIZE)
    };

    writer.set_color(14, 0); // Yellow
    writer.write_str("Exiting UEFI boot services and transferring to kernel...\n");

    // 18. Terminate UEFI Boot Services.
    let exit_res = unsafe { exit_boot_services_safe(image_handle, bs, raw_map_buffer)? };

    // From this point onward, NO UEFI BOOT SERVICES MAY BE CALLED!

    // 19. Parse and normalize memory map.
    let raw_map_slice = &raw_map_buffer[..exit_res.map_size];
    let parsed_count = parse_and_convert_memory_map(
        raw_map_slice,
        exit_res.descriptor_size,
        normalized_descriptors,
    )
    .unwrap_or(0);

    let final_descriptor_count = normalize_memory_map(normalized_descriptors, parsed_count);

    // 20. Populate RoverBootInfo.
    let mut flags = 0u32;
    if framebuffer.is_valid() {
        flags |= ROVER_FLAG_FRAMEBUFFER;
    }
    if acpi_ptr != 0 {
        flags |= ROVER_FLAG_ACPI;
    }
    if smbios_ptr != 0 {
        flags |= ROVER_FLAG_SMBIOS;
    }
    if initrd_module.size > 0 {
        flags |= ROVER_FLAG_INITRD;
    }
    if cmdline_desc.length > 0 {
        flags |= ROVER_FLAG_CMDLINE;
    }
    if secure_boot {
        flags |= ROVER_FLAG_SECURE_BOOT;
    }

    let mut bootloader_name = [0u8; 32];
    let name_src = b"Rover Bootloader 0.1.0\0";
    bootloader_name[..name_src.len()].copy_from_slice(name_src);

    let boot_info = RoverBootInfo {
        magic: ROVER_BOOT_MAGIC,
        version: ROVER_BOOT_PROTOCOL_VERSION,
        flags,
        bootloader_name,
        memory_map_addr: desc_array_phys,
        memory_map_entries: final_descriptor_count as u64,
        memory_map_entry_size: core::mem::size_of::<RoverMemoryDescriptor>() as u64,
        memory_map_entry_version: 1,
        pad0: 0,
        framebuffer,
        acpi_rsdp: acpi_ptr,
        smbios_entry: smbios_ptr,
        kernel_image: RoverKernelImage {
            physical_base: kernel_phys_base,
            virtual_base: min_addr,
            size: (total_pages * PAGE_SIZE) as u64,
            entry_point: kernel_entry_point,
        },
        initrd: initrd_module,
        cmdline: cmdline_desc,
        system_info: RoverSystemInfo::new(st.hdr.revision, current_arch(), secure_boot),
        reserved: [0u64; 8],
    };

    // Write boot info to allocated physical memory.
    unsafe {
        core::ptr::write(boot_info_phys as *mut RoverBootInfo, boot_info);
    }

    // 21. Execute architecture handoff!
    #[cfg(target_arch = "x86_64")]
    unsafe {
        crate::arch::x86_64::jump_to_kernel(
            kernel_entry_point,
            boot_info_phys as *const RoverBootInfo,
            stack_top,
        );
    }

    #[cfg(target_arch = "aarch64")]
    unsafe {
        crate::arch::aarch64::jump_to_kernel(
            kernel_entry_point,
            boot_info_phys as *const RoverBootInfo,
            stack_top,
        );
    }

    #[cfg(target_arch = "riscv64")]
    unsafe {
        crate::arch::riscv64::jump_to_kernel(
            kernel_entry_point,
            boot_info_phys as *const RoverBootInfo,
            stack_top,
        );
    }

    #[cfg(target_arch = "x86")]
    unsafe {
        crate::arch::ia32::jump_to_kernel(
            kernel_entry_point as u32,
            boot_info_phys as *const RoverBootInfo,
            stack_top as u32,
        );
    }

    #[cfg(not(any(
        target_arch = "x86_64",
        target_arch = "aarch64",
        target_arch = "riscv64",
        target_arch = "x86"
    )))]
    {
        Err(BootError::UnsupportedArchitecture)
    }
}
