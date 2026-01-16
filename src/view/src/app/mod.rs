//! Main application logic

use gpui::*;
use blf::{BlfResult, LogObject};
use parser::dbc::DbcDatabase;
use parser::ldf::LdfDatabase;
use std::collections::HashMap;

use crate::models::*;
use crate::config::*;
use crate::filters::*;
use crate::library::LibraryManager;

/// Channel configuration for version creation
#[derive(Debug, Clone)]
pub struct ChannelConfig {
    pub channel_id: String,
    pub channel_name: String,
    pub file_path: String,
}

impl ChannelConfig {
    pub fn new() -> Self {
        Self {
            channel_id: String::new(),
            channel_name: String::new(),
            file_path: String::new(),
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.channel_id.is_empty()
            && !self.channel_name.is_empty()
            && !self.file_path.is_empty()
            && self.channel_id.parse::<u16>().map_or(false, |id| id >= 1 && id <= 255)
    }
}

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

    // Library management state
    pub library_manager: LibraryManager,
    pub selected_library_id: Option<String>,
    pub show_library_dialog: bool,
    pub new_library_name: SharedString,
    pub new_library_type: ChannelType,
    pub new_version_name: SharedString,
    pub new_version_description: SharedString,
    pub new_version_path: SharedString,

    // Channel configuration for version creation
    pub channel_configs: Vec<ChannelConfig>,
    pub show_add_channel_dialog: bool,
    pub new_channel_id: String,
    pub new_channel_name: String,
    pub new_channel_file: String,
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

            // Library management state
            library_manager: LibraryManager::new(),
            selected_library_id: None,
            show_library_dialog: false,
            new_library_name: "".into(),
            new_library_type: ChannelType::CAN,
            new_version_name: "".into(),
            new_version_description: "".into(),
            new_version_path: "".into(),

            // Channel configuration for version creation
            channel_configs: Vec::new(),
            show_add_channel_dialog: false,
            new_channel_id: String::new(),
            new_channel_name: String::new(),
            new_channel_file: String::new(),
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

    // ==================== Library Management Methods ====================

    /// Create a new library
    pub fn create_library(&mut self, name: String, channel_type: ChannelType, cx: &mut Context<Self>) {
        match self.library_manager.create_library(name.clone(), channel_type) {
            Ok(_) => {
                self.status_msg = format!("Created library '{}'", name).into();
                // Select the newly created library
                if let Some(lib) = self.library_manager.libraries().iter().find(|l| l.name == name) {
                    self.selected_library_id = Some(lib.id.clone());
                }
                self.save_library_config(cx);
            }
            Err(e) => {
                self.status_msg = format!("Failed to create library: {}", e).into();
            }
        }
        cx.notify();
    }

    /// Delete a library
    pub fn delete_library(&mut self, library_id: &str, cx: &mut Context<Self>) {
        match self.library_manager.delete_library(library_id, &self.app_config.mappings) {
            Ok(_) => {
                self.status_msg = "Library deleted successfully".into();
                if self.selected_library_id.as_ref() == Some(library_id) {
                    self.selected_library_id = None;
                }
                self.save_library_config(cx);
            }
            Err(e) => {
                self.status_msg = format!("Failed to delete library: {}", e).into();
            }
        }
        cx.notify();
    }

    /// Add a new version to a library
    pub fn add_library_version(
        &mut self,
        library_id: &str,
        name: String,
        path: String,
        description: String,
        cx: &mut Context<Self>
    ) {
        match self.library_manager.add_version(library_id, name.clone(), path.clone(), description.clone()) {
            Ok(_) => {
                // Validate the database
                let validation_msg = match self.library_manager.validate_database(&path) {
                    Ok(validation) => {
                        if validation.is_valid {
                            format!(
                                "Added version {} - {} messages, {} signals",
                                name, validation.message_count, validation.signal_count
                            )
                        } else {
                            format!("Added version {} (with warnings: {:?})", name, validation.error)
                        }
                    }
                    Err(e) => {
                        format!("Added version {} (validation failed: {})", name, e)
                    }
                };
                self.status_msg = validation_msg.into();
                self.save_library_config(cx);
            }
            Err(e) => {
                self.status_msg = format!("Failed to add version: {}", e).into();
            }
        }
        cx.notify();
    }

    /// Delete a version from a library
    pub fn delete_library_version(&mut self, library_id: &str, version_name: &str, cx: &mut Context<Self>) {
        match self.library_manager.remove_version(library_id, version_name, &self.app_config.mappings) {
            Ok(_) => {
                self.status_msg = format!("Deleted version {}", version_name).into();
                self.save_library_config(cx);
            }
            Err(e) => {
                self.status_msg = format!("Failed to delete version: {}", e).into();
            }
        }
        cx.notify();
    }

    /// Load and activate a library version
    pub fn load_library_version(&mut self, library_id: &str, version_name: &str, cx: &mut Context<Self>) {
        // Find library and version
        let library = match self.library_manager.find_library(library_id) {
            Some(lib) => lib,
            None => {
                self.status_msg = "Library not found".into();
                cx.notify();
                return;
            }
        };

        let version = match library.get_version(version_name) {
            Some(ver) => ver,
            None => {
                self.status_msg = "Version not found".into();
                cx.notify();
                return;
            }
        };

        // Load database
        match self.library_manager.load_database(&version.path, library.channel_type) {
            Ok(db) => {
                match db {
                    crate::library::Database::Dbc(dbc_db) => {
                        // Load to all CAN channels
                        for channel in 1..=16u16 {
                            self.dbc_channels.insert(channel, dbc_db.clone());
                        }
                    }
                    crate::library::Database::Ldf(ldf_db) => {
                        // Load to all LIN channels
                        for channel in 1..=16u16 {
                            self.ldf_channels.insert(channel, ldf_db.clone());
                        }
                    }
                }

                // Update active state
                self.app_config.active_library_id = Some(library_id.to_string());
                self.app_config.active_version_name = Some(version_name.to_string());

                self.status_msg = format!(
                    "Loaded {} v{}",
                    library.name,
                    version_name
                ).into();

                self.save_library_config(cx);
            }
            Err(e) => {
                self.status_msg = format!("Failed to load database: {}", e).into();
            }
        }
        cx.notify();
    }

    /// Validate a database file
    pub fn validate_database_file(&mut self, path: String) {
        match self.library_manager.validate_database(&path) {
            Ok(validation) => {
                if validation.is_valid {
                    self.status_msg = format!(
                        "Valid - {} messages, {} signals",
                        validation.message_count,
                        validation.signal_count
                    ).into();
                } else {
                    self.status_msg = format!("Invalid: {:?}", validation.error).into();
                }
            }
            Err(e) => {
                self.status_msg = format!("Validation error: {}", e).into();
            }
        }
    }

    /// Import database file and add as version
    pub fn import_database_as_version(&mut self, cx: &mut Context<Self>) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Database Files", &["dbc", "ldf"])
            .pick_file()
        {
            let path_str = path.to_string_lossy().to_string();

            // If a library is selected, add version directly
            if let Some(lib_id) = &self.selected_library_id {
                use crate::library::extract_version_from_path;
                let version_name = extract_version_from_path(&path);

                self.add_library_version(
                    lib_id,
                    version_name,
                    path_str,
                    "Imported from file".to_string(),
                    cx
                );
            } else {
                self.status_msg = "Please select a library first".into();
                cx.notify();
            }
        }
    }

    /// Save library configuration
    fn save_library_config(&self, cx: &mut Context<Self>) {
        // Sync library manager state to app_config
        self.app_config.libraries = self.library_manager.libraries().to_vec();
        self.save_config(cx);
    }

    /// Load library configuration
    fn load_library_config(&mut self) {
        // Load from app_config to library manager
        self.library_manager = LibraryManager::from_libraries(
            self.app_config.libraries.clone()
        );
    }

    /// Select a library
    pub fn select_library(&mut self, library_id: String, cx: &mut Context<Self>) {
        self.selected_library_id = Some(library_id);
        cx.notify();
    }

    /// Open create library dialog
    pub fn open_create_library_dialog(&mut self, cx: &mut Context<Self>) {
        self.show_library_dialog = true;
        self.new_library_name = "".into();
        self.new_library_type = ChannelType::CAN;
        cx.notify();
    }

    /// Open add version dialog
    pub fn open_add_version_dialog(&mut self, cx: &mut Context<Self>) {
        if self.selected_library_id.is_none() {
            self.status_msg = "Please select a library first".into();
            cx.notify();
            return;
        }

        // Auto-fill version name from current date
        self.new_version_name = format!("v{}", chrono::Utc::now().format("%Y%m%d")).into();
        self.new_version_description = "".into();
        self.new_version_path = "".into();
        self.show_library_dialog = true;
        cx.notify();
    }

    /// Close library dialog
    pub fn close_library_dialog(&mut self, cx: &mut Context<Self>) {
        self.show_library_dialog = false;
        self.new_library_name = "".into();
        self.new_version_description = "".into();
        self.new_version_path = "".into();
        cx.notify();
    }

    /// Browse for database file
    pub fn browse_database_file(&mut self, cx: &mut Context<Self>) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Database Files", &["dbc", "ldf"])
            .pick_file()
        {
            self.new_version_path = path.to_string_lossy().to_string().into();

            // Auto-extract version name from path if empty
            if self.new_version_name.is_empty() {
                use crate::library::extract_version_from_path;
                let version_name = extract_version_from_path(&path);
                self.new_version_name = version_name.into();
            }

            // Validate the file
            self.validate_database_file(self.new_version_name.to_string());
            cx.notify();
        }
    }

    // ==================== Channel Configuration Methods ====================

    /// Open add channel dialog
    pub fn open_add_channel_dialog(&mut self, cx: &mut Context<Self>) {
        self.show_add_channel_dialog = true;
        self.new_channel_id = String::new();
        self.new_channel_name = String::new();
        self.new_channel_file = String::new();
        cx.notify();
    }

    /// Close add channel dialog
    pub fn close_add_channel_dialog(&mut self, cx: &mut Context<Self>) {
        self.show_add_channel_dialog = false;
        self.new_channel_id.clear();
        self.new_channel_name.clear();
        self.new_channel_file.clear();
        cx.notify();
    }

    /// Browse for channel database file
    pub fn browse_channel_database_file(&mut self, cx: &mut Context<Self>) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Database Files", &["dbc", "ldf"])
            .pick_file()
        {
            self.new_channel_file = path.to_string_lossy().to_string();

            // Auto-fill channel name if empty
            if self.new_channel_name.is_empty() {
                if let Some(file_name) = path.file_stem() {
                    self.new_channel_name = file_name.to_string_lossy().to_string();
                }
            }

            cx.notify();
        }
    }

    /// Add channel configuration
    pub fn add_channel_config(&mut self, cx: &mut Context<Self>) {
        // Validate inputs
        if self.new_channel_id.is_empty() {
            self.status_msg = "Channel ID is required".into();
            cx.notify();
            return;
        }

        let channel_id = match self.new_channel_id.parse::<u16>() {
            Ok(id) if id >= 1 && id <= 255 => id,
            _ => {
                self.status_msg = "Invalid channel ID. Must be between 1 and 255".into();
                cx.notify();
                return;
            }
        };

        if self.new_channel_name.trim().is_empty() {
            self.status_msg = "Channel name is required".into();
            cx.notify();
            return;
        }

        if self.new_channel_file.trim().is_empty() {
            self.status_msg = "Database file is required".into();
            cx.notify();
            return;
        }

        // Check for duplicate channel ID
        if self.channel_configs.iter().any(|c| c.channel_id == self.new_channel_id) {
            self.status_msg = format!("Channel ID {} already exists", self.new_channel_id).into();
            cx.notify();
            return;
        }

        // Check file type matches library type
        let file_type = std::path::Path::new(&self.new_channel_file)
            .extension()
            .and_then(|e| e.to_str())
            .and_then(|e| DatabaseType::from_extension(e));

        let expected_type = if self.new_library_type == ChannelType::CAN {
            DatabaseType::DBC
        } else {
            DatabaseType::LDF
        };

        if file_type != Some(expected_type) {
            self.status_msg = format!(
                "File type mismatch. Expected {:?}, got {:?}",
                expected_type, file_type
            ).into();
            cx.notify();
            return;
        }

        // Add channel configuration
        let config = ChannelConfig {
            channel_id: self.new_channel_id.clone(),
            channel_name: self.new_channel_name.clone(),
            file_path: self.new_channel_file.clone(),
        };

        self.channel_configs.push(config);
        self.status_msg = format!("Added channel {} ({})", self.new_channel_id, self.new_channel_name).into();

        // Clear inputs
        self.new_channel_id.clear();
        self.new_channel_name.clear();
        self.new_channel_file.clear();

        cx.notify();
    }

    /// Remove channel configuration
    pub fn remove_channel_config(&mut self, index: usize, cx: &mut Context<Self>) {
        if index < self.channel_configs.len() {
            let removed = self.channel_configs.remove(index);
            self.status_msg = format!("Removed channel {}", removed.channel_id).into();
            cx.notify();
        }
    }

    /// Add version with channel configurations
    pub fn add_version_with_channels(&mut self, cx: &mut Context<Self>) {
        if let Some(lib_id) = &self.selected_library_id {
            // Validate inputs
            let version_name = self.new_version_name.to_string();
            if version_name.is_empty() {
                self.status_msg = "Version name is required".into();
                cx.notify();
                return;
            }

            if self.channel_configs.is_empty() {
                self.status_msg = "At least one channel is required".into();
                cx.notify();
                return;
            }

            // Create ChannelDatabase list
            let channel_dbs: Vec<ChannelDatabase> = self.channel_configs
                .iter()
                .filter_map(|config| {
                    let channel_id = config.channel_id.parse::<u16>().ok()?;
                    Some(ChannelDatabase::new(
                        channel_id,
                        config.channel_name.clone(),
                        config.file_path.clone(),
                    ))
                })
                .collect();

            // Add version
            match self.library_manager.add_version_with_channels(
                lib_id,
                version_name.clone(),
                self.new_version_description.to_string(),
                channel_dbs,
            ) {
                Ok(_) => {
                    self.status_msg = format!("Added version {} with {} channels", version_name, self.channel_configs.len()).into();
                    self.save_library_config(cx);

                    // Clear dialog state
                    self.close_add_channel_dialog(cx);
                    self.channel_configs.clear();
                }
                Err(e) => {
                    self.status_msg = format!("Failed to add version: {}", e).into();
                }
            }

            cx.notify();
        } else {
            self.status_msg = "Please select a library first".into();
            cx.notify();
        }
    }

    /// Clear channel configurations
    pub fn clear_channel_configs(&mut self, cx: &mut Context<Self>) {
        self.channel_configs.clear();
        self.status_msg = "Cleared all channel configurations".into();
        cx.notify();
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

                    // Load library configuration
                    self.load_library_config();

                    // Auto-activate library version if specified
                    if let (Some(lib_id), Some(ver_name)) =
                        (&self.app_config.active_library_id, &self.app_config.active_version_name)
                    {
                        // Note: We can't call cx.notify here as we don't have cx
                        // Just update the channels
                        self.load_library_version_silent(lib_id, ver_name);
                    }
                }
                Err(e) => {
                    self.status_msg =
                        format!("Config load error: {}. Using default config.", e).into();
                    self.app_config = AppConfig::default();
                    self.load_library_config();
                }
            }
        } else {
            self.status_msg = "Ready - GPUI version initialized".into();
            self.load_library_config();
        }
    }

    /// Load library version without UI notification (for startup)
    fn load_library_version_silent(&mut self, library_id: &str, version_name: &str) {
        if let Some(library) = self.library_manager.find_library(library_id) {
            if let Some(version) = library.get_version(version_name) {
                if let Ok(db) = self.library_manager.load_database(&version.path, library.channel_type) {
                    match db {
                        crate::library::Database::Dbc(dbc_db) => {
                            for channel in 1..=16u16 {
                                self.dbc_channels.insert(channel, dbc_db.clone());
                            }
                        }
                        crate::library::Database::Ldf(ldf_db) => {
                            for channel in 1..=16u16 {
                                self.ldf_channels.insert(channel, ldf_db.clone());
                            }
                        }
                    }
                }
            }
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
