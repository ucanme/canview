//! CanViewApp implementation blocks
//!
//! This file contains all impl blocks for CanViewApp.

use super::state::{AppView, CanViewApp, LibraryManager, ScrollbarDragState};
use crate::AppConfig;
use crate::ChannelType;
use crate::rendering::calculate_column_widths;
use blf::{BlfResult, LogObject, read_blf_from_file};
use gpui::{prelude::*, *};
use gpui_component::input::{InputEvent, InputState};
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
            dropdown_just_opened: false,
            // Channel filter
            channel_filter: None,
            channel_filter_text: "".into(),
            show_channel_filter_input: false,
            channel_filter_scroll_offset: px(0.0),
            channel_filter_scroll_handle: gpui::UniformListScrollHandle::new(),
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
            // Channel configuration dialog
            show_channel_config_dialog: false,
            new_channel_id: String::new(),
            new_channel_name: String::new(),
            new_channel_db_path: String::new(),
            editing_channel_index: None,
            channel_id_input: None,
            channel_name_input: None,
            show_add_channel_input: true,
            channel_db_path_input: None,
            new_channel_type: ChannelType::CAN,
            pending_file_path: None,
            // Deprecated fields for backward compatibility
            focused_library_input: None,
            is_editing_library_name: false,
            library_input_state: crate::ui::components::ime_text_input::ImeTextInputState::default(
            ),
            library_focus_handle: None,
            ime_handler_registered: false,
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

                            // 统计信息
                            let total_versions: usize = self
                                .library_manager
                                .libraries()
                                .iter()
                                .map(|lib| lib.versions.len())
                                .sum();
                            let total_channels: usize = self
                                .library_manager
                                .libraries()
                                .iter()
                                .flat_map(|lib| &lib.versions)
                                .map(|ver| ver.channel_databases.len())
                                .sum();

                            eprintln!("  ✅ 加载完成:");
                            eprintln!("     - {} 个库", self.library_manager.libraries().len());
                            eprintln!("     - {} 个版本", total_versions);
                            eprintln!("     - {} 个通道", total_channels);

                            // 显示库列表
                            for library in self.library_manager.libraries() {
                                eprintln!(
                                    "     📦 {}: {} 个版本",
                                    library.name,
                                    library.versions.len()
                                );
                            }

                            self.status_msg = format!(
                                "Configuration loaded: {} libraries, {} versions, {} channels",
                                self.library_manager.libraries().len(),
                                total_versions,
                                total_channels
                            )
                            .into();
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

    fn apply_blf_result(&mut self, result: anyhow::Result<BlfResult>) {
        match result {
            Ok(result) => {
                self.status_msg = format!("Loaded BLF: {} objects", result.objects.len()).into();

                // === 调试输出：检查时间戳 ===
                println!("\n=== BLF 时间戳诊断 ===");
                println!("基准时间: {:?}", result.file_stats.measurement_start_time);
                println!("总消息数: {}", result.objects.len());

                // 检查前 10 条消息的时间戳
                println!("\n前 10 条消息的时间戳:");
                for (i, obj) in result.objects.iter().take(10).enumerate() {
                    let ts = obj.timestamp();
                    println!(
                        "  Message {}: {} ns ({:.9} s)",
                        i,
                        ts,
                        ts as f64 / 1_000_000_000.0
                    );
                }

                // 检查时间戳是否都相同
                if result.objects.len() > 1 {
                    let first_ts = result.objects[0].timestamp();
                    let last_ts = result.objects.last().unwrap().timestamp();
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

                // Parse start time
                let st = result.file_stats.measurement_start_time.clone();
                let date_opt =
                    chrono::NaiveDate::from_ymd_opt(st.year as i32, st.month as u32, st.day as u32);
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

    fn load_config(&mut self, _cx: &mut Context<Self>) {
        // TODO: File dialog integration requires fixing GPUI async lifetime issues on Windows
        self.status_msg =
            "Config loading temporarily unavailable. Please use command-line arguments.".into();
    }

    fn import_database_file(&mut self, _cx: &mut Context<Self>) {
        // TODO: File dialog integration requires fixing GPUI async lifetime issues on Windows
        self.status_msg =
            "Database import temporarily unavailable. Please use library management.".into();
    }
    fn get_timestamp_string(&self, timestamp: u64) -> String {
        if let Some(start) = &self.start_time {
            let msg_time = *start + chrono::Duration::nanoseconds(timestamp as i64);
            // Format: YYYY-MM-DD HH:MM:SS.mmmmmm (microseconds)
            msg_time.format("%Y-%m-%d %H:%M:%S%.6f").to_string()
        } else {
            // If no start time, show nanoseconds as seconds with microsecond precision
            format!("{:.6}", timestamp as f64 / 1_000_000_000.0)
        }
    }

    #[allow(dead_code)]
    fn render_message_row(&self, msg: &LogObject, index: usize) -> impl IntoElement {
        let (time_str, channel_id, msg_type, id_str, dlc_str, data_str, signals_str) = match msg {
            LogObject::CanMessage(can_msg) => {
                let timestamp = can_msg.header.object_time_stamp;
                let time_str = self.get_timestamp_string(timestamp);
                let actual_data_len = can_msg.data.len().min(can_msg.dlc as usize);
                let data_hex = can_msg
                    .data
                    .iter()
                    .take(actual_data_len)
                    .map(|b| format!("{:02X}", b))
                    .collect::<Vec<_>>()
                    .join(" ");
                let signals = if let Some(db) = self.dbc_channels.get(&can_msg.channel) {
                    if let Some(message) = db.messages.get(&can_msg.id) {
                        message
                            .signals
                            .iter()
                            .map(|(name, signal)| {
                                let val = signal.decode(&can_msg.data);
                                format!("{}={:.2}", name, val)
                            })
                            .collect::<Vec<_>>()
                            .join(", ")
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                };

                (
                    time_str,
                    can_msg.channel,
                    "CAN".to_string(),
                    format!("0x{:03X}", can_msg.id),
                    actual_data_len.to_string(),
                    data_hex,
                    signals,
                )
            }
            LogObject::LinMessage(lin_msg) => {
                let timestamp = lin_msg.header.object_time_stamp;
                let time_str = self.get_timestamp_string(timestamp);
                let actual_data_len = lin_msg.data.len().min(lin_msg.dlc as usize);
                let data_hex = lin_msg
                    .data
                    .iter()
                    .take(actual_data_len)
                    .map(|b| format!("{:02X}", b))
                    .collect::<Vec<_>>()
                    .join(" ");

                let signals = if let Some(db) = self.ldf_channels.get(&lin_msg.channel) {
                    // Search for the frame with the matching ID
                    if let Some(frame) = db.frames.values().find(|f| f.id == lin_msg.id as u32) {
                        frame
                            .signals
                            .iter()
                            .filter_map(|mapping| {
                                db.signals
                                    .get(&mapping.signal_name)
                                    .map(|sig| (mapping, sig))
                            })
                            .map(|(mapping, signal)| {
                                let val = signal.decode(&lin_msg.data, mapping.offset);
                                format!("{}={}", signal.name, val)
                            })
                            .collect::<Vec<_>>()
                            .join(", ")
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                };

                (
                    time_str,
                    lin_msg.channel,
                    "LIN".to_string(),
                    format!("0x{:02X}", lin_msg.id),
                    actual_data_len.to_string(),
                    data_hex,
                    signals,
                )
            }
            _ => (
                "Unknown".to_string(),
                0,
                "Other".to_string(),
                "-".to_string(),
                "-".to_string(),
                "-".to_string(),
                String::new(),
            ),
        };

        let bg_color = if index.is_multiple_of(2) {
            rgb(0x09090b) // Zed's dark background (zebra)
        } else {
            rgb(0x0c0c0e) // Zed's dark background (base)
        };

        div()
            .flex()
            .w_full()
            .min_h(px(24.)) // Slightly taller for better readability
            .bg(bg_color)
            .border_b_1()
            .border_color(rgb(0x2a2a2a)) // Semi-transparent border like Zed
            .items_center()
            .text_sm() // Slightly larger text like Zed
            .text_color(rgb(0xcdd6f4)) // Zed's default text color
            .hover(|style| style.bg(rgb(0x1f1f1f))) // Subtle hover like Zed
            .cursor_pointer()
            .child(
                div()
                    .w(px(100.))
                    .px_3()
                    .py_1()
                    .text_color(rgb(0x646473)) // Zed's muted color
                    .child(time_str),
            )
            .child(
                div()
                    .w(px(40.))
                    .px_2()
                    .py_1()
                    .text_color(rgb(0x7dcfff)) // Zed's blue
                    .child(channel_id.to_string()),
            )
            .child(
                div()
                    .w(px(50.))
                    .px_2()
                    .py_1()
                    .text_color(rgb(0xa6e3a1)) // Zed's green
                    .child(msg_type),
            )
            .child(
                div()
                    .w(px(70.))
                    .px_2()
                    .py_1()
                    .text_color(rgb(0xf9e2af)) // Zed's yellow
                    .child(id_str),
            )
            .child(div().w(px(40.)).px_2().py_1().child(dlc_str))
            .child(
                div()
                    .w(px(150.))
                    .px_2()
                    .py_1()
                    .text_color(rgb(0xb4befe)) // Zed's purple
                    .child(data_str),
            )
            .child(
                div()
                    .flex_1()
                    .px_2()
                    .py_1()
                    .text_color(rgb(0x9399b2)) // Zed's comment color
                    .child(signals_str),
            )
    }

    /// Import a database file
    /// Save the current configuration to file
    fn save_config(&self, cx: &mut Context<Self>) {
        let config_path = PathBuf::from("multi_channel_config.json");
        if let Ok(content) = serde_json::to_string_pretty(&self.app_config) {
            if std::fs::write(&config_path, content).is_ok() {
                cx.notify();
            }
        }
    }
}
impl CanViewApp {
    fn toggle_maximize(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // Initialize display bounds on first use
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

        if self.is_maximized {
            // Restore to normal size - create new window with saved bounds
            if let Some(saved_bounds) = self.saved_window_bounds {
                // Clone all necessary state
                let current_view = self.current_view;
                let messages = self.messages.clone();
                let status_msg = self.status_msg.clone();
                let dbc_channels = self.dbc_channels.clone();
                let ldf_channels = self.ldf_channels.clone();
                let app_config = self.app_config.clone();
                let selected_signals = self.selected_signals.clone();
                let start_time = self.start_time;
                let config_dir = self.config_dir.clone();
                let config_file_path = self.config_file_path.clone();
                let display_bounds = self.display_bounds;

                // Open new window with saved bounds
                cx.open_window(
                    WindowOptions {
                        window_bounds: Some(WindowBounds::Windowed(saved_bounds)),
                        titlebar: Some(TitlebarOptions {
                            title: Some("CANVIEW - Bus Data Analyzer".into()),
                            appears_transparent: true,
                            traffic_light_position: None,
                        }),
                        kind: gpui::WindowKind::Normal,
                        ..Default::default()
                    },
                    |_window, cx| {
                        cx.new(|_| {
                            Self::new_with_state(
                                current_view,
                                messages,
                                status_msg,
                                dbc_channels,
                                ldf_channels,
                                app_config,
                                selected_signals,
                                start_time,
                                config_dir,
                                config_file_path,
                                false, // is_maximized = false
                                None,  // saved_window_bounds = None
                                display_bounds,
                            )
                        })
                    },
                )
                .ok();

                // Close current window
                window.remove_window();
            }
        } else {
            // Save current bounds before maximizing
            let current_bounds = window.bounds();
            self.saved_window_bounds = Some(current_bounds);

            // Clone all necessary state
            let current_view = self.current_view;
            let messages = self.messages.clone();
            let status_msg = self.status_msg.clone();
            let dbc_channels = self.dbc_channels.clone();
            let ldf_channels = self.ldf_channels.clone();
            let app_config = self.app_config.clone();
            let selected_signals = self.selected_signals.clone();
            let start_time = self.start_time;
            let config_dir = self.config_dir.clone();
            let config_file_path = self.config_file_path.clone();
            let display_bounds = self.display_bounds;

            // Open new maximized window
            if let Some(maximized_bounds) = self.display_bounds {
                cx.open_window(
                    WindowOptions {
                        window_bounds: Some(WindowBounds::Windowed(maximized_bounds)),
                        titlebar: Some(TitlebarOptions {
                            title: Some("CANVIEW - Bus Data Analyzer".into()),
                            appears_transparent: true,
                            traffic_light_position: None,
                        }),
                        kind: gpui::WindowKind::Normal,
                        ..Default::default()
                    },
                    |_window, cx| {
                        cx.new(|_| {
                            Self::new_with_state(
                                current_view,
                                messages,
                                status_msg,
                                dbc_channels,
                                ldf_channels,
                                app_config,
                                selected_signals,
                                start_time,
                                config_dir,
                                config_file_path,
                                true,                 // is_maximized = true
                                Some(current_bounds), // saved_window_bounds
                                display_bounds,
                            )
                        })
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
            dropdown_just_opened: false,
            // Channel filter
            channel_filter: None,
            channel_filter_text: "".into(),
            show_channel_filter_input: false,
            channel_filter_scroll_offset: px(0.0),
            channel_filter_scroll_handle: gpui::UniformListScrollHandle::new(),
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
            library_input_state: crate::ui::components::ime_text_input::ImeTextInputState::default(
            ),
            library_focus_handle: None,
            ime_handler_registered: false,
        };

        // Load startup config (this will reset some state, so do it carefully)
        // We skip loading config if we're restoring state
        if !is_maximized {
            app.load_startup_config();
        }

        app
    }

    fn update_container_height(&mut self, window: &mut Window) {
        // Get window bounds
        let window_size = window.bounds();
        let window_height = f32::from(window_size.size.height);

        // Calculate actual list container height
        // Window height - top bar (56px) - status bar (24px) - log header (28px)
        let container_height = window_height - 56.0 - 24.0 - 28.0;

        // Only update if it changed significantly (more than 10px difference)
        if (container_height - self.list_container_height).abs() > 10.0 {
            self.list_container_height = container_height;
        }
    }

    fn render_library_view(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        use crate::ui::views::library_management::render_library_management_view;

        // Initialize input states if needed (only do this once)
        // Note: We can't create InputState here without window, so we'll handle it differently
        // The Input components will be created lazily when needed

        gpui::div()
            .flex_1()
            .size_full()
            .child(render_library_management_view(
                self.library_manager.libraries(),
                &self.selected_library_id,
                &self.selected_version_id, // Add selected version ID
                &self.app_config.mappings,
                self.show_library_dialog
                    && self.library_dialog_type == super::state::LibraryDialogType::Create,
                self.show_version_input,
                &self.new_library_name,
                &self.new_version_name,
                &self.focused_library_input,
                self.library_cursor_position,
                self.new_version_cursor_position,
                self.library_name_input.as_ref(),
                self.version_name_input.as_ref(),
                self.show_add_channel_input,
                self.channel_id_input.as_ref(),
                self.channel_name_input.as_ref(),
                self.channel_db_path_input.as_ref(),
                &self.new_channel_db_path, // Add this parameter
                self.new_channel_type,     // Add channel type parameter
                cx,
            ))
    }

    fn render_log_view(&self, view: Entity<CanViewApp>) -> impl IntoElement {
        // Clone view for use in multiple closures
        let view_clone1 = view.clone();
        let view_clone2 = view.clone();

        // Apply filters (both ID and Channel)
        let filtered_messages: Vec<LogObject> = match (self.id_filter, self.channel_filter) {
            (None, None) => self.messages.clone(),
            (Some(filter_id), None) => {
                // Only ID filter
                self.messages
                    .iter()
                    .filter(|msg| match msg {
                        LogObject::CanMessage(can_msg) => can_msg.id == filter_id,
                        LogObject::CanMessage2(can_msg) => can_msg.id == filter_id,
                        LogObject::CanFdMessage(fd_msg) => fd_msg.id == filter_id,
                        LogObject::CanFdMessage64(fd_msg) => fd_msg.id == filter_id,
                        LogObject::LinMessage(lin_msg) => lin_msg.id as u32 == filter_id,
                        LogObject::LinMessage2(_) => false,
                        _ => false,
                    })
                    .cloned()
                    .collect()
            }
            (None, Some(filter_ch)) => {
                // Only Channel filter
                self.messages
                    .iter()
                    .filter(|msg| match msg {
                        LogObject::CanMessage(can_msg) => can_msg.channel == filter_ch,
                        LogObject::CanMessage2(can_msg) => can_msg.channel == filter_ch,
                        LogObject::CanFdMessage(fd_msg) => fd_msg.channel == filter_ch,
                        LogObject::CanFdMessage64(fd_msg) => fd_msg.channel as u16 == filter_ch,
                        LogObject::LinMessage(lin_msg) => lin_msg.channel == filter_ch,
                        LogObject::LinMessage2(_) => false,
                        _ => false,
                    })
                    .cloned()
                    .collect()
            }
            (Some(filter_id), Some(filter_ch)) => {
                // Both filters
                self.messages
                    .iter()
                    .filter(|msg| match msg {
                        LogObject::CanMessage(can_msg) => {
                            can_msg.id == filter_id && can_msg.channel == filter_ch
                        }
                        LogObject::CanMessage2(can_msg) => {
                            can_msg.id == filter_id && can_msg.channel == filter_ch
                        }
                        LogObject::CanFdMessage(fd_msg) => {
                            fd_msg.id == filter_id && fd_msg.channel == filter_ch
                        }
                        LogObject::CanFdMessage64(fd_msg) => {
                            fd_msg.id == filter_id && fd_msg.channel as u16 == filter_ch
                        }
                        LogObject::LinMessage(lin_msg) => {
                            lin_msg.id as u32 == filter_id && lin_msg.channel == filter_ch
                        }
                        LogObject::LinMessage2(_) => false,
                        _ => false,
                    })
                    .cloned()
                    .collect()
            }
        };

        // Save filtered message count BEFORE filtered_messages is moved
        let filtered_count = filtered_messages.len();

        let dbc_channels = self.dbc_channels.clone();
        let ldf_channels = self.ldf_channels.clone();
        let start_time = self.start_time;
        let scroll_handle = self.list_scroll_handle.clone();
        let id_display_decimal = self.id_display_decimal;
        let id_filter = self.id_filter;
        let id_filter_text = self.id_filter_text.clone();

        // Calculate column widths based on ALL messages (not filtered), to keep layout consistent
        let (time_width, ch_width, type_width, id_width, dlc_width) =
            calculate_column_widths(&self.messages, &dbc_channels, &ldf_channels, start_time);

        // Clone view for use in event handlers
        let view_for_mouse_move = view.clone();
        let view_for_mouse_up = view.clone();
        let view_for_scrollbar = view.clone();
        let view_for_keyboard = view.clone();

        // Clone for dialog display
        let _id_filter_text_for_dialog = id_filter_text.clone();

        div()
            .size_full()
            .flex()
            .flex_col()
            .relative()  // Add relative positioning for absolute children
            // Handle keyboard input for ID filter
            .on_key_down(move |event, _window, cx| {
                eprintln!("Global on_key_down: keystroke={}", event.keystroke);
                // Check if filter box is active
                let show_filter = view_for_keyboard.read(cx).show_id_filter_input;
                eprintln!("  show_filter={}", show_filter);

                // If filter box is active, handle input for it
                if show_filter {
                    eprintln!("  Filter box active, handling input");
                    let keystroke_str = format!("{}", event.keystroke);
                    match keystroke_str.as_str() {
                        "backspace" => {
                            view_for_keyboard.update(cx, |app, cx| {
                                let mut text = app.id_filter_text.to_string();
                                if !text.is_empty() {
                                    text.pop();
                                    app.id_filter_text = text.into();
                                    eprintln!("  Filter text (backspace): {}", app.id_filter_text);
                                    cx.notify();
                                }
                            });
                            return;  // Don't continue to default handler
                        }
                        "escape" => {
                            view_for_keyboard.update(cx, |app, cx| {
                                app.show_id_filter_input = false;
                                eprintln!("  Filter box closed (escape)");
                                cx.notify();
                            });
                            return;
                        }
                        "enter" => {
                            view_for_keyboard.update(cx, |app, cx| {
                                // Apply filter and close
                                if let Ok(parsed_id) = u32::from_str_radix(app.id_filter_text.as_ref(), 10) {
                                    if !app.id_filter_text.is_empty() {
                                        app.id_filter = Some(parsed_id);
                                    }
                                }
                                app.show_id_filter_input = false;
                                eprintln!("  Filter applied (enter): id={:?}", app.id_filter);
                                cx.notify();
                            });
                            return;
                        }
                        _ => {
                            // Handle digit input
                            if keystroke_str.len() == 1 {
                                if let Some(ch) = keystroke_str.chars().next() {
                                    if ch.is_ascii_digit() {
                                        view_for_keyboard.update(cx, |app, cx| {
                                            let mut text = app.id_filter_text.to_string();
                                            text.push(ch);
                                            app.id_filter_text = text.into();
                                            eprintln!("  Filter text: {}", app.id_filter_text);
                                            cx.notify();
                                        });
                                        return;
                                    }
                                }
                            }
                        }
                    }
                    // For non-digit keys when filter is active, still don't pass through
                    return;
                }

                // Convert Keystroke to string for matching
                let keystroke_str = format!("{}", event.keystroke);
                match keystroke_str.as_str() {
                    // Backspace to delete
                    "backspace" => {
                        view_for_keyboard.update(cx, |app, cx| {
                            let mut text = app.id_filter_text.to_string();
                            if !text.is_empty() {
                                text.pop();
                                let new_text = text.clone();
                                app.id_filter_text = text.into();

                                if new_text.is_empty() {
                                    app.id_filter = None;
                                } else if let Ok(parsed_id) = u32::from_str_radix(&new_text, 10) {
                                    app.id_filter = Some(parsed_id);
                                } else {
                                    app.id_filter = None;
                                }
                                cx.notify();
                            }
                        });
                    }
                    // Escape to clear filter
                    "escape" => {
                        view_for_keyboard.update(cx, |app, cx| {
                            app.id_filter = None;
                            app.id_filter_text = "".into();
                            cx.notify();
                        });
                    }
                    _ => {
                        // Check if it's a single digit (0-9)
                        if keystroke_str.len() == 1 {
                            let ch = keystroke_str.chars().next().unwrap();
                            if ch.is_ascii_digit() {
                                view_for_keyboard.update(cx, |app, cx| {
                                    let mut text = app.id_filter_text.to_string();
                                    text.push(ch);
                                    let new_text = text.clone();
                                    app.id_filter_text = text.into();

                                    // Try to parse the ID
                                    if let Ok(parsed_id) = u32::from_str_radix(&new_text, 10) {
                                        app.id_filter = Some(parsed_id);
                                    }
                                    cx.notify();
                                });
                            }
                        }
                    }
                }
            })
            // Global mouse move handler for scrollbar dragging
            .on_mouse_move(move |event, _window, cx| {
                let drag_state = view_for_mouse_move.read(cx).scrollbar_drag_state.as_ref();
                let Some(drag) = drag_state else {
                    return;
                };

                // Check if left mouse button is still pressed
                // If not, clear the drag state to prevent ghost dragging
                if event.pressed_button != Some(MouseButton::Left) {
                    view_for_mouse_move.update(cx, |app, _cx| {
                        app.scrollbar_drag_state = None;
                    });
                    return;
                }

                let current_y = event.position.y;
                let container_h = view_for_mouse_move.read(cx).list_container_height;
                let row_h = 22.0;

                // Use filtered message count from drag state
                let filtered_count = drag.filtered_count;
                let total_content_height = filtered_count as f32 * row_h;
                let max_scroll_offset = (total_content_height - container_h).max(0.0);

                if max_scroll_offset <= 0.0 {
                    return;
                }

                // Calculate thumb dimensions with dynamic minimum size
                let thumb_ratio = (container_h / total_content_height).min(1.0);

                // Use same dynamic minimum thumb size
                let min_thumb_size = if filtered_count > 100 {
                    15.0
                } else if filtered_count > 50 {
                    20.0
                } else {
                    30.0
                };

                let thumb_h = (thumb_ratio * container_h).max(min_thumb_size);
                let track_h = (container_h - thumb_h).max(0.0);

                // Calculate thumb position based on mouse Y
                // Convert start_scroll_offset to thumb position at drag start
                let start_thumb_top = if max_scroll_offset > 0.0 {
                    (drag.start_scroll_offset / max_scroll_offset) * track_h
                } else {
                    0.0
                };

                // Calculate new thumb top based on mouse movement
                let delta_y = f32::from(current_y - drag.start_y);
                let new_thumb_top = (start_thumb_top + delta_y).clamp(0.0, track_h);

                // Convert thumb position back to scroll offset
                let scroll_progress = new_thumb_top / track_h;
                let new_scroll_offset = (scroll_progress * max_scroll_offset).clamp(0.0, max_scroll_offset);

                // Convert to item index based on FILTERED messages
                let visible_items = (container_h / row_h).ceil() as usize;
                let max_start_index = filtered_count.saturating_sub(visible_items);

                // Calculate target index based on scroll offset
                let target_index = ((new_scroll_offset / row_h).round() as usize).clamp(0, max_start_index);

                // Use Bottom strategy only when we're at the very end
                // This ensures the last row is visible at the bottom
                if target_index >= max_start_index.saturating_sub(1) {
                    view_for_mouse_move.read(cx).list_scroll_handle.scroll_to_item_strict(
                        filtered_count.saturating_sub(1),
                        gpui::ScrollStrategy::Bottom
                    );
                } else {
                    view_for_mouse_move.read(cx).list_scroll_handle.scroll_to_item_strict(target_index, gpui::ScrollStrategy::Top);
                }
                cx.notify(view_for_mouse_move.entity_id());
            })
            // Global mouse up handler - this will catch mouse up anywhere
            .on_mouse_up(MouseButton::Left, move |_event, _window, cx| {
                // Always clear drag state on mouse up, anywhere in the window
                view_for_mouse_up.update(cx, |app, _cx| {
                    app.scrollbar_drag_state = None;

                    // Close filter dropdowns if clicking outside
                    // Check if dropdown was just opened (in which case, don't close it)
                    if !app.dropdown_just_opened && !app.mouse_over_filter_dropdown {
                        // Close ID filter dropdown if open
                        if app.show_id_filter_input {
                            app.show_id_filter_input = false;
                        }
                        // Close channel filter dropdown if open
                        if app.show_channel_filter_input {
                            app.show_channel_filter_input = false;
                        }
                    }

                    // Reset flags after processing
                    app.mouse_over_filter_dropdown = false;
                    app.dropdown_just_opened = false;
                });
            })
            .child(
                // Zed-style header with calculated column widths and proper alignment
                div()
                    .w_full()
                    .h(px(28.))
                    .bg(rgb(0x1f1f1f))
                    .border_b_1()
                    .border_color(rgb(0x2a2a2a))
                    .flex()
                    .items_center()
                    .text_xs()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(rgb(0x9ca3af))
                    .child(
                        div()
                            .w(px(60.))
                            .px_3()
                            .py_1()
                            .flex()
                            .items_center()
                            .flex_shrink_0()
                            .whitespace_nowrap()
                            .overflow_hidden()
                            .child("#")
                    )
                    .child(
                        div()
                            .w(time_width)
                            .px_3()
                            .py_1()
                            .flex()
                            .items_center()
                            .flex_shrink_0()
                            .whitespace_nowrap()
                            .overflow_hidden()
                            .child("TIME")
                    )
                    .child(
                        {
                            let _view_for_ch_filter = view.clone();
                            div()
                                .w(ch_width)
                                .px_2()
                                .py_1()
                                .flex()
                                .items_center()
                                .flex_shrink_0()
                                .whitespace_nowrap()
                                .overflow_hidden()
                                .child("CH")
                                .child(
                                    div()
                                        .text_xs()
                                        .cursor_pointer()
                                        .text_color(if self.channel_filter.is_some() {
                                            rgb(0x60a5fa)
                                        } else {
                                            rgb(0x4b5563)
                                        })
                                        .hover(|style| style.bg(rgb(0x374151)))
                                        .rounded(px(2.))
                                        .ml_0p5()  // Small left margin to bring it closer to CH
                                        .pl_0()  // No left padding
                                        .pr_0()  // No right padding
                                        .py_0p5()
                                        .on_mouse_down(gpui::MouseButton::Left, {
                                            let view = view.clone();
                                            move |_event, _window, cx| {
                                                view.update(cx, |app, cx| {
                                                    // If filter is active, clicking clears it
                                                    // If filter is not active, clicking shows dropdown
                                                    if app.channel_filter.is_some() {
                                                        eprintln!("Clearing channel filter");
                                                        app.channel_filter = None;
                                                        app.channel_filter_text = "".into();
                                                        app.show_channel_filter_input = false;
                                                    } else {
                                                        eprintln!("Before: show_channel_filter_input={}", app.show_channel_filter_input);
                                                        app.show_channel_filter_input = !app.show_channel_filter_input;
                                                        eprintln!("After: show_channel_filter_input={}", app.show_channel_filter_input);

                                                        // If we're opening the dropdown, set the flag to prevent immediate close
                                                        if app.show_channel_filter_input {
                                                            app.dropdown_just_opened = true;
                                                        }
                                                    }
                                                    cx.notify();
                                                });
                                            }
                                        })
                                        .child(if self.channel_filter.is_some() { "✓" } else { "⚙" })
                                )
                        }
                    )
                    .child(
                        div()
                            .w(type_width)
                            .px_2()
                            .py_1()
                            .flex()
                            .items_center()
                            .flex_shrink_0()
                            .whitespace_nowrap()
                            .overflow_hidden()
                            .child("TYPE")
                    )
                    .child(
                        div()
                            .w(id_width)
                            .pl_2()  // Only left padding
                            .pr_0()  // No right padding
                            .py_1()
                            .flex()
                            .items_center()
                            .flex_shrink_0()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .child(
                                        div()
                                            .cursor_pointer()
                                            .rounded(px(2.))
                                            .pl_1()  // Left padding only
                                            .pr_0()  // No right padding
                                            .py_0p5()
                                            .hover(|style| style.bg(rgb(0x374151)))
                                            .on_mouse_down(gpui::MouseButton::Left, {
                                                let view = view.clone();
                                                move |_, _, cx| {
                                                    view.update(cx, |app, cx| {
                                                        app.id_display_decimal = !app.id_display_decimal;
                                                        cx.notify();
                                                    });
                                                }
                                            })
                                            .child(
                                                div()
                                                    .flex()
                                                    .items_center()
                                                    .gap_0p5()
                                                    .child("ID")
                                                    .child(
                                                        div()
                                                            .text_xs()
                                                            .text_color(rgb(0x6b7280))
                                                            .child(if id_display_decimal { "10" } else { "16" })
                                                    )
                                            )
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .cursor_pointer()
                                            .text_color(if id_filter.is_some() {
                                                rgb(0x60a5fa)
                                            } else {
                                                rgb(0x4b5563)
                                            })
                                            .hover(|style| style.bg(rgb(0x374151)))
                                            .rounded(px(2.))
                                            .pl_1()  // Left padding only
                                            .pr_0()  // No right padding
                                            .py_0p5()
                                            .on_mouse_down(gpui::MouseButton::Left, {
                                                let view = view.clone();
                                                move |event, _, cx| {
                                                    eprintln!("Gear clicked! Position: {:?}", event.position);
                                                    view.update(cx, |app, cx| {
                                                        // If filter is active, clicking clears it
                                                        // If filter is not active, clicking shows dropdown
                                                        if app.id_filter.is_some() {
                                                            eprintln!("Clearing filter");
                                                            app.id_filter = None;
                                                            app.id_filter_text = "".into();
                                                            app.show_id_filter_input = false;
                                                        } else {
                                                            eprintln!("Before: show_id_filter_input={}", app.show_id_filter_input);
                                                            app.show_id_filter_input = !app.show_id_filter_input;
                                                            eprintln!("After: show_id_filter_input={}", app.show_id_filter_input);

                                                            // If we're opening the dropdown, set the flag to prevent immediate close
                                                            if app.show_id_filter_input {
                                                                app.dropdown_just_opened = true;
                                                            }
                                                        }
                                                        cx.notify();
                                                    });
                                                }
                                            })
                                            .child(if id_filter.is_some() { "✓" } else { "⚙" })
                                    )
                            )
                    )
                    .child(
                        div()
                            .w(dlc_width)
                            .px_2()
                            .py_1()
                            .flex()
                            .items_center()
                            .flex_shrink_0()
                            .whitespace_nowrap()
                            .overflow_hidden()
                            .child("DLC")
                    )
                    .child(
                        div()
                            .flex_1()  // DATA列使用flex_1()占据剩余空间
                            .px_2()
                            .py_1()
                            .flex()
                            .items_center()
                            .whitespace_nowrap()
                            .child("DATA")
                    ),
            )
            .child(
                // Content area with simple list
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .relative()
                    .when(self.messages.is_empty(), |parent| {
                        // Show placeholder when no messages
                        parent.child(
                            div()
                                .flex_1()
                                .flex()
                                .items_center()
                                .justify_center()
                                .child(
                                    div()
                                        .text_lg()
                                        .text_color(rgb(0x6b7280))
                                        .child("No messages loaded. Click '📂 Open BLF' to load a file.")
                                )
                        )
                    })
                    .when(!filtered_messages.is_empty(), |parent| {
                        // Show all messages with uniform_list - it should support scrolling
                        let display_count = filtered_messages.len();
                        let view_entity = view.clone();

                        parent.child(
                            gpui::uniform_list(
                                "message-list",
                                display_count,
                                move |range: std::ops::Range<usize>, _window: &mut gpui::Window, cx: &mut gpui::App| {
                                    // Track scroll position by observing the visible range
                                    let first_visible = range.start;
                                    view_entity.update(cx, |v, _cx| {
                                        v.scroll_offset = px(first_visible as f32 * 22.0);
                                    });

                                    range
                                        .map(|index| {
                                            if let Some(msg) = filtered_messages.get(index) {
                                                Self::render_message_row_static_with_widths(
                                                    msg,
                                                    index,
                                                    time_width,
                                                    ch_width,
                                                    type_width,
                                                    id_width,
                                                    dlc_width,
                                                    &dbc_channels,
                                                    &ldf_channels,
                                                    start_time,
                                                    id_display_decimal,
                                                    view_entity.read(cx).show_id_filter_input,  // Disable hover when filter dropdown is open
                                                )
                                            } else {
                                                div().into_any_element()
                                            }
                                        })
                                        .collect::<Vec<_>>()
                                }
                            )
                            .track_scroll(&scroll_handle)
                            .flex_1()
                        )
                    })
                    .child({
                        // Calculate scrollbar dimensions based on FILTERED content
                        let row_height = 22.0;
                        let total_height = filtered_count as f32 * row_height;
                        let container_height = self.list_container_height;

                        // Smooth thumb height calculation - thumb represents proportion of visible content
                        let thumb_height_ratio = if total_height > 0.0 {
                            (container_height / total_height).min(1.0)
                        } else {
                            1.0
                        };

                        let max_scroll = (total_height - container_height).max(0.0);

                        // Improved dynamic minimum thumb size - scales smoothly with content
                        // Use a logarithmic scale for better UX across all dataset sizes
                        let min_thumb_size = if filtered_count <= 10 {
                            container_height  // Show full height for very small lists
                        } else if filtered_count <= 50 {
                            container_height * 0.5  // At least half visible for small lists
                        } else if filtered_count <= 200 {
                            40.0  // Reasonable minimum for medium lists
                        } else if filtered_count <= 1000 {
                            25.0  // Smaller for large lists
                        } else {
                            15.0  // Minimum for very large lists (still usable)
                        };

                        // Calculate thumb height with smooth transition
                        let ideal_thumb_height = thumb_height_ratio * container_height;
                        let thumb_height = ideal_thumb_height.max(min_thumb_size).min(container_height);
                        let thumb_height_px = px(thumb_height);

                        // Calculate scrollable track height (container minus thumb)
                        let track_height = (container_height - thumb_height).max(0.0);

                        // Calculate thumb position based on current scroll offset
                        let current_scroll_offset = f32::from(self.scroll_offset);
                        let thumb_top = if max_scroll > 0.0 && track_height > 0.0 {
                            // For very large datasets, scroll_offset may not reach max_scroll
                            // when using ScrollStrategy::Bottom. So we clamp the ratio.
                            let scroll_progress = (current_scroll_offset / max_scroll).min(1.0).max(0.0);

                            // Check if we're at the actual bottom
                            let container_h = self.list_container_height;
                            let row_h = 22.0_f32;
                            let visible_items = (container_h / row_h).ceil() as usize;
                            let max_start_index = filtered_count.saturating_sub(visible_items);
                            let current_start_index = (current_scroll_offset / row_h).round() as usize;

                            // If we're at the last page, force thumb to bottom
                            // This ensures the thumb visually reaches the end
                            if current_start_index >= max_start_index.saturating_sub(5) {
                                track_height
                            } else {
                                scroll_progress * track_height
                            }
                        } else {
                            0.0
                        };
                        let thumb_top_px = px(thumb_top);

                        let scroll_handle_clone = scroll_handle.clone();
                        let view_for_scrollbar_inner = view_for_scrollbar.clone();
                        let view_for_scroll_track = view_for_scrollbar.clone();

                        // Scrollbar container
                        div()
                            .absolute()
                            .right_0()
                            .top_0()
                            .bottom_0()  // Match the actual list container height
                            .w(px(12.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .bg(rgb(0x1a1a1a))
                            .child(
                                // Scrollbar track (clickable area)
                                div()
                                    .size_full()
                                    .relative()
                                    .on_mouse_down(gpui::MouseButton::Left, move |event, _window, cx| {
                                        let raw_click_y = f32::from(event.position.y);
                                        let offset_to_list = 84.0;
                                        let container_h = view_for_scroll_track.read(cx).list_container_height;
                                        let row_h = row_height;

                                        if filtered_count == 0 {
                                            return;
                                        }

                                        // Calculate thumb dimensions based on FILTERED messages with dynamic minimum size
                                        let total_content_height = filtered_count as f32 * row_h;
                                        let thumb_ratio = (container_h / total_content_height).min(1.0);

                                        // Use same improved minimum thumb size calculation as rendering
                                        let min_thumb_size = if filtered_count <= 10 {
                                            container_h
                                        } else if filtered_count <= 50 {
                                            container_h * 0.5
                                        } else if filtered_count <= 200 {
                                            40.0
                                        } else if filtered_count <= 1000 {
                                            25.0
                                        } else {
                                            15.0
                                        };

                                        let thumb_h = (thumb_ratio * container_h).max(min_thumb_size).min(container_h);
                                        let track_h = (container_h - thumb_h).max(0.0);

                                        // Adjust click position to be relative to container
                                        let click_y = (raw_click_y - offset_to_list).clamp(0.0, container_h);

                                        if track_h <= 0.0 {
                                            return;
                                        }

                                        // Calculate where thumb top should be based on click position
                                        // The click_y is in range [0, container_h], but thumb top can only be in [0, track_h]
                                        // When click_y is at bottom (container_h), thumb_top should be at track_h
                                        let scroll_ratio = click_y / container_h;
                                        let _desired_thumb_top = (scroll_ratio * track_h).clamp(0.0, track_h);

                                        // Calculate target index based on FILTERED messages
                                        let visible_items = (container_h / row_h).ceil() as usize;
                                        let max_start_index = filtered_count.saturating_sub(visible_items);

                                        let target_index = if max_start_index > 0 {
                                            (scroll_ratio * max_start_index as f32).round() as usize
                                        } else {
                                            0
                                        }.clamp(0, max_start_index);

                                        // Use Bottom strategy only when we're at the very end
                                        // This ensures the last row is visible at the bottom
                                        if target_index >= max_start_index.saturating_sub(1) {
                                            scroll_handle_clone.scroll_to_item_strict(
                                                filtered_count.saturating_sub(1),
                                                gpui::ScrollStrategy::Bottom
                                            );
                                        } else {
                                            scroll_handle_clone.scroll_to_item_strict(target_index, gpui::ScrollStrategy::Top);
                                        }
                                        cx.notify(view_for_scroll_track.entity_id());
                                    })
                                    .child(
                                        // Thumb with drag functionality
                                        div()
                                            .w(px(8.))
                                            .h(thumb_height_px)
                                            .top(thumb_top_px)
                                            .absolute()
                                            .bg(rgb(0x6a6a6a))
                                            .rounded(px(4.))
                                            .hover(|style| style.bg(rgb(0x7a7a7a)))
                                            .cursor_grab()
                                            .on_mouse_down(gpui::MouseButton::Left, {
                                                let view_for_thumb = view_for_scrollbar_inner.clone();
                                                move |event, _window, cx| {
                                                    // Initialize drag state
                                                    let start_y = event.position.y;
                                                    let start_scroll_offset = f32::from(view_for_thumb.read(cx).scroll_offset);

                                                    // Set drag state
                                                    view_for_thumb.update(cx, |app, _cx| {
                                                    app.scrollbar_drag_state = Some(ScrollbarDragState {
                                                        start_y,
                                                        start_scroll_offset,
                                                        filtered_count,
                                                    });
                                                });

                                            }
                                            })
                                    )
                            )
                    })
            )
            // Filter dropdown - SHOW ALL IDs WITH SCROLL
            .when(self.show_id_filter_input, |parent| {
                // Calculate ALL unique IDs from messages
                let mut unique_ids = std::collections::HashSet::new();
                for msg in self.messages.iter() {  // Scan ALL messages
                    match msg {
                        LogObject::CanMessage(m) => { unique_ids.insert(m.id); }
                        LogObject::CanMessage2(m) => { unique_ids.insert(m.id); }
                        LogObject::CanFdMessage(m) => { unique_ids.insert(m.id); }
                        LogObject::CanFdMessage64(m) => { unique_ids.insert(m.id); }
                        LogObject::LinMessage(m) => { unique_ids.insert(m.id as u32); }
                        _ => {}
                    }
                }
                let mut id_list: Vec<u32> = unique_ids.into_iter().collect();
                id_list.sort();

                let filter_left = 60.0 + f32::from(time_width) + f32::from(ch_width) + f32::from(type_width) + f32::from(id_width) - 40.0;

                eprintln!("=== Filter dropdown rendering ===");
                eprintln!("  Found {} unique IDs", id_list.len());

                parent.child(
                    {
                        let id_list_clone = id_list.clone();
                        let view_for_scroll = view.clone();
                        let id_list_for_wheel = id_list.clone();
                        // Clone the scroll handle for use in closures
                        let filter_scroll_handle = self.filter_scroll_handle.clone();
                        let filter_scroll_handle_for_uniform = filter_scroll_handle.clone();

                        div()
                            .absolute()
                            .left(px(filter_left))
                            .top(px(32.))
                            .w(px(150.))
                            .h(px(300.))
                            .bg(rgb(0x1f2937))
                            .border_1()
                            .border_color(rgb(0x3b82f6))
                            .rounded(px(4.))
                            .shadow_lg()
                            .flex()
                            .flex_col()
                            .overflow_hidden()  // Important: clip content
                            // Track mouse move to disable main list hover when over dropdown
                            .on_mouse_move({
                                let view_for_scroll = view_for_scroll.clone();
                                move |_event, _window, cx| {
                                    view_for_scroll.update(cx, |app, cx| {
                                        app.mouse_over_filter_dropdown = true;
                                        cx.notify();
                                    });
                                }
                            })
                            // Block all mouse events from reaching the main list
                            .on_mouse_up(gpui::MouseButton::Left, {
                                let view_for_scroll = view_for_scroll.clone();
                                move |_event, _window, cx| {
                                    view_for_scroll.update(cx, |app, cx| {
                                        app.mouse_over_filter_dropdown = true;
                                        cx.notify();
                                    });
                                }
                            })
                            .on_mouse_down(gpui::MouseButton::Left, {
                                let view_for_scroll = view_for_scroll.clone();
                                move |_event, _window, cx| {
                                    view_for_scroll.update(cx, |app, cx| {
                                        app.mouse_over_filter_dropdown = true;
                                        cx.notify();
                                    });
                                }
                            })
                            // Capture wheel events at container level and manually scroll
                            .on_scroll_wheel(move |event, _window, cx| {

                                // Calculate scroll delta
                                let delta_y = match event.delta {
                                    gpui::ScrollDelta::Lines(point) => point.y * 24.0,
                                    gpui::ScrollDelta::Pixels(pixels) => f32::from(pixels.y),
                                };

                                // Get current scroll offset
                                let current_offset = view_for_scroll.read(cx).filter_scroll_offset;
                                let current_offset_f32 = f32::from(current_offset);

                                // Calculate new scroll position
                                let row_height = 24.0;
                                let total_items = id_list_for_wheel.len();
                                let container_height = 300.0;
                                let total_height = total_items as f32 * row_height;
                                let max_scroll = (total_height - container_height).max(0.0);

                                let new_offset = (current_offset_f32 - delta_y).clamp(0.0, max_scroll);

                                // Update state
                                view_for_scroll.update(cx, |app, cx| {
                                    app.filter_scroll_offset = px(new_offset);
                                    cx.notify();
                                });

                                // Manually scroll the uniform_list using the persistent handle
                                let target_index = ((new_offset / row_height).round() as usize)
                                    .clamp(0, total_items.saturating_sub(1));

                                filter_scroll_handle.scroll_to_item_strict(
                                    target_index,
                                    gpui::ScrollStrategy::Top
                                );

                                eprintln!("Manual scroll: delta={:.2}, offset={:.2} -> {:.2}, index={}",
                                    delta_y, current_offset_f32, new_offset, target_index);
                            })
                            .child(
                                uniform_list(
                                    "filter-dropdown",
                                    id_list_clone.len(),
                                    move |range: std::ops::Range<usize>, _window: &mut gpui::Window, _cx: &mut gpui::App| {
                                        range
                                            .map(|index| {
                                                let id = id_list_clone[index];
                                                div()
                                                    .w_full()
                                                    .px_3()
                                                    .py_2()
                                                    .h(px(24.))
                                                    .text_sm()
                                                    .text_color(rgb(0xffffff))
                                                    .hover(|style| style.bg(rgb(0x374151)))
                                                    .cursor_pointer()
                                                    // Block all mouse events from propagating to the main list
                                                    .on_mouse_move(move |_event, _window, cx| {
                                                    })
                                                    .on_mouse_up(gpui::MouseButton::Left, move |_event, _window, cx| {
                                                    })
                                                    .on_mouse_down(gpui::MouseButton::Left, {
                                                        let view = view_clone1.clone();
                                                        move |_event, _window, cx| {
                                                            eprintln!("Selected ID: {}", id);
                                                            view.update(cx, |app, cx| {
                                                                app.id_filter = Some(id);
                                                                app.id_filter_text = id.to_string().into();
                                                                app.show_id_filter_input = false;
                                                                app.mouse_over_filter_dropdown = false;  // Reset hover flag
                                                                cx.notify();
                                                            });
                                                        }
                                                    })
                                                    .child(format!("ID: {}", id))
                                                    .into_any_element()
                                            })
                                            .collect::<Vec<_>>()
                                    },
                                )
                                .track_scroll(&filter_scroll_handle_for_uniform)
                                .flex_1()
                            )
                    }
                )
            })
            // Channel filter dropdown
            .when(self.show_channel_filter_input, |parent| {
                // Calculate ALL unique channels from messages
                let mut unique_channels = std::collections::HashSet::new();
                for msg in self.messages.iter() {
                    match msg {
                        LogObject::CanMessage(m) => { unique_channels.insert(m.channel); }
                        LogObject::CanMessage2(m) => { unique_channels.insert(m.channel); }
                        LogObject::CanFdMessage(m) => { unique_channels.insert(m.channel); }
                        LogObject::CanFdMessage64(m) => { unique_channels.insert(m.channel as u16); }
                        LogObject::LinMessage(m) => { unique_channels.insert(m.channel); }
                        LogObject::LinMessage2(_) => {}
                        _ => {}
                    }
                }
                let mut channel_list: Vec<u16> = unique_channels.into_iter().collect();
                channel_list.sort();

                let filter_left = 60.0 + f32::from(time_width) + 10.0; // Position after TIME column

                eprintln!("=== Channel filter dropdown rendering ===");
                eprintln!("  Found {} unique channels", channel_list.len());

                parent.child(
                    {
                        let channel_list_clone = channel_list.clone();
                        let view_for_scroll = view.clone();
                        let channel_list_for_wheel = channel_list.clone();
                        // Clone the scroll handle for use in closures
                        let filter_scroll_handle = self.channel_filter_scroll_handle.clone();
                        let filter_scroll_handle_for_uniform = filter_scroll_handle.clone();

                        div()
                            .absolute()
                            .left(px(filter_left))
                            .top(px(32.))
                            .w(px(120.))
                            .h(px(300.))
                            .bg(rgb(0x1f2937))
                            .border_1()
                            .border_color(rgb(0x3b82f6))
                            .rounded(px(4.))
                            .shadow_lg()
                            .flex()
                            .flex_col()
                            .overflow_hidden()
                            // Track mouse move to disable main list hover when over dropdown
                            .on_mouse_move({
                                let view_for_scroll = view_for_scroll.clone();
                                move |_event, _window, cx| {
                                    view_for_scroll.update(cx, |app, cx| {
                                        app.mouse_over_filter_dropdown = true;
                                        cx.notify();
                                    });
                                }
                            })
                            // Block all mouse events from reaching the main list
                            .on_mouse_up(gpui::MouseButton::Left, {
                                let view_for_scroll = view_for_scroll.clone();
                                move |_event, _window, cx| {
                                    view_for_scroll.update(cx, |app, cx| {
                                        app.mouse_over_filter_dropdown = true;
                                        cx.notify();
                                    });
                                }
                            })
                            .on_mouse_down(gpui::MouseButton::Left, {
                                let view_for_scroll = view_for_scroll.clone();
                                move |_event, _window, cx| {
                                    view_for_scroll.update(cx, |app, cx| {
                                        app.mouse_over_filter_dropdown = true;
                                        cx.notify();
                                    });
                                }
                            })
                            // Capture wheel events at container level and manually scroll
                            .on_scroll_wheel(move |event, _window, cx| {

                                // Calculate scroll delta
                                let delta_y = match event.delta {
                                    gpui::ScrollDelta::Lines(point) => point.y * 24.0,
                                    gpui::ScrollDelta::Pixels(pixels) => f32::from(pixels.y),
                                };

                                // Get current scroll offset
                                let current_offset = view_for_scroll.read(cx).channel_filter_scroll_offset;
                                let current_offset_f32 = f32::from(current_offset);

                                // Calculate new scroll position
                                let row_height = 24.0;
                                let total_items = channel_list_for_wheel.len();
                                let container_height = 300.0;
                                let total_height = total_items as f32 * row_height;
                                let max_scroll = (total_height - container_height).max(0.0);

                                let new_offset = (current_offset_f32 - delta_y).clamp(0.0, max_scroll);

                                // Update state
                                view_for_scroll.update(cx, |app, cx| {
                                    app.channel_filter_scroll_offset = px(new_offset);
                                    cx.notify();
                                });

                                // Manually scroll the uniform_list using the persistent handle
                                let target_index = ((new_offset / row_height).round() as usize)
                                    .clamp(0, total_items.saturating_sub(1));

                                filter_scroll_handle.scroll_to_item_strict(
                                    target_index,
                                    gpui::ScrollStrategy::Top
                                );

                                eprintln!("Channel filter scroll: delta={:.2}, offset={:.2} -> {:.2}, index={}",
                                    delta_y, current_offset_f32, new_offset, target_index);
                            })
                            .child(
                                uniform_list(
                                    "channel-filter-dropdown",
                                    channel_list_clone.len(),
                                    move |range: std::ops::Range<usize>, _window: &mut gpui::Window, _cx: &mut gpui::App| {
                                        range
                                            .map(|index| {
                                                let channel = channel_list_clone[index];
                                                div()
                                                    .w_full()
                                                    .px_3()
                                                    .py_2()
                                                    .h(px(24.))
                                                    .text_sm()
                                                    .text_color(rgb(0xffffff))
                                                    .hover(|style| style.bg(rgb(0x374151)))
                                                    .cursor_pointer()
                                                    // Block all mouse events from propagating to the main list
                                                    .on_mouse_move(move |_event, _window, cx| {
                                                    })
                                                    .on_mouse_up(gpui::MouseButton::Left, move |_event, _window, cx| {
                                                    })
                                                    .on_mouse_down(gpui::MouseButton::Left, {
                                                        let view = view_clone2.clone();
                                                        move |_event, _window, cx| {
                                                            eprintln!("Selected Channel: {}", channel);
                                                            view.update(cx, |app, cx| {
                                                                app.channel_filter = Some(channel);
                                                                app.channel_filter_text = channel.to_string().into();
                                                                app.show_channel_filter_input = false;
                                                                app.mouse_over_filter_dropdown = false;  // Reset hover flag
                                                                cx.notify();
                                                            });
                                                        }
                                                    })
                                                    .child(format!("CH: {}", channel))
                                                    .into_any_element()
                                            })
                                            .collect::<Vec<_>>()
                                    },
                                )
                                .track_scroll(&filter_scroll_handle_for_uniform)
                                .flex_1()
                            )
                    }
                )
            })
    }

    #[allow(dead_code)]
    // Render channel filter dropdown
    fn render_channel_filter_dropdown(
        &self,
        parent: gpui::Div,
        view: Entity<CanViewApp>,
        _ch_width: gpui::Pixels,
        time_width: gpui::Pixels,
    ) -> gpui::Div {
        parent.when(self.show_channel_filter_input, |parent| {
            // Calculate ALL unique channels from messages
            let mut unique_channels = std::collections::HashSet::new();
            for msg in self.messages.iter() {
                match msg {
                    LogObject::CanMessage(m) => {
                        unique_channels.insert(m.channel);
                    }
                    LogObject::CanMessage2(m) => {
                        unique_channels.insert(m.channel);
                    }
                    LogObject::CanFdMessage(m) => {
                        unique_channels.insert(m.channel);
                    }
                    LogObject::CanFdMessage64(m) => {
                        unique_channels.insert(m.channel as u16);
                    }
                    LogObject::LinMessage(m) => {
                        unique_channels.insert(m.channel);
                    }
                    LogObject::LinMessage2(_) => {}
                    _ => {}
                }
            }
            let mut channel_list: Vec<u16> = unique_channels.into_iter().collect();
            channel_list.sort();

            let filter_left = 60.0 + f32::from(time_width) + 10.0; // Position after TIME column

            eprintln!("=== Channel filter dropdown rendering ===");
            eprintln!("  Found {} unique channels", channel_list.len());

            parent.child({
                let channel_list_clone = channel_list.clone();
                let view_for_scroll = view.clone();
                let channel_list_for_wheel = channel_list.clone();
                // Clone the scroll handle for use in closures
                let filter_scroll_handle = self.channel_filter_scroll_handle.clone();
                let filter_scroll_handle_for_uniform = filter_scroll_handle.clone();

                div()
                    .absolute()
                    .left(px(filter_left))
                    .top(px(32.))
                    .w(px(120.))
                    .h(px(300.))
                    .bg(rgb(0x1f2937))
                    .border_1()
                    .border_color(rgb(0x3b82f6))
                    .rounded(px(4.))
                    .shadow_lg()
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    // Track mouse move to disable main list hover when over dropdown
                    .on_mouse_move({
                        let view_for_scroll = view_for_scroll.clone();
                        move |_event, _window, cx| {
                            view_for_scroll.update(cx, |app, cx| {
                                app.mouse_over_filter_dropdown = true;
                                cx.notify();
                            });
                        }
                    })
                    // Block all mouse events from reaching the main list
                    .on_mouse_up(gpui::MouseButton::Left, {
                        let view_for_scroll = view_for_scroll.clone();
                        move |_event, _window, cx| {
                            view_for_scroll.update(cx, |app, cx| {
                                app.mouse_over_filter_dropdown = true;
                                cx.notify();
                            });
                        }
                    })
                    .on_mouse_down(gpui::MouseButton::Left, {
                        let view_for_scroll = view_for_scroll.clone();
                        move |_event, _window, cx| {
                            view_for_scroll.update(cx, |app, cx| {
                                app.mouse_over_filter_dropdown = true;
                                cx.notify();
                            });
                        }
                    })
                    // Capture wheel events at container level and manually scroll
                    .on_scroll_wheel(move |event, _window, cx| {

                        // Calculate scroll delta
                        let delta_y = match event.delta {
                            gpui::ScrollDelta::Lines(point) => point.y * 24.0,
                            gpui::ScrollDelta::Pixels(pixels) => f32::from(pixels.y),
                        };

                        // Get current scroll offset
                        let current_offset = view_for_scroll.read(cx).channel_filter_scroll_offset;
                        let current_offset_f32 = f32::from(current_offset);

                        // Calculate new scroll position
                        let row_height = 24.0;
                        let total_items = channel_list_for_wheel.len();
                        let container_height = 300.0;
                        let total_height = total_items as f32 * row_height;
                        let max_scroll = (total_height - container_height).max(0.0);

                        let new_offset = (current_offset_f32 - delta_y).clamp(0.0, max_scroll);

                        // Update state
                        view_for_scroll.update(cx, |app, cx| {
                            app.channel_filter_scroll_offset = px(new_offset);
                            cx.notify();
                        });

                        // Manually scroll the uniform_list using the persistent handle
                        let target_index = ((new_offset / row_height).round() as usize)
                            .clamp(0, total_items.saturating_sub(1));

                        filter_scroll_handle
                            .scroll_to_item_strict(target_index, gpui::ScrollStrategy::Top);

                        eprintln!(
                            "Channel filter scroll: delta={:.2}, offset={:.2} -> {:.2}, index={}",
                            delta_y, current_offset_f32, new_offset, target_index
                        );
                    })
                    .child(
                        uniform_list(
                            "channel-filter-dropdown",
                            channel_list_clone.len(),
                            move |range: std::ops::Range<usize>,
                                  _window: &mut gpui::Window,
                                  _cx: &mut gpui::App| {
                                range
                                    .map(|index| {
                                        let channel = channel_list_clone[index];
                                        div()
                                            .w_full()
                                            .px_3()
                                            .py_2()
                                            .h(px(24.))
                                            .text_sm()
                                            .text_color(rgb(0xffffff))
                                            .hover(|style| style.bg(rgb(0x374151)))
                                            .cursor_pointer()
                                            // Block all mouse events from propagating to the main list
                                            .on_mouse_move(move |_event, _window, cx| {
                                            })
                                            .on_mouse_up(
                                                gpui::MouseButton::Left,
                                                move |_event, _window, cx| {
                                                },
                                            )
                                            .on_mouse_down(gpui::MouseButton::Left, {
                                                let view = view.clone();
                                                move |_event, _window, cx| {
                                                    eprintln!("Selected Channel: {}", channel);
                                                    view.update(cx, |app, cx| {
                                                        app.channel_filter = Some(channel);
                                                        app.channel_filter_text =
                                                            channel.to_string().into();
                                                        app.show_channel_filter_input = false;
                                                        app.mouse_over_filter_dropdown = false; // Reset hover flag
                                                        cx.notify();
                                                    });
                                                }
                                            })
                                            .child(format!("CH: {}", channel))
                                            .into_any_element()
                                    })
                                    .collect::<Vec<_>>()
                            },
                        )
                        .track_scroll(&filter_scroll_handle_for_uniform)
                        .flex_1(),
                    )
            })
        })
    }

    fn get_message_strings(
        msg: &LogObject,
        start_time: Option<chrono::NaiveDateTime>,
        decimal: bool,
    ) -> (String, u16, String, String, String, String) {
        let format_id = |id: u32| -> String {
            if decimal {
                id.to_string()
            } else {
                format!("0x{:03X}", id)
            }
        };

        match msg {
            LogObject::CanMessage(can_msg) => {
                let timestamp = can_msg.header.object_time_stamp;
                let time_str = if let Some(start) = start_time {
                    let msg_time = start + chrono::Duration::nanoseconds(timestamp as i64);
                    // Format: YYYY-MM-DD HH:MM:SS.mmmmmm (microseconds)
                    msg_time.format("%Y-%m-%d %H:%M:%S%.6f").to_string()
                } else {
                    // If no start time, show nanoseconds as seconds with microsecond precision
                    let seconds = timestamp as f64 / 1_000_000_000.0;
                    format!("{:.6}", seconds)
                };

                let actual_data_len = can_msg.data.len().min(can_msg.dlc as usize);
                let data_hex = can_msg
                    .data
                    .iter()
                    .take(actual_data_len)
                    .map(|b| format!("{:02X}", b))
                    .collect::<Vec<_>>()
                    .join(" ");

                (
                    time_str,
                    can_msg.channel,
                    "CAN".to_string(),
                    format_id(can_msg.id),
                    actual_data_len.to_string(),
                    data_hex,
                )
            }
            LogObject::CanMessage2(can_msg) => {
                let timestamp = can_msg.header.object_time_stamp;
                let time_str = if let Some(start) = start_time {
                    let msg_time = start + chrono::Duration::nanoseconds(timestamp as i64);
                    msg_time.format("%Y-%m-%d %H:%M:%S%.6f").to_string()
                } else {
                    let seconds = timestamp as f64 / 1_000_000_000.0;
                    format!("{:.6}", seconds)
                };

                let actual_data_len = can_msg.data.len().min(can_msg.dlc as usize);
                let data_hex = can_msg
                    .data
                    .iter()
                    .take(actual_data_len)
                    .map(|b| format!("{:02X}", b))
                    .collect::<Vec<_>>()
                    .join(" ");

                (
                    time_str,
                    can_msg.channel,
                    "CAN2".to_string(),
                    format_id(can_msg.id),
                    actual_data_len.to_string(),
                    data_hex,
                )
            }
            LogObject::CanErrorFrame(err) => {
                let timestamp = err.header.object_time_stamp;
                let time_str = if let Some(start) = start_time {
                    let msg_time = start + chrono::Duration::nanoseconds(timestamp as i64);
                    msg_time.format("%Y-%m-%d %H:%M:%S%.6f").to_string()
                } else {
                    let seconds = timestamp as f64 / 1_000_000_000.0;
                    format!("{:.6}", seconds)
                };

                (
                    time_str,
                    err.channel,
                    "CAN_ERR".to_string(),
                    "-".to_string(),
                    err.length.to_string(),
                    "-".to_string(),
                )
            }
            LogObject::CanFdMessage(fd_msg) => {
                let timestamp = fd_msg.header.object_time_stamp;
                let time_str = if let Some(start) = start_time {
                    let msg_time = start + chrono::Duration::nanoseconds(timestamp as i64);
                    msg_time.format("%Y-%m-%d %H:%M:%S%.6f").to_string()
                } else {
                    let seconds = timestamp as f64 / 1_000_000_000.0;
                    format!("{:.6}", seconds)
                };

                let actual_data_len = fd_msg.data.len().min(fd_msg.dlc as usize);
                let data_hex = fd_msg
                    .data
                    .iter()
                    .take(actual_data_len)
                    .map(|b| format!("{:02X}", b))
                    .collect::<Vec<_>>()
                    .join(" ");

                (
                    time_str,
                    fd_msg.channel,
                    "CAN_FD".to_string(),
                    format_id(fd_msg.id),
                    actual_data_len.to_string(),
                    data_hex,
                )
            }
            LogObject::CanFdMessage64(fd_msg) => {
                let timestamp = fd_msg.header.object_time_stamp;
                let time_str = if let Some(start) = start_time {
                    let msg_time = start + chrono::Duration::nanoseconds(timestamp as i64);
                    msg_time.format("%Y-%m-%d %H:%M:%S%.6f").to_string()
                } else {
                    let seconds = timestamp as f64 / 1_000_000_000.0;
                    format!("{:.6}", seconds)
                };

                let actual_data_len = fd_msg.data.len().min(fd_msg.valid_data_bytes as usize);
                let data_hex = fd_msg
                    .data
                    .iter()
                    .take(actual_data_len)
                    .map(|b| format!("{:02X}", b))
                    .collect::<Vec<_>>()
                    .join(" ");

                (
                    time_str,
                    fd_msg.channel as u16,
                    "CAN_FD64".to_string(),
                    format_id(fd_msg.id),
                    actual_data_len.to_string(),
                    data_hex,
                )
            }
            LogObject::CanOverloadFrame(ov) => {
                let timestamp = ov.header.object_time_stamp;
                let time_str = if let Some(start) = start_time {
                    let msg_time = start + chrono::Duration::nanoseconds(timestamp as i64);
                    msg_time.format("%Y-%m-%d %H:%M:%S%.6f").to_string()
                } else {
                    let seconds = timestamp as f64 / 1_000_000_000.0;
                    format!("{:.6}", seconds)
                };

                (
                    time_str,
                    ov.channel,
                    "CAN_OV".to_string(),
                    "-".to_string(),
                    "-".to_string(),
                    "-".to_string(),
                )
            }
            LogObject::LinMessage(lin_msg) => {
                let timestamp = lin_msg.header.object_time_stamp;
                let time_str = if let Some(start) = start_time {
                    let msg_time = start + chrono::Duration::nanoseconds(timestamp as i64);
                    // Format: YYYY-MM-DD HH:MM:SS.mmmmmm (microseconds)
                    msg_time.format("%Y-%m-%d %H:%M:%S%.6f").to_string()
                } else {
                    format!("{:.6}", timestamp as f64 / 1_000_000_000.0)
                };

                let actual_data_len = lin_msg.data.len().min(lin_msg.dlc as usize);
                let data_hex = lin_msg
                    .data
                    .iter()
                    .take(actual_data_len)
                    .map(|b| format!("{:02X}", b))
                    .collect::<Vec<_>>()
                    .join(" ");

                (
                    time_str,
                    lin_msg.channel,
                    "LIN".to_string(),
                    format_id(lin_msg.id as u32),
                    actual_data_len.to_string(),
                    data_hex,
                )
            }
            LogObject::LinMessage2(lin_msg) => {
                let timestamp = lin_msg.header.object_time_stamp;
                let time_str = if let Some(start) = start_time {
                    let msg_time = start + chrono::Duration::nanoseconds(timestamp as i64);
                    msg_time.format("%Y-%m-%d %H:%M:%S%.6f").to_string()
                } else {
                    let seconds = timestamp as f64 / 1_000_000_000.0;
                    format!("{:.6}", seconds)
                };

                let actual_data_len = lin_msg.data.len();
                let data_hex = lin_msg
                    .data
                    .iter()
                    .map(|b| format!("{:02X}", b))
                    .collect::<Vec<_>>()
                    .join(" ");

                (
                    time_str,
                    0_u16,
                    "LIN2".to_string(),
                    "-".to_string(),
                    actual_data_len.to_string(),
                    data_hex,
                )
            }
            _ => {
                let type_name = format!("{:?}", msg);
                (
                    "-".to_string(),
                    0_u16,
                    type_name.split('(').next().unwrap_or("UNKNOWN").to_string(),
                    "-".to_string(),
                    "-".to_string(),
                    "-".to_string(),
                )
            }
        }
    }

    // Render message row with pre-calculated widths for perfect alignment
    fn render_message_row_static_with_widths(
        msg: &LogObject,
        _index: usize,
        time_width: gpui::Pixels,
        ch_width: gpui::Pixels,
        type_width: gpui::Pixels,
        id_width: gpui::Pixels,
        dlc_width: gpui::Pixels,
        _dbc_channels: &HashMap<u16, DbcDatabase>,
        _ldf_channels: &HashMap<u16, LdfDatabase>,
        start_time: Option<chrono::NaiveDateTime>,
        decimal: bool,
        disable_hover: bool, // New parameter to disable hover effect
    ) -> gpui::AnyElement {
        let (time_str, channel_id, msg_type, id_str, dlc_str, data_str) =
            Self::get_message_strings(msg, start_time, decimal);

        let bg_color = rgb(0x181818); // Simplified background
        let type_color = match msg_type.as_str() {
            "CAN" | "CAN2" => rgb(0x34d399),
            "CAN_ERR" => rgb(0xef4444),
            "CAN_FD" | "CAN_FD64" => rgb(0x8b5cf6),
            "CAN_OV" => rgb(0xf59e0b),
            "LIN" | "LIN2" => rgb(0x60a5fa),
            _ => rgb(0x9ca3af),
        };

        div()
            .flex()
            .w_full()
            .min_h(px(22.))
            .bg(bg_color)
            .border_b_1()
            .border_color(rgb(0x2a2a2a))
            .items_center()
            .text_xs()
            .text_color(rgb(0xd1d5db))
            .when(!disable_hover, |div| {
                div.hover(|style| style.bg(rgb(0x1f2937)))
            })
            .cursor_pointer()
            .overflow_hidden() // Ensure row doesn't overflow
            .child(
                // Line number column
                div()
                    .w(px(60.))
                    .px_3()
                    .py_1()
                    .flex()
                    .items_center()
                    .flex_shrink_0()
                    .text_color(rgb(0x6b7280))
                    .whitespace_nowrap()
                    .overflow_hidden()
                    .child(format!("{}", _index + 1)),
            )
            .child(
                div()
                    .w(time_width)
                    .px_3()
                    .py_1()
                    .flex()
                    .items_center()
                    .flex_shrink_0()
                    .text_color(rgb(0x9ca3af))
                    .whitespace_nowrap()
                    .overflow_hidden()
                    .child(time_str),
            )
            .child(
                div()
                    .w(ch_width)
                    .px_2()
                    .py_1()
                    .flex()
                    .items_center()
                    .flex_shrink_0()
                    .text_color(rgb(0x60a5fa))
                    .whitespace_nowrap()
                    .overflow_hidden()
                    .child(channel_id.to_string()),
            )
            .child(
                div()
                    .w(type_width)
                    .px_2()
                    .py_1()
                    .flex()
                    .items_center()
                    .flex_shrink_0()
                    .text_color(type_color)
                    .whitespace_nowrap()
                    .overflow_hidden()
                    .child(msg_type),
            )
            .child(
                div()
                    .w(id_width)
                    .px_2()
                    .py_1()
                    .flex()
                    .items_center()
                    .flex_shrink_0()
                    .text_color(rgb(0xfbbf24))
                    .whitespace_nowrap()
                    .overflow_hidden()
                    .child(id_str),
            )
            .child(
                div()
                    .w(dlc_width)
                    .px_2()
                    .py_1()
                    .flex()
                    .items_center()
                    .flex_shrink_0()
                    .whitespace_nowrap()
                    .overflow_hidden()
                    .child(dlc_str),
            )
            .child(
                div()
                    .flex_1() // DATA列使用flex_1()占据剩余空间
                    .px_2()
                    .py_1()
                    .flex()
                    .items_center()
                    .text_color(rgb(0xa78bfa))
                    .whitespace_nowrap()
                    .child(data_str),
            )
            .into_any_element()
    }

    #[allow(dead_code)]
    // Static helper to format timestamp with microseconds
    fn format_timestamp_static(
        timestamp: u64,
        start_time: Option<chrono::NaiveDateTime>,
    ) -> String {
        if let Some(start) = start_time {
            let msg_time = start + chrono::Duration::nanoseconds(timestamp as i64);
            // Format: YYYY-MM-DD HH:MM:SS.mmmmmm (microseconds)
            msg_time.format("%Y-%m-%d %H:%M:%S%.6f").to_string()
        } else {
            // If no start time, show nanoseconds as seconds with microsecond precision
            format!("{:.6}", timestamp as f64 / 1_000_000_000.0)
        }
    }

    #[allow(dead_code)]
    // Static helper to render a message row (needed for uniform_list closure)
    fn render_message_row_static(
        msg: &LogObject,
        index: usize,
        _dbc_channels: &HashMap<u16, DbcDatabase>,
        _ldf_channels: &HashMap<u16, LdfDatabase>,
        start_time: Option<chrono::NaiveDateTime>,
    ) -> gpui::AnyElement {
        let (time_str, channel_id, msg_type, id_str, dlc_str, data_str): (
            String,
            u16,
            String,
            String,
            String,
            String,
        ) = match msg {
            // CAN Message Types
            LogObject::CanMessage(can_msg) => {
                let timestamp = can_msg.header.object_time_stamp;
                let time_str = Self::format_timestamp_static(timestamp, start_time);

                let actual_data_len = can_msg.data.len().min(can_msg.dlc as usize);
                let data_hex = can_msg
                    .data
                    .iter()
                    .take(actual_data_len)
                    .map(|b| format!("{:02X}", b))
                    .collect::<Vec<_>>()
                    .join(" ");

                (
                    time_str,
                    can_msg.channel,
                    "CAN".to_string(),
                    format!("0x{:03X}", can_msg.id),
                    actual_data_len.to_string(),
                    data_hex,
                )
            }
            LogObject::CanMessage2(can_msg) => {
                let timestamp = can_msg.header.object_time_stamp;
                let time_str = Self::format_timestamp_static(timestamp, start_time);

                let actual_data_len = can_msg.data.len().min(can_msg.dlc as usize);
                let data_hex = can_msg
                    .data
                    .iter()
                    .take(actual_data_len)
                    .map(|b| format!("{:02X}", b))
                    .collect::<Vec<_>>()
                    .join(" ");

                (
                    time_str,
                    can_msg.channel,
                    "CAN2".to_string(),
                    format!("0x{:03X}", can_msg.id),
                    actual_data_len.to_string(),
                    data_hex,
                )
            }
            LogObject::CanErrorFrame(err) => {
                let timestamp = err.header.object_time_stamp;
                let time_str = Self::format_timestamp_static(timestamp, start_time);

                (
                    time_str,
                    err.channel,
                    "CAN_ERR".to_string(),
                    "-".to_string(),
                    err.length.to_string(),
                    "-".to_string(),
                )
            }
            LogObject::CanFdMessage(fd_msg) => {
                let timestamp = fd_msg.header.object_time_stamp;
                let time_str = Self::format_timestamp_static(timestamp, start_time);

                let actual_data_len = fd_msg.data.len().min(fd_msg.dlc as usize);
                let data_hex = fd_msg
                    .data
                    .iter()
                    .take(actual_data_len)
                    .map(|b| format!("{:02X}", b))
                    .collect::<Vec<_>>()
                    .join(" ");

                (
                    time_str,
                    fd_msg.channel, // Convert u8 to u16
                    "CAN_FD".to_string(),
                    format!("0x{:03X}", fd_msg.id),
                    actual_data_len.to_string(),
                    data_hex,
                )
            }
            LogObject::CanFdMessage64(fd_msg) => {
                let timestamp = fd_msg.header.object_time_stamp;
                let time_str = Self::format_timestamp_static(timestamp, start_time);

                let actual_data_len = fd_msg.data.len().min(fd_msg.valid_data_bytes as usize);
                let data_hex = fd_msg
                    .data
                    .iter()
                    .take(actual_data_len)
                    .map(|b| format!("{:02X}", b))
                    .collect::<Vec<_>>()
                    .join(" ");

                (
                    time_str,
                    fd_msg.channel as u16, // Convert u8 to u16
                    "CAN_FD64".to_string(),
                    format!("0x{:03X}", fd_msg.id),
                    actual_data_len.to_string(),
                    data_hex,
                )
            }
            LogObject::CanOverloadFrame(ov) => {
                let timestamp = ov.header.object_time_stamp;
                let time_str = Self::format_timestamp_static(timestamp, start_time);

                (
                    time_str,
                    ov.channel,
                    "CAN_OV".to_string(),
                    "-".to_string(),
                    "-".to_string(),
                    "-".to_string(),
                )
            }

            // LIN Message Types
            LogObject::LinMessage(lin_msg) => {
                let timestamp = lin_msg.header.object_time_stamp;
                let time_str = if let Some(start) = start_time {
                    let msg_time = start + chrono::Duration::nanoseconds(timestamp as i64);
                    // Format: YYYY-MM-DD HH:MM:SS.mmmmmm (microseconds)
                    msg_time.format("%Y-%m-%d %H:%M:%S%.6f").to_string()
                } else {
                    format!("{:.6}", timestamp as f64 / 1_000_000_000.0)
                };

                let actual_data_len = lin_msg.data.len().min(lin_msg.dlc as usize);
                let data_hex = lin_msg
                    .data
                    .iter()
                    .take(actual_data_len)
                    .map(|b| format!("{:02X}", b))
                    .collect::<Vec<_>>()
                    .join(" ");

                (
                    time_str,
                    lin_msg.channel,
                    "LIN".to_string(),
                    format!("0x{:02X}", lin_msg.id),
                    actual_data_len.to_string(),
                    data_hex,
                )
            }
            LogObject::LinMessage2(lin_msg) => {
                let timestamp = lin_msg.header.object_time_stamp;
                let time_str = Self::format_timestamp_static(timestamp, start_time);

                let actual_data_len = lin_msg.data.len();
                let data_hex = lin_msg
                    .data
                    .iter()
                    .map(|b| format!("{:02X}", b))
                    .collect::<Vec<_>>()
                    .join(" ");

                (
                    time_str,
                    0_u16,
                    "LIN2".to_string(),
                    "-".to_string(),
                    actual_data_len.to_string(),
                    data_hex,
                )
            }

            // Default case for other types (LIN errors, FlexRay, etc.)
            _ => {
                let type_name = format!("{:?}", msg);
                (
                    "-".to_string(),
                    0_u16,
                    type_name.split('(').next().unwrap_or("UNKNOWN").to_string(),
                    "-".to_string(),
                    "-".to_string(),
                    "-".to_string(),
                )
            }
        };

        let bg_color = if index.is_multiple_of(2) {
            rgb(0x181818)
        } else {
            rgb(0x1a1a1a)
        };

        // Color code message types
        let type_color = match msg_type.as_str() {
            "CAN" | "CAN2" => rgb(0x34d399),        // Green for normal CAN
            "CAN_ERR" => rgb(0xef4444),             // Red for errors
            "CAN_FD" | "CAN_FD64" => rgb(0x8b5cf6), // Purple for CAN FD
            "CAN_OV" => rgb(0xf59e0b),              // Orange for overload
            "LIN" | "LIN2" => rgb(0x60a5fa),        // Blue for LIN
            "LIN_CRC" | "LIN_RX_ERR" | "LIN_TX_ERR" => rgb(0xef4444), // Red for LIN errors
            "LIN_WAKE" => rgb(0xfbbf24),            // Yellow for wakeup
            "LIN_SLEEP" => rgb(0x6b7280),           // Gray for sleep
            "FLEX" | "FLEX_SYNC" => rgb(0xec4899),  // Pink for FlexRay
            _ => rgb(0x9ca3af),                     // Default gray
        };

        div()
            .flex()
            .w_full()
            .min_h(px(22.))
            .bg(bg_color)
            .border_b_1()
            .border_color(rgb(0x2a2a2a))
            .items_center()
            .text_xs()
            .text_color(rgb(0xd1d5db))
            .hover(|style| style.bg(rgb(0x1f2937)))
            .cursor_pointer()
            .child(
                div()
                    .px_3()
                    .py_1()
                    .text_color(rgb(0x9ca3af))
                    .whitespace_nowrap()
                    .child(time_str),
            )
            .child(
                div()
                    .px_2()
                    .py_1()
                    .text_color(rgb(0x60a5fa))
                    .whitespace_nowrap()
                    .child(channel_id.to_string()),
            )
            .child(
                div()
                    .px_2()
                    .py_1()
                    .text_color(type_color)
                    .whitespace_nowrap()
                    .child(msg_type),
            )
            .child(
                div()
                    .px_2()
                    .py_1()
                    .text_color(rgb(0xfbbf24))
                    .whitespace_nowrap()
                    .child(id_str),
            )
            .child(div().px_2().py_1().whitespace_nowrap().child(dlc_str))
            .child(
                div()
                    .px_2()
                    .py_1()
                    .text_color(rgb(0xa78bfa))
                    .whitespace_nowrap()
                    .child(data_str),
            )
            .into_any_element()
    }

    fn render_config_view(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .p_6()
            .flex()
            .flex_col()
            .gap_4()
            .text_color(rgb(0xd1d5db))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_lg()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(rgb(0xffffff))
                            .child("Configuration"),
                    )
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .child(
                                div()
                                    .px_3()
                                    .py_1()
                                    .bg(rgb(0x3b82f6))
                                    .rounded(px(4.))
                                    .cursor_pointer()
                                    .hover(|style| style.bg(rgb(0x2563eb)))
                                    .text_color(rgb(0xffffff))
                                    .text_sm()
                                    .child("Import Database")
                                    .on_mouse_down(gpui::MouseButton::Left, {
                                        let view = cx.entity().clone();
                                        move |_event, _window, cx| {
                                            view.update(cx, |this, cx| {
                                                this.import_database_file(cx);
                                            });
                                        }
                                    }),
                            )
                            .child(
                                div()
                                    .px_3()
                                    .py_1()
                                    .bg(rgb(0x10b981))
                                    .rounded(px(4.))
                                    .cursor_pointer()
                                    .hover(|style| style.bg(rgb(0x059669)))
                                    .text_color(rgb(0xffffff))
                                    .text_sm()
                                    .child("Save Config")
                                    .on_mouse_down(gpui::MouseButton::Left, {
                                        let view = cx.entity().clone();
                                        move |_event, _window, cx| {
                                            view.update(cx, |this, cx| {
                                                this.save_config(cx);
                                            });
                                        }
                                    }),
                            ),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .bg(rgb(0x1f1f1f))
                    .border_1()
                    .border_color(rgb(0x2a2a2a))
                    .rounded(px(8.))
                    .flex()
                    .flex_col()
                    .gap_2()
                    .p_4()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(rgb(0xffffff))
                            .child("Channel Mappings"),
                    )
                    .child(div().flex_1().flex().flex_col().gap_2().children(
                        self.app_config.mappings.iter().map(|mapping| {
                            div()
                                .p_3()
                                .bg(rgb(0x374151))
                                .rounded(px(4.))
                                .flex()
                                .items_center()
                                .justify_between()
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .child(
                                            div()
                                                .text_sm()
                                                .font_weight(FontWeight::MEDIUM)
                                                .text_color(rgb(0xffffff))
                                                .child(format!(
                                                    "Channel {} ({})",
                                                    mapping.channel_id,
                                                    if mapping.channel_type == ChannelType::CAN {
                                                        "CAN"
                                                    } else {
                                                        "LIN"
                                                    }
                                                )),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(rgb(0x9ca3af))
                                                .child(mapping.path.clone()),
                                        ),
                                )
                        }),
                    )),
            )
            .child(
                // Status bar
                div()
                    .p_4()
                    .bg(rgb(0x1f1f1f))
                    .border_1()
                    .border_color(rgb(0x2a2a2a))
                    .rounded(px(8.))
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(rgb(0xffffff))
                            .child("System Status"),
                    )
                    .child(
                        div()
                            .flex()
                            .gap_4()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0x9ca3af))
                                    .child(format!("Messages: {}", self.messages.len())),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0x9ca3af))
                                    .child(format!("DBC: {}", self.dbc_channels.len())),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0x9ca3af))
                                    .child(format!("LIN: {}", self.ldf_channels.len())),
                            ),
                    ),
            )
    }
}
impl Render for CanViewApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Update container height based on current window size
        self.update_container_height(window);

        // Initialize channel input states if needed (when show_add_channel_input is true)
        if self.show_add_channel_input {
            if self.channel_id_input.is_none() {
                eprintln!("📝 Creating channel_id_input in render...");
                let input = cx.new(|cx| {
                    InputState::new(window, cx)
                        .placeholder("Channel ID")
                });
                cx.subscribe(&input, |this, input, event, cx| {
                    if let InputEvent::Change = event {
                        this.new_channel_id = input.read(cx).text().to_string();
                        eprintln!("DEBUG: ID change: {}", this.new_channel_id);
                        // cx.notify(); // Optional, but let's keep it minimal to avoid flicker
                    }
                })
                .detach();
                self.channel_id_input = Some(input);
            }

            if self.channel_name_input.is_none() {
                eprintln!("📝 Creating channel_name_input in render...");
                let input = cx.new(|cx| {
                    InputState::new(window, cx).placeholder("Channel name")
                });
                cx.subscribe(&input, |this, input, event, cx| {
                    if let InputEvent::Change = event {
                        this.new_channel_name = input.read(cx).text().to_string();
                        eprintln!("DEBUG: Name change: {}", this.new_channel_name);
                    }
                })
                .detach();
                self.channel_name_input = Some(input);
            }
        }

        // Check for file dialog result (non-blocking poll)
        if let Some(mut receiver) = self.pending_file_path.take() {
            match receiver.try_recv() {
                Ok(Some(path_str)) => {
                    // File selected successfully
                    self.new_channel_db_path = path_str.clone();
                    self.status_msg = format!("✅ Selected: {}", path_str).into();
                    cx.notify();
                }
                Ok(None) => {
                    // User cancelled
                    self.status_msg = "❌ File selection cancelled".into();
                    cx.notify();
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    // Not ready yet, put it back
                    self.pending_file_path = Some(receiver);
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    // Thread ended without result
                    self.status_msg = "".into();
                }
            }
        }

        let view = cx.entity().clone();

        div()
            .size_full()
            .flex()
            .flex_col()
            .on_key_down({
                let view = view.clone();
                move |event, _window, cx| {
                    eprintln!("=== ROOT LEVEL on_key_down ===");
                    eprintln!("keystroke: {}", event.keystroke);
                    eprintln!(
                        "show_id_filter_input: {}",
                        view.read(cx).show_id_filter_input
                    );

                    let keystroke_str = format!("{}", event.keystroke);

                    // Handle library dialog input
                    if keystroke_str.as_str() == "enter" {
                        let show_library_dialog = view.read(cx).show_library_dialog;
                        if show_library_dialog {
                            eprintln!("📥 Enter pressed in library dialog");

                            // Read input value BEFORE entering update block to avoid nested update conflict
                            let library_name = view
                                .read(cx)
                                .library_name_input
                                .as_ref()
                                .map(|i| i.read(cx).value().to_string())
                                .unwrap_or_default();

                            view.update(cx, |app, cx| {
                                eprintln!(
                                    "⏎ Creating library from ROOT handler: '{}'",
                                    library_name
                                );

                                if !library_name.trim().is_empty() {
                                    app.new_library_name = library_name.clone();
                                    app.create_library(cx);
                                }

                                // Close the dialog
                                app.show_library_dialog = false;
                                app.library_name_input = None;
                                cx.notify();
                            });
                            return;
                        }

                        // Handle version input
                        let show_version_input = view.read(cx).show_version_input;
                        if show_version_input {
                            eprintln!("📥 Enter pressed in version input");

                            // Read input value BEFORE entering update block to avoid nested update conflict
                            let version_name = view
                                .read(cx)
                                .version_name_input
                                .as_ref()
                                .map(|input| input.read(cx).value().to_string())
                                .unwrap_or_default();

                            view.update(cx, |app, cx| {
                                // Store the version name before calling add_library_version
                                app.new_version_name = version_name.clone();

                                eprintln!("⏎ Adding version from ROOT handler: '{}'", version_name);
                                app.add_library_version(cx);

                                // Close the input
                                app.show_version_input = false;
                                app.version_name_input = None;
                                cx.notify();
                            });
                            return;
                        }
                    }

                    // Only handle when filter is active
                    let show_filter = view.read(cx).show_id_filter_input;
                    if show_filter {
                        let keystroke_str = format!("{}", event.keystroke);
                        match keystroke_str.as_str() {
                            "backspace" => {
                                view.update(cx, |app, cx| {
                                    let mut text = app.id_filter_text.to_string();
                                    if !text.is_empty() {
                                        text.pop();
                                        app.id_filter_text = text.into();
                                        eprintln!(
                                            "Filter text (backspace): {}",
                                            app.id_filter_text
                                        );
                                        cx.notify();
                                    }
                                });
                            }
                            "escape" => {
                                view.update(cx, |app, cx| {
                                    app.show_id_filter_input = false;
                                    eprintln!("Filter closed (escape)");
                                    cx.notify();
                                });
                            }
                            "enter" => {
                                view.update(cx, |app, cx| {
                                    if let Ok(parsed_id) =
                                        u32::from_str_radix(app.id_filter_text.as_ref(), 10)
                                    {
                                        if !app.id_filter_text.is_empty() {
                                            app.id_filter = Some(parsed_id);
                                        }
                                    }
                                    app.show_id_filter_input = false;
                                    eprintln!("Filter applied (enter): id={:?}", app.id_filter);
                                    cx.notify();
                                });
                            }
                            _ => {
                                if keystroke_str.len() == 1 {
                                    if let Some(ch) = keystroke_str.chars().next() {
                                        if ch.is_ascii_digit() {
                                            view.update(cx, |app, cx| {
                                                let mut text = app.id_filter_text.to_string();
                                                text.push(ch);
                                                app.id_filter_text = text.into();
                                                eprintln!("Filter text: {}", app.id_filter_text);
                                                cx.notify();
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            })
            .child(
                // Unified top bar with all options - Zed style
                div()
                    .h(px(48.)) // Slightly shorter, more like Zed
                    .bg(rgb(0x0c0c0e)) // Zed's panel background
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_4()
                    .border_b_1()
                    .border_color(rgb(0x1a1a1a)) // Very subtle border
                    .relative()
                    .child(div().absolute().inset_0().window_control_area(WindowControlArea::Drag))
                    .child(
                        // Left: App branding and navigation tabs (draggable area)
                        div()
                            .when(cfg!(target_os = "macos"), |div| div.pl(px(80.)))
                            .flex()
                            .items_center()
                            .h_full()
                            .gap_4()
                            
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_0() // Tighter spacing like Zed
                                    .child(
                                        div()
                                            .px_3()
                                            
                                            .py(px(1.5))
                                            .text_xs()
                                            .font_weight(FontWeight::MEDIUM)
                                            .cursor_pointer()
                                            .rounded(px(3.)) // Smaller radius like Zed
                                            .bg(if self.current_view == AppView::LogView {
                                                rgb(0x1e1e2e) // Zed-style active tab
                                            } else {
                                                rgb(0x0c0c0e) // Transparent
                                            })
                                            .text_color(if self.current_view == AppView::LogView {
                                                rgb(0xcdd6f4) // Zed's text
                                            } else {
                                                rgb(0x646473) // Zed's muted
                                            })
                                            .hover(|style| {
                                                if self.current_view != AppView::LogView {
                                                    style
                                                        .bg(rgb(0x151515)) // Very subtle hover
                                                        .text_color(rgb(0x9399b2))
                                                } else {
                                                    style
                                                }
                                            })
                                            .on_mouse_down(gpui::MouseButton::Left, {
                                                let view = view.clone();
                                                move |_event, _, cx| {
                                                    view.update(cx, |view, _| {
                                                        view.current_view = AppView::LogView
                                                    });
                                                }
                                            })
                                            .child("Logs"),
                                    )
                                    .child(
                                        div()
                                            .px_3()
                                            
                                            .py(px(1.5))
                                            .text_xs()
                                            .font_weight(FontWeight::MEDIUM)
                                            .cursor_pointer()
                                            .rounded(px(3.)) // Smaller radius like Zed
                                            .bg(if self.current_view == AppView::LibraryView {
                                                rgb(0x1e1e2e) // Zed-style active tab (blue)
                                            } else {
                                                rgb(0x0c0c0e) // Transparent
                                            })
                                            .text_color(
                                                if self.current_view == AppView::LibraryView {
                                                    rgb(0xcdd6f4) // Zed's text
                                                } else {
                                                    rgb(0x646473) // Zed's muted
                                                },
                                            )
                                            .hover(|style| {
                                                if self.current_view != AppView::LibraryView {
                                                    style
                                                        .bg(rgb(0x151515)) // Very subtle hover
                                                        .text_color(rgb(0x9399b2))
                                                } else {
                                                    style
                                                }
                                            })
                                            .on_mouse_down(gpui::MouseButton::Left, {
                                                let view = view.clone();
                                                move |_event, _, cx| {
                                                    view.update(cx, |view, _| {
                                                        view.current_view = AppView::LibraryView
                                                    });
                                                }
                                            })
                                            .child("Library"),
                                    ),
                            ),
                    )
                    .child(
                        // Center: Status and stats - Zed style
                        div()
                            .flex()
                            .items_center()
                            .h_full()
                            .gap_4()
                            
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0x646473)) // Zed's muted
                                    .child(self.status_msg.clone()),
                            )
                            .child(div().w(px(1.0)).h(px(12.0)).bg(rgb(0x1a1a1a))) // Subtle divider
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2() // Tighter spacing
                                    .text_xs()
                                    .text_color(rgb(0x9ca3af))
                                    .child(format!("{} msgs", self.messages.len()))
                                    .child(format!("{} DBC", self.dbc_channels.len()))
                                    .child(format!("{} LIN", self.ldf_channels.len())),
                            ),
                    )
                    .child(
                        // Right: Action buttons and window controls
                        div()
                            .flex()
                            .items_center()
                            .h_full()
                            .gap_2()
                            
                            .child(
                                div()
                                    .px_3()
                                    
                                    .py(px(1.5))
                                    .text_xs()
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(rgb(0xcdd6f4)) // Zed's text
                                    .bg(rgb(0x1a1f2e)) // Zed-style subtle green
                                    .rounded(px(3.)) // Smaller radius
                                    .cursor_pointer()
                                    .hover(|style| style.bg(rgb(0x252f3a))) // Subtle hover
                                    .on_mouse_down(gpui::MouseButton::Left, {
                                        let view = view.clone();
                                        move |_event, _, cx| {
                                            let view = view.clone();
                                            cx.spawn(async move |cx| {
                                                if let Some(file) = rfd::AsyncFileDialog::new()
                                                    .add_filter("BLF Files", &["blf", "bin"])
                                                    .pick_file()
                                                    .await
                                                {
                                                    let path = file.path().to_owned();

                                                    let _ = cx.update(|cx| {
                                                        view.update(cx, |view, _| {
                                                            view.status_msg =
                                                                "Loading BLF...".into();
                                                        });
                                                    });

                                                    let result = cx
                                                        .background_executor()
                                                        .spawn(async move {
                                                            read_blf_from_file(&path).map_err(|e| {
                                                                anyhow::Error::msg(format!(
                                                                    "{:?}",
                                                                    e
                                                                ))
                                                            })
                                                        })
                                                        .await;

                                                    let _ = cx.update(|cx| {
                                                        view.update(cx, |view, cx| {
                                                            view.apply_blf_result(result);
                                                            cx.notify(); // Notify that the window needs to be re-rendered
                                                        });
                                                    });
                                                }
                                                Ok::<(), anyhow::Error>(())
                                            })
                                            .detach();
                                        }
                                    })
                                    .child("Open BLF"),
                            )
                            .child(
                                // Window controls separator
                                div().w(px(12.)), // Smaller separator
                            )
                            .child(
                                // Minimize button - Zed style
                                div()
                                    
                                    .w(px(28.)) // Slightly smaller
                                    .h(px(28.))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .cursor_pointer()
                                    .hover(|style| style.bg(rgb(0x121212))) // Very subtle hover
                                    .child(div().w(px(10.)).h(px(1.)).bg(rgb(0x646473))) // Zed's muted
                                    .on_mouse_down(
                                        gpui::MouseButton::Left,
                                        |_event, window, cx| {
                                            window.minimize_window();
                                        },
                                    ),
                            )
                            .child(
                                // Maximize/Restore button - Zed style
                                div()
                                    
                                    .w(px(28.)) // Slightly smaller
                                    .h(px(28.))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .cursor_pointer()
                                    .hover(|style| style.bg(rgb(0x121212))) // Very subtle hover
                                    .child(
                                        div()
                                            .w(px(9.))
                                            .h(px(9.))
                                            .border_1()
                                            .border_color(rgb(0x646473)), // Zed's muted
                                    )
                                    .on_mouse_down(gpui::MouseButton::Left, {
                                        let view = view.clone();
                                        move |_event, window, cx| {
                                            // Use our custom toggle that properly manages state
                                            view.update(cx, |view, cx| {
                                                view.toggle_maximize(window, cx);
                                            });
                                        }
                                    }),
                            )
                            .child(
                                // Close button - Zed style
                                div()
                                    
                                    .w(px(28.)) // Slightly smaller
                                    .h(px(28.))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .cursor_pointer()
                                    .hover(|style| style.bg(rgb(0x3a1a1a))) // Subtle red hover
                                    .child(div().text_sm().text_color(rgb(0x646473)).child("×")) // Zed's muted
                                    .on_mouse_down(
                                        gpui::MouseButton::Left,
                                        |_event, window, cx| {
                                            window.remove_window();
                                        },
                                    ),
                            ),
                    ),
            )
            .child(
                // Content area - Zed style
                div()
                    .flex_1()
                    .bg(rgb(0x0c0c0e)) // Zed's main background
                    .overflow_hidden()
                    .child(match self.current_view {
                        AppView::LogView => {
                            self.render_log_view(cx.entity().clone()).into_any_element()
                        }
                        AppView::ConfigView => self.render_config_view(cx).into_any_element(),

                        AppView::LibraryView => self.render_library_view(cx).into_any_element(),
                    }),
            )
            .child(
                // Zed-style status bar at bottom
                div()
                    .h(px(24.))
                    .bg(rgb(0x1e1e1e))
                    .border_t_1()
                    .border_color(rgb(0x2a2a2a))
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_3()
                    .text_xs()
                    .text_color(rgb(0x9ca3af))
                    .child(
                        // Left: File info
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .child(div().child(format!("{} messages", self.messages.len())))
                            .child(div().child(format!("{} DBC channels", self.dbc_channels.len())))
                            .child(
                                div().child(format!("{} LIN channels", self.ldf_channels.len())),
                            ),
                    )
                    .child(
                        // Right: Status with resize handle
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .child(div().child(if self.is_streaming_mode {
                                "Streaming Mode"
                            } else {
                                "Normal Mode"
                            }))
                            .child(div().child(self.status_msg.clone()))
                            .child(
                                // Resize handle in bottom-right corner
                                div()
                                    .ml_2()
                                    .w(px(16.))
                                    .h(px(16.))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .child(
                                        div()
                                            .w(px(10.))
                                            .h(px(10.))
                                            .border_r_2()
                                            .border_b_2()
                                            .border_color(rgb(0x6b7280))
                                            .opacity(0.5),
                                    )
                                    .hover(|style| style.opacity(1.0)),
                            ),
                    ),
            )
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
        match self
            .library_manager
            .delete_library(library_id, &self.app_config.mappings)
        {
            Ok(_) => {
                self.status_msg = format!("Library deleted").into();
                if self.selected_library_id.as_ref() == Some(&library_id.to_string()) {
                    self.selected_library_id = None;
                }
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

    /// Load a library version
    pub fn load_library_version(
        &mut self,
        library_id: &str,
        version_name: &str,
        cx: &mut Context<Self>,
    ) {
        // Reset add channel input state when loading a new version
        self.hide_add_channel_input(cx);
        
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

        // Load the database for each channel in the version
        let channel_dbs = &version.channel_databases;

        if channel_dbs.is_empty() {
            // Use the default path (backward compatibility)
            let path = &version.path;
            match self
                .library_manager
                .load_database(path, library.channel_type)
            {
                Ok(database) => {
                    match database {
                        crate::library::Database::Dbc(dbc) => {
                            self.dbc_channels.insert(1, dbc);
                        }
                        crate::library::Database::Ldf(ldf) => {
                            self.ldf_channels.insert(1, ldf);
                        }
                    }
                    self.status_msg =
                        format!("Loaded version {} of {}", version_name, library.name).into();
                }
                Err(e) => {
                    self.status_msg = format!("Error loading database: {}", e).into();
                }
            }
        } else {
            // Load all configured channels
            for channel_db in channel_dbs {
                match self
                    .library_manager
                    .load_database(&channel_db.database_path, library.channel_type)
                {
                    Ok(database) => match database {
                        crate::library::Database::Dbc(dbc) => {
                            self.dbc_channels.insert(channel_db.channel_id, dbc);
                        }
                        crate::library::Database::Ldf(ldf) => {
                            self.ldf_channels.insert(channel_db.channel_id, ldf);
                        }
                    },
                    Err(e) => {
                        self.status_msg =
                            format!("Error loading channel {}: {}", channel_db.channel_id, e)
                                .into();
                    }
                }
            }
            self.status_msg = format!(
                "Loaded version {} of {} ({} channels)",
                version_name,
                library.name,
                channel_dbs.len()
            )
            .into();
        }

        cx.notify();
    }

    // ========== Channel Configuration Methods ==========

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
            eprintln!("DEBUG: Manual Read ID: '{}', Listener ID: '{}'", id_text, self.new_channel_id);
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
                let library = self.library_manager.find_library(&library_id).unwrap();
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
        let library = self.library_manager.find_library_mut(&library_id).unwrap();
        if let Some(version) = library.versions.iter_mut().find(|v| v.name == version_name) {
            match version.add_channel_database(channel_db) {
                Ok(_) => {
                    self.status_msg = format!("Channel {} added successfully", channel_id).into();
                    // Keep input row open for continuous adding
                    self.show_add_channel_input = true;

                    // Clear input fields
                    self.new_channel_id.clear();
                    self.new_channel_name.clear();
                    self.new_channel_db_path.clear();

                    // Reset input entities so they can be recreated next time
                    self.channel_id_input = None;
                    self.channel_name_input = None;
                    self.channel_db_path_input = None;

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
