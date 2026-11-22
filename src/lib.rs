//! # Rover Bootloader
//!
//! An independent, modern, modular, high-assurance UEFI bootloader
//! implemented entirely in native `#![no_std]` Rust.
//!
//! The bootloader provides complete UEFI platform initialization, native FAT/ESP
//! filesystem traversal, robust configuration parsing, zero-allocation ELF64 kernel
//! executable validation and loading, memory map acquisition and normalization,
//! safe `ExitBootServices` transitions, and architecture-specific handoff to
//! target kernels.

#![cfg_attr(not(test), no_std)]
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]

pub mod arch;
pub mod boot;
pub mod config;
pub mod error;
pub mod formats;
pub mod fs;
pub mod graphics;
pub mod memory;
pub mod protocol;
pub mod security;

pub use boot::execute_boot_pipeline;
pub use config::{BootConfig, BootEntry};
pub use error::BootError;
pub use formats::{Elf64Header, Elf64ProgramHeader, ElfBinary, PeBinary};
pub use graphics::ConsoleWriter;
pub use protocol::{
    RoverArchId, RoverBootInfo, RoverCommandLine, RoverFramebuffer, RoverKernelImage,
    RoverMemoryDescriptor, RoverMemoryType, RoverModule, RoverPixelFormat, RoverSystemInfo,
};
