//! # Path encoding tests
//!
//! Validates UEFI path normalization and UTF-16 encoding.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unreadable_literal,
    clippy::needless_raw_string_hashes,
    clippy::items_after_statements,
    clippy::needless_range_loop,
    clippy::cast_possible_truncation,
    clippy::cast_lossless
)]

use rover_bootloader::fs::{encode_uefi_path, path::MAX_PATH_LENGTH};

#[test]
fn test_path_forward_slash_conversion() {
    let mut buf = [0u16; MAX_PATH_LENGTH];
    let len = encode_uefi_path("/boot/kernel.elf", &mut buf).unwrap();

    assert_eq!(len, 16);
    // Backslash is 0x005C
    assert_eq!(buf[0], b'\\' as u16);
    assert_eq!(buf[1], b'b' as u16);
    assert_eq!(buf[5], b'\\' as u16);
    assert_eq!(buf[16], 0); // Null terminator
}

#[test]
fn test_path_windows_backslash_preserved() {
    let mut buf = [0u16; MAX_PATH_LENGTH];
    let len = encode_uefi_path("\\EFI\\BOOT\\rover.cfg", &mut buf).unwrap();

    assert_eq!(len, 19);
    assert_eq!(buf[0], b'\\' as u16);
    assert_eq!(buf[4], b'\\' as u16);
    assert_eq!(buf[9], b'\\' as u16);
    assert_eq!(buf[19], 0);
}

#[test]
fn test_path_overflow_rejection() {
    let mut buf = [0u16; MAX_PATH_LENGTH];
    let long_path = [b'a'; 300];
    let s = core::str::from_utf8(&long_path).unwrap();

    assert!(encode_uefi_path(s, &mut buf).is_err());
}
