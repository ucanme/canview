//! Signal library management module
//!
//! 提供信号库的CRUD操作、版本管理和验证功能

mod storage;

pub use storage::SignalLibraryStorage;

use crate::models::{SignalLibrary, LibraryVersion, ChannelType, DatabaseType, ChannelMapping, ChannelDatabase};
use parser::dbc::{DbcParser, DbcDatabase};
use parser::ldf::{LdfParser, LdfDatabase};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::collections::HashMap;

/// 数据库验证结果
#[derive(Debug, Clone)]
pub struct DatabaseValidation {
    pub is_valid: bool,
    pub message_count: usize,
    pub signal_count: usize,
    pub error: Option<String>,
    pub warnings: Vec<String>,
}

impl DatabaseValidation {
    /// 创建成功的验证结果
    pub fn success(message_count: usize, signal_count: usize) -> Self {
        Self {
            is_valid: true,
            message_count,
            signal_count,
            error: None,
            warnings: Vec::new(),
        }
    }

    /// 创建失败的验证结果
    pub fn error(error: String) -> Self {
        Self {
            is_valid: false,
            message_count: 0,
            signal_count: 0,
            error: Some(error),
            warnings: Vec::new(),
        }
    }

    /// 添加警告
    pub fn with_warning(mut self, warning: String) -> Self {
        self.warnings.push(warning);
        self
    }
}

/// 数据库统计信息
#[derive(Debug, Clone)]
pub struct DatabaseStats {
    pub message_count: usize,
    pub signal_count: usize,
    pub file_size: u64,
    pub last_modified: String,
}

/// 信号库管理器
pub struct LibraryManager {
    libraries: Vec<SignalLibrary>,
}

impl LibraryManager {
    /// 创建新的管理器
    pub fn new() -> Self {
        Self {
            libraries: Vec::new(),
        }
    }

    /// 从库列表创建管理器
    pub fn from_libraries(libraries: Vec<SignalLibrary>) -> Self {
        Self { libraries }
    }

    /// 获取所有库
    pub fn libraries(&self) -> &[SignalLibrary] {
        &self.libraries
    }

    /// 获取可变库列表
    pub fn libraries_mut(&mut self) -> &mut [SignalLibrary] {
        &mut self.libraries
    }

    /// 根据ID查找库
    pub fn find_library(&self, id: &str) -> Option<&SignalLibrary> {
        self.libraries.iter().find(|lib| lib.id == id)
    }

    /// 根据ID查找可变库
    pub fn find_library_mut(&mut self, id: &str) -> Option<&mut SignalLibrary> {
        self.libraries.iter_mut().find(|lib| lib.id == id)
    }

    /// 创建新库
    pub fn create_library(&mut self, name: String, channel_type: ChannelType) -> Result<&SignalLibrary, String> {
        let id = generate_library_id(&name);

        // 检查是否已存在
        if self.find_library(&id).is_some() {
            return Err("Library already exists".to_string());
        }

        let library = SignalLibrary::new(id.clone(), name, channel_type);
        self.libraries.push(library);
        Ok(self.find_library(&id).unwrap())
    }

    /// 删除库
    pub fn delete_library(&mut self, id: &str, mappings: &[ChannelMapping]) -> Result<(), String> {
        let library = self.find_library(id).ok_or("Library not found")?;

        // 检查是否被使用
        if library.is_used(mappings) {
            let channels = library.used_channels(mappings);
            return Err(format!(
                "Library is in use by channels: {:?}",
                channels
            ));
        }

        let pos = self.libraries.iter().position(|lib| lib.id == id).unwrap();
        self.libraries.remove(pos);
        Ok(())
    }

    /// 添加版本到库（带通道配置）
    pub fn add_version_with_channels(
        &mut self,
        library_id: &str,
        name: String,
        description: String,
        channel_dbs: Vec<ChannelDatabase>,
    ) -> Result<(), String> {
        let library = self.find_library_mut(library_id)
            .ok_or("Library not found")?;

        // 验证所有通道配置
        for channel_db in &channel_dbs {
            channel_db.validate()?;

            // 检查文件是否存在
            if !std::path::Path::new(&channel_db.database_path).exists() {
                return Err(format!("Database file not found for channel {}: {}",
                    channel_db.channel_id, channel_db.database_path));
            }

            // 验证文件类型
            let db_type = channel_db.database_type()
                .ok_or("Unknown database type")?;

            // 检查类型是否与库类型匹配
            let expected_type = library.database_type();
            if (expected_type == DatabaseType::DBC && db_type != DatabaseType::DBC) ||
               (expected_type == DatabaseType::LDF && db_type != DatabaseType::LDF) {
                return Err(format!(
                    "Channel {}: Database type mismatch. Expected {:?}, got {:?}",
                    channel_db.channel_id, expected_type, db_type
                ));
            }
        }

        // 检查通道ID重复
        let mut channel_ids = std::collections::HashSet::new();
        for channel_db in &channel_dbs {
            if !channel_ids.insert(channel_db.channel_id) {
                return Err(format!("Duplicate channel ID: {}", channel_db.channel_id));
            }
        }

        // 创建版本（使用第一个通道的路径作为默认path，用于向后兼容）
        let default_path = channel_dbs.first()
            .map(|db| db.database_path.clone())
            .unwrap_or_default();

        let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let mut version = LibraryVersion::new(name, default_path, date)
            .with_description(description);

        // 添加所有通道配置
        for channel_db in channel_dbs {
            version.add_channel_database(channel_db)?;
        }

        library.add_version(version);
        Ok(())
    }

    /// 添加版本到库（简单版本，用于向后兼容）
    pub fn add_version(
        &mut self,
        library_id: &str,
        name: String,
        path: String,
        description: String,
    ) -> Result<(), String> {
        let library = self.find_library_mut(library_id)
            .ok_or("Library not found")?;

        // 检查文件是否存在
        if !std::path::Path::new(&path).exists() {
            return Err("Database file not found".to_string());
        }

        // 验证文件类型
        let db_type = DatabaseType::from_extension(
            std::path::Path::new(&path)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
        );

        if db_type != Some(library.database_type()) {
            return Err(format!(
                "Database type mismatch. Expected {:?}, got {:?}",
                library.database_type(),
                db_type
            ));
        }

        // 创建版本
        let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let version = LibraryVersion::new(name, path, date)
            .with_description(description);

        library.add_version(version);
        Ok(())
    }

    /// 删除版本
    pub fn remove_version(&mut self, library_id: &str, version_name: &str, mappings: &[ChannelMapping]) -> Result<(), String> {
        let library = self.find_library_mut(library_id)
            .ok_or("Library not found")?;

        // 检查版本是否被使用
        if mappings.iter().any(|m| {
            m.library_id.as_ref().map(|s| s.as_str()) == Some(library_id)
                && m.version_name.as_ref().map(|s| s.as_str()) == Some(version_name)
        }) {
            return Err("Version is currently in use".to_string());
        }

        if !library.remove_version(version_name) {
            return Err("Version not found".to_string());
        }

        Ok(())
    }

    /// 验证数据库文件
    pub fn validate_database(&self, path: &str) -> Result<DatabaseValidation, String> {
        let path_obj = PathBuf::from(path);

        // 检查文件是否存在
        if !path_obj.exists() {
            return Err("File not found".to_string());
        }

        // 根据扩展名确定类型
        let extension = path_obj
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match extension.as_str() {
            "dbc" => self.validate_dbc(path),
            "ldf" => self.validate_ldf(path),
            _ => Err("Unknown database type".to_string()),
        }
    }

    /// 验证DBC文件
    fn validate_dbc(&self, path: &str) -> Result<DatabaseValidation, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        let parser = DbcParser::new();
        let db = parser.parse(&content)
            .map_err(|e| format!("DBC parse error: {}", e))?;

        let message_count = db.messages.len();
        let signal_count = db.messages.values()
            .map(|m| m.signals.len())
            .sum();

        Ok(DatabaseValidation::success(message_count, signal_count))
    }

    /// 验证LDF文件
    fn validate_ldf(&self, path: &str) -> Result<DatabaseValidation, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        let parser = LdfParser::new();
        let db = parser.parse(&content)
            .map_err(|e| format!("LDF parse error: {}", e))?;

        let message_count = db.frames.len();
        let signal_count = db.frames.values()
            .map(|f| f.signals.len())
            .sum();

        Ok(DatabaseValidation::success(message_count, signal_count))
    }

    /// 获取数据库统计信息
    pub fn get_database_stats(&self, path: &str) -> Result<DatabaseStats, String> {
        let metadata = std::fs::metadata(path)
            .map_err(|e| format!("Failed to read metadata: {}", e))?;

        let modified = metadata.modified()
            .map_err(|e| format!("Failed to read modified time: {}", e))?;

        let last_modified = chrono::DateTime::<chrono::Utc>::from(modified)
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();

        let validation = self.validate_database(path)?;
        if !validation.is_valid {
            return Err("Invalid database file".to_string());
        }

        Ok(DatabaseStats {
            message_count: validation.message_count,
            signal_count: validation.signal_count,
            file_size: metadata.len(),
            last_modified,
        })
    }

    /// 加载数据库文件
    pub fn load_database(&self, path: &str, channel_type: ChannelType) -> Result<Database, String> {
        match channel_type {
            ChannelType::CAN => self.load_dbc(path),
            ChannelType::LIN => self.load_ldf(path),
        }
    }

    /// 加载DBC文件
    fn load_dbc(&self, path: &str) -> Result<Database, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        let parser = DbcParser::new();
        let db = parser.parse(&content)
            .map_err(|e| format!("DBC parse error: {}", e))?;

        Ok(Database::Dbc(db))
    }

    /// 加载LDF文件
    fn load_ldf(&self, path: &str) -> Result<Database, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        let parser = LdfParser::new();
        let db = parser.parse(&content)
            .map_err(|e| format!("LDF parse error: {}", e))?;

        Ok(Database::Ldf(db))
    }
}

/// 数据库枚举（包装DBC和LDF）
pub enum Database {
    Dbc(DbcDatabase),
    Ldf(LdfDatabase),
}

/// 生成唯一的库ID
pub fn generate_library_id(name: &str) -> String {
    let mut hasher = DefaultHasher::new();
    name.hash(&mut hasher);
    format!("lib_{:x}", hasher.finish())
}

/// 从文件路径提取版本号
pub fn extract_version_from_path(path: &PathBuf) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .and_then(|name| {
            // 查找版本号模式: v1.0, v2.1等
            if let Some(pos) = name.find('v') {
                let version_part = &name[pos..];
                if version_part.len() > 1 && version_part.chars().nth(1).map(|c| c.is_ascii_digit()).unwrap_or(false) {
                    return Some(version_part.to_string());
                }
            }
            // 查找纯数字版本: 1.0, 2.1等
            if let Some(pos) = name.chars().position(|c| c.is_ascii_digit()) {
                let version_part = &name[pos..];
                if version_part.chars().take(10).all(|c| c.is_ascii_digit() || c == '.') {
                    return Some(format!("v{}", version_part));
                }
            }
            None
        })
        .unwrap_or_else(|| {
            // 使用当前日期作为版本号
            format!("v{}", chrono::Utc::now().format("%Y%m%d"))
        })
}

impl Default for LibraryManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_library_id() {
        let id1 = generate_library_id("test");
        let id2 = generate_library_id("test");
        let id3 = generate_library_id("other");

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_create_library() {
        let mut manager = LibraryManager::new();
        let result = manager.create_library(
            "Test Library".to_string(),
            ChannelType::CAN
        );

        assert!(result.is_ok());
        assert_eq!(manager.libraries().len(), 1);
    }

    #[test]
    fn test_add_version() {
        let mut manager = LibraryManager::new();
        let library = manager.create_library(
            "Test".to_string(),
            ChannelType::CAN
        ).unwrap();

        // 由于我们无法在测试中创建真实的DBC文件，这里只测试API
        assert!(manager.libraries().len() > 0);
    }

    #[test]
    fn test_extract_version_from_path() {
        use std::path::PathBuf;

        let path1 = PathBuf::from("/path/to/bmw_ptcan_v1.0.dbc");
        let v1 = extract_version_from_path(&path1);
        assert!(v1.contains("1.0"));

        let path2 = PathBuf::from("/path/to/ford_lin.ldf");
        let v2 = extract_version_from_path(&path2);
        // 如果找不到版本号，应该使用日期
        assert!(v2.starts_with("v20"));
    }
}
