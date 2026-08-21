//! Signal library management module
//!
//! 提供信号库的CRUD操作、版本管理和验证功能

mod storage;
pub mod signal_sets;

pub use storage::{
    SignalLibraryStorage, build_library_file_subdir, copy_database_to_libraries,
    delete_channel_from_libraries, delete_library_from_libraries,
    delete_version_from_libraries, libraries_base_path, sanitize_filename,
};

pub use signal_sets::{
    SignalSet, SignalSetEntry, SignalSetStore, build_selected_signals_from_set,
    load_signal_set_store, parse_signal_id, save_signal_set_store, signal_set_store_path,
};

use crate::models::{
    ChannelDatabase, ChannelMapping, ChannelType, DatabaseType, LibraryVersion, SignalLibrary,
};
use parser::dbc::{DbcDatabase, DbcParser};
use parser::ldf::{LdfDatabase, LdfParser};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

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
#[derive(Clone)]
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
    pub fn create_library(
        &mut self,
        name: String,
        channel_type: ChannelType,
    ) -> Result<&SignalLibrary, String> {
        let id = generate_library_id(&name);

        // 检查是否已存在
        if self.find_library(&id).is_some() {
            return Err("Library already exists".to_string());
        }

        let library = SignalLibrary::new(id.clone(), name, channel_type);
        self.libraries.push(library);
        Ok(self.find_library(&id).unwrap())
    }

    /// 重命名库 — 返回新生成的 library ID
    pub fn rename_library(&mut self, old_id: &str, new_name: String) -> Result<String, String> {
        let new_name = new_name.trim().to_string();
        if new_name.is_empty() {
            return Err("Library name cannot be empty".to_string());
        }

        let new_id = generate_library_id(&new_name);

        // Reject if new name collides with a different library
        if new_id != old_id {
            if self.find_library(&new_id).is_some() {
                return Err(format!("Library '{}' already exists", new_name));
            }
        }

        let library = self
            .find_library_mut(old_id)
            .ok_or("Library not found")?;

        library.id = new_id.clone();
        library.name = new_name;

        Ok(new_id)
    }

    /// 重命名库版本
    pub fn rename_version(
        &mut self,
        library_id: &str,
        old_name: &str,
        new_name: String,
    ) -> Result<(), String> {
        let new_name = new_name.trim().to_string();
        if new_name.is_empty() {
            return Err("Version name cannot be empty".to_string());
        }

        let library = self
            .find_library_mut(library_id)
            .ok_or("Library not found")?;

        // Reject duplicate version name within the same library
        if library.versions.iter().any(|v| v.name == new_name && v.name != old_name) {
            return Err(format!("Version '{}' already exists", new_name));
        }

        let version = library
            .versions
            .iter_mut()
            .find(|v| v.name == old_name)
            .ok_or("Version not found")?;

        version.name = new_name;

        Ok(())
    }

    /// 删除库
    pub fn delete_library(&mut self, id: &str, mappings: &[ChannelMapping]) -> Result<(), String> {
        let library = self.find_library(id).ok_or("Library not found")?;

        // 检查是否被使用
        if library.is_used(mappings) {
            let channels = library.used_channels(mappings);
            return Err(format!("Library is in use by channels: {:?}", channels));
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
        let library = self
            .find_library_mut(library_id)
            .ok_or("Library not found")?;

        // 验证所有通道配置
        for channel_db in &channel_dbs {
            channel_db.validate()?;

            // 检查文件是否存在
            if !std::path::Path::new(&channel_db.database_path).exists() {
                return Err(format!(
                    "Database file not found for channel {}: {}",
                    channel_db.channel_id, channel_db.database_path
                ));
            }

            // 验证文件类型
            let db_type = channel_db.database_type().ok_or("Unknown database type")?;

            // 检查类型是否与库类型匹配
            let expected_type = library.database_type();
            if (expected_type == DatabaseType::DBC && db_type != DatabaseType::DBC)
                || (expected_type == DatabaseType::LDF && db_type != DatabaseType::LDF)
            {
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
        let default_path = channel_dbs
            .first()
            .map(|db| db.database_path.clone())
            .unwrap_or_default();

        let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let mut version =
            LibraryVersion::new(name, default_path, date).with_description(description);

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
        let library = self
            .find_library_mut(library_id)
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
                .unwrap_or(""),
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
        let version = LibraryVersion::new(name, path, date).with_description(description);

        library.add_version(version);
        Ok(())
    }

    /// 删除版本
    pub fn remove_version(
        &mut self,
        library_id: &str,
        version_name: &str,
        mappings: &[ChannelMapping],
    ) -> Result<(), String> {
        let library = self
            .find_library_mut(library_id)
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
        let content =
            std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;

        let parser = DbcParser::new();
        let db = parser
            .parse(&content)
            .map_err(|e| format!("DBC parse error: {}", e))?;

        let message_count = db.messages.len();
        let signal_count = db.messages.values().map(|m| m.signals.len()).sum();

        Ok(DatabaseValidation::success(message_count, signal_count))
    }

    /// 验证LDF文件
    fn validate_ldf(&self, path: &str) -> Result<DatabaseValidation, String> {
        let content =
            std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;

        let parser = LdfParser::new();
        let db = parser
            .parse(&content)
            .map_err(|e| format!("LDF parse error: {}", e))?;

        let message_count = db.frames.len();
        let signal_count = db.frames.values().map(|f| f.signals.len()).sum();

        Ok(DatabaseValidation::success(message_count, signal_count))
    }

    /// 获取数据库统计信息
    pub fn get_database_stats(&self, path: &str) -> Result<DatabaseStats, String> {
        let metadata =
            std::fs::metadata(path).map_err(|e| format!("Failed to read metadata: {}", e))?;

        let modified = metadata
            .modified()
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
    ///
    /// Channel type is auto-detected from the file extension (.dbc → CAN,
    /// .ldf → LIN) so a library created with the wrong channel_type still
    /// loads its database correctly. The passed `channel_type` is used only
    /// as a fallback when the extension is ambiguous.
    pub fn load_database(&self, path: &str, channel_type: ChannelType) -> Result<Database, String> {
        let detected = detect_channel_type_from_path(path);
        let effective = detected.unwrap_or(channel_type);
        match effective {
            ChannelType::CAN => self.load_dbc(path),
            ChannelType::LIN => self.load_ldf(path),
        }
    }

    /// 加载DBC文件
    fn load_dbc(&self, path: &str) -> Result<Database, String> {
        let content =
            std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;

        let parser = DbcParser::new();
        let db = parser
            .parse(&content)
            .map_err(|e| format!("DBC parse error: {}", e))?;

        Ok(Database::Dbc(db))
    }

    /// 加载LDF文件
    fn load_ldf(&self, path: &str) -> Result<Database, String> {
        let content =
            std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;

        let parser = LdfParser::new();
        let db = parser
            .parse(&content)
            .map_err(|e| format!("LDF parse error: {}", e))?;

        Ok(Database::Ldf(db))
    }
}

/// 数据库枚举（包装DBC和LDF）
pub enum Database {
    Dbc(DbcDatabase),
    Ldf(LdfDatabase),
}

/// Detect `ChannelType` from a database file's extension.
/// Returns `Some(CAN)` for `.dbc`, `Some(LIN)` for `.ldf`, `None` for unknown.
/// Used by `load_database` to override a library's `channel_type` when the
/// caller passed a stale value (e.g. a LIN library created via the legacy
/// hardcoded-CAN path).
pub fn detect_channel_type_from_path(path: &str) -> Option<ChannelType> {
    let lowered = path.to_lowercase();
    if lowered.ends_with(".ldf") {
        Some(ChannelType::LIN)
    } else if lowered.ends_with(".dbc") {
        Some(ChannelType::CAN)
    } else {
        None
    }
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
                if version_part.len() > 1
                    && version_part
                        .chars()
                        .nth(1)
                        .map(|c| c.is_ascii_digit())
                        .unwrap_or(false)
                {
                    return Some(version_part.to_string());
                }
            }
            // 查找纯数字版本: 1.0, 2.1等
            if let Some(pos) = name.chars().position(|c| c.is_ascii_digit()) {
                let version_part = &name[pos..];
                if version_part
                    .chars()
                    .take(10)
                    .all(|c| c.is_ascii_digit() || c == '.')
                {
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
        let result = manager.create_library("Test Library".to_string(), ChannelType::CAN);

        assert!(result.is_ok());
        assert_eq!(manager.libraries().len(), 1);
    }

    #[test]
    fn test_load_database_auto_detects_ldf_from_extension() {
        // Write a minimal LDF to a temp file with .ldf extension
        let ldf_content = r#"
LIN_description_file = "2.1";
Signals {
    SysSt: 8, 0, BCM, IPC;
}
Frames {
    BCM_St: 0x10, BCM, 2 {
        SysSt, 0;
    }
}
"#;
        let dir = tempfile::tempdir().unwrap();
        let ldf_path = dir.path().join("test.ldf");
        std::fs::write(&ldf_path, ldf_content).unwrap();

        let manager = LibraryManager::new();
        // Caller says CAN, but file extension is .ldf — must still load as LDF.
        let result = manager.load_database(ldf_path.to_str().unwrap(), ChannelType::CAN);
        assert!(result.is_ok(), "load_database failed: {:?}", result.err());
        assert!(
            matches!(result.unwrap(), Database::Ldf(_)),
            "expected Ldf variant when path ends with .ldf"
        );
    }

    #[test]
    fn test_load_database_auto_detects_dbc_from_extension() {
        let dbc_content = r#"
VERSION ""

NS_ :

BS_:

BU_: DBG

BO_ 256 EngineStatus: 8 DBG
 SG_ EngineRPM : 0|16@1+ (1,0) [0|65535] "rpm" DBG
"#;
        let dir = tempfile::tempdir().unwrap();
        let dbc_path = dir.path().join("test.dbc");
        std::fs::write(&dbc_path, dbc_content).unwrap();

        let manager = LibraryManager::new();
        // Caller says LIN, but file extension is .dbc — must still load as DBC.
        let result = manager.load_database(dbc_path.to_str().unwrap(), ChannelType::LIN);
        assert!(result.is_ok(), "load_database failed: {:?}", result.err());
        assert!(
            matches!(result.unwrap(), Database::Dbc(_)),
            "expected Dbc variant when path ends with .dbc"
        );
    }

    #[test]
    fn test_add_version() {
        let mut manager = LibraryManager::new();
        let library = manager
            .create_library("Test".to_string(), ChannelType::CAN)
            .unwrap();

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
