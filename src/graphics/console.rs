//! # Console output
//!
//! Text output utilities and macros wrapping [`rover_uefi::protocols::console::output`].

use core::fmt;
use rover_uefi::abi::types::Char16;
use rover_uefi::protocols::console::output::EfiSimpleTextOutputProtocol;

/// Helper writer for UEFI console output.
pub struct ConsoleWriter<'a> {
    output: *mut EfiSimpleTextOutputProtocol,
    _marker: core::marker::PhantomData<&'a ()>,
}

impl ConsoleWriter<'_> {
    /// Creates a new console writer wrapping the UEFI text output protocol.
    #[must_use]
    pub const fn new(output: *mut EfiSimpleTextOutputProtocol) -> Self {
        Self {
            output,
            _marker: core::marker::PhantomData,
        }
    }

    /// Clears the console screen.
    pub fn clear(&mut self) {
        if !self.output.is_null() {
            // SAFETY: output pointer is non-null.
            let _ = unsafe { ((*self.output).clear_screen)(self.output) };
        }
    }

    /// Sets the text attribute (foreground and background color).
    pub fn set_color(&mut self, foreground: u8, background: u8) {
        if !self.output.is_null() {
            let attr = ((background as usize & 0x07) << 4) | (foreground as usize & 0x0F);
            // SAFETY: output pointer is non-null.
            let _ = unsafe { ((*self.output).set_attribute)(self.output, attr) };
        }
    }

    /// Writes an ASCII / UTF-8 string to the console.
    pub fn write_str(&mut self, s: &str) {
        if self.output.is_null() {
            return;
        }

        let mut buf: [Char16; 128] = [0; 128];
        let mut idx = 0;

        for ch in s.chars() {
            if ch == '\n' {
                // In UEFI console, newline requires CR + LF.
                if idx + 2 >= buf.len() - 1 {
                    buf[idx] = 0;
                    // SAFETY: buf is null-terminated.
                    unsafe { ((*self.output).output_string)(self.output, buf.as_mut_ptr()) };
                    idx = 0;
                }
                buf[idx] = u16::from(b'\r');
                idx += 1;
                buf[idx] = u16::from(b'\n');
                idx += 1;
                continue;
            }

            let mut utf16 = [0u16; 2];
            let encoded = ch.encode_utf16(&mut utf16);

            for unit in encoded.iter().copied() {
                if idx >= buf.len() - 1 {
                    buf[idx] = 0;
                    // SAFETY: buf is null-terminated.
                    unsafe { ((*self.output).output_string)(self.output, buf.as_mut_ptr()) };
                    idx = 0;
                }
                buf[idx] = unit;
                idx += 1;
            }
        }

        if idx > 0 {
            buf[idx] = 0;
            // SAFETY: buf is null-terminated.
            unsafe { ((*self.output).output_string)(self.output, buf.as_mut_ptr()) };
        }
    }
}

impl fmt::Write for ConsoleWriter<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_str(s);
        Ok(())
    }
}
