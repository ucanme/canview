#![allow(non_snake_case)]

use blf::{read_blf_from_file, LogObject};
use dioxus::prelude::*;
use parser::dbc::{DbcDatabase, DbcParser};
use parser::ldf::{LdfDatabase, LdfParser};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use dioxus::desktop::{Config, WindowBuilder};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Copy)]
enum ChannelType {
    CAN,
    LIN,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
struct LibraryVersion {
    name: String,
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
    channel_type: ChannelType,
    channel_id: u16,
    path: String,
    description: String,
    library_id: Option<String>,
    version_name: Option<String>,
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

#[derive(Clone, PartialEq)]
enum DeleteAction {
    Library(usize),
    Mapping(usize),
}

fn main() {
    let config = Config::new().with_window(
        WindowBuilder::new()
            .with_title("CanView")
            .with_resizable(true),
    );

    dioxus::desktop::launch::launch(App, vec![], config);
}

#[component]
fn App() -> Element {
    let mut messages = use_signal(|| Vec::<LogObject>::new());
    let mut status_msg = use_signal(|| "Ready".to_string());
    // Removed mut as per compiler suggestion
    let mut dbc_channels = use_signal(|| HashMap::<u16, DbcDatabase>::new());
    let mut ldf_channels = use_signal(|| HashMap::<u16, LdfDatabase>::new());

    // View state
    let mut current_view = use_signal(|| AppView::LogView);

    // Config state
    let mut app_config = use_signal(|| AppConfig::default()); // Made mutable for set()
    let mut config_dir = use_signal(|| None::<std::path::PathBuf>);
    let mut config_file_path = use_signal(|| None::<std::path::PathBuf>);
    // Chart state
    let selected_signals = use_signal(|| Vec::<String>::new()); // Format: "ChID:MsgID:SigName"
    let mut start_time = use_signal(|| None::<chrono::NaiveDateTime>);

    let handle_open_click = move |_| {
        spawn(async move {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("BLF Files", &["blf", "bin"])
                .pick_file()
            {
                status_msg.set(format!("Loading BLF..."));
                match read_blf_from_file(&path) {
                    Ok(result) => {
                        status_msg.set(format!("Loaded BLF: {} objects", result.objects.len()));

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
                            start_time.set(Some(chrono::NaiveDateTime::new(date, time)));
                        } else {
                            start_time.set(None);
                        }

                        messages.set(result.objects);
                    }
                    Err(e) => status_msg.set(format!("Error: {:?}", e)),
                }
            }
        });
    };
    let handle_load_config_click = move |_| {
        spawn(async move {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Config Files", &["json"])
                .pick_file()
            {
                status_msg.set(format!("Loading config..."));
                if let Ok(content) = std::fs::read_to_string(&path) {
                    match serde_json::from_str::<AppConfig>(&content) {
                        Ok(config) => {
                            app_config.set(config);
                            config_dir.set(Some(
                                path.parent()
                                    .unwrap_or(std::path::Path::new("."))
                                    .to_path_buf(),
                            ));
                            config_file_path.set(Some(path.clone()));
                            status_msg.set("Configuration loaded successfully".to_string());
                        }
                        Err(e) => status_msg.set(format!("Config Error: {}", e)),
                    }
                }
            }
        });
    };

    // Startup initialization
    use_effect(move || {
        spawn(async move {
            let path = std::path::PathBuf::from("multi_channel_config.json");
            if path.exists() {
                status_msg.set("Found saved config, loading...".to_string());
                if let Ok(content) = std::fs::read_to_string(&path) {
                    match serde_json::from_str::<AppConfig>(&content) {
                        Ok(config) => {
                            // Update State
                            app_config.set(config.clone());
                            let parent = path
                                .parent()
                                .unwrap_or(std::path::Path::new("."))
                                .to_path_buf();
                            config_dir.set(Some(parent.clone()));
                            config_file_path.set(Some(path.clone()));
                            status_msg.set("Configuration loaded.".to_string());

                            // Auto-Apply if Active Version Exists
                            if let (Some(l_id), Some(v_name)) =
                                (&config.active_library_id, &config.active_version_name)
                            {
                                status_msg.set(format!("Applying active version: {}...", v_name));

                                let mut dbc_write = dbc_channels.write();
                                let mut ldf_write = ldf_channels.write();
                                dbc_write.clear();
                                ldf_write.clear();

                                for mapping in config.mappings.iter() {
                                    if mapping.library_id.as_ref() == Some(l_id)
                                        && mapping.version_name.as_ref() == Some(v_name)
                                    {
                                        if mapping.path.is_empty() {
                                            continue;
                                        }
                                        let full_path = parent.join(&mapping.path);
                                        if let Ok(content) = std::fs::read_to_string(&full_path) {
                                            match mapping.channel_type {
                                                ChannelType::CAN => {
                                                    let parser = DbcParser::new();
                                                    if let Ok(db) = parser.parse(&content) {
                                                        dbc_write.insert(mapping.channel_id, db);
                                                    }
                                                }
                                                ChannelType::LIN => {
                                                    let parser = LdfParser::new();
                                                    if let Ok(db) = parser.parse(&content) {
                                                        ldf_write.insert(mapping.channel_id, db);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                status_msg.set(format!("Restored Active Version: {}", v_name));
                            }
                        }
                        Err(e) => status_msg.set(format!("Startup Config Error: {}", e)),
                    }
                }
            } else {
                status_msg.set("Ready (No saved config found)".to_string());
            }
        });
    });

    rsx! {
        div {
            style: "display: flex; flex-direction: column; height: 100vh; font-family: 'Segoe UI', sans-serif; overflow: hidden;",

            // Top Toolbar (Menu Bar)
            div {
                style: "height: 40px; background: #f0f0f0; border-bottom: 1px solid #ccc; display: flex; align-items: center; padding: 0 16px; gap: 16px; user-select: none;",

                // View Switcher
                div {
                    style: "display: flex; background: #ddd; border-radius: 6px; padding: 2px; margin-right: 16px;",
                    button {
                        onclick: move |_| current_view.set(AppView::LogView),
                        style: format!("border: none; padding: 4px 12px; border-radius: 4px; cursor: pointer; font-size: 13px; font-weight: 500; background: {}; color: {};",
                            if current_view() == AppView::LogView { "white" } else { "transparent" },
                            if current_view() == AppView::LogView { "#333" } else { "#666" }
                        ),
                        "Logs"
                    }
                    button {
                        onclick: move |_| current_view.set(AppView::ConfigView),
                        style: format!("border: none; padding: 4px 12px; border-radius: 4px; cursor: pointer; font-size: 13px; font-weight: 500; background: {}; color: {};",
                            if current_view() == AppView::ConfigView { "white" } else { "transparent" },
                            if current_view() == AppView::ConfigView { "#333" } else { "#666" }
                        ),
                        "Config"
                    }
                    button {
                        onclick: move |_| current_view.set(AppView::ChartView),
                        style: format!("border: none; padding: 4px 12px; border-radius: 4px; cursor: pointer; font-size: 13px; font-weight: 500; background: {}; color: {};",
                            if current_view() == AppView::ChartView { "white" } else { "transparent" },
                            if current_view() == AppView::ChartView { "#333" } else { "#666" }
                        ),
                        "Charts"
                    }
                }

                button {
                    onclick: handle_open_click,
                    style: "background: #0078d4; color: white; border: none; font-size: 13px; cursor: pointer; padding: 6px 12px; border-radius: 4px; display: flex; align-items: center; gap: 8px; &:hover {{ background: #106ebe; }}",
                    span { "📂" }
                    "Open BLF File"
                }

                button {
                    onclick: handle_load_config_click,
                    style: "background: #ffc107; color: #000; border: none; font-size: 13px; cursor: pointer; padding: 6px 12px; border-radius: 4px; display: flex; align-items: center; gap: 8px; &:hover {{ background: #e0a800; }}",
                    span { "📥" }
                    "Load Config"
                }


                div {
                    style: "width: 1px; height: 20px; background: #ccc;",
                }

                span {
                    style: "font-size: 13px; color: #555;",
                    "{status_msg}"
                }
            }

            // Content
            div {
                style: "flex: 1; overflow: hidden; background: #fff;",
                match current_view() {
                    AppView::LogView => rsx! {
                        div {
                            style: "height: 100%; overflow: auto;",
                            table {
                                style: "width: 100%; border-collapse: collapse; font-size: 13px;",
                                thead {
                                    tr {
                                        style: "text-align: left; background: #fafafa; border-bottom: 2px solid #ddd; position: sticky; top: 0;",
                                        th { style: "padding: 8px; width: 120px;", "Time" }
                                        th { style: "padding: 8px; width: 50px;", "Ch" }
                                        th { style: "padding: 8px; width: 80px;", "Type" }
                                        th { style: "padding: 8px; width: 80px;", "ID" }
                                        th { style: "padding: 8px; width: 60px;", "DLC" }
                                        th { style: "padding: 8px; width: 200px;", "Data" }
                                        th { style: "padding: 8px;", "Decoded Signals" }
                                    }
                                }
                                tbody {
                                    for (i, msg) in messages.read().iter().enumerate().take(500) {
                                        MessageRow { key: "{i}", msg: msg.clone(), dbc_channels, ldf_channels }
                                    }
                                }
                            }
                        }
                    },
                    AppView::ConfigView => rsx! {
                        ConfigView {
                            app_config: app_config,
                            status_msg: status_msg,
                            dbc_channels: dbc_channels,

                            ldf_channels: ldf_channels,
                            config_dir: config_dir,
                            config_file_path: config_file_path,
                        }
                    },
                    AppView::ChartView => rsx! {
                        ChartView {
                            dbc_channels: dbc_channels,
                            ldf_channels: ldf_channels,
                            messages: messages,
                            selected_signals: selected_signals,
                            start_time: start_time,
                        }
                    }
                }
            }

        }
    }
}

#[component]
fn MessageRow(
    msg: LogObject,
    dbc_channels: Signal<HashMap<u16, DbcDatabase>>,
    ldf_channels: Signal<HashMap<u16, LdfDatabase>>,
) -> Element {
    // Extract data based on type
    let res = match &msg {
        LogObject::CanMessage(m) => Some((
            m.header.object_time_stamp,
            m.channel,
            "CAN",
            format!("0x{:X}", m.id),
            m.dlc,
            m.data
                .iter()
                .take(m.dlc as usize)
                .map(|b| format!("{:02X}", b))
                .collect::<Vec<_>>()
                .join(" "),
            {
                if let Some(db) = dbc_channels.read().get(&m.channel) {
                    if let Some(dbc_msg) = db.messages.get(&m.id) {
                        let signals: Vec<String> = dbc_msg
                            .signals
                            .values()
                            .map(|s| {
                                format!(
                                    "{}: {:.2}{}",
                                    s.name,
                                    s.decode(&m.data),
                                    if s.unit.is_empty() {
                                        "".to_string()
                                    } else {
                                        format!(" {}", s.unit)
                                    }
                                )
                            })
                            .collect();
                        format!("[Ch{}:{}] {}", m.channel, dbc_msg.name, signals.join(", "))
                    } else {
                        "".to_string()
                    }
                } else {
                    "".to_string()
                }
            },
        )),
        LogObject::CanMessage2(m) => Some((
            m.header.object_time_stamp,
            m.channel,
            "CAN",
            format!("0x{:X}", m.id),
            m.dlc,
            m.data
                .iter()
                .take(m.dlc as usize)
                .map(|b| format!("{:02X}", b))
                .collect::<Vec<_>>()
                .join(" "),
            {
                if let Some(db) = dbc_channels.read().get(&m.channel) {
                    if let Some(dbc_msg) = db.messages.get(&m.id) {
                        let signals: Vec<String> = dbc_msg
                            .signals
                            .values()
                            .map(|s| {
                                format!(
                                    "{}: {:.2}{}",
                                    s.name,
                                    s.decode(&m.data),
                                    if s.unit.is_empty() {
                                        "".to_string()
                                    } else {
                                        format!(" {}", s.unit)
                                    }
                                )
                            })
                            .collect();
                        format!("[Ch{}:{}] {}", m.channel, dbc_msg.name, signals.join(", "))
                    } else {
                        "".to_string()
                    }
                } else {
                    "".to_string()
                }
            },
        )),
        LogObject::CanFdMessage(m) => Some((
            m.header.object_time_stamp,
            m.channel,
            "CAN FD",
            format!("0x{:X}", m.id),
            m.valid_data_bytes,
            m.data
                .iter()
                .take(m.valid_data_bytes as usize)
                .map(|b| format!("{:02X}", b))
                .collect::<Vec<_>>()
                .join(" "),
            {
                if let Some(db) = dbc_channels.read().get(&m.channel) {
                    if let Some(dbc_msg) = db.messages.get(&m.id) {
                        let signals: Vec<String> = dbc_msg
                            .signals
                            .values()
                            .map(|s| {
                                format!(
                                    "{}: {:.2}{}",
                                    s.name,
                                    s.decode(&m.data),
                                    if s.unit.is_empty() {
                                        "".to_string()
                                    } else {
                                        format!(" {}", s.unit)
                                    }
                                )
                            })
                            .collect();
                        format!("[Ch{}:{}] {}", m.channel, dbc_msg.name, signals.join(", "))
                    } else {
                        "".to_string()
                    }
                } else {
                    "".to_string()
                }
            },
        )),
        LogObject::CanFdMessage64(m) => Some((
            m.header.object_time_stamp,
            m.channel as u16,
            "CAN FD64",
            format!("0x{:X}", m.id),
            m.valid_data_bytes,
            m.data
                .iter()
                .take(m.valid_data_bytes as usize)
                .map(|b| format!("{:02X}", b))
                .collect::<Vec<_>>()
                .join(" "),
            {
                if let Some(db) = dbc_channels.read().get(&(m.channel as u16)) {
                    if let Some(dbc_msg) = db.messages.get(&m.id) {
                        let signals: Vec<String> = dbc_msg
                            .signals
                            .values()
                            .map(|s| {
                                format!(
                                    "{}: {:.2}{}",
                                    s.name,
                                    s.decode(&m.data),
                                    if s.unit.is_empty() {
                                        "".to_string()
                                    } else {
                                        format!(" {}", s.unit)
                                    }
                                )
                            })
                            .collect();
                        format!("[Ch{}:{}] {}", m.channel, dbc_msg.name, signals.join(", "))
                    } else {
                        "".to_string()
                    }
                } else {
                    "".to_string()
                }
            },
        )),
        LogObject::LinMessage(m) => Some((
            m.header.object_time_stamp,
            m.channel,
            "LIN",
            format!("0x{:X}", m.id),
            m.dlc,
            m.data
                .iter()
                .take(m.dlc as usize)
                .map(|b| format!("{:02X}", b))
                .collect::<Vec<_>>()
                .join(" "),
            {
                if let Some(db) = ldf_channels.read().get(&m.channel) {
                    if let Some(frame) = db.frames.values().find(|f| f.id == m.id as u32) {
                        let signals: Vec<String> = frame
                            .signals
                            .iter()
                            .map(|mapping| {
                                if let Some(sig) = db.signals.get(&mapping.signal_name) {
                                    format!("{}: {}", sig.name, sig.decode(&m.data, mapping.offset))
                                } else {
                                    "".to_string()
                                }
                            })
                            .filter(|s| !s.is_empty())
                            .collect();
                        format!("[Ch{}:{}] {}", m.channel, frame.name, signals.join(", "))
                    } else {
                        "".to_string()
                    }
                } else {
                    "".to_string()
                }
            },
        )),
        _ => None,
    };

    if let Some((timestamp, ch, type_name, id, dlc, data_hex, decoded)) = res {
        let time_formatted = format!("{:.6}", timestamp as f64 / 1_000_000_000.0);
        rsx! {
            tr {
                style: "border-bottom: 1px solid #eee; &:hover {{ background: #f9f9f9; }}",
                td { style: "padding: 8px; color: #666; font-family: monospace;", "{time_formatted}" }
                td { style: "padding: 8px; font-weight: 500;", "{ch}" }
                td {
                    style: "padding: 8px;",
                    span {
                        style: format!("padding: 2px 6px; border-radius: 4px; font-size: 11px; font-weight: bold; background: {}; color: white;",
                            if type_name == "CAN" { "#0078d4" }
                            else if type_name == "CAN FD" || type_name == "CAN FD64" { "#6f42c1" }
                            else { "#28a745" }
                        ),
                        "{type_name}"
                    }
                }
                td { style: "padding: 8px; font-family: monospace; color: #0078d4;", "{id}" }
                td { style: "padding: 8px;", "{dlc}" }
                td { style: "padding: 8px; font-family: monospace;", "{data_hex}" }
                td { style: "padding: 8px; color: #666; font-style: italic;", "{decoded}" }
            }
        }
    } else {
        rsx! {}
    }
}
#[component]
fn ConfigView(
    app_config: Signal<AppConfig>,
    status_msg: Signal<String>,
    dbc_channels: Signal<HashMap<u16, DbcDatabase>>,
    ldf_channels: Signal<HashMap<u16, LdfDatabase>>,
    config_dir: Signal<Option<std::path::PathBuf>>,
    mut config_file_path: Signal<Option<std::path::PathBuf>>,
) -> Element {
    let mut selected_lib = use_signal(|| None::<usize>);
    let mut selected_version = use_signal(|| None::<usize>);
    let mut show_add_lib = use_signal(|| false);
    let mut new_lib_name = use_signal(|| String::new());
    let mut delete_dialog = use_signal(|| None::<DeleteAction>);

    rsx! {
        div {
            style: "display: flex; height: 100%; background: #f5f5f5;",

            // Sidebar: Libraries
            div {
                style: "width: 250px; border-right: 1px solid #ddd; background: white; padding: 20px; display: flex; flex-direction: column;",

                div {
                    style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 10px;",
                    h3 { style: "margin: 0; font-size: 16px; color: #333;", "Signal Libraries" }
                    button {
                        onclick: move |_| {
                            show_add_lib.set(true);
                        },
                        style: "padding: 2px 8px; background: #0078d4; color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 12px;",
                        "+ Add"
                    }
                }
                div {
                    style: "flex: 1; overflow-y: auto;",
                    for (l_idx, lib) in app_config.read().libraries.iter().enumerate() {
                        div {
                            key: "{lib.id}",
                            style: format!("margin-bottom: 8px; border-radius: 6px; overflow: hidden; border: 1px solid {}; background: {};",
                                if selected_lib() == Some(l_idx) && selected_version().is_none() { "#0078d4" } else { "#eee" },
                                if selected_lib() == Some(l_idx) && selected_version().is_none() { "#f0f7ff" } else { "white" }
                            ),
                            div {
                                style: "padding: 8px; display: flex; justify-content: space-between; align-items: center; cursor: pointer;",
                                onclick: move |_| {
                                    selected_lib.set(Some(l_idx));
                                    selected_version.set(None);
                                },
                                span { style: "font-weight: 600; font-size: 13px;", "{lib.name}" }
                                div {
                                    style: "display: flex; gap: 4px;",
                                    button {
                                        onclick: move |e| {
                                            e.stop_propagation();
                                            let mut config = app_config.write();
                                            let v_num = config.libraries[l_idx].versions.len() + 1;
                                            config.libraries[l_idx].versions.push(LibraryVersion {
                                                name: format!("V{}", v_num),
                                                date: chrono::Local::now().format("%Y-%m-%d").to_string(),
                                            });
                                        },
                                        style: "color: #28a745; background: none; border: none; cursor: pointer; font-size: 14px; padding: 0 4px;",
                                        title: "Add Version",
                                        "+"
                                    }
                                    button {
                                        onclick: move |e| {
                                            e.stop_propagation();
                                            delete_dialog.set(Some(DeleteAction::Library(l_idx)));
                                        },
                                        style: "color: #dc3545; background: none; border: none; cursor: pointer; font-size: 14px; padding: 0 4px;",
                                        "×"
                                    }
                                }
                            }
                            // Versions list
                            div {
                                style: "background: #fcfcfc; border-top: 1px solid #eee;",
                                for (v_idx, v) in lib.versions.iter().enumerate() {
                                    div {
                                        key: "{v.name}",
                                        onclick: move |_| {
                                            selected_lib.set(Some(l_idx));
                                            selected_version.set(Some(v_idx));
                                        },
                                        style: format!("padding: 6px 6px 6px 20px; font-size: 12px; cursor: pointer; border-left: 3px solid {}; background: {};",
                                            if selected_lib() == Some(l_idx) && selected_version() == Some(v_idx) { "#0078d4" } else { "transparent" },
                                            if selected_lib() == Some(l_idx) && selected_version() == Some(v_idx) { "#eef6ff" } else { "transparent" }
                                        ),
                                        "{v.name}"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Main area
            div {
                style: "flex: 1; padding: 30px; overflow-y: auto;",

                {
                    match (selected_lib(), selected_version()) {
                        (Some(lib_idx), Some(v_idx)) => {
                            let config = app_config.read();
                            if let (Some(lib), Some(v)) = (config.libraries.get(lib_idx), config.libraries.get(lib_idx).and_then(|l| l.versions.get(v_idx))) {
                                let v = v.clone();
                                let lib_id = lib.id.clone();
                                let v_name = v.name.clone();

                                rsx! {
                                    div {
                                        div {
                                            style: "display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 25px;",
                                            div {
                                                h2 { style: "margin: 0; color: #333;", "{lib.name} - {v.name}" }
                                                p { style: "color: #666; font-size: 14px; margin: 5px 0 0 0;", "Configure signal definition file and channel assignments for this version." }
                                            }
                                            {
                                                let is_active = config.active_library_id.as_ref() == Some(&lib_id) && config.active_version_name.as_ref() == Some(&v_name);
                                                let lib_id = lib_id.clone();
                                                let v_name = v_name.clone();
                                                rsx! {
                                                    button {
                                                        onclick: move |_| {
                                                            // SCOPED WRITE LOCK TO AVOID E0503
                                                            {
                                                                let mut config = app_config.write();
                                                                config.active_library_id = Some(lib_id.clone());
                                                                config.active_version_name = Some(v_name.clone());
                                                            }

                                                            let l_id = lib_id.clone();
                                                            let v_name = v_name.clone();

                                                            spawn(async move {
                                                                let mut dbc_write = dbc_channels.write();
                                                                let mut ldf_write = ldf_channels.write();
                                                                dbc_write.clear();
                                                                ldf_write.clear();

                                                                let config = app_config.read();
                                                                let dir = config_dir();

                                                                for mapping in config.mappings.iter() {
                                                                    if mapping.library_id.as_ref() == Some(&l_id) && mapping.version_name.as_ref() == Some(&v_name) {
                                                                        if mapping.path.is_empty() {
                                                                            continue;
                                                                        }
                                                                        let full_path = if let Some(dir) = &dir {
                                                                            dir.join(&mapping.path)
                                                                        } else {
                                                                            std::path::PathBuf::from(&mapping.path)
                                                                        };
                                                                        if let Ok(content) = std::fs::read_to_string(&full_path) {
                                                                            match mapping.channel_type {
                                                                                ChannelType::CAN => {
                                                                                    let parser = DbcParser::new();
                                                                                    if let Ok(db) = parser.parse(&content) {
                                                                                        dbc_write.insert(mapping.channel_id, db);
                                                                                    }
                                                                                }
                                                                                ChannelType::LIN => {
                                                                                    let parser = LdfParser::new();
                                                                                    if let Ok(db) = parser.parse(&content) {
                                                                                        ldf_write.insert(mapping.channel_id, db);
                                                                                    }
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                                status_msg.set(format!("Applied Version: {}", v_name));
                                                            });
                                                        },
                                                        disabled: is_active,
                                                        style: format!("padding: 10px 20px; border-radius: 6px; border: none; font-weight: 600; cursor: pointer; transition: all 0.2s; {}",
                                                            if is_active { "background: #28a745; color: white; cursor: default;" } else { "background: #0078d4; color: white;" }
                                                        ),
                                                        if is_active { "✓ Active Decoding Version" } else { "Set as Active Decoding Version" }
                                                    }
                                                }
                                            }
                                        }

                                        // 1. Channel Assignments Section
                                        div {
                                            style: "background: white; padding: 20px; border-radius: 8px; box-shadow: 0 2px 8px rgba(0,0,0,0.05); border: 1px solid #eee;",
                                            div {
                                                style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px;",
                                                h3 { style: "margin: 0; font-size: 14px; color: #555; text-transform: uppercase; letter-spacing: 0.5px;", "Channel & File Mappings" }
                                                {
                                                    let lib_id = lib_id.clone();
                                                    let v_name = v_name.clone();
                                                    rsx! {
                                                        button {
                                                            onclick: move |_| {
                                                                {
                                                                    let new_channel_id = {
                                                                        let config = app_config.read();
                                                                        config.mappings.len() as u16 + 1
                                                                    };
                                                                    app_config.write().mappings.push(ChannelMapping {
                                                                        channel_type: ChannelType::CAN,
                                                                        channel_id: new_channel_id,
                                                                        path: String::new(),
                                                                        description: String::new(),
                                                                        library_id: Some(lib_id.clone()),
                                                                        version_name: Some(v_name.clone()),
                                                                    });
                                                                }
                                                            },
                                                            style: "padding: 8px 16px; background: #28a745; color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 13px; font-weight: 500;",
                                                            "+ Add Mapping"
                                                        }
                                                    }
                                                }
                                            }

                                            table {
                                                style: "width: 100%; border-collapse: collapse;",
                                                thead {
                                                    tr {
                                                        style: "border-bottom: 1px solid #eee; color: #888; font-size: 12px;",
                                                        th { style: "padding: 8px; text-align: left;", "Protocol" }
                                                        th { style: "padding: 8px; text-align: left;", "Ch ID" }
                                                        th { style: "padding: 8px; text-align: left;", "Signal Definition File" }
                                                        th { style: "padding: 8px; text-align: left;", "Description" }
                                                        th { style: "padding: 8px; text-align: center;", "Actions" }
                                                    }
                                                }
                                                tbody {
                                                    for (m_idx, mapping) in app_config.read().mappings.iter().enumerate() {
                                                        if mapping.library_id.as_ref() == Some(&lib_id) && mapping.version_name.as_ref() == Some(&v_name) {
                                                            tr {
                                                                key: "{m_idx}",
                                                                style: "border-bottom: 1px solid #f8f9fa;",
                                                                td { style: "padding: 8px;",
                                                                    select {
                                                                        style: "padding: 4px; border: 1px solid #ddd; border-radius: 4px; background: white; font-size: 12px;",
                                                                        onchange: move |e| {
                                                                            let val = match e.value().as_str() {
                                                                                "CAN" => ChannelType::CAN,
                                                                                "LIN" => ChannelType::LIN,
                                                                                _ => ChannelType::CAN,
                                                                            };
                                                                            app_config.write().mappings[m_idx].channel_type = val;
                                                                        },
                                                                        option { value: "CAN", selected: mapping.channel_type == ChannelType::CAN, "CAN" }
                                                                        option { value: "LIN", selected: mapping.channel_type == ChannelType::LIN, "LIN" }
                                                                    }
                                                                }
                                                                td { style: "padding: 8px;",
                                                                    input {
                                                                        style: "width: 50px; padding: 4px; border: 1px solid #ddd; border-radius: 4px; font-size: 12px;",
                                                                        r#type: "number",
                                                                        value: "{mapping.channel_id}",
                                                                        oninput: move |e| {
                                                                            if let Ok(ch) = e.value().parse::<u16>() {
                                                                                app_config.write().mappings[m_idx].channel_id = ch;
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                                td { style: "padding: 8px; display: flex; gap: 6px; align-items: center;",
                                                                    div {
                                                                        style: "flex: 1; padding: 4px 8px; background: #f8f9fa; border: 1px solid #ddd; border-radius: 4px; font-family: monospace; font-size: 11px; max-width: 250px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                                                                        if mapping.path.is_empty() { "No file" } else { "{mapping.path}" }
                                                                    }
                                                                    button {
                                                                        onclick: move |_| {
                                                                            let p_type = app_config.read().mappings[m_idx].channel_type;
                                                                            spawn(async move {
                                                                                let mut dialog = rfd::FileDialog::new();
                                                                                match p_type {
                                                                                    ChannelType::CAN => dialog = dialog.add_filter("DBC Database", &["dbc"]),
                                                                                    ChannelType::LIN => dialog = dialog.add_filter("LDF Database", &["ldf"]),
                                                                                }
                                                                                if let Some(p) = dialog.pick_file() {
                                                                                    app_config.write().mappings[m_idx].path = p.to_string_lossy().to_string();
                                                                                }
                                                                            });
                                                                        },
                                                                        style: "padding: 4px 8px; background: #0078d4; color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 10px;",
                                                                        "Browse"
                                                                    }
                                                                }
                                                                td { style: "padding: 8px;",
                                                                    input {
                                                                        style: "width: 100%; padding: 4px; border: 1px solid #ddd; border-radius: 4px; font-size: 12px;",
                                                                        value: "{mapping.description}",
                                                                        oninput: move |e| {
                                                                            app_config.write().mappings[m_idx].description = e.value();
                                                                        }
                                                                    }
                                                                }
                                                                td { style: "padding: 8px; text-align: center;",
                                                                    button {
                                                                        onclick: move |_| {
                                                                            delete_dialog.set(Some(DeleteAction::Mapping(m_idx)));
                                                                        },
                                                                        style: "color: #dc3545; background: none; border: none; cursor: pointer; font-size: 16px;",
                                                                        "×"
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }

                                        // 3. Apply Actions
                                        div {
                                            style: "margin-top: 40px; display: flex; align-items: center; gap: 20px;",
                                            button {
                                                onclick: move |_| {
                                                    spawn(async move {
                                                        // 1. Apply Logic (duplicated for now to avoid closure issues)
                                                        let config = app_config.read();
                                                        if let (Some(l_id), Some(v_name)) = (&config.active_library_id, &config.active_version_name) {
                                                                let mut dbc_write = dbc_channels.write();
                                                                let mut ldf_write = ldf_channels.write();
                                                                dbc_write.clear();
                                                                ldf_write.clear();

                                                                let dir = config_dir();

                                                                for mapping in config.mappings.iter() {
                                                                    // Clone l_id and v_name here if needed, or just iterate
                                                                    if mapping.library_id.as_ref() == Some(l_id) && mapping.version_name.as_ref() == Some(v_name) {
                                                                        if mapping.path.is_empty() {
                                                                            continue;
                                                                        }
                                                                        let full_path = if let Some(dir) = &dir {
                                                                            dir.join(&mapping.path)
                                                                        } else {
                                                                            std::path::PathBuf::from(&mapping.path)
                                                                        };
                                                                        if let Ok(content) = std::fs::read_to_string(&full_path) {
                                                                            match mapping.channel_type {
                                                                                ChannelType::CAN => {
                                                                                    let parser = DbcParser::new();
                                                                                    if let Ok(db) = parser.parse(&content) {
                                                                                        dbc_write.insert(mapping.channel_id, db);
                                                                                    }
                                                                                }
                                                                                ChannelType::LIN => {
                                                                                    let parser = LdfParser::new();
                                                                                    if let Ok(db) = parser.parse(&content) {
                                                                                        ldf_write.insert(mapping.channel_id, db);
                                                                                    }
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                                status_msg.set(format!("Applied Version: {}", v_name));
                                                        }

                                                        // 2. Save Logic
                                                        let config_clone = app_config.read().clone();
                                                        let path_to_save = if let Some(p) = config_file_path() {
                                                            p
                                                        } else {
                                                            std::path::PathBuf::from("multi_channel_config.json")
                                                        };

                                                        if let Ok(json) = serde_json::to_string_pretty(&config_clone) {
                                                            match std::fs::write(&path_to_save, json) {
                                                                Ok(_) => {
                                                                    status_msg.set(format!("Configuration Saved to {:?}", path_to_save));
                                                                    if config_file_path().is_none() {
                                                                        config_file_path.set(Some(path_to_save));
                                                                    }
                                                                },
                                                                Err(e) => {
                                                                    status_msg.set(format!("Error saving config: {}", e));
                                                                    // Fallback to dialog if save fails (optional, but good for UX)
                                                                    if let Some(path) = rfd::FileDialog::new().add_filter("JSON", &["json"]).save_file() {
                                                                         if let Ok(json) = serde_json::to_string_pretty(&config_clone) {
                                                                             let _ = std::fs::write(&path, json);
                                                                             status_msg.set("Configuration Applied and Saved".to_string());
                                                                         }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    });
                                                },
                                                style: "padding: 12px 24px; background: #0078d4; color: white; border: none; border-radius: 6px; cursor: pointer; font-weight: bold; font-size: 16px;",
                                                "Apply & Save"
                                            }
                                            span { style: "color: #888; font-size: 12px; font-style: italic;", "Applying will reload all mapped signal databases." }
                                        }
                                    }
                                }
                            } else {
                                rsx! { div { "Workspace Error: Selection mismatch" } }
                            }
                        }
                        _ => {
                            let config = app_config.read();
                            rsx! {
                                div {
                                    style: "height: 100%; display: flex; flex-direction: column; align-items: center; justify-content: center; color: #888; text-align: center;",
                                    h2 { style: "margin-bottom: 10px; color: #555;", "Signal Configuration Summary" }
                                    p { style: "max-width: 400px; margin-bottom: 30px;", "Select a Library Version in the sidebar to configure its file and channel mappings." }

                                    div {
                                        style: "width: 100%; max-width: 800px; background: white; padding: 20px; border-radius: 8px; border: 1px solid #eee; box-shadow: 0 2px 4px rgba(0,0,0,0.05);",
                                        h3 { style: "margin-top: 0; font-size: 14px; text-transform: uppercase; text-align: left;", "Current Mapping Status" }
                                        if config.mappings.is_empty() {
                                            p { style: "font-style: italic; color: #aaa; padding: 20px;", "No channels configured yet." }
                                        } else {
                                            table {
                                                style: "width: 100%; border-collapse: collapse; text-align: left;",
                                                thead {
                                                    tr {
                                                        style: "border-bottom: 1px solid #eee; color: #888; font-size: 12px;",
                                                        th { style: "padding: 10px;", "Channel" }
                                                        th { style: "padding: 10px;", "Library / Version" }
                                                        th { style: "padding: 10px;", "Signal File" }
                                                    }
                                                }
                                                tbody {
                                                    for m in config.mappings.iter() {
                                                        tr {
                                                            style: "border-bottom: 1px solid #f9f9f9;",
                                                            td { style: "padding: 10px; font-size: 13px;", "{m.channel_type:?} {m.channel_id}" }
                                                            td { style: "padding: 10px; font-size: 13px;",
                                                                {
                                                                    if let (Some(l_id), Some(v_name)) = (&m.library_id, &m.version_name) {
                                                                        let lib_name = config.libraries.iter().find(|l| l.id == *l_id).map(|l| &l.name).unwrap_or(l_id);
                                                                        rsx! { "{lib_name} / {v_name}" }
                                                                    } else {
                                                                        rsx! { span { style: "color: #dc3545;", "Unassigned" } }
                                                                    }
                                                                }
                                                            }
                                                            td { style: "padding: 10px; font-family: monospace; font-size: 11px; color: #666;",
                                                                if m.path.is_empty() {
                                                                    span { style: "color: #dc3545;", "Missing File" }
                                                                } else {
                                                                    "{m.path}"
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if show_add_lib() {
                div {
                    style: "position: fixed; top: 0; left: 0; width: 100%; height: 100%; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 1000;",
                    div {
                        style: "background: white; padding: 30px; border-radius: 8px; width: 400px;",
                        h3 { "Add New Library" }
                        input {
                            style: "width: 100%; padding: 10px; margin: 10px 0; border: 1px solid #ddd; border-radius: 4px;",
                            placeholder: "Library Name",
                            value: "{new_lib_name}",
                            oninput: move |e| new_lib_name.set(e.value().clone())
                        }
                        div {
                            style: "display: flex; gap: 10px; justify-content: flex-end;",
                            button {
                                onclick: move |_| show_add_lib.set(false),
                                style: "padding: 8px 16px; background: #6c757d; color: white; border: none; border-radius: 4px; cursor: pointer;",
                                "Cancel"
                            }
                            button {
                                onclick: move |_| {
                                    let name = new_lib_name();
                                    if !name.is_empty() {
                                        let mut config = app_config.write();
                                        let libs = &mut config.libraries;
                                        if libs.iter().any(|l| l.name == name) {
                                            status_msg.set(format!("Library '{}' already exists in this category", name));
                                            return;
                                        }
                                        libs.push(SignalLibrary {
                                            id: name.to_lowercase().replace(" ", "_"),
                                            name,
                                            versions: vec![],
                                        });
                                        show_add_lib.set(false);
                                        new_lib_name.set(String::new());
                                    }
                                },
                                style: "padding: 8px 16px; background: #0078d4; color: white; border: none; border-radius: 4px; cursor: pointer;",
                                "Add"
                            }
                        }
                    }
                }
            }

            if let Some(action) = delete_dialog() {
                div {
                    style: "position: fixed; top: 0; left: 0; width: 100%; height: 100%; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 1001;",
                    div {
                        style: "background: white; padding: 30px; border-radius: 8px; width: 400px; box-shadow: 0 4px 12px rgba(0,0,0,0.15);",
                        h3 { style: "margin-top: 0; color: #dc3545;", "Confirm Deletion" }
                        p {
                            match action {
                                DeleteAction::Library(_) => "Are you sure you want to delete this library and all its versions?",
                                DeleteAction::Mapping(_) => "Are you sure you want to remove this mapping?",
                            }
                        }
                        div {
                            style: "display: flex; gap: 10px; justify-content: flex-end; margin-top: 25px;",
                            button {
                                onclick: move |_| delete_dialog.set(None),
                                style: "padding: 8px 16px; background: #6c757d; color: white; border: none; border-radius: 4px; cursor: pointer;",
                                "Cancel"
                            }
                            button {
                                onclick: move |_| {
                                    let mut config = app_config.write();
                                    match action {
                                        DeleteAction::Library(idx) => {
                                            let libs = &mut config.libraries;
                                            libs.remove(idx);
                                            if selected_lib() == Some(idx) {
                                                selected_lib.set(None);
                                            } else if let Some(sel_idx) = selected_lib() {
                                                if sel_idx > idx {
                                                    selected_lib.set(Some(sel_idx - 1));
                                                }
                                            }
                                        }

                                        DeleteAction::Mapping(m_idx) => {
                                            config.mappings.remove(m_idx);
                                        }
                                    }
                                    delete_dialog.set(None);
                                },
                                style: "padding: 8px 16px; background: #dc3545; color: white; border: none; border-radius: 4px; cursor: pointer;",
                                "Delete"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ChartView(
    dbc_channels: Signal<HashMap<u16, DbcDatabase>>,
    ldf_channels: Signal<HashMap<u16, LdfDatabase>>,
    messages: Signal<Vec<LogObject>>,
    mut selected_signals: Signal<Vec<String>>,
    start_time: Signal<Option<chrono::NaiveDateTime>>,
) -> Element {
    let mut svg_data = use_signal(|| String::new());

    // Effect to regenerate chart when signals or messages change
    use_effect(move || {
        let signals = selected_signals.read();
        if signals.is_empty() {
            svg_data.set(String::new());
            return;
        }

        let msgs = messages.read();
        let dbcs = dbc_channels.read();
        let start_time_val = *start_time.read();

        use chrono::{Duration, NaiveDateTime};
        use plotters::prelude::*;

        let mut buffer = String::new();

        if let Some(t0) = start_time_val {
            // Absolute Time Plotting
            let mut plot_data: HashMap<String, Vec<(NaiveDateTime, f64)>> = HashMap::new();
            for sig_key in signals.iter() {
                plot_data.insert(sig_key.clone(), Vec::new());
            }

            let mut min_t = t0;
            let mut max_t = t0;
            let mut min_y = f64::MAX;
            let mut max_y = f64::MIN;
            let mut has_data = false;

            for msg in msgs.iter() {
                let (channel, id, data, timestamp_ns) = match msg {
                    LogObject::CanMessage(m) => {
                        (m.channel, m.id, &m.data[..], m.header.object_time_stamp)
                    }
                    LogObject::CanMessage2(m) => {
                        (m.channel, m.id, &m.data[..], m.header.object_time_stamp)
                    }
                    LogObject::CanFdMessage(m) => {
                        (m.channel, m.id, &m.data[..], m.header.object_time_stamp)
                    }
                    _ => continue,
                };

                if let Some(db) = dbcs.get(&channel) {
                    if let Some(dbc_msg) = db.messages.get(&id) {
                        for sig in dbc_msg.signals.values() {
                            let key = format!("{}:{}:{}", channel, id, sig.name);
                            if let Some(data_vec) = plot_data.get_mut(&key) {
                                let time = t0 + Duration::nanoseconds(timestamp_ns as i64);
                                let value = sig.decode(data);
                                data_vec.push((time, value));

                                if !has_data {
                                    min_t = time;
                                    max_t = time;
                                } else {
                                    if time < min_t {
                                        min_t = time;
                                    }
                                    if time > max_t {
                                        max_t = time;
                                    }
                                }
                                if value < min_y {
                                    min_y = value;
                                }
                                if value > max_y {
                                    max_y = value;
                                }
                                has_data = true;
                            }
                        }
                    }
                }
            }

            if has_data {
                // Add padding to Y
                let y_padding = (max_y - min_y) * 0.1;
                if y_padding == 0.0 {
                    min_y -= 1.0;
                    max_y += 1.0;
                } else {
                    min_y -= y_padding;
                    max_y += y_padding;
                }

                // Add padding to X (Time) approx
                if min_t == max_t {
                    max_t = min_t + Duration::seconds(1);
                }

                // Convert NaiveDateTime to timestamps for plotting
                let min_t_ts = min_t.and_utc().timestamp() as f64
                    + min_t.and_utc().timestamp_subsec_nanos() as f64 / 1_000_000_000.0;
                let max_t_ts = max_t.and_utc().timestamp() as f64
                    + max_t.and_utc().timestamp_subsec_nanos() as f64 / 1_000_000_000.0;

                // Convert plot data to timestamp-value pairs
                let mut plot_data_ts: HashMap<String, Vec<(f64, f64)>> = HashMap::new();
                for (key, points) in plot_data.iter() {
                    let converted_points: Vec<(f64, f64)> = points
                        .iter()
                        .map(|(dt, y)| {
                            (
                                dt.and_utc().timestamp() as f64
                                    + dt.and_utc().timestamp_subsec_nanos() as f64
                                        / 1_000_000_000.0,
                                *y,
                            )
                        })
                        .collect();
                    plot_data_ts.insert(key.clone(), converted_points);
                }

                let root = SVGBackend::with_string(&mut buffer, (800, 600)).into_drawing_area();
                root.fill(&WHITE).unwrap();

                let mut chart = ChartBuilder::on(&root)
                    .caption(
                        format!("Signal Plot (Start: {})", t0.format("%Y-%m-%d %H:%M:%S")),
                        ("sans-serif", 20).into_font(),
                    )
                    .margin(10)
                    .x_label_area_size(40)
                    .y_label_area_size(50)
                    .build_cartesian_2d(min_t_ts..max_t_ts, min_y..max_y)
                    .unwrap();

                chart.configure_mesh().x_labels(5).draw().unwrap();

                let colors = [RED, BLUE, GREEN, MAGENTA, CYAN];
                for (i, (key, points)) in plot_data_ts.iter().enumerate() {
                    let color = colors[i % colors.len()];
                    chart
                        .draw_series(LineSeries::new(points.clone(), color.stroke_width(2)))
                        .unwrap()
                        .label(key)
                        .legend(move |(x, y)| {
                            PathElement::new(vec![(x, y), (x + 20, y)], color.filled())
                        });
                }
                chart
                    .configure_series_labels()
                    .background_style(&WHITE.mix(0.8))
                    .border_style(&BLACK)
                    .draw()
                    .unwrap();
            } else {
                buffer = "<svg><text x='10' y='20'>No Data</text></svg>".to_string();
            }
        } else {
            // Relative Time Plotting (Fallback)
            let mut plot_data: HashMap<String, Vec<(f64, f64)>> = HashMap::new();
            for sig_key in signals.iter() {
                plot_data.insert(sig_key.clone(), Vec::new());
            }

            let mut min_x = f64::MAX;
            let mut max_x = f64::MIN;
            let mut min_y = f64::MAX;
            let mut max_y = f64::MIN;
            let mut has_data = false;

            for msg in msgs.iter() {
                let (channel, id, data, timestamp_ns) = match msg {
                    LogObject::CanMessage(m) => {
                        (m.channel, m.id, &m.data[..], m.header.object_time_stamp)
                    }
                    LogObject::CanMessage2(m) => {
                        (m.channel, m.id, &m.data[..], m.header.object_time_stamp)
                    }
                    LogObject::CanFdMessage(m) => {
                        (m.channel, m.id, &m.data[..], m.header.object_time_stamp)
                    }
                    _ => continue,
                };

                if let Some(db) = dbcs.get(&channel) {
                    if let Some(dbc_msg) = db.messages.get(&id) {
                        for sig in dbc_msg.signals.values() {
                            let key = format!("{}:{}:{}", channel, id, sig.name);
                            if let Some(data_vec) = plot_data.get_mut(&key) {
                                let time = timestamp_ns as f64 / 1_000_000_000.0;
                                let value = sig.decode(data);
                                data_vec.push((time, value));

                                has_data = true;
                                if time < min_x {
                                    min_x = time;
                                }
                                if time > max_x {
                                    max_x = time;
                                }
                                if value < min_y {
                                    min_y = value;
                                }
                                if value > max_y {
                                    max_y = value;
                                }
                            }
                        }
                    }
                }
            }

            if has_data {
                let y_padding = (max_y - min_y) * 0.1;
                if y_padding == 0.0 {
                    min_y -= 1.0;
                    max_y += 1.0;
                } else {
                    min_y -= y_padding;
                    max_y += y_padding;
                }

                let root = SVGBackend::with_string(&mut buffer, (800, 600)).into_drawing_area();
                root.fill(&WHITE).unwrap();

                let mut chart = ChartBuilder::on(&root)
                    .caption(
                        "Signal Plot (Relative Time)",
                        ("sans-serif", 30).into_font(),
                    )
                    .margin(10)
                    .x_label_area_size(40)
                    .y_label_area_size(50)
                    .build_cartesian_2d(min_x..max_x, min_y..max_y)
                    .unwrap();

                chart.configure_mesh().draw().unwrap();

                let colors = [RED, BLUE, GREEN, MAGENTA, CYAN];
                for (i, (key, points)) in plot_data.iter().enumerate() {
                    let color = colors[i % colors.len()];
                    chart
                        .draw_series(LineSeries::new(points.clone(), color.stroke_width(2)))
                        .unwrap()
                        .label(key)
                        .legend(move |(x, y)| {
                            PathElement::new(vec![(x, y), (x + 20, y)], color.filled())
                        });
                }
                chart
                    .configure_series_labels()
                    .background_style(&WHITE.mix(0.8))
                    .border_style(&BLACK)
                    .draw()
                    .unwrap();
            } else {
                buffer = "<svg><text x='10' y='20'>No Data</text></svg>".to_string();
            }
        }

        use base64::prelude::*;
        let encoded = BASE64_STANDARD.encode(buffer);
        svg_data.set(format!("data:image/svg+xml;base64,{}", encoded));
    });

    let channels_data = dbc_channels.read().clone();
    let mut ui_channels = channels_data
        .into_iter()
        .map(|(id, db)| {
            let mut msgs: Vec<_> = db
                .messages
                .values()
                .map(|m| {
                    let mut sigs: Vec<_> = m
                        .signals
                        .values()
                        .map(|s| (s.name.clone(), format!("{}:{}:{}", id, m.id, s.name)))
                        .collect();
                    sigs.sort_by(|a, b| a.0.cmp(&b.0));
                    (m.name.clone(), m.id, sigs)
                })
                .collect();
            msgs.sort_by(|a, b| a.0.cmp(&b.0));
            (id, msgs)
        })
        .collect::<Vec<_>>();
    ui_channels.sort_by_key(|(id, _)| *id);

    rsx! {
        div {
            style: "display: flex; height: 100%;",
            // Signal Selector Sidebar
            div {
                style: "width: 300px; border-right: 1px solid #ddd; padding: 10px; overflow-y: auto; background: #f9f9f9;",
                h3 { "Available Signals" }
                // Iterate DBC Channels
                for (ch_id, msgs) in ui_channels {
                    div {
                        strong { "Channel {ch_id} (DBC)" }
                        for (msg_name, msg_id, sigs) in msgs {
                            details {
                                summary { style: "cursor: pointer; font-size: 13px;", "{msg_name} (0x{msg_id:X})" }
                                div { style: "padding-left: 15px;",
                                    for (sig_name, key) in sigs {
                                        div {
                                            style: "display: flex; align-items: center; font-size: 12px; margin: 2px 0;",
                                            input {
                                                r#type: "checkbox",
                                                checked: selected_signals.read().contains(&key),
                                                onchange: move |e| {
                                                    let k = key.clone();
                                                    let mut s = selected_signals.write();
                                                    if e.value() == "true" {
                                                        if !s.contains(&k) {
                                                            s.push(k);
                                                        }
                                                    } else {
                                                        s.retain(|x| x != &k);
                                                    }
                                                }
                                            }
                                            span { style: "margin-left: 5px;", "{sig_name}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Chart Area
            div {
                style: "flex: 1; padding: 20px; display: flex; flex-direction: column; align-items: center; justify-content: center;",
                if !svg_data.read().is_empty() {
                    img {
                        src: "{svg_data}",
                        style: "max-width: 100%; max-height: 100%; box-shadow: 0 4px 8px rgba(0,0,0,0.1); border: 1px solid #eee;"
                    }
                } else {
                    p { style: "color: #999;", "Select signals from the sidebar to plot." }
                }
            }
        }
    }
}
