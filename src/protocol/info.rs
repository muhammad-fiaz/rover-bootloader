//! # Rover boot information structure
//!
//! The authoritative root information structure passed to the target kernel.

use super::framebuffer::RoverFramebuffer;
use super::memory::RoverMemoryDescriptor;
use super::module::{RoverCommandLine, RoverKernelImage, RoverModule, RoverSystemInfo};

/// The Rover Boot Protocol magic byte sequence ("`ROVER_BOOT_V1\0\0\0`").
pub const ROVER_BOOT_MAGIC: [u8; 16] = *b"ROVER_BOOT_V1\0\0\0";

/// Current Rover Boot Protocol version.
pub const ROVER_BOOT_PROTOCOL_VERSION: u32 = 1;

/// Flag indicating that a valid linear framebuffer is present.
pub const ROVER_FLAG_FRAMEBUFFER: u32 = 1 << 0;

/// Flag indicating that an ACPI RSDP table pointer is present.
pub const ROVER_FLAG_ACPI: u32 = 1 << 1;

/// Flag indicating that an SMBIOS entry point pointer is present.
pub const ROVER_FLAG_SMBIOS: u32 = 1 << 2;

/// Flag indicating that an initial ramdisk (initrd) was loaded.
pub const ROVER_FLAG_INITRD: u32 = 1 << 3;

/// Flag indicating that a kernel command-line string is present.
pub const ROVER_FLAG_CMDLINE: u32 = 1 << 4;

/// Flag indicating that Secure Boot is enabled.
pub const ROVER_FLAG_SECURE_BOOT: u32 = 1 << 5;

/// Master boot information block passed from the bootloader to the target kernel.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct RoverBootInfo {
    /// Protocol magic bytes matching [`ROVER_BOOT_MAGIC`].
    pub magic: [u8; 16],
    /// Protocol specification version matching [`ROVER_BOOT_PROTOCOL_VERSION`].
    pub version: u32,
    /// Bitmask flags indicating which fields are populated.
    pub flags: u32,
    /// Human-readable bootloader name and version.
    pub bootloader_name: [u8; 32],
    /// Physical address of the array of [`RoverMemoryDescriptor`] entries.
    pub memory_map_addr: u64,
    /// Number of memory descriptors in the memory map array.
    pub memory_map_entries: u64,
    /// Size in bytes of each descriptor in the memory map.
    pub memory_map_entry_size: u64,
    /// Memory descriptor format version.
    pub memory_map_entry_version: u32,
    /// Padding for 64-bit alignment.
    pub pad0: u32,
    /// Framebuffer configuration and aperture mapping.
    pub framebuffer: RoverFramebuffer,
    /// Physical address of the ACPI Root System Description Pointer (RSDP), or 0.
    pub acpi_rsdp: u64,
    /// Physical address of the SMBIOS entry point table, or 0.
    pub smbios_entry: u64,
    /// Loaded kernel executable image geometry.
    pub kernel_image: RoverKernelImage,
    /// Initial ramdisk module, if loaded.
    pub initrd: RoverModule,
    /// Kernel command-line arguments.
    pub cmdline: RoverCommandLine,
    /// Firmware and platform identity details.
    pub system_info: RoverSystemInfo,
    /// Reserved space for forward-compatible protocol extensions.
    pub reserved: [u64; 8],
}

impl RoverBootInfo {
    /// Validates the structure magic and version.
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        let mut i = 0;
        while i < 16 {
            if self.magic[i] != ROVER_BOOT_MAGIC[i] {
                return false;
            }
            i += 1;
        }
        self.version == ROVER_BOOT_PROTOCOL_VERSION
    }

    /// Returns a slice of the memory descriptors if the pointer and count are valid.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `memory_map_addr` points to valid physical/virtual memory
    /// containing at least `memory_map_entries` descriptors.
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub unsafe fn memory_map(&self) -> &[RoverMemoryDescriptor] {
        if self.memory_map_addr == 0 || self.memory_map_entries == 0 {
            return &[];
        }
        // SAFETY: Ensured by caller.
        unsafe {
            core::slice::from_raw_parts(
                self.memory_map_addr as *const RoverMemoryDescriptor,
                self.memory_map_entries as usize,
            )
        }
    }
}
