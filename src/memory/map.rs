//! # Memory map processing and normalization
//!
//! Iteration, parsing, and normalization of UEFI firmware memory maps.

use crate::error::BootError;
use crate::protocol::memory::{RoverMemoryDescriptor, RoverMemoryType};
use rover_uefi::base::{EfiMemoryDescriptor, MemoryType};

/// Converts a UEFI [`MemoryType`] into a [`RoverMemoryType`].
#[must_use]
pub fn convert_memory_type(mem_type: MemoryType) -> RoverMemoryType {
    RoverMemoryType::from(mem_type)
}

/// Parses raw UEFI memory map buffer into an array of [`RoverMemoryDescriptor`].
///
/// Returns the number of descriptors written to `dest`.
///
/// # Errors
///
/// Returns [`BootError::MemoryMapBufferTooSmall`] if `dest` has insufficient space.
pub fn parse_and_convert_memory_map(
    raw_buffer: &[u8],
    descriptor_size: usize,
    dest: &mut [RoverMemoryDescriptor],
) -> Result<usize, BootError> {
    if descriptor_size < core::mem::size_of::<EfiMemoryDescriptor>() {
        return Err(BootError::MemoryMapBufferTooSmall);
    }

    let count = raw_buffer.len() / descriptor_size;
    if count > dest.len() {
        return Err(BootError::MemoryMapBufferTooSmall);
    }

    let mut out_idx = 0;
    for i in 0..count {
        let offset = i * descriptor_size;
        // SAFETY: We verified offset + descriptor_size <= raw_buffer.len().
        let raw_desc = unsafe {
            core::ptr::read_unaligned(
                raw_buffer
                    .as_ptr()
                    .add(offset)
                    .cast::<EfiMemoryDescriptor>(),
            )
        };

        let rover_type = convert_memory_type(raw_desc.memory_type);
        dest[out_idx] = RoverMemoryDescriptor::new(
            rover_type,
            raw_desc.physical_start.to_u64(),
            raw_desc.virtual_start.to_u64(),
            raw_desc.number_of_pages,
            raw_desc.attribute,
        );
        out_idx += 1;
    }

    Ok(out_idx)
}

/// Normalizes and coalesces adjacent contiguous memory descriptors of identical type and attributes.
///
/// Returns the new merged descriptor count.
pub fn normalize_memory_map(descriptors: &mut [RoverMemoryDescriptor], count: usize) -> usize {
    if count <= 1 {
        return count;
    }

    // Sort descriptors by physical_start.
    for i in 0..count {
        for j in (i + 1)..count {
            if descriptors[j].physical_start < descriptors[i].physical_start {
                descriptors.swap(i, j);
            }
        }
    }

    let mut write_idx = 0;
    for read_idx in 1..count {
        let current = descriptors[write_idx];
        let next = descriptors[read_idx];

        let current_end = current.physical_end();
        if current.region_type == next.region_type
            && current.attribute == next.attribute
            && current_end == next.physical_start
        {
            // Merge contiguous region.
            descriptors[write_idx].number_of_pages =
                current.number_of_pages.saturating_add(next.number_of_pages);
        } else {
            write_idx += 1;
            descriptors[write_idx] = next;
        }
    }

    write_idx + 1
}
