use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub fn libraries_base_path(config_file_path: Option<&Path>) -> PathBuf {
    if let Some(dir) = config_file_path.and_then(|p| p.parent()) {
        return dir.join("libraries");
    }
    // Fall back to a path relative to the executable directory so that
    // the libraries folder is always found regardless of the working directory.
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            return exe_dir.join("libraries");
        }
    }
    PathBuf::from("libraries")
}

pub fn build_library_file_subdir(
    library_name: &str,
    version_name: &str,
    channel_name: &str,
) -> PathBuf {
    PathBuf::from(sanitize_filename(library_name))
        .join(sanitize_filename(version_name))
        .join(sanitize_filename(channel_name))
}

pub fn copy_database_to_libraries(
    base_dir: &Path,
    library_name: &str,
    version_name: &str,
    channel_name: &str,
    source_path: &Path,
) -> Result<PathBuf> {
    let target_dir = base_dir.join(build_library_file_subdir(
        library_name,
        version_name,
        channel_name,
    ));
    fs::create_dir_all(&target_dir)
        .context(format!("Failed to create target directory: {:?}", target_dir))?;

    let extension = source_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("dbc");

    let file_name = source_path
        .file_name()
        .and_then(|n| n.to_str())
        .map(sanitize_filename)
        .unwrap_or_else(|| format!("database.{}", extension));

    let dest_path = target_dir.join(file_name);
    fs::copy(source_path, &dest_path).context(format!(
        "Failed to copy database from {:?} to {:?}",
        source_path, dest_path
    ))?;

    Ok(dest_path)
}

pub fn delete_library_from_libraries(base_dir: &Path, library_name: &str) -> Result<()> {
    let library_dir = base_dir.join(sanitize_filename(library_name));
    if library_dir.exists() {
        fs::remove_dir_all(&library_dir)
            .context(format!("Failed to delete library directory: {:?}", library_dir))?;
    }
    Ok(())
}

pub fn delete_version_from_libraries(
    base_dir: &Path,
    library_name: &str,
    version_name: &str,
) -> Result<()> {
    let version_dir = base_dir
        .join(sanitize_filename(library_name))
        .join(sanitize_filename(version_name));
    if version_dir.exists() {
        fs::remove_dir_all(&version_dir)
            .context(format!("Failed to delete version directory: {:?}", version_dir))?;
    }
    Ok(())
}

pub fn delete_channel_from_libraries(
    base_dir: &Path,
    library_name: &str,
    version_name: &str,
    channel_name: &str,
) -> Result<()> {
    let channel_dir = base_dir.join(build_library_file_subdir(
        library_name,
        version_name,
        channel_name,
    ));
    if channel_dir.exists() {
        fs::remove_dir_all(&channel_dir)
            .context(format!("Failed to delete channel directory: {:?}", channel_dir))?;
    }

    Ok(())
}

/// 信号库本地存储管理器
///
/// 目录结构：
/// ```
/// config/
/// └── signal_library/
///     └── {library_name}/
///         └── {version_name}/
///             ├── database.dbc  (或 .ldf)
///             └── metadata.json
/// ```
pub struct SignalLibraryStorage {
    base_path: PathBuf,
}

impl SignalLibraryStorage {
    /// 创建新的存储管理器
    pub fn new() -> Result<Self> {
        let base_path = Self::get_base_path()?;

        // 确保基础目录存在
        fs::create_dir_all(&base_path).context("Failed to create signal library base directory")?;

        Ok(Self { base_path })
    }

    /// 获取基础路径
    /// 优先使用程序所在目录，如果失败则使用当前目录
    fn get_base_path() -> Result<PathBuf> {
        // 尝试获取可执行文件所在目录
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                return Ok(exe_dir.join("config").join("signal_library"));
            }
        }

        // 回退到当前目录
        let current_dir = std::env::current_dir().context("Failed to get current directory")?;
        Ok(current_dir.join("config").join("signal_library"))
    }

    /// 获取库的目录路径
    pub fn get_library_path(&self, library_name: &str) -> PathBuf {
        self.base_path.join(sanitize_filename(library_name))
    }

    /// 获取版本的目录路径
    pub fn get_version_path(&self, library_name: &str, version_name: &str) -> PathBuf {
        self.get_library_path(library_name)
            .join(sanitize_filename(version_name))
    }

    /// 复制数据库文件到本地存储
    ///
    /// # 参数
    /// - `library_name`: 库名称
    /// - `version_name`: 版本名称
    /// - `source_path`: 源数据库文件路径
    ///
    /// # 返回
    /// 复制后的文件路径
    pub fn copy_database(
        &self,
        library_name: &str,
        version_name: &str,
        source_path: &Path,
    ) -> Result<PathBuf> {
        // 创建版本目录
        let version_dir = self.get_version_path(library_name, version_name);
        fs::create_dir_all(&version_dir).context(format!(
            "Failed to create version directory: {:?}",
            version_dir
        ))?;

        // 获取文件扩展名
        let extension = source_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("dbc");

        // 获取文件名
        let file_name = source_path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| sanitize_filename(n))
            .unwrap_or_else(|| format!("database.{}", extension));

        // 目标文件路径
        let dest_path = version_dir.join(file_name);

        // 复制文件
        fs::copy(source_path, &dest_path).context(format!(
            "Failed to copy database from {:?} to {:?}",
            source_path, dest_path
        ))?;

        println!("✅ Database copied to: {:?}", dest_path);

        Ok(dest_path)
    }

    /// 获取数据库文件路径（如果存在）
    pub fn get_database_path(&self, library_name: &str, version_name: &str) -> Option<PathBuf> {
        let version_dir = self.get_version_path(library_name, version_name);

        // 尝试查找 .dbc 或 .ldf 文件
        for ext in &["dbc", "ldf", "DBC", "LDF"] {
            let db_path = version_dir.join(format!("database.{}", ext));
            if db_path.exists() {
                return Some(db_path);
            }
        }

        None
    }

    /// 删除库的所有数据
    pub fn delete_library(&self, library_name: &str) -> Result<()> {
        let library_path = self.get_library_path(library_name);
        if library_path.exists() {
            fs::remove_dir_all(&library_path)
                .context(format!("Failed to delete library: {:?}", library_path))?;
            println!("🗑️ Library deleted: {:?}", library_path);
        }
        Ok(())
    }

    /// 删除特定版本的数据
    pub fn delete_version(&self, library_name: &str, version_name: &str) -> Result<()> {
        let version_path = self.get_version_path(library_name, version_name);
        if version_path.exists() {
            fs::remove_dir_all(&version_path)
                .context(format!("Failed to delete version: {:?}", version_path))?;
            println!("🗑️ Version deleted: {:?}", version_path);
        }
        Ok(())
    }

    /// 列出所有库
    pub fn list_libraries(&self) -> Result<Vec<String>> {
        if !self.base_path.exists() {
            return Ok(Vec::new());
        }

        let mut libraries = Vec::new();

        for entry in fs::read_dir(&self.base_path)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    libraries.push(name.to_string());
                }
            }
        }

        Ok(libraries)
    }

    /// 列出库的所有版本
    pub fn list_versions(&self, library_name: &str) -> Result<Vec<String>> {
        let library_path = self.get_library_path(library_name);

        if !library_path.exists() {
            return Ok(Vec::new());
        }

        let mut versions = Vec::new();

        for entry in fs::read_dir(&library_path)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    versions.push(name.to_string());
                }
            }
        }

        Ok(versions)
    }

    /// 获取基础路径（用于显示）
    pub fn base_path(&self) -> &Path {
        &self.base_path
    }
}

/// 清理文件名，移除不安全的字符
pub fn sanitize_filename(name: &str) -> String {
    let sanitized: String = name
        .trim()
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c => c,
        })
        .collect();

    if sanitized.is_empty() {
        "unnamed".to_string()
    } else {
        sanitized
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("normal_name"), "normal_name");
        assert_eq!(sanitize_filename("name/with\\slashes"), "name_with_slashes");
        assert_eq!(
            sanitize_filename("name:with*special?chars"),
            "name_with_special_chars"
        );
    }

    #[test]
    fn test_storage_paths() {
        let storage = SignalLibraryStorage::new().unwrap();

        let lib_path = storage.get_library_path("test_lib");
        assert!(lib_path.ends_with("signal_library/test_lib"));

        let ver_path = storage.get_version_path("test_lib", "v1.0");
        assert!(ver_path.ends_with("signal_library/test_lib/v1.0"));
    }

    #[test]
    fn test_copy_database() -> Result<()> {
        let temp_dir = TempDir::new()?;

        // 创建测试数据库文件
        let source_file = temp_dir.path().join("test.dbc");
        let mut file = fs::File::create(&source_file)?;
        file.write_all(b"test database content")?;

        // 复制文件
        let storage = SignalLibraryStorage::new()?;
        let dest_path = storage.copy_database("test_lib", "v1.0", &source_file)?;

        // 验证文件存在
        assert!(dest_path.exists());

        // 验证内容
        let content = fs::read_to_string(&dest_path)?;
        assert_eq!(content, "test database content");

        // 清理
        storage.delete_library("test_lib")?;

        Ok(())
    }
}
