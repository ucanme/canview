//! Data models for the CanView application

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
}

fn default_channel_type() -> ChannelType {
    ChannelType::CAN
}

/// Application configuration
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct AppConfig {
    pub mappings: Vec<ChannelMapping>,
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
