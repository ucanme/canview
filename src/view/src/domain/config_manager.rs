//! Configuration management for channels and signal libraries
//!
//! This module handles configuration loading, saving, and validation
//! without UI dependencies.

use crate::models::{AppConfig, ChannelMapping, ChannelType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Channel configuration with database mapping
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ChannelConfig {
    /// Channel number
    pub channel: u16,
    /// Channel name (e.g., "CAN1", "LIN2")
    pub name: String,
    /// Channel type (CAN or LIN)
    pub channel_type: ChannelType,
    /// Path to DBC or LDF database file
    pub database_path: Option<PathBuf>,
    /// Whether this channel is enabled
    pub enabled: bool,
}

impl ChannelConfig {
    /// Create a new channel configuration
    pub fn new(channel: u16, name: String, channel_type: ChannelType) -> Self {
        Self {
            channel,
            name,
            channel_type,
            database_path: None,
            enabled: true,
        }
    }

    /// Create a new CAN channel
    pub fn can(channel: u16, name: String) -> Self {
        Self::new(channel, name, ChannelType::CAN)
    }

    /// Create a new LIN channel
    pub fn lin(channel: u16, name: String) -> Self {
        Self::new(channel, name, ChannelType::LIN)
    }

    /// Set the database path
    pub fn with_database(mut self, path: PathBuf) -> Self {
        self.database_path = Some(path);
        self
    }

    /// Enable the channel
    pub fn enable(mut self) -> Self {
        self.enabled = true;
        self
    }

    /// Disable the channel
    pub fn disable(mut self) -> Self {
        self.enabled = false;
        self
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.name.is_empty() {
            return Err(ConfigError::InvalidChannelName(
                "Channel name cannot be empty".to_string(),
            ));
        }

        // Validate database path if set
        if let Some(db_path) = &self.database_path {
            if !db_path.exists() {
                return Err(ConfigError::DatabaseNotFound(db_path.clone()));
            }

            // Validate file extension matches channel type
            let ext = db_path.extension().and_then(|e| e.to_str()).unwrap_or("");

            match self.channel_type {
                ChannelType::CAN => {
                    if !ext.eq_ignore_ascii_case("dbc") {
                        return Err(ConfigError::InvalidDatabaseType {
                            expected: "dbc".to_string(),
                            found: ext.to_string(),
                        });
                    }
                }
                ChannelType::LIN => {
                    if !ext.eq_ignore_ascii_case("ldf") {
                        return Err(ConfigError::InvalidDatabaseType {
                            expected: "ldf".to_string(),
                            found: ext.to_string(),
                        });
                    }
                }
            }
        }

        Ok(())
    }
}

/// Signal library version configuration
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct LibraryVersionConfig {
    /// Version identifier (e.g., "v1.0", "2024-01-15")
    pub version_id: String,
    /// Human-readable version name
    pub version_name: String,
    /// Path to database file for this version
    pub database_path: PathBuf,
    /// Version description
    pub description: Option<String>,
    /// Creation timestamp
    pub created_at: Option<String>,
}

impl LibraryVersionConfig {
    /// Create a new library version
    pub fn new(version_id: String, database_path: PathBuf) -> Self {
        Self {
            version_id,
            version_name: String::new(),
            database_path,
            description: None,
            created_at: None,
        }
    }

    /// Set version name
    pub fn with_name(mut self, name: String) -> Self {
        self.version_name = name;
        self
    }

    /// Set description
    pub fn with_description(mut self, desc: String) -> Self {
        self.description = Some(desc);
        self
    }
}

/// Signal library configuration
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct LibraryConfig {
    /// Library unique identifier
    pub library_id: String,
    /// Library name
    pub name: String,
    /// Channel type this library supports
    pub channel_type: ChannelType,
    /// Available versions
    pub versions: Vec<LibraryVersionConfig>,
    /// Currently active version ID
    pub active_version: Option<String>,
    /// Library description
    pub description: Option<String>,
}

impl LibraryConfig {
    /// Create a new library configuration
    pub fn new(library_id: String, name: String, channel_type: ChannelType) -> Self {
        Self {
            library_id,
            name,
            channel_type,
            versions: Vec::new(),
            active_version: None,
            description: None,
        }
    }

    /// Add a version to this library
    pub fn add_version(&mut self, version: LibraryVersionConfig) {
        self.versions.push(version);
    }

    /// Set the active version
    pub fn set_active_version(&mut self, version_id: String) -> Result<(), ConfigError> {
        // Check if version exists
        let exists = self.versions.iter().any(|v| v.version_id == version_id);
        if !exists {
            return Err(ConfigError::VersionNotFound(version_id));
        }

        self.active_version = Some(version_id);
        Ok(())
    }

    /// Get the active version configuration
    pub fn get_active_version(&self) -> Option<&LibraryVersionConfig> {
        self.active_version
            .as_ref()
            .and_then(|version_id| self.versions.iter().find(|v| &v.version_id == version_id))
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.library_id.is_empty() {
            return Err(ConfigError::InvalidLibraryId(
                "Library ID cannot be empty".to_string(),
            ));
        }

        if self.name.is_empty() {
            return Err(ConfigError::InvalidLibraryName(
                "Library name cannot be empty".to_string(),
            ));
        }

        // Validate active version exists
        if let Some(ref active_id) = self.active_version {
            if !self.versions.iter().any(|v| &v.version_id == active_id) {
                return Err(ConfigError::VersionNotFound(active_id.clone()));
            }
        }

        // Validate all versions
        for version in &self.versions {
            if !version.database_path.exists() {
                return Err(ConfigError::DatabaseNotFound(version.database_path.clone()));
            }
        }

        Ok(())
    }
}

/// Complete application configuration
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CompleteConfig {
    /// Application-level settings
    pub app_config: AppConfig,
    /// Channel configurations
    pub channels: Vec<ChannelConfig>,
    /// Signal libraries
    pub libraries: Vec<LibraryConfig>,
}

impl CompleteConfig {
    /// Create a new complete configuration
    pub fn new() -> Self {
        Self {
            app_config: AppConfig::default(),
            channels: Vec::new(),
            libraries: Vec::new(),
        }
    }

    /// Add a channel configuration
    pub fn add_channel(&mut self, channel: ChannelConfig) {
        self.channels.push(channel);
    }

    /// Add a library configuration
    pub fn add_library(&mut self, library: LibraryConfig) {
        self.libraries.push(library);
    }

    /// Get channel by number
    pub fn get_channel(&self, channel: u16) -> Option<&ChannelConfig> {
        self.channels.iter().find(|c| c.channel == channel)
    }

    /// Get library by ID
    pub fn get_library(&self, library_id: &str) -> Option<&LibraryConfig> {
        self.libraries.iter().find(|l| l.library_id == library_id)
    }

    /// Validate the complete configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate all channels
        for channel in &self.channels {
            channel.validate()?;
        }

        // Validate all libraries
        for library in &self.libraries {
            library.validate()?;
        }

        Ok(())
    }
}

impl Default for CompleteConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration manager
pub struct ConfigManager {
    /// Configuration directory
    config_dir: PathBuf,
    /// Configuration file path
    config_file: PathBuf,
    /// Current configuration
    config: CompleteConfig,
    /// Whether configuration has been modified since last save
    modified: bool,
}

impl ConfigManager {
    /// Create a new configuration manager
    pub fn new(config_dir: PathBuf) -> Self {
        let config_file = config_dir.join("canview_config.json");

        Self {
            config_dir,
            config_file,
            config: CompleteConfig::new(),
            modified: false,
        }
    }

    /// Get the configuration directory
    pub fn config_dir(&self) -> &Path {
        &self.config_dir
    }

    /// Get the configuration file path
    pub fn config_file(&self) -> &Path {
        &self.config_file
    }

    /// Check if configuration has been modified
    pub fn is_modified(&self) -> bool {
        self.modified
    }

    /// Load configuration from file
    pub fn load(&mut self) -> Result<(), ConfigError> {
        if !self.config_file.exists() {
            // Config file doesn't exist yet, use defaults
            self.config = CompleteConfig::new();
            self.modified = false;
            return Ok(());
        }

        // Read configuration file
        let content = fs::read_to_string(&self.config_file)
            .map_err(|e| ConfigError::IoError(format!("Failed to read config file: {}", e)))?;

        // Parse JSON
        self.config = serde_json::from_str(&content)
            .map_err(|e| ConfigError::ParseError(format!("Failed to parse config: {}", e)))?;

        // Validate configuration
        self.config.validate()?;

        self.modified = false;
        Ok(())
    }

    /// Save configuration to file
    pub fn save(&mut self) -> Result<(), ConfigError> {
        // Ensure config directory exists
        if !self.config_dir.exists() {
            fs::create_dir_all(&self.config_dir).map_err(|e| {
                ConfigError::IoError(format!("Failed to create config directory: {}", e))
            })?;
        }

        // Validate before saving
        self.config.validate()?;

        // Serialize to JSON with pretty formatting
        let content = serde_json::to_string_pretty(&self.config).map_err(|e| {
            ConfigError::SerializeError(format!("Failed to serialize config: {}", e))
        })?;

        // Write to file
        fs::write(&self.config_file, content)
            .map_err(|e| ConfigError::IoError(format!("Failed to write config file: {}", e)))?;

        self.modified = false;
        Ok(())
    }

    /// Get the current configuration
    pub fn config(&self) -> &CompleteConfig {
        &self.config
    }

    /// Get mutable reference to configuration
    pub fn config_mut(&mut self) -> &mut CompleteConfig {
        self.modified = true;
        &mut self.config
    }

    /// Replace the entire configuration
    pub fn set_config(&mut self, config: CompleteConfig) -> Result<(), ConfigError> {
        config.validate()?;
        self.config = config;
        self.modified = true;
        Ok(())
    }

    /// Import configuration from a JSON file
    pub fn import_from_file(&mut self, path: &Path) -> Result<(), ConfigError> {
        let content = fs::read_to_string(path)
            .map_err(|e| ConfigError::IoError(format!("Failed to read import file: {}", e)))?;

        let config: CompleteConfig = serde_json::from_str(&content)
            .map_err(|e| ConfigError::ParseError(format!("Failed to parse import file: {}", e)))?;

        config.validate()?;
        self.config = config;
        self.modified = true;
        Ok(())
    }

    /// Export configuration to a JSON file
    pub fn export_to_file(&self, path: &Path) -> Result<(), ConfigError> {
        let content = serde_json::to_string_pretty(&self.config).map_err(|e| {
            ConfigError::SerializeError(format!("Failed to serialize for export: {}", e))
        })?;

        fs::write(path, content)
            .map_err(|e| ConfigError::IoError(format!("Failed to write export file: {}", e)))?;

        Ok(())
    }

    /// Reset configuration to defaults
    pub fn reset_to_defaults(&mut self) {
        self.config = CompleteConfig::new();
        self.modified = true;
    }

    /// Create a backup of the current configuration
    pub fn create_backup(&self) -> Result<PathBuf, ConfigError> {
        let backup_dir = self.config_dir.join("backups");
        if !backup_dir.exists() {
            fs::create_dir_all(&backup_dir).map_err(|e| {
                ConfigError::IoError(format!("Failed to create backup directory: {}", e))
            })?;
        }

        // Create backup filename with timestamp
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let backup_file = backup_dir.join(format!("config_backup_{}.json", timestamp));

        // Copy current config to backup
        fs::copy(&self.config_file, &backup_file)
            .map_err(|e| ConfigError::IoError(format!("Failed to create backup: {}", e)))?;

        Ok(backup_file)
    }

    /// List available backups
    pub fn list_backups(&self) -> Result<Vec<PathBuf>, ConfigError> {
        let backup_dir = self.config_dir.join("backups");

        if !backup_dir.exists() {
            return Ok(Vec::new());
        }

        let mut backups = Vec::new();

        let entries = fs::read_dir(&backup_dir)
            .map_err(|e| ConfigError::IoError(format!("Failed to read backup directory: {}", e)))?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                backups.push(path);
            }
        }

        backups.sort();
        backups.reverse(); // Most recent first
        Ok(backups)
    }

    /// Restore from a backup
    pub fn restore_from_backup(&mut self, backup_path: &Path) -> Result<(), ConfigError> {
        if !backup_path.exists() {
            return Err(ConfigError::BackupNotFound(backup_path.to_path_buf()));
        }

        let content = fs::read_to_string(backup_path)
            .map_err(|e| ConfigError::IoError(format!("Failed to read backup file: {}", e)))?;

        let config: CompleteConfig = serde_json::from_str(&content)
            .map_err(|e| ConfigError::ParseError(format!("Failed to parse backup file: {}", e)))?;

        config.validate()?;
        self.config = config;
        self.modified = true;
        Ok(())
    }
}

/// Configuration errors
#[derive(Clone, Debug, PartialEq)]
pub enum ConfigError {
    /// Invalid channel name
    InvalidChannelName(String),
    /// Database file not found
    DatabaseNotFound(PathBuf),
    /// Invalid database type
    InvalidDatabaseType { expected: String, found: String },
    /// Invalid library ID
    InvalidLibraryId(String),
    /// Invalid library name
    InvalidLibraryName(String),
    /// Version not found
    VersionNotFound(String),
    /// IO error
    IoError(String),
    /// Parse error
    ParseError(String),
    /// Serialize error
    SerializeError(String),
    /// Backup not found
    BackupNotFound(PathBuf),
    /// Validation error
    ValidationError(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::InvalidChannelName(msg) => write!(f, "Invalid channel name: {}", msg),
            ConfigError::DatabaseNotFound(path) => {
                write!(f, "Database file not found: {}", path.display())
            }
            ConfigError::InvalidDatabaseType { expected, found } => {
                write!(
                    f,
                    "Invalid database type: expected {}, found {}",
                    expected, found
                )
            }
            ConfigError::InvalidLibraryId(msg) => write!(f, "Invalid library ID: {}", msg),
            ConfigError::InvalidLibraryName(msg) => write!(f, "Invalid library name: {}", msg),
            ConfigError::VersionNotFound(id) => write!(f, "Version not found: {}", id),
            ConfigError::IoError(msg) => write!(f, "IO error: {}", msg),
            ConfigError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            ConfigError::SerializeError(msg) => write!(f, "Serialize error: {}", msg),
            ConfigError::BackupNotFound(path) => {
                write!(f, "Backup not found: {}", path.display())
            }
            ConfigError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl std::error::Error for ConfigError {}

/// Convert legacy channel mappings to new channel configs
pub fn convert_legacy_mappings(mappings: &[ChannelMapping]) -> Vec<ChannelConfig> {
    mappings
        .iter()
        .map(|mapping| {
            let db_path = if mapping.path.is_empty() {
                None
            } else {
                Some(PathBuf::from(&mapping.path))
            };

            ChannelConfig {
                channel: mapping.channel_id,
                name: if mapping.description.is_empty() {
                    format!("{}{}", mapping.channel_type, mapping.channel_id)
                } else {
                    mapping.description.clone()
                },
                channel_type: mapping.channel_type,
                database_path: db_path,
                enabled: true,
            }
        })
        .collect()
}

/// Convert channel configs to legacy channel mappings
pub fn convert_to_legacy_mappings(channels: &[ChannelConfig]) -> Vec<ChannelMapping> {
    channels
        .iter()
        .map(|config| ChannelMapping {
            channel_id: config.channel,
            channel_type: config.channel_type,
            path: config
                .database_path
                .as_ref()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default(),
            description: config.name.clone(),
            library_id: None,
            version_name: None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_config_creation() {
        let config = ChannelConfig::can(1, "CAN1".to_string());
        assert_eq!(config.channel, 1);
        assert_eq!(config.name, "CAN1");
        assert_eq!(config.channel_type, ChannelType::CAN);
        assert!(config.database_path.is_none());
        assert!(config.enabled);
    }

    #[test]
    fn test_channel_config_builder() {
        let config = ChannelConfig::lin(2, "LIN2".to_string())
            .with_database(PathBuf::from("test.ldf"))
            .disable();

        assert_eq!(config.channel, 2);
        assert_eq!(config.channel_type, ChannelType::LIN);
        assert_eq!(config.database_path, Some(PathBuf::from("test.ldf")));
        assert!(!config.enabled);
    }

    #[test]
    fn test_library_config() {
        let mut lib = LibraryConfig::new(
            "lib1".to_string(),
            "Test Library".to_string(),
            ChannelType::CAN,
        );

        let version = LibraryVersionConfig::new("v1.0".to_string(), PathBuf::from("test.dbc"))
            .with_name("Version 1.0".to_string());

        lib.add_version(version);
        lib.set_active_version("v1.0".to_string()).unwrap();

        assert_eq!(lib.versions.len(), 1);
        assert_eq!(lib.active_version, Some("v1.0".to_string()));
        assert!(lib.get_active_version().is_some());
    }

    #[test]
    fn test_complete_config() {
        let mut config = CompleteConfig::new();

        let channel = ChannelConfig::can(1, "CAN1".to_string());
        config.add_channel(channel);

        assert_eq!(config.channels.len(), 1);
        assert!(config.get_channel(1).is_some());
        assert!(config.get_channel(2).is_none());
    }

    #[test]
    fn test_config_validation() {
        let config = CompleteConfig::new();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_convert_legacy_mappings() {
        let mappings = vec![ChannelMapping {
            channel_type: ChannelType::CAN,
            channel_id: 1,
            path: "test.dbc".to_string(),
            description: "CAN Channel 1".to_string(),
            library_id: None,
            version_name: None,
        }];

        let configs = convert_legacy_mappings(&mappings);
        assert_eq!(configs.len(), 1);
        assert_eq!(configs[0].channel, 1);
        assert_eq!(configs[0].channel_type, ChannelType::CAN);
        assert_eq!(configs[0].database_path, Some(PathBuf::from("test.dbc")));
        assert_eq!(configs[0].name, "CAN Channel 1");
    }

    #[test]
    fn test_library_validation() {
        let mut lib = LibraryConfig::new("".to_string(), "Test".to_string(), ChannelType::CAN);
        assert!(lib.validate().is_err());

        lib.library_id = "lib1".to_string();
        assert!(lib.validate().is_ok());

        lib.active_version = Some("nonexistent".to_string());
        assert!(lib.validate().is_err());
    }
}
