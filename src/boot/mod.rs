//! # Boot orchestration subsystem
//!
//! High-level boot flow, interactive menu, and kernel handoff execution.

pub mod menu;
pub mod pipeline;

pub use menu::run_boot_menu;
pub use pipeline::execute_boot_pipeline;
