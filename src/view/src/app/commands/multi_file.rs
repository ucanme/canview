//! 多文件加载与移除命令
//!
//! 注意：本模块的类型在 Task 3+ 才会被 UI 层引用，在此之前会有 dead_code 警告。
//! `#![allow(dead_code)]` 是临时措施，Task 3+ 接入后即可移除。

#![allow(dead_code)]

use std::path::PathBuf;

/// 加载模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadMode {
    /// 单选 Open BLF... — 清空已加载文件后加载新文件
    Replace,
    /// 多选 Open Multiple BLF... — 追加到已加载文件
    Append,
}

/// 加载一个或多个 BLF 文件
#[derive(Debug, Clone)]
pub struct LoadBlfFiles {
    pub paths: Vec<PathBuf>,
    pub mode: LoadMode,
}

impl LoadBlfFiles {
    pub fn new(paths: Vec<PathBuf>, mode: LoadMode) -> Self {
        Self { paths, mode }
    }

    pub fn is_empty(&self) -> bool {
        self.paths.is_empty()
    }

    pub fn len(&self) -> usize {
        self.paths.len()
    }
}

/// 移除单个已加载文件
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemoveFile {
    pub file_id: u32,
}

impl RemoveFile {
    pub fn new(file_id: u32) -> Self {
        Self { file_id }
    }
}

/// 移除所有已加载文件
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemoveAllFiles;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_mode_replace() {
        let cmd = LoadBlfFiles::new(vec![PathBuf::from("/a.blf")], LoadMode::Replace);
        assert_eq!(cmd.mode, LoadMode::Replace);
        assert_eq!(cmd.len(), 1);
        assert!(!cmd.is_empty());
    }

    #[test]
    fn test_load_mode_append() {
        let cmd = LoadBlfFiles::new(
            vec![PathBuf::from("/a.blf"), PathBuf::from("/b.blf")],
            LoadMode::Append,
        );
        assert_eq!(cmd.mode, LoadMode::Append);
        assert_eq!(cmd.len(), 2);
    }

    #[test]
    fn test_load_blf_files_empty() {
        let cmd = LoadBlfFiles::new(Vec::new(), LoadMode::Append);
        assert!(cmd.is_empty());
        assert_eq!(cmd.len(), 0);
    }

    #[test]
    fn test_remove_file_command() {
        let cmd = RemoveFile::new(42);
        assert_eq!(cmd.file_id, 42);
    }
}
