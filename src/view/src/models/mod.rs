//! Data models for the CanView application

pub mod library;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Re-export library types
pub use library::{SignalLibrary, LibraryVersion, DatabaseType};

/// Channel type enumeration
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Copy)]
pub enum ChannelType {
    CAN,
    LIN,
}

impl ChannelType {
    pub fn is_can(&self) -> bool {
        matches!(self, ChannelType::CAN)
    }

    pub fn is_lin(&self) -> bool {
        matches!(self, ChannelType::LIN)
    }
}

/// Channel mapping configuration
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ChannelMapping {
    #[serde(default = "default_channel_type")]
    pub channel_type: ChannelType,
    #[serde(alias = "channel")]
    pub channel_id: u16,
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub description: String,
    /// 关联的信号库ID
    #[serde(default)]
    pub library_id: Option<String>,
    /// 激活的版本名称
    #[serde(default)]
    pub version_name: Option<String>,
}

fn default_channel_type() -> ChannelType {
    ChannelType::CAN
}

/// Application configuration
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct AppConfig {
    /// 信号库列表
    #[serde(default)]
    pub libraries: Vec<SignalLibrary>,
    /// 通道映射列表
    pub mappings: Vec<ChannelMapping>,
    /// 当前激活的库ID
    #[serde(default)]
    pub active_library_id: Option<String>,
    /// 当前激活的版本名称
    #[serde(default)]
    pub active_version_name: Option<String>,
}

/// Application view enumeration
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppView {
    LogView,
    ConfigView,
    ChartView,
}

/// State for tracking scrollbar drag operation
#[derive(Clone)]
pub struct ScrollbarDragState {
    pub start_y: Pixels,
    pub start_scroll_offset: f32,
    pub filtered_count: usize,
}
