//! # PE/COFF parser tests
//!
//! Validates parsing of PE32+ (64-bit) and PE32 (32-bit) headers and metadata.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unreadable_literal,
    clippy::needless_raw_string_hashes,
    clippy::items_after_statements,
    clippy::needless_range_loop,
    clippy::cast_possible_truncation
)]

use rover_bootloader::error::BootError;
use rover_bootloader::formats::pe::{
    DOS_MAGIC, PE_SIGNATURE, PE32_MAGIC, PE32_PLUS_MAGIC, PeBinary,
};

fn create_valid_pe32_plus_binary() -> Vec<u8> {
    let mut data = vec![0u8; 512];

    // DOS Header
    let dos_magic = DOS_MAGIC.to_le_bytes();
    data[0] = dos_magic[0];
    data[1] = dos_magic[1];

    // e_lfanew at offset 60 -> point to PE header at offset 128
    let pe_offset = 128u32.to_le_bytes();
    data[60..64].copy_from_slice(&pe_offset);

    // PE Signature at offset 128 ("PE\0\0")
    let pe_sig = PE_SIGNATURE.to_le_bytes();
    data[128..132].copy_from_slice(&pe_sig);

    // COFF Header at offset 132
    // Machine = x86_64 (0x8664)
    data[132..134].copy_from_slice(&0x8664u16.to_le_bytes());
    // NumberOfSections = 1
    data[134..136].copy_from_slice(&1u16.to_le_bytes());
    // SizeOfOptionalHeader = 112 (0x70)
    data[148..150].copy_from_slice(&112u16.to_le_bytes());

    // Optional Header at offset 152
    // Magic = PE32+ (0x020B)
    data[152..154].copy_from_slice(&PE32_PLUS_MAGIC.to_le_bytes());
    // AddressOfEntryPoint = 0x1000 (offset 16 from opt_offset -> 152 + 16 = 168)
    data[168..172].copy_from_slice(&0x1000u32.to_le_bytes());
    // ImageBase = 0x400000 (offset 24 from opt_offset -> 152 + 24 = 176)
    data[176..184].copy_from_slice(&0x400000u64.to_le_bytes());
    // SizeOfImage = 0x8000 (offset 56 from opt_offset -> 152 + 56 = 208)
    data[208..212].copy_from_slice(&0x8000u32.to_le_bytes());

    data
}

#[test]
fn test_pe32_plus_valid_parse() {
    let data = create_valid_pe32_plus_binary();
    let pe = PeBinary::parse(&data).unwrap();

    assert_eq!(pe.pe_offset(), 128);
    assert_eq!(pe.machine(), 0x8664);
    assert_eq!(pe.entry_point_rva(), 0x1000);
    assert_eq!(pe.image_base(), 0x400000);
    assert_eq!(pe.size_of_image(), 0x8000);
    assert_eq!(pe.raw_bytes().len(), data.len());
}

#[test]
fn test_pe_truncated_rejection() {
    let mut data = create_valid_pe32_plus_binary();
    data.truncate(40);
    assert_eq!(
        PeBinary::parse(&data).unwrap_err(),
        BootError::ImageTruncated
    );
}

#[test]
fn test_pe_bad_dos_magic_rejection() {
    let mut data = create_valid_pe32_plus_binary();
    data[0] = 0;
    data[1] = 0;
    assert_eq!(
        PeBinary::parse(&data).unwrap_err(),
        BootError::InvalidElfMagic
    );
}

#[test]
fn test_pe_bad_pe_signature_rejection() {
    let mut data = create_valid_pe32_plus_binary();
    data[128] = 0;
    assert_eq!(
        PeBinary::parse(&data).unwrap_err(),
        BootError::InvalidElfMagic
    );
}

#[test]
fn test_pe32_32bit_valid_parse() {
    let mut data = vec![0u8; 512];
    data[0] = 0x4D;
    data[1] = 0x5A; // MZ
    data[60..64].copy_from_slice(&64u32.to_le_bytes()); // pe_offset = 64

    data[64..68].copy_from_slice(&PE_SIGNATURE.to_le_bytes());
    // Machine = i386 (0x014C)
    data[68..70].copy_from_slice(&0x014Cu16.to_le_bytes());
    // SizeOfOptionalHeader = 96
    data[84..86].copy_from_slice(&96u16.to_le_bytes());

    // Optional header at 88
    data[88..90].copy_from_slice(&PE32_MAGIC.to_le_bytes()); // PE32
    data[104..108].copy_from_slice(&0x2000u32.to_le_bytes()); // Entry point
    data[116..120].copy_from_slice(&0x400000u32.to_le_bytes()); // ImageBase 32-bit
    data[144..148].copy_from_slice(&0x6000u32.to_le_bytes()); // SizeOfImage

    let pe = PeBinary::parse(&data).unwrap();
    assert_eq!(pe.machine(), 0x014C);
    assert_eq!(pe.entry_point_rva(), 0x2000);
    assert_eq!(pe.image_base(), 0x400000);
    assert_eq!(pe.size_of_image(), 0x6000);
}
