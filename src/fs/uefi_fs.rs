//! # UEFI file system operations
//!
//! File discovery and reading abstractions wrapping [`rover_uefi::protocols::storage`].

use core::ffi::c_void;

use super::path::{MAX_PATH_LENGTH, encode_uefi_path};
use crate::error::BootError;
use rover_uefi::abi::types::{Char16, UIntN};
use rover_uefi::protocols::storage::filesystem::file::{
    EfiFileProtocol, FILE_MODE_READ, FILE_READ_ONLY,
};
use rover_uefi::protocols::storage::filesystem::metadata::EfiFileInfo;
use rover_uefi::protocols::storage::guid::FILE_INFO_ID;

#[repr(align(8))]
struct AlignedBuffer([u8; 256]);

/// Opens a file relative to a root directory handle.
///
/// # Safety
///
/// `root` must be a valid, non-null pointer to an initialized [`EfiFileProtocol`].
///
/// # Errors
///
/// Returns [`BootError`] if path encoding fails or the firmware returns an error.
pub unsafe fn open_file(
    root: *mut EfiFileProtocol,
    path: &str,
) -> Result<*mut EfiFileProtocol, BootError> {
    if root.is_null() {
        return Err(BootError::FileSystemNotFound);
    }

    let mut utf16_path: [Char16; MAX_PATH_LENGTH] = [0; MAX_PATH_LENGTH];
    encode_uefi_path(path, &mut utf16_path)?;

    let mut file_handle: *mut EfiFileProtocol = core::ptr::null_mut();
    // SAFETY: root is non-null, file_handle is a valid stack local, utf16_path is null-terminated.
    let status = unsafe {
        ((*root).open)(
            root,
            &raw mut file_handle,
            utf16_path.as_mut_ptr(),
            FILE_MODE_READ,
            FILE_READ_ONLY,
        )
    };

    if !status.is_success() || file_handle.is_null() {
        return Err(BootError::FileNotFound);
    }

    Ok(file_handle)
}

/// Queries the size in bytes of an open file using `EFI_FILE_INFO`.
///
/// # Safety
///
/// `file` must be a valid, non-null pointer to an initialized [`EfiFileProtocol`].
///
/// # Errors
///
/// Returns [`BootError`] if the file info could not be retrieved.
#[allow(clippy::cast_ptr_alignment)]
pub unsafe fn query_file_size(file: *mut EfiFileProtocol) -> Result<u64, BootError> {
    if file.is_null() {
        return Err(BootError::FileNotFound);
    }

    // Allocate an 8-byte aligned stack buffer for EFI_FILE_INFO (at least 256 bytes for name).
    let mut info_buf = AlignedBuffer([0u8; 256]);
    let mut info_size: UIntN = info_buf.0.len();
    let mut guid = FILE_INFO_ID;

    // SAFETY: file is non-null, out parameters point to valid stack locals.
    let status = unsafe {
        ((*file).get_info)(
            file,
            &raw mut guid,
            &raw mut info_size,
            info_buf.0.as_mut_ptr().cast::<c_void>(),
        )
    };

    if !status.is_success() {
        return Err(BootError::FileReadError);
    }

    let file_info = unsafe { &*(info_buf.0.as_ptr().cast::<EfiFileInfo>()) };
    Ok(file_info.file_size)
}

/// Reads the entire contents or up to `buffer.len()` bytes from an open file.
///
/// # Safety
///
/// `file` must be a valid, non-null pointer to an initialized [`EfiFileProtocol`].
///
/// # Errors
///
/// Returns [`BootError`] if the read operation fails.
pub unsafe fn read_file(file: *mut EfiFileProtocol, buffer: &mut [u8]) -> Result<usize, BootError> {
    if file.is_null() {
        return Err(BootError::FileNotFound);
    }

    let mut read_size: UIntN = buffer.len();
    // SAFETY: file is non-null, buffer pointer and read_size are valid for the call.
    let status = unsafe {
        ((*file).read)(
            file,
            &raw mut read_size,
            buffer.as_mut_ptr().cast::<c_void>(),
        )
    };

    if !status.is_success() {
        return Err(BootError::FileReadError);
    }

    Ok(read_size)
}

/// Closes an open file handle.
///
/// # Safety
///
/// If non-null, `file` must point to an initialized [`EfiFileProtocol`].
pub unsafe fn close_file(file: *mut EfiFileProtocol) {
    if !file.is_null() {
        // SAFETY: file is checked non-null.
        let _ = unsafe { ((*file).close)(file) };
    }
}

/// Convenience helper to open, read, and close a file in a single operation.
///
/// # Safety
///
/// `root` must be a valid, non-null pointer to an initialized [`EfiFileProtocol`].
///
/// # Errors
///
/// Returns [`BootError`] if the file cannot be found or read.
pub unsafe fn load_file_into_buffer(
    root: *mut EfiFileProtocol,
    path: &str,
    buffer: &mut [u8],
) -> Result<usize, BootError> {
    let file = unsafe { open_file(root, path)? };
    let result = unsafe { read_file(file, buffer) };
    unsafe { close_file(file) };
    result
}
