//! # Boot configuration entry
//!
//! Representation of a single bootable kernel target configuration.

/// A configured boot target entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BootEntry<'a> {
    /// Human-readable title displayed in the boot menu.
    pub title: &'a str,
    /// Path on the boot filesystem to the kernel executable image.
    pub kernel_path: &'a str,
    /// Optional path to an initial ramdisk (initrd) image.
    pub initrd_path: Option<&'a str>,
    /// Optional kernel command-line arguments.
    pub cmdline: Option<&'a str>,
    /// Optional display resolution request (e.g., "1920x1080").
    pub resolution: Option<&'a str>,
    /// Optional expected SHA-256 hash in hexadecimal format for integrity verification.
    pub expected_sha256: Option<&'a str>,
}

impl<'a> BootEntry<'a> {
    /// Creates a new boot entry with the specified title and kernel path.
    #[must_use]
    pub const fn new(title: &'a str, kernel_path: &'a str) -> Self {
        Self {
            title,
            kernel_path,
            initrd_path: None,
            cmdline: None,
            resolution: None,
            expected_sha256: None,
        }
    }
}
