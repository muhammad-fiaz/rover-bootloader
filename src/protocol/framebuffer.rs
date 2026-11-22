//! # Rover boot protocol framebuffer definitions
//!
//! Framebuffer format and geometry descriptions passed to the target kernel.

/// Pixel format enumeration for the graphics display.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum RoverPixelFormat {
    /// 32-bit color: Red 8-bit, Green 8-bit, Blue 8-bit, Reserved 8-bit.
    Rgb = 0,
    /// 32-bit color: Blue 8-bit, Green 8-bit, Red 8-bit, Reserved 8-bit.
    Bgr = 1,
    /// Custom bitmask color described by mask fields.
    Bitmask = 2,
    /// BLT-only framebuffer without direct memory aperture access.
    BltOnly = 3,
    /// Unknown or unsupported pixel format.
    Unknown = 4,
}

/// Linear framebuffer metadata provided to the kernel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct RoverFramebuffer {
    /// Physical base address of the video memory aperture.
    pub base: u64,
    /// Total size in bytes of the framebuffer mapping.
    pub size: u64,
    /// Horizontal resolution in pixels.
    pub width: u32,
    /// Vertical resolution in pixels.
    pub height: u32,
    /// Number of pixels per horizontal scanline (stride).
    pub stride: u32,
    /// Color format of the framebuffer pixels.
    pub format: RoverPixelFormat,
    /// Bitmask for red color channel (valid when `format == Bitmask`).
    pub red_mask: u32,
    /// Bitmask for green color channel (valid when `format == Bitmask`).
    pub green_mask: u32,
    /// Bitmask for blue color channel (valid when `format == Bitmask`).
    pub blue_mask: u32,
    /// Bitmask for reserved/alpha channel (valid when `format == Bitmask`).
    pub reserved_mask: u32,
}

impl RoverFramebuffer {
    /// Returns an empty or uninitialized framebuffer descriptor.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            base: 0,
            size: 0,
            width: 0,
            height: 0,
            stride: 0,
            format: RoverPixelFormat::Unknown,
            red_mask: 0,
            green_mask: 0,
            blue_mask: 0,
            reserved_mask: 0,
        }
    }

    /// Returns `true` if a usable linear framebuffer is present.
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        self.base != 0 && self.size != 0 && self.width != 0 && self.height != 0
    }
}
