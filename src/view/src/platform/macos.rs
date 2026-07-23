//! macOS-specific platform utilities
//!
//! Helpers for the macOS build of the app. The bundled .icns file is
//! consumed by scripts/package-macos.sh at packaging time; the binary
//! itself does not embed the icon. See scripts/make-app-bundle.sh for a
//! quick dev wrapper that creates a .app around a cargo build output.

/// Placeholder — kept so the module has a callable entry point if future
/// runtime icon work is added (cocoa/objc would be required).
pub fn init() {}
