use blf::{read_blf_from_file, BlfResult, LogObject};
use gpui::{prelude::*, *};
use parser::dbc::{DbcDatabase, DbcParser};
use parser::ldf::{LdfDatabase, LdfParser};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

// 定义枚举和结构体
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Copy)]
enum ChannelType {
    CAN,
    LIN,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
struct LibraryVersion {
    name: String,
    path: String,
    date: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
struct SignalLibrary {
    id: String,
    name: String,
    versions: Vec<LibraryVersion>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
struct ChannelMapping {
    #[serde(default = "default_channel_type")]
    channel_type: ChannelType,
    #[serde(alias = "channel")]
    channel_id: u16,
    #[serde(default)]
    path: String,
    #[serde(default)]
    description: String,
    library_id: Option<String>,
    version_name: Option<String>,
}

fn default_channel_type() -> ChannelType {
    ChannelType::CAN
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
struct AppConfig {
    libraries: Vec<SignalLibrary>,
    mappings: Vec<ChannelMapping>,
    active_library_id: Option<String>,
    active_version_name: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum AppView {
    LogView,
    ConfigView,
    ChartView,
}

// 定义我们的根视图结构
struct CanViewApp {
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
    // Tracks whether the window is currently maximized (used for UI state)
    is_maximized: bool,
    // Whether the app is in streaming mode (used in the status area)
    is_streaming_mode: bool,
    // Window state for manual maximize/restore
    saved_window_bounds: Option<Bounds<Pixels>>,
    display_bounds: Option<Bounds<Pixels>>,
    // Scroll handle for uniform_list
    list_scroll_handle: gpui::UniformListScrollHandle,
    // Scrollbar drag state
    scrollbar_drag_state: Option<ScrollbarDragState>,
    // Scroll offset for manual scrolling
    scroll_offset: gpui::Pixels,
    // Cached list container height for scrollbar calculations
    list_container_height: f32,
    // ID display mode: true for decimal, false for hexadecimal
    id_display_decimal: bool,
    // ID filter: Some(id_value) to filter messages, None to show all
    id_filter: Option<u32>,
    // ID filter input text
    id_filter_text: SharedString,
    // Show ID filter input dialog
    show_id_filter_input: bool,
    // Filter dropdown scroll offset
    filter_scroll_offset: gpui::Pixels,
    // Filter dropdown scroll handle - must persist across renders
    filter_scroll_handle: gpui::UniformListScrollHandle,
    // Track if mouse is hovering over filter dropdown
    mouse_over_filter_dropdown: bool,
    // Channel filter: Some(channel_value) to filter messages, None to show all
    channel_filter: Option<u16>,
    // Channel filter input text
    channel_filter_text: SharedString,
    // Show channel filter input dialog
    show_channel_filter_input: bool,
    // Channel filter dropdown scroll offset
    channel_filter_scroll_offset: gpui::Pixels,
    // Channel filter dropdown scroll handle
    channel_filter_scroll_handle: gpui::UniformListScrollHandle,
}

// State for tracking scrollbar drag operation
#[derive(Clone)]
struct ScrollbarDragState {
    start_y: Pixels,
    start_scroll_offset: f32,
}

impl CanViewApp {
    fn new() -> Self {
        // 启动时加载配�?        app.load_startup_config();
        Self {
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
            // Channel filter
            channel_filter: None,
            channel_filter_text: "".into(),
            show_channel_filter_input: false,
            channel_filter_scroll_offset: px(0.0),
            channel_filter_scroll_handle: gpui::UniformListScrollHandle::new(),
        }
    }

    fn load_startup_config(&mut self) {
        let path = PathBuf::from("multi_channel_config.json");
        if path.exists() {
            self.status_msg = "Found saved config, loading...".into();
            if let Ok(content) = std::fs::read_to_string(&path) {
                match serde_json::from_str::<AppConfig>(&content) {
                    Ok(mut config) => {
                        // Fill in missing paths from library versions for legacy configs
                        for mapping in &mut config.mappings {
                            if mapping.path.is_empty() {
                                if let Some(lib_id) = &mapping.library_id {
                                    if let Some(version_name) = &mapping.version_name {
                                        if let Some(library) =
                                            config.libraries.iter().find(|l| l.id == *lib_id)
                                        {
                                            if let Some(version) = library
                                                .versions
                                                .iter()
                                                .find(|v| v.name == *version_name)
                                            {
                                                mapping.path = version.path.clone();
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        self.app_config = config.clone();
                        self.config_dir = Some(
                            path.parent()
                                .unwrap_or(std::path::Path::new("../../../../.."))
                                .to_path_buf(),
                        );
                        self.config_file_path = Some(path);
                        self.status_msg = "Configuration loaded.".into();

                        // Auto-Apply if Active Version Exists
                        if let (Some(l_id), Some(v_name)) =
                            (&config.active_library_id, &config.active_version_name)
                        {
                            self.apply_active_version(l_id, v_name);
                        }
                    }
                    Err(e) => {
                        self.status_msg =
                            format!("Config load error: {}. Using default config.", e).into();
                        // Initialize with empty config instead of failing
                        self.app_config = AppConfig::default();
                    }
                }
            }
        } else {
            self.status_msg = "Ready - GPUI version initialized".into();
        }
    }

    fn apply_active_version(&mut self, library_id: &str, version_name: &str) {
        if let Some(parent) = &self.config_dir {
            self.status_msg = format!("Applying active version: {}...", version_name).into();

            self.dbc_channels.clear();
            self.ldf_channels.clear();

            for mapping in &self.app_config.mappings {
                if mapping.library_id.as_ref() == Some(&library_id.to_string())
                    && mapping.version_name.as_ref() == Some(&version_name.to_string())
                {
                    // Try to find the actual file path from library versions
                    let file_path = if !mapping.path.is_empty() {
                        mapping.path.clone()
                    } else if let Some(library) = self
                        .app_config
                        .libraries
                        .iter()
                        .find(|l| l.id == library_id)
                    {
                        if let Some(version) =
                            library.versions.iter().find(|v| v.name == version_name)
                        {
                            version.path.clone()
                        } else {
                            continue;
                        }
                    } else {
                        continue;
                    };

                    let full_path = parent.join(&file_path);
                    if let Ok(content) = std::fs::read_to_string(&full_path) {
                        match mapping.channel_type {
                            ChannelType::CAN => {
                                let parser = DbcParser::new();
                                if let Ok(db) = parser.parse(&content) {
                                    self.dbc_channels.insert(mapping.channel_id, db);
                                }
                            }
                            ChannelType::LIN => {
                                let parser = LdfParser::new();
                                if let Ok(db) = parser.parse(&content) {
                                    self.ldf_channels.insert(mapping.channel_id, db);
                                }
                            }
                        }
                    }
                }
            }
            self.status_msg = format!(
                "Loaded {} DBC and {} LIN channels",
                self.dbc_channels.len(),
                self.ldf_channels.len()
            )
            .into();
        }
    }

    fn apply_blf_result(&mut self, result: anyhow::Result<BlfResult>) {
        match result {
            Ok(result) => {
                self.status_msg = format!("Loaded BLF: {} objects", result.objects.len()).into();

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
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Config Files", &["json"])
            .pick_file()
        {
            self.status_msg = "Loading config...".into();
            if let Ok(content) = std::fs::read_to_string(&path) {
                match serde_json::from_str::<AppConfig>(&content) {
                    Ok(config) => {
                        self.app_config = config;
                        self.config_dir = Some(
                            path.parent()
                                .unwrap_or(std::path::Path::new("../../../../.."))
                                .to_path_buf(),
                        );
                        self.config_file_path = Some(path);
                        self.status_msg = "Configuration loaded successfully".into();
                    }
                    Err(e) => {
                        self.status_msg = format!("Config Error: {}", e).into();
                    }
                }
            }
        }
    }

    #[allow(dead_code)]
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
                let data_hex = can_msg
                    .data
                    .iter()
                    .take(can_msg.dlc as usize)
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
                    can_msg.dlc.to_string(),
                    data_hex,
                    signals,
                )
            }
            LogObject::LinMessage(lin_msg) => {
                let timestamp = lin_msg.header.object_time_stamp;
                let time_str = self.get_timestamp_string(timestamp);
                let data_hex = lin_msg
                    .data
                    .iter()
                    .take(lin_msg.dlc as usize)
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
                    lin_msg.dlc.to_string(),
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
            rgb(0x181818)
        } else {
            rgb(0x1a1a1a)
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
                    .w(px(100.))
                    .px_3()
                    .py_1()
                    .text_color(rgb(0x9ca3af))
                    .child(time_str),
            )
            .child(
                div()
                    .w(px(40.))
                    .px_2()
                    .py_1()
                    .text_color(rgb(0x60a5fa))
                    .child(channel_id.to_string()),
            )
            .child(
                div()
                    .w(px(50.))
                    .px_2()
                    .py_1()
                    .text_color(rgb(0x34d399))
                    .child(msg_type),
            )
            .child(
                div()
                    .w(px(70.))
                    .px_2()
                    .py_1()
                    .text_color(rgb(0xfbbf24))
                    .child(id_str),
            )
            .child(div().w(px(40.)).px_2().py_1().child(dlc_str))
            .child(
                div()
                    .w(px(150.))
                    .px_2()
                    .py_1()
                    .text_color(rgb(0xa78bfa))
                    .child(data_str),
            )
            .child(
                div()
                    .flex_1()
                    .px_2()
                    .py_1()
                    .text_color(rgb(0x9ca3af))
                    .child(signals_str),
            )
    }
}

// 实现视图

impl Render for CanViewApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Update container height based on current window size
        self.update_container_height(window);

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
                // Unified top bar with all options
                div()
                    .h(px(56.))
                    .bg(rgb(0x181818))
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_4()
                    .border_b_1()
                    .border_color(rgb(0x2a2a2a))
                    .window_control_area(WindowControlArea::Drag)
                    .child(
                        // Left: App branding and navigation tabs (draggable area)
                        div()
                            .flex()
                            .items_center()
                            .h_full()
                            .gap_6()
                            .window_control_area(WindowControlArea::Drag)
                            .bg(rgb(0x151515))
                            .rounded(px(6.))
                            .px_2()
                            .py_1()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        // App logo icon - simplified CAN bus waveform
                                        div()
                                            .w(px(24.))
                                            .h(px(24.))
                                            .rounded(px(6.))
                                            .bg(rgb(0x1e293b))
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .gap(px(2.))
                                            .child(
                                                div()
                                                    .w(px(3.))
                                                    .h(px(3.))
                                                    .rounded(px(1.5))
                                                    .bg(rgb(0x34d399)),
                                            )
                                            .child(
                                                div()
                                                    .w(px(3.))
                                                    .h(px(3.))
                                                    .rounded(px(1.5))
                                                    .bg(rgb(0x60a5fa)),
                                            )
                                            .child(
                                                div()
                                                    .w(px(4.))
                                                    .h(px(4.))
                                                    .rounded(px(2.))
                                                    .bg(rgb(0x818cf8)),
                                            )
                                            .child(
                                                div()
                                                    .w(px(3.))
                                                    .h(px(3.))
                                                    .rounded(px(1.5))
                                                    .bg(rgb(0x60a5fa)),
                                            )
                                            .child(
                                                div()
                                                    .w(px(3.))
                                                    .h(px(3.))
                                                    .rounded(px(1.5))
                                                    .bg(rgb(0x34d399)),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .text_color(rgb(0xffffff))
                                            .font_weight(FontWeight::BOLD)
                                            .text_base()
                                            .child("CANVIEW"),
                                    ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_1()
                                    .child(
                                        div()
                                            .px_3()
                                            .py_1()
                                            .text_xs()
                                            .font_weight(FontWeight::MEDIUM)
                                            .cursor_pointer()
                                            .rounded(px(4.))
                                            .bg(if self.current_view == AppView::LogView {
                                                rgb(0x3b82f6)
                                            } else {
                                                rgb(0x2a2a2a)
                                            })
                                            .text_color(if self.current_view == AppView::LogView {
                                                rgb(0xffffff)
                                            } else {
                                                rgb(0x9ca3af)
                                            })
                                            .hover(|style| {
                                                if self.current_view != AppView::LogView {
                                                    style
                                                        .bg(rgb(0x374151))
                                                        .text_color(rgb(0xd1d5db))
                                                } else {
                                                    style
                                                }
                                            })
                                            .on_mouse_down(gpui::MouseButton::Left, {
                                                let view = view.clone();
                                                move |_, _, cx| {
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
                                            .py_1()
                                            .text_xs()
                                            .font_weight(FontWeight::MEDIUM)
                                            .cursor_pointer()
                                            .rounded(px(4.))
                                            .bg(if self.current_view == AppView::ConfigView {
                                                rgb(0x3b82f6)
                                            } else {
                                                rgb(0x2a2a2a)
                                            })
                                            .text_color(
                                                if self.current_view == AppView::ConfigView {
                                                    rgb(0xffffff)
                                                } else {
                                                    rgb(0x9ca3af)
                                                },
                                            )
                                            .hover(|style| {
                                                if self.current_view != AppView::ConfigView {
                                                    style
                                                        .bg(rgb(0x374151))
                                                        .text_color(rgb(0xd1d5db))
                                                } else {
                                                    style
                                                }
                                            })
                                            .on_mouse_down(gpui::MouseButton::Left, {
                                                let view = view.clone();
                                                move |_, _, cx| {
                                                    view.update(cx, |view, _| {
                                                        view.current_view = AppView::ConfigView
                                                    });
                                                }
                                            })
                                            .child("Config"),
                                    )
                                    .child(
                                        div()
                                            .px_3()
                                            .py_1()
                                            .text_xs()
                                            .font_weight(FontWeight::MEDIUM)
                                            .cursor_pointer()
                                            .rounded(px(4.))
                                            .bg(if self.current_view == AppView::ChartView {
                                                rgb(0x3b82f6)
                                            } else {
                                                rgb(0x2a2a2a)
                                            })
                                            .text_color(
                                                if self.current_view == AppView::ChartView {
                                                    rgb(0xffffff)
                                                } else {
                                                    rgb(0x9ca3af)
                                                },
                                            )
                                            .hover(|style| {
                                                if self.current_view != AppView::ChartView {
                                                    style
                                                        .bg(rgb(0x374151))
                                                        .text_color(rgb(0xd1d5db))
                                                } else {
                                                    style
                                                }
                                            })
                                            .on_mouse_down(gpui::MouseButton::Left, {
                                                let view = view.clone();
                                                move |_, _, cx| {
                                                    view.update(cx, |view, _| {
                                                        view.current_view = AppView::ChartView
                                                    });
                                                }
                                            })
                                            .child("Analytics"),
                                    ),
                            ),
                    )
                    .child(
                        // Center: Status and stats
                        div()
                            .flex()
                            .items_center()
                            .h_full()
                            .gap_4()
                            .window_control_area(WindowControlArea::Drag)
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0x6b7280))
                                    .child(self.status_msg.clone()),
                            )
                            .child(div().w(px(1.)).h(px(16.)).bg(rgb(0x374151)))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_3()
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
                            .window_control_area(WindowControlArea::Drag)
                            .child(
                                div()
                                    .px_3()
                                    .py_1()
                                    .text_xs()
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(rgb(0xffffff))
                                    .bg(rgb(0x059669))
                                    .rounded(px(4.))
                                    .cursor_pointer()
                                    .hover(|style| style.bg(rgb(0x047857)))
                                    .on_mouse_down(gpui::MouseButton::Left, {
                                        let view = view.clone();
                                        move |_, _, app| {
                                            let view = view.clone();
                                            app.spawn(async move |cx| {
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
                                    .child("📂 Open BLF"),
                            )
                            .child(
                                div()
                                    .px_3()
                                    .py_1()
                                    .text_xs()
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(rgb(0xffffff))
                                    .bg(rgb(0xd97706))
                                    .rounded(px(4.))
                                    .cursor_pointer()
                                    .hover(|style| style.bg(rgb(0xb45309)))
                                    .on_mouse_down(gpui::MouseButton::Left, {
                                        let view = view.clone();
                                        move |_, _, cx| {
                                            view.update(cx, |view, cx| view.load_config(cx));
                                        }
                                    })
                                    .child("⚙️ Load Config"),
                            )
                            .child(
                                div()
                                    .px_3()
                                    .py_1()
                                    .text_xs()
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(rgb(0x9ca3af))
                                    .bg(rgb(0x374151))
                                    .rounded(px(4.))
                                    .cursor_pointer()
                                    .hover(|style| {
                                        style.bg(rgb(0x4b5563)).text_color(rgb(0xd1d5db))
                                    })
                                    .child("💾 Export"),
                            )
                            .child(
                                // Window controls separator
                                div().w(px(16.)),
                            )
                            .child(
                                // Minimize button
                                div()
                                    .w(px(32.))
                                    .h(px(32.))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .cursor_pointer()
                                    .hover(|style| style.bg(rgb(0x374151)))
                                    .child(div().w(px(12.)).h(px(1.)).bg(rgb(0x9ca3af)))
                                    .on_mouse_down(
                                        gpui::MouseButton::Left,
                                        |_event, window, _app| {
                                            window.minimize_window();
                                        },
                                    ),
                            )
                            .child(
                                // Maximize/Restore button
                                div()
                                    .w(px(32.))
                                    .h(px(32.))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .cursor_pointer()
                                    .hover(|style| style.bg(rgb(0x374151)))
                                    .child(
                                        div()
                                            .w(px(10.))
                                            .h(px(10.))
                                            .border_1()
                                            .border_color(rgb(0x9ca3af)),
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
                                // Close button
                                div()
                                    .w(px(32.))
                                    .h(px(32.))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .cursor_pointer()
                                    .hover(|style| style.bg(rgb(0xdc2626)))
                                    .child(div().text_sm().text_color(rgb(0x9ca3af)).child("×"))
                                    .on_mouse_down(
                                        gpui::MouseButton::Left,
                                        |_event, window, _app| {
                                            window.remove_window();
                                        },
                                    ),
                            ),
                    ),
            )
            .child(
                // Content area
                div()
                    .flex_1()
                    .bg(rgb(0x181818))
                    .overflow_hidden()
                    .child(match self.current_view {
                        AppView::LogView => {
                            self.render_log_view(cx.entity().clone()).into_any_element()
                        }
                        AppView::ConfigView => self.render_config_view().into_any_element(),
                        AppView::ChartView => self.render_chart_view().into_any_element(),
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
            // Channel filter
            channel_filter: None,
            channel_filter_text: "".into(),
            show_channel_filter_input: false,
            channel_filter_scroll_offset: px(0.0),
            channel_filter_scroll_handle: gpui::UniformListScrollHandle::new(),
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

        let dbc_channels = self.dbc_channels.clone();
        let ldf_channels = self.ldf_channels.clone();
        let start_time = self.start_time;
        let scroll_handle = self.list_scroll_handle.clone();
        let total_messages = self.messages.len();
        let id_display_decimal = self.id_display_decimal;
        let id_filter = self.id_filter;
        let id_filter_text = self.id_filter_text.clone();

        // Calculate column widths based on ALL messages (not filtered), to keep layout consistent
        let (time_width, ch_width, type_width, id_width, dlc_width) =
            Self::calculate_column_widths(&self.messages, &dbc_channels, &ldf_channels, start_time);

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
                let total_messages = view_for_mouse_move.read(cx).messages.len();
                let total_content_height = total_messages as f32 * row_h;
                let max_scroll_offset = (total_content_height - container_h).max(0.0);

                if max_scroll_offset <= 0.0 {
                    return;
                }

                // Calculate thumb dimensions
                let thumb_ratio = (container_h / total_content_height).min(1.0);
                let thumb_h = (thumb_ratio * container_h).max(20.0);
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

                // Convert to item index
                let visible_items = (container_h / row_h).ceil() as usize;
                let max_start_index = total_messages.saturating_sub(visible_items);

                // Calculate target index based on scroll offset
                let target_index = ((new_scroll_offset / row_h).round() as usize).clamp(0, max_start_index);

                // Use Bottom strategy only when we're at the very end
                // This ensures the last row is visible at the bottom
                if target_index >= max_start_index.saturating_sub(1) {
                    view_for_mouse_move.read(cx).list_scroll_handle.scroll_to_item_strict(
                        total_messages.saturating_sub(1),
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
                                .justify_between()
                                .flex_shrink_0()
                                .whitespace_nowrap()
                                .overflow_hidden()
                                .child("CH")
                                .when(self.channel_filter.is_some(), |this| {
                                    this.child(
                                        gpui::div()
                                            .text_color(rgb(0x3b82f6))
                                            .child("✓")
                                    )
                                })
                                .child(
                                    div()
                                        .cursor_pointer()
                                        .on_mouse_down(gpui::MouseButton::Left, {
                                            let view = view.clone();
                                            move |_event, _window, cx| {
                                                view.update(cx, |app, cx| {
                                                    app.show_channel_filter_input = !app.show_channel_filter_input;
                                                    cx.notify();
                                                });
                                            }
                                        })
                                        .child("⚙")
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
                        // Calculate scrollbar dimensions based on actual content
                        let row_height = 22.0;
                        let total_height = total_messages as f32 * row_height;
                        let container_height = self.list_container_height;

                        let max_scroll = (total_height - container_height).max(0.0);

                        // Calculate thumb height with minimum visibility
                        let thumb_height_ratio = if total_height > 0.0 {
                            (container_height / total_height).min(1.0)
                        } else {
                            1.0
                        };
                        let thumb_height = (thumb_height_ratio * container_height).max(20.0);  // Minimum 20px
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
                            let max_start_index = total_messages.saturating_sub(visible_items);
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

                                        if total_messages == 0 {
                                            return;
                                        }

                                        // Calculate thumb dimensions
                                        let total_content_height = total_messages as f32 * row_h;
                                        let thumb_ratio = (container_h / total_content_height).min(1.0);
                                        let thumb_h = (thumb_ratio * container_h).max(20.0);
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

                                        // Calculate target index
                                        let visible_items = (container_h / row_h).ceil() as usize;
                                        let max_start_index = total_messages.saturating_sub(visible_items);

                                        let target_index = if max_start_index > 0 {
                                            (scroll_ratio * max_start_index as f32).round() as usize
                                        } else {
                                            0
                                        }.clamp(0, max_start_index);

                                        // Use Bottom strategy only when we're at the very end
                                        // This ensures the last row is visible at the bottom
                                        if target_index >= max_start_index.saturating_sub(1) {
                                            scroll_handle_clone.scroll_to_item_strict(
                                                total_messages.saturating_sub(1),
                                                gpui::ScrollStrategy::Bottom
                                            );
                                        } else {
                                            scroll_handle_clone.scroll_to_item_strict(target_index, gpui::ScrollStrategy::Top);
                                        }
                                        cx.notify(view_for_scroll_track.entity_id());
                                        cx.stop_propagation();
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
                                                    });
                                                });

                                                cx.stop_propagation();
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
                                    cx.stop_propagation();
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
                                    cx.stop_propagation();
                                    view_for_scroll.update(cx, |app, cx| {
                                        app.mouse_over_filter_dropdown = true;
                                        cx.notify();
                                    });
                                }
                            })
                            .on_mouse_down(gpui::MouseButton::Left, {
                                let view_for_scroll = view_for_scroll.clone();
                                move |_event, _window, cx| {
                                    cx.stop_propagation();
                                    view_for_scroll.update(cx, |app, cx| {
                                        app.mouse_over_filter_dropdown = true;
                                        cx.notify();
                                    });
                                }
                            })
                            // Capture wheel events at container level and manually scroll
                            .on_scroll_wheel(move |event, _window, cx| {
                                cx.stop_propagation();

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
                                                        cx.stop_propagation();
                                                    })
                                                    .on_mouse_up(gpui::MouseButton::Left, move |_event, _window, cx| {
                                                        cx.stop_propagation();
                                                    })
                                                    .on_mouse_down(gpui::MouseButton::Left, {
                                                        let view = view_clone1.clone();
                                                        move |_event, _window, cx| {
                                                            eprintln!("Selected ID: {}", id);
                                                            cx.stop_propagation();
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
                                    cx.stop_propagation();
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
                                    cx.stop_propagation();
                                    view_for_scroll.update(cx, |app, cx| {
                                        app.mouse_over_filter_dropdown = true;
                                        cx.notify();
                                    });
                                }
                            })
                            .on_mouse_down(gpui::MouseButton::Left, {
                                let view_for_scroll = view_for_scroll.clone();
                                move |_event, _window, cx| {
                                    cx.stop_propagation();
                                    view_for_scroll.update(cx, |app, cx| {
                                        app.mouse_over_filter_dropdown = true;
                                        cx.notify();
                                    });
                                }
                            })
                            // Capture wheel events at container level and manually scroll
                            .on_scroll_wheel(move |event, _window, cx| {
                                cx.stop_propagation();

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
                                                        cx.stop_propagation();
                                                    })
                                                    .on_mouse_up(gpui::MouseButton::Left, move |_event, _window, cx| {
                                                        cx.stop_propagation();
                                                    })
                                                    .on_mouse_down(gpui::MouseButton::Left, {
                                                        let view = view_clone2.clone();
                                                        move |_event, _window, cx| {
                                                            eprintln!("Selected Channel: {}", channel);
                                                            cx.stop_propagation();
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
                            cx.stop_propagation();
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
                            cx.stop_propagation();
                            view_for_scroll.update(cx, |app, cx| {
                                app.mouse_over_filter_dropdown = true;
                                cx.notify();
                            });
                        }
                    })
                    .on_mouse_down(gpui::MouseButton::Left, {
                        let view_for_scroll = view_for_scroll.clone();
                        move |_event, _window, cx| {
                            cx.stop_propagation();
                            view_for_scroll.update(cx, |app, cx| {
                                app.mouse_over_filter_dropdown = true;
                                cx.notify();
                            });
                        }
                    })
                    // Capture wheel events at container level and manually scroll
                    .on_scroll_wheel(move |event, _window, cx| {
                        cx.stop_propagation();

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
                                                cx.stop_propagation();
                                            })
                                            .on_mouse_up(
                                                gpui::MouseButton::Left,
                                                move |_event, _window, cx| {
                                                    cx.stop_propagation();
                                                },
                                            )
                                            .on_mouse_down(gpui::MouseButton::Left, {
                                                let view = view.clone();
                                                move |_event, _window, cx| {
                                                    eprintln!("Selected Channel: {}", channel);
                                                    cx.stop_propagation();
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

    // Calculate optimal column widths based on ALL messages content
    // Note: DATA column uses flex_1() so we don't calculate its width
    fn calculate_column_widths(
        messages: &[LogObject],
        _dbc_channels: &HashMap<u16, DbcDatabase>,
        _ldf_channels: &HashMap<u16, LdfDatabase>,
        start_time: Option<chrono::NaiveDateTime>,
    ) -> (
        gpui::Pixels,
        gpui::Pixels,
        gpui::Pixels,
        gpui::Pixels,
        gpui::Pixels,
    ) {
        // Define minimum widths for each column (for header text)
        let mut max_time_width = 50.0_f32; // "TIME" header
        let mut max_ch_width = 30.0_f32; // "CH" header
        let mut max_type_width = 50.0_f32; // "TYPE" header
        let mut max_id_width = 80.0_f32; // "ID" header with gear icon (ID + 10 + ⚙ = ~70px)
        let mut max_dlc_width = 40.0_f32; // "DLC" header

        // Calculate widths based on ALL messages
        // Use a smarter sampling strategy:
        // - For small datasets (<1000), scan all
        // - For large datasets, scan in intervals to get representative sample
        let sample_size = messages.len();
        let step = if sample_size > 5000 {
            sample_size / 1000 // Sample ~1000 messages spread evenly
        } else if sample_size > 1000 {
            sample_size / 500 // Sample ~500 messages spread evenly
        } else {
            1 // Scan all messages
        };

        for (i, msg) in messages.iter().enumerate() {
            // Skip messages based on step size for large datasets
            if i % step != 0 {
                continue;
            }

            let (time_str, channel_id, msg_type, id_str, dlc_str, _data_str) =
                Self::get_message_strings(msg, start_time, true); // Use decimal for width calculation

            // Calculate exact width needed for each column
            // Using 8.0 pixels per character (monospace font approximation)
            // Add padding: horizontal padding (px_2 or px_3) + some margin
            max_time_width = max_time_width.max(time_str.len() as f32 * 8.0 + 16.0); // px_3 = 12px + 4px margin
            max_ch_width = max_ch_width.max(channel_id.to_string().len() as f32 * 8.0 + 10.0); // px_2 = 8px + 2px margin
            max_type_width = max_type_width.max(msg_type.len() as f32 * 8.0 + 10.0);
            max_id_width = max_id_width.max(id_str.len() as f32 * 8.0 + 10.0);
            max_dlc_width = max_dlc_width.max(dlc_str.len() as f32 * 8.0 + 10.0);
        }

        // Apply maximum limits to prevent columns from becoming too wide
        // This ensures the table remains readable even with very long content
        max_time_width = max_time_width.min(300.0);
        max_ch_width = max_ch_width.min(80.0);
        max_type_width = max_type_width.min(120.0);
        max_id_width = max_id_width.min(100.0);
        max_dlc_width = max_dlc_width.min(80.0);

        // Round to integer pixels to ensure consistency across all rows
        // This prevents rounding errors that can cause misalignment
        max_time_width = max_time_width.round();
        max_ch_width = max_ch_width.round();
        max_type_width = max_type_width.round();
        max_id_width = max_id_width.round();
        max_dlc_width = max_dlc_width.round();

        // Return calculated widths (excluding DATA which uses flex_1)
        (
            px(max_time_width),
            px(max_ch_width),
            px(max_type_width),
            px(max_id_width),
            px(max_dlc_width),
        )
    }

    // Helper to extract message strings without rendering
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

                let data_hex = can_msg
                    .data
                    .iter()
                    .take(can_msg.dlc as usize)
                    .map(|b| format!("{:02X}", b))
                    .collect::<Vec<_>>()
                    .join(" ");

                (
                    time_str,
                    can_msg.channel,
                    "CAN".to_string(),
                    format_id(can_msg.id),
                    can_msg.dlc.to_string(),
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

                let data_hex = can_msg
                    .data
                    .iter()
                    .take(can_msg.dlc as usize)
                    .map(|b| format!("{:02X}", b))
                    .collect::<Vec<_>>()
                    .join(" ");

                (
                    time_str,
                    can_msg.channel,
                    "CAN2".to_string(),
                    format_id(can_msg.id),
                    can_msg.dlc.to_string(),
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

                let data_hex = fd_msg
                    .data
                    .iter()
                    .take(fd_msg.dlc as usize)
                    .map(|b| format!("{:02X}", b))
                    .collect::<Vec<_>>()
                    .join(" ");

                (
                    time_str,
                    fd_msg.channel,
                    "CAN_FD".to_string(),
                    format_id(fd_msg.id),
                    fd_msg.dlc.to_string(),
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

                let data_hex = fd_msg
                    .data
                    .iter()
                    .take(fd_msg.valid_data_bytes as usize)
                    .map(|b| format!("{:02X}", b))
                    .collect::<Vec<_>>()
                    .join(" ");

                (
                    time_str,
                    fd_msg.channel as u16,
                    "CAN_FD64".to_string(),
                    format_id(fd_msg.id),
                    fd_msg.dlc.to_string(),
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

                let data_hex = lin_msg
                    .data
                    .iter()
                    .take(lin_msg.dlc as usize)
                    .map(|b| format!("{:02X}", b))
                    .collect::<Vec<_>>()
                    .join(" ");

                (
                    time_str,
                    lin_msg.channel,
                    "LIN".to_string(),
                    format_id(lin_msg.id as u32),
                    lin_msg.dlc.to_string(),
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
                    "8".to_string(),
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

                let data_hex = can_msg
                    .data
                    .iter()
                    .take(can_msg.dlc as usize)
                    .map(|b| format!("{:02X}", b))
                    .collect::<Vec<_>>()
                    .join(" ");

                (
                    time_str,
                    can_msg.channel,
                    "CAN".to_string(),
                    format!("0x{:03X}", can_msg.id),
                    can_msg.dlc.to_string(),
                    data_hex,
                )
            }
            LogObject::CanMessage2(can_msg) => {
                let timestamp = can_msg.header.object_time_stamp;
                let time_str = Self::format_timestamp_static(timestamp, start_time);

                let data_hex = can_msg
                    .data
                    .iter()
                    .take(can_msg.dlc as usize)
                    .map(|b| format!("{:02X}", b))
                    .collect::<Vec<_>>()
                    .join(" ");

                (
                    time_str,
                    can_msg.channel,
                    "CAN2".to_string(),
                    format!("0x{:03X}", can_msg.id),
                    can_msg.dlc.to_string(),
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

                let data_hex = fd_msg
                    .data
                    .iter()
                    .take(fd_msg.dlc as usize)
                    .map(|b| format!("{:02X}", b))
                    .collect::<Vec<_>>()
                    .join(" ");

                (
                    time_str,
                    fd_msg.channel, // Convert u8 to u16
                    "CAN_FD".to_string(),
                    format!("0x{:03X}", fd_msg.id),
                    fd_msg.dlc.to_string(),
                    data_hex,
                )
            }
            LogObject::CanFdMessage64(fd_msg) => {
                let timestamp = fd_msg.header.object_time_stamp;
                let time_str = Self::format_timestamp_static(timestamp, start_time);

                let data_hex = fd_msg
                    .data
                    .iter()
                    .take(fd_msg.dlc as usize)
                    .map(|b| format!("{:02X}", b))
                    .collect::<Vec<_>>()
                    .join(" ");

                (
                    time_str,
                    fd_msg.channel as u16, // Convert u8 to u16
                    "CAN_FD64".to_string(),
                    format!("0x{:03X}", fd_msg.id),
                    fd_msg.dlc.to_string(),
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

                let data_hex = lin_msg
                    .data
                    .iter()
                    .take(lin_msg.dlc as usize)
                    .map(|b| format!("{:02X}", b))
                    .collect::<Vec<_>>()
                    .join(" ");

                (
                    time_str,
                    lin_msg.channel,
                    "LIN".to_string(),
                    format!("0x{:02X}", lin_msg.id),
                    lin_msg.dlc.to_string(),
                    data_hex,
                )
            }
            LogObject::LinMessage2(lin_msg) => {
                let timestamp = lin_msg.header.object_time_stamp;
                let time_str = Self::format_timestamp_static(timestamp, start_time);

                (
                    time_str,
                    0_u16,
                    "LIN2".to_string(),
                    "-".to_string(),
                    "-".to_string(),
                    "-".to_string(),
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

    fn render_config_view(&self) -> impl IntoElement {
        div()
            .size_full()
            .p_6()
            .flex()
            .flex_col()
            .gap_4()
            .text_color(rgb(0xd1d5db))
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::MEDIUM)
                    .mb_4()
                    .text_color(rgb(0xffffff))
                    .child("Configuration"),
            )
            .child(
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
                            .child("Status"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x9ca3af))
                            .child(format!("Messages loaded: {}", self.messages.len())),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x9ca3af))
                            .child(format!("DBC channels: {}", self.dbc_channels.len())),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x9ca3af))
                            .child(format!("LIN channels: {}", self.ldf_channels.len())),
                    ),
            )
    }

    fn render_chart_view(&self) -> impl IntoElement {
        div()
            .size_full()
            .p_6()
            .flex()
            .flex_col()
            .gap_4()
            .text_color(rgb(0xd1d5db))
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::MEDIUM)
                    .mb_4()
                    .text_color(rgb(0xffffff))
                    .child("Analytics"),
            )
            .child(
                div()
                    .p_6()
                    .bg(rgb(0x1f1f1f))
                    .border_1()
                    .border_color(rgb(0x2a2a2a))
                    .rounded(px(8.))
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap_3()
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0x9ca3af))
                            .child("Chart visualization coming soon"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x6b7280))
                            .child("Real-time signal analysis and plotting"),
                    ),
            )
    }
}

fn main() {
    env_logger::init();

    let app = Application::new();
    app.run(move |cx| {
        cx.spawn(async move |cx| {
            let options = WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point::new(px(200.0), px(150.0)),
                    size: gpui::Size {
                        width: px(1600.0),
                        height: px(1000.0),
                    },
                })),
                titlebar: Some(TitlebarOptions {
                    title: Some("CANVIEW - Bus Data Analyzer".into()),
                    appears_transparent: true,
                    traffic_light_position: None,
                }),
                kind: gpui::WindowKind::Normal,
                ..Default::default()
            };
            cx.open_window(options, |_window, cx| cx.new(|_cx| CanViewApp::new()))?;
            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
}
