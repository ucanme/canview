# Drag-and-Drop BLF Loading Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Load `.blf` files by dragging them onto the app window (or Dock icon on macOS), with extension filtering, one-level folder expansion, clear-on-open semantics, in-progress cancellation, large-file guard, and a hover overlay.

**Architecture:** Add a single `src/view/src/handlers/drag_drop.rs` module exposing `handle_drop`, called from an `on_drop` handler attached to the content-area div in `impls_rendering.rs` and from an `Application::on_open_urls` handler in `main.rs`. The load pipeline reuses the existing `apply_blf_result_append_one` concurrent spawn loop; the only new logic is path filtering, folder expansion, status-message summaries, and the in-flight cancel/restart queue.

**Tech Stack:** Rust 2024, GPUI (zed fork), gpui-component, smol, rfd, blf crate (`read_blf_from_file`).

## Global Constraints

- Only `.blf` extension is accepted via drag (NOT `.bin`, even though File menu accepts both).
- Folders are expanded **one level deep only** (no recursion).
- Every drop calls `remove_all_files()` before loading (replace-not-append semantics).
- Large-file guard: total > 1 GB triggers Yes/No dialog; No → "Loading cancelled" status.
- In-flight load is cancelled (`is_cancelled = true`) before queueing the new drop; the new drop runs once `loading_progress` becomes `None`.
- Hover overlay covers only the content area, NOT the top bar or status bar.
- `FileDropEvent::Submit` does NOT carry paths; stash paths from `Entered` and read on `Submit`.

---

## File Structure

- **Create** `src/view/src/handlers/drag_drop.rs` — entry point `handle_drop` + `filter_blf_paths` + `expand_folders`.
- **Modify** `src/view/src/handlers/mod.rs` — add `pub mod drag_drop;` re-export.
- **Modify** `src/view/src/app/state.rs` — add `drag_drop_hover: Option<usize>`, `pending_drop_stash: Vec<PathBuf>`, `pending_drop_paths: Vec<PathBuf>`; init in `Default`/`new`.
- **Modify** `src/view/src/app/impls_rendering.rs` — attach `.on_drop` to content-area div (around line 1834); add hover overlay rendering as another child of the content-area div; add tick-loop check at top of `render` to drain `pending_drop_paths`.
- **Modify** `src/view/src/main.rs` — register `Application::on_open_urls` handler for macOS Dock-icon drop.
- **Test** `src/view/src/handlers/drag_drop.rs` — `#[cfg(test)] mod tests` for `filter_blf_paths` and `expand_folders`.

---

### Task 1: State additions for drag-drop

**Files:**
- Modify: `src/view/src/app/state.rs:111-122` (add 3 fields after `show_blf_errors_popover`)
- Modify: `src/view/src/app/state.rs:297-310` (init in `Default::default` impl)
- Modify: `src/view/src/app/impls.rs:44-55` and `:800-810` (init in `new` and any other constructors)

**Interfaces:**
- Consumes: existing `LoadingProgress` struct from `state.rs`.
- Produces: three new public fields on `CanViewApp`:
  - `pub drag_drop_hover: Option<usize>` — `Some(n)` while drag-over is active; `n` = count of BLF files in the drag (0 means all non-BLF).
  - `pub pending_drop_stash: Vec<std::path::PathBuf>` — raw paths from `FileDropEvent::Entered`, drained on `Submit` or `Exited`.
  - `pub pending_drop_paths: Vec<std::path::PathBuf>` — validated BLF paths waiting for an in-flight load to finish; drained by the tick loop.

- [ ] **Step 1: Add fields to struct**

Edit `src/view/src/app/state.rs` after line 122 (`show_blf_errors_popover`):

```rust
    // Drag-and-drop state (in-window + Dock icon)
    /// `Some(n)` while FileDropEvent::Entered is active; `n` is the BLF
    /// file count (0 = all non-BLF). Cleared on Exited or Submit.
    pub drag_drop_hover: Option<usize>,
    /// Raw paths from FileDropEvent::Entered — Submit doesn't carry
    /// paths, so we stash them here on Entered and read on Submit.
    pub pending_drop_stash: Vec<std::path::PathBuf>,
    /// Validated BLF paths waiting for an in-flight load to finish.
    /// Drained by the render tick when `loading_progress` is None.
    pub pending_drop_paths: Vec<std::path::PathBuf>,
```

- [ ] **Step 2: Find every struct initializer and add the fields**

Run: `grep -n "show_blf_errors_popover: false" src/view/src/app/`

For each match, add three lines right after it:

```rust
            drag_drop_hover: None,
            pending_drop_stash: Vec::new(),
            pending_drop_paths: Vec::new(),
```

(Expected matches: `state.rs:304`, `impls.rs:51`, `impls.rs:807` — there may be more; update every one.)

- [ ] **Step 3: Verify it compiles**

Run: `cargo build --release` from `src/view/`
Expected: 0 errors. Warnings OK.

- [ ] **Step 4: Commit**

```bash
git add src/view/src/app/state.rs src/view/src/app/impls.rs
git commit -m "feat(drag-drop): add state fields for hover, stash, pending paths

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

### Task 2: Path-filtering and folder-expansion helpers with unit tests

**Files:**
- Create: `src/view/src/handlers/drag_drop.rs`
- Modify: `src/view/src/handlers/mod.rs` (add `pub mod drag_drop;`)

**Interfaces:**
- Consumes: `std::path::PathBuf`.
- Produces:
  - `pub fn filter_blf_paths(paths: Vec<PathBuf>) -> (Vec<PathBuf>, Vec<String>)` — returns (accepted `.blf` paths, names of skipped non-BLF files).
  - `pub fn expand_folders(paths: Vec<PathBuf>) -> (Vec<PathBuf>, Vec<String>)` — for each directory in `paths`, lists its immediate `.blf` children (no recursion); returns (collected BLF paths from all sources, per-folder summary lines).

- [ ] **Step 1: Create the module with stubs**

Create `src/view/src/handlers/drag_drop.rs`:

```rust
//! Drag-and-drop BLF loading — entry point and path helpers.
//!
//! `handle_drop` is called from the in-window `.on_drop` handler in
//! `impls_rendering.rs` and from the macOS Dock-icon `on_open_urls`
//! handler in `main.rs`.

use std::path::{Path, PathBuf};

/// Filter paths: keep `.blf` (case-insensitive), return the rest as
/// skipped-name strings (file_name only, not full path).
pub fn filter_blf_paths(paths: Vec<PathBuf>) -> (Vec<PathBuf>, Vec<String>) {
    let mut accepted = Vec::new();
    let mut skipped = Vec::new();
    for p in paths {
        if p.extension().and_then(|e| e.to_str()).map(|e| e.eq_ignore_ascii_case("blf")).unwrap_or(false) {
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
                    .filter(|c| c.is_file() && c.extension().and_then(|e| e.to_str()).map(|e| e.eq_ignore_ascii_case("blf")).unwrap_or(false))
                    .collect(),
                Err(err) => {
                    summaries.push(format!("Could not read folder {:?}: {}", p.display(), err));
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
    // Filled in by Task 4.
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn touch(path: &Path) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, b"").unwrap();
    }

    #[test]
    fn filter_blf_paths_keeps_blf_case_insensitive() {
        let tmp = tempdir().unwrap();
        let a = tmp.path().join("a.blf");
        let b = tmp.path().join("b.BLF");
        let c = tmp.path().join("c.bin");
        touch(&a);
        touch(&b);
        touch(&c);
        let (accepted, skipped) = filter_blf_paths(vec![a.clone(), b.clone(), c.clone()]);
        assert_eq!(accepted.len(), 2);
        assert_eq!(skipped, vec!["c.bin".to_string()]);
    }

    #[test]
    fn filter_blf_paths_handles_no_extension() {
        let tmp = tempdir().unwrap();
        let p = tmp.path().join("readme");
        touch(&p);
        let (accepted, skipped) = filter_blf_paths(vec![p]);
        assert!(accepted.is_empty());
        assert_eq!(skipped, vec!["readme".to_string()]);
    }

    #[test]
    fn expand_folders_one_level_only_no_recurse() {
        let tmp = tempdir().unwrap();
        // top folder with 2 BLF + 1 subfolder containing 1 BLF
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
        let tmp = tempdir().unwrap();
        let empty = tmp.path().join("empty");
        fs::create_dir_all(&empty).unwrap();
        let (paths, summaries) = expand_folders(vec![empty.clone()]);
        assert!(paths.is_empty());
        assert!(summaries[0].contains("Folder had no BLF files"));
    }

    #[test]
    fn expand_folders_passes_files_through() {
        let tmp = tempdir().unwrap();
        let f = tmp.path().join("x.blf");
        touch(&f);
        let (paths, _) = expand_folders(vec![f.clone()]);
        assert_eq!(paths, vec![f]);
    }

    fn tempdir() -> std::io::Result<TempDir> {
        TempDir::new()
    }

    // minimal TempDir helper to avoid pulling tempfile dev-dependency
    struct TempDir(PathBuf);
    impl TempDir {
        fn new() -> std::io::Result<Self> {
            let mut p = std::env::temp_dir();
            p.push(format!("canview-test-{}", std::process::id()));
            p.push(format!("{}", rand_u64()));
            fs::create_dir_all(&p)?;
            Ok(Self(p))
        }
        fn path(&self) -> &Path { &self.0 }
    }
    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    fn rand_u64() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64
    }
}
```

Edit `src/view/src/handlers/mod.rs`:

```rust
pub mod drag_drop;
```

(Add the line above in the existing module list — keep other entries unchanged.)

- [ ] **Step 2: Run the tests to verify they pass**

Run: `cd src/view && cargo test --release handlers::drag_drop::tests -- --nocapture`
Expected: 5 tests pass.

If `rand_u64` collides (two tests get the same nanosecond stamp), add `+ i` per call. Use `static ATOMIC: AtomicU64` if needed for uniqueness.

- [ ] **Step 3: Commit**

```bash
git add src/view/src/handlers/drag_drop.rs src/view/src/handlers/mod.rs
git commit -m "feat(drag-drop): add path filtering and folder expansion helpers

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

### Task 3: In-window drop overlay (visual feedback only, no load yet)

**Files:**
- Modify: `src/view/src/app/impls_rendering.rs:1833-1856` (content-area div)

**Interfaces:**
- Consumes: `app.drag_drop_hover` from Task 1.
- Produces: visual overlay during `FileDropEvent::Entered`; cleared on `Exited` or `Submit`.

- [ ] **Step 1: Attach `.on_drop` to the content-area div**

Find the content-area div around line 1834 in `impls_rendering.rs`:

```rust
            .child(
                // Content area - Zed style
                div()
                    .flex_1()
                    .bg(rgb(0x0c0c0e))
                    .overflow_hidden()
                    .relative()
                    .child(match self.current_view {
```

Add an `.on_drop` handler and a hover overlay as a second child of the content-area div. Use `gpui::FileDropEvent`. The handler needs `view` cloned from the top of `render` (already available as `let view = cx.entity().clone();` at line ~1693).

```rust
            .child(
                // Content area - Zed style
                div()
                    .flex_1()
                    .bg(rgb(0x0c0c0e))
                    .overflow_hidden()
                    .relative()
                    .on_drop::<gpui::FileDropEvent>({
                        let view = view.clone();
                        move |event: gpui::FileDropEvent, _window, cx| {
                            view.update(cx, |app, cx| {
                                match event {
                                    gpui::FileDropEvent::Entered { paths, .. } => {
                                        let (blf, _) = crate::handlers::drag_drop::filter_blf_paths(paths.clone());
                                        app.drag_drop_hover = Some(blf.len());
                                        app.pending_drop_stash = paths;
                                        cx.notify();
                                    }
                                    gpui::FileDropEvent::Exited => {
                                        app.drag_drop_hover = None;
                                        app.pending_drop_stash.clear();
                                        cx.notify();
                                    }
                                    gpui::FileDropEvent::Submit { .. } => {
                                        let stash = std::mem::take(&mut app.pending_drop_stash);
                                        app.drag_drop_hover = None;
                                        crate::handlers::drag_drop::handle_drop(app, cx, stash);
                                        cx.notify();
                                    }
                                    _ => {}
                                }
                            });
                            gpui::DropDownResult::default()
                        }
                    })
                    .child(match self.current_view {
                        AppView::LogView => { /* ... unchanged ... */ }
                        // ...
                    })
                    // Library picker overlay — covers only the content area
                    .when_some(
                        crate::ui::components::render_library_picker_overlay(self, view.clone()),
                        |el, picker| el.child(picker),
                    )
                    // Drag-drop hover overlay — covers only the content area
                    .when_some(self.drag_drop_hover, |el, n| {
                        el.child(
                            div()
                                .absolute()
                                .top_0()
                                .left_0()
                                .w_full()
                                .h_full()
                                .bg(gpui::rgba(0x00000022))
                                .flex()
                                .items_center()
                                .justify_center()
                                .child(
                                    if *n > 0 {
                                        format!("📂 Drop to load {} BLF file(s)", n)
                                    } else {
                                        "⚠️ No BLF files in drop".to_string()
                                    },
                                )
                                .pointer_events_none(),
                        )
                    }),
            )
```

Note: `gpui::DropDownResult` may be `gpui::DropResult` — verify against the GPUI version. Run `grep -n "pub enum DropResult\|pub enum FileDropEvent\|pub type DropDownResult" ~/.cargo/git/checkouts/zed-a70e2ad075855582/ee0e370/crates/gpui/src/` to find the correct return type for `.on_drop`. Adjust the closure to return that type.

- [ ] **Step 2: Verify it compiles**

Run: `cd src/view && cargo build --release 2>&1 | grep -E "^error|error\[" | head -10`
Expected: no errors (or only type-name errors you fix in this step).

- [ ] **Step 3: Manually verify the overlay**

Build and run. Drag a `.blf` file over the window without releasing. Expected: a translucent overlay with "📂 Drop to load 1 BLF file(s)" appears over the content area only; the top bar and status bar remain visible. Drag away without dropping — overlay clears. (Submit does nothing yet because `handle_drop` is a stub.)

- [ ] **Step 4: Commit**

```bash
git add src/view/src/app/impls_rendering.rs
git commit -m "feat(drag-drop): add in-window hover overlay and on_drop handler

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

### Task 4: `handle_drop` — filter, expand, clear, queue or load

**Files:**
- Modify: `src/view/src/handlers/drag_drop.rs` (replace the stub `handle_drop`)

**Interfaces:**
- Consumes:
  - `crate::app::CanViewApp` (has `remove_all_files`, `loading_progress`, `status_msg`, `pending_drop_paths`)
  - `crate::app::state::LoadingProgress`
  - `blf::read_blf_from_file`
  - `rfd::AsyncMessageDialog`, `rfd::MessageButtons`, `rfd::MessageDialogResult`
- Produces: full `handle_drop` implementation.

- [ ] **Step 1: Replace the stub `handle_drop`**

Replace the `pub fn handle_drop` body in `src/view/src/handlers/drag_drop.rs` with the real implementation. Note: `handle_drop` cannot be `async` because it's called from the `on_drop` closure which is sync; the actual file reading happens in `cx.background_executor().spawn` like the existing multi-file menu path. So `handle_drop` just queues/cancels and (when no load is in flight) starts a detached async task.

```rust
use gpui::Context;

pub fn handle_drop(
    app: &mut crate::app::CanViewApp,
    cx: &mut gpui::Context<crate::app::CanViewApp>,
    paths: Vec<PathBuf>,
) {
    use crate::app::state::LoadingProgress;

    // 1. Filter + expand
    let (blf_paths, skipped) = filter_blf_paths(paths);
    let (blf_paths, folder_summaries) = expand_folders(blf_paths);

    // 2. status_msg summaries
    let mut msgs: Vec<String> = Vec::new();
    if !skipped.is_empty() {
        let names = skipped.iter().take(3).cloned().collect::<Vec<_>>().join(", ");
        let extra = if skipped.len() > 3 { format!(", +{} more", skipped.len() - 3) } else { String::new() };
        msgs.push(format!("Skipped {} non-BLF file(s): {}{}", skipped.len(), names, extra));
    }
    msgs.extend(folder_summaries);
    if !msgs.is_empty() {
        app.status_msg = msgs.join("  |  ").into();
        cx.notify();
    }

    if blf_paths.is_empty() {
        return;
    }

    // 3. Large-file guard (> 1 GB)
    const FILE_SIZE_THRESHOLD: u64 = 1_000_000_000;
    let total_size: u64 = blf_paths.iter()
        .filter_map(|p| std::fs::metadata(p).ok())
        .map(|m| m.len())
        .sum();
    if total_size > FILE_SIZE_THRESHOLD {
        // Spawn a detached async dialog; if user says No, set status_msg and bail.
        let view_handle = cx.entity().clone();
        cx.spawn(async move |cx| {
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
            // Yes → kick off load. We need to re-enter handle_drop's load path,
            // but the size guard already passed. Use a helper that skips the
            // dialog and goes straight to start_load.
            let _ = cx.update(|cx| {
                view_handle.update(cx, |app, cx| {
                    start_load_or_queue(app, cx, blf_paths);
                });
            });
        }).detach();
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
        // Cancel in-flight; queue for next tick
        p.is_cancelled = true;
        app.pending_drop_paths.extend(blf_paths);
        return;
    }
    // No load in flight — clear and start now
    app.remove_all_files();
    start_concurrent_load(app, cx, blf_paths);
}

/// Reuses the concurrent spawn + apply_blf_result_append_one pipeline
/// from the "Open Multiple BLF..." menu path.
fn start_concurrent_load(
    app: &mut crate::app::CanViewApp,
    cx: &mut gpui::Context<crate::app::CanViewApp>,
    paths: Vec<PathBuf>,
) {
    use crate::app::state::LoadingProgress;
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
    cx.spawn(async move |cx| {
        let mut tasks = Vec::new();
        for path in paths.clone() {
            let task = cx.background_executor().spawn(async move {
                let result = read_blf_from_file(&path).map_err(|e| anyhow::Error::msg(format!("{:?}", e)));
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
    }).detach();
}
```

- [ ] **Step 2: Add tick-loop drain at the top of `render`**

In `impls_rendering.rs`, at the very top of `fn render` (after the existing pre-amble, before the `let view = cx.entity().clone();` line), add:

```rust
        // Drain pending drop paths when no load is in flight.
        if self.loading_progress.is_none() && !self.pending_drop_paths.is_empty() {
            let pending = std::mem::take(&mut self.pending_drop_paths);
            crate::handlers::drag_drop::start_concurrent_load_after_clear(self, cx, pending);
        }
```

`start_concurrent_load_after_clear` is the same as `start_concurrent_load` but skips the in-flight check (caller already verified). Expose it as `pub` from `drag_drop.rs`:

```rust
/// Called from the render tick when `loading_progress` is None and
/// `pending_drop_paths` is non-empty. Clears current files (replace
/// semantics) then runs the concurrent load.
pub fn start_concurrent_load_after_clear(
    app: &mut crate::app::CanViewApp,
    cx: &mut gpui::Context<crate::app::CanViewApp>,
    paths: Vec<PathBuf>,
) {
    app.remove_all_files();
    start_concurrent_load(app, cx, paths);
}
```

- [ ] **Step 3: Verify it compiles**

Run: `cd src/view && cargo build --release 2>&1 | grep -E "^error|error\[" | head -20`
Expected: no errors. If `apply_blf_result_append_one` is `pub(crate)` and the handler module is a sibling of `app`, it's visible. If not, change visibility on `apply_blf_result_append_one` from `pub(crate)` to `pub` (one-line change in `impls.rs`).

- [ ] **Step 4: Manually verify single-file drop**

Build, run. Drag one `.blf` file onto the window. Expected:
- Hover overlay shows "📂 Drop to load 1 BLF file(s)".
- On release, `remove_all_files()` runs, BLF parses, log view populates, left-side status bar shows `📂 <file_name>` or `📂 N files` after load.

- [ ] **Step 5: Manually verify multi-file drop**

Drag 3 `.blf` files. Expected: status bar shows `⏳ Loading X/3`, files popover lists all 3 after load.

- [ ] **Step 6: Manually verify mixed drop**

Drag 2 `.blf` + 1 `.pdf`. Expected: BLF files load, `status_msg` shows `Skipped 1 non-BLF file(s): report.pdf`.

- [ ] **Step 7: Manually verify folder drop**

Drag a folder containing 5 `.blf` files at top level + 1 subfolder. Expected: 5 BLF load, subfolder skipped, `status_msg` shows `Expanded folder: ... (5 BLF found)`.

- [ ] **Step 8: Manually verify in-flight cancel + new drop**

Start loading 10 BLF files (slow). Mid-load, drop 3 more. Expected: first load's `is_cancelled` flips on, once it drains, the 3 new files load from scratch (replace semantics).

- [ ] **Step 9: Manually verify large-file guard**

Drop 1.5 GB of BLF. Expected: Yes/No dialog. No → "Loading cancelled". Yes → loads.

- [ ] **Step 10: Commit**

```bash
git add src/view/src/handlers/drag_drop.rs src/view/src/app/impls_rendering.rs src/view/src/app/impls.rs
git commit -m "feat(drag-drop): implement handle_drop with clear, queue, large-file guard

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

### Task 5: macOS Dock-icon drop via `on_open_urls`

**Files:**
- Modify: `src/view/src/main.rs` (register `Application::on_open_urls`)

**Interfaces:**
- Consumes: `crate::handlers::drag_drop::handle_drop`, `cx.windows()`, `gpui::Application::on_open_urls`.
- Produces: paths dropped on the Dock icon while the app is running (or at cold start) flow through the same `handle_drop` path as in-window drops.

- [ ] **Step 1: Find the main entry point and the window-creation closure**

Run: `grep -n "fn main\|Application::new\|cx.open_window\|on_open_urls" src/view/src/main.rs`

Note the line where `Application::new` is created and where `cx.open_window` is called.

- [ ] **Step 2: Register `on_open_urls` before `cx.run`**

After `Application::new` is created (or `cx` is the app context), add:

```rust
    // macOS Dock-icon drop: file:// URLs for dropped files.
    // Linux/Windows behavior is unverified — only wire macOS path here.
    cx.on_open_urls({
        move |urls: Vec<String>, cx| {
            let paths: Vec<std::path::PathBuf> = urls.iter()
                .filter_map(|u| url::Url::parse(u).ok())
                .filter(|u| u.scheme() == "file")
                .filter_map(|u| u.to_file_path().ok())
                .collect();
            if paths.is_empty() { return; }

            // Find the first CanViewApp window and dispatch.
            for window in cx.windows() {
                let entity = window.entity();
                if let Some(view) = entity.downcast::<crate::app::CanViewApp>() {
                    view.update(cx, |app, cx| {
                        crate::handlers::drag_drop::handle_drop(app, cx, paths.clone());
                    });
                    return;
                }
            }
            // No window yet — paths will be lost. (Cold-start Dock drop is
            // a separate, larger feature; out of scope for this iteration.)
        }
    });
```

Note: the exact `on_open_urls` API may differ. Run `grep -rn "on_open_urls" ~/.cargo/git/checkouts/zed-a70e2ad075855582/ee0e370/crates/gpui/src/` to find the signature. If it takes a closure that receives `Vec<String>` and `&mut App`, use that. If it takes URLs as `Vec<Url>` directly, drop the `url::Url::parse` step.

Add `url = "2"` to `src/view/Cargo.toml` `[dependencies]` if not already present. Run `grep -n '^url' src/view/Cargo.toml` to check.

- [ ] **Step 3: Verify it compiles**

Run: `cd src/view && cargo build --release 2>&1 | grep -E "^error|error\[" | head -10`
Expected: no errors.

- [ ] **Step 4: Manually verify Dock-icon drop (macOS)**

Build, run. While app is running, drag a `.blf` file onto the Dock icon. Expected: same path as in-window drop — `remove_all_files()`, parse, log view populates.

- [ ] **Step 5: Commit**

```bash
git add src/view/src/main.rs src/view/Cargo.toml
git commit -m "feat(drag-drop): wire macOS Dock-icon drop via on_open_urls

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

### Task 6: Update spec & README references (docs)

**Files:**
- Modify: `README.md` and `README_zh.md` (mention drag-and-drop in the "How to load files" section, if one exists)
- Modify: `docs/USAGE.md` and `docs/USAGE_zh.md` (add drag-and-drop as a load method)

- [ ] **Step 1: Find the existing "load files" documentation**

Run: `grep -n "Open BLF\|Open Multiple\|File menu\|loading files\|加载" README.md README_zh.md docs/USAGE.md docs/USAGE_zh.md`

- [ ] **Step 2: Add a one-paragraph section**

In each file, after the File-menu load description, add a short paragraph noting drag-and-drop as an alternative:

```markdown
**Drag-and-drop:** You can also drag `.blf` files (or a folder containing them) onto the app window. Only `.blf` is accepted; non-BLF files are skipped with a status message. Dropping clears any currently-loaded files first. On macOS, dropping on the Dock icon also works.
```

For the Chinese version:

```markdown
**拖拽加载:** 也可将 `.blf` 文件(或包含它们的文件夹)直接拖到应用窗口。仅接受 `.blf` 扩展名,非 BLF 文件会被跳过并通过状态栏提示。每次拖拽会先清空已加载的文件。macOS 上拖到 Dock 图标也可加载。
```

- [ ] **Step 3: Commit**

```bash
git add README.md README_zh.md docs/USAGE.md docs/USAGE_zh.md
git commit -m "docs: mention drag-and-drop as a BLF load method

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Self-Review

**Spec coverage check:**

| Spec section | Covered by |
|---|---|
| Extension filter (.blf only) | Task 2 `filter_blf_paths` |
| Folder expansion (1 level) | Task 2 `expand_folders` |
| Clear-on-open | Task 4 `start_load_or_queue` calls `remove_all_files()` |
| Loading-in-progress cancel+queue | Task 4 `start_load_or_queue` + Task 4 Step 2 tick drain |
| Large-file protection | Task 4 Step 1 size guard + Yes/No dialog |
| Visual feedback overlay (content area only) | Task 3 `.on_drop` + hover overlay |
| Dock-icon drop (macOS) | Task 5 `on_open_urls` |
| status_msg summaries for skipped/expanded | Task 4 Step 1 |

**Placeholder scan:** No TBD / TODO / "implement later". All code blocks are complete.

**Type consistency:**
- `drag_drop_hover: Option<usize>` — used identically in Task 1 (state) and Task 3 (overlay).
- `pending_drop_stash: Vec<PathBuf>` — set in Task 3 `Entered`, drained in Task 3 `Submit`.
- `pending_drop_paths: Vec<PathBuf>` — populated in Task 4 `start_load_or_queue`, drained in Task 4 Step 2 tick.
- `handle_drop(app: &mut CanViewApp, cx: &mut Context<CanViewApp>, paths: Vec<PathBuf>)` — same signature in Task 2 stub, Task 4 implementation, Task 5 Dock handler.
- `start_concurrent_load(app, cx, paths)` — internal helper, used by `start_load_or_queue` and `start_concurrent_load_after_clear` consistently.

**Known risks the implementer should verify:**
1. `gpui::FileDropEvent` variant names — `Entered { paths, .. }`, `Exited`, `Submit { .. }` — confirm against the GPUI source. The `paths` field may be named differently.
2. `gpui::DropResult` / `DropDownResult` — the return type of `.on_drop`. Confirm and adjust.
3. `Application::on_open_urls` signature — confirm the closure parameter types against GPUI source.
4. `url` crate already in Cargo.toml? If not, add it (Task 5).
5. `apply_blf_result_append_one` visibility — change from `pub(crate)` to `pub` if the handler module can't see it.
