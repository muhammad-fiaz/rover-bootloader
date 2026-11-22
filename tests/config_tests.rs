//! # Boot configuration tests
//!
//! Validates parsing of `rover.cfg` configuration files across multiple syntax formats.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unreadable_literal,
    clippy::needless_raw_string_hashes,
    clippy::items_after_statements,
    clippy::needless_range_loop,
    clippy::cast_possible_truncation
)]

use rover_bootloader::config::parser::BootConfig;
use rover_bootloader::error::BootError;

#[test]
fn test_config_block_syntax() {
    let cfg_str = r#"
        # Rover OS configuration
        default = 1
        timeout = 10

        entry "Rover OS Primary" {
            kernel = /boot/kernel.elf
            initrd = /boot/initrd.img
            cmdline = console=tty0 root=/dev/sda1
            resolution = 1920x1080
            sha256 = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        }

        entry "Rover OS Rescue" {
            kernel = /boot/rescue.elf
            cmdline = single
        }
    "#;

    let config = BootConfig::parse(cfg_str).unwrap();
    assert_eq!(config.default_entry, 1);
    assert_eq!(config.timeout_seconds, 10);
    assert_eq!(config.count(), 2);

    let entries = config.entries();
    let e0 = entries[0].as_ref().unwrap();
    assert_eq!(e0.title, "Rover OS Primary");
    assert_eq!(e0.kernel_path, "/boot/kernel.elf");
    assert_eq!(e0.initrd_path, Some("/boot/initrd.img"));
    assert_eq!(e0.cmdline, Some("console=tty0 root=/dev/sda1"));
    assert_eq!(e0.resolution, Some("1920x1080"));
    assert_eq!(
        e0.expected_sha256,
        Some("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855")
    );

    let e1 = entries[1].as_ref().unwrap();
    assert_eq!(e1.title, "Rover OS Rescue");
    assert_eq!(e1.kernel_path, "/boot/rescue.elf");
    assert_eq!(e1.initrd_path, None);
    assert_eq!(e1.cmdline, Some("single"));

    assert_eq!(config.selected_entry().unwrap().title, "Rover OS Rescue");
}

#[test]
fn test_config_ini_syntax() {
    let cfg_str = r#"
        default = 0
        timeout = 3

        [entry]
        title = Rover Default
        kernel = \kernel.elf
        cmdline = init=/bin/sh
    "#;

    let config = BootConfig::parse(cfg_str).unwrap();
    assert_eq!(config.count(), 1);
    let entry = config.selected_entry().unwrap();
    assert_eq!(entry.title, "Rover Default");
    assert_eq!(entry.kernel_path, "\\kernel.elf");
    assert_eq!(entry.cmdline, Some("init=/bin/sh"));
}

#[test]
fn test_config_fallback() {
    let fallback = BootConfig::fallback();
    assert_eq!(fallback.count(), 1);
    let entry = fallback.selected_entry().unwrap();
    assert_eq!(entry.kernel_path, "\\kernel.elf");
    assert_eq!(fallback.timeout_seconds, 5);
}

#[test]
fn test_config_missing_kernel_rejection() {
    let bad_cfg = r#"
        entry "No Kernel" {
            cmdline = foo
        }
    "#;
    let err = BootConfig::parse(bad_cfg).unwrap_err();
    assert_eq!(err, BootError::ConfigMissingKernel);
}
