//! # Native ELF parser and loader
//!
//! Robust, zero-allocation parser and loader for ELF64 (and ELF32) executable binaries.

use crate::error::BootError;

/// ELF magic identifier bytes (`\x7fELF`).
pub const ELF_MAGIC: [u8; 4] = [0x7F, b'E', b'L', b'F'];

/// 32-bit ELF class.
pub const ELFCLASS32: u8 = 1;

/// 64-bit ELF class.
pub const ELFCLASS64: u8 = 2;

/// Little-endian 2's complement data encoding.
pub const ELFDATA2LSB: u8 = 1;

/// Current ELF version.
pub const EV_CURRENT: u8 = 1;

/// Executable file type.
pub const ET_EXEC: u16 = 2;

/// Shared object / position-independent executable file type.
pub const ET_DYN: u16 = 3;

/// x86 machine architecture.
pub const EM_386: u16 = 3;

/// ARM machine architecture.
pub const EM_ARM: u16 = 40;

/// `x86_64` machine architecture.
pub const EM_X86_64: u16 = 62;

/// `AArch64` machine architecture.
pub const EM_AARCH64: u16 = 183;

/// RISC-V machine architecture.
pub const EM_RISCV: u16 = 243;

/// `LoongArch` machine architecture.
pub const EM_LOONGARCH: u16 = 258;

/// Loadable segment type.
pub const PT_LOAD: u32 = 1;

/// Segment executable flag.
pub const PF_X: u32 = 1;

/// Segment writable flag.
pub const PF_W: u32 = 2;

/// Segment readable flag.
pub const PF_R: u32 = 4;

/// 64-bit ELF file header.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Elf64Header {
    /// ELF identification bytes.
    pub e_ident: [u8; 16],
    /// Object file type.
    pub e_type: u16,
    /// Required architecture.
    pub e_machine: u16,
    /// Object file version.
    pub e_version: u32,
    /// Entry point virtual address.
    pub e_entry: u64,
    /// Program header table file offset.
    pub e_phoff: u64,
    /// Section header table file offset.
    pub e_shoff: u64,
    /// Processor-specific flags.
    pub e_flags: u32,
    /// ELF header size in bytes.
    pub e_ehsize: u16,
    /// Program header table entry size.
    pub e_phentsize: u16,
    /// Program header table entry count.
    pub e_phnum: u16,
    /// Section header table entry size.
    pub e_shentsize: u16,
    /// Section header table entry count.
    pub e_shnum: u16,
    /// Section header string table index.
    pub e_shstrndx: u16,
}

/// 64-bit ELF program header entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct Elf64ProgramHeader {
    /// Segment type.
    pub p_type: u32,
    /// Segment flags.
    pub p_flags: u32,
    /// Segment file offset.
    pub p_offset: u64,
    /// Segment virtual address.
    pub p_vaddr: u64,
    /// Segment physical address.
    pub p_paddr: u64,
    /// Segment size in file.
    pub p_filesz: u64,
    /// Segment size in memory.
    pub p_memsz: u64,
    /// Segment alignment.
    pub p_align: u64,
}

/// A parsed ELF binary referencing the underlying raw image bytes.
#[derive(Debug)]
pub struct ElfBinary<'a> {
    data: &'a [u8],
    header: Elf64Header,
}

impl<'a> ElfBinary<'a> {
    /// Parses and validates an ELF64 binary from a byte slice.
    ///
    /// # Errors
    ///
    /// Returns a [`BootError`] if the binary is truncated, has invalid magic,
    /// is not 64-bit little-endian, or contains malformed program headers.
    pub fn parse(data: &'a [u8]) -> Result<Self, BootError> {
        if data.len() < core::mem::size_of::<Elf64Header>() {
            return Err(BootError::ImageTruncated);
        }

        // Validate ELF magic.
        if data[0..4] != ELF_MAGIC {
            return Err(BootError::InvalidElfMagic);
        }

        // Validate 64-bit class and little-endian encoding.
        if data[4] != ELFCLASS64 {
            return Err(BootError::InvalidElfClass);
        }
        if data[5] != ELFDATA2LSB {
            return Err(BootError::InvalidElfData);
        }
        if data[6] != EV_CURRENT {
            return Err(BootError::InvalidElfVersion);
        }

        // Read ELF header safely using unaligned reads.
        let header = unsafe { core::ptr::read_unaligned(data.as_ptr().cast::<Elf64Header>()) };

        if header.e_type != ET_EXEC && header.e_type != ET_DYN {
            return Err(BootError::InvalidElfType);
        }

        let ph_size = core::mem::size_of::<Elf64ProgramHeader>();
        if (header.e_phentsize as usize) < ph_size {
            return Err(BootError::InvalidProgramHeader);
        }

        // Check program header table boundary.
        let ph_table_end = header
            .e_phoff
            .checked_add(u64::from(header.e_phnum).saturating_mul(u64::from(header.e_phentsize)))
            .ok_or(BootError::InvalidProgramHeader)?;

        if ph_table_end > data.len() as u64 {
            return Err(BootError::InvalidProgramHeader);
        }

        let binary = Self { data, header };

        // Validate all loadable segments.
        for ph in binary.program_headers() {
            if ph.p_type == PT_LOAD {
                let seg_end = ph
                    .p_offset
                    .checked_add(ph.p_filesz)
                    .ok_or(BootError::SegmentOutOfBounds)?;
                if seg_end > data.len() as u64 {
                    return Err(BootError::SegmentOutOfBounds);
                }
                if ph.p_filesz > ph.p_memsz {
                    return Err(BootError::SegmentOutOfBounds);
                }
            }
        }

        // Validate that the entry point resides in an executable segment.
        let mut entry_valid = false;
        for ph in binary.program_headers() {
            if ph.p_type == PT_LOAD && (ph.p_flags & PF_X) != 0 {
                let start = if ph.p_paddr != 0 {
                    ph.p_paddr
                } else {
                    ph.p_vaddr
                };
                let end = start.saturating_add(ph.p_memsz);
                if header.e_entry >= start && header.e_entry < end {
                    entry_valid = true;
                    break;
                }
            }
        }

        if !entry_valid && header.e_type == ET_EXEC {
            return Err(BootError::InvalidEntryPoint);
        }

        Ok(binary)
    }

    /// Returns a reference to the parsed ELF64 header.
    #[must_use]
    pub const fn header(&self) -> &Elf64Header {
        &self.header
    }

    /// Returns the executable entry point address.
    #[must_use]
    pub const fn entry_point(&self) -> u64 {
        self.header.e_entry
    }

    /// Verifies that the ELF binary matches the specified processor architecture.
    ///
    /// # Errors
    ///
    /// Returns [`BootError::InvalidElfMachine`] if the machine architecture does not match.
    pub fn validate_architecture(&self, expected_machine: u16) -> Result<(), BootError> {
        if self.header.e_machine != expected_machine {
            return Err(BootError::InvalidElfMachine);
        }
        Ok(())
    }

    /// Returns an iterator over the program headers.
    #[allow(clippy::cast_possible_truncation)]
    pub fn program_headers(&self) -> impl Iterator<Item = Elf64ProgramHeader> + '_ {
        let count = self.header.e_phnum as usize;
        let entry_size = self.header.e_phentsize as usize;
        let ph_offset = self.header.e_phoff as usize;

        (0..count).filter_map(move |i| {
            let offset = ph_offset.checked_add(i.checked_mul(entry_size)?)?;
            if offset + core::mem::size_of::<Elf64ProgramHeader>() <= self.data.len() {
                // SAFETY: We verified offset + size <= data.len().
                Some(unsafe {
                    core::ptr::read_unaligned(
                        self.data.as_ptr().add(offset).cast::<Elf64ProgramHeader>(),
                    )
                })
            } else {
                None
            }
        })
    }

    /// Calculates the minimum and maximum physical memory span needed for all `PT_LOAD` segments.
    ///
    /// Returns `(min_addr, max_addr)`.
    ///
    /// # Errors
    ///
    /// Returns [`BootError::InvalidProgramHeader`] if no `PT_LOAD` segments exist.
    pub fn memory_bounds(&self) -> Result<(u64, u64), BootError> {
        let mut min_addr = u64::MAX;
        let mut max_addr = 0u64;
        let mut has_load = false;

        for ph in self.program_headers() {
            if ph.p_type == PT_LOAD {
                has_load = true;
                let addr = if ph.p_paddr != 0 {
                    ph.p_paddr
                } else {
                    ph.p_vaddr
                };
                let end = addr
                    .checked_add(ph.p_memsz)
                    .ok_or(BootError::SegmentOutOfBounds)?;
                if addr < min_addr {
                    min_addr = addr;
                }
                if end > max_addr {
                    max_addr = end;
                }
            }
        }

        if !has_load || min_addr > max_addr {
            return Err(BootError::InvalidProgramHeader);
        }

        Ok((min_addr, max_addr))
    }

    /// Loads all `PT_LOAD` segments into a target physical memory slice.
    ///
    /// Segments are copied from the file image. Any BSS area (`memsz > filesz`) is zero-filled.
    ///
    /// # Errors
    ///
    /// Returns a [`BootError`] if any segment falls outside the target memory slice.
    #[allow(clippy::cast_possible_truncation)]
    pub fn load_into_memory(&self, dest: &mut [u8], base_address: u64) -> Result<(), BootError> {
        for ph in self.program_headers() {
            if ph.p_type != PT_LOAD {
                continue;
            }

            let seg_addr = if ph.p_paddr != 0 {
                ph.p_paddr
            } else {
                ph.p_vaddr
            };

            let rel_offset = seg_addr
                .checked_sub(base_address)
                .ok_or(BootError::SegmentOutOfBounds)? as usize;

            let file_size = ph.p_filesz as usize;
            let mem_size = ph.p_memsz as usize;

            if rel_offset
                .checked_add(mem_size)
                .ok_or(BootError::SegmentOutOfBounds)?
                > dest.len()
            {
                return Err(BootError::SegmentOutOfBounds);
            }

            let file_offset = ph.p_offset as usize;
            if file_offset
                .checked_add(file_size)
                .ok_or(BootError::SegmentOutOfBounds)?
                > self.data.len()
            {
                return Err(BootError::SegmentOutOfBounds);
            }

            // Copy file content.
            if file_size > 0 {
                dest[rel_offset..rel_offset + file_size]
                    .copy_from_slice(&self.data[file_offset..file_offset + file_size]);
            }

            // Zero out BSS section if mem_size > file_size.
            if mem_size > file_size {
                dest[rel_offset + file_size..rel_offset + mem_size].fill(0);
            }
        }

        Ok(())
    }
}
