//! # Interactive boot menu
//!
//! Visual boot entry selection menu with keyboard navigation and timeout countdown.

use crate::config::parser::BootConfig;
use crate::graphics::console::ConsoleWriter;
use rover_uefi::protocols::console::input::{
    CHAR_CARRIAGE_RETURN, EfiInputKey, EfiSimpleTextInputProtocol, SCAN_DOWN, SCAN_UP,
};
use rover_uefi::tables::boot::EfiBootServices;

/// Runs the interactive boot menu and returns the index of the selected boot entry.
///
/// # Safety
///
/// If non-null, `con_in` must point to an initialized [`EfiSimpleTextInputProtocol`].
/// If non-null, `bs` must point to an initialized [`EfiBootServices`].
pub unsafe fn run_boot_menu(
    config: &BootConfig<'_>,
    writer: &mut ConsoleWriter<'_>,
    con_in: *mut EfiSimpleTextInputProtocol,
    bs: *mut EfiBootServices,
) -> usize {
    let count = config.count();
    if count == 0 {
        return 0;
    }
    if count == 1 && config.timeout_seconds == 0 {
        return 0;
    }

    let mut selected = if config.default_entry < count {
        config.default_entry
    } else {
        0
    };

    let mut remaining_timeout = config.timeout_seconds;

    loop {
        // Redraw menu.
        writer.clear();
        writer.set_color(11, 0); // Light cyan
        writer.write_str("=====================================================\n");
        writer.write_str("               ROVER BOOTLOADER v0.1.0               \n");
        writer.write_str("=====================================================\n\n");

        writer.set_color(15, 0); // White
        writer.write_str("Select a boot entry:\n\n");

        let entries = config.entries();
        for (i, entry_opt) in entries.iter().enumerate() {
            if let Some(entry) = entry_opt {
                if i == selected {
                    writer.set_color(14, 1); // Yellow on blue
                    writer.write_str(" -> ");
                } else {
                    writer.set_color(7, 0); // Light gray on black
                    writer.write_str("    ");
                }

                writer.write_str(entry.title);
                if i == config.default_entry {
                    writer.write_str(" (default)");
                }
                writer.write_str("\n");
            }
        }

        writer.set_color(7, 0);
        writer.write_str("\n-----------------------------------------------------\n");
        writer.write_str("Use [Up/Down] arrows to select, [Enter] to boot.\n");

        if remaining_timeout > 0 {
            writer.write_str("Automatic boot in ");
            // Print remaining seconds
            #[allow(clippy::cast_possible_truncation)]
            let sec_char = (b'0' + (remaining_timeout.unsigned_abs() as u8 % 10)) as char;
            let mut sec_buf = [0u8; 4];
            let s = sec_char.encode_utf8(&mut sec_buf);
            writer.write_str(s);
            writer.write_str(" seconds... Press any key to stop timer.\n");
        } else if remaining_timeout == 0 {
            // Timeout expired, boot selected.
            break;
        }

        // Wait for key or 1-second timeout.
        if !con_in.is_null() && !bs.is_null() {
            // Check for keystroke.
            let mut key = EfiInputKey {
                scan_code: 0,
                unicode_char: 0,
            };

            let key_status = unsafe { ((*con_in).read_key_stroke)(con_in, &raw mut key) };

            if key_status.is_success() {
                // Key pressed! Stop timeout.
                remaining_timeout = -1;

                if key.scan_code == SCAN_UP {
                    selected = selected.saturating_sub(1);
                } else if key.scan_code == SCAN_DOWN {
                    if selected + 1 < count {
                        selected += 1;
                    }
                } else if key.unicode_char == CHAR_CARRIAGE_RETURN
                    || key.unicode_char == u16::from(b'\n')
                {
                    break;
                }
            } else if remaining_timeout > 0 {
                // Sleep 1 second (1,000,000 microseconds).
                unsafe { ((*bs).stall)(1_000_000) };
                remaining_timeout -= 1;
            } else {
                // No timeout active, stall briefly to avoid burning CPU (100 ms).
                unsafe { ((*bs).stall)(100_000) };
            }
        } else {
            // No input device or boot services, immediately boot default.
            break;
        }
    }

    selected
}
