//! Startup configuration loading
//!
//! Handles loading configuration at application startup.

use std::path::PathBuf;
use crate::AppConfig;

/// Load startup configuration from the default config file
pub fn load_startup_config() -> (AppConfig, Option<PathBuf>, Option<PathBuf>, String) {
    let path = PathBuf::from("multi_channel_config.json");
    
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            match serde_json::from_str::<AppConfig>(&content) {
                Ok(config) => {
                    let config_dir = Some(
                        path.parent()
                            .unwrap_or(std::path::Path::new("../../../../.."))
                            .to_path_buf(),
                    );
                    let config_file_path = Some(path);
                    let status_msg = "Configuration loaded.".to_string();
                    
                    return (config, config_dir, config_file_path, status_msg);
                }
                Err(e) => {
                    let status_msg = format!("Config load error: {}. Using default config.", e);
                    return (AppConfig::default(), None, None, status_msg);
                }
            }
        }
    }
    
    // Default: no config file found
    let status_msg = "Ready - GPUI version initialized".to_string();
    (AppConfig::default(), None, None, status_msg)
}
