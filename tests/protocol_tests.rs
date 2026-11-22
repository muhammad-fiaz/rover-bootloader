//! # Rover boot protocol tests
//!
//! Validates memory descriptor processing, coalescing, structure layout, and protocol verification.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unreadable_literal,
    clippy::needless_raw_string_hashes,
    clippy::items_after_statements,
    clippy::needless_range_loop,
    clippy::cast_possible_truncation
)]

use rover_bootloader::memory::map::{normalize_memory_map, parse_and_convert_memory_map};
use rover_bootloader::protocol::framebuffer::{RoverFramebuffer, RoverPixelFormat};
use rover_bootloader::protocol::info::{
    ROVER_BOOT_MAGIC, ROVER_BOOT_PROTOCOL_VERSION, ROVER_FLAG_FRAMEBUFFER, RoverBootInfo,
};
use rover_bootloader::protocol::memory::{RoverMemoryDescriptor, RoverMemoryType};
use rover_bootloader::protocol::module::{
    RoverArchId, RoverCommandLine, RoverKernelImage, RoverModule, RoverSystemInfo,
};
use rover_uefi::base::{EfiMemoryDescriptor, EfiPhysicalAddress, EfiVirtualAddress, MemoryType};

#[test]
fn test_boot_info_magic_and_validation() {
    let mut info = RoverBootInfo {
        magic: ROVER_BOOT_MAGIC,
        version: ROVER_BOOT_PROTOCOL_VERSION,
        flags: ROVER_FLAG_FRAMEBUFFER,
        bootloader_name: [0u8; 32],
        memory_map_addr: 0,
        memory_map_entries: 0,
        memory_map_entry_size: core::mem::size_of::<RoverMemoryDescriptor>() as u64,
        memory_map_entry_version: 1,
        pad0: 0,
        framebuffer: RoverFramebuffer::empty(),
        acpi_rsdp: 0,
        smbios_entry: 0,
        kernel_image: RoverKernelImage::empty(),
        initrd: RoverModule::empty(),
        cmdline: RoverCommandLine::empty(),
        system_info: RoverSystemInfo::new(0, RoverArchId::X86_64, false),
        reserved: [0u64; 8],
    };

    assert!(info.is_valid());

    // Corrupt magic.
    info.magic[0] = 0;
    assert!(!info.is_valid());
}

#[test]
fn test_memory_descriptor_calculations() {
    let desc = RoverMemoryDescriptor::new(RoverMemoryType::Usable, 0x100000, 0, 16, 0);

    assert_eq!(desc.byte_length(), 16 * 4096);
    assert_eq!(desc.physical_end(), 0x100000 + 16 * 4096);
}

#[test]
fn test_memory_map_coalescing() {
    let mut descriptors = [
        // Region 2 (out of order)
        RoverMemoryDescriptor::new(RoverMemoryType::Usable, 0x2000, 0, 2, 0xF),
        // Region 1
        RoverMemoryDescriptor::new(RoverMemoryType::Usable, 0x0000, 0, 2, 0xF),
        // Region 3 (different type, should not merge)
        RoverMemoryDescriptor::new(RoverMemoryType::Reserved, 0x4000, 0, 1, 0xF),
    ];

    let count = normalize_memory_map(&mut descriptors, 3);

    // Regions 1 (0x0000, 2 pages = 0x2000 bytes) and 2 (0x2000, 2 pages) should coalesce into 1 descriptor with 4 pages.
    assert_eq!(count, 2);
    assert_eq!(descriptors[0].physical_start, 0x0000);
    assert_eq!(descriptors[0].number_of_pages, 4);
    assert_eq!(descriptors[0].region_type, RoverMemoryType::Usable);

    assert_eq!(descriptors[1].physical_start, 0x4000);
    assert_eq!(descriptors[1].number_of_pages, 1);
    assert_eq!(descriptors[1].region_type, RoverMemoryType::Reserved);
}

#[test]
fn test_parse_and_convert_memory_map() {
    let mut raw_buf = vec![0u8; core::mem::size_of::<EfiMemoryDescriptor>() * 2];

    let desc1 = EfiMemoryDescriptor {
        memory_type: MemoryType::LOADER_DATA,
        physical_start: EfiPhysicalAddress::from_u64(0x1000),
        virtual_start: EfiVirtualAddress::from_u64(0),
        number_of_pages: 4,
        attribute: 0x8,
    };

    let desc2 = EfiMemoryDescriptor {
        memory_type: MemoryType::ACPI_RECLAIM_MEMORY,
        physical_start: EfiPhysicalAddress::from_u64(0x5000),
        virtual_start: EfiVirtualAddress::from_u64(0),
        number_of_pages: 1,
        attribute: 0x8,
    };

    let desc_size = core::mem::size_of::<EfiMemoryDescriptor>();
    unsafe {
        core::ptr::write_unaligned(raw_buf.as_mut_ptr().cast::<EfiMemoryDescriptor>(), desc1);
        core::ptr::write_unaligned(
            raw_buf
                .as_mut_ptr()
                .add(desc_size)
                .cast::<EfiMemoryDescriptor>(),
            desc2,
        );
    }

    let mut dest = [RoverMemoryDescriptor::new(RoverMemoryType::Reserved, 0, 0, 0, 0); 4];
    let count = parse_and_convert_memory_map(&raw_buf, desc_size, &mut dest).unwrap();

    assert_eq!(count, 2);
    assert_eq!(dest[0].region_type, RoverMemoryType::BootloaderReclaimable);
    assert_eq!(dest[0].physical_start, 0x1000);
    assert_eq!(dest[0].number_of_pages, 4);

    assert_eq!(dest[1].region_type, RoverMemoryType::AcpiReclaimable);
    assert_eq!(dest[1].physical_start, 0x5000);
}

#[test]
fn test_framebuffer_validity() {
    let empty = RoverFramebuffer::empty();
    assert!(!empty.is_valid());

    let valid = RoverFramebuffer {
        base: 0xE000_0000,
        size: 1920 * 1080 * 4,
        width: 1920,
        height: 1080,
        stride: 1920,
        format: RoverPixelFormat::Rgb,
        red_mask: 0,
        green_mask: 0,
        blue_mask: 0,
        reserved_mask: 0,
    };
    assert!(valid.is_valid());
}
