//! # Cryptography tests
//!
//! Validates native SHA-256 implementation against standard test vectors.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unreadable_literal,
    clippy::needless_raw_string_hashes,
    clippy::items_after_statements,
    clippy::needless_range_loop,
    clippy::cast_possible_truncation
)]

use rover_bootloader::security::{sha256, verify_sha256_hex};

const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";

#[test]
fn test_sha256_empty_string() {
    let digest = sha256(b"");
    let expected = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
    assert!(verify_sha256_hex(b"", expected));

    let mut actual_hex = [0u8; 64];
    for (i, &byte) in digest.iter().enumerate() {
        actual_hex[i * 2] = HEX_CHARS[(byte >> 4) as usize];
        actual_hex[i * 2 + 1] = HEX_CHARS[(byte & 0x0F) as usize];
    }
    assert_eq!(core::str::from_utf8(&actual_hex).unwrap(), expected);
}

#[test]
fn test_sha256_standard_vector_abc() {
    let input = b"abc";
    let expected = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
    assert!(verify_sha256_hex(input, expected));
}

#[test]
fn test_sha256_multi_block_vector() {
    let input = b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq";
    let expected = "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1";
    assert!(verify_sha256_hex(input, expected));
}

#[test]
fn test_sha256_verification_mismatch() {
    let input = b"kernel image bytes";
    let wrong_hash = "0000000000000000000000000000000000000000000000000000000000000000";
    assert!(!verify_sha256_hex(input, wrong_hash));
    assert!(!verify_sha256_hex(input, "short"));
}
