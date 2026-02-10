//! Windows-specific window utilities
//!
//! This module provides platform-specific functionality for Windows,
//! such as window positioning and maximization.

#[cfg(target_os = "windows")]
use std::ffi::OsStr;
#[cfg(target_os = "windows")]
use std::os::windows::prelude::*;
#[cfg(target_os = "windows")]
use std::ptr;

#[cfg(target_os = "windows")]
#[repr(C)]
struct RECT {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

#[cfg(target_os = "windows")]
#[repr(C)]
struct POINT {
    x: i32,
    y: i32,
}

#[cfg(target_os = "windows")]
#[link(name = "user32")]
unsafe extern "system" {
    fn GetActiveWindow() -> isize;
    fn SetWindowPos(
        hwnd: isize,
        hwnd_insert_after: isize,
        x: i32,
        y: i32,
        cx: i32,
        cy: i32,
        flags: u32,
    ) -> i32;
    fn ShowWindow(hwnd: isize, n_cmd_show: i32) -> i32;
}

#[cfg(target_os = "windows")]
const SW_MAXIMIZE: i32 = 3;
#[cfg(target_os = "windows")]
const SW_RESTORE: i32 = 9;

#[cfg(target_os = "windows")]
const SWP_NOSIZE: u32 = 0x0001;
#[cfg(target_os = "windows")]
const SWP_NOZORDER: u32 = 0x0004;
#[cfg(target_os = "windows")]
const SWP_NOACTIVATE: u32 = 0x0010;
#[cfg(target_os = "windows")]
const SWP_SHOWWINDOW: u32 = 0x0040;
#[cfg(target_os = "windows")]
const SWP_FRAMECHANGED: u32 = 0x0020;

/// Set window position and size on Windows
///
/// # Arguments
/// * `x` - X coordinate in pixels
/// * `y` - Y coordinate in pixels
/// * `width` - Window width in pixels
/// * `height` - Window height in pixels
///
/// # Safety
/// This function calls Windows API directly and should only be called
/// from the main UI thread.
#[cfg(target_os = "windows")]
pub unsafe fn set_window_position(x: i32, y: i32, width: i32, height: i32) -> Result<(), String> {
    let hwnd = GetActiveWindow();
    if hwnd == 0 {
        return Err("Failed to get active window".to_string());
    }

    let result = SetWindowPos(
        hwnd,
        0, // hwnd_insert_after
        x,
        y,
        width,
        height,
        SWP_NOZORDER | SWP_NOACTIVATE,
    );

    if result == 0 {
        Err("SetWindowPos failed".to_string())
    } else {
        Ok(())
    }
}

/// Maximize the active window on Windows
#[cfg(target_os = "windows")]
pub unsafe fn maximize_window() -> Result<(), String> {
    let hwnd = GetActiveWindow();
    if hwnd == 0 {
        return Err("Failed to get active window".to_string());
    }

    let result = ShowWindow(hwnd, SW_MAXIMIZE);
    if result == 0 {
        Err("ShowWindow maximize failed".to_string())
    } else {
        Ok(())
    }
}

/// Restore the active window from maximized state on Windows
#[cfg(target_os = "windows")]
pub unsafe fn restore_window() -> Result<(), String> {
    let hwnd = GetActiveWindow();
    if hwnd == 0 {
        return Err("Failed to get active window".to_string());
    }

    let result = ShowWindow(hwnd, SW_RESTORE);
    if result == 0 {
        Err("ShowWindow restore failed".to_string())
    } else {
        Ok(())
    }
}

// Stub implementations for non-Windows platforms
#[cfg(not(target_os = "windows"))]
pub fn set_window_position(_x: i32, _y: i32, _width: i32, _height: i32) -> Result<(), String> {
    Err!("Window positioning not supported on this platform".to_string())
}

#[cfg(not(target_os = "windows"))]
pub fn maximize_window() -> Result<(), String> {
    Err!("Window maximize not supported on this platform".to_string())
}

#[cfg(not(target_os = "windows"))]
pub fn restore_window() -> Result<(), String> {
    Err!("Window restore not supported on this platform".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_os = "windows")]
    fn test_set_window_position() {
        // This test requires an active window, so it should be run manually
        // or in an integration test environment
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_unsupported_platform() {
        assert!(set_window_position(0, 0, 800, 600).is_err());
        assert!(maximize_window().is_err());
        assert!(restore_window().is_err());
    }
}
