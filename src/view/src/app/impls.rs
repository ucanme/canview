//! CanViewApp implementation blocks
//!
//! This file contains all impl blocks for CanViewApp.

use super::state::{AppView, CanViewApp, LibraryManager};
use crate::AppConfig;
use crate::ChannelType;
use blf::{BlfResult, LogObject};
use gpui::{prelude::*, *};
use gpui_component::input::InputState;
use parser::dbc::DbcDatabase;
use parser::ldf::LdfDatabase;
use std::collections::HashMap;
use std::path::PathBuf;

impl CanViewApp {
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
            plot_data: std::sync::Arc::from([]),
            plot_full_data: std::sync::Arc::from([]),
            config_dir: None,
            config_file_path: None,
            signal_storage: crate::library::SignalLibraryStorage::new().ok(),
            // Default window/app states
            is_maximized: false,
            is_streaming_mode: false,
            saved_window_bounds: None,
            display_bounds: None,
            // Initialize uniform list scroll handle
            list_scroll_handle: gpui::UniformListScrollHandle::new(),
            // Initialize scrollbar drag state
            scrollbar_drag_state: None,
            // Initialize scroll offset
            scroll_offset: px(0.0),
            // Initialize list container height (will be updated dynamically)
            list_container_height: 850.0,
            // Default to decimal ID display
            id_display_decimal: true,
            // ID filter: None means show all messages
            id_filter: None,
            id_filter_text: "".into(),
            // Hide ID filter input dialog by default
            show_id_filter_input: false,
            // Initialize filter scroll offset
            filter_scroll_offset: px(0.0),
            // Initialize filter scroll handle
            filter_scroll_handle: gpui::UniformListScrollHandle::new(),
            // Initialize mouse tracking
            mouse_over_filter_dropdown: false,
            mouse_down_on_filter_dropdown: false,
            dropdown_just_opened: false,
            // Channel filter
            channel_filter: None,
            channel_filter_text: "".into(),
            show_channel_filter_input: false,
            channel_filter_scroll_offset: px(0.0),
            channel_filter_scroll_handle: gpui::UniformListScrollHandle::new(),
            signal_filter_text: "".into(),
            signal_search_input: None,
            signal_scroll_handle: UniformListScrollHandle::new(),
            // Library management
            library_manager: LibraryManager::new(),
            selected_library_id: None,
            selected_version_id: None,
            new_library_name: String::new(),
            library_cursor_position: 0,
            library_versions_expanded: true,
            show_version_input: false,
            new_version_name: String::new(),
            new_version_cursor_position: 0,
            show_library_dialog: false,
            library_dialog_type: super::state::LibraryDialogType::Create,
            library_search_query: String::new(),
            library_filter_type: None,
            // gpui-component input support
            library_name_input: None,
            version_name_input: None,
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
            channel_id_input: None,
            channel_name_input: None,
            show_add_channel_input: false,
            channel_db_path_input: None,
            new_channel_type: ChannelType::CAN,
            pending_file_path: None,
            // Deprecated fields for backward compatibility
            focused_library_input: None,
            is_editing_library_name: false,
            library_input_state: crate::app::state::SimpleDeprecatedInputState::default(),
            library_focus_handle: None,
            ime_handler_registered: false,
            plot_zoom_start: None,
            plot_zoom_end: None,
            plot_full_time_min: None,
            plot_full_time_max: None,
            is_dragging_zoom: false,
            zoom_drag_start_x: None,
            zoom_drag_current_x: None,
            show_plot_points: true,
            hover_point: None,
            plot_hover_time: None,
            plot_hover_x: None,
            plot_width_px: px(0.0),
            // File menu dropdown state
            show_file_menu: false,
            // Server state
            server_handle: None,
            show_share_dialog: false,
            show_import_dialog: false,
            import_url: String::new(),
            import_status: None,
            import_url_input: None,
            pending_import: None,
        };

        // 🔧 启动时加载配置
        app.load_startup_config();

        app
    }

    fn load_startup_config(&mut self) {
        let path = PathBuf::from("multi_channel_config.json");
        if path.exists() {
            self.status_msg = "Found saved config, loading...".into();
            if let Ok(content) = std::fs::read_to_string(&path) {
                match serde_json::from_str::<AppConfig>(&content) {
                    Ok(config) => {
                        // 保存配置
                        self.app_config = config.clone();
                        self.config_dir = Some(
                            path.parent()
                                .unwrap_or(std::path::Path::new("../../../../.."))
                                .to_path_buf(),
                        );
                        self.config_file_path = Some(path);

                        // 🔧 加载信号库
                        if !config.libraries.is_empty() {
                            eprintln!("📚 加载信号库配置...");
                            eprintln!("  找到 {} 个信号库", config.libraries.len());

                            // 将库加载到 library_manager
                            self.library_manager =
                                LibraryManager::from_libraries(config.libraries.clone());

                            // 统计信息并显示
                            let (lib_count, ver_count, chan_count) =
                                Self::display_library_stats(&self.library_manager);

                            self.status_msg = format!(
                                "Configuration loaded: {} libraries, {} versions, {} channels",
                                lib_count, ver_count, chan_count
                            )
                            .into();

                            // 🔄 自动加载每个通道激活的版本
                            for mapping in &config.mappings {
                                if let (Some(lib_id), Some(ver_name)) =
                                    (&mapping.library_id, &mapping.version_name)
                                {
                                    eprintln!(
                                        "  🔄 自动加载通道 {} 的库 {} 版本 {}",
                                        mapping.channel_id, lib_id, ver_name
                                    );
                                    // Actually, we can just load it directly since we don't need UI context for the core load
                                    self.internal_load_library_version(
                                        mapping.channel_id,
                                        lib_id,
                                        ver_name,
                                    );
                                }
                            }
                        } else {
                            self.status_msg =
                                "Configuration loaded (no libraries configured).".into();
                        }
                    }
                    Err(e) => {
                        self.status_msg =
                            format!("Config load error: {}. Using default config.", e).into();
                        // Initialize with empty config instead of failing
                        self.app_config = AppConfig::default();
                        eprintln!("❌ 配置加载失败: {}", e);
                    }
                }
            }
        } else {
            self.status_msg = "Ready - GPUI version initialized".into();
            eprintln!("ℹ️  未找到配置文件，使用默认配置");
        }
    }

    pub(crate) fn apply_blf_result(&mut self, result: anyhow::Result<BlfResult>) {
        match result {
            Ok(result) => {
                let error_count = result.errors.len();

                // 如果有解析错误，先打印到控制台
                if error_count > 0 {
                    Self::log_blf_errors(&result.errors, result.objects.len());
                }

                // 只有在成功加载后才清空之前的数据
                self.messages.clear();
                self.plot_data = std::sync::Arc::from([]);
                self.plot_full_data = std::sync::Arc::from([]);
                self.selected_signals.clear();

                // 自动切换到数据列表视图
                self.current_view = AppView::LogView;

                // 根据是否有错误设置不同的状态栏消息
                if error_count > 0 {
                    let first_error = &result.errors[0];
                    self.status_msg = format!(
                        "⚠️ Loaded {} messages | {} errors (first: {})",
                        result.objects.len(),
                        error_count,
                        first_error
                    )
                    .into();
                } else {
                    self.status_msg = format!("✅ Loaded {} messages", result.objects.len()).into();
                }

                // === 调试输出：检查时间戳 ===
                println!("基准时间: {:?}", result.file_stats.measurement_start_time);
                Self::print_timestamp_diagnostics(&result.objects);

                // Parse start time with nanosecond precision
                self.start_time = Self::parse_blf_start_time(&result.file_stats.measurement_start_time);

                self.messages = result.objects;
            }
            Err(e) => {
                // 在状态栏显示详细的错误信息（不清空之前的数据）
                self.status_msg = format!("❌ File Error: {}", e).into();

                // 保持当前视图不变，不切换到 LogView
                // 这样用户可以看到之前成功加载的数据

                // 打印详细错误信息到控制台
                Self::display_blf_load_error(&e);
            }
        }
    }

    fn load_config(&mut self, _cx: &mut Context<Self>) {
        // TODO: File dialog integration requires fixing GPUI async lifetime issues on Windows
        self.status_msg =
            "Config loading temporarily unavailable. Please use command-line arguments.".into();
    }

    pub(crate) fn import_database_file(&mut self, _cx: &mut Context<Self>) {
        // TODO: File dialog integration requires fixing GPUI async lifetime issues on Windows
        self.status_msg =
            "Database import temporarily unavailable. Please use library management.".into();
    }
    pub(crate) fn get_timestamp_string(&self, timestamp: u64) -> String {
        if let Some(start) = &self.start_time {
            let msg_time = *start + chrono::Duration::nanoseconds(timestamp as i64);
            // Format: YYYY-MM-DD HH:MM:SS.mmmmmm (microseconds)
            msg_time.format("%Y-%m-%d %H:%M:%S%.6f").to_string()
        } else {
            // If no start time, show nanoseconds as seconds with microsecond precision
            format!("{:.6}", timestamp as f64 / 1_000_000_000.0)
        }
    }

    /// Handle pending file dialog result
    ///
    /// Helper method to check and process file dialog results.
    /// Returns true if a result was processed, false otherwise.
    pub(crate) fn handle_file_dialog_result(&mut self, cx: &mut Context<Self>) -> bool {
        if let Some(receiver) = self.pending_file_path.take() {
            match receiver.try_recv() {
                Ok(Some(path_str)) => {
                    // File selected successfully
                    self.new_channel_db_path = path_str.clone();
                    self.set_status(format!("✅ Selected: {}", path_str), cx);
                    true
                }
                Ok(None) => {
                    // User cancelled
                    self.set_status("❌ File selection cancelled", cx);
                    true
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    // Not ready yet, put it back
                    self.pending_file_path = Some(receiver);
                    false
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    // Thread ended without result
                    self.set_status("", cx);
                    true
                }
            }
        } else {
            false
        }
    }

    #[allow(dead_code)]
    /// Format data bytes as hexadecimal string
    ///
    /// Helper method to convert CAN/LIN data to hex string representation.
    /// This reduces code duplication in message rendering.
    pub(crate) fn format_data_hex(data: &[u8], dlc: u8) -> String {
        let actual_data_len = data.len().min(dlc as usize);
        data.iter()
            .take(actual_data_len)
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Extract and format CAN signals from DBC database
    ///
    /// Helper method to decode and format signals from a CAN message.
    /// Returns a comma-separated string of signal=value pairs.
    pub(crate) fn extract_can_signals(
        &self,
        channel: u16,
        msg_id: u32,
        data: &[u8],
    ) -> String {
        if let Some(db) = self.dbc_channels.get(&channel) {
            if let Some(message) = db.messages.get(&msg_id) {
                return message
                    .signals
                    .iter()
                    .map(|(name, signal)| {
                        let val = signal.decode(data);
                        format!("{}={:.2}", name, val)
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
            }
        }
        String::new()
    }

    /// Extract and format LIN signals from LDF database
    ///
    /// Helper method to decode and format signals from a LIN message.
    /// Returns a comma-separated string of signal=value pairs.
    pub(crate) fn extract_lin_signals(
        &self,
        channel: u16,
        frame_id: u8,
        data: &[u8],
    ) -> String {
        if let Some(db) = self.ldf_channels.get(&channel) {
            // Search for the frame with the matching ID
            if let Some(frame) = db.frames.values().find(|f| f.id == frame_id as u32) {
                return frame
                    .signals
                    .iter()
                    .filter_map(|mapping| {
                        db.signals
                            .get(&mapping.signal_name)
                            .map(|sig| (mapping, sig))
                    })
                    .map(|(mapping, signal)| {
                        let val = signal.decode(data, mapping.offset);
                        format!("{}={}", signal.name, val)
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
            }
        }
        String::new()
    }

    /// Import a database file
    /// Save the current configuration to file
    pub(crate) fn save_config(&self, cx: &mut Context<Self>) {
        let config_path = PathBuf::from("multi_channel_config.json");
        if let Ok(content) = serde_json::to_string_pretty(&self.app_config) {
            if std::fs::write(&config_path, content).is_ok() {
                cx.notify();
            }
        }
    }
}
impl CanViewApp {
    pub(crate) fn toggle_maximize(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // Initialize display bounds on first use
        self.initialize_display_bounds(cx);

        // Preserve runtime state across window recreation
        let runtime_state = self.save_runtime_state();

        // CRITICAL SAFETY: Reset all unsafe state before window recreation
        // These fields contain Entity handles or complex state that becomes invalid
        // after window recreation. Must be reset to None/default values.

        // Reset all Entity<InputState> handles (they point to destroyed window objects)
        self.library_name_input = None;
        self.version_name_input = None;
        self.channel_id_input = None;
        self.channel_name_input = None;
        self.channel_db_path_input = None;
        self.signal_search_input = None;
        self.signal_filter_text = "".into();  // Reset filter text to prevent observe callback loop

        // Reset view to LogView (safest view with no complex state)
        self.current_view = AppView::LogView;

        // Reset dialog states
        self.show_library_dialog = false;
        self.show_channel_config_dialog = false;
        self.show_add_channel_input = false;
        self.show_version_input = false;

        // Reset scroll handles (create fresh instances)
        use gpui::UniformListScrollHandle;
        self.list_scroll_handle = UniformListScrollHandle::new();
        self.filter_scroll_handle = UniformListScrollHandle::new();
        self.channel_filter_scroll_handle = UniformListScrollHandle::new();
        self.signal_scroll_handle = UniformListScrollHandle::new();

        // Reset interaction state
        self.scrollbar_drag_state = None;
        self.scroll_offset = px(0.0);

        // Reset only transient plot interaction state (not zoom state or width)
        self.is_dragging_zoom = false;
        self.zoom_drag_start_x = None;
        self.zoom_drag_current_x = None;
        self.hover_point = None;
        self.plot_hover_time = None;
        self.plot_hover_x = None;

        if self.is_maximized {
            // Restore to normal size
            if let Some(saved_bounds) = self.saved_window_bounds {
                // Open new window with saved bounds
                // New window should NOT be maximized (is_maximized = false)
                // Also pass None for saved_window_bounds (restored, so no saved bounds)
                cx.open_window(
                    WindowOptions {
                        window_bounds: Some(WindowBounds::Windowed(saved_bounds)),
                        titlebar: Some(TitlebarOptions {
                            title: Some("CANVIEW - Bus Data Analyzer".into()),
                            appears_transparent: true,
                            traffic_light_position: None,
                        }),
                        kind: WindowKind::Normal,
                        ..Default::default()
                    },
                    |window, cx| {
                        let mut app = cx.new(|_cx| Self::new_with_maximized_state_and_bounds(false, None));
                        let view = app.clone();
                        // Restore runtime state and load configuration
                        app.update(cx, |app, cx| {
                            app.restore_runtime_state(runtime_state);
                            app.load_startup_config();
                            cx.notify();
                        });
                        cx.new(|cx| gpui_component::Root::new(view, window, cx))
                    },
                )
                .ok();

                // Close current window
                window.remove_window();
            }
        } else {
            // Maximize to display bounds
            if let Some(display_bounds) = self.display_bounds {
                // Save current bounds first
                let saved_bounds = Some(window.bounds());

                // Open new maximized window
                // New window should be maximized (is_maximized = true)
                // Pass saved bounds so it can restore later
                cx.open_window(
                    WindowOptions {
                        window_bounds: Some(WindowBounds::Windowed(display_bounds)),
                        titlebar: Some(TitlebarOptions {
                            title: Some("CANVIEW - Bus Data Analyzer".into()),
                            appears_transparent: true,
                            traffic_light_position: None,
                        }),
                        kind: WindowKind::Normal,
                        ..Default::default()
                    },
                    |window, cx| {
                        let mut app = cx.new(|_cx| Self::new_with_maximized_state_and_bounds(true, saved_bounds));
                        // Restore runtime state and load configuration
                        let view = app.clone();
                        app.update(cx, |app, cx| {
                            app.restore_runtime_state(runtime_state);
                            app.load_startup_config();
                            cx.notify();
                        });
                        cx.new(|cx| gpui_component::Root::new(view, window, cx))
                    },
                )
                .ok();

                // Close current window
                window.remove_window();
            }
        }
    }

    fn new_with_state(
        current_view: AppView,
        messages: Vec<LogObject>,
        status_msg: SharedString,
        dbc_channels: HashMap<u16, DbcDatabase>,
        ldf_channels: HashMap<u16, LdfDatabase>,
        app_config: AppConfig,
        selected_signals: Vec<String>,
        start_time: Option<chrono::NaiveDateTime>,
        config_dir: Option<PathBuf>,
        config_file_path: Option<PathBuf>,
        is_maximized: bool,
        saved_window_bounds: Option<Bounds<Pixels>>,
        display_bounds: Option<Bounds<Pixels>>,
        library_manager: LibraryManager,
    ) -> Self {
        let mut app = Self {
            current_view,
            messages,
            status_msg,
            dbc_channels,
            ldf_channels,
            app_config,
            selected_signals,
            start_time,
            plot_data: std::sync::Arc::from([]),
            plot_full_data: std::sync::Arc::from([]),
            config_dir,
            config_file_path,
            signal_storage: crate::library::SignalLibraryStorage::new().ok(),
            is_maximized,
            is_streaming_mode: false,
            saved_window_bounds,
            display_bounds,
            list_scroll_handle: gpui::UniformListScrollHandle::new(),
            scrollbar_drag_state: None,
            scroll_offset: px(0.0),
            list_container_height: 850.0,
            id_display_decimal: true, // Default to decimal
            id_filter: None,
            id_filter_text: "".into(),
            show_id_filter_input: false,
            filter_scroll_offset: px(0.0),
            filter_scroll_handle: gpui::UniformListScrollHandle::new(),
            mouse_over_filter_dropdown: false,
            mouse_down_on_filter_dropdown: false,
            dropdown_just_opened: false,
            // Channel filter
            channel_filter: None,
            channel_filter_text: "".into(),
            show_channel_filter_input: false,
            channel_filter_scroll_offset: px(0.0),
            channel_filter_scroll_handle: gpui::UniformListScrollHandle::new(),
            signal_filter_text: "".into(),
            signal_search_input: None,
            signal_scroll_handle: gpui::UniformListScrollHandle::new(),
            // Library management
            library_manager,
            selected_library_id: None,
            selected_version_id: None,
            new_library_name: String::new(),
            library_cursor_position: 0,
            library_versions_expanded: true,
            show_version_input: false,
            new_version_name: String::new(),
            new_version_cursor_position: 0,
            show_library_dialog: false,
            library_dialog_type: super::state::LibraryDialogType::Create,
            library_search_query: String::new(),
            library_filter_type: None,
            // gpui-component input support
            library_name_input: None,
            version_name_input: None,
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
            channel_id_input: None,
            channel_name_input: None,
            show_add_channel_input: false,
            channel_db_path_input: None,
            new_channel_type: ChannelType::CAN,
            pending_file_path: None,
            // Deprecated fields for backward compatibility
            focused_library_input: None,
            is_editing_library_name: false,
            library_input_state: crate::app::state::SimpleDeprecatedInputState::default(),
            library_focus_handle: None,
            ime_handler_registered: false,
            plot_zoom_start: None,
            plot_zoom_end: None,
            plot_full_time_min: None,
            plot_full_time_max: None,
            is_dragging_zoom: false,
            zoom_drag_start_x: None,
            zoom_drag_current_x: None,
            show_plot_points: true,
            hover_point: None,
            plot_hover_time: None,
            plot_hover_x: None,
            plot_width_px: px(0.0),
            // File menu dropdown state
            show_file_menu: false,
            // Server state
            server_handle: None,
            show_share_dialog: false,
            show_import_dialog: false,
            import_url: String::new(),
            import_status: None,
            import_url_input: None,
            pending_import: None,
        };

        // Load startup config (this will reset some state, so do it carefully)
        // We skip loading config if we're restoring state
        if !is_maximized {
            app.load_startup_config();
        }

        app
    }

    pub(crate) fn update_container_height(&mut self, window: &mut Window) {
        // Get window bounds
        let window_size = window.bounds();
        let window_height = f32::from(window_size.size.height);

        // Calculate actual list container height
        // Window height - top bar (56px) - status bar (24px) - log header (28px)
        let container_height = window_height - 37.0 - 25.0 - 29.0;  // 37px top bar (36+1 border), 25px status bar (24+1 border), 29px header (28+1 border)

        // Only update if it changed significantly (more than 10px difference)
        if (container_height - self.list_container_height).abs() > 10.0 {
            self.list_container_height = container_height;
        }
    }

}

// ========== Library Management Methods ==========
impl CanViewApp {
    /// Create a new library
    pub fn create_library(&mut self, cx: &mut Context<Self>) {
        if self.new_library_name.trim().is_empty() {
            self.status_msg = "Library name cannot be empty".into();
            cx.notify();
            return;
        }

        match self.library_manager.create_library(
            self.new_library_name.clone(),
            ChannelType::CAN, // Default to CAN for now
        ) {
            Ok(library) => {
                eprintln!("✅ Library created successfully: {}", library.name);

                // Sync to app_config for persistence
                self.app_config.libraries = self.library_manager.libraries().to_vec();

                // Save config to file
                self.save_config(cx);

                self.status_msg = format!("Library '{}' created", self.new_library_name).into();
                self.new_library_name.clear();
                self.show_library_dialog = false;
                cx.notify();
            }
            Err(e) => {
                eprintln!("❌ Error creating library: {}", e);
                self.status_msg = format!("Error creating library: {}", e).into();
                cx.notify();
            }
        }
    }

    /// Delete a library
    pub fn delete_library(&mut self, library_id: &str, cx: &mut Context<Self>) {
        // Collect channel IDs from this library BEFORE deleting it
        let channel_ids_to_remove: Vec<u16> = self
            .library_manager
            .find_library(library_id)
            .map(|lib| {
                lib.versions
                    .iter()
                    .flat_map(|v| v.channel_databases.iter().map(|db| db.channel_id))
                    .collect()
            })
            .unwrap_or_default();

        match self
            .library_manager
            .delete_library(library_id, &self.app_config.mappings)
        {
            Ok(_) => {
                self.status_msg = "Library deleted".into();
                if self.selected_library_id.as_ref() == Some(&library_id.to_string()) {
                    self.selected_library_id = None;
                    self.selected_version_id = None;
                }
                // Clear runtime channel caches so plot view no longer shows them
                for ch_id in channel_ids_to_remove {
                    self.dbc_channels.remove(&ch_id);
                    self.ldf_channels.remove(&ch_id);
                }
                // Sync config
                self.app_config.libraries = self.library_manager.libraries().to_vec();
                self.save_config(cx);
                cx.notify();
            }
            Err(e) => {
                self.status_msg = format!("Error deleting library: {}", e).into();
                cx.notify();
            }
        }
    }

    /// Add a version to a library
    pub fn add_library_version(&mut self, cx: &mut Context<Self>) {
        let library_id = match &self.selected_library_id {
            Some(id) => id.clone(),
            None => {
                self.status_msg = "No library selected".into();
                cx.notify();
                return;
            }
        };

        // Get version name from input if available
        let version_name = if let Some(input) = &self.version_name_input {
            input.read(cx).value().to_string()
        } else {
            self.new_version_name.clone()
        };

        if version_name.trim().is_empty() {
            self.status_msg = "Version name cannot be empty".into();
            cx.notify();
            return;
        }

        eprintln!(
            "📝 Adding version: '{}' to library: {}",
            version_name, library_id
        );

        // Hide the input dialog
        self.show_version_input = false;
        cx.notify();

        // Create version directly, bypassing file existence check
        // TODO: File dialog integration requires fixing GPUI async lifetime issues on Windows
        let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let version = crate::models::library::LibraryVersion::new(
            version_name.clone(),
            String::new(), // Empty path for now
            date,
        )
        .with_description(format!(
            "Created version '{}' (database file to be added)",
            version_name
        ));

        // Add version directly to library
        if let Some(library) = self.library_manager.find_library_mut(&library_id) {
            library.add_version(version.clone());
            eprintln!("✅ Version '{}' added successfully", version_name);

            // Sync to app_config for persistence
            self.app_config.libraries = self.library_manager.libraries().to_vec();

            // Save config to file
            self.save_config(cx);

            self.status_msg = format!(
                "Version '{}' created successfully. Use 'Add Database File' to attach a database.",
                version_name
            )
            .into();
            self.new_version_name.clear();
            cx.notify();
        } else {
            eprintln!("❌ Error: Library not found");
            self.status_msg = "Error: Library not found".into();
            cx.notify();
        }
    }

    /// Delete a version from a library
    pub fn delete_library_version(
        &mut self,
        library_id: &str,
        version_name: &str,
        cx: &mut Context<Self>,
    ) {
        match self.library_manager.remove_version(
            library_id,
            version_name,
            &self.app_config.mappings,
        ) {
            Ok(_) => {
                self.status_msg = format!("Version '{}' deleted", version_name).into();
                cx.notify();
            }
            Err(e) => {
                self.status_msg = format!("Error deleting version: {}", e).into();
                cx.notify();
            }
        }
    }

    // ---------- Rename library ------------------------------------------------

    pub fn start_rename_library(&mut self, library_id: String, current_name: String, window: &mut Window, cx: &mut Context<Self>) {
        self.renaming_library_id = Some(library_id);
        self.renaming_version_name = None;
        self.rename_library_text = current_name.clone();
        let input = cx.new(|cx| {
            InputState::new(window, cx).default_value(current_name)
        });
        self.rename_library_input = Some(input);
        cx.notify();
    }

    pub fn commit_rename_library(&mut self, cx: &mut Context<Self>) {
        let old_id = match self.renaming_library_id.clone() {
            Some(id) => id,
            None => return,
        };
        let new_name = if let Some(input) = &self.rename_library_input {
            input.read(cx).value().to_string()
        } else {
            self.rename_library_text.clone()
        };
        let new_name = new_name.trim().to_string();
        if new_name.is_empty() {
            self.status_msg = "Library name cannot be empty".into();
            cx.notify();
            return;
        }

        match self.library_manager.rename_library(&old_id, new_name.clone()) {
            Ok(new_id) => {
                // Update all mappings that reference old library id
                for mapping in &mut self.app_config.mappings {
                    if mapping.library_id.as_deref() == Some(&old_id) {
                        mapping.library_id = Some(new_id.clone());
                    }
                }
                // Keep the selected library pointing to the renamed library
                if self.selected_library_id.as_deref() == Some(&old_id) {
                    self.selected_library_id = Some(new_id.clone());
                }
                self.app_config.libraries = self.library_manager.libraries().to_vec();
                self.save_config(cx);
                self.status_msg = format!("Library renamed to '{}'", new_name).into();
            }
            Err(e) => {
                self.status_msg = format!("Rename failed: {}", e).into();
            }
        }
        self.cancel_rename_library(cx);
    }

    pub fn cancel_rename_library(&mut self, cx: &mut Context<Self>) {
        self.renaming_library_id = None;
        self.rename_library_input = None;
        self.rename_library_text.clear();
        cx.notify();
    }

    // ---------- Rename version ------------------------------------------------

    pub fn start_rename_version(&mut self, version_name: String, window: &mut Window, cx: &mut Context<Self>) {
        self.renaming_version_name = Some(version_name.clone());
        self.renaming_library_id = None;
        self.rename_version_text = version_name.clone();
        let input = cx.new(|cx| {
            InputState::new(window, cx).default_value(version_name)
        });
        self.rename_version_input = Some(input);
        cx.notify();
    }

    pub fn commit_rename_version(&mut self, cx: &mut Context<Self>) {
        let library_id = match &self.selected_library_id {
            Some(id) => id.clone(),
            None => return,
        };
        let old_name = match self.renaming_version_name.clone() {
            Some(n) => n,
            None => return,
        };
        let new_name = if let Some(input) = &self.rename_version_input {
            input.read(cx).value().to_string()
        } else {
            self.rename_version_text.clone()
        };
        let new_name = new_name.trim().to_string();
        if new_name.is_empty() {
            self.status_msg = "Version name cannot be empty".into();
            cx.notify();
            return;
        }

        match self.library_manager.rename_version(&library_id, &old_name, new_name.clone()) {
            Ok(()) => {
                // Update all mappings that reference old version name within this library
                for mapping in &mut self.app_config.mappings {
                    if mapping.library_id.as_deref() == Some(&library_id)
                        && mapping.version_name.as_deref() == Some(&old_name)
                    {
                        mapping.version_name = Some(new_name.clone());
                    }
                }
                // Update selected_version_id if it points to the old name
                if self.selected_version_id.as_deref() == Some(&old_name) {
                    self.selected_version_id = Some(new_name.clone());
                }
                self.app_config.libraries = self.library_manager.libraries().to_vec();
                self.save_config(cx);
                self.status_msg = format!("Version renamed to '{}'", new_name).into();
            }
            Err(e) => {
                self.status_msg = format!("Rename failed: {}", e).into();
            }
        }
        self.cancel_rename_version(cx);
    }

    pub fn cancel_rename_version(&mut self, cx: &mut Context<Self>) {
        self.renaming_version_name = None;
        self.rename_version_input = None;
        self.rename_version_text.clear();
        cx.notify();
    }

    /// Load a library version
    pub fn load_library_version(
        &mut self,
        library_id: &str,
        version_name: &str,
        cx: &mut Context<Self>,
    ) {
        // Reset add channel input state when loading a new version
        self.hide_add_channel_input(cx);

        self.internal_load_library_version(1, library_id, version_name);

        cx.notify();
    }

    /// Apply a version to mappings and load it
    pub fn apply_version_to_mappings(
        &mut self,
        library_id: &str,
        version_name: &str,
        cx: &mut Context<Self>,
    ) {
        eprintln!(
            "🖱️ Applying version {} of {} to mappings",
            version_name, library_id
        );

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

        // Update mappings
        for channel_db in &version.channel_databases {
            if let Some(mapping) = self
                .app_config
                .mappings
                .iter_mut()
                .find(|m| m.channel_id == channel_db.channel_id)
            {
                mapping.library_id = Some(library_id.to_string());
                mapping.version_name = Some(version_name.to_string());
                mapping.channel_type = library.channel_type;
            } else {
                self.app_config
                    .mappings
                    .push(crate::models::ChannelMapping {
                        channel_id: channel_db.channel_id,
                        channel_type: library.channel_type,
                        library_id: Some(library_id.to_string()),
                        version_name: Some(version_name.to_string()),
                        path: String::new(),
                        description: String::new(),
                    });
            }
        }

        // Load into memory
        self.internal_load_library_version(1, library_id, version_name);

        // Save config
        self.save_config(cx);

        self.status_msg =
            format!("✅ Applied version {} to all plot channels", version_name).into();
        cx.notify();
    }

    /// Internal method to load a library version without GPUI context
    fn internal_load_library_version(
        &mut self,
        default_channel_id: u16,
        library_id: &str,
        version_name: &str,
    ) {
        eprintln!(
            "DEBUG: Internal load library version: lib={}, ver={}, ch={}",
            library_id, version_name, default_channel_id
        );

        let library = match self.library_manager.find_library(library_id) {
            Some(lib) => lib,
            None => {
                self.status_msg = "Library not found".into();
                return;
            }
        };

        let version = match library.get_version(version_name) {
            Some(ver) => ver,
            None => {
                self.status_msg = "Version not found".into();
                return;
            }
        };

        // Extract needed data from library to avoid borrow conflicts
        let library_name = library.name.clone();
        let channel_type = library.channel_type;

        // Load the database for each channel in the version
        // Clone to break borrow chain so we can mutably borrow self later
        let channel_dbs: Vec<_> = version.channel_databases.clone();
        let num_channels = channel_dbs.len();

        if channel_dbs.is_empty() {
            // Use the default path (backward compatibility)
            let path = &version.path;

            // Validate path
            if let Err(err_msg) = self.validate_database_path(path, &format!("version '{}'", version_name)) {
                self.status_msg = err_msg.clone().into();
                eprintln!("ERROR: {}", err_msg);
                return;
            }

            match self
                .library_manager
                .load_database(path, channel_type)
            {
                Ok(database) => {
                    self.insert_database_into_channel(database, default_channel_id);
                    self.status_msg =
                        format!("✅ Loaded version {} of {}", version_name, library_name).into();
                }
                Err(e) => {
                    self.status_msg = format!("❌ Error loading database: {}", e).into();
                    eprintln!("ERROR: Failed to load database from '{}': {}", path, e);
                }
            }
        } else {
            // Load all configured channels
            for channel_db in channel_dbs {
                // Validate path
                let context = format!("channel {}", channel_db.channel_id);
                if let Err(err_msg) = self.validate_database_path(&channel_db.database_path, &context) {
                    self.status_msg = err_msg.clone().into();
                    eprintln!("ERROR: {}", err_msg);
                    continue;
                }

                match self
                    .library_manager
                    .load_database(&channel_db.database_path, channel_type)
                {
                    Ok(database) => {
                        self.insert_database_into_channel(database, channel_db.channel_id);
                    }
                    Err(e) => {
                        self.status_msg =
                            format!("❌ Error loading channel {}: {}", channel_db.channel_id, e)
                                .into();
                        eprintln!(
                            "ERROR: Failed to load database for channel {} from '{}': {}",
                            channel_db.channel_id, channel_db.database_path, e
                        );
                    }
                }
            }
            self.status_msg = format!(
                "Loaded version {} of {} ({} channels)",
                version_name,
                library_name,
                num_channels
            )
            .into();
        }

        eprintln!(
            "DEBUG: Current DBC channels: {:?}",
            self.dbc_channels.keys()
        );
        eprintln!(
            "DEBUG: Current LDF channels: {:?}",
            self.ldf_channels.keys()
        );
    }

    // ========== Channel Configuration Methods ==========

    /// Validate database path and file existence
    ///
    /// Helper method to validate database file path.
    /// Returns Ok(()) if valid, Err with error message if invalid.
    fn validate_database_path(&self, path: &str, context: &str) -> Result<(), String> {
        // Check if path is empty
        if path.trim().is_empty() {
            return Err(format!(
                "❌ Database path is empty for {}. Please add a database file in the Library view.",
                context
            ));
        }

        // Check if file exists
        if !std::path::Path::new(path).exists() {
            return Err(format!(
                "❌ Database file not found: {}. Please check the file path in Library view.",
                path
            ));
        }

        Ok(())
    }

    /// Insert database into the appropriate channel map
    ///
    /// Helper method to insert a loaded database (DBC or LDF) into the
    /// corresponding channel map based on the database type.
    fn insert_database_into_channel(&mut self, database: crate::library::Database, channel_id: u16) {
        match database {
            crate::library::Database::Dbc(dbc) => {
                eprintln!("DEBUG: Inserting DBC into channel {}", channel_id);
                self.dbc_channels.insert(channel_id, dbc);
            }
            crate::library::Database::Ldf(ldf) => {
                eprintln!("DEBUG: Inserting LDF into channel {}", channel_id);
                self.ldf_channels.insert(channel_id, ldf);
            }
        }
    }

    /// Print timestamp diagnostics for BLF data
    ///
    /// Helper method to validate and print timestamp information
    /// for loaded BLF objects, helping identify timestamp issues.
    fn print_timestamp_diagnostics(objects: &[blf::LogObject]) {
        println!("\n=== BLF 时间戳诊断 ===");
        println!("总消息数: {}", objects.len());

        // 检查前 10 条消息的时间戳
        println!("\n前 10 条消息的时间戳:");
        for (i, obj) in objects.iter().take(10).enumerate() {
            let ts = obj.timestamp();
            println!(
                "  Message {}: {} ns ({:.9} s)",
                i,
                ts,
                ts as f64 / 1_000_000_000.0
            );
        }

        // 检查时间戳是否都相同
        if objects.len() > 1 {
            let first_ts = objects[0].timestamp();
            let last_ts = objects.last().unwrap().timestamp();
            let time_span = (last_ts - first_ts) as f64 / 1_000_000_000.0;

            println!("\n时间跨度分析:");
            println!("  第一条: {} ns", first_ts);
            println!("  最后一条: {} ns", last_ts);
            println!("  时间跨度: {:.6} 秒", time_span);

            if time_span < 0.000001 {
                println!("  ⚠️  警告: 所有消息的时间戳几乎相同!");
            } else {
                println!("  ✅ 时间戳正常变化");
            }
        }
        println!("===================\n");
    }

    /// Log BLF parsing errors to console
    ///
    /// Helper method to print BLF parsing errors in a formatted way.
    fn log_blf_errors(errors: &[blf::BlfParseError], object_count: usize) {
        eprintln!("\n⚠️  BLF 解析过程中发现 {} 个错误:", errors.len());
        for (i, error) in errors.iter().enumerate() {
            eprintln!("  错误 {}: {}", i + 1, error);
        }
        eprintln!(
            "  ✅ 但仍成功解析了 {} 个对象，这些对象将正常显示\n",
            object_count
        );
    }

    /// Parse BLF file start time from SystemTime
    ///
    /// Helper method to convert BLF file statistics SystemTime
    /// to chrono NaiveDateTime with nanosecond precision.
    fn parse_blf_start_time(st: &blf::SystemTime) -> Option<chrono::NaiveDateTime> {
        println!("\n起始时间解析:");
        println!("  原始 SystemTime: {:?}", st);

        let date_opt = chrono::NaiveDate::from_ymd_opt(st.year as i32, st.month as u32, st.day as u32);
        let time_opt = chrono::NaiveTime::from_hms_nano_opt(
            st.hour as u32,
            st.minute as u32,
            st.second as u32,
            st.milliseconds as u32 * 1_000_000, // Convert milliseconds to nanoseconds
        );

        match (date_opt, time_opt) {
            (Some(date), Some(time)) => {
                let dt = chrono::NaiveDateTime::new(date, time);
                println!("  ✅ 解析成功: {:?}", dt);
                Some(dt)
            }
            _ => {
                println!("  ❌ 解析失败");
                None
            }
        }
    }

    /// Display BLF file loading error
    ///
    /// Helper method to print and display BLF file loading errors.
    fn display_blf_load_error<E: std::fmt::Display + std::fmt::Debug>(error: E) {
        eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        eprintln!("📂 BLF File Loading Failed");
        eprintln!("Error: {:?}", error);
        eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    }

    /// Display library loading statistics
    ///
    /// Helper method to print library manager statistics after loading.
    fn display_library_stats(library_manager: &crate::library::LibraryManager) -> (usize, usize, usize) {
        // 统计信息
        let total_versions: usize = library_manager
            .libraries()
            .iter()
            .map(|lib| lib.versions.len())
            .sum();
        let total_channels: usize = library_manager
            .libraries()
            .iter()
            .flat_map(|lib| &lib.versions)
            .map(|ver| ver.channel_databases.len())
            .sum();

        eprintln!("  ✅ 加载完成:");
        eprintln!("     - {} 个库", library_manager.libraries().len());
        eprintln!("     - {} 个版本", total_versions);
        eprintln!("     - {} 个通道", total_channels);

        // 显示库列表
        for library in library_manager.libraries() {
            eprintln!(
                "     📦 {}: {} 个版本",
                library.name,
                library.versions.len()
            );
        }

        (library_manager.libraries().len(), total_versions, total_channels)
    }

    /// Initialize display bounds from context
    ///
    /// Helper method to calculate and set display bounds on first use.
    fn initialize_display_bounds(&mut self, cx: &mut Context<Self>) {
        if self.display_bounds.is_none() {
            let displays = cx.displays();
            if let Some(display) = displays.first() {
                let display_bounds = display.bounds();
                // Leave a small margin for the task bar and dock
                let margin = px(4.0);
                self.display_bounds = Some(Bounds {
                    origin: Point::new(margin, margin),
                    size: Size {
                        width: display_bounds.size.width - margin * 2.0,
                        height: display_bounds.size.height - margin * 2.0,
                    },
                });
            }
        }
    }

    /// Set status message and trigger UI update
    ///
    /// Helper method to consistently set status messages and notify the UI.
    /// This ensures all status updates follow the same pattern.
    fn set_status(&mut self, msg: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.status_msg = msg.into();
        cx.notify();
    }

    /// Get zebra striping background color
    ///
    /// Helper method to determine background color for zebra striping in lists.
    /// Returns alternating colors based on index parity.
    pub(crate) fn get_zebra_bg_color(index: usize) -> Rgba {
        if index.is_multiple_of(2) {
            rgb(0x09090b) // Zed's dark background (zebra)
        } else {
            rgb(0x0c0c0e) // Zed's dark background (base)
        }
    }

    /// Show channel input for adding a new channel (inline)
    pub fn show_add_channel_dialog(&mut self, cx: &mut Context<Self>) {
        self.show_add_channel_input = true;
        self.new_channel_id.clear();
        self.new_channel_name.clear();
        self.new_channel_db_path.clear();
        self.editing_channel_index = None;
        cx.notify();
    }

    /// Hide channel input and clear values
    pub fn hide_add_channel_input(&mut self, cx: &mut Context<Self>) {
        self.show_add_channel_input = false;
        self.new_channel_id.clear();
        self.new_channel_name.clear();
        self.new_channel_db_path.clear();
        // Drop the input entities so they are recreated fresh next time
        self.channel_id_input = None;
        self.channel_name_input = None;
        self.channel_db_path_input = None;
        cx.notify();
    }

    /// Save channel configuration
    pub fn save_channel_config(&mut self, cx: &mut Context<Self>) {
        // Debug: print current state
        eprintln!("DEBUG: Saving channel config");
        eprintln!("DEBUG: new_channel_id before: '{}'", self.new_channel_id);
        eprintln!(
            "DEBUG: new_channel_name before: '{}'",
            self.new_channel_name
        );
        eprintln!(
            "DEBUG: new_channel_db_path before: '{}'",
            self.new_channel_db_path
        );

        // Read values from input fields (Manual read as primary)
        // Note: Validation on input creation is currently removed to avoid issues.
        if let Some(id_input) = &self.channel_id_input {
            let id_text = id_input.read(cx).text().to_string();
            eprintln!(
                "DEBUG: Manual Read ID: '{}', Listener ID: '{}'",
                id_text, self.new_channel_id
            );
            // If listener failed, fallback to manual read
            if self.new_channel_id.is_empty() && !id_text.is_empty() {
                self.new_channel_id = id_text;
            } else if !id_text.is_empty() {
                self.new_channel_id = id_text;
            }
        } else {
            self.status_msg = "Error: Input lost. Try reopening.".into();
            cx.notify();
            return;
        }

        if let Some(name_input) = &self.channel_name_input {
            let name_text = name_input.read(cx).text().to_string();
            self.new_channel_name = name_text;
        }

        if self.new_channel_id.is_empty() {
            self.status_msg = "Please enter channel ID".into();
            cx.notify();
            return;
        }

        if self.new_channel_name.is_empty() {
            self.status_msg = "Please enter channel name".into();
            cx.notify();
            return;
        }

        if self.new_channel_db_path.is_empty() {
            self.status_msg = "Please select a database file".into();
            cx.notify();
            return;
        }

        // Path is set automatically when file is selected via "Select File..." button
        // No need to read from input since path display is read-only
        eprintln!(
            "DEBUG: Database path from file selector: '{}'",
            self.new_channel_db_path
        );

        eprintln!(
            "DEBUG: Final values - ID: '{}', Name: '{}', Path: '{}'",
            self.new_channel_id, self.new_channel_name, self.new_channel_db_path
        );

        // Validate inputs
        let channel_id: u16 = match self.new_channel_id.trim().parse() {
            Ok(id) if id > 0 && id <= 255 => id,
            _ => {
                self.status_msg = "Invalid channel ID. Must be between 1 and 255".into();
                cx.notify();
                return;
            }
        };

        if self.new_channel_name.trim().is_empty() {
            self.status_msg = "Channel name cannot be empty".into();
            cx.notify();
            return;
        }

        if self.new_channel_db_path.trim().is_empty() {
            self.status_msg = "Please select a database file or enter a path".into();
            cx.notify();
            return;
        }

        // Get the selected library and version
        let library_id = match &self.selected_library_id {
            Some(id) => id.clone(),
            None => {
                self.status_msg = "No library selected".into();
                cx.notify();
                return;
            }
        };

        // Find version name first to avoid borrow issues
        let version_name = {
            let library = match self.library_manager.find_library(&library_id) {
                Some(lib) => lib,
                None => {
                    self.status_msg = "Library not found".into();
                    cx.notify();
                    return;
                }
            };

            let version = match library.latest_version() {
                Some(v) => v,
                None => {
                    self.status_msg = "No version found. Please add a version first.".into();
                    cx.notify();
                    return;
                }
            };

            version.name.clone()
        };

        // Set selected_version_id if not already set
        if self.selected_version_id.is_none() {
            self.selected_version_id = Some(version_name.clone());
        }

        // Create channel database config
        let mut channel_db = crate::models::library::ChannelDatabase::new(
            self.new_channel_type,
            channel_id,
            self.new_channel_name.trim().to_string(),
            self.new_channel_db_path.trim().to_string(),
        );

        // 🔧 自动复制文件到本地存储
        if let Some(ref storage) = self.signal_storage {
            // 获取库名用于存储路径
            let library_name = {
                let library = match self.library_manager.find_library(&library_id) {
                    Some(lib) => lib,
                    None => {
                        self.status_msg = "Error: Library not found during file copy".into();
                        cx.notify();
                        return;
                    }
                };
                library.name.clone()
            };

            // 复制文件到本地存储
            let source_path = std::path::Path::new(&self.new_channel_db_path);
            match storage.copy_database(&library_name, &version_name, source_path) {
                Ok(local_path) => {
                    // 使用本地路径更新 channel_db
                    channel_db.database_path = local_path.to_string_lossy().to_string();
                    eprintln!("✅ Database file copied to local storage: {:?}", local_path);
                }
                Err(e) => {
                    self.status_msg = format!("Failed to copy database file: {}", e).into();
                    cx.notify();
                    return;
                }
            }
        } else {
            eprintln!("⚠️  Signal storage not available, using original path");
        }

        // Validate the channel config
        if let Err(e) = channel_db.validate() {
            let msg = format!("Validation error: {}", e);
            eprintln!("❌ {}", msg);
            self.status_msg = msg.into();
            cx.notify();
            return;
        }

        // Add to the version (we need mutable access)
        let library = match self.library_manager.find_library_mut(&library_id) {
            Some(lib) => lib,
            None => {
                self.status_msg = "Error: Library not found".into();
                cx.notify();
                return;
            }
        };
        if let Some(version) = library.versions.iter_mut().find(|v| v.name == version_name) {
            match version.add_channel_database(channel_db) {
                Ok(_) => {
                    self.status_msg = format!("Channel {} added successfully", channel_id).into();

                    // Close input form cleanly — prevents inconsistent state
                    // where show_add_channel_input=true but all input entities are None.
                    // User can click "Add Channel" again to add more channels.
                    self.hide_add_channel_input(cx);

                    // Reset type to CAN
                    self.new_channel_type = crate::models::ChannelType::CAN;

                    // 🔄 同步到 app_config
                    self.app_config.libraries = self.library_manager.libraries().to_vec();

                    // 💾 自动保存配置
                    self.save_config(cx);
                    eprintln!("✅ Configuration saved automatically");

                    cx.notify();
                }
                Err(e) => {
                    self.status_msg = format!("Error adding channel: {}", e).into();
                    cx.notify();
                }
            }
        }
    }

    /// Delete channel from version
    pub fn delete_channel(&mut self, channel_id: u16, cx: &mut Context<Self>) {
        let library_id = match &self.selected_library_id {
            Some(id) => id.clone(),
            None => return,
        };

        let version_name = match &self.selected_version_id {
            Some(name) => name.clone(),
            None => return,
        };

        let library = match self.library_manager.find_library_mut(&library_id) {
            Some(lib) => lib,
            None => return,
        };

        if let Some(version) = library.versions.iter_mut().find(|v| v.name == version_name) {
            // Remove from configuration
            version
                .channel_databases
                .retain(|db| db.channel_id != channel_id);

            // Remove from runtime cache
            self.dbc_channels.remove(&channel_id);
            self.ldf_channels.remove(&channel_id);

            // Sync to app config
            self.app_config.libraries = self.library_manager.libraries().to_vec();

            // Save to disk
            self.save_config(cx);

            self.status_msg = format!("Channel {} deleted", channel_id).into();
            cx.notify();
        }
    }

    pub fn cancel_channel_config(&mut self, cx: &mut Context<Self>) {
        self.show_add_channel_input = false;
        self.new_channel_id.clear();
        self.new_channel_name.clear();
        self.new_channel_db_path.clear();

        // Reset input entities
        self.channel_id_input = None;
        self.channel_name_input = None;
        self.channel_db_path_input = None;

        // Reset type to CAN
        self.new_channel_type = crate::models::ChannelType::CAN;

        self.editing_channel_index = None;
        cx.notify();
    }

    /// Show the library dialog
    pub fn show_library_dialog(
        &mut self,
        dialog_type: super::state::LibraryDialogType,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.library_dialog_type = dialog_type;
        self.show_library_dialog = true;

        // Initialize input states when dialog is shown
        if self.library_name_input.is_none() {
            self.library_name_input =
                Some(cx.new(|cx| InputState::new(window, cx).placeholder("Enter library name...")));
        }

        cx.notify();
    }

    /// Hide the library dialog
    pub fn hide_library_dialog(&mut self, cx: &mut Context<Self>) {
        self.show_library_dialog = false;
        self.new_library_name.clear();
        self.new_version_name.clear();
        cx.notify();
    }

    /// Quick import a database file
    pub fn quick_import_database(&mut self, cx: &mut Context<Self>) {
        // TODO: File dialog integration requires fixing GPUI async lifetime issues on Windows
        self.status_msg =
            "Quick import temporarily unavailable. Please use library management interface.".into();
        cx.notify();
    }
}
