//! File loading commands
//!
//! Command handlers for BLF file loading and database import operations.

use std::path::PathBuf;
use blf::BlfResult;

/// Load a BLF file
pub struct LoadBlfFile {
    pub file_path: PathBuf,
}

impl LoadBlfFile {
    pub fn new(file_path: PathBuf) -> Self {
        Self { file_path }
    }

    /// Get the file path as a reference
    pub fn path(&self) -> &PathBuf {
        &self.file_path
    }

    /// Check if the file exists
    pub fn file_exists(&self) -> bool {
        self.file_path.exists()
    }

    /// Get the file name for display
    pub fn file_name(&self) -> Option<String> {
        self.file_path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())
    }

    /// Create a status message for loading
    pub fn loading_message(&self) -> String {
        format!(
            "Loading BLF: {}",
            self.file_name().unwrap_or_else(|| "unknown".to_string())
        )
    }
}

/// Import a database file (DBC/LDF)
pub struct ImportDatabaseFile {
    pub file_path: Option<PathBuf>,
    pub channel_id: Option<u16>,
}

impl ImportDatabaseFile {
    pub fn new(file_path: Option<PathBuf>, channel_id: Option<u16>) -> Self {
        Self {
            file_path,
            channel_id,
        }
    }

    /// Create command for importing without a specific file (file dialog will be used)
    pub fn with_dialog() -> Self {
        Self {
            file_path: None,
            channel_id: None,
        }
    }

    /// Create command for importing to a specific channel
    pub fn for_channel(file_path: PathBuf, channel_id: u16) -> Self {
        Self {
            file_path: Some(file_path),
            channel_id: Some(channel_id),
        }
    }

    /// Check if this command has a file path
    pub fn has_file_path(&self) -> bool {
        self.file_path.is_some()
    }

    /// Check if this command targets a specific channel
    pub fn has_channel(&self) -> bool {
        self.channel_id.is_some()
    }
}

/// Process BLF loading result
///
/// This struct encapsulates the result of a BLF file loading operation
/// and provides helper methods for processing it.
pub struct ProcessBlfResult {
    pub result: anyhow::Result<BlfResult>,
}

impl ProcessBlfResult {
    pub fn new(result: anyhow::Result<BlfResult>) -> Self {
        Self { result }
    }

    /// Check if the load was successful
    pub fn is_success(&self) -> bool {
        self.result.is_ok()
    }

    /// Check if there were parsing errors (non-fatal)
    pub fn has_parse_errors(&self) -> bool {
        if let Ok(result) = &self.result {
            !result.errors.is_empty()
        } else {
            false
        }
    }

    /// Get the number of successfully loaded objects
    pub fn object_count(&self) -> usize {
        if let Ok(result) = &self.result {
            result.objects.len()
        } else {
            0
        }
    }

    /// Get the number of parse errors
    pub fn error_count(&self) -> usize {
        if let Ok(result) = &self.result {
            result.errors.len()
        } else {
            0
        }
    }

    /// Get a user-friendly status message
    pub fn status_message(&self) -> String {
        match &self.result {
            Ok(result) => {
                let error_count = result.errors.len();
                if error_count > 0 {
                    let first_error = &result.errors[0];
                    format!(
                        "⚠️ Loaded {} messages | {} errors (first: {})",
                        result.objects.len(),
                        error_count,
                        first_error
                    )
                } else {
                    format!("✅ Loaded {} messages", result.objects.len())
                }
            }
            Err(e) => {
                format!("❌ File Error: {}", e)
            }
        }
    }

    /// Get detailed error message for console output
    pub fn console_message(&self) -> Option<String> {
        match &self.result {
            Ok(result) => {
                if result.errors.len() > 0 {
                    let mut msg = format!(
                        "⚠️  BLF 解析过程中发现 {} 个错误:\n",
                        result.errors.len()
                    );
                    for (i, error) in result.errors.iter().enumerate() {
                        msg.push_str(&format!("  错误 {}: {}\n", i + 1, error));
                    }
                    msg.push_str(&format!(
                        "  ✅ 但仍成功解析了 {} 个对象，这些对象将正常显示",
                        result.objects.len()
                    ));
                    Some(msg)
                } else {
                    None
                }
            }
            Err(e) => {
                Some(format!(
                    "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\
                     📂 BLF File Loading Failed\n\
                     Error: {:?}\n\
                     ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
                    e
                ))
            }
        }
    }

    /// Extract the BLF result if successful
    pub fn into_result(self) -> anyhow::Result<BlfResult> {
        self.result
    }

    /// Get reference to the BLF result if successful
    pub fn get_result(&self) -> Option<&BlfResult> {
        self.result.as_ref().ok()
    }
}

/// BLF file loading statistics
///
/// Provides diagnostic information about loaded BLF files.
#[derive(Debug, Clone)]
pub struct BlfLoadStats {
    pub object_count: usize,
    pub error_count: usize,
    pub has_parse_errors: bool,
    pub first_error: Option<String>,
}

impl From<&ProcessBlfResult> for BlfLoadStats {
    fn from(result: &ProcessBlfResult) -> Self {
        if let Ok(blf_result) = &result.result {
            let first_error = blf_result.errors.first().map(|e| e.to_string());
            Self {
                object_count: blf_result.objects.len(),
                error_count: blf_result.errors.len(),
                has_parse_errors: !blf_result.errors.is_empty(),
                first_error,
            }
        } else {
            Self {
                object_count: 0,
                error_count: 1,
                has_parse_errors: false,
                first_error: result.result.as_ref().err().map(|e| e.to_string()),
            }
        }
    }
}

impl BlfLoadStats {
    /// Check if the load was completely successful (no errors)
    pub fn is_perfect(&self) -> bool {
        self.object_count > 0 && self.error_count == 0
    }

    /// Check if the load has data (even with errors)
    pub fn has_data(&self) -> bool {
        self.object_count > 0
    }

    /// Get a summary string
    pub fn summary(&self) -> String {
        if self.is_perfect() {
            format!("✅ Loaded {} messages", self.object_count)
        } else if self.has_data() {
            format!(
                "⚠️ Loaded {} messages with {} errors",
                self.object_count, self.error_count
            )
        } else {
            format!("❌ Failed to load file")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_blf_file_creation() {
        let path = PathBuf::from("/test/path.blf");
        let cmd = LoadBlfFile::new(path.clone());

        assert_eq!(cmd.path(), &path);
        assert_eq!(cmd.file_name(), Some("path.blf".to_string()));
    }

    #[test]
    fn test_import_database_with_dialog() {
        let cmd = ImportDatabaseFile::with_dialog();

        assert!(!cmd.has_file_path());
        assert!(!cmd.has_channel());
    }

    #[test]
    fn test_import_database_for_channel() {
        let path = PathBuf::from("/test/database.dbc");
        let cmd = ImportDatabaseFile::for_channel(path, 1);

        assert!(cmd.has_file_path());
        assert!(cmd.has_channel());
    }
}
