//! Library management commands
//!
//! Command handlers for library and version management operations.

use crate::models::ChannelType;

/// Create a new library
pub struct CreateLibrary {
    pub name: String,
    pub channel_type: ChannelType,
}

impl CreateLibrary {
    pub fn new(name: String, channel_type: ChannelType) -> Self {
        Self { name, channel_type }
    }

    /// Validate the library name
    pub fn is_valid(&self) -> bool {
        !self.name.trim().is_empty()
    }

    /// Get error message if invalid
    pub fn validation_error(&self) -> Option<String> {
        if self.name.trim().is_empty() {
            Some("Library name cannot be empty".to_string())
        } else {
            None
        }
    }

    /// Get success message
    pub fn success_message(&self) -> String {
        format!("Library '{}' created", self.name)
    }
}

/// Delete a library
pub struct DeleteLibrary {
    pub library_id: String,
}

impl DeleteLibrary {
    pub fn new(library_id: String) -> Self {
        Self { library_id }
    }

    /// Get reference to library ID
    pub fn id(&self) -> &str {
        &self.library_id
    }

    /// Get success message
    pub fn success_message(&self) -> String {
        "Library deleted".to_string()
    }

    /// Get error message prefix
    pub fn error_message(&self) -> String {
        "Error deleting library".to_string()
    }
}

/// Add a version to a library
pub struct AddLibraryVersion {
    pub library_id: Option<String>,
    pub version_name: String,
    pub description: Option<String>,
    pub database_path: Option<String>,
}

impl AddLibraryVersion {
    pub fn new(
        library_id: Option<String>,
        version_name: String,
    ) -> Self {
        Self {
            library_id,
            version_name,
            description: None,
            database_path: None,
        }
    }

    /// Set the version description
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// Set the database path
    pub fn with_database_path(mut self, path: String) -> Self {
        self.database_path = Some(path);
        self
    }

    /// Validate the version data
    pub fn is_valid(&self) -> bool {
        self.library_id.is_some() && !self.version_name.trim().is_empty()
    }

    /// Get validation error if any
    pub fn validation_error(&self) -> Option<String> {
        if self.library_id.is_none() {
            Some("No library selected".to_string())
        } else if self.version_name.trim().is_empty() {
            Some("Version name cannot be empty".to_string())
        } else {
            None
        }
    }

    /// Get the library ID
    pub fn library_id(&self) -> Option<&str> {
        self.library_id.as_deref()
    }

    /// Check if a library is selected
    pub fn has_library(&self) -> bool {
        self.library_id.is_some()
    }

    /// Get success message
    pub fn success_message(&self) -> String {
        if self.database_path.as_ref().map_or(false, |p| !p.is_empty()) {
            format!(
                "Version '{}' created successfully with database",
                self.version_name
            )
        } else {
            format!(
                "Version '{}' created successfully. Use 'Add Database File' to attach a database.",
                self.version_name
            )
        }
    }
}

/// Delete a version from a library
pub struct DeleteLibraryVersion {
    pub library_id: String,
    pub version_name: String,
}

impl DeleteLibraryVersion {
    pub fn new(library_id: String, version_name: String) -> Self {
        Self {
            library_id,
            version_name,
        }
    }

    /// Get reference to library ID
    pub fn library_id(&self) -> &str {
        &self.library_id
    }

    /// Get reference to version name
    pub fn version_name(&self) -> &str {
        &self.version_name
    }

    /// Get success message
    pub fn success_message(&self) -> String {
        format!("Version '{}' deleted", self.version_name)
    }

    /// Get error message prefix
    pub fn error_message(&self) -> String {
        "Error deleting version".to_string()
    }
}

/// Load a library version
pub struct LoadLibraryVersion {
    pub library_id: String,
    pub version_name: String,
    pub default_channel_id: u16,
}

impl LoadLibraryVersion {
    pub fn new(library_id: String, version_name: String, default_channel_id: u16) -> Self {
        Self {
            library_id,
            version_name,
            default_channel_id,
        }
    }

    /// Create with default channel ID 1
    pub fn with_default_channel(library_id: String, version_name: String) -> Self {
        Self::new(library_id, version_name, 1)
    }

    /// Get reference to library ID
    pub fn library_id(&self) -> &str {
        &self.library_id
    }

    /// Get reference to version name
    pub fn version_name(&self) -> &str {
        &self.version_name
    }

    /// Get default channel ID
    pub fn default_channel_id(&self) -> u16 {
        self.default_channel_id
    }

    /// Get debug message
    pub fn debug_message(&self) -> String {
        format!(
            "Internal load library version: lib={}, ver={}, ch={}",
            self.library_id,
            self.version_name,
            self.default_channel_id
        )
    }
}

/// Apply a library version to channel mappings
pub struct ApplyVersionToMappings {
    pub library_id: String,
    pub version_name: String,
}

impl ApplyVersionToMappings {
    pub fn new(library_id: String, version_name: String) -> Self {
        Self {
            library_id,
            version_name,
        }
    }

    /// Get reference to library ID
    pub fn library_id(&self) -> &str {
        &self.library_id
    }

    /// Get reference to version name
    pub fn version_name(&self) -> &str {
        &self.version_name
    }

    /// Get debug message
    pub fn debug_message(&self) -> String {
        format!(
            "Applying version {} of {} to mappings",
            self.version_name,
            self.library_id
        )
    }

    /// Get success message
    pub fn success_message(&self) -> String {
        format!("✅ Applied version {} to all plot channels", self.version_name)
    }
}

/// Library operation result
///
/// Encapsulates the result of a library management operation.
#[derive(Debug, Clone)]
pub enum LibraryOperationResult {
    Created(String),
    Deleted(String),
    Updated(String),
    Error(String),
}

impl LibraryOperationResult {
    /// Check if the operation was successful
    pub fn is_success(&self) -> bool {
        !matches!(self, Self::Error(_))
    }

    /// Get the status message
    pub fn message(&self) -> String {
        match self {
            Self::Created(msg) => msg.clone(),
            Self::Deleted(msg) => msg.clone(),
            Self::Updated(msg) => msg.clone(),
            Self::Error(msg) => msg.clone(),
        }
    }

    /// Get reference to the message
    pub fn as_str(&self) -> &str {
        match self {
            Self::Created(msg) |
            Self::Deleted(msg) |
            Self::Updated(msg) |
            Self::Error(msg) => msg.as_str(),
        }
    }
}

impl From<Result<String, String>> for LibraryOperationResult {
    fn from(result: Result<String, String>) -> Self {
        match result {
            Ok(msg) => Self::Created(msg),
            Err(msg) => Self::Error(msg),
        }
    }
}

/// Library version information
#[derive(Debug, Clone)]
pub struct LibraryVersionInfo {
    pub library_id: String,
    pub version_name: String,
    pub description: Option<String>,
    pub database_path: Option<String>,
    pub created_date: String,
}

impl LibraryVersionInfo {
    pub fn new(
        library_id: String,
        version_name: String,
        created_date: String,
    ) -> Self {
        Self {
            library_id,
            version_name,
            description: None,
            database_path: None,
            created_date,
        }
    }

    /// Check if the version has a database path
    pub fn has_database(&self) -> bool {
        self.database_path
            .as_ref()
            .map_or(false, |p| !p.trim().is_empty())
    }

    /// Check if the database file exists
    pub fn database_exists(&self) -> bool {
        self.database_path
            .as_ref()
            .map_or(false, |p| std::path::Path::new(p).exists())
    }

    /// Get error message if database is missing
    pub fn database_error(&self) -> Option<String> {
        if !self.has_database() {
            Some(format!(
                "Database path is empty for version '{}'. Please add a database file in the Library view.",
                self.version_name
            ))
        } else if !self.database_exists() {
            Some(format!(
                "Database file not found: {}. Please check the file path in Library view.",
                self.database_path.as_ref().unwrap()
            ))
        } else {
            None
        }
    }

    /// Get display name
    pub fn display_name(&self) -> String {
        if let Some(desc) = &self.description {
            format!("{} - {}", self.version_name, desc)
        } else {
            self.version_name.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_library_validation() {
        let cmd = CreateLibrary::new("TestLib".to_string(), ChannelType::CAN);
        assert!(cmd.is_valid());
        assert!(cmd.validation_error().is_none());
    }

    #[test]
    fn test_create_library_empty_name() {
        let cmd = CreateLibrary::new("   ".to_string(), ChannelType::CAN);
        assert!(!cmd.is_valid());
        assert!(cmd.validation_error().is_some());
    }

    #[test]
    fn test_add_library_version_validation() {
        let cmd = AddLibraryVersion::new(Some("lib123".to_string()), "v1.0".to_string());
        assert!(cmd.is_valid());
        assert!(cmd.has_library());
    }

    #[test]
    fn test_add_library_version_no_library() {
        let cmd = AddLibraryVersion::new(None, "v1.0".to_string());
        assert!(!cmd.is_valid());
        assert!(!cmd.has_library());
    }

    #[test]
    fn test_library_operation_result() {
        let result = LibraryOperationResult::Created("Success".to_string());
        assert!(result.is_success());
        assert_eq!(result.message(), "Success");

        let error = LibraryOperationResult::Error("Failed".to_string());
        assert!(!error.is_success());
        assert_eq!(error.message(), "Failed");
    }
}
