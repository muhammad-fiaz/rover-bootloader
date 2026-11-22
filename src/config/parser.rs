//! # Boot configuration parser
//!
//! Zero-allocation parser for `rover.cfg` configuration files.

use super::entry::BootEntry;
use crate::error::BootError;

/// Maximum number of boot entries supported in a single configuration.
pub const MAX_BOOT_ENTRIES: usize = 16;

/// Parsed bootloader configuration.
#[derive(Debug)]
pub struct BootConfig<'a> {
    /// Zero-based index of the default boot entry.
    pub default_entry: usize,
    /// Timeout in seconds before automatically selecting the default entry.
    /// A value of 0 boots immediately; -1 disables timeout.
    pub timeout_seconds: i32,
    entries: [Option<BootEntry<'a>>; MAX_BOOT_ENTRIES],
    entry_count: usize,
}

impl<'a> BootConfig<'a> {
    /// Creates an empty configuration with reasonable defaults.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            default_entry: 0,
            timeout_seconds: 5,
            entries: [None; MAX_BOOT_ENTRIES],
            entry_count: 0,
        }
    }

    /// Returns a slice of valid boot entries.
    #[must_use]
    pub fn entries(&self) -> &[Option<BootEntry<'a>>] {
        &self.entries[..self.entry_count]
    }

    /// Returns the number of loaded boot entries.
    #[must_use]
    pub const fn count(&self) -> usize {
        self.entry_count
    }

    /// Returns the default boot entry, if any exists.
    #[must_use]
    pub fn selected_entry(&self) -> Option<&BootEntry<'a>> {
        if self.entry_count == 0 {
            None
        } else if self.default_entry < self.entry_count {
            self.entries[self.default_entry].as_ref()
        } else {
            self.entries[0].as_ref()
        }
    }

    /// Appends a boot entry to the configuration if space allows.
    ///
    /// # Errors
    ///
    /// Returns [`BootError::ConfigSyntaxError`] if the maximum entry count is exceeded.
    pub fn add_entry(&mut self, entry: BootEntry<'a>) -> Result<(), BootError> {
        if self.entry_count >= MAX_BOOT_ENTRIES {
            return Err(BootError::ConfigSyntaxError);
        }
        self.entries[self.entry_count] = Some(entry);
        self.entry_count += 1;
        Ok(())
    }

    /// Parses configuration text from a string slice without heap allocations.
    ///
    /// # Errors
    ///
    /// Returns [`BootError::ConfigSyntaxError`] if parsing encounters invalid syntax.
    pub fn parse(text: &'a str) -> Result<Self, BootError> {
        let mut config = Self::new();
        let mut current_entry: Option<BootEntry<'a>> = None;

        for raw_line in text.lines() {
            let line = trim_str(raw_line);
            // Skip empty lines and comment lines.
            if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                continue;
            }

            // Check for entry block start: `entry "Title" {` or `entry Title {`
            if let Some(entry_decl) = line.strip_prefix("entry") {
                if let Some(prev) = current_entry.take() {
                    config.add_entry(prev)?;
                }

                let rest = trim_str(entry_decl);
                let title_part = if let Some(stripped) = rest.strip_suffix('{') {
                    trim_str(stripped)
                } else {
                    rest
                };
                let title = unquote_str(title_part);
                current_entry = Some(BootEntry::new(title, ""));
                continue;
            }

            // Check for entry block close: `}`
            if line == "}" {
                if let Some(entry) = current_entry.take() {
                    if entry.kernel_path.is_empty() {
                        return Err(BootError::ConfigMissingKernel);
                    }
                    config.add_entry(entry)?;
                }
                continue;
            }

            // Check for section header: `[entry]`
            if line.eq_ignore_ascii_case("[entry]") {
                if let Some(prev) = current_entry.take() {
                    config.add_entry(prev)?;
                }
                current_entry = Some(BootEntry::new("Default Kernel", ""));
                continue;
            }

            // Parse key-value assignment: `key = value`
            if let Some((key_part, val_part)) = split_once(line, '=') {
                let key = trim_str(key_part);
                let value = unquote_str(trim_str(val_part));

                if let Some(ref mut entry) = current_entry {
                    match key {
                        "title" | "name" => entry.title = value,
                        "kernel" | "image" | "path" => entry.kernel_path = value,
                        "initrd" | "ramdisk" | "module" => entry.initrd_path = Some(value),
                        "cmdline" | "args" | "append" => entry.cmdline = Some(value),
                        "resolution" | "mode" => entry.resolution = Some(value),
                        "sha256" | "hash" => entry.expected_sha256 = Some(value),
                        _ => {}
                    }
                } else {
                    match key {
                        "default" => {
                            if let Some(num) = parse_usize(value) {
                                config.default_entry = num;
                            }
                        }
                        "timeout" => {
                            if let Some(num) = parse_i32(value) {
                                config.timeout_seconds = num;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        // Commit final entry if open without closing brace.
        if let Some(entry) = current_entry {
            if entry.kernel_path.is_empty() {
                return Err(BootError::ConfigMissingKernel);
            }
            config.add_entry(entry)?;
        }

        Ok(config)
    }

    /// Returns a default fallback configuration pointing to `\kernel.elf`.
    #[must_use]
    pub const fn fallback() -> Self {
        let mut config = Self::new();
        config.entries[0] = Some(BootEntry {
            title: "Rover OS (Fallback)",
            kernel_path: "\\kernel.elf",
            initrd_path: None,
            cmdline: None,
            resolution: None,
            expected_sha256: None,
        });
        config.entry_count = 1;
        config
    }
}

impl Default for BootConfig<'_> {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to trim ASCII whitespace.
fn trim_str(s: &str) -> &str {
    let bytes = s.as_bytes();
    let mut start = 0;
    while start < bytes.len()
        && (bytes[start] == b' ' || bytes[start] == b'\t' || bytes[start] == b'\r')
    {
        start += 1;
    }
    let mut end = bytes.len();
    while end > start
        && (bytes[end - 1] == b' ' || bytes[end - 1] == b'\t' || bytes[end - 1] == b'\r')
    {
        end -= 1;
    }
    // SAFETY: start and end are on valid ASCII whitespace boundaries.
    unsafe { core::str::from_utf8_unchecked(&bytes[start..end]) }
}

/// Helper function to remove surrounding quotes.
fn unquote_str(s: &str) -> &str {
    let bytes = s.as_bytes();
    if bytes.len() >= 2
        && ((bytes[0] == b'"' && bytes[bytes.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[bytes.len() - 1] == b'\''))
    {
        // SAFETY: sliced inner bytes.
        unsafe { core::str::from_utf8_unchecked(&bytes[1..bytes.len() - 1]) }
    } else {
        s
    }
}

/// Splits a string on the first occurrence of a delimiter character.
fn split_once(s: &str, delimiter: char) -> Option<(&str, &str)> {
    for (idx, ch) in s.char_indices() {
        if ch == delimiter {
            let left = &s[..idx];
            let right = &s[idx + ch.len_utf8()..];
            return Some((left, right));
        }
    }
    None
}

/// Parses an unsigned integer from an ASCII string.
fn parse_usize(s: &str) -> Option<usize> {
    let mut acc: usize = 0;
    let bytes = s.as_bytes();
    if bytes.is_empty() {
        return None;
    }
    for &b in bytes {
        if !b.is_ascii_digit() {
            return None;
        }
        acc = acc.checked_mul(10)?.checked_add(usize::from(b - b'0'))?;
    }
    Some(acc)
}

/// Parses a signed 32-bit integer from an ASCII string.
fn parse_i32(s: &str) -> Option<i32> {
    let bytes = s.as_bytes();
    if bytes.is_empty() {
        return None;
    }
    let (is_negative, digits) = if bytes[0] == b'-' {
        (true, &bytes[1..])
    } else {
        (false, bytes)
    };
    if digits.is_empty() {
        return None;
    }
    let mut acc: i32 = 0;
    for &b in digits {
        if !b.is_ascii_digit() {
            return None;
        }
        acc = acc.checked_mul(10)?.checked_add(i32::from(b - b'0'))?;
    }
    if is_negative { Some(-acc) } else { Some(acc) }
}
