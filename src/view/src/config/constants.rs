//! Configuration constants and helper functions
//!
//! This module contains configuration-related constants and helper functions
//! for the CanView application.

use std::path::PathBuf;

/// Default configuration file name
pub const DEFAULT_CONFIG_FILE: &str = "multi_channel_config.json";

/// Get the default configuration file path
pub fn get_default_config_path() -> PathBuf {
    PathBuf::from(DEFAULT_CONFIG_FILE)
}

/// Format a configuration load error message
pub fn format_config_error(error: serde_json::Error) -> String {
    format!("Config load error: {}. Using default config.", error)
}

/// Format a configuration loaded success message
pub fn config_loaded_message() -> &'static str {
    "Configuration loaded."
}

/// Format a configuration not found message
pub fn config_not_found_message() -> &'static str {
    "Ready - GPUI version initialized"
}

/// Format a configuration found message
pub fn config_found_message() -> &'static str {
    "Found saved config, loading..."
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_default_config_path() {
        let path = get_default_config_path();
        assert_eq!(path, PathBuf::from(DEFAULT_CONFIG_FILE));
    }

    #[test]
    fn test_format_config_error() {
        let error = serde_json::from_str::<serde_json::Value>("invalid json")
            .unwrap_err();
        let msg = format_config_error(error);
        assert!(msg.contains("Config load error"));
    }

    #[test]
    fn test_message_constants() {
        assert_eq!(config_loaded_message(), "Configuration loaded.");
        assert_eq!(config_not_found_message(), "Ready - GPUI version initialized");
        assert_eq!(config_found_message(), "Found saved config, loading...");
    }
}
