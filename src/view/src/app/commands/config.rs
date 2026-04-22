//! Configuration commands
//!
//! Command handlers for configuration management.

use std::path::PathBuf;

/// Save application configuration
pub struct SaveConfig {
    pub config_dir: Option<PathBuf>,
    pub config_file: Option<PathBuf>,
}

impl SaveConfig {
    pub fn new(config_dir: Option<PathBuf>, config_file: Option<PathBuf>) -> Self {
        Self {
            config_dir,
            config_file,
        }
    }

    /// Check if configuration directory exists
    pub fn has_config_dir(&self) -> bool {
        self.config_dir.is_some()
    }

    /// Get the configuration file path
    pub fn config_file_path(&self) -> Option<&PathBuf> {
        self.config_file.as_ref()
    }

    /// Validate configuration setup
    pub fn is_valid(&self) -> bool {
        self.config_dir.is_some() && self.config_file.is_some()
    }
}

/// Load application configuration
pub struct LoadConfig {
    pub config_dir: Option<PathBuf>,
    pub config_file: Option<PathBuf>,
}

impl LoadConfig {
    pub fn new(config_dir: Option<PathBuf>, config_file: Option<PathBuf>) -> Self {
        Self {
            config_dir,
            config_file,
        }
    }

    /// Check if this is the first run (no config file exists)
    pub fn is_first_run(&self) -> bool {
        self.config_file.is_none() || 
            self.config_file.as_ref().map_or(true, |p| !p.exists())
    }
}

/// Load startup configuration
pub struct LoadStartupConfig;

impl LoadStartupConfig {
    pub fn new() -> Self {
        Self
    }
}
