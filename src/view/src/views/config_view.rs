//! Configuration view rendering
//!
//! This module contains the configuration view rendering functionality.
//! The actual implementation is delegated to the library_view module.

use gpui::{Context, IntoElement};
use crate::library_view;
use crate::library::LibraryManager;
use crate::models::ChannelMapping;
use crate::CanViewApp;

/// Render the configuration view
///
/// This function renders the library management interface by delegating
/// to the library_view module's render_library_management_view function.
///
/// # Arguments
/// * `library_manager` - Reference to the library manager
/// * `selected_library_id` - Currently selected library ID
/// * `mappings` - Channel mappings configuration
/// * `new_library_name` - Name for a new library being created
/// * `cursor_position` - Cursor position in the library name input
/// * `versions_expanded` - Whether the versions section is expanded
/// * `show_version_input` - Whether the version input is visible
/// * `new_version_name` - Name for a new version being created
/// * `version_cursor_position` - Cursor position in the version name input
/// * `cx` - GPUI context
///
/// # Returns
/// An element that can be rendered in the UI
pub fn render_config_view(
    library_manager: &LibraryManager,
    selected_library_id: &Option<String>,
    mappings: &[ChannelMapping],
    new_library_name: String,
    cursor_position: usize,
    versions_expanded: bool,
    show_version_input: bool,
    new_version_name: String,
    version_cursor_position: usize,
    cx: &mut Context<CanViewApp>,
) -> impl IntoElement {
    library_view::render_library_management_view(
        library_manager,
        selected_library_id,
        mappings,
        new_library_name,
        cursor_position,
        versions_expanded,
        show_version_input,
        new_version_name,
        version_cursor_position,
        cx
    )
}
