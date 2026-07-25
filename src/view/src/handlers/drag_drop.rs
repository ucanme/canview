//! Drag-and-drop BLF loading — entry point and path helpers.
//!
//! `handle_drop` is called from the in-window `.on_drop` handler in
//! `impls_rendering.rs` and from the macOS Dock-icon `on_open_urls`
//! handler in `main.rs`.

use std::path::PathBuf;

/// Filter paths: keep `.blf` (case-insensitive), return the rest as
/// skipped-name strings (file_name only, not full path).
pub fn filter_blf_paths(paths: Vec<PathBuf>) -> (Vec<PathBuf>, Vec<String>) {
    let mut accepted = Vec::new();
    let mut skipped = Vec::new();
    for p in paths {
        let is_blf = p
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.eq_ignore_ascii_case("blf"))
            .unwrap_or(false);
        if is_blf {
            accepted.push(p);
        } else if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
            skipped.push(name.to_string());
        }
    }
    (accepted, skipped)
}

/// Expand top-level folders one level deep. For each directory in
/// `paths`, list immediate `.blf` children (no recursion). Non-dir
/// paths pass through unchanged. Returns (collected BLF paths, summary
/// lines for status_msg).
pub fn expand_folders(paths: Vec<PathBuf>) -> (Vec<PathBuf>, Vec<String>) {
    let mut out = Vec::new();
    let mut summaries = Vec::new();
    for p in paths {
        if p.is_dir() {
            let mut blf_in_dir: Vec<PathBuf> = match std::fs::read_dir(&p) {
                Ok(rd) => rd
                    .filter_map(|e| e.ok())
                    .map(|e| e.path())
                    .filter(|c| {
                        c.is_file()
                            && c.extension()
                                .and_then(|e| e.to_str())
                                .map(|e| e.eq_ignore_ascii_case("blf"))
                                .unwrap_or(false)
                    })
                    .collect(),
                Err(err) => {
                    summaries.push(format!("Could not read folder {}: {}", p.display(), err));
                    continue;
                }
            };
            let n = blf_in_dir.len();
            if n == 0 {
                summaries.push(format!("Folder had no BLF files: {}", p.display()));
            } else {
                summaries.push(format!("Expanded folder: {} ({} BLF found)", p.display(), n));
            }
            blf_in_dir.sort();
            out.extend(blf_in_dir);
        } else {
            out.push(p);
        }
    }
    (out, summaries)
}

/// Entry point. Called from `.on_drop` (in-window) and `on_open_urls`
/// (Dock icon). See spec for the full state-machine. Implementation is
/// added in Task 4.
pub fn handle_drop(
    _app: &mut crate::app::CanViewApp,
    _cx: &mut gpui::Context<crate::app::CanViewApp>,
    _paths: Vec<PathBuf>,
) {
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn next_id() -> u64 {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        nanos.wrapping_add(n)
    }

    /// Minimal TempDir to avoid pulling `tempfile` as a dev-dependency.
    struct TempDir(PathBuf);
    impl TempDir {
        fn new() -> std::io::Result<Self> {
            let mut p = std::env::temp_dir();
            p.push(format!("canview-test-{}-{}", std::process::id(), next_id()));
            fs::create_dir_all(&p)?;
            Ok(Self(p))
        }
        fn path(&self) -> &Path {
            &self.0
        }
    }
    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn touch(path: &Path) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, b"").unwrap();
    }

    #[test]
    fn filter_blf_paths_keeps_blf_case_insensitive() {
        let tmp = TempDir::new().unwrap();
        let a = tmp.path().join("a.blf");
        let b = tmp.path().join("b.BLF");
        let c = tmp.path().join("c.bin");
        touch(&a);
        touch(&b);
        touch(&c);
        let (accepted, skipped) = filter_blf_paths(vec![a, b, c]);
        assert_eq!(accepted.len(), 2);
        assert_eq!(skipped, vec!["c.bin".to_string()]);
    }

    #[test]
    fn filter_blf_paths_handles_no_extension() {
        let tmp = TempDir::new().unwrap();
        let p = tmp.path().join("readme");
        touch(&p);
        let (accepted, skipped) = filter_blf_paths(vec![p]);
        assert!(accepted.is_empty());
        assert_eq!(skipped, vec!["readme".to_string()]);
    }

    #[test]
    fn expand_folders_one_level_only_no_recurse() {
        let tmp = TempDir::new().unwrap();
        touch(&tmp.path().join("a.blf"));
        touch(&tmp.path().join("b.blf"));
        touch(&tmp.path().join("sub/c.blf"));
        let (paths, summaries) = expand_folders(vec![tmp.path().to_path_buf()]);
        assert_eq!(paths.len(), 2, "subfolder BLF must not be recursed");
        assert_eq!(summaries.len(), 1);
        assert!(summaries[0].contains("2 BLF found"));
    }

    #[test]
    fn expand_folders_empty_folder_message() {
        let tmp = TempDir::new().unwrap();
        let empty = tmp.path().join("empty");
        fs::create_dir_all(&empty).unwrap();
        let (paths, summaries) = expand_folders(vec![empty]);
        assert!(paths.is_empty());
        assert!(summaries[0].contains("Folder had no BLF files"));
    }

    #[test]
    fn expand_folders_passes_files_through() {
        let tmp = TempDir::new().unwrap();
        let f = tmp.path().join("x.blf");
        touch(&f);
        let (paths, _) = expand_folders(vec![f.clone()]);
        assert_eq!(paths, vec![f]);
    }
}
