//! Application module
//!
//! Contains the core application state and implementation.

mod impls;
mod impls_helper;
mod state;
mod helpers;
mod commands;

// Re-export the main types
pub use state::{AppView, CanViewApp, LibraryDialogType};

// Re-export helper functions
pub use helpers::{
    convert_timestamp_to_seconds,
    format_timestamp_static,
    format_timestamp_with_date,
    format_time_difference,
};

// Re-export commands
pub use commands::{
    navigation::{NavigateToView, ToggleMaximize, UpdateContainerHeight},
    dialog::{ShowLibraryDialog, HideLibraryDialog, ShowChannelConfigDialog, HideChannelConfigDialog},
    config::{SaveConfig, LoadConfig, LoadStartupConfig},
    load::{LoadBlfFile, ImportDatabaseFile, ProcessBlfResult, BlfLoadStats},
    library::{CreateLibrary, DeleteLibrary, AddLibraryVersion, DeleteLibraryVersion, LoadLibraryVersion, ApplyVersionToMappings, LibraryOperationResult, LibraryVersionInfo},
};

// Define actions for text input handling (public, so other modules can use them)
// Note: actions! macro defines the types in the current scope, not in a separate module
gpui::actions!(library_input, [Backspace, Delete, Left, Right, Home, End]);

// Export the context name for use in UI
pub const LIBRARY_INPUT_CONTEXT: &str = "LibraryInput";
