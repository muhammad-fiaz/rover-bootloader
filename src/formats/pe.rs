//! # Native PE/COFF parser
//!
//! Zero-allocation parser for PE32+ (64-bit) and PE32 (32-bit) UEFI executable images.

use crate::error::BootError;

/// DOS header magic bytes (`MZ`).
pub const DOS_MAGIC: u16 = 0x5A4D;

/// PE signature bytes (`PE\0\0`).
pub const PE_SIGNATURE: u32 = 0x0000_4550;

/// PE32 optional header magic.
pub const PE32_MAGIC: u16 = 0x010B;

/// PE32+ (64-bit) optional header magic.
pub const PE32_PLUS_MAGIC: u16 = 0x020B;

/// A parsed PE/COFF executable binary view.
#[derive(Debug)]
pub struct PeBinary<'a> {
    data: &'a [u8],
    pe_offset: usize,
    machine: u16,
    entry_point: u32,
    image_base: u64,
    size_of_image: u32,
}

impl<'a> PeBinary<'a> {
    /// Parses a PE image from raw bytes.
    ///
    /// # Errors
    ///
    /// Returns [`BootError`] if the image is malformed or truncated.
    pub fn parse(data: &'a [u8]) -> Result<Self, BootError> {
        if data.len() < 64 {
            return Err(BootError::ImageTruncated);
        }

        // Check DOS header.
        let dos_magic = u16::from_le_bytes([data[0], data[1]]);
        if dos_magic != DOS_MAGIC {
            return Err(BootError::InvalidElfMagic);
        }

        // e_lfanew is at offset 0x3C (60).
        let pe_offset = u32::from_le_bytes([data[60], data[61], data[62], data[63]]) as usize;
        if pe_offset + 24 > data.len() {
            return Err(BootError::ImageTruncated);
        }

        // Check PE signature.
        let pe_sig = u32::from_le_bytes([
            data[pe_offset],
            data[pe_offset + 1],
            data[pe_offset + 2],
            data[pe_offset + 3],
        ]);
        if pe_sig != PE_SIGNATURE {
            return Err(BootError::InvalidElfMagic);
        }

        let coff_offset = pe_offset + 4;
        let machine = u16::from_le_bytes([data[coff_offset], data[coff_offset + 1]]);
        let opt_size =
            u16::from_le_bytes([data[coff_offset + 16], data[coff_offset + 17]]) as usize;

        let opt_offset = coff_offset + 20;
        if opt_offset + opt_size > data.len() || opt_size < 28 {
            return Err(BootError::ImageTruncated);
        }

        let opt_magic = u16::from_le_bytes([data[opt_offset], data[opt_offset + 1]]);
        let entry_point = u32::from_le_bytes([
            data[opt_offset + 16],
            data[opt_offset + 17],
            data[opt_offset + 18],
            data[opt_offset + 19],
        ]);

        let (image_base, size_of_image) = if opt_magic == PE32_PLUS_MAGIC {
            if opt_size < 30 {
                return Err(BootError::ImageTruncated);
            }
            let base = u64::from_le_bytes([
                data[opt_offset + 24],
                data[opt_offset + 25],
                data[opt_offset + 26],
                data[opt_offset + 27],
                data[opt_offset + 28],
                data[opt_offset + 29],
                data[opt_offset + 30],
                data[opt_offset + 31],
            ]);
            let size = u32::from_le_bytes([
                data[opt_offset + 56],
                data[opt_offset + 57],
                data[opt_offset + 58],
                data[opt_offset + 59],
            ]);
            (base, size)
        } else if opt_magic == PE32_MAGIC {
            let base = u64::from(u32::from_le_bytes([
                data[opt_offset + 28],
                data[opt_offset + 29],
                data[opt_offset + 30],
                data[opt_offset + 31],
            ]));
            let size = u32::from_le_bytes([
                data[opt_offset + 56],
                data[opt_offset + 57],
                data[opt_offset + 58],
                data[opt_offset + 59],
            ]);
            (base, size)
        } else {
            return Err(BootError::InvalidElfClass);
        };

        Ok(Self {
            data,
            pe_offset,
            machine,
            entry_point,
            image_base,
            size_of_image,
        })
    }

    /// Returns the offset of the PE header from the start of the image.
    #[must_use]
    pub const fn pe_offset(&self) -> usize {
        self.pe_offset
    }

    /// Returns the target machine architecture.
    #[must_use]
    pub const fn machine(&self) -> u16 {
        self.machine
    }

    /// Returns the relative virtual address (RVA) of the entry point.
    #[must_use]
    pub const fn entry_point_rva(&self) -> u32 {
        self.entry_point
    }

    /// Returns the preferred image base.
    #[must_use]
    pub const fn image_base(&self) -> u64 {
        self.image_base
    }

    /// Returns the total virtual size of the image.
    #[must_use]
    pub const fn size_of_image(&self) -> u32 {
        self.size_of_image
    }

    /// Returns the raw data slice.
    #[must_use]
    pub const fn raw_bytes(&self) -> &'a [u8] {
        self.data
    }
}
