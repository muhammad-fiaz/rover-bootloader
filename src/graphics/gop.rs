//! # Graphics output protocol integration
//!
//! Framebuffer discovery, mode query, and configuration using UEFI GOP.

use core::ffi::c_void;

use crate::error::BootError;
use crate::protocol::framebuffer::{RoverFramebuffer, RoverPixelFormat};
use rover_uefi::abi::types::UIntN;
use rover_uefi::protocols::graphics::guid::GRAPHICS_OUTPUT_GUID;
use rover_uefi::protocols::graphics::mode::EfiGraphicsOutputModeInformation;
use rover_uefi::protocols::graphics::pixel::PixelFormat;
use rover_uefi::protocols::graphics::protocol::EfiGraphicsOutputProtocol;
use rover_uefi::tables::boot::EfiBootServices;

/// Converts a UEFI [`PixelFormat`] into a [`RoverPixelFormat`].
#[must_use]
pub fn convert_pixel_format(format: PixelFormat) -> RoverPixelFormat {
    if format == PixelFormat::RGB_RESERVED {
        RoverPixelFormat::Rgb
    } else if format == PixelFormat::BGR_RESERVED {
        RoverPixelFormat::Bgr
    } else if format == PixelFormat::BITMASK {
        RoverPixelFormat::Bitmask
    } else if format == PixelFormat::BLT_ONLY {
        RoverPixelFormat::BltOnly
    } else {
        RoverPixelFormat::Unknown
    }
}

/// Locates the `EFI_GRAPHICS_OUTPUT_PROTOCOL` and extracts the active framebuffer geometry.
///
/// If `requested_resolution` is specified (e.g. "1920x1080"), attempts to match and set that mode.
///
/// # Safety
///
/// `bs` must be a valid, live pointer to an initialized [`EfiBootServices`].
///
/// # Errors
///
/// Returns [`BootError::GraphicsModeUnsupported`] if no graphics protocol or supported mode exists.
pub unsafe fn discover_framebuffer(
    bs: *mut EfiBootServices,
    requested_resolution: Option<&str>,
) -> Result<RoverFramebuffer, BootError> {
    if bs.is_null() {
        return Err(BootError::GraphicsModeUnsupported);
    }

    let mut gop_ptr: *mut c_void = core::ptr::null_mut();
    let mut gop_guid = GRAPHICS_OUTPUT_GUID;
    // SAFETY: bs is valid, out pointer points to valid local stack memory.
    let status = unsafe {
        ((*bs).locate_protocol)(&raw mut gop_guid, core::ptr::null_mut(), &raw mut gop_ptr)
    };

    if !status.is_success() || gop_ptr.is_null() {
        return Err(BootError::GraphicsModeUnsupported);
    }

    let gop = gop_ptr.cast::<EfiGraphicsOutputProtocol>();

    // If a resolution is requested, attempt to set the matching mode.
    if let Some((w, h)) = requested_resolution.and_then(parse_resolution_string) {
        let _ = try_set_resolution(gop, bs, w, h);
    }

    let mode_ptr = unsafe { (*gop).mode };
    if mode_ptr.is_null() {
        return Err(BootError::GraphicsModeUnsupported);
    }

    let mode = unsafe { &*mode_ptr };
    if mode.info.is_null() {
        return Err(BootError::GraphicsModeUnsupported);
    }

    let info = unsafe { &*mode.info };
    let format = convert_pixel_format(info.pixel_format);

    let mut fb = RoverFramebuffer {
        base: mode.framebuffer_base.to_u64(),
        size: mode.framebuffer_size as u64,
        width: info.horizontal_resolution,
        height: info.vertical_resolution,
        stride: info.pixels_per_scan_line,
        format,
        red_mask: info.pixel_information.red_mask,
        green_mask: info.pixel_information.green_mask,
        blue_mask: info.pixel_information.blue_mask,
        reserved_mask: info.pixel_information.reserved_mask,
    };

    // If stride is zero, fallback to horizontal resolution.
    if fb.stride == 0 {
        fb.stride = fb.width;
    }

    Ok(fb)
}

/// Attempts to find and set a display mode matching the requested width and height.
fn try_set_resolution(
    gop: *mut EfiGraphicsOutputProtocol,
    bs: *mut EfiBootServices,
    width: u32,
    height: u32,
) -> Result<(), BootError> {
    let mode_ptr = unsafe { (*gop).mode };
    if mode_ptr.is_null() {
        return Err(BootError::GraphicsModeUnsupported);
    }

    let max_mode = unsafe { (*mode_ptr).max_mode };
    for mode_num in 0..max_mode {
        let mut info_size: UIntN = 0;
        let mut info_ptr: *mut EfiGraphicsOutputModeInformation = core::ptr::null_mut();

        // SAFETY: gop is valid, out params are stack locals.
        let status =
            unsafe { ((*gop).query_mode)(gop, mode_num, &raw mut info_size, &raw mut info_ptr) };

        if status.is_success() && !info_ptr.is_null() {
            let info = unsafe { &*info_ptr };
            let matches = info.horizontal_resolution == width && info.vertical_resolution == height;

            // Free the firmware-allocated mode info buffer.
            let _ = unsafe { ((*bs).free_pool)(info_ptr.cast::<c_void>()) };

            if matches {
                // SAFETY: gop is non-null.
                let set_status = unsafe { ((*gop).set_mode)(gop, mode_num) };
                if set_status.is_success() {
                    return Ok(());
                }
            }
        }
    }

    Err(BootError::GraphicsModeUnsupported)
}

/// Parses a resolution string like "1920x1080" into `(width, height)`.
fn parse_resolution_string(s: &str) -> Option<(u32, u32)> {
    let mut parts = s.split('x');
    let width_str = parts.next()?.trim();
    let height_str = parts.next()?.trim();
    let width = parse_u32(width_str)?;
    let height = parse_u32(height_str)?;
    Some((width, height))
}

fn parse_u32(s: &str) -> Option<u32> {
    let mut acc: u32 = 0;
    for &b in s.as_bytes() {
        if !b.is_ascii_digit() {
            return None;
        }
        acc = acc.checked_mul(10)?.checked_add(u32::from(b - b'0'))?;
    }
    Some(acc)
}
