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
/// (Dock icon).
///
/// Pipeline:
/// 1. Filter to .blf + expand top-level folders.
/// 2. status_msg summaries for skipped/expanded.
/// 3. Large-file guard: total > 1 GB → Yes/No dialog; No → abort.
/// 4. If a load is in flight, cancel it and queue the paths to
///    `pending_drop_paths` (drained by the render tick).
/// 5. Otherwise, clear and start the concurrent load (reuses the
///    existing `apply_blf_result_append_one` pipeline).
pub fn handle_drop(
    app: &mut crate::app::CanViewApp,
    cx: &mut gpui::Context<crate::app::CanViewApp>,
    paths: Vec<PathBuf>,
) {
    let (blf_paths, skipped) = filter_blf_paths(paths);
    let (blf_paths, folder_summaries) = expand_folders(blf_paths);

    let mut msgs: Vec<String> = Vec::new();
    if !skipped.is_empty() {
        let names = skipped.iter().take(3).cloned().collect::<Vec<_>>().join(", ");
        let extra = if skipped.len() > 3 {
            format!(", +{} more", skipped.len() - 3)
        } else {
            String::new()
        };
        msgs.push(format!(
            "Skipped {} non-BLF file(s): {}{}",
            skipped.len(),
            names,
            extra
        ));
    }
    msgs.extend(folder_summaries);
    if !msgs.is_empty() {
        app.status_msg = msgs.join("  |  ").into();
        cx.notify();
    }

    if blf_paths.is_empty() {
        return;
    }

    // Large-file guard (> 1 GB)
    const FILE_SIZE_THRESHOLD: u64 = 1_000_000_000;
    let total_size: u64 = blf_paths
        .iter()
        .filter_map(|p| std::fs::metadata(p).ok())
        .map(|m| m.len())
        .sum();

    if total_size > FILE_SIZE_THRESHOLD {
        let view_handle = cx.entity().clone();
        let paths_for_dialog = blf_paths.clone();
        cx.spawn(async move |this, cx| {
            let _ = this;
            let confirmed = rfd::AsyncMessageDialog::new()
                .set_title("Large File Warning")
                .set_description(&format!(
                    "You are about to load {:.2} GB of BLF files. This may take significant time and memory. Continue?",
                    total_size as f64 / 1_000_000_000.0
                ))
                .set_buttons(rfd::MessageButtons::YesNo)
                .show()
                .await;
            if confirmed != rfd::MessageDialogResult::Yes {
                let _ = cx.update(|cx| {
                    view_handle.update(cx, |app, cx| {
                        app.status_msg = "Loading cancelled".into();
                        cx.notify();
                    });
                });
                return;
            }
            let _ = cx.update(|cx| {
                view_handle.update(cx, |app, cx| {
                    start_load_or_queue(app, cx, paths_for_dialog);
                });
            });
        })
        .detach();
        return;
    }

    start_load_or_queue(app, cx, blf_paths);
}

/// Decide: if a load is in flight, cancel it and queue; else start now.
fn start_load_or_queue(
    app: &mut crate::app::CanViewApp,
    cx: &mut gpui::Context<crate::app::CanViewApp>,
    blf_paths: Vec<PathBuf>,
) {
    if let Some(p) = &mut app.loading_progress {
        // Cancel in-flight; queue for next tick drain.
        p.is_cancelled = true;
        app.pending_drop_paths.extend(blf_paths);
        return;
    }
    // No load in flight — clear and start now.
    app.remove_all_files();
    start_concurrent_load(app, cx, blf_paths);
}

/// Concurrent load pipeline (reuses the existing multi-file menu path).
fn start_concurrent_load(
    app: &mut crate::app::CanViewApp,
    cx: &mut gpui::Context<crate::app::CanViewApp>,
    paths: Vec<PathBuf>,
) {
    use crate::app::LoadingProgress;
    use blf::read_blf_from_file;

    let total = paths.len();
    app.loading_progress = Some(LoadingProgress {
        total_files: total,
        completed_files: 0,
        current_file_name: None,
        total_messages_so_far: 0,
        is_cancelled: false,
    });
    app.status_msg = format!("⏳ Loading 0/{} files...", total).into();

    let view = cx.entity().clone();
    cx.spawn(async move |_this, cx| {
        let mut tasks = Vec::new();
        for path in paths.clone() {
            let task = cx.background_executor().spawn(async move {
                let result = read_blf_from_file(&path)
                    .map_err(|e| anyhow::Error::msg(format!("{:?}", e)));
                (path, result)
            });
            tasks.push(task);
        }
        for task in tasks {
            let (path, result) = task.await;
            let _ = cx.update(|cx| {
                view.update(cx, |app, cx| {
                    app.apply_blf_result_append_one(result, path);
                    cx.notify();
                });
            });
        }
    })
    .detach();
}

/// Called from the render tick when `loading_progress` is None and
/// `pending_drop_paths` is non-empty. Clears current files (replace
/// semantics) then runs the concurrent load.
pub fn drain_pending_drop(
    app: &mut crate::app::CanViewApp,
    cx: &mut gpui::Context<crate::app::CanViewApp>,
) {
    if app.loading_progress.is_none() && !app.pending_drop_paths.is_empty() {
        let pending = std::mem::take(&mut app.pending_drop_paths);
        app.remove_all_files();
        start_concurrent_load(app, cx, pending);
    }
}

/// Pull paths dropped on the Dock icon (stashed by `main.rs`'s
/// `on_open_urls` handler) into `pending_drop_paths` so the regular
/// tick drain will pick them up. Called from the render tick.
pub fn drain_dock_drop_queue(app: &mut crate::app::CanViewApp) {
    if let Ok(mut q) = crate::DOCK_DROP_QUEUE.lock() {
        if !q.is_empty() {
            app.pending_drop_paths.extend(q.drain(..));
        }
    }
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
