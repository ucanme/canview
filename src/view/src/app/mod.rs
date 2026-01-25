//! Application module
//!
//! Contains the core application state and implementation.

mod state;
mod impls;

// Re-export the main types
pub use state::{CanViewApp, AppView, ScrollbarDragState, LibraryManager, LibraryDialogType};

// Define actions for text input handling (public, so other modules can use them)
// Note: actions! macro defines the types in the current scope, not in a separate module
gpui::actions!(library_input, [Backspace, Delete, Left, Right, Home, End]);

// Export the context name for use in UI
pub const LIBRARY_INPUT_CONTEXT: &str = "LibraryInput";
