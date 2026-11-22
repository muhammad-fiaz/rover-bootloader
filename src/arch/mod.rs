//! # Architecture-specific subsystems
//!
//! CPU architecture detection, calling conventions, and kernel entry handoff thunks.

pub mod aarch64;
pub mod ia32;
pub mod riscv64;
pub mod x86_64;

use crate::protocol::module::RoverArchId;

/// Detects the current compile-time target architecture.
#[must_use]
pub const fn current_arch() -> RoverArchId {
    #[cfg(target_arch = "x86_64")]
    {
        RoverArchId::X86_64
    }
    #[cfg(target_arch = "aarch64")]
    {
        RoverArchId::Aarch64
    }
    #[cfg(target_arch = "riscv64")]
    {
        RoverArchId::Riscv64
    }
    #[cfg(target_arch = "x86")]
    {
        RoverArchId::Ia32
    }
    #[cfg(target_arch = "arm")]
    {
        RoverArchId::Arm32
    }
    #[cfg(target_arch = "loongarch64")]
    {
        RoverArchId::LoongArch64
    }
    #[cfg(not(any(
        target_arch = "x86_64",
        target_arch = "aarch64",
        target_arch = "riscv64",
        target_arch = "x86",
        target_arch = "arm",
        target_arch = "loongarch64"
    )))]
    {
        RoverArchId::Unknown
    }
}
