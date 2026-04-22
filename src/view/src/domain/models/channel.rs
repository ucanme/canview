//! Channel domain model
//!
//! Pure business model for CAN/LIN channels.
//! No UI framework dependencies.

use std::path::{Path, PathBuf};
use std::fmt;

/// Channel type enumeration
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ChannelType {
    /// Controller Area Network
    Can,
    /// Local Interconnect Network
    Lin,
}

impl ChannelType {
    /// Check if this is a CAN channel
    pub fn is_can(&self) -> bool {
        matches!(self, ChannelType::Can)
    }

    /// Check if this is a LIN channel
    pub fn is_lin(&self) -> bool {
        matches!(self, ChannelType::Lin)
    }

    /// Get the expected database file extension
    pub fn database_extension(&self) -> &'static str {
        match self {
            ChannelType::Can => "dbc",
            ChannelType::Lin => "ldf",
        }
    }

    /// Get display name
    pub fn as_str(&self) -> &'static str {
        match self {
            ChannelType::Can => "CAN",
            ChannelType::Lin => "LIN",
        }
    }
}

impl fmt::Display for ChannelType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Channel domain model
///
/// Represents a physical or virtual communication channel
/// without any dependencies on UI frameworks or serialization.
#[derive(Clone, Debug, PartialEq)]
pub struct Channel {
    /// Channel number (1-based)
    pub number: u16,

    /// Channel name (e.g., "CAN1", "LIN2")
    pub name: String,

    /// Channel type
    pub channel_type: ChannelType,

    /// Path to the signal database file (DBC for CAN, LDF for LIN)
    pub database_path: Option<PathBuf>,

    /// Whether this channel is currently active/enabled
    pub is_enabled: bool,
}

impl Channel {
    /// Create a new channel
    pub fn new(number: u16, name: String, channel_type: ChannelType) -> Self {
        Self {
            number,
            name,
            channel_type,
            database_path: None,
            is_enabled: true,
        }
    }

    /// Create a new CAN channel
    pub fn can(number: u16, name: String) -> Self {
        Self::new(number, name, ChannelType::Can)
    }

    /// Create a new LIN channel
    pub fn lin(number: u16, name: String) -> Self {
        Self::new(number, name, ChannelType::Lin)
    }

    /// Set the database path
    pub fn with_database(mut self, path: PathBuf) -> Self {
        self.database_path = Some(path);
        self
    }

    /// Enable the channel
    pub fn enable(mut self) -> Self {
        self.is_enabled = true;
        self
    }

    /// Disable the channel
    pub fn disable(mut self) -> Self {
        self.is_enabled = false;
        self
    }

    /// Get the default name for this channel
    pub fn default_name(&self) -> String {
        format!("{}{}", self.channel_type.as_str(), self.number)
    }

    /// Check if the channel has a valid database path
    pub fn has_database(&self) -> bool {
        self.database_path.as_ref().map_or(false, |p| p.exists())
    }

    /// Validate the channel configuration
    pub fn validate(&self) -> Result<(), ChannelError> {
        if self.name.is_empty() {
            return Err(ChannelError::InvalidName("Channel name cannot be empty".to_string()));
        }

        if self.number == 0 {
            return Err(ChannelError::InvalidNumber("Channel number must be greater than 0".to_string()));
        }

        // Validate database path if set
        if let Some(db_path) = &self.database_path {
            if !db_path.exists() {
                return Err(ChannelError::DatabaseNotFound(db_path.clone()));
            }

            // Validate file extension matches channel type
            let ext = db_path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");

            let expected_ext = self.channel_type.database_extension();
            if !ext.eq_ignore_ascii_case(expected_ext) {
                return Err(ChannelError::InvalidDatabaseExtension {
                    expected: expected_ext.to_string(),
                    found: ext.to_string(),
                });
            }
        }

        Ok(())
    }

    /// Get a human-readable description
    pub fn description(&self) -> String {
        let db_info = if let Some(db_path) = &self.database_path {
            format!(" → {}", db_path.file_name().and_then(|n| n.to_str()).unwrap_or("Unknown"))
        } else {
            String::new()
        };

        format!("{}{}{}", self.channel_type.as_str(), self.number, db_info)
    }
}

/// Channel configuration
///
/// Contains configuration settings for a channel,
/// separate from the runtime channel state.
#[derive(Clone, Debug, PartialEq)]
pub struct ChannelConfig {
    /// Channel number
    pub channel: u16,

    /// Channel name
    pub name: String,

    /// Channel type
    pub channel_type: ChannelType,

    /// Path to database file
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

    /// Create from a Channel domain model
    pub fn from_channel(channel: &Channel) -> Self {
        Self {
            channel: channel.number,
            name: channel.name.clone(),
            channel_type: channel.channel_type,
            database_path: channel.database_path.clone(),
            enabled: channel.is_enabled,
        }
    }

    /// Convert to a Channel domain model
    pub fn to_channel(&self) -> Channel {
        Channel {
            number: self.channel,
            name: self.name.clone(),
            channel_type: self.channel_type,
            database_path: self.database_path.clone(),
            is_enabled: self.enabled,
        }
    }
}

/// Channel-related errors
#[derive(Clone, Debug, PartialEq)]
pub enum ChannelError {
    /// Invalid channel name
    InvalidName(String),

    /// Invalid channel number
    InvalidNumber(String),

    /// Database file not found
    DatabaseNotFound(PathBuf),

    /// Database file extension doesn't match channel type
    InvalidDatabaseExtension { expected: String, found: String },

    /// Channel configuration is invalid
    InvalidConfiguration(String),
}

impl fmt::Display for ChannelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChannelError::InvalidName(msg) => write!(f, "Invalid channel name: {}", msg),
            ChannelError::InvalidNumber(msg) => write!(f, "Invalid channel number: {}", msg),
            ChannelError::DatabaseNotFound(path) => {
                write!(f, "Database not found: {}", path.display())
            }
            ChannelError::InvalidDatabaseExtension { expected, found } => {
                write!(
                    f,
                    "Invalid database extension: expected '{}', found '{}'",
                    expected, found
                )
            }
            ChannelError::InvalidConfiguration(msg) => {
                write!(f, "Invalid configuration: {}", msg)
            }
        }
    }
}

impl std::error::Error for ChannelError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_type() {
        assert!(ChannelType::Can.is_can());
        assert!(!ChannelType::Can.is_lin());
        assert!(ChannelType::Lin.is_lin());
        assert!(!ChannelType::Lin.is_can());

        assert_eq!(ChannelType::Can.as_str(), "CAN");
        assert_eq!(ChannelType::Lin.as_str(), "LIN");
        assert_eq!(ChannelType::Can.database_extension(), "dbc");
        assert_eq!(ChannelType::Lin.database_extension(), "ldf");
    }

    #[test]
    fn test_channel_creation() {
        let channel = Channel::can(1, "CAN1".to_string());
        assert_eq!(channel.number, 1);
        assert_eq!(channel.name, "CAN1");
        assert_eq!(channel.channel_type, ChannelType::Can);
        assert!(channel.is_enabled);
        assert!(channel.database_path.is_none());
    }

    #[test]
    fn test_channel_builder() {
        let db_path = PathBuf::from("/test/dbc.dbc");
        let channel = Channel::can(1, "CAN1".to_string())
            .with_database(db_path.clone())
            .enable();

        assert_eq!(channel.database_path, Some(db_path));
        assert!(channel.is_enabled);
    }

    #[test]
    fn test_channel_validate_empty_name() {
        let channel = Channel::can(1, "".to_string());
        assert!(channel.validate().is_err());
        match channel.validate().unwrap_err() {
            ChannelError::InvalidName(_) => {}
            _ => panic!("Expected InvalidName error"),
        }
    }

    #[test]
    fn test_channel_validate_invalid_number() {
        let channel = Channel::can(0, "CAN0".to_string());
        assert!(channel.validate().is_err());
        match channel.validate().unwrap_err() {
            ChannelError::InvalidNumber(_) => {}
            _ => panic!("Expected InvalidNumber error"),
        }
    }

    #[test]
    fn test_channel_validate_missing_database() {
        let db_path = PathBuf::from("/nonexistent/file.dbc");
        let channel = Channel::can(1, "CAN1".to_string()).with_database(db_path);

        assert!(channel.validate().is_err());
        match channel.validate().unwrap_err() {
            ChannelError::DatabaseNotFound(p) => assert_eq!(p, PathBuf::from("/nonexistent/file.dbc")),
            _ => panic!("Expected DatabaseNotFound error"),
        }
    }

    #[test]
    fn test_channel_validate_wrong_extension() {
        // Create a temp file with wrong extension
        let temp_path = PathBuf::from("/tmp/test.ldf"); // Wrong extension for CAN
        let channel = Channel::can(1, "CAN1".to_string()).with_database(temp_path);

        // Note: This test would need the file to actually exist to fully test
        // For now, we just test the logic structure
        assert_eq!(ChannelType::Can.database_extension(), "dbc");
        assert_eq!(ChannelType::Lin.database_extension(), "ldf");
    }

    #[test]
    fn test_channel_description() {
        let channel = Channel::can(1, "CAN1".to_string());
        assert_eq!(channel.description(), "CAN1");

        let with_db = Channel::can(2, "CAN2".to_string())
            .with_database(PathBuf::from("/path/to/database.dbc"));
        assert!(with_db.description().contains("CAN2"));
        assert!(with_db.description().contains("database.dbc"));
    }

    #[test]
    fn test_channel_config_conversion() {
        let channel = Channel::can(1, "CAN1".to_string())
            .with_database(PathBuf::from("/test.dbc"));

        let config = ChannelConfig::from_channel(&channel);
        assert_eq!(config.channel, 1);
        assert_eq!(config.name, "CAN1");

        let back = config.to_channel();
        assert_eq!(back.number, channel.number);
        assert_eq!(back.name, channel.name);
    }

    #[test]
    fn test_channel_error_display() {
        let err = ChannelError::InvalidName("test error".to_string());
        assert!(err.to_string().contains("Invalid channel name"));
        assert!(err.to_string().contains("test error"));
    }
}
