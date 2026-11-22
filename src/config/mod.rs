//! # Boot configuration subsystem
//!
//! Robust parser and representations for bootloader configuration files (`rover.cfg`).

pub mod entry;
pub mod parser;

pub use entry::BootEntry;
pub use parser::{BootConfig, MAX_BOOT_ENTRIES};
