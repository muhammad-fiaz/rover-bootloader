//! # Secure Boot detection
//!
//! Queries firmware variables to detect active UEFI Secure Boot enforcement.

use core::ffi::c_void;
use rover_uefi::abi::types::{Char16, UIntN};
use rover_uefi::base::EfiGuid;
use rover_uefi::tables::runtime::EfiRuntimeServices;

/// UEFI Global Variable Vendor GUID (`8BE4DF61-93CA-11D2-AA0D-00E098032B8C`).
pub const EFI_GLOBAL_VARIABLE_GUID: EfiGuid = EfiGuid::new(
    0x8BE4_DF61,
    0x93CA,
    0x11D2,
    [0xAA, 0x0D, 0x00, 0xE0, 0x98, 0x03, 0x2B, 0x8C],
);

/// UTF-16 encoded "SecureBoot\0" string.
const SECURE_BOOT_VAR_NAME: [Char16; 11] = [
    b'S' as u16,
    b'e' as u16,
    b'c' as u16,
    b'u' as u16,
    b'r' as u16,
    b'e' as u16,
    b'B' as u16,
    b'o' as u16,
    b'o' as u16,
    b't' as u16,
    0,
];

/// Detects whether UEFI Secure Boot is active on the system.
///
/// Returns `true` if `SecureBoot` variable is set to 1.
///
/// # Safety
///
/// If non-null, `rt` must point to an initialized [`EfiRuntimeServices`].
pub unsafe fn is_secure_boot_enabled(rt: *mut EfiRuntimeServices) -> bool {
    if rt.is_null() {
        return false;
    }

    let mut name = SECURE_BOOT_VAR_NAME;
    let mut vendor = EFI_GLOBAL_VARIABLE_GUID;
    let mut attr: u32 = 0;
    let mut val: u8 = 0;
    let mut data_size: UIntN = core::mem::size_of::<u8>();

    // SAFETY: rt is non-null, parameters point to valid stack locals.
    let status = unsafe {
        ((*rt).get_variable)(
            name.as_mut_ptr(),
            &raw mut vendor,
            &raw mut attr,
            &raw mut data_size,
            (&raw mut val).cast::<c_void>(),
        )
    };

    status.is_success() && val == 1
}
