//! # Bootloader memory allocation helpers
//!
//! Page and pool allocation routines wrapping UEFI Boot Services.

use core::ffi::c_void;

use crate::error::BootError;
use rover_uefi::base::{AllocationType, EfiPhysicalAddress, MemoryType};
use rover_uefi::tables::boot::EfiBootServices;

/// Allocates 4 KiB physical memory pages via UEFI Boot Services.
///
/// # Safety
///
/// `bs` must be a valid, non-null pointer to an initialized [`EfiBootServices`].
///
/// # Errors
///
/// Returns [`BootError::MemoryAllocationFailed`] if firmware cannot satisfy the allocation.
pub unsafe fn allocate_pages(
    bs: *mut EfiBootServices,
    alloc_type: AllocationType,
    mem_type: MemoryType,
    pages: usize,
    address: u64,
) -> Result<u64, BootError> {
    if bs.is_null() || pages == 0 {
        return Err(BootError::MemoryAllocationFailed);
    }

    let mut phys_addr = EfiPhysicalAddress::from_u64(address);
    // SAFETY: bs is non-null and points to valid EfiBootServices table.
    let status = unsafe { ((*bs).allocate_pages)(alloc_type, mem_type, pages, &raw mut phys_addr) };

    if !status.is_success() {
        return Err(BootError::MemoryAllocationFailed);
    }

    Ok(phys_addr.to_u64())
}

/// Frees previously allocated 4 KiB memory pages.
///
/// # Safety
///
/// If non-null, `bs` must point to an initialized [`EfiBootServices`].
///
/// # Errors
///
/// Returns [`BootError`] if firmware fails to free the requested range.
pub unsafe fn free_pages(
    bs: *mut EfiBootServices,
    address: u64,
    pages: usize,
) -> Result<(), BootError> {
    if bs.is_null() || pages == 0 {
        return Ok(());
    }

    let phys_addr = EfiPhysicalAddress::from_u64(address);
    // SAFETY: bs is non-null.
    let status = unsafe { ((*bs).free_pages)(phys_addr, pages) };
    if !status.is_success() {
        return Err(BootError::from(status));
    }

    Ok(())
}

/// Allocates a pool memory buffer.
///
/// # Safety
///
/// `bs` must be a valid, non-null pointer to an initialized [`EfiBootServices`].
///
/// # Errors
///
/// Returns [`BootError::MemoryAllocationFailed`] if allocation fails.
pub unsafe fn allocate_pool(
    bs: *mut EfiBootServices,
    mem_type: MemoryType,
    size: usize,
) -> Result<*mut u8, BootError> {
    if bs.is_null() || size == 0 {
        return Err(BootError::MemoryAllocationFailed);
    }

    let mut ptr: *mut c_void = core::ptr::null_mut();
    // SAFETY: bs is valid, ptr is a stack local.
    let status = unsafe { ((*bs).allocate_pool)(mem_type, size, &raw mut ptr) };

    if !status.is_success() || ptr.is_null() {
        return Err(BootError::MemoryAllocationFailed);
    }

    Ok(ptr.cast::<u8>())
}

/// Frees a pool memory buffer.
///
/// # Safety
///
/// If non-null, `bs` must point to an initialized [`EfiBootServices`].
/// If non-null, `buffer` must have been allocated via `allocate_pool`.
///
/// # Errors
///
/// Returns [`BootError`] if freeing the buffer fails.
pub unsafe fn free_pool(bs: *mut EfiBootServices, buffer: *mut u8) -> Result<(), BootError> {
    if bs.is_null() || buffer.is_null() {
        return Ok(());
    }

    // SAFETY: bs and buffer are non-null.
    let status = unsafe { ((*bs).free_pool)(buffer.cast::<c_void>()) };
    if !status.is_success() {
        return Err(BootError::from(status));
    }

    Ok(())
}
