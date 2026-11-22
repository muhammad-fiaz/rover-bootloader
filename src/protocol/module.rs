//! # Rover boot protocol modules and system descriptors
//!
//! Descriptors for kernel image placement, ramdisk modules, command lines, and system identity.

/// Processor architecture identifier in the boot protocol.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum RoverArchId {
    /// Unknown or generic architecture.
    Unknown = 0,
    /// x86-64 (AMD64 / Intel 64).
    X86_64 = 1,
    /// 64-bit ARM (`AArch64`).
    Aarch64 = 2,
    /// 64-bit RISC-V (RV64GC).
    Riscv64 = 3,
    /// 32-bit x86 (IA-32).
    Ia32 = 4,
    /// 32-bit ARM.
    Arm32 = 5,
    /// 64-bit `LoongArch` (LA64).
    LoongArch64 = 6,
}

/// Information describing the loaded kernel executable image.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct RoverKernelImage {
    /// Physical base address where kernel code and data were placed.
    pub physical_base: u64,
    /// Virtual base address as specified by the ELF headers.
    pub virtual_base: u64,
    /// Total memory size in bytes allocated for all segments.
    pub size: u64,
    /// Kernel entry point address to which control is transferred.
    pub entry_point: u64,
}

impl RoverKernelImage {
    /// Returns an empty kernel image descriptor.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            physical_base: 0,
            virtual_base: 0,
            size: 0,
            entry_point: 0,
        }
    }
}

/// A loaded boot module, such as an initial ramdisk (initrd).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct RoverModule {
    /// Physical start address of the loaded module.
    pub physical_start: u64,
    /// Size of the module in bytes.
    pub size: u64,
    /// UTF-8 null-terminated module identifier or mount path.
    pub name: [u8; 64],
}

impl RoverModule {
    /// Creates an empty module descriptor.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            physical_start: 0,
            size: 0,
            name: [0u8; 64],
        }
    }
}

/// Kernel command line pointer and length.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct RoverCommandLine {
    /// Physical address of the UTF-8 null-terminated command line buffer.
    pub ptr: u64,
    /// Length of the command line string in bytes (excluding null terminator).
    pub length: u64,
}

impl RoverCommandLine {
    /// Returns an empty command line.
    #[must_use]
    pub const fn empty() -> Self {
        Self { ptr: 0, length: 0 }
    }
}

/// Firmware and platform identity information passed to the kernel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct RoverSystemInfo {
    /// Firmware environment type (1 = UEFI).
    pub firmware_type: u32,
    /// Firmware revision reported by the system table.
    pub firmware_revision: u32,
    /// Processor architecture of the bootloader binary.
    pub arch: RoverArchId,
    /// Whether Secure Boot is enabled on the platform (1 = enabled, 0 = disabled).
    pub secure_boot: u8,
    /// Reserved for alignment and future flags.
    pub reserved: [u8; 3],
}

impl RoverSystemInfo {
    /// Creates a system info descriptor.
    #[must_use]
    pub const fn new(firmware_revision: u32, arch: RoverArchId, secure_boot: bool) -> Self {
        Self {
            firmware_type: 1, // UEFI
            firmware_revision,
            arch,
            secure_boot: if secure_boot { 1 } else { 0 },
            reserved: [0u8; 3],
        }
    }
}
