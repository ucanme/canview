//! File handling utilities
//!
//! This module contains utility functions for file operations
//! such as importing database files and opening BLF files.

use std::path::PathBuf;

/// Import a database file (DBC or LDF)
///
/// # Arguments
/// * `db_type` - Either "dbc" or "ldf"
///
/// # Returns
/// Result containing the file path, or an error message
pub fn import_database_file(db_type: &str) -> Result<PathBuf, String> {
    let filter_extension = match db_type {
        "dbc" => "DBC Files",
        "ldf" => "LDF Files",
        _ => return Err("Invalid database type".to_string()),
    };

    let file_ext = match db_type {
        "dbc" => "dbc",
        "ldf" => "ldf",
        _ => return Err("Invalid database type".to_string()),
    };

    if let Some(path) = rfd::FileDialog::new()
        .add_filter(filter_extension, &[file_ext])
        .pick_file()
    {
        Ok(path)
    } else {
        Err("No file selected".to_string())
    }
}

/// Import any database file (DBC or LDF)
///
/// # Returns
/// Result containing the file path, or an error message
pub fn import_any_database_file() -> Result<PathBuf, String> {
    if let Some(path) = rfd::FileDialog::new()
        .add_filter("Database Files", &["dbc", "ldf"])
        .pick_file()
    {
        Ok(path)
    } else {
        Err("No file selected".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import_database_file_invalid_type() {
        let result = import_database_file("invalid");
        assert!(result.is_err());
    }
}
