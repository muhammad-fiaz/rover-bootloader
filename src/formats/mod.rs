//! # Executable image formats
//!
//! Native parsers and loaders for ELF and PE executable images.

pub mod elf;
pub mod pe;

pub use elf::{Elf64Header, Elf64ProgramHeader, ElfBinary};
pub use pe::PeBinary;
