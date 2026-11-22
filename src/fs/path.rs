//! # Path manipulation
//!
//! UEFI path normalization and UTF-16 encoding helpers.

use crate::error::BootError;
use rover_uefi::abi::types::Char16;

/// Maximum path length supported in UEFI file paths.
pub const MAX_PATH_LENGTH: usize = 260;

/// Encodes an ASCII/UTF-8 path into a null-terminated UEFI UTF-16 buffer.
///
/// Forward slashes (`/`) are automatically converted to backslashes (`\`).
///
/// # Errors
///
/// Returns [`BootError::FileNotFound`] if the path exceeds [`MAX_PATH_LENGTH`] code units.
pub fn encode_uefi_path(
    path: &str,
    output: &mut [Char16; MAX_PATH_LENGTH],
) -> Result<usize, BootError> {
    let mut out_idx = 0;

    for ch in path.chars() {
        if out_idx >= MAX_PATH_LENGTH - 1 {
            return Err(BootError::FileNotFound);
        }

        let normalized = if ch == '/' { '\\' } else { ch };
        let mut utf16_buf = [0u16; 2];
        let encoded = normalized.encode_utf16(&mut utf16_buf);

        for unit in encoded.iter().copied() {
            if out_idx >= MAX_PATH_LENGTH - 1 {
                return Err(BootError::FileNotFound);
            }
            output[out_idx] = unit;
            out_idx += 1;
        }
    }

    // Append null terminator.
    output[out_idx] = 0;
    Ok(out_idx)
}
