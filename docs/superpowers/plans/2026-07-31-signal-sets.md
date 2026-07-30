# Signal Sets Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add named signal sets per signal library — created from the plot sidebar selection, applied via a dropdown at the top of the plot sidebar to batch-load signals for playback.

**Architecture:** New module `src/view/src/library/signal_sets.rs` owns the data model and persistence (`signal_sets.json` next to `multi_channel_config.json`). New controller `src/view/src/controllers/signal_set_controller.rs` exposes save/apply/clear/delete operations. UI changes are confined to `src/view/src/ui/views/plot_sidebar.rs` (dropdown row + save-as-set button + inline rename input + header badge). Existing `library_controller` rename/delete/apply paths get a small hook to migrate store keys and reset `active_signal_set`.

**Tech Stack:** Rust nightly, GPUI, serde, gpui-component Input. Existing patterns: `library/storage.rs` for path resolution; `controllers/config_controller.rs::save_config` for save pattern; `plot_sidebar.rs::extract_signal_items` for pure-function-with-tests pattern.

## Global Constraints

- All new code is in the `can-viewer` workspace (`src/view/Cargo.toml`'s package). No external crate additions.
- Persist `signal_sets.json` in the same directory as `multi_channel_config.json` (resolution: prefer `app.config_file_path.parent()`, else executable dir, else cwd — matches `libraries_base_path` in `src/view/src/library/storage.rs:5-17`).
- Reuse existing `gpui_component::input::Input` for the inline rename input (same pattern as the existing search box in `plot_sidebar.rs:509-516`). The inline rename uses an `Entity<InputState>` field on `CanViewerApp`, created lazily on first show and read/written via the Input's value.
- Build the dropdown as a bespoke div with show/hide state, consistent with the existing id/channel filter dropdowns in `src/view/src/app/impls_rendering.rs:300-363`. Do NOT use `crate::ui::components::dropdown::Dropdown::new().build()` — that component has no `on_select` callback (see `src/view/src/ui/components/dropdown.rs:64-133`); it's a non-functional display widget in this codebase.
- All text strings to user are Chinese (existing UI is Chinese-prefixed): `保存为信号集…`, `取消`, `信号选择`, `清除全部`, `绘制 N 个信号`, etc. New strings: `选择信号集…`, `当前库无信号集`, `先激活一个信号库`, `✕ 清除当前选择`, `已加载集 'X'（N 个信号）`, `已保存集 'X'（N 个信号）`, `集名不能为空`, `先激活一个信号库`, `没有可保存的有效信号`, `集 'X' 已存在`, `库未找到`, `集未找到`.
- Every code task ends with `cargo build -p viewer` passing AND `cargo test -p viewer <test_name>` passing. Commit after each task.
- The repo's commit style: lowercase `type(scope):` prefix, e.g. `feat(signal-sets):`, `test(signal-sets):`, `refactor(signal-sets):`. See `git log --oneline -20` for the established style.

---

## File Structure

**New files:**
- `src/view/src/library/signal_sets.rs` — data model (`SignalSetEntry`, `SignalSet`, `SignalSetStore`), pure functions (`parse_signal_id`, `build_selected_signals_from_set`), persistence (`signal_set_store_path`, `load_signal_set_store`, `save_signal_set_store`), unit tests.
- `src/view/src/controllers/signal_set_controller.rs` — controller functions (`save_current_selection_as_signal_set`, `apply_signal_set`, `clear_active_signal_set`, `delete_signal_set`).

**Modified files:**
- `src/view/src/library/mod.rs` — add `mod signal_sets;` and `pub use signal_sets::{...}`.
- `src/view/src/controllers/mod.rs` — add `pub mod signal_set_controller;` and `pub use signal_set_controller::*;`.
- `src/view/src/app/state.rs` — add four fields to `CanViewerApp` (`signal_set_store`, `active_signal_set`, `pending_signal_set_name`, `show_save_set_input`); initialize them in `new_with_maximized_state_and_bounds`; carry them through `save_runtime_state` / `restore_runtime_state` so they survive window maximize/restore.
- `src/view/src/controllers/library_controller.rs` — add hooks in `apply_version_to_mappings`, `rename_library`, `delete_library`.
- `src/view/src/ui/views/plot_sidebar.rs` — dropdown row, save-as-set button, inline rename input, header badge, checkbox-click active-set-clear hook, `build_set_dropdown_items` pure function + tests.

---

## Task 1: Data model + pure functions + persistence

**Files:**
- Create: `src/view/src/library/signal_sets.rs`
- Modify: `src/view/src/library/mod.rs` (add module + re-exports)

**Interfaces:**
- Produces:
  - `pub struct SignalSetEntry { pub channel_id: u16, pub msg_id: u32, pub signal_name: String }`
  - `pub struct SignalSet { pub name: String, pub entries: Vec<SignalSetEntry> }`
  - `pub struct SignalSetStore { pub sets_by_library: std::collections::HashMap<String, Vec<SignalSet>> }`
  - `pub fn parse_signal_id(sig_id: &str) -> Option<SignalSetEntry>`
  - `pub fn build_selected_signals_from_set(set: &SignalSet, channel_type: crate::models::ChannelType) -> Vec<String>`
  - `pub fn signal_set_store_path(config_file_path: Option<&std::path::Path>) -> std::path::PathBuf`
  - `pub fn load_signal_set_store(config_file_path: Option<&std::path::Path>) -> SignalSetStore`
  - `pub fn save_signal_set_store(store: &SignalSetStore, config_file_path: Option<&std::path::Path>) -> Result<(), String>`

- [ ] **Step 1: Write the failing tests**

Create `src/view/src/library/signal_sets.rs` with just the tests (the module will fail to compile because structs/functions don't exist yet, which is the point).

```rust
//! Signal sets — named collections of (channel, msg_id, signal) tuples per library.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SignalSetEntry {
    pub channel_id: u16,
    pub msg_id: u32,
    pub signal_name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SignalSet {
    pub name: String,
    pub entries: Vec<SignalSetEntry>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SignalSetStore {
    #[serde(default)]
    pub sets_by_library: HashMap<String, Vec<SignalSet>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ChannelType;

    #[test]
    fn test_parse_signal_id_valid_can() {
        let entry = parse_signal_id("CAN:1:0x100:EngineSpeed").unwrap();
        assert_eq!(entry.channel_id, 1);
        assert_eq!(entry.msg_id, 256);
        assert_eq!(entry.signal_name, "EngineSpeed");
    }

    #[test]
    fn test_parse_signal_id_valid_lin() {
        let entry = parse_signal_id("LIN:2:0x20:Speed").unwrap();
        assert_eq!(entry.channel_id, 2);
        assert_eq!(entry.msg_id, 32);
        assert_eq!(entry.signal_name, "Speed");
    }

    #[test]
    fn test_parse_signal_id_decimal_msg_id() {
        let entry = parse_signal_id("CAN:1:256:Speed").unwrap();
        assert_eq!(entry.msg_id, 256);
    }

    #[test]
    fn test_parse_signal_id_invalid_bus() {
        assert!(parse_signal_id("J1939:1:0x100:Speed").is_none());
    }

    #[test]
    fn test_parse_signal_id_empty_signal_name() {
        assert!(parse_signal_id("CAN:1:0x100:").is_none());
    }

    #[test]
    fn test_parse_signal_id_too_few_parts() {
        assert!(parse_signal_id("CAN:1:0x100").is_none());
        assert!(parse_signal_id("CAN:1").is_none());
        assert!(parse_signal_id("CAN").is_none());
    }

    #[test]
    fn test_parse_signal_id_bad_channel() {
        assert!(parse_signal_id("CAN:abc:0x100:Speed").is_none());
    }

    #[test]
    fn test_parse_signal_id_bad_msg_id() {
        assert!(parse_signal_id("CAN:1:0xGG:Speed").is_none());
    }

    #[test]
    fn test_build_selected_signals_from_set_empty() {
        let set = SignalSet { name: "empty".into(), entries: Vec::new() };
        assert!(build_selected_signals_from_set(&set, ChannelType::CAN).is_empty());
    }

    #[test]
    fn test_build_selected_signals_from_set_can() {
        let set = SignalSet {
            name: "s".into(),
            entries: vec![SignalSetEntry {
                channel_id: 1,
                msg_id: 256,
                signal_name: "EngineSpeed".into(),
            }],
        };
        let out = build_selected_signals_from_set(&set, ChannelType::CAN);
        assert_eq!(out, vec!["CAN:1:0x100:EngineSpeed".to_string()]);
    }

    #[test]
    fn test_build_selected_signals_from_set_lin() {
        let set = SignalSet {
            name: "s".into(),
            entries: vec![SignalSetEntry {
                channel_id: 2,
                msg_id: 32,
                signal_name: "Speed".into(),
            }],
        };
        let out = build_selected_signals_from_set(&set, ChannelType::LIN);
        assert_eq!(out, vec!["LIN:2:0x20:Speed".to_string()]);
    }

    #[test]
    fn test_build_selected_signals_preserves_order() {
        let set = SignalSet {
            name: "s".into(),
            entries: vec![
                SignalSetEntry { channel_id: 1, msg_id: 256, signal_name: "A".into() },
                SignalSetEntry { channel_id: 1, msg_id: 512, signal_name: "B".into() },
                SignalSetEntry { channel_id: 2, msg_id: 768, signal_name: "C".into() },
            ],
        };
        let out = build_selected_signals_from_set(&set, ChannelType::CAN);
        assert_eq!(out[0], "CAN:1:0x100:A");
        assert_eq!(out[1], "CAN:1:0x200:B");
        assert_eq!(out[2], "CAN:2:0x300:C");
    }

    #[test]
    fn test_store_roundtrip() {
        let mut store = SignalSetStore::default();
        store.sets_by_library.insert(
            "lib_a".into(),
            vec![
                SignalSet {
                    name: "set1".into(),
                    entries: vec![SignalSetEntry {
                        channel_id: 1,
                        msg_id: 256,
                        signal_name: "EngineSpeed".into(),
                    }],
                },
                SignalSet {
                    name: "set2".into(),
                    entries: vec![
                        SignalSetEntry { channel_id: 1, msg_id: 256, signal_name: "A".into() },
                        SignalSetEntry { channel_id: 2, msg_id: 512, signal_name: "B".into() },
                    ],
                },
            ],
        );
        store.sets_by_library.insert("lib_b".into(), Vec::new());

        let json = serde_json::to_string(&store).unwrap();
        let back: SignalSetStore = serde_json::from_str(&json).unwrap();
        assert_eq!(store, back);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p viewer signal_sets`
Expected: FAIL to compile (`parse_signal_id`, `build_selected_signals_from_set` not defined).

- [ ] **Step 3: Add module + exports**

Edit `src/view/src/library/mod.rs`:

After line 5 (`mod storage;`), add:
```rust
mod signal_sets;
```

In the existing `pub use storage::{...};` block (lines 7-11), add a new re-export below it:
```rust
pub use signal_sets::{
    SignalSet, SignalSetEntry, SignalSetStore, build_selected_signals_from_set,
    load_signal_set_store, parse_signal_id, save_signal_set_store, signal_set_store_path,
};
```

- [ ] **Step 4: Implement the pure functions + persistence**

Append to `src/view/src/library/signal_sets.rs` (above the `#[cfg(test)]` block):

```rust
/// Parse a `selected_signals`-format string ("BUS:CH:MSG_ID:SIG_NAME")
/// into a `SignalSetEntry`. Returns None on malformed input.
///
/// `bus` is discarded — the parent library's `channel_type` decides bus.
pub fn parse_signal_id(sig_id: &str) -> Option<SignalSetEntry> {
    let parts: Vec<&str> = sig_id.split(':').collect();
    if parts.len() < 4 {
        return None;
    }
    let bus = parts[0];
    if bus != "CAN" && bus != "LIN" {
        return None;
    }
    let channel_id = parts[1].parse::<u16>().ok()?;
    let msg_id_str = parts[2];
    let msg_id = if let Some(hex) = msg_id_str.strip_prefix("0x") {
        u32::from_str_radix(hex, 16).ok()?
    } else {
        msg_id_str.parse::<u32>().ok()?
    };
    let signal_name = parts[3..].join(":");
    if signal_name.is_empty() {
        return None;
    }
    Some(SignalSetEntry {
        channel_id,
        msg_id,
        signal_name,
    })
}

/// Rebuild `selected_signals`-format strings from a set + parent library's channel_type.
pub fn build_selected_signals_from_set(
    set: &SignalSet,
    channel_type: crate::models::ChannelType,
) -> Vec<String> {
    let bus = match channel_type {
        crate::models::ChannelType::CAN => "CAN",
        crate::models::ChannelType::LIN => "LIN",
    };
    set.entries
        .iter()
        .map(|e| format!("{}:{}:0x{:x}:{}", bus, e.channel_id, e.msg_id, e.signal_name))
        .collect()
}

/// Resolve the path to `signal_sets.json` next to `multi_channel_config.json`.
pub fn signal_set_store_path(config_file_path: Option<&Path>) -> PathBuf {
    if let Some(dir) = config_file_path.and_then(|p| p.parent()) {
        return dir.join("signal_sets.json");
    }
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            return exe_dir.join("signal_sets.json");
        }
    }
    PathBuf::from("signal_sets.json")
}

/// Load the store from disk. Missing file → empty store.
pub fn load_signal_set_store(config_file_path: Option<&Path>) -> SignalSetStore {
    let path = signal_set_store_path(config_file_path);
    if !path.exists() {
        return SignalSetStore::default();
    }
    match std::fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(e) => {
            eprintln!("⚠️  Failed to read signal_sets.json: {}", e);
            SignalSetStore::default()
        }
    }
}

/// Save the store to disk. Errors return a String.
pub fn save_signal_set_store(
    store: &SignalSetStore,
    config_file_path: Option<&Path>,
) -> Result<(), String> {
    let path = signal_set_store_path(config_file_path);
    let content = serde_json::to_string_pretty(store)
        .map_err(|e| format!("Failed to serialize signal sets: {}", e))?;
    std::fs::write(&path, content).map_err(|e| {
        format!("Failed to write {}: {}", path.display(), e)
    })
}
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test -p viewer signal_sets`
Expected: PASS — all 13 tests green.

- [ ] **Step 6: Build to verify no compile errors elsewhere**

Run: `cargo build -p viewer`
Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add src/view/src/library/signal_sets.rs src/view/src/library/mod.rs
git commit -m "feat(signal-sets): add data model, pure parsers, and JSON persistence"
```

---

## Task 2: App state fields + runtime save/restore

**Files:**
- Modify: `src/view/src/app/state.rs`
  - Add 4 fields to `CanViewerApp` (after `pending_add_channel_focus` at line 257, before Server state at line 259)
  - Initialize them in `new_with_maximized_state_and_bounds` (in the `Self { ... }` block, end around line 438)
  - Add them to `RuntimeState` struct (around line 24-45)
  - Save them in `save_runtime_state` (around line 444-471)
  - Restore them in `restore_runtime_state` (around line 475-508)

**Interfaces:**
- Consumes: `crate::library::signal_sets::SignalSetStore`, `crate::library::signal_sets::load_signal_set_store`.
- Produces: `CanViewerApp` now has `pub signal_set_store: SignalSetStore`, `pub active_signal_set: Option<(String, String)>`, `pub pending_signal_set_name: Option<String>`, `pub show_save_set_input: bool` — all readable/writable from controller and UI code.

- [ ] **Step 1: Add fields to `CanViewerApp` struct**

In `src/view/src/app/state.rs`, find the `pending_add_channel_focus` field (line 257):

```rust
    // Add-channel form Enter-key focus chain: set by PressEnter subscribe,
    // consumed by render() which has window access.
    pub pending_add_channel_focus: Option<PendingAddChannelFocus>,
```

Below it (before the `// Server state` comment at line 259), insert:

```rust

    // Signal sets — named collections of (ch, msg_id, signal) per library
    pub signal_set_store: crate::library::signal_sets::SignalSetStore,
    /// Currently active set as (library_id, set_name); cleared on library switch / manual edit
    pub active_signal_set: Option<(String, String)>,
    /// Inline input buffer when saving the current selection as a set
    pub pending_signal_set_name: Option<String>,
    /// Whether the inline "save as set" input row is currently shown
    pub show_save_set_input: bool,
```

- [ ] **Step 2: Add fields to `RuntimeState` struct**

Find `RuntimeState` (lines 22-45) — the struct that survives window maximize/restore. After the existing `expanded_messages` field (line 44), add:

```rust
    pub expanded_messages: std::collections::HashSet<(u16, u32)>, // (ch_id, msg_id)
    // Signal set session state
    pub signal_set_store: crate::library::signal_sets::SignalSetStore,
    pub active_signal_set: Option<(String, String)>,
```

Note: `pending_signal_set_name` and `show_save_set_input` are transient UI state — they don't need to survive window recreate (matching the pattern of `pending_add_channel_focus` which is also not in `RuntimeState`). They reset to `None` / `false` on recreate, which is the desired behavior (a recreate mid-typing should not preserve the half-typed name).

- [ ] **Step 3: Initialize the new fields in `new_with_maximized_state_and_bounds`**

In the `Self { ... }` block ending around line 438-439, find the last field:

```rust
            pending_import: None,
        }
    }
```

Before `pending_import: None,` add:

```rust
            // Signal sets
            signal_set_store: crate::library::signal_sets::load_signal_set_store(None),
            active_signal_set: None,
            pending_signal_set_name: None,
            show_save_set_input: false,
```

**Note:** `load_signal_set_store(None)` is called at construction time when `config_file_path` is not yet known. This falls back to the executable-dir path. After `load_startup_config` runs and sets `config_file_path`, the store is already in memory; subsequent save calls will pass `app.config_file_path.as_deref()` and land in the right directory. The startup load on first launch returns an empty store; for an existing `signal_sets.json` next to the exe, this load picks it up correctly.

- [ ] **Step 4: Save the new fields in `save_runtime_state`**

Find `save_runtime_state` (around line 444). At the end of the `RuntimeState { ... }` literal (before the closing `}` around line 470), add:

```rust
            expanded_messages: self.expanded_messages.clone(),
            signal_set_store: self.signal_set_store.clone(),
            active_signal_set: self.active_signal_set.clone(),
        }
```

- [ ] **Step 5: Restore the new fields in `restore_runtime_state`**

Find `restore_runtime_state` (around line 475). Before the final `eprintln!("✅ State restored...` (around line 506), add:

```rust
        self.expanded_messages = state.expanded_messages;
        self.signal_set_store = state.signal_set_store;
        self.active_signal_set = state.active_signal_set;
        eprintln!("✅ State restored. Now have: {:?} view, {} files, {} messages, {} plot series",
```

- [ ] **Step 6: Build to verify compile**

Run: `cargo build -p viewer`
Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add src/view/src/app/state.rs
git commit -m "feat(signal-sets): add signal_set_store and active_signal_set to app state"
```

---

## Task 3: Signal set controller (save / apply / clear / delete)

**Files:**
- Create: `src/view/src/controllers/signal_set_controller.rs`
- Modify: `src/view/src/controllers/mod.rs` (add module + re-export)

**Interfaces:**
- Consumes:
  - `crate::library::signal_sets::{SignalSet, SignalSetEntry, parse_signal_id, build_selected_signals_from_set, save_signal_set_store}` (from Task 1)
  - `crate::app::CanViewerApp` with fields `selected_signals`, `app_config.active_library_id`, `library_manager`, `signal_set_store`, `active_signal_set`, `show_save_set_input`, `pending_signal_set_name`, `status_msg`, `config_file_path` (from Task 2)
  - `crate::ui::views::chart_view::extract_and_update_series_data(&mut CanViewerApp)` (existing — see `src/view/src/ui/views/chart_view.rs:1486`)
- Produces:
  - `pub fn save_current_selection_as_signal_set(app: &mut CanViewerApp, name: &str, cx: &mut Context<CanViewerApp>)`
  - `pub fn apply_signal_set(app: &mut CanViewerApp, library_id: &str, set_name: &str, cx: &mut Context<CanViewerApp>)`
  - `pub fn clear_active_signal_set(app: &mut CanViewerApp, cx: &mut Context<CanViewerApp>)`
  - `pub fn delete_signal_set(app: &mut CanViewerApp, library_id: &str, set_name: &str, cx: &mut Context<CanViewerApp>)`

- [ ] **Step 1: Add module + exports to controllers/mod.rs**

Edit `src/view/src/controllers/mod.rs`:

After line 6 (`pub mod library_controller;`), add:
```rust
pub mod signal_set_controller;
```

After line 14 (`pub use ui_controller::*;`), add:
```rust
pub use signal_set_controller::*;
```

- [ ] **Step 2: Write the controller file**

Create `src/view/src/controllers/signal_set_controller.rs`:

```rust
//! Signal set controller — business logic for save/apply/clear/delete operations.

use crate::app::CanViewerApp;
use crate::library::signal_sets::{
    SignalSet, build_selected_signals_from_set, parse_signal_id, save_signal_set_store,
};
use gpui::Context;

/// Save the current `selected_signals` as a new signal set under the active library.
/// Validates name, requires an active library, requires ≥1 parseable signal.
pub fn save_current_selection_as_signal_set(
    app: &mut CanViewerApp,
    name: &str,
    cx: &mut Context<CanViewerApp>,
) {
    let name = name.trim().to_string();
    if name.is_empty() {
        app.status_msg = "集名不能为空".into();
        cx.notify();
        return;
    }

    let library_id = match &app.app_config.active_library_id {
        Some(id) => id.clone(),
        None => {
            app.status_msg = "先激活一个信号库".into();
            cx.notify();
            return;
        }
    };

    // Parse selected_signals → entries (silently skip unparseable ones)
    let mut entries: Vec<_> = Vec::new();
    for sig_id in &app.selected_signals {
        match parse_signal_id(sig_id) {
            Some(entry) => entries.push(entry),
            None => eprintln!("⚠️  Skipping unparseable signal_id: {}", sig_id),
        }
    }
    if entries.is_empty() {
        app.status_msg = "没有可保存的有效信号".into();
        cx.notify();
        return;
    }

    // Check duplicate name
    let sets = app
        .signal_set_store
        .sets_by_library
        .entry(library_id.clone())
        .or_default();
    if sets.iter().any(|s| s.name == name) {
        app.status_msg = format!("集 '{}' 已存在", name).into();
        cx.notify();
        return;
    }

    let count = entries.len();
    sets.push(SignalSet {
        name: name.clone(),
        entries,
    });
    if let Err(e) = save_signal_set_store(&app.signal_set_store, app.config_file_path.as_deref()) {
        eprintln!("❌ Failed to save signal sets: {}", e);
        app.status_msg = format!("警告：无法保存信号集到磁盘 ({})", e).into();
    } else {
        app.status_msg = format!("已保存集 '{}'（{} 个信号）", name, count).into();
    }
    app.show_save_set_input = false;
    app.pending_signal_set_name = None;
    cx.notify();
}

/// Apply a named set to `selected_signals` and trigger plot refresh.
pub fn apply_signal_set(
    app: &mut CanViewerApp,
    library_id: &str,
    set_name: &str,
    cx: &mut Context<CanViewerApp>,
) {
    let library = match app.library_manager.find_library(library_id) {
        Some(lib) => lib.clone(),
        None => {
            app.status_msg = "库未找到".into();
            cx.notify();
            return;
        }
    };
    let set = match app
        .signal_set_store
        .sets_by_library
        .get(library_id)
        .and_then(|sets| sets.iter().find(|s| s.name == set_name))
    {
        Some(s) => s.clone(),
        None => {
            app.status_msg = "集未找到".into();
            cx.notify();
            return;
        }
    };

    app.selected_signals.clear();
    let rebuilt = build_selected_signals_from_set(&set, library.channel_type);
    app.selected_signals.extend(rebuilt);
    app.active_signal_set = Some((library_id.to_string(), set_name.to_string()));

    // Trigger plot refresh
    crate::ui::views::chart_view::extract_and_update_series_data(app);

    app.status_msg = format!("已加载集 '{}'（{} 个信号）", set_name, set.entries.len()).into();
    cx.notify();
}

/// Clear the active set selection AND clear selected_signals.
/// Triggered by the dropdown's "✕" item.
pub fn clear_active_signal_set(app: &mut CanViewerApp, cx: &mut Context<CanViewerApp>) {
    app.active_signal_set = None;
    app.selected_signals.clear();
    crate::ui::views::chart_view::extract_and_update_series_data(app);
    app.status_msg = "已清除集选择".into();
    cx.notify();
}

/// Delete a set from the store (no UI in this spec; symmetric API).
pub fn delete_signal_set(
    app: &mut CanViewerApp,
    library_id: &str,
    set_name: &str,
    cx: &mut Context<CanViewerApp>,
) {
    if let Some(sets) = app.signal_set_store.sets_by_library.get_mut(library_id) {
        if let Some(pos) = sets.iter().position(|s| s.name == set_name) {
            sets.remove(pos);
            let _ = save_signal_set_store(&app.signal_set_store, app.config_file_path.as_deref());
            if let Some((lid, _)) = &app.active_signal_set {
                if lid == library_id {
                    app.active_signal_set = None;
                }
            }
            app.status_msg = format!("已删除集 '{}'", set_name).into();
            cx.notify();
            return;
        }
    }
    app.status_msg = "集未找到".into();
    cx.notify();
}
```

- [ ] **Step 3: Build to verify compile**

Run: `cargo build -p viewer`
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add src/view/src/controllers/signal_set_controller.rs src/view/src/controllers/mod.rs
git commit -m "feat(signal-sets): add save/apply/clear/delete controller functions"
```

---

## Task 4: Library controller hooks (apply / rename / delete)

**Files:**
- Modify: `src/view/src/controllers/library_controller.rs`
  - `apply_version_to_mappings` (lines 168-235): reset `active_signal_set = None` at end
  - `rename_library` (lines 129-152 of `src/view/src/library/mod.rs::LibraryManager::rename_library` is the domain method — but the controller path is `crate::controllers::library_controller::rename_library` doesn't exist; check `library_controller.rs` for rename entry point)

**Pre-step: Locate the rename entry point**

Search the codebase for where rename is invoked from the UI. Run:

```bash
grep -n "rename_library\|rename_version" src/view/src/controllers/library_controller.rs src/view/src/ui/views/library_management.rs src/view/src/app/impls.rs
```

If a controller `rename_library` exists in `library_controller.rs`, hook there. If rename is invoked directly from the UI handler (no controller wrapper), add the migration logic at the UI call site or add a new controller wrapper. The spec assumes a controller wrapper exists; verify and adjust.

**Interfaces:**
- Consumes:
  - `app.signal_set_store.sets_by_library` (from Task 2)
  - `app.active_signal_set` (from Task 2)
  - `save_signal_set_store(&store, app.config_file_path.as_deref())` (from Task 1)

- [ ] **Step 1: Hook `apply_version_to_mappings`**

In `src/view/src/controllers/library_controller.rs`, find `apply_version_to_mappings` (lines 168-235). After the final `cx.notify();` (around line 234), but before the function's closing `}`, insert:

```rust
    // Active library/version changed — old signal set no longer applies
    app.active_signal_set = None;
```

This goes inside the function body, after `crate::controllers::config_controller::save_config(app, cx);` (line 231) and before `app.status_msg = format!(...)` (line 233). Place it right before `app.status_msg =`:

```rust
    // Save config
    crate::controllers::config_controller::save_config(app, cx);

    // Active library/version changed — old signal set no longer applies
    app.active_signal_set = None;

    app.status_msg = format!("✅ Applied version {} to {} channels", version_name, count).into();
    cx.notify();
}
```

- [ ] **Step 2: Hook `delete_library`**

In `src/view/src/controllers/library_controller.rs::delete_library` (lines 44-61), inside the `Ok(_)` branch after the existing `cx.notify();` and before the closing `}`, add the signal set cleanup:

```rust
    match app.library_manager.delete_library(library_id, &app.app_config.mappings) {
        Ok(_) => {
            app.status_msg = format!("Library deleted").into();
            if app.selected_library_id.as_ref() == Some(&library_id.to_string()) {
                app.selected_library_id = None;
            }

            // Drop signal sets owned by the deleted library
            if app.signal_set_store.sets_by_library.remove(library_id).is_some() {
                let _ = crate::library::save_signal_set_store(
                    &app.signal_set_store,
                    app.config_file_path.as_deref(),
                );
            }
            if let Some((lid, _)) = &app.active_signal_set {
                if lid == library_id {
                    app.active_signal_set = None;
                }
            }

            cx.notify();
        }
```

- [ ] **Step 3: Hook rename path**

Run to find the rename entry point:

```bash
grep -n "rename_library\|rename_version" src/view/src/controllers/library_controller.rs src/view/src/ui/views/library_management.rs src/view/src/app/impls.rs
```

Expected output will reveal where the rename call lives. Two scenarios:

**Scenario A** (controller wrapper exists): If `controllers/library_controller.rs` has a `pub fn rename_library(app, ...)`, add the store-key migration inside that function after `app.library_manager.rename_library(...)` succeeds:

```rust
// After successful rename_library_manager call, before cx.notify():
if let Some(sets) = app.signal_set_store.sets_by_library.remove(&old_id) {
    app.signal_set_store.sets_by_library.insert(new_id.clone(), sets);
    let _ = crate::library::save_signal_set_store(
        &app.signal_set_store,
        app.config_file_path.as_deref(),
    );
}
if let Some((lid, _)) = &app.active_signal_set {
    if lid == &old_id {
        let set_name = app.active_signal_set.as_ref().unwrap().1.clone();
        app.active_signal_set = Some((new_id.clone(), set_name));
    }
}
```

**Scenario B** (rename is invoked directly from UI): Find the call site (likely in `library_management.rs` or `impls.rs`). The cleanest fix: add a new `pub fn rename_library(app, old_id, new_name, cx)` wrapper in `controllers/library_controller.rs` that calls `app.library_manager.rename_library(old_id, new_name)`, then migrates the signal_set_store as in Scenario A, then saves config. Update the UI call site to invoke the new controller wrapper.

Choose based on what `grep` finds. Either way, the migration code is identical.

- [ ] **Step 4: Build to verify compile**

Run: `cargo build -p viewer`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/view/src/controllers/library_controller.rs
git commit -m "feat(signal-sets): hook library apply/rename/delete to migrate set store"
```

---

## Task 5: `build_set_dropdown_items` pure function + tests

**Files:**
- Modify: `src/view/src/ui/views/plot_sidebar.rs`
  - Add `SetDropdownItem` enum and `build_set_dropdown_items` function (after the existing `extract_signal_items` function, around line 453)
  - Add tests in the existing `#[cfg(test)] mod tests` block at the bottom (around line 625)

**Interfaces:**
- Consumes:
  - `crate::app::CanViewerApp` with `app_config.active_library_id` (existing), `signal_set_store.sets_by_library` (Task 2), `active_signal_set` (Task 2)
- Produces:
  - `pub enum SetDropdownItem { Placeholder(String), Set { name: String, count: usize }, ClearActive }`
  - `pub fn build_set_dropdown_items(app: &CanViewerApp) -> Vec<SetDropdownItem>`

- [ ] **Step 1: Write the failing tests**

In `src/view/src/ui/views/plot_sidebar.rs`, scroll to the `#[cfg(test)] mod tests` block (starts at line 625). Inside the `mod tests` block (after the existing helpers around line 631, before the first `#[test]`), add:

```rust
    /// App with active library `lib_x` and a given list of (set_name, count) sets under it.
    fn app_with_sets(active_lib_id: Option<&str>, sets: Vec<(&str, usize)>, active_set: Option<(&str, &str)>) -> CanViewerApp {
        let mut app = CanViewerApp::new_state();
        if let Some(id) = active_lib_id {
            app.app_config.active_library_id = Some(id.to_string());
            let store_sets: Vec<_> = sets.into_iter().map(|(name, n)| crate::library::signal_sets::SignalSet {
                name: name.to_string(),
                entries: (0..n).map(|i| crate::library::signal_sets::SignalSetEntry {
                    channel_id: 1, msg_id: i as u32, signal_name: format!("sig{}", i),
                }).collect(),
            }).collect();
            app.signal_set_store.sets_by_library.insert(id.to_string(), store_sets);
        }
        if let Some((lid, sname)) = active_set {
            app.active_signal_set = Some((lid.to_string(), sname.to_string()));
        }
        app
    }

    #[test]
    fn build_set_dropdown_items_no_active_library() {
        let app = app_with_sets(None, Vec::new(), None);
        let items = build_set_dropdown_items(&app);
        assert_eq!(items.len(), 1);
        match &items[0] {
            SetDropdownItem::Placeholder(msg) => assert_eq!(msg, "先激活一个信号库"),
            other => panic!("expected Placeholder, got {:?}", other),
        }
    }

    #[test]
    fn build_set_dropdown_items_active_lib_no_sets() {
        let app = app_with_sets(Some("lib_x"), Vec::new(), None);
        let items = build_set_dropdown_items(&app);
        assert_eq!(items.len(), 1);
        match &items[0] {
            SetDropdownItem::Placeholder(msg) => assert_eq!(msg, "当前库无信号集"),
            other => panic!("expected Placeholder, got {:?}", other),
        }
    }

    #[test]
    fn build_set_dropdown_items_active_lib_with_sets_no_active() {
        let app = app_with_sets(Some("lib_x"), vec![("Engine", 2), ("Battery", 3)], None);
        let items = build_set_dropdown_items(&app);
        assert_eq!(items.len(), 2);
        assert!(matches!(&items[0], SetDropdownItem::Set { name, count: 2 } if name == "Engine"));
        assert!(matches!(&items[1], SetDropdownItem::Set { name, count: 3 } if name == "Battery"));
    }

    #[test]
    fn build_set_dropdown_items_with_active_set_appends_clear() {
        let app = app_with_sets(
            Some("lib_x"),
            vec![("Engine", 2)],
            Some(("lib_x", "Engine")),
        );
        let items = build_set_dropdown_items(&app);
        assert_eq!(items.len(), 2);
        assert!(matches!(&items[0], SetDropdownItem::Set { name, .. } if name == "Engine"));
        assert!(matches!(&items[1], SetDropdownItem::ClearActive));
    }

    #[test]
    fn build_set_dropdown_items_active_set_on_other_lib_still_lists() {
        // Edge case: active_signal_set points at a non-active library.
        // build_set_dropdown_items is driven by active_library_id for listing;
        // ClearActive is appended whenever active_signal_set.is_some(), regardless of lib match.
        // (The dropdown UI itself only shows ClearActive when the active set is on the
        // currently-active library, but that's a UI concern, not this pure function's.)
        let app = app_with_sets(
            Some("lib_x"),
            vec![("Engine", 2)],
            Some(("lib_other", "OtherSet")),
        );
        let items = build_set_dropdown_items(&app);
        assert_eq!(items.len(), 2);
        assert!(matches!(&items[1], SetDropdownItem::ClearActive));
    }
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p viewer plot_sidebar::tests::build_set_dropdown`
Expected: FAIL to compile (`SetDropdownItem`, `build_set_dropdown_items` not defined).

- [ ] **Step 3: Add the enum and function**

In `src/view/src/ui/views/plot_sidebar.rs`, after the closing `}` of `extract_signal_items` (line 453) and before `pub fn render_signal_sidebar` (line 457), insert:

```rust
/// Items produced by `build_set_dropdown_items` for the signal-sets dropdown.
#[derive(Clone, Debug, PartialEq)]
pub enum SetDropdownItem {
    /// Disabled placeholder (no active library / no sets)
    Placeholder(String),
    /// A named set with its entry count
    Set { name: String, count: usize },
    /// "✕ Clear set selection" trailing item
    ClearActive,
}

/// Pure function: build the list of items shown in the signal-sets dropdown.
pub fn build_set_dropdown_items(app: &CanViewerApp) -> Vec<SetDropdownItem> {
    let Some(lib_id) = &app.app_config.active_library_id else {
        return vec![SetDropdownItem::Placeholder("先激活一个信号库".into())];
    };
    let Some(sets) = app.signal_set_store.sets_by_library.get(lib_id) else {
        return vec![SetDropdownItem::Placeholder("当前库无信号集".into())];
    };
    if sets.is_empty() {
        return vec![SetDropdownItem::Placeholder("当前库无信号集".into())];
    }
    let mut items: Vec<SetDropdownItem> = sets
        .iter()
        .map(|s| SetDropdownItem::Set {
            name: s.name.clone(),
            count: s.entries.len(),
        })
        .collect();
    if app.active_signal_set.is_some() {
        items.push(SetDropdownItem::ClearActive);
    }
    items
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p viewer plot_sidebar::tests::build_set_dropdown`
Expected: PASS — all 5 tests green.

- [ ] **Step 5: Build to verify no wider compile errors**

Run: `cargo build -p viewer`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src/view/src/ui/views/plot_sidebar.rs
git commit -m "test(signal-sets): add build_set_dropdown_items pure function with tests"
```

---

## Task 6: Plot sidebar UI — dropdown row + save-as-set button + inline rename + header badge + checkbox hook

**Files:**
- Modify: `src/view/src/ui/views/plot_sidebar.rs`:
  - Add `pub show_signal_set_dropdown: bool` is NOT a new app field — we'll add it. **Re-check Task 2:** `show_save_set_input` is in app state but `show_signal_set_dropdown` is a new transient UI state we need for the dropdown open/close. Add it as a new field in Task 2's pattern; for this task we'll add it inline. **Decision:** add `pub show_signal_set_dropdown: bool` to `CanViewerApp` in `state.rs` (small follow-up edit; document here).
  - `render_signal_sidebar` (around line 457): insert dropdown row between header and search box
  - `render_sidebar_item::SignalItem` (around line 222-233): clear `active_signal_set` on checkbox toggle
  - Bottom action bar (around line 561-622): add third button "保存为信号集…" and conditional inline rename input
  - Header section (around line 474-497): add active-set badge

**Sub-step 0: Add `show_signal_set_dropdown` and `signal_set_name_input` fields to app state**

Edit `src/view/src/app/state.rs`:

In `CanViewerApp`, after the `show_save_set_input: bool` field added in Task 2, add:

```rust
    /// Whether the signal-sets dropdown menu is currently open
    pub show_signal_set_dropdown: bool,
    /// Inline-input entity for the "save as signal set" name field.
    /// Lazily created on first show (see render_signal_set_dropdown_row
    /// initialization in impls_rendering.rs render loop).
    pub signal_set_name_input: Option<gpui::Entity<gpui_component::input::InputState>>,
```

In `new_with_maximized_state_and_bounds` initialization block, after `show_save_set_input: false,`, add:

```rust
            show_signal_set_dropdown: false,
            signal_set_name_input: None,
```

(Don't add either to `RuntimeState` — these are transient UI state, like `pending_add_channel_focus` and `signal_search_input`.)

**Sub-step 0b: Initialize `signal_set_name_input` in the render loop**

In `src/view/src/app/impls_rendering.rs`, find the existing initialization block for `signal_search_input` (around line 1686-1706, inside the `render` method where `window: &mut Window` is in scope). Immediately after the `signal_search_input` initialization block (after line 1707 `}`), add:

```rust
        // Initialize signal set name input (lazily, on first render)
        if self.signal_set_name_input.is_none() {
            let input = cx.new(|cx| InputState::new(window, cx).placeholder("输入集名并按回车保存…"));
            cx.subscribe(&input, |this, input, event, cx| {
                if let gpui_component::input::InputEvent::PressEnter { .. } = event {
                    let name = input.read(cx).value().to_string();
                    crate::controllers::signal_set_controller::save_current_selection_as_signal_set(
                        this, &name, cx,
                    );
                }
            })
            .detach();
            self.signal_set_name_input = Some(input);
        }
```

Note: `window` is in scope here (it's a parameter of `render`). The `PressEnter` subscribe handles Enter-to-save. Esc-to-cancel is handled by the `取消` button in `render_save_set_input_row` (see Step 2 below).

- [ ] **Step 1: Add the dropdown row in `render_signal_sidebar`**

In `src/view/src/ui/views/plot_sidebar.rs`, find `render_signal_sidebar` (line 457). Locate the header child (lines 474-497). The current structure is:

```rust
.child(
    // Sidebar Header
    div()
        .px_4()...
)
.child(
    // Search Box
    div()...
)
```

Between the header `.child(...)` and the search box `.child(...)`, insert the dropdown row:

```rust
.child(render_signal_set_dropdown_row(app, view.clone(), cx))
```

Then add the helper function below `render_signal_sidebar` (before `render_sidebar_item` or wherever the existing helpers live; placement doesn't matter as long as it's in scope):

```rust
/// Render the signal-sets dropdown row: trigger button + popup menu.
fn render_signal_set_dropdown_row(
    app: &CanViewerApp,
    view: Entity<CanViewerApp>,
    cx: &mut Context<CanViewerApp>,
) -> impl IntoElement {
    let items = build_set_dropdown_items(app);
    let is_open = app.show_signal_set_dropdown;
    let active_set_label = app.active_signal_set.as_ref()
        .and_then(|(lid, sname)| {
            if app.app_config.active_library_id.as_ref() == Some(lid) {
                Some(sname.clone())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "选择信号集…".to_string());

    // Determine disabled state from the items list
    let is_disabled = matches!(items.first(), Some(SetDropdownItem::Placeholder(_)));

    let view_for_toggle = view.clone();
    let view_for_items = view.clone();

    let trigger = div()
        .w_full()
        .px_4()
        .py_2()
        .border_b_1()
        .border_color(rgb(0x27272a))
        .flex()
        .items_center()
        .gap_2()
        .when(!is_disabled, |el| el.cursor_pointer().hover(|s| s.bg(rgb(0x1a1a1b))))
        .when(is_disabled, |el| el.opacity(0.5))
        .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
            if !this.show_signal_set_dropdown {
                this.show_signal_set_dropdown = true;
                cx.notify();
            }
        }))
        .child(
            div()
                .text_xs()
                .text_color(rgb(0x71717a))
                .w(px(56.0))
                .child("信号集:")
        )
        .child(
            div()
                .flex_1()
                .text_xs()
                .text_color(if is_disabled { rgb(0x52525b) } else { rgb(0xe4e4e7) })
                .child(active_set_label.clone())
        )
        .child(
            div()
                .text_xs()
                .text_color(rgb(0x71717a))
                .child(if is_disabled { "" } else if is_open { "▴" } else { "▾" })
        );

    // Popup menu (only when open)
    let popup = if is_open && !is_disabled {
        let mut children: Vec<AnyElement> = Vec::new();
        for (idx, item) in items.iter().enumerate() {
            let view = view_for_items.clone();
            let elem = match item {
                SetDropdownItem::Placeholder(msg) => div()
                    .px_4().py_2()
                    .text_xs().text_color(rgb(0x52525b))
                    .child(msg.clone())
                    .into_any_element(),
                SetDropdownItem::Set { name, count } => {
                    let name = name.clone();
                    let count = *count;
                    div()
                        .px_4().py_2()
                        .cursor_pointer().hover(|s| s.bg(rgb(0x1f1f22)))
                        .on_mouse_down(MouseButton::Left, {
                            let view = view.clone();
                            move |_, _, cx| {
                                view.update(cx, |this, cx| {
                                    let lib_id = this.app_config.active_library_id.clone().unwrap_or_default();
                                    crate::controllers::signal_set_controller::apply_signal_set(
                                        this, &lib_id, &name, cx,
                                    );
                                    this.show_signal_set_dropdown = false;
                                    cx.notify();
                                });
                            }
                        })
                        .flex().items_center().justify_between()
                        .child(div().text_xs().text_color(rgb(0xd4d4d8)).child(name.clone()))
                        .child(div().text_xs().text_color(rgb(0x71717a)).child(format!("({})", count)))
                        .into_any_element()
                }
                SetDropdownItem::ClearActive => div()
                    .px_4().py_2()
                    .border_t_1().border_color(rgb(0x27272a))
                    .cursor_pointer().hover(|s| s.bg(rgb(0x1f1f22)))
                    .on_mouse_down(MouseButton::Left, {
                        let view = view.clone();
                        move |_, _, cx| {
                            view.update(cx, |this, cx| {
                                crate::controllers::signal_set_controller::clear_active_signal_set(this, cx);
                                this.show_signal_set_dropdown = false;
                                cx.notify();
                            });
                        }
                    })
                    .text_xs().text_color(rgb(0xef4444))
                    .child("✕ 清除当前选择")
                    .into_any_element(),
            };
            children.push(elem);
        }
        // Click-outside handler: an invisible full-screen overlay behind the popup that closes on click
        Some(
            div()
                .absolute()
                .top_0().left_0()
                .size_full()
                .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                    this.show_signal_set_dropdown = false;
                    cx.notify();
                }))
                // Position the popup just below the trigger (top: ~50px from sidebar top)
                .child(
                    div()
                        .absolute()
                        .top(px(50.0))
                        .left_0()
                        .right_0()
                        .bg(rgb(0x18181b))
                        .border_1()
                        .border_color(rgb(0x27272a))
                        .rounded_b(px(4.0))
                        .shadow_lg()
                        .flex().flex_col()
                        .children(children)
                )
        )
    } else {
        None
    };

    div()
        .relative()
        .child(trigger)
        .children(popup)
}
```

**Note on click-outside:** The overlay approach (a full-screen invisible `div` that closes on click) is the simplest GPUI pattern that doesn't require tracking mouse coordinates. The popup `div` is positioned absolutely above the overlay so clicks on popup items hit the popup first; clicks elsewhere hit the overlay and close.

- [ ] **Step 2: Modify the bottom action bar**

In `render_signal_sidebar` (lines 560-622), the current bottom bar has two children: `清除全部 (N)` and `绘制 N 个信号 (Plot)`. We add a third button `保存为信号集…` next to them, and replace the whole bar with an inline input when `show_save_set_input` is true.

Find the bottom bar block starting at line 560:

```rust
.child(
    // Bottom Action Bar: Clear all | Plot N signals
    if selected_count > 0 {
        div()
            .p_2()
            ...
            .child(/* Clear all button */)
            .child(/* Plot button */)
    } else {
        div()
    }
)
```

Replace the entire `if selected_count > 0 { ... } else { ... }` expression with:

```rust
.child(
    if app.show_save_set_input {
        // Inline rename input row
        render_save_set_input_row(app, view.clone(), cx)
    } else if selected_count > 0 {
        div()
            .p_2()
            .bg(rgb(0x131314))
            .border_t_1()
            .border_color(rgb(0x27272a))
            .flex()
            .gap_2()
            .child(
                // Clear all button (existing)
                div()
                    .px_3()
                    .py_1p5()
                    .bg(rgb(0x3f3f46))
                    .rounded(px(4.0))
                    .cursor_pointer()
                    .hover(|s| s.bg(rgb(0x52525b)))
                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                        this.clear_selected_signals(cx);
                    }))
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0xffffff))
                            .child(format!("清除全部 ({})", selected_count))
                    )
            )
            .child(
                // Plot button (existing)
                div()
                    .flex_1()
                    .px_3()
                    .py_1p5()
                    .bg(rgb(0x3b82f6))
                    .rounded(px(4.0))
                    .cursor_pointer()
                    .hover(|s| s.bg(rgb(0x2563eb)))
                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                        crate::ui::views::chart_view::extract_and_update_series_data(this);
                        cx.notify();
                    }))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .gap_2()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(0xffffff))
                                    .child(format!("绘制 {} 个信号 (Plot)", selected_count))
                            )
                    )
            )
            .child(
                // Save as signal set button (NEW)
                div()
                    .px_3()
                    .py_1p5()
                    .bg(rgb(0x3f3f46))
                    .rounded(px(4.0))
                    .cursor_pointer()
                    .hover(|s| s.bg(rgb(0x52525b)))
                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                        this.show_save_set_input = true;
                        this.pending_signal_set_name = Some(String::new());
                        cx.notify();
                    }))
                    .child(
                        div().text_xs().text_color(rgb(0xffffff)).child("保存为信号集…")
                    )
            )
    } else {
        div()
    }
)
```

Then add the helper function `render_save_set_input_row` near `render_signal_set_dropdown_row`:

```rust
/// Render the inline "save as signal set" input row (replaces the bottom bar
/// when show_save_set_input is true). Enter saves (via InputState subscribe);
/// 取消 button aborts. The Input entity is created lazily in impls_rendering.rs
/// render loop (Sub-step 0b) and bound here.
fn render_save_set_input_row(
    app: &CanViewerApp,
    view: Entity<CanViewerApp>,
    _cx: &mut Context<CanViewerApp>,
) -> impl IntoElement {
    let input_entity = app.signal_set_name_input.clone();
    let view_for_cancel = view.clone();

    div()
        .p_2()
        .bg(rgb(0x131314))
        .border_t_1()
        .border_color(rgb(0x27272a))
        .flex()
        .gap_2()
        .child(
            if let Some(input) = input_entity {
                div()
                    .flex_1()
                    .h(px(32.0))
                    .flex()
                    .items_center()
                    .child(gpui_component::input::Input::new(&input).appearance(true))
                    .into_any_element()
            } else {
                div()
                    .flex_1()
                    .h(px(32.0))
                    .flex()
                    .items_center()
                    .px_2()
                    .text_xs()
                    .text_color(rgb(0x52525b))
                    .child("初始化输入…")
                    .into_any_element()
            }
        )
        .child(
            div()
                .px_3()
                .py_1p5()
                .bg(rgb(0x3f3f46))
                .rounded(px(4.0))
                .cursor_pointer()
                .hover(|s| s.bg(rgb(0x52525b)))
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    view_for_cancel.update(cx, |this, cx| {
                        this.show_save_set_input = false;
                        this.pending_signal_set_name = None;
                        cx.notify();
                    });
                })
                .child(div().text_xs().text_color(rgb(0xffffff)).child("取消"))
        )
}
```

**Notes:**

- The `signal_set_name_input` `Entity<InputState>` is created in Sub-step 0b (lazily on first render). The Input reads its value from the entity itself, not from `pending_signal_set_name`; `pending_signal_set_name` is no longer the source of truth for the input text. Keep `pending_signal_set_name` as a separate field for backward compat / programmatic save, but the Input drives the actual text. The `PressEnter` subscribe reads `input.read(cx).value()` directly and passes to the controller.
- The `取消` button aborts the form by setting `show_save_set_input = false` and `pending_signal_set_name = None`. Optionally, the InputState's value can be cleared on cancel via `input.update(cx, |state, cx| state.set_value("", window, cx))`, but the controller reads the value fresh on the next Enter so a stale value is harmless.
- The Input component handles its own keyboard focus, IME, and text editing — no need for `on_key_down` hacks.

- [ ] **Step 3: Add active-set badge to sidebar header**

In `render_signal_sidebar`, find the header (lines 474-497). The current header has a title and a count. Insert an active-set badge between them (only when `active_signal_set.is_some()` and the lib matches `active_library_id`).

Current header structure:
```rust
.child(
    div()
        .text_xs()
        .font_weight(FontWeight::BOLD)
        .text_color(rgb(0xe4e4e7))
        .child("信号选择 (Signals)")
)
.child(
    div()
        .text_xs()
        .text_color(rgb(0x71717a))
        .child(format!("{}", item_count))
)
```

Replace with:

```rust
.child(
    div()
        .flex()
        .items_center()
        .gap_2()
        .child(
            div()
                .text_xs()
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(0xe4e4e7))
                .child("信号选择 (Signals)")
        )
        .when_some(
            app.active_signal_set.as_ref().and_then(|(lid, sname)| {
                if app.app_config.active_library_id.as_ref() == Some(lid) {
                    Some(sname.clone())
                } else {
                    None
                }
            }),
            |this, set_name| {
                this.child(
                    div()
                        .px_1p5()
                        .py(px(1.0))
                        .bg(rgb(0x3b82f6))
                        .rounded(px(3.0))
                        .text_xs()
                        .text_color(rgb(0xffffff))
                        .child(set_name)
                )
            }
        )
)
.child(
    div()
        .text_xs()
        .text_color(rgb(0x71717a))
        .child(format!("{}", item_count))
)
```

- [ ] **Step 4: Add checkbox-click hook to clear `active_signal_set`**

In `src/view/src/ui/views/plot_sidebar.rs`, find the `SignalItem` rendering (around lines 190-249). The current checkbox click handler (lines 221-233) is:

```rust
.on_mouse_down(MouseButton::Left, {
    let view = view.clone();
    move |_, _, cx| {
        view.update(cx, |this, cx| {
            if let Some(pos) = this.selected_signals.iter().position(|s| s == &sig_id) {
                this.selected_signals.remove(pos);
            } else {
                this.selected_signals.push(sig_id.clone());
            }
            cx.notify();
        });
    }
})
```

Replace with:

```rust
.on_mouse_down(MouseButton::Left, {
    let view = view.clone();
    move |_, _, cx| {
        view.update(cx, |this, cx| {
            if let Some(pos) = this.selected_signals.iter().position(|s| s == &sig_id) {
                this.selected_signals.remove(pos);
            } else {
                this.selected_signals.push(sig_id.clone());
            }
            // Manual edit: clear active set binding so the dropdown reverts to "Select a set…"
            this.active_signal_set = None;
            cx.notify();
        });
    }
})
```

- [ ] **Step 5: Build to verify compile**

Run: `cargo build -p viewer`
Expected: PASS. If the compiler complains about unused `view_for_save` (declared in the inline rename row), remove that variable.

- [ ] **Step 6: Run all existing tests to ensure no regression**

Run: `cargo test -p viewer`
Expected: PASS — all existing tests + the 5 new `build_set_dropdown` tests + the 13 `signal_sets` tests all green.

- [ ] **Step 7: Run clippy to catch lints**

Run: `cargo clippy -p viewer -- -D warnings 2>&1 | head -50`
Expected: PASS (or only minor lint warnings; fix the obvious ones).

- [ ] **Step 8: Commit**

```bash
git add src/view/src/ui/views/plot_sidebar.rs src/view/src/app/state.rs
git commit -m "feat(signal-sets): plot sidebar dropdown + save-as-set + inline rename + header badge"
```

---

## Task 7: Manual verification checklist

**Files:** No code changes. The engineer runs the app and confirms behavior.

- [ ] **Step 1: Build the release binary**

Run: `cargo build --release -p viewer`
Expected: PASS.

- [ ] **Step 2: Launch the app**

Run: `cargo run --release -p viewer` (or directly `./target/release/viewer`).
Expected: app window opens, no panics.

- [ ] **Step 3: Configure a library and load a BLF**

1. Open the Library tab. Create a library "EngineLib". Add a DBC version with one channel. Activate the version.
2. Switch to the Log tab. Open `sample.blf` (or `can_*.blf`).
3. Switch to the Signal Plot tab.
4. The sidebar dropdown at top should show "选择信号集…" (active library is "EngineLib"). Below: "信号集: 选择信号集…".

- [ ] **Step 4: Save a signal set**

1. In the sidebar, expand the channel, expand a message, check 3 signals.
2. Bottom bar: `[清除全部 (3)] [绘制 3 个信号 (Plot)] [保存为信号集…]`.
3. Click "保存为信号集…". Bottom bar becomes `[输入集名并按回车保存…] [取消]`.
4. Type "Engine signals". Press Enter.
5. Expected: status bar shows "已保存集 'Engine signals'（3 个信号）". Bottom bar reverts to the 3-button form.
6. Restart the app. Open the same Library and BLF. Go to Signal Plot. The dropdown should now list "Engine signals (3)".

- [ ] **Step 5: Apply a set**

1. Clear selection (`清除全部`). selected_signals is empty.
2. Click the dropdown. Click "Engine signals (3)".
3. Expected: status "已加载集 'Engine signals'（3 个信号）"; sidebar checkboxes show 3 selected; chart plots 3 lines.
4. Header badge shows "Engine signals" next to "信号选择 (Signals)".
5. Dropdown now shows "✕ 清除当前选择" at the bottom.

- [ ] **Step 6: Manual edit clears active set**

1. With "Engine signals" active, click an unchecked signal in the sidebar (a 4th one).
2. Expected: 4 signals selected; header badge disappears; dropdown trigger text reverts to "选择信号集…".
3. selected_signals has 4 entries (the original 3 + the new one).

- [ ] **Step 7: ✕ clears everything**

1. Re-select "Engine signals" from the dropdown (3 signals).
2. Click the dropdown again. Click "✕ 清除当前选择".
3. Expected: selected_signals empty; chart cleared; dropdown shows "选择信号集…".

- [ ] **Step 8: Library switch clears active set**

1. Apply "Engine signals" (3 signals).
2. Switch to Library tab. Activate a different library version.
3. Switch back to Signal Plot.
4. Expected: `active_signal_set` cleared; dropdown shows "选择信号集…" or "当前库无信号集" if the new active library has no sets.

- [ ] **Step 9: Library rename migrates sets**

1. Apply "Engine signals" in lib "EngineLib".
2. Switch to Library tab. Rename "EngineLib" to "EngineLib2".
3. Switch back to Signal Plot.
4. Expected: dropdown shows "Engine signals (3)" still — the set migrated with the rename. (If "Engine signals" was active before the rename, the active set should still show, now bound to "EngineLib2".)

- [ ] **Step 10: Library delete removes sets**

1. Switch to Library tab. Delete "EngineLib2".
2. Confirm deletion (if the library is in use, this should fail; otherwise it succeeds).
3. Switch back to Signal Plot. The dropdown for the new active library (if any) shows its sets, not "Engine signals".

- [ ] **Step 11: Edge cases**

1. Try saving with an empty name → status "集名不能为空", input row stays open.
2. Try saving with a duplicate name → status "集 'X' 已存在", input row stays open.
3. Deactivate the library (if possible via UI), then click "保存为信号集…" → status "先激活一个信号库". If there's no UI to deactivate a library, skip this step.

- [ ] **Step 12: Commit verification log (optional)**

If the engineer wants to record the manual run, write a short summary as a comment in the PR description. No file changes.

---

## Self-Review Checklist (engineer: run after Task 6, before merge)

- [ ] Spec section "数据模型" → Task 1 ✓
- [ ] Spec section "App state" → Task 2 ✓
- [ ] Spec section "控制器" → Task 3 ✓
- [ ] Spec section "与现有库变更钩子集成" → Task 4 ✓
- [ ] Spec section "UI - 5.1 dropdown row" → Task 6 Step 1 ✓
- [ ] Spec section "UI - 5.2 save-as-set button + inline rename" → Task 6 Step 2 ✓
- [ ] Spec section "UI - 5.3 header badge" → Task 6 Step 3 ✓
- [ ] Spec section "UI - 5.4 manual edit clears active set" → Task 6 Step 4 ✓
- [ ] Spec section "UI - 5.5 build_set_dropdown_items pure function" → Task 5 ✓
- [ ] Spec section "单元测试" → Task 1 (signal_sets) + Task 5 (build_set_dropdown_items) ✓
- [ ] Spec section "手动验证清单" → Task 7 ✓
- [ ] Spec section "错误处理" → Task 3 (controller validations) ✓
- [ ] Spec section "边缘情况" → Task 4 (rename/delete hooks) + Task 6 Step 4 (manual edit) ✓
- [ ] No `TODO` / `TBD` / `...` placeholders in any task ✓
- [ ] All function/type names consistent across tasks: `SignalSetEntry`, `SignalSet`, `SignalSetStore`, `parse_signal_id`, `build_selected_signals_from_set`, `signal_set_store_path`, `load_signal_set_store`, `save_signal_set_store`, `save_current_selection_as_signal_set`, `apply_signal_set`, `clear_active_signal_set`, `delete_signal_set`, `SetDropdownItem`, `build_set_dropdown_items`, `active_signal_set`, `signal_set_store`, `pending_signal_set_name`, `show_save_set_input`, `show_signal_set_dropdown`, `signal_set_name_input` ✓
