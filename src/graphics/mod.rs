//! # Graphics and console subsystem
//!
//! Text output protocol handling, formatted printing, and Graphics Output Protocol (GOP) discovery.

pub mod console;
pub mod gop;

pub use console::ConsoleWriter;
pub use gop::{convert_pixel_format, discover_framebuffer};
