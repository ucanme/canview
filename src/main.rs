use blf::{read_blf_from_file, LogObject};
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

#[derive(Debug, Clone, PartialEq)]
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
}

impl CanViewApp {
    fn new() -> Self {
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
        };

        // 启动时加载配置
        app.load_startup_config();
        app
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
                                .unwrap_or(std::path::Path::new("."))
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

    fn open_blf_file(&mut self, _cx: &mut Context<Self>) {
        // Use synchronous file dialog for simplicity
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("BLF Files", &["blf", "bin"])
            .pick_file()
        {
            self.status_msg = "Loading BLF...".into();
            match read_blf_from_file(&path) {
                Ok(result) => {
                    self.status_msg =
                        format!("Loaded BLF: {} objects", result.objects.len()).into();

                    // Parse start time
                    let st = result.file_stats.measurement_start_time.clone();
                    let date_opt = chrono::NaiveDate::from_ymd_opt(
                        st.year as i32,
                        st.month as u32,
                        st.day as u32,
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
                                .unwrap_or(std::path::Path::new("."))
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

    fn get_timestamp_string(&self, timestamp: u64) -> String {
        if let Some(start) = &self.start_time {
            let msg_time = *start + chrono::Duration::nanoseconds(timestamp as i64);
            msg_time.format("%H:%M:%S%.3f").to_string()
        } else {
            format!("{:.3}", timestamp as f64 / 1000000.0)
        }
    }

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
                            .map(|(name, _signal)| {
                                // Simplified signal extraction for now
                                format!("{}=?", name)
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

                (
                    time_str,
                    lin_msg.channel,
                    "LIN".to_string(),
                    format!("0x{:02X}", lin_msg.id),
                    lin_msg.dlc.to_string(),
                    data_hex,
                    String::new(), // LIN signals parsing not implemented yet
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

        let bg_color = if index % 2 == 0 {
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
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_col()
            .child(
                // Unified top bar with all options
                div()
                    .h(px(48.))
                    .bg(rgb(0x181818))
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_4()
                    .border_b_1()
                    .border_color(rgb(0x2a2a2a))
                    .child(
                        // Left: App branding and navigation tabs
                        div()
                            .flex()
                            .items_center()
                            .gap_6()
                            .child(
                                div()
                                    .text_color(rgb(0xffffff))
                                    .font_weight(FontWeight::BOLD)
                                    .text_base()
                                    .child("CanView"),
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
                                            .child("Charts"),
                                    ),
                            ),
                    )
                    .child(
                        // Center: Status and stats
                        div()
                            .flex()
                            .items_center()
                            .gap_4()
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
                        // Right: Action buttons
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
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
                        AppView::LogView => self.render_log_view().into_any_element(),
                        AppView::ConfigView => self.render_config_view().into_any_element(),
                        AppView::ChartView => self.render_chart_view().into_any_element(),
                    }),
            )
    }
}

impl CanViewApp {
    fn render_log_view(&self) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_col()
            .child(
                // Zed-style header
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
                    .child(div().w(px(100.)).px_3().child("TIME"))
                    .child(div().w(px(40.)).px_2().child("CH"))
                    .child(div().w(px(50.)).px_2().child("TYPE"))
                    .child(div().w(px(70.)).px_2().child("ID"))
                    .child(div().w(px(40.)).px_2().child("DLC"))
                    .child(div().w(px(150.)).px_2().child("DATA"))
                    .child(div().flex_1().px_2().child("SIGNALS")),
            )
            .child(
                // Message list
                div().flex_1().overflow_y_hidden().child(
                    div().w_full().flex().flex_col().children(
                        self.messages
                            .iter()
                            .take(500)
                            .enumerate()
                            .map(|(i, msg)| self.render_message_row(msg, i)),
                    ),
                ),
            )
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
                    title: None,
                    appears_transparent: true,
                    traffic_light_position: Some(Point::new(px(12.0), px(12.0))),
                }),

                ..Default::default()
            };
            cx.open_window(options, |_window, cx| cx.new(|_cx| CanViewApp::new()))?;
            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
}
