//! Main application logic

use gpui::*;
use blf::{BlfResult, LogObject};
use parser::dbc::DbcDatabase;
use parser::ldf::LdfDatabase;
use std::collections::HashMap;

use crate::models::*;
use crate::config::*;
use crate::filters::*;

/// Main CanView application structure
pub struct CanViewApp {
    // View state
    pub current_view: AppView,
    pub messages: Vec<LogObject>,
    pub status_msg: SharedString,

    // Database channels
    pub dbc_channels: HashMap<u16, DbcDatabase>,
    pub ldf_channels: HashMap<u16, LdfDatabase>,

    // Configuration
    pub app_config: AppConfig,
    pub selected_signals: Vec<String>,
    pub start_time: Option<chrono::NaiveDateTime>,
    pub config_dir: Option<std::path::PathBuf>,
    pub config_file_path: Option<std::path::PathBuf>,

    // Window state
    pub is_maximized: bool,
    pub is_streaming_mode: bool,
    pub saved_window_bounds: Option<Bounds<Pixels>>,
    pub display_bounds: Option<Bounds<Pixels>>,

    // Scroll state
    pub list_scroll_handle: gpui::UniformListScrollHandle,
    pub scrollbar_drag_state: Option<ScrollbarDragState>,
    pub scroll_offset: gpui::Pixels,
    pub list_container_height: f32,

    // ID filter state
    pub id_display_decimal: bool,
    pub id_filter: Option<u32>,
    pub id_filter_text: SharedString,
    pub show_id_filter_input: bool,
    pub filter_scroll_offset: gpui::Pixels,
    pub filter_scroll_handle: gpui::UniformListScrollHandle,
    pub mouse_over_filter_dropdown: bool,
    pub dropdown_just_opened: bool,

    // Channel filter state
    pub channel_filter: Option<u16>,
    pub channel_filter_text: SharedString,
    pub show_channel_filter_input: bool,
    pub channel_filter_scroll_offset: gpui::Pixels,
    pub channel_filter_scroll_handle: gpui::UniformListScrollHandle,
}

impl CanViewApp {
    /// Create a new CanView application instance
    pub fn new() -> Self {
        let mut app = Self {
            current_view: AppView::LogView,
            messages: Vec::new(),
            status_msg: "Ready".into(),
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
            list_scroll_handle: gpui::UniformListScrollHandle::new(),
            scrollbar_drag_state: None,
            scroll_offset: px(0.0),
            list_container_height: 850.0,
            id_display_decimal: true,
            id_filter: None,
            id_filter_text: "".into(),
            show_id_filter_input: false,
            filter_scroll_offset: px(0.0),
            filter_scroll_handle: gpui::UniformListScrollHandle::new(),
            mouse_over_filter_dropdown: false,
            dropdown_just_opened: false,
            channel_filter: None,
            channel_filter_text: "".into(),
            show_channel_filter_input: false,
            channel_filter_scroll_offset: px(0.0),
            channel_filter_scroll_handle: gpui::UniformListScrollHandle::new(),
        };

        // Load startup config
        app.load_startup_config();

        app
    }

    /// Apply BLF parsing result
    pub fn apply_blf_result(&mut self, result: anyhow::Result<BlfResult>) {
        match result {
            Ok(result) => {
                self.status_msg = format!("Loaded BLF: {} objects", result.objects.len()).into();

                // Parse start time
                let st = result.file_stats.measurement_start_time.clone();
                let date_opt = chrono::NaiveDate::from_ymd_opt(
                    st.year as i32,
                    st.month as u32,
                    st.day as u32
                );
                let time_opt = chrono::NaiveTime::from_hms_milli_opt(
                    st.hour as u32,
                    st.minute as u32,
                    st.second as u32,
                    st.milliseconds as u32,
                );

                if let (Some(date), Some(time)) = (date_opt, time_opt) {
                    self.start_time = Some(chrono::NaiveDateTime::new(date, time));
                } else {
                    self.start_time = None;
                }

                self.messages = result.objects;
            }
            Err(e) => {
                self.status_msg = format!("Error: {:?}", e).into();
            }
        }
    }

    /// Get filtered messages based on current filters
    pub fn get_filtered_messages(&self) -> Vec<LogObject> {
        match (self.id_filter, self.channel_filter) {
            (None, None) => self.messages.clone(),
            (Some(filter_id), None) => filter_by_id(&self.messages, filter_id),
            (None, Some(filter_ch)) => filter_by_channel(&self.messages, filter_ch),
            (Some(filter_id), Some(filter_ch)) => {
                filter_by_id_and_channel(&self.messages, filter_id, filter_ch)
            }
        }
    }

    /// Toggle between decimal and hexadecimal ID display
    pub fn toggle_id_display(&mut self) {
        self.id_display_decimal = !self.id_display_decimal;
        self.status_msg = if self.id_display_decimal {
            "ID display: Decimal".into()
        } else {
            "ID display: Hexadecimal".into()
        };
    }

    /// Update list container height
    pub fn update_list_container_height(&mut self, container_height: f32) {
        // Only update if it changed significantly (more than 10px difference)
        if (container_height - self.list_container_height).abs() > 10.0 {
            self.list_container_height = container_height;
        }
    }
}

impl ConfigManager for CanViewApp {
    fn load_config(&mut self, _cx: &mut Context<Self>) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Config Files", &["json"])
            .pick_file()
        {
            self.status_msg = "Loading config...".into();
            match load_config_from_path(path.clone()) {
                Ok(config) => {
                    self.app_config = config;
                    self.config_dir = path.parent().map(|p| p.to_path_buf());
                    self.config_file_path = Some(path);
                    self.status_msg = "Configuration loaded.".into();
                }
                Err(e) => {
                    self.status_msg = format!("Config load error: {}", e).into();
                }
            }
        }
    }

    fn save_config(&self, cx: &mut Context<Self>) {
        let config_path = std::path::PathBuf::from(config::DEFAULT_CONFIG_FILE);
        match save_config_to_path(&self.app_config, &config_path) {
            Ok(_) => {
                self.status_msg = "Configuration saved.".into();
                cx.notify();
            }
            Err(e) => {
                self.status_msg = format!("Failed to save config: {}", e).into();
            }
        }
    }

    fn load_startup_config(&mut self) {
        let path = std::path::PathBuf::from(config::DEFAULT_CONFIG_FILE);
        if path.exists() {
            self.status_msg = "Found saved config, loading...".into();
            match load_config_from_path(path.clone()) {
                Ok(config) => {
                    self.app_config = config.clone();
                    self.config_dir = Some(
                        path.parent()
                            .unwrap_or(std::path::Path::new("../../../../.."))
                            .to_path_buf(),
                    );
                    self.config_file_path = Some(path);
                    self.status_msg = "Configuration loaded.".into();
                }
                Err(e) => {
                    self.status_msg =
                        format!("Config load error: {}. Using default config.", e).into();
                    self.app_config = AppConfig::default();
                }
            }
        } else {
            self.status_msg = "Ready - GPUI version initialized".into();
        }
    }

    fn import_database_file(&mut self, cx: &mut Context<Self>) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Database Files", &["dbc", "ldf"])
            .pick_file()
        {
            let path_str = path.to_string_lossy().to_string();
            self.status_msg = format!("Loaded database: {}", path_str).into();
            cx.notify();
        }
    }
}
