//! Application state structures
//!
//! This module contains the core application state structures.

use gpui::{Bounds, Pixels, UniformListScrollHandle, Entity};
use parser::dbc::DbcDatabase;
use parser::ldf::LdfDatabase;
use std::collections::HashMap;
use std::path::PathBuf;
use blf::LogObject;

// Import AppConfig and ChannelMapping from crate root (defined in main.rs)
use crate::{AppConfig, ChannelMapping, ChannelType};

// Import the real LibraryManager from the library module
pub use crate::library::LibraryManager;

// Import DatabaseType for library filtering
use crate::models::library::DatabaseType;

// Import gpui-component input support
use gpui_component::input::InputState;

/// Application view modes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppView {
    LogView,
    ConfigView,
    ChartView,
    LibraryView,
}

/// State for tracking scrollbar drag operation
#[derive(Clone)]
pub struct ScrollbarDragState {
    pub start_y: Pixels,
    pub start_scroll_offset: f32,
    pub filtered_count: usize,  // Number of filtered messages at drag start
}

/// Main application state
pub struct CanViewApp {
    // View state
    pub current_view: AppView,

    // Data
    pub messages: Vec<LogObject>,
    pub dbc_channels: HashMap<u16, DbcDatabase>,
    pub ldf_channels: HashMap<u16, LdfDatabase>,
    pub app_config: AppConfig,
    pub selected_signals: Vec<String>,
    pub start_time: Option<chrono::NaiveDateTime>,

    // Configuration
    pub config_dir: Option<PathBuf>,
    pub config_file_path: Option<PathBuf>,

    // Window state
    pub is_maximized: bool,
    pub is_streaming_mode: bool,
    pub saved_window_bounds: Option<Bounds<Pixels>>,
    pub display_bounds: Option<Bounds<Pixels>>,

    // Scroll state
    pub list_scroll_handle: UniformListScrollHandle,
    pub scrollbar_drag_state: Option<ScrollbarDragState>,
    pub scroll_offset: Pixels,
    pub list_container_height: f32,

    // Display settings
    pub id_display_decimal: bool,  // true for decimal, false for hexadecimal

    // ID filter
    pub id_filter: Option<u32>,
    pub id_filter_text: gpui::SharedString,
    pub show_id_filter_input: bool,

    // Filter dropdown state
    pub filter_scroll_offset: Pixels,
    pub filter_scroll_handle: UniformListScrollHandle,
    pub mouse_over_filter_dropdown: bool,
    pub dropdown_just_opened: bool,

    // Channel filter
    pub channel_filter: Option<u16>,
    pub channel_filter_text: gpui::SharedString,
    pub show_channel_filter_input: bool,
    pub channel_filter_scroll_offset: Pixels,
    pub channel_filter_scroll_handle: UniformListScrollHandle,

    // Status message
    pub status_msg: gpui::SharedString,

    // Library management
    pub library_manager: LibraryManager,
    pub selected_library_id: Option<String>,
    pub selected_version_id: Option<String>,  // Add selected version ID
    pub new_library_name: String,
    pub library_cursor_position: usize,
    pub library_versions_expanded: bool,
    pub show_version_input: bool,
    pub new_version_name: String,
    pub new_version_cursor_position: usize,
    pub show_library_dialog: bool,

    // gpui-component input support for library management
    pub library_name_input: Option<Entity<InputState>>,
    pub version_name_input: Option<Entity<InputState>>,
    pub library_dialog_type: LibraryDialogType,
    pub library_search_query: String,
    pub library_filter_type: Option<DatabaseType>,

    // Channel configuration dialog state
    pub show_channel_config_dialog: bool,
    pub new_channel_id: String,
    pub new_channel_name: String,
    pub new_channel_db_path: String,
    pub editing_channel_index: Option<usize>,  // None for adding new, Some(index) for editing
    pub channel_id_input: Option<Entity<InputState>>,
    pub channel_name_input: Option<Entity<InputState>>,
    pub show_add_channel_input: bool,  // Controls inline input display in channel list
    pub channel_db_path_input: Option<Entity<InputState>>,  // For database path input
    pub new_channel_type: ChannelType,  // Store selected channel type (CAN/LIN)
    pub pending_file_path: Option<std::sync::mpsc::Receiver<Option<String>>>,  // For file dialog result

    // Deprecated: These fields are kept for backward compatibility during migration
    #[deprecated(note = "Use library_name_input instead")]
    pub focused_library_input: Option<String>,
    #[deprecated(note = "Use library_name_input instead")]
    pub is_editing_library_name: bool,
    #[deprecated(note = "Use library_name_input instead")]
    pub library_input_state: crate::ui::components::ime_text_input::ImeTextInputState,
    #[deprecated(note = "Not needed with gpui-component Input")]
    pub library_focus_handle: Option<gpui::FocusHandle>,
    #[deprecated(note = "Not needed with gpui-component Input")]
    pub ime_handler_registered: bool,
}

/// Library dialog type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LibraryDialogType {
    Create,
    AddVersion,
    QuickImport,
}

impl CanViewApp {
    /// Create a new CanViewApp instance with default state
    pub fn new_state() -> Self {
        Self {
            current_view: AppView::LogView,
            messages: Vec::new(),
            status_msg: gpui::SharedString::from(""),
            dbc_channels: HashMap::new(),
            ldf_channels: HashMap::new(),
            app_config: AppConfig::default(),
            selected_signals: Vec::new(),
            start_time: None,
            config_dir: None,
            config_file_path: None,
            is_maximized: false,
            is_streaming_mode: false,
            saved_window_bounds: None,
            display_bounds: None,
            list_scroll_handle: UniformListScrollHandle::new(),
            scrollbar_drag_state: None,
            scroll_offset: gpui::px(0.0),
            list_container_height: 850.0,
            id_display_decimal: false,
            id_filter: None,
            id_filter_text: gpui::SharedString::from(""),
            show_id_filter_input: false,
            filter_scroll_offset: gpui::px(0.0),
            filter_scroll_handle: UniformListScrollHandle::new(),
            mouse_over_filter_dropdown: false,
            dropdown_just_opened: false,
            channel_filter: None,
            channel_filter_text: gpui::SharedString::from(""),
            show_channel_filter_input: false,
            channel_filter_scroll_offset: gpui::px(0.0),
            channel_filter_scroll_handle: UniformListScrollHandle::new(),
            library_manager: LibraryManager::new(),
            selected_library_id: None,
            selected_version_id: None,  // Initialize selected version ID
            new_library_name: String::new(),
            library_cursor_position: 0,
            library_versions_expanded: true,
            show_version_input: false,
            new_version_name: String::new(),
            new_version_cursor_position: 0,
            show_library_dialog: false,
            library_dialog_type: LibraryDialogType::Create,
            library_search_query: String::new(),
            library_filter_type: None,
            // gpui-component input support
            library_name_input: None,  // Will be initialized when cx is available
            version_name_input: None,  // Will be initialized when cx is available
            // Channel configuration dialog
            show_channel_config_dialog: false,
            new_channel_id: String::new(),
            new_channel_name: String::new(),
            new_channel_db_path: String::new(),
            editing_channel_index: None,
            channel_id_input: None,  // Will be initialized when cx is available
            channel_name_input: None,  // Will be initialized when cx is available
            show_add_channel_input: false,
            channel_db_path_input: None,  // Will be initialized when cx is available
            new_channel_type: ChannelType::CAN,  // Default to CAN
            pending_file_path: None,  // For file dialog result
            // Deprecated fields for backward compatibility
            focused_library_input: None,
            is_editing_library_name: false,
            library_input_state: crate::ui::components::ime_text_input::ImeTextInputState::default(),
            library_focus_handle: None,
            ime_handler_registered: false,
        }
    }
}
