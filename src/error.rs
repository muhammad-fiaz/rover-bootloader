//! # Error definitions
//!
//! Strongly-typed error enum and conversions for Rover Bootloader.

use core::fmt;
use rover_uefi::base::EfiStatus;

/// All errors that can occur during the boot process.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BootError {
    /// An underlying UEFI firmware call returned an error status.
    Uefi(EfiStatus),
    /// The target CPU architecture is unsupported by the bootloader.
    UnsupportedArchitecture,
    /// The boot device could not be located.
    DeviceNotFound,
    /// The file system on the boot device could not be opened.
    FileSystemNotFound,
    /// A requested file (kernel, initrd, config) was not found.
    FileNotFound,
    /// An error occurred while reading from a file.
    FileReadError,
    /// The kernel image is smaller than an ELF header.
    ImageTruncated,
    /// The ELF magic bytes (`\x7fELF`) are invalid.
    InvalidElfMagic,
    /// The ELF class is invalid or unsupported (e.g. not 64-bit when required).
    InvalidElfClass,
    /// The ELF data encoding is invalid (e.g. not little-endian).
    InvalidElfData,
    /// The ELF version is not current (`EV_CURRENT`).
    InvalidElfVersion,
    /// The ELF type is unsupported (must be `ET_EXEC` or `ET_DYN`).
    InvalidElfType,
    /// The ELF target machine does not match the active processor architecture.
    InvalidElfMachine,
    /// The program header offset or count is invalid.
    InvalidProgramHeader,
    /// A program segment's data exceeds the binary image bounds.
    SegmentOutOfBounds,
    /// Program headers contain overlapping or conflicting segments.
    OverlappingSegments,
    /// Physical memory allocation for a program segment failed.
    MemoryAllocationFailed,
    /// The ELF entry point is outside any loaded executable segment.
    InvalidEntryPoint,
    /// Boot configuration syntax error.
    ConfigSyntaxError,
    /// Boot configuration contains no boot entries or is missing a kernel path.
    ConfigMissingKernel,
    /// The memory map buffer was too small for firmware descriptors.
    MemoryMapBufferTooSmall,
    /// Termination of UEFI boot services failed after all retry attempts.
    ExitBootServicesFailed,
    /// Cryptographic checksum or signature verification of the image failed.
    SecurityVerificationFailed,
    /// No bootable kernel entry could be located.
    NoBootableEntry,
    /// The requested graphics mode is not supported by firmware.
    GraphicsModeUnsupported,
    /// The kernel stack allocation failed.
    StackAllocationFailed,
}

impl From<EfiStatus> for BootError {
    #[inline]
    fn from(status: EfiStatus) -> Self {
        Self::Uefi(status)
    }
}

impl fmt::Display for BootError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Uefi(status) => write!(f, "UEFI error: {status:?}"),
            Self::UnsupportedArchitecture => write!(f, "unsupported CPU architecture"),
            Self::DeviceNotFound => write!(f, "boot device not found"),
            Self::FileSystemNotFound => write!(f, "file system not found"),
            Self::FileNotFound => write!(f, "file not found"),
            Self::FileReadError => write!(f, "error reading file"),
            Self::ImageTruncated => write!(f, "image truncated"),
            Self::InvalidElfMagic => write!(f, "invalid ELF magic"),
            Self::InvalidElfClass => write!(f, "unsupported ELF class"),
            Self::InvalidElfData => write!(f, "unsupported ELF data encoding"),
            Self::InvalidElfVersion => write!(f, "unsupported ELF version"),
            Self::InvalidElfType => write!(f, "unsupported ELF type"),
            Self::InvalidElfMachine => write!(f, "ELF target machine architecture mismatch"),
            Self::InvalidProgramHeader => write!(f, "invalid ELF program header"),
            Self::SegmentOutOfBounds => write!(f, "ELF segment offset exceeds image bounds"),
            Self::OverlappingSegments => write!(f, "ELF segments overlap"),
            Self::MemoryAllocationFailed => write!(f, "memory allocation failed"),
            Self::InvalidEntryPoint => write!(f, "ELF entry point invalid"),
            Self::ConfigSyntaxError => write!(f, "boot configuration syntax error"),
            Self::ConfigMissingKernel => write!(f, "boot configuration missing kernel path"),
            Self::MemoryMapBufferTooSmall => write!(f, "memory map buffer too small"),
            Self::ExitBootServicesFailed => write!(f, "failed to exit UEFI boot services"),
            Self::SecurityVerificationFailed => {
                write!(f, "kernel security integrity verification failed")
            }
            Self::NoBootableEntry => write!(f, "no bootable entry found"),
            Self::GraphicsModeUnsupported => write!(f, "requested graphics mode unsupported"),
            Self::StackAllocationFailed => write!(f, "kernel stack allocation failed"),
        }
    }
}
