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

/// 通道数据库配置
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChannelDatabase {
    /// 通道类型 (CAN/LIN)
    #[serde(default)]
    pub channel_type: crate::models::ChannelType,
    /// 通道ID
    pub channel_id: u16,
    /// 通道名称
    pub channel_name: String,
    /// 数据库文件路径
    pub database_path: String,
}

impl ChannelDatabase {
    /// 创建新的通道数据库配置
    pub fn new(
        channel_type: crate::models::ChannelType,
        channel_id: u16,
        channel_name: String,
        database_path: String,
    ) -> Self {
        Self {
            channel_type,
            channel_id,
            channel_name,
            database_path,
        }
    }

    /// 验证通道配置
    pub fn validate(&self) -> Result<(), String> {
        // 检查通道ID是否有效（1-255）
        if self.channel_id == 0 || self.channel_id > 255 {
            return Err(format!(
                "Invalid channel ID: {}. Must be between 1 and 255",
                self.channel_id
            ));
        }

        // 检查通道名称是否为空
        if self.channel_name.trim().is_empty() {
            return Err("Channel name cannot be empty".to_string());
        }

        // 检查数据库文件路径是否为空
        if self.database_path.trim().is_empty() {
            return Err("Database path cannot be empty".to_string());
        }

        Ok(())
    }

    /// 获取数据库类型
    pub fn database_type(&self) -> Option<DatabaseType> {
        std::path::Path::new(&self.database_path)
            .extension()
            .and_then(|e| e.to_str())
            .and_then(|ext| DatabaseType::from_extension(ext))
    }
}

/// 信号库版本
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LibraryVersion {
    /// 版本名称/号（如 "v1.0", "v2.1"）
    pub name: String,
    /// 默认数据库文件路径（向后兼容）
    pub path: String,
    /// 创建日期
    pub date: String,
    /// 版本描述
    #[serde(default)]
    pub description: String,
    /// 每个通道的数据库文件配置列表
    /// 存储结构：按通道类型分组的配置
    #[serde(default)]
    pub channel_databases: Vec<ChannelDatabase>,
}

impl LibraryVersion {
    /// 创建新版本
    pub fn new(name: String, path: String, date: String) -> Self {
        Self {
            name,
            path,
            date,
            description: String::new(),
            channel_databases: Vec::new(),
        }
    }

    /// 设置描述
    pub fn with_description(mut self, description: String) -> Self {
        self.description = description;
        self
    }

    /// 添加通道数据库配置
    pub fn add_channel_database(&mut self, channel_db: ChannelDatabase) -> Result<(), String> {
        // 验证通道配置
        channel_db.validate()?;

        // 检查通道ID是否已存在
        if self
            .channel_databases
            .iter()
            .any(|db| db.channel_id == channel_db.channel_id)
        {
            return Err(format!(
                "Channel ID {} already exists in this version",
                channel_db.channel_id
            ));
        }

        self.channel_databases.push(channel_db);
        Ok(())
    }

    /// 获取指定通道的数据库配置
    pub fn get_channel_database(&self, channel_id: u16) -> Option<&ChannelDatabase> {
        self.channel_databases
            .iter()
            .find(|db| db.channel_id == channel_id)
    }

    /// 获取所有CAN通道
    pub fn get_can_channels(&self) -> Vec<&ChannelDatabase> {
        self.channel_databases
            .iter()
            .filter(|db| db.database_type() == Some(DatabaseType::DBC))
            .collect()
    }

    /// 获取所有LIN通道
    pub fn get_lin_channels(&self) -> Vec<&ChannelDatabase> {
        self.channel_databases
            .iter()
            .filter(|db| db.database_type() == Some(DatabaseType::LDF))
            .collect()
    }

    /// 获取通道数据库列表（用于向后兼容）
    pub fn get_channel_map(&self) -> HashMap<u16, String> {
        let mut map = HashMap::new();
        for db in &self.channel_databases {
            map.insert(db.channel_id, db.database_path.clone());
        }
        // 如果没有配置通道数据库，使用默认path
        if map.is_empty() && !self.path.is_empty() {
            // 尝试推断通道ID
            if let Some(1) = Some(1) {
                map.insert(1, self.path.clone());
            }
        }
        map
    }

    /// 检查通道ID是否已被使用
    pub fn is_channel_id_used(&self, channel_id: u16) -> bool {
        self.channel_databases
            .iter()
            .any(|db| db.channel_id == channel_id)
    }

    /// 获取已使用的通道ID列表
    pub fn get_used_channel_ids(&self) -> Vec<u16> {
        self.channel_databases
            .iter()
            .map(|db| db.channel_id)
            .collect()
    }

    /// 获取版本统计信息
    pub fn get_stats(&self) -> VersionStats {
        let can_count = self.get_can_channels().len();
        let lin_count = self.get_lin_channels().len();

        VersionStats {
            total_channels: self.channel_databases.len(),
            can_channels: can_count,
            lin_channels: lin_count,
        }
    }
}

/// 版本统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionStats {
    pub total_channels: usize,
    pub can_channels: usize,
    pub lin_channels: usize,
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
        mappings
            .iter()
            .any(|m| m.library_id.as_ref() == Some(&self.id))
    }

    /// 获取使用此库的通道列表
    pub fn used_channels(&self, mappings: &[super::ChannelMapping]) -> Vec<u16> {
        mappings
            .iter()
            .filter(|m| m.library_id.as_ref() == Some(&self.id))
            .map(|m| m.channel_id)
            .collect()
    }

    /// 获取当前激活的版本名称
    pub fn active_version_name(&self, mappings: &[super::ChannelMapping]) -> Option<String> {
        mappings
            .iter()
            .filter(|m| m.library_id.as_ref() == Some(&self.id))
            .find_map(|m| m.version_name.clone())
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

        lib.add_version(LibraryVersion::new(
            "v1.0".to_string(),
            "/path1".to_string(),
            "2024-01-01".to_string(),
        ));
        lib.add_version(LibraryVersion::new(
            "v2.0".to_string(),
            "/path2".to_string(),
            "2024-01-02".to_string(),
        ));
        lib.add_version(LibraryVersion::new(
            "v1.5".to_string(),
            "/path3".to_string(),
            "2024-01-03".to_string(),
        ));

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
