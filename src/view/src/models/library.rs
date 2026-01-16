//! Signal library data models
//!
//! 提供信号库的数据结构定义，包括库、版本和通道映射

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 数据库文件类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DatabaseType {
    DBC,
    LDF,
}

impl DatabaseType {
    pub fn extension(&self) -> &str {
        match self {
            DatabaseType::DBC => "dbc",
            DatabaseType::LDF => "ldf",
        }
    }

    pub fn icon(&self) -> &str {
        match self {
            DatabaseType::DBC => "📋",
            DatabaseType::LDF => "🔗",
        }
    }

    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "dbc" => Some(DatabaseType::DBC),
            "ldf" => Some(DatabaseType::LDF),
            _ => None,
        }
    }
}

/// 信号库版本
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LibraryVersion {
    /// 版本名称/号（如 "v1.0", "v2.1"）
    pub name: String,
    /// 数据库文件路径
    pub path: String,
    /// 创建日期
    pub date: String,
    /// 版本描述
    #[serde(default)]
    pub description: String,
    /// 每个通道的数据库文件映射
    /// Key: channel_id, Value: database file path
    #[serde(default)]
    pub channel_databases: HashMap<u16, String>,
}

impl LibraryVersion {
    /// 创建新版本
    pub fn new(name: String, path: String, date: String) -> Self {
        Self {
            name,
            path,
            date,
            description: String::new(),
            channel_databases: HashMap::new(),
        }
    }

    /// 设置描述
    pub fn with_description(mut self, description: String) -> Self {
        self.description = description;
        self
    }

    /// 添加通道数据库映射
    pub fn add_channel_database(&mut self, channel_id: u16, db_path: String) {
        self.channel_databases.insert(channel_id, db_path);
    }

    /// 获取通道的数据库路径
    pub fn get_channel_database(&self, channel_id: u16) -> Option<&String> {
        self.channel_databases.get(&channel_id)
            .or_else(|| if !self.path.is_empty() { Some(&self.path) } else { None })
    }
}

/// 信号库
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SignalLibrary {
    /// 库的唯一标识符
    pub id: String,
    /// 库名称
    pub name: String,
    /// 库的类型（CAN/LIN）
    #[serde(default = "default_channel_type")]
    pub channel_type: super::ChannelType,
    /// 版本列表（按版本号降序排列，最新版本在前）
    pub versions: Vec<LibraryVersion>,
}

fn default_channel_type() -> super::ChannelType {
    super::ChannelType::CAN
}

impl SignalLibrary {
    /// 创建新库
    pub fn new(id: String, name: String, channel_type: super::ChannelType) -> Self {
        Self {
            id,
            name,
            channel_type,
            versions: Vec::new(),
        }
    }

    /// 获取最新版本
    pub fn latest_version(&self) -> Option<&LibraryVersion> {
        self.versions.first()
    }

    /// 获取指定名称的版本
    pub fn get_version(&self, name: &str) -> Option<&LibraryVersion> {
        self.versions.iter().find(|v| v.name == name)
    }

    /// 获取可变版本引用
    pub fn get_version_mut(&mut self, name: &str) -> Option<&mut LibraryVersion> {
        self.versions.iter_mut().find(|v| v.name == name)
    }

    /// 获取数据库类型
    pub fn database_type(&self) -> DatabaseType {
        match self.channel_type {
            super::ChannelType::CAN => DatabaseType::DBC,
            super::ChannelType::LIN => DatabaseType::LDF,
        }
    }

    /// 添加新版本
    pub fn add_version(&mut self, version: LibraryVersion) {
        // 检查是否已存在同名版本
        if !self.versions.iter().any(|v| v.name == version.name) {
            self.versions.push(version);
            self.sort_versions();
        }
    }

    /// 移除版本
    pub fn remove_version(&mut self, name: &str) -> bool {
        if let Some(pos) = self.versions.iter().position(|v| v.name == name) {
            self.versions.remove(pos);
            true
        } else {
            false
        }
    }

    /// 对版本进行排序（按版本号降序）
    pub fn sort_versions(&mut self) {
        self.versions.sort_by(|a, b| {
            let v_a = extract_version_number(&a.name);
            let v_b = extract_version_number(&b.name);
            v_b.partial_cmp(&v_a).unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    /// 检查库是否被使用
    pub fn is_used(&self, mappings: &[super::ChannelMapping]) -> bool {
        mappings.iter().any(|m| m.library_id.as_ref() == Some(&self.id))
    }

    /// 获取使用此库的通道列表
    pub fn used_channels(&self, mappings: &[super::ChannelMapping]) -> Vec<u16> {
        mappings.iter()
            .filter(|m| m.library_id.as_ref() == Some(&self.id))
            .map(|m| m.channel_id)
            .collect()
    }
}

/// 从版本字符串中提取数字用于排序
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
    fn test_library_version_sorting() {
        let mut lib = SignalLibrary::new(
            "test".to_string(),
            "Test Library".to_string(),
            super::ChannelType::CAN,
        );

        lib.add_version(LibraryVersion::new("v1.0".to_string(), "/path1".to_string(), "2024-01-01".to_string()));
        lib.add_version(LibraryVersion::new("v2.0".to_string(), "/path2".to_string(), "2024-01-02".to_string()));
        lib.add_version(LibraryVersion::new("v1.5".to_string(), "/path3".to_string(), "2024-01-03".to_string()));

        // Should be sorted: v2.0, v1.5, v1.0
        assert_eq!(lib.versions[0].name, "v2.0");
        assert_eq!(lib.versions[1].name, "v1.5");
        assert_eq!(lib.versions[2].name, "v1.0");
    }

    #[test]
    fn test_database_type() {
        assert_eq!(DatabaseType::from_extension("dbc"), Some(DatabaseType::DBC));
        assert_eq!(DatabaseType::from_extension("ldf"), Some(DatabaseType::LDF));
        assert_eq!(DatabaseType::from_extension("xyz"), None);
    }
}
