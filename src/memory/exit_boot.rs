//! # UEFI exit boot services transition
//!
//! Safe handling of the firmware memory map and `ExitBootServices` loop with stale key recovery.

use crate::error::BootError;
use rover_uefi::abi::types::UIntN;
use rover_uefi::base::{EfiHandle, EfiMemoryDescriptor};
use rover_uefi::tables::boot::EfiBootServices;

/// Maximum number of attempts to exit boot services before failing.
pub const MAX_EXIT_ATTEMPTS: usize = 5;

/// Information returned after successfully terminating UEFI Boot Services.
#[derive(Clone, Copy, Debug)]
pub struct ExitBootServicesResult {
    /// Number of valid bytes in the memory map buffer.
    pub map_size: usize,
    /// Size in bytes of each descriptor entry.
    pub descriptor_size: usize,
    /// Descriptor format version.
    pub descriptor_version: u32,
}

/// Safely terminates UEFI Boot Services with retry on stale memory map key.
///
/// # Safety
///
/// After this function returns successfully, the caller must never invoke any UEFI Boot Services
/// function pointer.
///
/// # Errors
///
/// Returns [`BootError::ExitBootServicesFailed`] if the transition cannot be completed.
#[allow(clippy::cast_ptr_alignment)]
pub unsafe fn exit_boot_services_safe(
    image_handle: EfiHandle,
    bs: *mut EfiBootServices,
    buffer: &mut [u8],
) -> Result<ExitBootServicesResult, BootError> {
    if bs.is_null() {
        return Err(BootError::ExitBootServicesFailed);
    }

    for _ in 0..MAX_EXIT_ATTEMPTS {
        let mut map_size: UIntN = buffer.len();
        let mut map_key: UIntN = 0;
        let mut descriptor_size: UIntN = 0;
        let mut descriptor_version: u32 = 0;

        // SAFETY: bs is valid, buffer is caller-provided mutable slice, out params are stack locals.
        let status = unsafe {
            ((*bs).get_memory_map)(
                &raw mut map_size,
                buffer.as_mut_ptr().cast::<EfiMemoryDescriptor>(),
                &raw mut map_key,
                &raw mut descriptor_size,
                &raw mut descriptor_version,
            )
        };

        if !status.is_success() {
            continue;
        }

        // SAFETY: Must not make any memory-allocating or event call between get_memory_map and exit_boot_services.
        let exit_status = unsafe { ((*bs).exit_boot_services)(image_handle, map_key) };

        if exit_status.is_success() {
            return Ok(ExitBootServicesResult {
                map_size,
                descriptor_size,
                descriptor_version,
            });
        }
    }

    Err(BootError::ExitBootServicesFailed)
}
