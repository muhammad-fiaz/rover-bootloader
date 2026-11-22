//! # Rover boot protocol memory definitions
//!
//! Standard memory descriptors and types reported to the target kernel.

use rover_uefi::base::MemoryType;

/// Classification of physical memory regions passed to the kernel.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum RoverMemoryType {
    /// Available system RAM suitable for general kernel use.
    Usable = 1,
    /// Reserved memory region that must not be used by the kernel.
    Reserved = 2,
    /// ACPI reclaimable memory containing tables that may be reused after parsing.
    AcpiReclaimable = 3,
    /// ACPI Non-Volatile Storage (NVS) memory required across sleep/wake transitions.
    AcpiNvs = 4,
    /// Defective hardware memory regions.
    BadMemory = 5,
    /// Memory used by the bootloader that can be reclaimed after boot initialization.
    BootloaderReclaimable = 6,
    /// Memory containing loaded kernel image code, data, and boot modules.
    KernelAndModules = 7,
    /// Memory mapped for the graphics framebuffer.
    Framebuffer = 8,
}

impl From<MemoryType> for RoverMemoryType {
    fn from(mem_type: MemoryType) -> Self {
        if mem_type == MemoryType::LOADER_CODE || mem_type == MemoryType::LOADER_DATA {
            Self::BootloaderReclaimable
        } else if mem_type == MemoryType::EXIT_BOOT_SERVICES_CODE
            || mem_type == MemoryType::EXIT_BOOT_SERVICES_DATA
            || mem_type.to_raw() == 7
        {
            Self::Usable
        } else if mem_type == MemoryType::ACPI_RECLAIM_MEMORY {
            Self::AcpiReclaimable
        } else if mem_type == MemoryType::ACPI_MEMORY_MAP {
            Self::AcpiNvs
        } else if mem_type == MemoryType::UNUSABLE_MEMORY {
            Self::BadMemory
        } else {
            Self::Reserved
        }
    }
}

/// A physical memory region descriptor in the Rover Boot Protocol.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct RoverMemoryDescriptor {
    /// The memory region type.
    pub region_type: RoverMemoryType,
    /// Padding for 64-bit alignment.
    pub pad: u32,
    /// The physical starting address of the memory region.
    pub physical_start: u64,
    /// The virtual starting address (if mapped), or zero.
    pub virtual_start: u64,
    /// The number of 4 KiB pages in the region.
    pub number_of_pages: u64,
    /// Memory cacheability and access attributes.
    pub attribute: u64,
}

impl RoverMemoryDescriptor {
    /// Creates a new memory descriptor.
    #[must_use]
    pub const fn new(
        region_type: RoverMemoryType,
        physical_start: u64,
        virtual_start: u64,
        number_of_pages: u64,
        attribute: u64,
    ) -> Self {
        Self {
            region_type,
            pad: 0,
            physical_start,
            virtual_start,
            number_of_pages,
            attribute,
        }
    }

    /// Returns the length in bytes of the region.
    #[must_use]
    pub const fn byte_length(&self) -> u64 {
        self.number_of_pages.saturating_mul(4096)
    }

    /// Returns the physical end address (exclusive).
    #[must_use]
    pub const fn physical_end(&self) -> u64 {
        self.physical_start.saturating_add(self.byte_length())
    }
}
