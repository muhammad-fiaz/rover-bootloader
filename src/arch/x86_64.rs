//! # x86-64 architecture handoff
//!
//! Kernel entry state preparation and control transfer following System V AMD64 ABI.

use crate::protocol::info::RoverBootInfo;

/// Transfers execution control to the x86-64 kernel entry point.
///
/// Sets up registers per System V AMD64 ABI:
/// - Interrupts disabled (`cli`).
/// - Direction flag cleared (`cld`).
/// - `RDI` = pointer to [`RoverBootInfo`].
/// - `RSP` = `stack_top` (16-byte aligned).
/// - `RBP` = 0.
///
/// # Safety
///
/// `entry_point` must point to valid kernel executable code.
/// `stack_top` must be a valid, mapped 16-byte aligned stack top.
/// Boot services must already have been terminated.
#[cfg(target_arch = "x86_64")]
pub unsafe fn jump_to_kernel(
    entry_point: u64,
    boot_info: *const RoverBootInfo,
    stack_top: u64,
) -> ! {
    // SAFETY: Assembly instructions set machine state and jump to kernel entry point.
    unsafe {
        core::arch::asm!(
            "cli",
            "cld",
            "mov rsp, {stack}",
            "xor rbp, rbp",
            "jmp {entry}",
            stack = in(reg) stack_top,
            entry = in(reg) entry_point,
            in("rdi") boot_info,
            options(noreturn)
        );
    }
}

/// Fallback for compiling on non-x86_64 targets.
#[cfg(not(target_arch = "x86_64"))]
pub unsafe fn jump_to_kernel(
    _entry_point: u64,
    _boot_info: *const RoverBootInfo,
    _stack_top: u64,
) -> ! {
    panic!("cannot jump to x86_64 kernel from non-x86_64 build");
}
