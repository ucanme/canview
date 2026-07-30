//! CanViewerApp implementation blocks
//!
//! This file contains all impl blocks for CanViewerApp.

use super::state::{AppView, CanViewerApp, LibraryManager};
use crate::AppConfig;
use crate::ChannelType;
use blf::{BlfResult, LogObject};
use gpui::{prelude::*, *};
use gpui_component::input::InputState;
use parser::dbc::DbcDatabase;
use parser::ldf::LdfDatabase;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

static FILE_ID_COUNTER: AtomicU32 = AtomicU32::new(1);

fn next_file_id() -> u32 {
    FILE_ID_COUNTER.fetch_add(1, Ordering::Relaxed)
}

impl CanViewerApp {
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
            active_library_id: None,
            active_version_name: None,
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
            plot_hover_y: None,
            plot_width_px: px(0.0),
            plot_scroll_handle: gpui::ScrollHandle::new(),
            // File menu dropdown state
            show_file_menu: false,
            // Help menu dropdown state
            show_help_menu: false,
            selected_row_index: None,
            expanded_channels: std::collections::HashSet::new(),
            expanded_messages: std::collections::HashSet::new(),
            pending_add_channel_focus: None,
            // Signal sets
            signal_set_store: crate::library::signal_sets::load_signal_set_store(None),
            active_signal_set: None,
            pending_signal_set_name: None,
            show_save_set_input: false,
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
        };

        // 🔧 启动时加载配置
        app.load_startup_config();

        app
    }

    fn load_startup_config(&mut self) {
        // Prefer a config file next to the executable so it is always found
        // regardless of the working directory.
        let exe_config = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("multi_channel_config.json")));
        let path = exe_config
            .filter(|p| p.exists())
            .unwrap_or_else(|| PathBuf::from("multi_channel_config.json"));
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
                        // Restore active library/version state
                        if let Some(ref lib_id) = config.active_library_id.clone() {
                            self.active_library_id = Some(lib_id.clone());
                            self.active_version_name = config.active_version_name.clone();
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

    /// 单文件 Replace:清空 files,加入单个 segment,重建 merged
    pub(crate) fn apply_blf_result_single(
        &mut self,
        result: anyhow::Result<BlfResult>,
        path: PathBuf,
    ) {
        match result {
            Ok(result) => {
                let error_count = result.errors.len();
                if error_count > 0 {
                    Self::log_blf_errors(&result.errors, result.objects.len());
                }

                // Replace 模式：清空所有已有数据
                self.files.clear();
                self.merged = crate::domain::multi_file::MergedView::empty();
                self.plot_data = std::sync::Arc::from([]);
                self.plot_full_data = std::sync::Arc::from([]);
                self.selected_signals.clear();
                // 清理可能残留的 loading_progress（多文件加载未完成时单选触发）
                self.loading_progress = None;

                self.current_view = AppView::LogView;

                let file_id = next_file_id();
                let file_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|s| s.to_string());
                let segment = crate::domain::multi_file::FileSegment::from_blf_result(
                    result, file_id, path,
                );

                if error_count > 0 {
                    self.blf_parse_errors = segment.errors.clone();
                    self.status_msg = format!("{} parse error(s) — see details", error_count).into();
                } else {
                    self.blf_parse_errors.clear();
                    self.show_blf_errors_popover = false;
                    self.status_msg = format!("✅ Loaded {} messages", segment.object_count).into();
                }

                self.start_time = segment.start_time;
                self.blf_bytes_total = segment.bytes_total;
                self.blf_bytes_consumed = segment.bytes_consumed;

                self.files.push(std::sync::Arc::new(segment));
                self.rebuild_merged();
                self.messages = self.merged.messages.to_vec();
                self.current_file_name = file_name;
                self.library_picker_dismissed = false;
                self.library_picker_selected_version.clear();
            }
            Err(e) => {
                // Replace 模式失败：不清空已有数据，保留旧的 files/messages
                self.status_msg = format!("❌ File Error: {}", e).into();
                Self::display_blf_load_error(&e);
                self.blf_bytes_total = 0;
                self.blf_bytes_consumed = 0;
                self.blf_parse_errors.clear();
                self.show_blf_errors_popover = false;
            }
        }
    }

    /// 多文件 Append:单个文件解析完成时调用,追加 segment 后重建 merged
    pub(crate) fn apply_blf_result_append_one(
        &mut self,
        result: anyhow::Result<BlfResult>,
        path: PathBuf,
    ) {
        // 检查取消标志:已取消则不再追加新完成的 segment
        if let Some(p) = &self.loading_progress {
            if p.is_cancelled {
                return;
            }
        }

        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_string());

        match result {
            Ok(result) => {
                let error_count = result.errors.len();
                if error_count > 0 {
                    Self::log_blf_errors(&result.errors, result.objects.len());
                }

                let file_id = next_file_id();
                let segment = crate::domain::multi_file::FileSegment::from_blf_result(
                    result, file_id, path,
                );

                let msg_count = segment.object_count;
                self.files.push(std::sync::Arc::new(segment));
                self.rebuild_merged();
                self.messages = self.merged.messages.to_vec();
                // 更新 start_time 为所有文件中最早的 measurement_start_time
                self.start_time = self.files.iter().filter_map(|f| f.start_time).min();

                // 更新汇总进度
                if let Some(p) = &mut self.loading_progress {
                    p.completed_files += 1;
                    p.total_messages_so_far += msg_count;
                    p.current_file_name = file_name.clone();
                }

                // 更新 StatusBar
                let total_files = self.files.len();
                let total_msgs = self.messages.len();
                let failed = self.files.iter().filter(|f| !f.errors.is_empty()).count();
                let all_done = self.loading_progress.as_ref()
                    .map(|p| p.completed_files >= p.total_files)
                    .unwrap_or(true);
                if failed > 0 {
                    if all_done {
                        self.status_msg = format!(
                            "Loaded {}/{} files ({} failed) — {} messages",
                            total_files - failed, total_files, failed, total_msgs
                        ).into();
                    } else {
                        self.status_msg = format!(
                            "⏳ Loading {}/{} files ({} failed so far) — {} messages",
                            self.loading_progress.as_ref().map(|p| p.completed_files).unwrap_or(0),
                            self.loading_progress.as_ref().map(|p| p.total_files).unwrap_or(0),
                            failed, total_msgs
                        ).into();
                    }
                } else if let Some(p) = &self.loading_progress {
                    if p.completed_files < p.total_files {
                        self.status_msg = format!(
                            "⏳ Loading {}/{} files — {} messages",
                            p.completed_files, p.total_files, total_msgs
                        ).into();
                    } else {
                        self.status_msg = format!(
                            "✅ Loaded {} files, {} messages",
                            total_files, total_msgs
                        ).into();
                    }
                } else {
                    self.status_msg = format!(
                        "✅ Loaded {} files, {} messages",
                        total_files, total_msgs
                    ).into();
                }

                // 所有文件加载完成:清理 loading_progress
                if all_done {
                    self.loading_progress = None;
                }

                self.current_file_name = file_name;
            }
            Err(e) => {
                // Append 模式单文件失败:保留失败文件占位,messages 为空
                let file_id = next_file_id();
                let segment = crate::domain::multi_file::FileSegment {
                    file_id,
                    path: path.clone(),
                    file_name: file_name.clone().unwrap_or_else(|| "unknown".to_string()),
                    start_time: None,
                    start_ns: 0,
                    messages: std::sync::Arc::from([]),
                    errors: vec![format!("{}", e)],
                    bytes_total: 0,
                    bytes_consumed: 0,
                    object_count: 0,
                    time_min: None,
                    time_max: None,
                };
                self.files.push(std::sync::Arc::new(segment));

                let all_done = if let Some(p) = &mut self.loading_progress {
                    p.completed_files += 1;
                    p.current_file_name = file_name;
                    p.completed_files >= p.total_files
                } else {
                    true
                };

                let total_files = self.files.len();
                let failed = self.files.iter().filter(|f| !f.errors.is_empty()).count();
                self.status_msg = format!(
                    "Loaded {}/{} files ({} failed) — see Files",
                    total_files - failed, total_files, failed
                ).into();

                // 所有文件加载完成:清理 loading_progress
                if all_done {
                    self.loading_progress = None;
                }
            }
        }
    }

    /// 重建 merged 视图(所有 file 增删后调用)
    pub(crate) fn rebuild_merged(&mut self) {
        let segments: Vec<std::sync::Arc<crate::domain::multi_file::FileSegment>> =
            self.files.iter().cloned().collect();
        self.merged = crate::domain::multi_file::MergedView::from_segments(&segments);
    }

    /// 移除单个文件
    pub(crate) fn remove_file(&mut self, file_id: u32) {
        self.files.retain(|f| f.file_id != file_id);
        self.rebuild_merged();
        self.messages = self.merged.messages.to_vec();
        // 更新 start_time 为剩余文件中最早的 measurement_start_time
        self.start_time = self.files.iter().filter_map(|f| f.start_time).min();
        let total_files = self.files.len();
        let total_msgs = self.messages.len();
        if total_files == 0 {
            self.status_msg = "Ready".into();
            self.current_file_name = None;
            self.start_time = None;
            self.plot_data = std::sync::Arc::from([]);
            self.plot_full_data = std::sync::Arc::from([]);
            self.selected_signals.clear();
            self.blf_bytes_total = 0;
            self.blf_bytes_consumed = 0;
        } else {
            self.status_msg = format!("✅ Loaded {} files, {} messages", total_files, total_msgs).into();
        }
    }

    /// 移除所有文件
    pub(crate) fn remove_all_files(&mut self) {
        // 若正在加载,标记取消以阻止待处理任务重新填充 files
        if let Some(p) = &mut self.loading_progress {
            p.is_cancelled = true;
        }
        self.loading_progress = None;
        self.files.clear();
        self.merged = crate::domain::multi_file::MergedView::empty();
        self.messages.clear();
        self.plot_data = std::sync::Arc::from([]);
        self.plot_full_data = std::sync::Arc::from([]);
        self.selected_signals.clear();
        self.current_file_name = None;
        self.start_time = None;
        self.blf_bytes_total = 0;
        self.blf_bytes_consumed = 0;
        self.blf_parse_errors.clear();
        self.show_blf_errors_popover = false;
        self.show_files_popover = false;
        self.status_msg = "Ready".into();
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
        // `timestamp` 现在是绝对 Unix 纳秒(abs_ns),由 MergedView::from_segments
        // 在合并时写入 LogObject.timestamp()。把它直接转成 NaiveDateTime 显示。
        use chrono::{TimeZone, Utc};
        let dt = Utc.timestamp_nanos(timestamp as i64);
        dt.naive_utc().format("%Y-%m-%d %H:%M:%S%.6f").to_string()
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
            .join("")
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
    /// Save the current configuration to file.
    ///
    /// Uses the existing config file path when available. On first save, stores
    /// the config next to the executable so it is location-independent.
    pub(crate) fn save_config(&self, cx: &mut Context<Self>) {
        let config_path = self.config_file_path.clone().unwrap_or_else(|| {
            std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|d| d.join("multi_channel_config.json")))
                .unwrap_or_else(|| PathBuf::from("multi_channel_config.json"))
        });
        if let Ok(content) = serde_json::to_string_pretty(&self.app_config) {
            if std::fs::write(&config_path, content).is_ok() {
                cx.notify();
            }
        }
    }
}
impl CanViewerApp {
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
        self.plot_hover_y = None;

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
                            title: Some("can-viewer".into()),
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
                            title: Some("can-viewer".into()),
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
            active_library_id: None,
            active_version_name: None,
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
            plot_hover_y: None,
            plot_width_px: px(0.0),
            plot_scroll_handle: gpui::ScrollHandle::new(),
            // File menu dropdown state
            show_file_menu: false,
            // Help menu dropdown state
            show_help_menu: false,
            selected_row_index: None,
            expanded_channels: std::collections::HashSet::new(),
            expanded_messages: std::collections::HashSet::new(),
            pending_add_channel_focus: None,
            // Signal sets
            signal_set_store: crate::library::signal_sets::load_signal_set_store(None),
            active_signal_set: None,
            pending_signal_set_name: None,
            show_save_set_input: false,
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
impl CanViewerApp {
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
        // Collect cleanup info BEFORE deleting it
        let (library_name, channel_ids_to_remove): (Option<String>, Vec<u16>) = self
            .library_manager
            .find_library(library_id)
            .map(|lib| (
                Some(lib.name.clone()),
                lib.versions
                    .iter()
                    .flat_map(|v| v.channel_databases.iter().map(|db| db.channel_id))
                    .collect(),
            ))
            .unwrap_or((None, Vec::new()));

        // Remove any channel mappings that reference this library before deletion
        // so the is_used check doesn't block it
        for mapping in self.app_config.mappings.iter_mut() {
            if mapping.library_id.as_deref() == Some(library_id) {
                mapping.library_id = None;
                mapping.version_name = None;
            }
        }
        // Clear active library state if it references this library
        if self.active_library_id.as_deref() == Some(library_id) {
            self.active_library_id = None;
            self.active_version_name = None;
            self.app_config.active_library_id = None;
            self.app_config.active_version_name = None;
        }

        match self
            .library_manager
            .delete_library(library_id, &self.app_config.mappings)
        {
            Ok(_) => {
                let cleanup_result = if let Some(library_name) = library_name {
                    let libraries_dir =
                        crate::library::libraries_base_path(self.config_file_path.as_deref());
                    crate::library::delete_library_from_libraries(&libraries_dir, &library_name)
                        .map_err(|e| e.to_string())
                } else {
                    Ok(())
                };

                self.status_msg = match cleanup_result {
                    Ok(()) => "Library deleted".into(),
                    Err(e) => format!("Library deleted, but local files cleanup failed: {}", e).into(),
                };
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
        let cleanup_info = self
            .library_manager
            .find_library(library_id)
            .and_then(|library| {
                library
                    .versions
                    .iter()
                    .find(|version| version.name == version_name)
                    .map(|version| {
                        (
                            library.name.clone(),
                            version
                                .channel_databases
                                .iter()
                                .map(|db| db.channel_id)
                                .collect::<Vec<_>>(),
                        )
                    })
            });

        // Clean up mappings referencing this version before deletion
        for mapping in self.app_config.mappings.iter_mut() {
            if mapping.library_id.as_deref() == Some(library_id)
                && mapping.version_name.as_deref() == Some(version_name)
            {
                mapping.library_id = None;
                mapping.version_name = None;
            }
        }
        // Clear active state if this version was active
        if self.active_library_id.as_deref() == Some(library_id)
            && self.active_version_name.as_deref() == Some(version_name)
        {
            self.active_library_id = None;
            self.active_version_name = None;
            self.app_config.active_library_id = None;
            self.app_config.active_version_name = None;
        }

        match self.library_manager.remove_version(
            library_id,
            version_name,
            &self.app_config.mappings,
        ) {
            Ok(_) => {
                if self.selected_library_id.as_deref() == Some(library_id)
                    && self.selected_version_id.as_deref() == Some(version_name)
                {
                    self.selected_version_id = None;
                }

                if let Some((library_name, channel_ids)) = cleanup_info {
                    let libraries_dir =
                        crate::library::libraries_base_path(self.config_file_path.as_deref());
                    let cleanup_result = crate::library::delete_version_from_libraries(
                        &libraries_dir,
                        &library_name,
                        version_name,
                    )
                    .map_err(|e| e.to_string());

                    for ch_id in channel_ids {
                        self.dbc_channels.remove(&ch_id);
                        self.ldf_channels.remove(&ch_id);
                    }

                    self.status_msg = match cleanup_result {
                        Ok(()) => format!("Version '{}' deleted", version_name).into(),
                        Err(e) => format!(
                            "Version '{}' deleted, but local files cleanup failed: {}",
                            version_name, e
                        )
                        .into(),
                    };
                } else {
                    self.status_msg = format!("Version '{}' deleted", version_name).into();
                }

                self.app_config.libraries = self.library_manager.libraries().to_vec();
                self.save_config(cx);
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

                // Migrate signal-set store key from old library id to new id
                if let Some(sets) = self.signal_set_store.sets_by_library.remove(&old_id) {
                    self.signal_set_store.sets_by_library.insert(new_id.clone(), sets);
                    let _ = crate::library::save_signal_set_store(
                        &self.signal_set_store,
                        self.config_file_path.as_deref(),
                    );
                }
                if let Some((lid, _)) = &self.active_signal_set {
                    if lid == &old_id {
                        let set_name = self.active_signal_set.as_ref().unwrap().1.clone();
                        self.active_signal_set = Some((new_id.clone(), set_name));
                    }
                }

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

        // 刷新 plot 数据：库版本变了，已选信号对应的 series 要重新从新 DBC 提取
        crate::ui::views::chart_view::extract_and_update_series_data(self);

        self.status_msg =
            format!("✅ Applied version {} to all plot channels", version_name).into();
        cx.notify();
    }

    /// Activate a library version for use in log decoding and plot
    pub fn activate_library_version(
        &mut self,
        library_id: &str,
        version_name: &str,
        cx: &mut Context<Self>,
    ) {
        self.apply_version_to_mappings(library_id, version_name, cx);
        self.active_library_id = Some(library_id.to_string());
        self.active_version_name = Some(version_name.to_string());
        // Persist active state so it survives restarts
        self.app_config.active_library_id = Some(library_id.to_string());
        self.app_config.active_version_name = Some(version_name.to_string());
        // 右侧 lib badge 已经显示 `📚 lib_name / version_name`,
        // status_msg 不再重复库名,避免激活时出现两个 library 引用。
        self.status_msg = "✅ Library activated".into();
        self.save_config(cx);
        cx.notify();
    }

    /// Deactivate the currently active library version, if any.
    ///
    /// Clears the in-memory active state AND the persisted config so the
    /// picker overlay can re-appear (when a BLF is loaded) and other
    /// libraries can be activated next.
    pub fn deactivate_library_version(&mut self, cx: &mut Context<Self>) {
        let was_active = self.active_library_id.is_some();
        self.active_library_id = None;
        self.active_version_name = None;
        self.app_config.active_library_id = None;
        self.app_config.active_version_name = None;
        // Clear library_id/version_name on all channel mappings so the
        // signal decoder stops producing decoded signals.
        for mapping in &mut self.app_config.mappings {
            mapping.library_id = None;
            mapping.version_name = None;
        }
        if was_active {
            self.status_msg = "Library deactivated".into();
            self.save_config(cx);
        }
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

    /// Reset plot state without triggering a re-render. Used by tests and by
    /// `clear_selected_signals`. Splits the data-reset from the notify so the
    /// reset can be unit-tested without a context.
    pub fn clear_plot_state(&mut self) {
        self.selected_signals.clear();
        self.plot_data = std::sync::Arc::from([]);
        self.plot_full_data = std::sync::Arc::from([]);
        self.plot_zoom_start = None;
        self.plot_zoom_end = None;
    }

    /// Clear all selected signals and the current plot data. Bound to the
    /// "Clear all" button in the plot sidebar.
    pub fn clear_selected_signals(&mut self, cx: &mut Context<Self>) {
        self.clear_plot_state();
        cx.notify();
    }

    /// Toggle the expansion of a channel in the plot sidebar.
    /// Uses accordion mode: expanding channel B collapses all other channels
    /// (and removes their messages from `expanded_messages`).
    pub fn toggle_channel_expanded(&mut self, ch_id: u16) {
        if self.expanded_channels.contains(&ch_id) {
            self.expanded_channels.remove(&ch_id);
        } else {
            self.expanded_channels.clear();
            self.expanded_channels.insert(ch_id);
            // Drop messages belonging to other channels
            self.expanded_messages.retain(|(c, _)| *c == ch_id);
        }
    }

    /// Toggle the expansion of a message in the plot sidebar.
    /// Multiple messages can be expanded simultaneously (no accordion).
    pub fn toggle_message_expanded(&mut self, ch_id: u16, msg_id: u32) {
        let key = (ch_id, msg_id);
        if self.expanded_messages.contains(&key) {
            self.expanded_messages.remove(&key);
        } else {
            self.expanded_messages.insert(key);
        }
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

        // 🔧 自动复制文件到 libraries/{library}/{version}/{channel}/
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

        let source_path = std::path::Path::new(&self.new_channel_db_path);
        let libraries_dir = crate::library::libraries_base_path(self.config_file_path.as_deref());
        match crate::library::copy_database_to_libraries(
            &libraries_dir,
            &library_name,
            &version_name,
            self.new_channel_name.trim(),
            source_path,
        ) {
            Ok(local_path) => {
                channel_db.database_path = local_path.to_string_lossy().to_string();
                eprintln!("✅ Database file copied to local storage: {:?}", local_path);
            }
            Err(e) => {
                self.status_msg = format!("Failed to copy database file: {}", e).into();
                cx.notify();
                return;
            }
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

                    // Auto-reload if this version is currently active
                    let is_active_version = self.active_library_id.as_deref() == Some(library_id.as_str())
                        && self.active_version_name.as_deref() == Some(version_name.as_str());
                    if is_active_version {
                        self.apply_version_to_mappings(&library_id.clone(), &version_name.clone(), cx);
                    }

                    // 刷新 plot 数据（即使是非激活版本路径，也是 no-op 无害）
                    crate::ui::views::chart_view::extract_and_update_series_data(self);

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

        let library_name = library.name.clone();
        if let Some(version) = library.versions.iter_mut().find(|v| v.name == version_name) {
            let channel_name = version
                .channel_databases
                .iter()
                .find(|db| db.channel_id == channel_id)
                .map(|db| db.channel_name.clone());

            // Remove from configuration
            version
                .channel_databases
                .retain(|db| db.channel_id != channel_id);
            version.path = version
                .channel_databases
                .first()
                .map(|db| db.database_path.clone())
                .unwrap_or_default();

            // Remove from runtime cache
            self.dbc_channels.remove(&channel_id);
            self.ldf_channels.remove(&channel_id);

            // Clean up any mapping entry for this channel if it referenced the active version
            let is_active_version = self.active_library_id.as_deref() == Some(library_id.as_str())
                && self.active_version_name.as_deref() == Some(version_name.as_str());
            if is_active_version {
                for mapping in self.app_config.mappings.iter_mut() {
                    if mapping.channel_id == channel_id
                        && mapping.library_id.as_deref() == Some(library_id.as_str())
                    {
                        mapping.library_id = None;
                        mapping.version_name = None;
                    }
                }
            }

            // Sync to app config
            self.app_config.libraries = self.library_manager.libraries().to_vec();

            // Save to disk
            self.save_config(cx);

            let cleanup_result = if let Some(channel_name) = channel_name {
                let libraries_dir =
                    crate::library::libraries_base_path(self.config_file_path.as_deref());
                crate::library::delete_channel_from_libraries(
                    &libraries_dir,
                    &library_name,
                    &version_name,
                    &channel_name,
                )
                .map_err(|e| e.to_string())
            } else {
                Ok(())
            };

            // 若删除的是激活版本的通道，刷新 plot 数据
            if is_active_version {
                crate::ui::views::chart_view::extract_and_update_series_data(self);
            }

            self.status_msg = match cleanup_result {
                Ok(()) => format!("Channel {} deleted", channel_id).into(),
                Err(e) => format!(
                    "Channel {} deleted, but local files cleanup failed: {}",
                    channel_id, e
                )
                .into(),
            };
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{DataPoint, Series};

    fn make_app_with_selected(signal_id: &str) -> CanViewerApp {
        let mut app = CanViewerApp::new_state();
        app.selected_signals = vec![signal_id.to_string()];
        let series = Series {
            name: signal_id.to_string(),
            color: gpui::hsla(0.0, 0.0, 0.0, 1.0),
            unit: None,
            points: vec![DataPoint { time: 0.0, value: 1.0, index: 0 }].into(),
            time_labels: vec![],
        };
        app.plot_data = std::sync::Arc::from([series]);
        app.plot_zoom_start = Some(0.0);
        app.plot_zoom_end = Some(10.0);
        app
    }

    #[test]
    fn clear_selected_signals_resets_plot() {
        let mut app = make_app_with_selected("CAN:1:0x100:EngineSpeed");
        app.clear_plot_state();
        assert!(app.selected_signals.is_empty());
        assert!(app.plot_data.is_empty());
        assert!(app.plot_zoom_start.is_none());
        assert!(app.plot_zoom_end.is_none());
    }

    #[test]
    fn toggle_channel_expanded_accordion_mode() {
        let mut app = CanViewerApp::new_state();
        // Expand channel 1
        app.toggle_channel_expanded(1);
        assert!(app.expanded_channels.contains(&1));
        // Expand channel 2 — channel 1 should collapse (accordion)
        app.toggle_channel_expanded(2);
        assert!(!app.expanded_channels.contains(&1));
        assert!(app.expanded_channels.contains(&2));
        // Collapse channel 2
        app.toggle_channel_expanded(2);
        assert!(app.expanded_channels.is_empty());
    }

    #[test]
    fn toggle_channel_expanded_clears_other_channel_messages() {
        let mut app = CanViewerApp::new_state();
        // Expand channel 1 + a message under it
        app.toggle_channel_expanded(1);
        app.toggle_message_expanded(1, 0x100);
        assert!(app.expanded_messages.contains(&(1, 0x100)));
        // Expand channel 2 — messages belonging to channel 1 should be removed
        app.toggle_channel_expanded(2);
        assert!(!app.expanded_messages.contains(&(1, 0x100)));
    }

    #[test]
    fn toggle_message_expanded_independent_per_message() {
        let mut app = CanViewerApp::new_state();
        app.toggle_message_expanded(1, 0x100);
        app.toggle_message_expanded(1, 0x200);
        assert!(app.expanded_messages.contains(&(1, 0x100)));
        assert!(app.expanded_messages.contains(&(1, 0x200)));
        // Collapse one
        app.toggle_message_expanded(1, 0x100);
        assert!(!app.expanded_messages.contains(&(1, 0x100)));
        assert!(app.expanded_messages.contains(&(1, 0x200)));
    }
}
