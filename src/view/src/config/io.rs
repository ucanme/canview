//! Configuration file I/O operations
//!
//! Handles loading and saving configuration files.

use crate::AppConfig;
use std::path::PathBuf;

/// Load configuration from a file selected by the user
pub fn load_config_from_file() -> Option<(AppConfig, PathBuf, String)> {
    if let Some(path) = rfd::FileDialog::new()
        .add_filter("Config Files", &["json"])
        .pick_file()
    {
        if let Ok(content) = std::fs::read_to_string(&path) {
            match serde_json::from_str::<AppConfig>(&content) {
                Ok(config) => {
                    let status_msg = "Configuration loaded successfully".to_string();
                    return Some((config, path, status_msg));
                }
                Err(e) => {
                    let status_msg = format!("Config Error: {}", e);
                    return Some((AppConfig::default(), path, status_msg));
                }
            }
        }
    }
    None
}

/// Save configuration to a file
pub fn save_config_to_file(
    config: &AppConfig,
    default_path: Option<PathBuf>,
) -> Option<(PathBuf, String)> {
    let path = if let Some(existing_path) = default_path {
        existing_path
    } else {
        rfd::FileDialog::new()
            .add_filter("Config Files", &["json"])
            .save_file()?
    };

    match serde_json::to_string_pretty(config) {
        Ok(content) => {
            if std::fs::write(&path, content).is_ok() {
                let status_msg = "Configuration saved successfully".to_string();
                Some((path, status_msg))
            } else {
                Some((path, "Failed to write config file".to_string()))
            }
        }
        Err(e) => {
            let status_msg = format!("Failed to serialize config: {}", e);
            Some((path, status_msg))
        }
    }
}
