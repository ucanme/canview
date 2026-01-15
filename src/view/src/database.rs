// src/view/src/database.rs
//! 数据库验证和版本管理模块
//!
//! 提供DBC/LDF文件的验证、解析和版本管理功能

use std::path::PathBuf;
use parser::dbc::DbcParser;
use parser::ldf::LdfParser;
use serde::{Deserialize, Serialize};

/// 数据库文件类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DatabaseType {
    DBC,
    LDF,
}

impl DatabaseType {
    /// 获取文件扩展名
    pub fn extension(&self) -> &str {
        match self {
            DatabaseType::DBC => "dbc",
            DatabaseType::LDF => "ldf",
        }
    }

    /// 获取UI显示图标
    pub fn icon(&self) -> &str {
        match self {
            DatabaseType::DBC => "📋",
            DatabaseType::LDF => "🔗",
        }
    }

    /// 从文件路径推断类型
    pub fn from_path(path: &PathBuf) -> Option<Self> {
        path.extension()
            .and_then(|e| e.to_str())
            .map(|ext| match ext.to_lowercase().as_str() {
                "dbc" => DatabaseType::DBC,
                "ldf" => DatabaseType::LDF,
                _ => DatabaseType::DBC, // 默认
            })
    }
}

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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseStats {
    pub message_count: usize,
    pub signal_count: usize,
    pub file_size: u64,
    pub last_modified: String,
}

/// 版本信息扩展
pub trait VersionExt {
    /// 验证数据库文件
    fn validate(&self) -> Result<DatabaseValidation, String>;

    /// 获取数据库类型
    fn database_type(&self) -> DatabaseType;

    /// 检查文件是否存在
    fn file_exists(&self) -> bool;

    /// 获取文件大小
    fn file_size(&self) -> Option<u64>;

    /// 提取版本号
    fn extract_version(&self) -> String;
}

/// 为 LibraryVersion 实现 VersionExt (需要在 main.rs 中调用)
impl VersionExt for crate::LibraryVersion {
    fn validate(&self) -> Result<DatabaseValidation, String> {
        let path = PathBuf::from(&self.path);

        // 检查文件是否存在
        if !path.exists() {
            return Err(format!("File not found: {}", self.path));
        }

        // 根据扩展名解析
        let db_type = DatabaseType::from_path(&path);

        match db_type {
            Some(DatabaseType::DBC) => self.validate_dbc(),
            Some(DatabaseType::LDF) => self.validate_ldf(),
            None => Err("Unknown database type".to_string()),
        }
    }

    fn database_type(&self) -> DatabaseType {
        DatabaseType::from_path(&PathBuf::from(&self.path))
            .unwrap_or(DatabaseType::DBC)
    }

    fn file_exists(&self) -> bool {
        PathBuf::from(&self.path).exists()
    }

    fn file_size(&self) -> Option<u64> {
        std::fs::metadata(&self.path)
            .ok()
            .map(|m| m.len())
    }

    fn extract_version(&self) -> String {
        // 尝试从name字段提取版本号
        // 如果name已经是版本号(如 "v1.0"),直接返回
        if self.name.starts_with('v') || self.name.contains('.') {
            return self.name.clone();
        }

        // 否则尝试从路径提取
        PathBuf::from(&self.path)
            .file_stem()
            .and_then(|s| s.to_str())
            .and_then(|stem| {
                // 查找版本模式: v1.0, 2.1等
                if let Some(pos) = stem.find('v') {
                    let version_part = &stem[pos..];
                    if version_part.len() > 1 && version_part.chars().nth(1).map(|c| c.is_ascii_digit()).unwrap_or(false) {
                        return Some(version_part.to_string());
                    }
                }
                // 查找纯数字版本: 1.0, 2.1等
                if let Some(pos) = stem.chars().position(|c| c.is_ascii_digit()) {
                    let version_part = &stem[pos..];
                    if version_part.chars().take(10).all(|c| c.is_ascii_digit() || c == '.') {
                        return Some(format!("v{}", version_part));
                    }
                }
                None
            })
            .unwrap_or_else(|| self.name.clone())
    }
}

impl crate::LibraryVersion {
    /// 验证DBC文件
    pub fn validate_dbc(&self) -> Result<DatabaseValidation, String> {
        let content = std::fs::read_to_string(&self.path)
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
    pub fn validate_ldf(&self) -> Result<DatabaseValidation, String> {
        let content = std::fs::read_to_string(&self.path)
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
}

/// 信号库扩展
pub trait LibraryExt {
    /// 获取最新版本
    fn latest_version(&self) -> Option<&crate::LibraryVersion>;

    /// 获取指定名称的版本
    fn get_version(&self, name: &str) -> Option<&crate::LibraryVersion>;

    /// 获取数据库类型
    fn database_type(&self) -> DatabaseType;

    /// 检查库是否被使用
    fn is_used(&self, mappings: &[crate::ChannelMapping]) -> bool;

    /// 获取使用此库的通道列表
    fn used_channels(&self, mappings: &[crate::ChannelMapping]) -> Vec<u16>;

    /// 获取激活的版本名称
    fn active_version_name(&self, mappings: &[crate::ChannelMapping]) -> Option<String>;

    /// 添加新版本
    fn add_version(&mut self, name: String, path: String, date: String);

    /// 移除版本
    fn remove_version(&mut self, name: &str) -> bool;

    /// 对版本进行排序
    fn sort_versions(&mut self);
}

/// 为 SignalLibrary 实现 LibraryExt
impl LibraryExt for crate::SignalLibrary {
    fn latest_version(&self) -> Option<&crate::LibraryVersion> {
        self.versions.first()
    }

    fn get_version(&self, name: &str) -> Option<&crate::LibraryVersion> {
        self.versions.iter().find(|v| v.name == name)
    }

    fn database_type(&self) -> DatabaseType {
        self.versions.first()
            .map(|v| v.database_type())
            .unwrap_or(DatabaseType::DBC)
    }

    fn is_used(&self, mappings: &[crate::ChannelMapping]) -> bool {
        mappings.iter().any(|m| m.library_id.as_ref() == Some(&self.id))
    }

    fn used_channels(&self, mappings: &[crate::ChannelMapping]) -> Vec<u16> {
        mappings.iter()
            .filter(|m| m.library_id.as_ref() == Some(&self.id))
            .map(|m| m.channel_id)
            .collect()
    }

    fn active_version_name(&self, mappings: &[crate::ChannelMapping]) -> Option<String> {
        mappings.iter()
            .find(|m| m.library_id.as_ref() == Some(&self.id))
            .and_then(|m| m.version_name.clone())
    }

    fn add_version(&mut self, name: String, path: String, date: String) {
        let version = crate::LibraryVersion {
            name,
            path,
            date,
        };

        // 检查是否已存在同名版本
        if !self.versions.iter().any(|v| v.name == version.name) {
            self.versions.push(version);
            self.sort_versions();
        }
    }

    fn remove_version(&mut self, name: &str) -> bool {
        if let Some(pos) = self.versions.iter().position(|v| v.name == name) {
            self.versions.remove(pos);
            true
        } else {
            false
        }
    }

    fn sort_versions(&mut self) {
        // 按版本号排序(降序 - 最新版本在前)
        self.versions.sort_by(|a, b| {
            // 尝试解析版本号进行比较
            let v_a = extract_version_number(&a.name);
            let v_b = extract_version_number(&b.name);
            v_b.partial_cmp(&v_a).unwrap_or(std::cmp::Ordering::Equal)
        });
    }
}

/// 从版本字符串中提取数字用于排序
/// 例如: "v1.2" -> 1.02, "v2.0" -> 2.0
fn extract_version_number(version: &str) -> f64 {
    let cleaned = version
        .trim_start_matches('v')
        .trim_start_matches('V')
        .replace('_', ".");

    let parts: Vec<&str> = cleaned.split('.').collect();
    if parts.is_empty() {
        return 0.0;
    }

    let major = parts[0].parse::<f64>().unwrap_or(0.0);
    let minor = if parts.len() > 1 {
        parts[1].parse::<f64>().unwrap_or(0.0) / 100.0
    } else {
        0.0
    };

    major + minor
}

/// 通道映射扩展
pub trait MappingExt {
    /// 获取关联的库
    fn get_library<'a>(&self, libraries: &'a [crate::SignalLibrary]) -> Option<&'a crate::SignalLibrary>;

    /// 获取关联的版本
    fn get_version<'a>(&self, libraries: &'a [crate::SignalLibrary]) -> Option<&'a crate::LibraryVersion>;

    /// 设置库和版本
    fn set_library(&mut self, library_id: String, version_name: String);

    /// 获取显示名称
    fn display_name(&self, libraries: &[crate::SignalLibrary]) -> String;
}

impl MappingExt for crate::ChannelMapping {
    fn get_library<'a>(&self, libraries: &'a [crate::SignalLibrary]) -> Option<&'a crate::SignalLibrary> {
        let id = self.library_id.as_ref()?;
        libraries.iter().find(|lib| lib.id == *id)
    }

    fn get_version<'a>(&self, libraries: &'a [crate::SignalLibrary]) -> Option<&'a crate::LibraryVersion> {
        let lib = self.get_library(libraries)?;
        let version_name = self.version_name.as_ref()?;
        lib.get_version(version_name)
    }

    fn set_library(&mut self, library_id: String, version_name: String) {
        self.library_id = Some(library_id);
        self.version_name = Some(version_name);
    }

    fn display_name(&self, libraries: &[crate::SignalLibrary]) -> String {
        if let Some(lib) = self.get_library(libraries) {
            if let Some(version) = self.get_version(libraries) {
                return format!("{} v{}", lib.name, version.name);
            } else {
                return lib.name.clone();
            }
        }
        self.path.clone()
    }
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

/// 生成唯一的库ID
pub fn generate_library_id(name: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    name.hash(&mut hasher);
    format!("lib_{:x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_version_number() {
        assert_eq!(extract_version_number("v1.0"), 1.0);
        assert_eq!(extract_version_number("v2.1"), 2.01);
        assert_eq!(extract_version_number("v10.5"), 10.05);
        assert_eq!(extract_version_number("1.0"), 1.0);
        assert_eq!(extract_version_number("invalid"), 0.0);
    }

    #[test]
    fn test_database_type_from_path() {
        let dbc_path = PathBuf::from("/test/file.dbc");
        let ldf_path = PathBuf::from("/test/file.ldf");

        assert_eq!(DatabaseType::from_path(&dbc_path), Some(DatabaseType::DBC));
        assert_eq!(DatabaseType::from_path(&ldf_path), Some(DatabaseType::LDF));
    }

    #[test]
    fn test_extract_version_from_path() {
        let path1 = PathBuf::from("/path/to/bmw_ptcan_v1.0.dbc");
        let path2 = PathBuf::from("/path/to/ford_lin.ldf");

        let v1 = extract_version_from_path(&path1);
        let v2 = extract_version_from_path(&path2);

        assert!(v1.contains("1.0"));
        assert!(v2.starts_with("v20")); // 包含日期
    }
}
