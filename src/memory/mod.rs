//! # Memory subsystem
//!
//! Page allocation, UEFI memory map parsing, normalization, and safe boot services exit transitions.

pub mod allocator;
pub mod exit_boot;
pub mod map;

pub use allocator::{allocate_pages, allocate_pool, free_pages, free_pool};
pub use exit_boot::{ExitBootServicesResult, exit_boot_services_safe};
pub use map::{convert_memory_type, normalize_memory_map, parse_and_convert_memory_map};
