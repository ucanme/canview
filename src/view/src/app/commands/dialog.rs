//! Dialog commands
//!
//! Command handlers for dialog and popup management.

use crate::app::state::LibraryDialogType;

/// Show a library dialog
pub struct ShowLibraryDialog {
    pub dialog_type: LibraryDialogType,
}

impl ShowLibraryDialog {
    pub fn new(dialog_type: LibraryDialogType) -> Self {
        Self { dialog_type }
    }

    /// Create a "Create Library" dialog command
    pub fn create_library() -> Self {
        Self {
            dialog_type: LibraryDialogType::Create,
        }
    }

    /// Create an "Add Version" dialog command
    pub fn add_version() -> Self {
        Self {
            dialog_type: LibraryDialogType::AddVersion,
        }
    }

    /// Create a "Quick Import" dialog command
    pub fn quick_import() -> Self {
        Self {
            dialog_type: LibraryDialogType::QuickImport,
        }
    }
}

/// Hide the library dialog
pub struct HideLibraryDialog;

impl HideLibraryDialog {
    pub fn new() -> Self {
        Self
    }
}

/// Show channel configuration dialog
pub struct ShowChannelConfigDialog {
    pub editing_index: Option<usize>,
}

impl ShowChannelConfigDialog {
    pub fn new(editing_index: Option<usize>) -> Self {
        Self { editing_index }
    }

    /// Create command for adding a new channel
    pub fn add_channel() -> Self {
        Self {
            editing_index: None,
        }
    }

    /// Create command for editing an existing channel
    pub fn edit_channel(index: usize) -> Self {
        Self {
            editing_index: Some(index),
        }
    }
}

/// Hide channel configuration dialog
pub struct HideChannelConfigDialog;

impl HideChannelConfigDialog {
    pub fn new() -> Self {
        Self
    }
}
