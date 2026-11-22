//! # Architecture handoff for 64-bit ARM (`AArch64`)
//!
//! Kernel entry state preparation and control transfer on 64-bit ARM.

use crate::protocol::info::RoverBootInfo;

/// Transfers execution control to the AArch64 kernel entry point.
///
/// Sets up registers:
/// - Mask all DAIF interrupts.
/// - `X0` = pointer to [`RoverBootInfo`].
/// - `X1..X3` = 0.
/// - `SP` = `stack_top` (16-byte aligned).
/// - Branches to entry point.
///
/// # Safety
///
/// `entry_point` must point to valid AArch64 kernel code.
/// `stack_top` must be a valid 16-byte aligned stack top.
#[cfg(target_arch = "aarch64")]
pub unsafe fn jump_to_kernel(
    entry_point: u64,
    boot_info: *const RoverBootInfo,
    stack_top: u64,
) -> ! {
    // SAFETY: Machine register setup and branch.
    unsafe {
        core::arch::asm!(
            "msr daifset, #0xf",
            "mov sp, {stack}",
            "mov x0, {info}",
            "mov x1, #0",
            "mov x2, #0",
            "mov x3, #0",
            "br {entry}",
            stack = in(reg) stack_top,
            info = in(reg) boot_info,
            entry = in(reg) entry_point,
            options(noreturn)
        );
    }
}

/// Fallback for non-aarch64 targets.
///
/// # Safety
///
/// Must not be called on non-aarch64 builds.
///
/// # Panics
///
/// Always panics when called on non-aarch64 builds.
#[cfg(not(target_arch = "aarch64"))]
pub unsafe fn jump_to_kernel(
    _entry_point: u64,
    _boot_info: *const RoverBootInfo,
    _stack_top: u64,
) -> ! {
    panic!("cannot jump to aarch64 kernel from non-aarch64 build");
}
