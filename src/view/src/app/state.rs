//! Application state structures
//!
//! This module contains the core application state structures.

/// 多文件加载进度
#[derive(Clone, Debug, Default)]
pub struct LoadingProgress {
    pub total_files: usize,
    pub completed_files: usize,
    pub current_file_name: Option<String>,
    pub total_messages_so_far: usize,
    pub is_cancelled: bool,
}

/// Simple deprecated input state for backward compatibility
#[derive(Clone, Debug, Default)]
pub struct SimpleDeprecatedInputState {
    pub text: String,
    pub cursor_position: usize,
}

/// Runtime state that needs to be preserved across window operations
/// This data is NOT stored in config files but needs to survive window recreate
pub struct RuntimeState {
    pub current_view: AppView,
    pub files: Vec<std::sync::Arc<crate::domain::multi_file::FileSegment>>,
    pub merged: crate::domain::multi_file::MergedView,
    pub current_file_name: Option<String>,
    pub plot_data: std::sync::Arc<[crate::models::Series]>,
    /// Full (unfiltered) decoded data cache — used for fast zoom-only re-filter
    pub plot_full_data: std::sync::Arc<[crate::models::Series]>,
    pub plot_zoom_start: Option<f64>,
    pub plot_zoom_end: Option<f64>,
    pub plot_full_time_min: Option<f64>,
    pub plot_full_time_max: Option<f64>,
    pub show_plot_points: bool,
    pub selected_signals: Vec<String>,
    pub dbc_channels: HashMap<u16, DbcDatabase>,
    pub ldf_channels: HashMap<u16, LdfDatabase>,
    pub start_time: Option<chrono::NaiveDateTime>,
    pub active_library_id: Option<String>,
    pub active_version_name: Option<String>,
    pub expanded_channels: std::collections::HashSet<u16>,
    pub expanded_messages: std::collections::HashSet<(u16, u32)>,
}

use blf::LogObject;
use gpui::{Bounds, Entity, Pixels, UniformListScrollHandle};
use parser::dbc::DbcDatabase;
use parser::ldf::LdfDatabase;
use std::collections::HashMap;
use std::path::PathBuf;

// Import AppConfig and ChannelMapping from crate root (defined in main.rs)
use crate::{AppConfig, ChannelType};

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
    LibraryView,
    PlotView,
}

/// State for tracking scrollbar drag operation
#[derive(Clone)]
pub struct ScrollbarDragState {
    pub start_y: Pixels,
    pub start_scroll_offset: f32,
    pub filtered_count: usize, // Number of filtered messages at drag start
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
    pub plot_data: std::sync::Arc<[crate::models::Series]>,
    /// Full (unfiltered) decoded series — zoom only re-filters this, never re-decodes messages
    pub plot_full_data: std::sync::Arc<[crate::models::Series]>,

    // Configuration
    pub config_dir: Option<PathBuf>,
    pub config_file_path: Option<PathBuf>,

    // Signal library local storage
    pub signal_storage: Option<crate::library::SignalLibraryStorage>,

    // Window state
    pub is_maximized: bool,
    // Currently loaded BLF file name (for StatusBar display)
    pub current_file_name: Option<String>,
    // 多文件加载状态
    pub files: Vec<std::sync::Arc<crate::domain::multi_file::FileSegment>>,
    pub merged: crate::domain::multi_file::MergedView,
    pub show_files_popover: bool,
    pub loading_progress: Option<crate::app::state::LoadingProgress>,
    // Library picker UI state (not persisted to config)
    pub library_picker_dismissed: bool,
    pub library_picker_selected_version: std::collections::HashMap<String, String>,
    // BLF file size and parser-consumed bytes (for StatusBar progress)
    pub blf_bytes_total: u64,
    pub blf_bytes_consumed: u64,
    // BLF parse errors (full Display strings, shown in the details popover)
    pub blf_parse_errors: Vec<String>,
    // Controls the BLF errors details popover in the StatusBar
    pub show_blf_errors_popover: bool,
    // Drag-and-drop state
    /// Validated BLF paths waiting for an in-flight load to finish.
    /// Drained by the render tick when `loading_progress` is None.
    pub pending_drop_paths: Vec<std::path::PathBuf>,
    pub saved_window_bounds: Option<Bounds<Pixels>>,
    pub display_bounds: Option<Bounds<Pixels>>,

    // Scroll state
    pub list_scroll_handle: UniformListScrollHandle,
    pub scrollbar_drag_state: Option<ScrollbarDragState>,
    pub scroll_offset: Pixels,
    pub list_container_height: f32,

    // Display settings
    pub id_display_decimal: bool, // true for decimal, false for hexadecimal

    // ID filter
    pub id_filter: Option<u32>,
    pub id_filter_text: gpui::SharedString,
    pub show_id_filter_input: bool,

    // Filter dropdown state
    pub filter_scroll_offset: Pixels,
    pub filter_scroll_handle: UniformListScrollHandle,
    pub mouse_over_filter_dropdown: bool,
    pub mouse_down_on_filter_dropdown: bool,
    pub dropdown_just_opened: bool,

    // Channel filter
    pub channel_filter: Option<u16>,
    pub channel_filter_text: gpui::SharedString,
    pub show_channel_filter_input: bool,
    pub channel_filter_scroll_offset: Pixels,
    pub channel_filter_scroll_handle: UniformListScrollHandle,

    // Signal filter
    pub signal_filter_text: gpui::SharedString,
    pub signal_search_input: Option<Entity<InputState>>,
    pub signal_scroll_handle: UniformListScrollHandle,

    // Status message
    pub status_msg: gpui::SharedString,

    // Library management
    pub library_manager: LibraryManager,
    pub selected_library_id: Option<String>,
    pub selected_version_id: Option<String>, // Add selected version ID
    /// The currently "activated" library version used for log decoding and plot
    pub active_library_id: Option<String>,
    pub active_version_name: Option<String>,
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

    // Rename inline state
    pub renaming_library_id: Option<String>,
    pub renaming_version_name: Option<String>,
    pub rename_library_input: Option<Entity<InputState>>,
    pub rename_version_input: Option<Entity<InputState>>,
    pub rename_library_text: String,
    pub rename_version_text: String,

    // Channel configuration dialog state
    pub show_channel_config_dialog: bool,
    pub new_channel_id: String,
    pub new_channel_name: String,
    pub new_channel_db_path: String,
    pub editing_channel_index: Option<usize>, // None for adding new, Some(index) for editing
    pub channel_id_input: Option<Entity<InputState>>,
    pub channel_name_input: Option<Entity<InputState>>,
    pub show_add_channel_input: bool, // Controls inline input display in channel list
    pub channel_db_path_input: Option<Entity<InputState>>, // For database path input
    pub new_channel_type: ChannelType, // Store selected channel type (CAN/LIN)
    pub pending_file_path: Option<std::sync::mpsc::Receiver<Option<String>>>, // For file dialog result

    // Deprecated: These fields are kept for backward compatibility during migration
    #[deprecated(note = "Use library_name_input instead")]
    pub focused_library_input: Option<String>,
    #[deprecated(note = "Use library_name_input instead")]
    pub is_editing_library_name: bool,
    #[deprecated(note = "Use library_name_input instead")]
    pub library_input_state: SimpleDeprecatedInputState,
    #[deprecated(note = "Not needed with gpui-component Input")]
    pub library_focus_handle: Option<gpui::FocusHandle>,
    #[deprecated(note = "Not needed with gpui-component Input")]
    pub ime_handler_registered: bool,
    // Plot zoom state
    pub plot_zoom_start: Option<f64>,
    pub plot_zoom_end: Option<f64>,
    pub plot_full_time_min: Option<f64>,
    pub plot_full_time_max: Option<f64>,
    pub is_dragging_zoom: bool,
    pub zoom_drag_start_x: Option<Pixels>,
    pub zoom_drag_current_x: Option<Pixels>,

    // Plot display settings
    pub show_plot_points: bool,

    // Plot interaction state
    pub hover_point: Option<HoverPoint>,
    pub plot_hover_time: Option<f64>,
    pub plot_hover_x: Option<Pixels>,
    pub plot_width_px: Pixels,

    // File menu dropdown state
    pub show_file_menu: bool,
    // Help menu dropdown state
    pub show_help_menu: bool,

    // Selected row index (for row-highlight on click-to-copy)
    pub selected_row_index: Option<usize>,

    // Plot sidebar fold state (session-only, not persisted to disk)
    pub expanded_channels: std::collections::HashSet<u16>,
    pub expanded_messages: std::collections::HashSet<(u16, u32)>, // (ch_id, msg_id)

    // Add-channel form Enter-key focus chain: set by PressEnter subscribe,
    // consumed by render() which has window access.
    pub pending_add_channel_focus: Option<PendingAddChannelFocus>,

    // Server state
    pub server_handle: Option<crate::server::ServerHandle>,
    pub show_share_dialog: bool,
    pub share_url_copied: bool,
    pub copied_channel_id: Option<u16>,
    pub show_import_dialog: bool,
    pub import_url: String,
    pub import_status: Option<String>,
    pub import_url_input: Option<Entity<InputState>>,
    pub pending_import: Option<std::sync::mpsc::Receiver<Result<Vec<crate::models::SignalLibrary>, String>>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HoverPoint {
    pub time: f64,
    pub value: f64,
    pub x_px: Pixels,
    pub y_px: Pixels,
    pub series_name: String,
}

/// Which input to focus next when the user presses Enter in the add-channel form.
/// Set by `InputEvent::PressEnter` subscriptions (no window access there),
/// consumed by `render()` which has window access.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PendingAddChannelFocus {
    /// Focus the channel name input.
    ChannelName,
    /// Focus the ✓ Confirm button.
    ChannelConfirm,
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
        Self::new_with_maximized_state(false)
    }

    /// Create a new CanViewApp instance with specified maximize state
    pub fn new_with_maximized_state(is_maximized: bool) -> Self {
        Self::new_with_maximized_state_and_bounds(is_maximized, None)
    }

    /// Create a new CanViewApp instance with maximize state and saved bounds
    pub fn new_with_maximized_state_and_bounds(is_maximized: bool, saved_window_bounds: Option<Bounds<Pixels>>) -> Self {
        Self {
            current_view: AppView::LogView, // Force LogView to prevent chart/library crashes
            messages: Vec::new(),
            status_msg: gpui::SharedString::from(""),
            dbc_channels: HashMap::new(),
            ldf_channels: HashMap::new(),
            app_config: AppConfig::default(),
            selected_signals: Vec::new(),
            start_time: None,
            plot_data: std::sync::Arc::from([]),
            plot_full_data: std::sync::Arc::from([]),
            config_dir: None,
            config_file_path: None,
            signal_storage: crate::library::SignalLibraryStorage::new().ok(),
            is_maximized,
            current_file_name: None,
            files: Vec::new(),
            merged: crate::domain::multi_file::MergedView::empty(),
            show_files_popover: false,
            loading_progress: None,
            library_picker_dismissed: false,
            library_picker_selected_version: std::collections::HashMap::new(),
            blf_bytes_total: 0,
            blf_bytes_consumed: 0,
            blf_parse_errors: Vec::new(),
            show_blf_errors_popover: false,
            pending_drop_paths: Vec::new(),
            saved_window_bounds,
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
            mouse_down_on_filter_dropdown: false,
            dropdown_just_opened: false,
            channel_filter: None,
            channel_filter_text: gpui::SharedString::from(""),
            show_channel_filter_input: false,
            channel_filter_scroll_offset: gpui::px(0.0),
            channel_filter_scroll_handle: UniformListScrollHandle::new(),
            signal_filter_text: gpui::SharedString::from(""),
            library_manager: LibraryManager::new(),
            selected_library_id: None,
            selected_version_id: None, // Initialize selected version ID
            active_library_id: None,
            active_version_name: None,
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
            signal_search_input: None,
            signal_scroll_handle: UniformListScrollHandle::new(),
            // gpui-component input support
            library_name_input: None, // Will be initialized when cx is available
            version_name_input: None, // Will be initialized when cx is available
            // Rename inline state
            renaming_library_id: None,
            renaming_version_name: None,
            rename_library_input: None,
            rename_version_input: None,
            rename_library_text: String::new(),
            rename_version_text: String::new(),
            // Channel configuration dialog
            show_channel_config_dialog: false,
            new_channel_id: String::new(),
            new_channel_name: String::new(),
            new_channel_db_path: String::new(),
            editing_channel_index: None,
            channel_id_input: None, // Will be initialized when cx is available
            channel_name_input: None, // Will be initialized when cx is available
            show_add_channel_input: false,
            channel_db_path_input: None, // Will be initialized when cx is available
            new_channel_type: ChannelType::CAN, // Default to CAN
            pending_file_path: None,     // For file dialog result
            // Deprecated fields for backward compatibility
            focused_library_input: None,
            is_editing_library_name: false,
            library_input_state: SimpleDeprecatedInputState::default(),
            library_focus_handle: None,
            ime_handler_registered: false,
            // Plot zoom state
            plot_zoom_start: None,
            plot_zoom_end: None,
            plot_full_time_min: None,
            plot_full_time_max: None,
            is_dragging_zoom: false,
            zoom_drag_start_x: None,
            zoom_drag_current_x: None,
            // Plot display settings
            show_plot_points: true,
            hover_point: None,
            plot_hover_time: None,
            plot_hover_x: None,
            plot_width_px: gpui::px(0.0),
            // File menu dropdown state
            show_file_menu: false,
            // Help menu dropdown state
            show_help_menu: false,
            selected_row_index: None,
            expanded_channels: std::collections::HashSet::new(),
            expanded_messages: std::collections::HashSet::new(),
            pending_add_channel_focus: None,
            // Server state
            server_handle: None,
            show_share_dialog: false,
            share_url_copied: false,
            copied_channel_id: None,
            show_import_dialog: false,
            import_url: String::new(),
            import_status: None,
            import_url_input: None,
            pending_import: None,
        }
    }

    /// Save runtime state that needs to be preserved across window operations
    /// This includes loaded data and messages
    pub fn save_runtime_state(&self) -> RuntimeState {
        eprintln!("💾 Saving runtime state: {:?} view, {} files, {} plot series, zoom: {:?}-{:?}, {} signals, {} DBC, {} LDF",
            self.current_view, self.files.len(), self.plot_data.len(),
            self.plot_zoom_start, self.plot_zoom_end,
            self.selected_signals.len(),
            self.dbc_channels.len(), self.ldf_channels.len());
        RuntimeState {
            current_view: self.current_view.clone(),
            files: self.files.clone(),
            merged: self.merged.clone(),
            current_file_name: self.current_file_name.clone(),
            plot_data: self.plot_data.clone(),
            plot_full_data: self.plot_full_data.clone(),
            plot_zoom_start: self.plot_zoom_start,
            plot_zoom_end: self.plot_zoom_end,
            plot_full_time_min: self.plot_full_time_min,
            plot_full_time_max: self.plot_full_time_max,
            show_plot_points: self.show_plot_points,
            selected_signals: self.selected_signals.clone(),
            dbc_channels: self.dbc_channels.clone(),
            ldf_channels: self.ldf_channels.clone(),
            start_time: self.start_time,
            active_library_id: self.active_library_id.clone(),
            active_version_name: self.active_version_name.clone(),
            expanded_channels: self.expanded_channels.clone(),
            expanded_messages: self.expanded_messages.clone(),
        }
    }

    /// Restore runtime state after window operations
    /// This preserves the loaded configuration and messages when maximizing/restoring windows
    pub fn restore_runtime_state(&mut self, state: RuntimeState) {
        eprintln!("♻️  Restoring runtime state: {:?} view, {} files, {} plot series, zoom: {:?}-{:?}, {} signals, {} DBC, {} LDF",
            state.current_view,
            state.files.len(),
            state.plot_data.len(),
            state.plot_zoom_start, state.plot_zoom_end,
            state.selected_signals.len(),
            state.dbc_channels.len(), state.ldf_channels.len());
        self.current_view = state.current_view;
        self.files = state.files;
        // 兼容字段：从 merged.messages 派生 messages 快照（必须在 move 之前读取）
        self.messages = state.merged.messages.to_vec();
        self.merged = state.merged;
        self.current_file_name = state.current_file_name;
        self.plot_data = state.plot_data;
        self.plot_full_data = state.plot_full_data;
        self.plot_zoom_start = state.plot_zoom_start;
        self.plot_zoom_end = state.plot_zoom_end;
        self.plot_full_time_min = state.plot_full_time_min;
        self.plot_full_time_max = state.plot_full_time_max;
        self.show_plot_points = state.show_plot_points;
        // plot_width_px will be recalculated based on new window size
        self.plot_width_px = gpui::px(0.0);
        self.selected_signals = state.selected_signals;
        self.dbc_channels = state.dbc_channels;
        self.ldf_channels = state.ldf_channels;
        self.start_time = state.start_time;
        self.active_library_id = state.active_library_id;
        self.active_version_name = state.active_version_name;
        self.expanded_channels = state.expanded_channels;
        self.expanded_messages = state.expanded_messages;
        eprintln!("✅ State restored. Now have: {:?} view, {} files, {} messages, {} plot series",
            self.current_view, self.files.len(), self.messages.len(), self.plot_data.len());
    }
}
