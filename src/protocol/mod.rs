//! # Rover Boot Protocol
//!
//! Versioned specification and structures for communication between Rover Bootloader
//! and booted kernel targets.

pub mod framebuffer;
pub mod info;
pub mod memory;
pub mod module;

pub use framebuffer::{RoverFramebuffer, RoverPixelFormat};
pub use info::{
    ROVER_BOOT_MAGIC, ROVER_BOOT_PROTOCOL_VERSION, ROVER_FLAG_ACPI, ROVER_FLAG_CMDLINE,
    ROVER_FLAG_FRAMEBUFFER, ROVER_FLAG_INITRD, ROVER_FLAG_SECURE_BOOT, ROVER_FLAG_SMBIOS,
    RoverBootInfo,
};
pub use memory::{RoverMemoryDescriptor, RoverMemoryType};
pub use module::{RoverArchId, RoverCommandLine, RoverKernelImage, RoverModule, RoverSystemInfo};
