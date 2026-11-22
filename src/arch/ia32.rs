//! # IA-32 (x86 32-bit) architecture handoff
//!
//! Kernel entry state preparation and control transfer on 32-bit x86.

use crate::protocol::info::RoverBootInfo;

/// Transfers execution control to the IA-32 kernel entry point.
///
/// Sets up registers:
/// - Clears interrupts (`cli`).
/// - Clears direction flag (`cld`).
/// - Pushes pointer to [`RoverBootInfo`] and dummy return address to stack.
/// - Jumps to entry point.
///
/// # Safety
///
/// `entry_point` must point to valid 32-bit kernel code.
/// `stack_top` must be a valid 16-byte aligned stack top.
#[cfg(target_arch = "x86")]
pub unsafe fn jump_to_kernel(
    entry_point: u32,
    boot_info: *const RoverBootInfo,
    stack_top: u32,
) -> ! {
    // SAFETY: Machine state and jump.
    unsafe {
        core::arch::asm!(
            "cli",
            "cld",
            "mov esp, {stack}",
            "push {info}",
            "push 0",
            "jmp {entry}",
            stack = in(reg) stack_top,
            info = in(reg) boot_info,
            entry = in(reg) entry_point,
            options(noreturn)
        );
    }
}

/// Fallback for non-x86 targets.
///
/// # Safety
///
/// Must not be called on non-x86 builds.
///
/// # Panics
///
/// Always panics when called on non-x86 builds.
#[cfg(not(target_arch = "x86"))]
pub unsafe fn jump_to_kernel(
    _entry_point: u32,
    _boot_info: *const RoverBootInfo,
    _stack_top: u32,
) -> ! {
    panic!("cannot jump to IA-32 kernel from non-x86 build");
}
