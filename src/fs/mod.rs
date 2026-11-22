//! # Filesystem subsystem
//!
//! Path normalization and UEFI file I/O operations for discovering and reading boot assets.

pub mod path;
pub mod uefi_fs;

pub use path::encode_uefi_path;
pub use uefi_fs::{close_file, load_file_into_buffer, open_file, query_file_size, read_file};
