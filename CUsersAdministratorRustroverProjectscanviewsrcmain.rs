use blf::{read_blf_from_file, LogObject};
use gpui::{prelude::*, *};
use gpui_component::{button::*, *};
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
