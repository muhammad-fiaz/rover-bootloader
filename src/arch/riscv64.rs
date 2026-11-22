//! # RISC-V 64 architecture handoff
//!
//! Kernel entry state preparation and control transfer on 64-bit RISC-V.

use crate::protocol::info::RoverBootInfo;

/// Transfers execution control to the RISC-V 64 kernel entry point.
///
/// Sets up registers:
/// - Disables supervisor interrupts (`sstatus.SIE = 0`).
/// - `A0` = 0 (boot hart ID or legacy indicator).
/// - `A1` = pointer to [`RoverBootInfo`].
/// - `SP` = `stack_top` (16-byte aligned).
/// - Jumps to entry point.
///
/// # Safety
///
/// `entry_point` must point to valid RISC-V 64 kernel code.
/// `stack_top` must be a valid 16-byte aligned stack top.
#[cfg(target_arch = "riscv64")]
pub unsafe fn jump_to_kernel(
    entry_point: u64,
    boot_info: *const RoverBootInfo,
    stack_top: u64,
) -> ! {
    // SAFETY: Assembly instructions set machine state and jump to kernel entry point.
    unsafe {
        core::arch::asm!(
            "csrci sstatus, 0x2",
            "mv sp, {stack}",
            "mv a0, zero",
            "mv a1, {info}",
            "jr {entry}",
            stack = in(reg) stack_top,
            info = in(reg) boot_info,
            entry = in(reg) entry_point,
            options(noreturn)
        );
    }
}

/// Fallback for non-riscv64 targets.
///
/// # Safety
///
/// Must not be called on non-riscv64 builds.
///
/// # Panics
///
/// Always panics when called on non-riscv64 builds.
#[cfg(not(target_arch = "riscv64"))]
pub unsafe fn jump_to_kernel(
    _entry_point: u64,
    _boot_info: *const RoverBootInfo,
    _stack_top: u64,
) -> ! {
    panic!("cannot jump to riscv64 kernel from non-riscv64 build");
}
