//! # Security subsystem
//!
//! UEFI Secure Boot detection and native SHA-256 cryptographic verification for boot images.

pub mod secure_boot;
pub mod sha256;

pub use secure_boot::is_secure_boot_enabled;
pub use sha256::{sha256, verify_sha256_hex};
