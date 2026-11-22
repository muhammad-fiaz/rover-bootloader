//! # Rover Bootloader UEFI executable entry point
//!
//! Main entry point for the UEFI bootloader application.

#![cfg_attr(target_os = "uefi", no_std)]
#![cfg_attr(target_os = "uefi", no_main)]

#[cfg(target_os = "uefi")]
use core::fmt::Write;
#[cfg(target_os = "uefi")]
use rover_uefi::base::{EfiHandle, EfiStatus};
#[cfg(target_os = "uefi")]
use rover_uefi::tables::system::EfiSystemTable;

/// UEFI Application entry point.
///
/// Called directly by firmware after loading the bootloader binary into memory.
#[cfg(target_os = "uefi")]
#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "efiapi" fn efi_main(
    image_handle: EfiHandle,
    system_table: *const EfiSystemTable,
) -> EfiStatus {
    // SAFETY: system_table and image_handle are supplied by UEFI firmware per specification.
    let result = unsafe { rover_bootloader::execute_boot_pipeline(image_handle, system_table) };

    match result {
        Ok(()) => EfiStatus::SUCCESS,
        Err(err) => {
            if !system_table.is_null() {
                let st = unsafe { &*system_table };
                if !st.con_out.is_null() {
                    let mut writer = rover_bootloader::ConsoleWriter::new(st.con_out);
                    writer.set_color(12, 0); // Red foreground
                    let _ = write!(writer, "\nRover Bootloader: Fatal boot failure: {err}\n");
                }
            }
            EfiStatus::LOAD_ERROR
        }
    }
}

/// Panic handler for UEFI `#![no_std]` execution.
#[cfg(target_os = "uefi")]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            core::arch::asm!("hlt");
        }
        #[cfg(target_arch = "aarch64")]
        unsafe {
            core::arch::asm!("wfi");
        }
        #[cfg(target_arch = "riscv64")]
        unsafe {
            core::arch::asm!("wfi");
        }
    }
}

/// Fallback entry point for compiling and testing on host targets.
#[cfg(not(target_os = "uefi"))]
fn main() {
    println!("Rover Bootloader library and UEFI executable build.");
}
