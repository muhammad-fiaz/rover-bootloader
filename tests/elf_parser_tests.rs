//! # ELF parser and loader tests
//!
//! Validates ELF64 binary header parsing, program segment extraction, BSS initialization,
//! and error rejection.

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
use rover_bootloader::formats::elf::{
    ELF_MAGIC, ELFCLASS64, ELFDATA2LSB, EM_X86_64, ET_EXEC, EV_CURRENT, Elf64Header,
    Elf64ProgramHeader, ElfBinary, PF_R, PF_W, PF_X, PT_LOAD,
};

fn create_valid_elf64_binary() -> Vec<u8> {
    let mut data = vec![0u8; 256];

    let header = Elf64Header {
        e_ident: [
            ELF_MAGIC[0],
            ELF_MAGIC[1],
            ELF_MAGIC[2],
            ELF_MAGIC[3],
            ELFCLASS64,
            ELFDATA2LSB,
            EV_CURRENT,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ],
        e_type: ET_EXEC,
        e_machine: EM_X86_64,
        e_version: 1,
        e_entry: 0x1000,
        e_phoff: 64,
        e_shoff: 0,
        e_flags: 0,
        e_ehsize: 64,
        e_phentsize: 56,
        e_phnum: 1,
        e_shentsize: 0,
        e_shnum: 0,
        e_shstrndx: 0,
    };

    let phdr = Elf64ProgramHeader {
        p_type: PT_LOAD,
        p_flags: PF_R | PF_W | PF_X,
        p_offset: 120,
        p_vaddr: 0x1000,
        p_paddr: 0x1000,
        p_filesz: 16,
        p_memsz: 32, // 16 bytes of code + 16 bytes of BSS
        p_align: 4096,
    };

    // Write header.
    unsafe {
        core::ptr::write_unaligned(data.as_mut_ptr().cast::<Elf64Header>(), header);
        core::ptr::write_unaligned(data.as_mut_ptr().add(64).cast::<Elf64ProgramHeader>(), phdr);
    }

    // Write segment payload at offset 120.
    for i in 0..16 {
        data[120 + i] = (i + 1) as u8;
    }

    data
}

#[test]
fn test_elf64_valid_parse_and_bounds() {
    let data = create_valid_elf64_binary();
    let elf = ElfBinary::parse(&data).unwrap();

    assert_eq!(elf.entry_point(), 0x1000);
    assert!(elf.validate_architecture(EM_X86_64).is_ok());

    let (min_addr, max_addr) = elf.memory_bounds().unwrap();
    assert_eq!(min_addr, 0x1000);
    assert_eq!(max_addr, 0x1020);
}

#[test]
fn test_elf64_segment_loading_and_bss_clearing() {
    let data = create_valid_elf64_binary();
    let elf = ElfBinary::parse(&data).unwrap();

    let mut memory_dest = vec![0xFFu8; 64];
    elf.load_into_memory(&mut memory_dest, 0x1000).unwrap();

    // Verify first 16 bytes were copied from file.
    for i in 0..16 {
        assert_eq!(memory_dest[i], (i + 1) as u8);
    }

    // Verify next 16 bytes (BSS) were zeroed out.
    for i in 16..32 {
        assert_eq!(memory_dest[i], 0);
    }

    // Verify untouched memory beyond segment memsz remains 0xFF.
    assert_eq!(memory_dest[32], 0xFF);
}

#[test]
fn test_elf64_truncated_rejection() {
    let mut data = create_valid_elf64_binary();
    data.truncate(30);
    assert_eq!(
        ElfBinary::parse(&data).unwrap_err(),
        BootError::ImageTruncated
    );
}

#[test]
fn test_elf64_bad_magic_rejection() {
    let mut data = create_valid_elf64_binary();
    data[0] = 0x00;
    assert_eq!(
        ElfBinary::parse(&data).unwrap_err(),
        BootError::InvalidElfMagic
    );
}

#[test]
fn test_elf64_segment_overflow_rejection() {
    let mut data = create_valid_elf64_binary();
    // Corrupt filesz to exceed file length.
    let phdr = Elf64ProgramHeader {
        p_type: PT_LOAD,
        p_flags: PF_R | PF_W | PF_X,
        p_offset: 120,
        p_vaddr: 0x1000,
        p_paddr: 0x1000,
        p_filesz: 5000,
        p_memsz: 6000,
        p_align: 4096,
    };
    unsafe {
        core::ptr::write_unaligned(data.as_mut_ptr().add(64).cast::<Elf64ProgramHeader>(), phdr);
    }

    assert_eq!(
        ElfBinary::parse(&data).unwrap_err(),
        BootError::SegmentOutOfBounds
    );
}
