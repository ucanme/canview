//! Platform-specific utilities
//!
//! This module contains platform-specific functionality for window management,
//! file dialogs, and other OS-dependent features.

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "windows")]
pub use windows::{set_window_position, maximize_window, restore_window};

// Stub implementations for non-Windows platforms will be added later
#[cfg(not(target_os = "windows"))]
pub use windows::{set_window_position, maximize_window, restore_window};
