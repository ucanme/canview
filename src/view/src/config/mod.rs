//! Configuration management for the CanView application

use crate::models::AppConfig;
use gpui::*;
use std::path::PathBuf;

/// Configuration manager trait
pub trait ConfigManager {
    /// Load configuration from a file
    fn load_config(&mut self, cx: &mut Context<Self>);

    /// Save configuration to file
    fn save_config(&self, cx: &mut Context<Self>);

    /// Load startup configuration
    fn load_startup_config(&mut self);

    /// Import a database file
    fn import_database_file(&mut self, cx: &mut Context<Self>);
}

/// Default configuration file name
pub const DEFAULT_CONFIG_FILE: &str = "multi_channel_config.json";

/// Load configuration from a specific path
pub fn load_config_from_path(path: PathBuf) -> Result<AppConfig, String> {
    if !path.exists() {
        return Err("Configuration file not found".to_string());
    }

    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read config file: {}", e))?;

    let config: AppConfig = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse config: {}", e))?;

    Ok(config)
}

/// Save configuration to a specific path
pub fn save_config_to_path(config: &AppConfig, path: &PathBuf) -> Result<(), String> {
    let content = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    std::fs::write(path, content)
        .map_err(|e| format!("Failed to write config file: {}", e))?;

    Ok(())
}
