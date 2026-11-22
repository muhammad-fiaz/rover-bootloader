//! # Native SHA-256 implementation
//!
//! Zero-allocation, FIPS 180-4 compliant SHA-256 cryptographic hash function for boot image integrity.

/// Round constants for SHA-256 (first 32 bits of the fractional parts of cube roots of first 64 primes).
const K: [u32; 64] = [
    0x428A_2F98,
    0x7137_4491,
    0xB5C0_FBCF,
    0xE9B5_DBA5,
    0x3956_C25B,
    0x59F1_11F1,
    0x923F_82A4,
    0xAB1C_5ED5,
    0xD807_AA98,
    0x1283_5B01,
    0x2431_85BE,
    0x550C_7DC3,
    0x72BE_5D74,
    0x80DE_B1FE,
    0x9BDC_06A7,
    0xC19B_F174,
    0xE49B_69C1,
    0xEFBE_4786,
    0x0FC1_9DC6,
    0x240C_A1CC,
    0x2DE9_2C6F,
    0x4A74_84AA,
    0x5CB0_A9DC,
    0x76F9_88DA,
    0x983E_5152,
    0xA831_C66D,
    0xB003_27C8,
    0xBF59_7FC7,
    0xC6E0_0BF3,
    0xD5A7_9147,
    0x06CA_6351,
    0x1429_2967,
    0x27B7_0A85,
    0x2E1B_2138,
    0x4D2C_6DFC,
    0x5338_0D13,
    0x650A_7354,
    0x766A_0ABB,
    0x81C2_C92E,
    0x9272_2C85,
    0xA2BF_E8A1,
    0xA81A_664B,
    0xC24B_8B70,
    0xC76C_51A3,
    0xD192_E819,
    0xD699_0624,
    0xF40E_3585,
    0x106A_A070,
    0x19A4_C116,
    0x1E37_6C08,
    0x2748_774C,
    0x34B0_BCB5,
    0x391C_0CB3,
    0x4ED8_AA4A,
    0x5B9C_CA4F,
    0x682E_6FF3,
    0x748F_82EE,
    0x78A5_636F,
    0x84C8_7814,
    0x8CC7_0208,
    0x90BE_FFFA,
    0xA450_6CEB,
    0xBEF9_A3F7,
    0xC671_78F2,
];

/// Initial hash values (first 32 bits of fractional parts of square roots of first 8 primes).
const H_INIT: [u32; 8] = [
    0x6A09_E667,
    0xBB67_AE85,
    0x3C6E_F372,
    0xA54F_F53A,
    0x510E_527F,
    0x9B05_688C,
    0x1F83_D9AB,
    0x5BE0_CD19,
];

/// Hexadecimal character lookup table.
const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";

#[inline]
const fn ch(x: u32, y: u32, z: u32) -> u32 {
    (x & y) ^ (!x & z)
}

#[inline]
const fn maj(x: u32, y: u32, z: u32) -> u32 {
    (x & y) ^ (x & z) ^ (y & z)
}

#[inline]
const fn big_sigma0(x: u32) -> u32 {
    x.rotate_right(2) ^ x.rotate_right(13) ^ x.rotate_right(22)
}

#[inline]
const fn big_sigma1(x: u32) -> u32 {
    x.rotate_right(6) ^ x.rotate_right(11) ^ x.rotate_right(25)
}

#[inline]
const fn small_sigma0(x: u32) -> u32 {
    x.rotate_right(7) ^ x.rotate_right(18) ^ (x >> 3)
}

#[inline]
const fn small_sigma1(x: u32) -> u32 {
    x.rotate_right(17) ^ x.rotate_right(19) ^ (x >> 10)
}

/// Computes the SHA-256 digest of the given data slice.
#[must_use]
pub fn sha256(data: &[u8]) -> [u8; 32] {
    let mut state = H_INIT;
    let mut buffer = [0u8; 64];
    let mut buffer_len = 0;
    let total_bits = (data.len() as u64).saturating_mul(8);

    for &byte in data {
        buffer[buffer_len] = byte;
        buffer_len += 1;
        if buffer_len == 64 {
            process_block(&mut state, &buffer);
            buffer_len = 0;
        }
    }

    // Append 0x80 padding byte.
    buffer[buffer_len] = 0x80;
    buffer_len += 1;

    // If buffer length > 56, pad with zeroes, process block, and start a new block.
    if buffer_len > 56 {
        while buffer_len < 64 {
            buffer[buffer_len] = 0;
            buffer_len += 1;
        }
        process_block(&mut state, &buffer);
        buffer_len = 0;
    }

    // Pad remaining space with zeroes up to byte 56.
    while buffer_len < 56 {
        buffer[buffer_len] = 0;
        buffer_len += 1;
    }

    // Append 64-bit big-endian bit length.
    let len_bytes = total_bits.to_be_bytes();
    buffer[56..64].copy_from_slice(&len_bytes);
    process_block(&mut state, &buffer);

    // Format output digest into 32 big-endian bytes.
    let mut digest = [0u8; 32];
    for (i, &word) in state.iter().enumerate() {
        let bytes = word.to_be_bytes();
        digest[i * 4..(i + 1) * 4].copy_from_slice(&bytes);
    }

    digest
}

#[allow(clippy::many_single_char_names)]
fn process_block(state: &mut [u32; 8], block: &[u8; 64]) {
    let mut w = [0u32; 64];

    for i in 0..16 {
        w[i] = u32::from_be_bytes([
            block[i * 4],
            block[i * 4 + 1],
            block[i * 4 + 2],
            block[i * 4 + 3],
        ]);
    }

    for i in 16..64 {
        w[i] = small_sigma1(w[i - 2])
            .wrapping_add(w[i - 7])
            .wrapping_add(small_sigma0(w[i - 15]))
            .wrapping_add(w[i - 16]);
    }

    let mut a = state[0];
    let mut b = state[1];
    let mut c = state[2];
    let mut d = state[3];
    let mut e = state[4];
    let mut f = state[5];
    let mut g = state[6];
    let mut h = state[7];

    for i in 0..64 {
        let t1 = h
            .wrapping_add(big_sigma1(e))
            .wrapping_add(ch(e, f, g))
            .wrapping_add(K[i])
            .wrapping_add(w[i]);
        let t2 = big_sigma0(a).wrapping_add(maj(a, b, c));

        h = g;
        g = f;
        f = e;
        e = d.wrapping_add(t1);
        d = c;
        c = b;
        b = a;
        a = t1.wrapping_add(t2);
    }

    state[0] = state[0].wrapping_add(a);
    state[1] = state[1].wrapping_add(b);
    state[2] = state[2].wrapping_add(c);
    state[3] = state[3].wrapping_add(d);
    state[4] = state[4].wrapping_add(e);
    state[5] = state[5].wrapping_add(f);
    state[6] = state[6].wrapping_add(g);
    state[7] = state[7].wrapping_add(h);
}

/// Verifies whether the SHA-256 digest of `data` matches an expected hexadecimal string.
#[must_use]
pub fn verify_sha256_hex(data: &[u8], expected_hex: &str) -> bool {
    let expected = expected_hex.trim();
    if expected.len() != 64 {
        return false;
    }

    let actual = sha256(data);
    let mut actual_hex = [0u8; 64];

    for (i, &byte) in actual.iter().enumerate() {
        actual_hex[i * 2] = HEX_CHARS[(byte >> 4) as usize];
        actual_hex[i * 2 + 1] = HEX_CHARS[(byte & 0x0F) as usize];
    }

    for (a, e) in actual_hex.iter().zip(expected.as_bytes().iter()) {
        if *a != e.to_ascii_lowercase() {
            return false;
        }
    }

    true
}
