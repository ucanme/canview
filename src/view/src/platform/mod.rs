//! Platform-specific utilities
//!
//! This module contains platform-specific functionality for window management,
//! file dialogs, and other OS-dependent features.

pub mod windows;
#[cfg(target_os = "macos")]
pub mod macos;

pub use windows::{set_window_position, maximize_window, restore_window};
