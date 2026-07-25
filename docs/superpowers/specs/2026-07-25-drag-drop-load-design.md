# Drag-and-Drop BLF Loading — Design Spec

**Date:** 2026-07-25
**Branch:** `feat/ui-redesign`
**Supersedes / extends:** —
**Status:** Ready for implementation plan

## Goal

Users should be able to load BLF files by dragging them onto the app window or onto the Dock icon, instead of going through the File menu.

## User-Facing Behavior

### Extension filter

Only `.blf` is accepted. Files with other extensions (including `.bin`) are skipped, and `status_msg` reports the count and names of skipped files:

```
Skipped 2 non-BLF file(s): report.pdf, notes.txt
```

Skipped files don't block the rest of the drop — if a drop contains 3 BLF + 2 non-BLF, the 3 BLF still load.

> Note: The File menu's `Open BLF…` / `Open Multiple BLF…` dialogs still accept `.bin` (Vector's legacy BLF extension). Drag-and-drop is stricter per the user's request — `.blf` only. Updating the File menu to match is out of scope for this spec.

### Folder handling

Folders are expanded **one level deep only** (no recursion). The top-level contents of any dropped folder are scanned for `.blf` files; subfolders inside it are skipped. This matches "拖文件夹 → 展开顶层". A `status_msg` line lists folders that were expanded:

```
Expanded folder: /path/to/logs/ (3 BLF found)
```

If a top-level folder contains no BLF files:

```
Folder had no BLF files: /path/to/empty/
```

### Clear-on-open

Every successful drop clears any currently-loaded files before loading the new ones — single-file and multi-file drops both behave this way. This is a deliberate divergence from the File menu, where "Open Multiple BLF…" appends. Drag-and-drop is treated as "replace my session", per the user's spec.

Implementation: call `remove_all_files()` first, then run the load path. For a single file this collapses to the existing `apply_blf_result_single` (which already clears). For multiple files, call `remove_all_files()` then run the same concurrent-append pipeline used by "Open Multiple BLF…".

### Loading-in-progress behavior

If a drop arrives while `loading_progress.is_some()` (a previous load is in flight):

1. Set `loading_progress.is_cancelled = true` to abort the in-flight load (same as the Cancel button).
2. Wait for the in-flight tasks to observe the cancellation flag — the existing `apply_blf_result_append_one` already checks `is_cancelled` and short-circuits.
3. Once `loading_progress` is `None`, proceed with the new drop.

For the simplest implementation: when a drop arrives, set `is_cancelled = true`, then enqueue the new drop paths on a pending-drop queue; on the next render tick after `loading_progress` becomes `None`, drain the queue and run the load. This avoids racing with the in-flight task.

### Large-file protection

Reuse the 1 GB total-size threshold and Yes/No dialog from "Open Multiple BLF…". The dialog appears between `Submit` and the actual load start. If the user clicks No, `status_msg` reports "Loading cancelled" and no files are loaded.

### Visual feedback

When `FileDropEvent::Entered` fires with one or more BLF files:

- Set state `drag_drop_hover: Option<usize>` (count of BLF files about to drop, 0 means "all non-BLF, will be skipped").
- The root div renders a translucent overlay `bg(rgba(0x00000022))` covering the content area, with centered text:
  - `📂 Drop to load {N} BLF file(s)` when N > 0
  - `⚠️ No BLF files in drop` when N == 0
- On `Exited` or `Submit`, clear `drag_drop_hover`.

The overlay should NOT cover the top bar or status bar (so the user can still see the file count and current status). Attach the overlay to the content area div in `impls_rendering.rs`, not the root div.

### Dock-icon drop (macOS only)

`Application::on_open_urls` receives a `Vec<String>` of URL strings. On macOS, dropping files on the Dock icon produces `file://` URLs.

Plan:
1. In `main.rs`, register an `on_open_urls` handler that parses each URL, filters to `.blf`, and stores the resulting paths in a global pending queue (`Rc<RefCell<Vec<PathBuf>>>`).
2. After the main window's `CanViewApp` is created, check the queue and trigger the same drop-load path as in-window drag.
3. If the app is already running with a window, the handler uses `cx.windows()` to find the main window's view entity and dispatches the drop to it.

Linux/Windows Dock-icon drop support is out of scope for this iteration — `on_open_urls` may or may not fire on those platforms. The in-window drag path works everywhere.

## Architecture

### State additions (`src/view/src/app/state.rs`)

```rust
pub struct CanViewApp {
    // ... existing fields ...

    /// Tracks the in-progress drag-over state for visual feedback.
    /// `Some(count)` when FileDropEvent::Entered is active; the count
    /// is the number of BLF-typed files in the drag (0 = all non-BLF).
    pub drag_drop_hover: Option<usize>,

    /// Paths stashed during FileDropEvent::Entered — Submit doesn't
    /// carry paths, so we read from here when Submit fires.
    pub pending_drop_stash: Vec<std::path::PathBuf>,

    /// Pending drop paths waiting for an in-flight load to finish.
    /// Drained when `loading_progress` becomes `None`.
    pub pending_drop_paths: Vec<std::path::PathBuf>,
}
```

`pending_drop_stash` (raw paths from the in-window `Entered` event, cleared on `Submit` or `Exited`) is distinct from `pending_drop_paths` (validated BLF paths queued for the next free render tick when a load was in flight).

### New module `src/view/src/handlers/drag_drop.rs`

A single entry point that both the in-window drag handler and the Dock-icon URL handler call:

```rust
/// Filter paths to .blf and return (blf_paths, skipped_names).
pub fn filter_blf_paths(paths: Vec<PathBuf>) -> (Vec<PathBuf>, Vec<String>);

/// Expand top-level folders: for each directory in `paths`, list its
/// immediate .blf children. Returns (expanded_paths, folder_summaries).
pub fn expand_folders(paths: Vec<PathBuf>) -> (Vec<PathBuf>, Vec<String>);

/// Entry point called from the drop handler. Sets `pending_drop_paths`
/// (or triggers immediately if nothing is loading), updates status_msg
/// for skipped/expanded files, and starts the load once loading_progress
/// is clear.
pub fn handle_drop(app: &mut CanViewApp, cx: &mut Context<CanViewApp>, paths: Vec<PathBuf>);
```

The `handle_drop` function:
1. Calls `filter_blf_paths` + `expand_folders`.
2. Updates `status_msg` with skipped/expanded summaries.
3. If `loading_progress.is_some()`, sets `loading_progress.is_cancelled = true` and pushes the filtered paths to `pending_drop_paths`.
4. Otherwise, calls `remove_all_files()` then runs the load pipeline (reusing the existing concurrent spawn + `apply_blf_result_append_one` loop).

### Tick loop

In `render` or a per-frame hook, check: if `loading_progress.is_none()` and `pending_drop_paths` is non-empty, drain the queue and call the load pipeline. This handles the cancel-then-restart path without spawning nested async tasks.

### In-window drag handler (`src/view/src/app/impls_rendering.rs`)

Attach to the content area div:

```rust
.on_drop::<gpui::FileDropEvent>(move |event, window, cx| {
    match event {
        FileDropEvent::Entered { paths, .. } => {
            // count BLF-typed paths; set drag_drop_hover
        }
        FileDropEvent::Exited => {
            // clear drag_drop_hover
        }
        FileDropEvent::Submit { .. } => {
            // clear drag_drop_hover; the actual paths come from a
            // stash set during Entered (Submit doesn't carry paths)
        }
        _ => {}
    }
})
```

The `Submit` event doesn't include paths — only `Entered` does. So `handle_drop` must be called from `Entered`'s stashed paths when `Submit` fires. We stash `Vec<PathBuf>` on the app state during `Entered` (e.g., `pending_drop_stash: Vec<PathBuf>`), then on `Submit` we read it.

### Dock-icon handler (`src/view/src/main.rs`)

Register `app.on_open_urls` once at startup. The handler:
1. Parses each URL string as a file path (`file://` prefix stripped).
2. Filters to `.blf`.
3. Stashes the paths in a `Rc<RefCell<Vec<PathBuf>>>` shared with the window-creation closure.
4. If a window already exists (the handler can be called after startup), uses `cx.windows()` + `window.entity()` to get the `Entity<CanViewApp>` and dispatches `handle_drop` to it.

The shared stash needs to be drained once the first window's `CanViewApp` is constructed — the construction closure checks the stash and assigns to `pending_drop_paths` on the new app.

## Data Flow

```
File drop (in-window)
  → FileDropEvent::Entered { paths }
    → stash paths + set drag_drop_hover (count of BLF)
  → FileDropEvent::Submit
    → handle_drop(stashed paths)
      → filter_blf_paths → expand_folders
      → if loading_progress.is_some(): cancel + queue
      → else: remove_all_files() + concurrent load + apply_blf_result_append_one loop

File drop (Dock icon, macOS)
  → Application::on_open_urls(urls)
    → parse file:// URLs → filter .blf
    → if window exists: dispatch handle_drop(paths)
    → else: stash paths; consumed by first window's construction
```

## Error Handling

- **Non-BLF files in drop:** skipped, `status_msg` lists up to 3 names + count.
- **Empty folder:** skipped, `status_msg` reports folder path.
- **BLF parse error during drop load:** same path as File menu — `apply_blf_result_append_one` records `errors` on the segment, the file appears with `❌` in the Loaded Files popover, and the left-side ⚠️ indicator fires (already implemented).
- **File-read error (file deleted between drop and load):** segment gets a synthetic error string from `read_blf_from_file`, surfaces in the Loaded Files popover.
- **URL parse failure in Dock handler:** log via `eprintln!`, skip the URL.

## Testing

Manual test plan (no unit tests for the drag-drop UI layer — GPUI drag events are hard to test in isolation):

1. **Single BLF drop:** drop one `.blf` onto the window — replaces any loaded file, BLF parses, log view populates.
2. **Multiple BLF drop:** drop 3 `.blf` files — `remove_all_files` runs, then all three load concurrently, status bar shows `⏳ Loading X/3`, files popover lists all three.
3. **Mixed drop:** drop 2 `.blf` + 1 `.pdf` — BLF files load, `status_msg` shows `Skipped 1 non-BLF file(s): report.pdf`.
4. **Folder drop:** drop a folder containing 5 `.blf` files at top level + 1 subfolder — 5 BLF load, subfolder is skipped, `status_msg` shows `Expanded folder: ... (5 BLF found)`.
5. **Empty folder drop:** drop an empty folder — `status_msg` shows `Folder had no BLF files: ...`.
6. **All non-BLF drop:** drop a single `.png` — `drag_drop_hover` shows `⚠️ No BLF files in drop` on hover; on release nothing loads, `status_msg` shows `Skipped 1 non-BLF file(s): ...`.
7. **Cancel mid-load + new drop:** start loading 10 BLF files, mid-load drop 3 more — `is_cancelled` flips on, once pending load drains, the 3 new files load from scratch.
8. **Large-file warning:** drop 1.5 GB of BLF files — Yes/No dialog appears; No → "Loading cancelled"; Yes → loads.
9. **Dock-icon drop (macOS):** drop a BLF on the Dock icon while app is running — same path as in-window drop.
10. **Cold-start Dock-icon drop (macOS):** drop a BLF on the Dock icon while app is closed → app launches and loads the file automatically.

## Scope (non-goals)

- **Linux/Windows Dock-icon drop** — only `on_open_urls` is wired for macOS in this iteration.
- **Drag onto specific UI regions** (e.g., onto the Plot sidebar to add a single signal source) — out of scope, drop is window-wide.
- **Drag-and-drop DBC/LDF files to add channels** — out of scope; only BLF loading is wired.
- **Drag-drop reordering of loaded files** — out of scope; order is load order.
- **Persistent "recent drops" list** — out of scope.
