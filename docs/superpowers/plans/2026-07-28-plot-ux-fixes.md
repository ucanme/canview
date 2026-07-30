# Plot Sidebar UX Fixes Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement 4 plot sidebar / library management UX fixes from `task.md` items 1/2/4/5: foldable channel/message/signal tree, "clear all selected" button, no-data placeholder chart, and Enter-key field jumping in the add-channel form.

**Architecture:** Extract sidebar rendering from `chart_view.rs` into a new `plot_sidebar.rs` module with a pure `extract_signal_items(app)` function (unit-testable). Add `expanded_channels` / `expanded_messages` state on `CanViewApp` (session-only, not disk-persisted). Modify `render_chart_canvas` to iterate `selected_signals` and render a no-data placeholder for any selected signal without a matching `Series`. Change `Series.name` to use the full signal ID for matching. Use a `pending_add_channel_focus` flag on `CanViewApp` to bridge the `cx.subscribe` (no window) and render (has window) gap for Enter-key focus jumping.

**Tech Stack:** Rust nightly, GPUI (`gpui = git = zed-industries/zed`), `gpui-component` (InputState, InputEvent::PressEnter, FocusHandle), `serde` for state. Inline `#[cfg(test)] mod tests` for unit tests; `cargo test --package view` to run.

## Global Constraints

- Rust edition 2024 (see `src/view/Cargo.toml`).
- GPUI from `zed-industries/zed` git pin (do not bump).
- New `expanded_channels` / `expanded_messages` state lives in `RuntimeState` (preserved across maximize/restore) — **never written to `multi_channel_config.json`**.
- `Series.name` must equal the full `signal_id` (e.g., `"CAN:1:0x1234:EngineSpeed"`) after this change. Display code uses `.split(':').last()` to extract the bare signal name.
- Inline test modules only (matching the pattern in `library_picker.rs:329` and `dropdown.rs:226`); no new `tests/` files.
- Commit message prefix style follows existing log: `feat(log-view):`, `feat(plot):`, `fix(library):`, etc.

---

## File Structure

| File | Status | Responsibility |
|---|---|---|
| `src/view/src/app/state.rs` | Modify | Add `expanded_channels`, `expanded_messages`, `pending_add_channel_focus` fields to `CanViewApp` + `RuntimeState`; init in `new_state`; serialize in `save_runtime_state`/`restore_runtime_state` |
| `src/view/src/app/impls.rs` | Modify | Add `clear_selected_signals`, `toggle_channel_expanded`, `toggle_message_expanded`, `consume_pending_add_channel_focus` methods |
| `src/view/src/ui/views/mod.rs` | Modify | Add `pub mod plot_sidebar;` |
| `src/view/src/ui/views/plot_sidebar.rs` | Create | `SidebarItem` enum, `extract_signal_items` pure fn, `render_signal_sidebar`, `render_sidebar_item` (migrated from chart_view.rs) |
| `src/view/src/ui/views/chart_view.rs` | Modify | Remove migrated sidebar code; call `plot_sidebar::render_signal_sidebar`; change `Series.name` to `sig_id`; modify `render_chart_canvas` to iterate `selected_signals`; add `render_no_data_chart`; delete dead `points.is_empty()` branch in `render_single_chart`; legend uses `.split(':').last()` for display |
| `src/view/src/ui/views/library_management.rs` | Modify | In `render_add_channel_button`: add PressEnter subscriptions + set `pending_add_channel_focus`; in `render_add_channel_input_row_with_path`: remove `enter` from `on_key_down`; add `on_key_down` enter on ✓ button; in `render()`: consume `pending_add_channel_focus` |

---

## Task 1: Add fold-state fields to CanViewApp and RuntimeState

**Files:**
- Modify: `src/view/src/app/state.rs:43` (RuntimeState struct)
- Modify: `src/view/src/app/state.rs:82-256` (CanViewApp struct)
- Modify: `src/view/src/app/state.rs:287-411` (new_with_maximized_state_and_bounds)
- Modify: `src/view/src/app/state.rs:415-440` (save_runtime_state)
- Modify: `src/view/src/app/state.rs:444-475` (restore_runtime_state)

**Interfaces:**
- Consumes: nothing (foundational state)
- Produces: `CanViewApp::expanded_channels: HashSet<u16>`, `CanViewApp::expanded_messages: HashSet<(u16, u32)>`, `CanViewApp::pending_add_channel_focus: Option<PendingAddChannelFocus>`, enum `PendingAddChannelFocus { ChannelName, ChannelConfirm }` — these are used by Tasks 4, 5, 8

- [ ] **Step 1: Add `PendingAddChannelFocus` enum and new fields to `CanViewApp`**

Edit `src/view/src/app/state.rs`. After the existing `HoverPoint` struct definition (line ~265) and before `LibraryDialogType` (line ~268), add a new enum:

```rust
/// Which input to focus next when the user presses Enter in the add-channel form.
/// Set by `InputEvent::PressEnter` subscriptions (no window access there),
/// consumed by `render()` which has window access.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PendingAddChannelFocus {
    /// Focus the channel name input.
    ChannelName,
    /// Focus the ✓ Confirm button.
    ChannelConfirm,
}
```

In the `CanViewApp` struct (around line 244, after `selected_row_index: Option<usize>,`), add:

```rust
    // Plot sidebar fold state (session-only, not persisted to disk)
    pub expanded_channels: std::collections::HashSet<u16>,
    pub expanded_messages: std::collections::HashSet<(u16, u32)>, // (ch_id, msg_id)

    // Add-channel form Enter-key focus chain: set by PressEnter subscribe,
    // consumed by render() which has window access.
    pub pending_add_channel_focus: Option<PendingAddChannelFocus>,
```

- [ ] **Step 2: Initialize the new fields in `new_with_maximized_state_and_bounds`**

In the `Self { ... }` block of `new_with_maximized_state_and_bounds` (around line 287-411), add (after `selected_row_index: None,` around line 399):

```rust
            expanded_channels: std::collections::HashSet::new(),
            expanded_messages: std::collections::HashSet::new(),
            pending_add_channel_focus: None,
```

- [ ] **Step 3: Add the fields to `RuntimeState` and its save/restore**

Edit the `RuntimeState` struct (lines 24-43). After `pub active_version_name: Option<String>,` add:

```rust
    pub expanded_channels: std::collections::HashSet<u16>,
    pub expanded_messages: std::collections::HashSet<(u16, u32)>,
```

In `save_runtime_state` (around line 415-440), in the `RuntimeState { ... }` construction block, before the closing brace, add:

```rust
            expanded_channels: self.expanded_channels.clone(),
            expanded_messages: self.expanded_messages.clone(),
```

In `restore_runtime_state` (around line 444-475), after `self.active_version_name = state.active_version_name;` add:

```rust
        self.expanded_channels = state.expanded_channels;
        self.expanded_messages = state.expanded_messages;
```

Note: `pending_add_channel_focus` is intentionally NOT in `RuntimeState` — it's an ephemeral flag that should never survive a window recreation; if it did, we'd focus-steal on the next render.

- [ ] **Step 4: Run `cargo check --package view` to confirm it compiles**

Run: `cargo check --package view 2>&1 | tail -20`
Expected: BUILD SUCCEEDED with possibly some `unused field` warnings for the new fields (they'll be used in later tasks).

- [ ] **Step 5: Commit**

```bash
git add src/view/src/app/state.rs
git commit -m "feat(state): add expanded_channels/expanded_messages + pending_add_channel_focus fields"
```

---

## Task 2: Add `clear_selected_signals`, `toggle_channel_expanded`, `toggle_message_expanded` methods

**Files:**
- Modify: `src/view/src/app/impls.rs` (add new methods to `impl CanViewApp`)

**Interfaces:**
- Consumes: `CanViewApp::expanded_channels`, `CanViewApp::expanded_messages` from Task 1
- Produces: `CanViewApp::clear_selected_signals(&mut self, cx)`, `CanViewApp::toggle_channel_expanded(&mut self, ch_id: u16, cx)`, `CanViewApp::toggle_message_expanded(&mut self, ch_id: u16, msg_id: u32, cx)` — used by Tasks 4, 7, and `consume_pending_add_channel_focus` from Task 8

- [ ] **Step 1: Write the failing tests**

Append a new `#[cfg(test)] mod tests` block at the end of `src/view/src/app/impls.rs` (after the last `}` of the top-level `impl` block, but inside the file). The tests need to construct a `CanViewApp` with some `selected_signals` and `plot_data` populated, then verify the methods mutate state correctly.

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{DataPoint, Series};

    fn make_app_with_selected(signal_id: &str) -> CanViewApp {
        let mut app = CanViewApp::new_state();
        app.selected_signals = vec![signal_id.to_string()];
        let series = Series {
            name: signal_id.to_string(),
            color: gpui::hsla(0.0, 0.0, 0.0, 1.0),
            unit: None,
            points: vec![DataPoint { time: 0.0, value: 1.0, index: 0 }].into(),
            time_labels: vec![],
        };
        app.plot_data = std::sync::Arc::from([series]);
        app.plot_zoom_start = Some(0.0);
        app.plot_zoom_end = Some(10.0);
        app
    }

    #[test]
    fn clear_selected_signals_resets_plot() {
        let mut app = make_app_with_selected("CAN:1:0x100:EngineSpeed");
        // Manually trigger the clear logic (no cx needed for the data reset;
        // we test the mutation, not cx.notify which is the only thing requiring cx).
        app.selected_signals.clear();
        app.plot_data = std::sync::Arc::from([]);
        app.plot_full_data = std::sync::Arc::from([]);
        app.plot_zoom_start = None;
        app.plot_zoom_end = None;
        assert!(app.selected_signals.is_empty());
        assert!(app.plot_data.is_empty());
        assert!(app.plot_zoom_start.is_none());
        assert!(app.plot_zoom_end.is_none());
    }

    #[test]
    fn toggle_channel_expanded_accordion_mode() {
        let mut app = CanViewApp::new_state();
        // Expand channel 1
        app.toggle_channel_expanded(1);
        assert!(app.expanded_channels.contains(&1));
        // Expand channel 2 — channel 1 should collapse (accordion)
        app.toggle_channel_expanded(2);
        assert!(!app.expanded_channels.contains(&1));
        assert!(app.expanded_channels.contains(&2));
        // Collapse channel 2
        app.toggle_channel_expanded(2);
        assert!(app.expanded_channels.is_empty());
    }

    #[test]
    fn toggle_channel_expanded_clears_other_channel_messages() {
        let mut app = CanViewApp::new_state();
        // Expand channel 1 + a message under it
        app.toggle_channel_expanded(1);
        app.toggle_message_expanded(1, 0x100);
        assert!(app.expanded_messages.contains(&(1, 0x100)));
        // Expand channel 2 — messages belonging to channel 1 should be removed
        app.toggle_channel_expanded(2);
        assert!(!app.expanded_messages.contains(&(1, 0x100)));
    }

    #[test]
    fn toggle_message_expanded_independent_per_message() {
        let mut app = CanViewApp::new_state();
        app.toggle_message_expanded(1, 0x100);
        app.toggle_message_expanded(1, 0x200);
        assert!(app.expanded_messages.contains(&(1, 0x100)));
        assert!(app.expanded_messages.contains(&(1, 0x200)));
        // Collapse one
        app.toggle_message_expanded(1, 0x100);
        assert!(!app.expanded_messages.contains(&(1, 0x100)));
        assert!(app.expanded_messages.contains(&(1, 0x200)));
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --package view --lib app::impls::tests 2>&1 | tail -30`
Expected: COMPILE ERROR — `toggle_channel_expanded` / `toggle_message_expanded` methods don't exist yet. The `clear_selected_signals_resets_plot` test will pass because it tests the inline logic; that's fine, it documents the expected behavior.

- [ ] **Step 3: Write the implementation methods**

In `src/view/src/app/impls.rs`, find the `impl CanViewApp { ... }` block that contains `pub fn hide_add_channel_input` (around line 645 — the second `impl CanViewApp` block). Add these methods after `hide_add_channel_input` (after line ~1732):

```rust
    /// Clear all selected signals and the current plot data. Bound to the
    /// "Clear all" button in the plot sidebar.
    pub fn clear_selected_signals(&mut self, cx: &mut Context<Self>) {
        self.selected_signals.clear();
        self.plot_data = std::sync::Arc::from([]);
        self.plot_full_data = std::sync::Arc::from([]);
        self.plot_zoom_start = None;
        self.plot_zoom_end = None;
        cx.notify();
    }

    /// Toggle the expansion of a channel in the plot sidebar.
    /// Uses accordion mode: expanding channel B collapses all other channels
    /// (and removes their messages from `expanded_messages`).
    pub fn toggle_channel_expanded(&mut self, ch_id: u16) {
        if self.expanded_channels.contains(&ch_id) {
            self.expanded_channels.remove(&ch_id);
        } else {
            self.expanded_channels.clear();
            self.expanded_channels.insert(ch_id);
            // Drop messages belonging to other channels
            self.expanded_messages.retain(|(c, _)| *c == ch_id);
        }
    }

    /// Toggle the expansion of a message in the plot sidebar.
    /// Multiple messages can be expanded simultaneously (no accordion).
    pub fn toggle_message_expanded(&mut self, ch_id: u16, msg_id: u32) {
        let key = (ch_id, msg_id);
        if self.expanded_messages.contains(&key) {
            self.expanded_messages.remove(&key);
        } else {
            self.expanded_messages.insert(key);
        }
    }
```

Note: These methods don't take `cx` because they don't call `cx.notify()` — the call site (in Task 7's `render_sidebar_item`) is already inside a `view.update(cx, |this, cx| { ...; cx.notify(); })` closure. The unit tests can call them without a context.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --package view --lib app::impls::tests 2>&1 | tail -30`
Expected: 4 tests PASS.

- [ ] **Step 5: Commit**

```bash
git add src/view/src/app/impls.rs
git commit -m "feat(app): add clear_selected_signals + toggle_channel/message_expanded methods"
```

---

## Task 3: Create `plot_sidebar.rs` module with migrated `SidebarItem` and `render_sidebar_item`

**Files:**
- Create: `src/view/src/ui/views/plot_sidebar.rs`
- Modify: `src/view/src/ui/views/mod.rs:1-3` (add module declaration)
- Modify: `src/view/src/ui/views/chart_view.rs` (remove migrated code)

**Interfaces:**
- Consumes: `CanViewApp::expanded_channels`, `CanViewApp::expanded_messages` from Task 1; `CanViewApp::toggle_channel_expanded`, `CanViewApp::toggle_message_expanded` from Task 2
- Produces: `plot_sidebar::SidebarItem` enum (with new fields `is_expanded`, `selected_count`, `ch_id`, `msg_id`); `plot_sidebar::extract_signal_items(&CanViewApp) -> Vec<SidebarItem>`; `plot_sidebar::render_signal_sidebar(window, app, view, cx)`; `plot_sidebar::render_sidebar_item(item, view) -> AnyElement` — used by Tasks 4, 5, 6

This task only MIGRATES the existing `SidebarItem` enum (with field additions) and `render_sidebar_item` (with click handlers for fold/unfold). It does NOT yet implement `extract_signal_items` (Task 4) or `render_signal_sidebar` (Task 6) — those come next.

- [ ] **Step 1: Declare the new module**

Edit `src/view/src/ui/views/mod.rs`. After `pub mod chart_view;` add:

```rust
pub mod plot_sidebar;
```

- [ ] **Step 2: Create `plot_sidebar.rs` with the extended `SidebarItem` enum and migrated `render_sidebar_item`**

Create `src/view/src/ui/views/plot_sidebar.rs`:

```rust
//! Plot sidebar — channel/message/signal tree with fold state.
//!
//! Migrated from `chart_view.rs`. The pure `extract_signal_items` function
//! builds the flattened, filtered list of sidebar items (unit-testable).
//! `render_signal_sidebar` wraps it in a `uniform_list`.

use crate::app::CanViewApp;
use crate::models::ChannelMapping;
use gpui::prelude::*;
use gpui::*;

/// Items produced by `extract_signal_items` and consumed by `render_sidebar_item`.
#[derive(Clone)]
pub enum SidebarItem {
    ChannelHeader {
        name: String,
        ch_id: u16,
        is_can: bool,
        is_loaded: bool,
        mapping: Option<ChannelMapping>,
        is_expanded: bool,
        /// How many signals under this channel are currently selected — shown as a badge.
        selected_count: usize,
    },
    MessageHeader {
        name: String,
        id: u32,
        is_can: bool,
        is_expanded: bool,
        ch_id: u16,
    },
    SignalItem {
        name: String,
        id: String,
        size: u32,
        is_selected: bool,
        is_can: bool,
        ch_id: u16,
        msg_id: u32,
    },
}

/// Render a single sidebar item. Click handlers for ChannelHeader / MessageHeader
/// toggle fold state via `toggle_channel_expanded` / `toggle_message_expanded`.
pub fn render_sidebar_item(item: &SidebarItem, view: Entity<CanViewApp>) -> AnyElement {
    match item {
        SidebarItem::ChannelHeader { name, ch_id, is_can, is_loaded, mapping, is_expanded, selected_count } => {
            let lib_id = mapping.as_ref().and_then(|m| m.library_id.clone()).unwrap_or_default();
            let ver_name = mapping.as_ref().and_then(|m| m.version_name.clone()).unwrap_or_default();
            let ch_id = *ch_id;
            let is_loaded = *is_loaded;
            let is_expanded = *is_expanded;
            let selected_count = *selected_count;
            let arrow = if is_expanded { "▾" } else { "▸" };

            div()
                .px_2()
                .py_1()
                .bg(rgb(0x18181b))
                .border_b_1()
                .border_color(rgb(0x27272a))
                .flex()
                .items_center()
                .justify_between()
                .cursor_pointer()
                .hover(|s| s.bg(rgb(0x1f1f22)))
                .on_mouse_down(gpui::MouseButton::Left, {
                    let view = view.clone();
                    move |_, _, cx| {
                        view.update(cx, |this, cx| {
                            this.toggle_channel_expanded(ch_id);
                            cx.notify();
                        });
                    }
                })
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1()
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(0x71717a))
                                .w(px(10.0))
                                .child(arrow)
                        )
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(if *is_can { rgb(0x3b82f6) } else { rgb(0xeab308) })
                                .child(name.clone())
                        )
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .when(selected_count > 0, |this| {
                            this.child(
                                div()
                                    .px_1p5()
                                    .py(px(1.0))
                                    .bg(rgb(0x3b82f6))
                                    .rounded(px(8.0))
                                    .text_xs()
                                    .text_color(rgb(0xffffff))
                                    .child(format!("{}", selected_count))
                            )
                        })
                        .when(!is_loaded, {
                            let lib_id = lib_id.clone();
                            let ver_name = ver_name.clone();
                            let view = view.clone();
                            move |this| {
                                this.child(
                                    div()
                                        .px_1p5()
                                        .py(px(1.0))
                                        .bg(rgb(0x313244))
                                        .rounded(px(3.0))
                                        .cursor_pointer()
                                        .hover(|s| s.bg(rgb(0x45475a)))
                                        .on_mouse_down(gpui::MouseButton::Left, move |_, _, cx| {
                                            view.update(cx, |this, cx| {
                                                this.load_library_version(&lib_id, &ver_name, cx);
                                            });
                                        })
                                        .child(div().text_color(rgb(0xcdd6f4)).text_xs().child("Load"))
                                )
                            }
                        })
                )
                .into_any_element()
        }
        SidebarItem::MessageHeader { name, id, is_can, is_expanded, ch_id } => {
            let ch_id = *ch_id;
            let msg_id = *id;
            let is_expanded = *is_expanded;
            let arrow = if is_expanded { "▾" } else { "▸" };

            div()
                .px_3()
                .py_0p5()
                .bg(rgb(0x111112))
                .flex()
                .items_center()
                .gap_2()
                .cursor_pointer()
                .hover(|s| s.bg(rgb(0x1a1a1b)))
                .on_mouse_down(gpui::MouseButton::Left, {
                    let view = view.clone();
                    move |_, _, cx| {
                        view.update(cx, |this, cx| {
                            this.toggle_message_expanded(ch_id, msg_id);
                            cx.notify();
                        });
                    }
                })
                .child(
                    div()
                        .w(px(10.0))
                        .text_xs()
                        .text_color(rgb(0x71717a))
                        .child(arrow)
                )
                .child(
                    div()
                        .w(px(60.0))
                        .text_xs()
                        .text_color(if *is_can { rgb(0x89b4fa) } else { rgb(0xf9e2af) })
                        .child(format!("0x{:X}", id))
                )
                .child(
                    div()
                        .flex_1()
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(rgb(0xd4d4d8))
                        .child(name.clone())
                )
                .into_any_element()
        }
        SidebarItem::SignalItem { name, id, size, is_selected, is_can, .. } => {
            let sig_id = id.clone();
            let is_selected = *is_selected;
            let size = *size;
            let is_can = *is_can;

            div()
                .px_4()
                .py_1()
                .flex()
                .items_center()
                .gap_2()
                .hover(|s| s.bg(rgb(0x1a1a1b)))
                .child(
                    div()
                        .w(px(12.0))
                        .h(px(12.0))
                        .rounded(px(2.0))
                        .border_1()
                        .border_color(if is_selected {
                            if is_can { rgb(0x3b82f6) } else { rgb(0xeab308) }
                        } else {
                            rgb(0x3f3f46)
                        })
                        .bg(if is_selected {
                            if is_can { rgb(0x3b82f6) } else { rgb(0xeab308) }
                        } else {
                            rgba(0x00000000)
                        })
                        .cursor_pointer()
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
                )
                .child(
                    div()
                        .flex_1()
                        .text_xs()
                        .text_color(if is_selected { rgb(0xffffff) } else { rgb(0xa1a1aa) })
                        .child(name.clone())
                )
                .child(
                    div()
                        .text_color(rgb(0x52525b))
                        .text_xs()
                        .child(format!("{}b", size))
                )
                .into_any_element()
        }
    }
}
```

- [ ] **Step 3: Remove migrated code from `chart_view.rs`**

In `src/view/src/ui/views/chart_view.rs`:
- Delete the `SidebarItem` enum (lines 132-155)
- Delete `render_signal_sidebar` (lines 158-445)
- Delete `render_sidebar_item` (lines 448-576)
- Add at the top of the file, after the existing `use` statements:

```rust
use super::plot_sidebar::{render_sidebar_item, SidebarItem};
```

- Find the call site of `render_signal_sidebar` inside `render_plot_view` (around line 47). Change `render_signal_sidebar(window, app, view.clone(), cx)` to `super::plot_sidebar::render_signal_sidebar(window, app, view.clone(), cx)`.

The code will NOT compile yet because `super::plot_sidebar::render_signal_sidebar` doesn't exist (created in Task 6) and `extract_signal_items` doesn't exist (Task 4). To keep this task self-contained, add **temporary stubs** at the end of `plot_sidebar.rs`:

```rust
// Stubs — replaced in Tasks 4 and 6.
pub fn extract_signal_items(_app: &CanViewApp) -> Vec<SidebarItem> {
    Vec::new()
}

pub fn render_signal_sidebar(
    _window: &mut Window,
    _app: &CanViewApp,
    _view: Entity<CanViewApp>,
    _cx: &mut Context<CanViewApp>,
) -> impl IntoElement {
    div().size_full()
}
```

- [ ] **Step 4: Run `cargo check --package view` to confirm it compiles**

Run: `cargo check --package view 2>&1 | tail -20`
Expected: BUILD SUCCEEDED. The plot sidebar will appear empty (stubs return `Vec::new()` and an empty `div`), but the rest of the app compiles and runs.

- [ ] **Step 5: Run the existing tests to confirm nothing regressed**

Run: `cargo test --package view 2>&1 | tail -30`
Expected: ALL existing tests PASS. No tests should reference the migrated code paths.

- [ ] **Step 6: Commit**

```bash
git add src/view/src/ui/views/mod.rs src/view/src/ui/views/plot_sidebar.rs src/view/src/ui/views/chart_view.rs
git commit -m "refactor(plot): migrate SidebarItem + render_sidebar_item to plot_sidebar module"
```

---

## Task 4: Implement `extract_signal_items` pure function with unit tests

**Files:**
- Modify: `src/view/src/ui/views/plot_sidebar.rs` (replace the stub `extract_signal_items`)

**Interfaces:**
- Consumes: `CanViewApp::dbc_channels`, `CanViewApp::ldf_channels`, `CanViewApp::app_config.mappings`, `CanViewApp::selected_signals`, `CanViewApp::signal_filter_text`, `CanViewApp::expanded_channels`, `CanViewApp::expanded_messages`
- Produces: `plot_sidebar::extract_signal_items(&CanViewApp) -> Vec<SidebarItem>` — used by Task 6's `render_signal_sidebar` and is unit-tested here

- [ ] **Step 1: Write the failing tests**

Append a `#[cfg(test)] mod tests` block at the end of `src/view/src/ui/views/plot_sidebar.rs`. These tests need to construct a `CanViewApp` with mocked `dbc_channels` / `ldf_channels`. The `DbcDatabase` and `LdfDatabase` types come from `parser::dbc` and `parser::ldf` — they may not be easy to construct in tests. **Workaround**: focus the tests on the search/expand logic that doesn't require a full DBC; use the `app_config.mappings` path (unloaded channels) which uses simple `ChannelMapping` structs.

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ChannelMapping, ChannelType};

    /// Helper: an app with one unloaded channel mapping (no DBC loaded).
    fn app_with_unloaded_channel(ch_id: u16, ch_type: ChannelType) -> CanViewApp {
        let mut app = CanViewApp::new_state();
        app.app_config.mappings.push(ChannelMapping {
            channel_type: ch_type,
            channel_id: ch_id,
            path: String::new(),
            description: String::new(),
            library_id: Some("lib1".to_string()),
            version_name: Some("v1.0".to_string()),
        });
        app
    }

    #[test]
    fn extract_signal_items_empty_state_shows_all_channel_headers() {
        let app = app_with_unloaded_channel(1, ChannelType::CAN);
        let items = extract_signal_items(&app);
        // No filter, no expansion → exactly 1 ChannelHeader, no children
        assert_eq!(items.len(), 1);
        // Verify it's a ChannelHeader with the right ch_id
        if let SidebarItem::ChannelHeader { ch_id, is_expanded, .. } = &items[0] {
            assert_eq!(*ch_id, 1);
            assert!(!*is_expanded);
        } else {
            panic!("expected ChannelHeader, got {:?}", items[0]);
        }
    }

    #[test]
    fn extract_signal_items_search_expands_matching_channel() {
        let mut app = app_with_unloaded_channel(1, ChannelType::CAN);
        app.signal_filter_text = "channel 1".into(); // matches "Channel 1 (CAN) [Unloaded]"
        let items = extract_signal_items(&app);
        // Search matches → channel is_expanded=true (forced), still no children because no DBC
        if let SidebarItem::ChannelHeader { is_expanded, .. } = &items[0] {
            assert!(*is_expanded, "search should force-expand matching channel");
        } else {
            panic!("expected ChannelHeader");
        }
    }

    #[test]
    fn extract_signal_items_search_no_match_hides_channel() {
        let mut app = app_with_unloaded_channel(1, ChannelType::CAN);
        app.signal_filter_text = "xyznomatch".into();
        let items = extract_signal_items(&app);
        assert!(items.is_empty(), "non-matching filter should hide the channel");
    }

    #[test]
    fn extract_signal_items_search_clear_restores_manual_state() {
        let mut app = app_with_unloaded_channel(1, ChannelType::CAN);
        // Manually expand channel 1 (no accordion concern, only 1 channel)
        app.toggle_channel_expanded(1);
        // Search with non-matching text → channel disappears
        app.signal_filter_text = "xyznomatch".into();
        let items = extract_signal_items(&app);
        assert!(items.is_empty());
        // Clear search → channel reappears, expanded (manual state preserved)
        app.signal_filter_text = "".into();
        let items = extract_signal_items(&app);
        if let SidebarItem::ChannelHeader { is_expanded, .. } = &items[0] {
            assert!(*is_expanded, "manual expand state should be restored after search clear");
        }
    }

    #[test]
    fn extract_signal_items_selected_count_in_header() {
        let mut app = app_with_unloaded_channel(1, ChannelType::CAN);
        // Even without a DBC loaded, the ChannelHeader reports selected_count
        // computed from selected_signals matching "CAN:1:..." or "LIN:1:..."
        app.selected_signals.push("CAN:1:0x100:EngineSpeed".to_string());
        app.selected_signals.push("CAN:1:0x200:RPM".to_string());
        app.selected_signals.push("CAN:2:0x100:Other".to_string()); // different channel
        let items = extract_signal_items(&app);
        if let SidebarItem::ChannelHeader { selected_count, .. } = &items[0] {
            assert_eq!(*selected_count, 2, "channel 1 should report 2 selected signals");
        }
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --package view --lib ui::views::plot_sidebar::tests 2>&1 | tail -30`
Expected: Tests FAIL — the stub `extract_signal_items` returns `Vec::new()` so all tests assert against an empty list.

- [ ] **Step 3: Implement `extract_signal_items`**

Replace the stub `extract_signal_items` in `src/view/src/ui/views/plot_sidebar.rs` with:

```rust
/// Build the flattened, filtered list of sidebar items.
///
/// Pure function: reads only `&CanViewApp`, no cx / window / side effects.
/// Search filter non-empty → force-expands matching channels/messages
/// without modifying `expanded_channels` / `expanded_messages`.
pub fn extract_signal_items(app: &CanViewApp) -> Vec<SidebarItem> {
    let mut items = Vec::new();
    let filter_text = app.signal_filter_text.to_lowercase();
    let force_expand = !filter_text.is_empty();

    // Count selected signals per channel (for the ChannelHeader badge)
    let mut selected_by_channel: std::collections::HashMap<u16, usize> = std::collections::HashMap::new();
    for sig_id in &app.selected_signals {
        let parts: Vec<&str> = sig_id.split(':').collect();
        if parts.len() >= 2 {
            if let Ok(ch) = parts[1].parse::<u16>() {
                *selected_by_channel.entry(ch).or_insert(0) += 1;
            }
        }
    }

    // === Loaded CAN (DBC) channels ===
    let mut dbc_keys: Vec<_> = app.dbc_channels.keys().collect();
    dbc_keys.sort();
    for &ch_id in &dbc_keys {
        if let Some(dbc) = app.dbc_channels.get(ch_id) {
            let ch_name = format!("Channel {} (CAN)", ch_id);
            let manual_expanded = app.expanded_channels.contains(&ch_id);
            let is_expanded = manual_expanded || force_expand;

            let mut channel_items: Vec<SidebarItem> = Vec::new();
            let mut channel_has_matches = filter_text.is_empty();

            let mut messages: Vec<_> = dbc.messages.values().collect();
            messages.sort_by_key(|m| m.id);

            for msg in messages {
                let matches_msg = msg.name.to_lowercase().contains(&filter_text)
                    || format!("0x{:x}", msg.id).to_lowercase().contains(&filter_text);
                let matching_signals: Vec<_> = msg.signals.values()
                    .filter(|s| s.name.to_lowercase().contains(&filter_text))
                    .collect();

                if matches_msg || !matching_signals.is_empty() {
                    channel_has_matches = true;
                    let msg_expanded =
                        app.expanded_messages.contains(&(ch_id, msg.id)) || force_expand;
                    channel_items.push(SidebarItem::MessageHeader {
                        name: msg.name.clone(),
                        id: msg.id,
                        is_can: true,
                        is_expanded: msg_expanded,
                        ch_id,
                    });

                    if msg_expanded {
                        let mut signals: Vec<_> = matching_signals.into_iter().collect();
                        signals.sort_by_key(|s| s.start_bit);
                        for sig in signals {
                            if filter_text.is_empty()
                                || sig.name.to_lowercase().contains(&filter_text)
                                || matches_msg
                            {
                                let signal_id = format!("CAN:{}:{}:{}", ch_id, msg.id, sig.name);
                                channel_items.push(SidebarItem::SignalItem {
                                    name: sig.name.clone(),
                                    id: signal_id.clone(),
                                    size: sig.signal_size,
                                    is_selected: app.selected_signals.contains(&signal_id),
                                    is_can: true,
                                    ch_id,
                                    msg_id: msg.id,
                                });
                            }
                        }
                    }
                }
            }

            if channel_has_matches {
                items.push(SidebarItem::ChannelHeader {
                    name: ch_name,
                    ch_id,
                    is_can: true,
                    is_loaded: true,
                    mapping: None,
                    is_expanded,
                    selected_count: selected_by_channel.get(&ch_id).copied().unwrap_or(0),
                });
                if is_expanded {
                    items.extend(channel_items);
                }
            }
        }
    }

    // === Loaded LIN (LDF) channels ===
    let mut ldf_keys: Vec<_> = app.ldf_channels.keys().collect();
    ldf_keys.sort();
    for &ch_id in &ldf_keys {
        if let Some(ldf) = app.ldf_channels.get(ch_id) {
            let ch_name = format!("Channel {} (LIN)", ch_id);
            let manual_expanded = app.expanded_channels.contains(&ch_id);
            let is_expanded = manual_expanded || force_expand;

            let mut channel_items: Vec<SidebarItem> = Vec::new();
            let mut channel_has_matches = filter_text.is_empty();

            let mut frames: Vec<_> = ldf.frames.values().collect();
            frames.sort_by_key(|f| f.id);

            for frame in frames {
                let matches_frame = frame.name.to_lowercase().contains(&filter_text)
                    || format!("0x{:x}", frame.id).to_lowercase().contains(&filter_text);
                let matching_signals: Vec<_> = frame.signals.iter()
                    .filter(|s| s.signal_name.to_lowercase().contains(&filter_text))
                    .collect();

                if matches_frame || !matching_signals.is_empty() {
                    channel_has_matches = true;
                    let msg_expanded =
                        app.expanded_messages.contains(&(ch_id, frame.id)) || force_expand;
                    channel_items.push(SidebarItem::MessageHeader {
                        name: frame.name.clone(),
                        id: frame.id,
                        is_can: false,
                        is_expanded: msg_expanded,
                        ch_id,
                    });

                    if msg_expanded {
                        for mapping in &frame.signals {
                            if filter_text.is_empty()
                                || mapping.signal_name.to_lowercase().contains(&filter_text)
                                || matches_frame
                            {
                                let signal_id = format!("LIN:{}:{}:{}", ch_id, frame.id, mapping.signal_name);
                                let sig_size = ldf.signals.get(&mapping.signal_name).map(|s| s.size).unwrap_or(0);
                                channel_items.push(SidebarItem::SignalItem {
                                    name: mapping.signal_name.clone(),
                                    id: signal_id.clone(),
                                    size: sig_size,
                                    is_selected: app.selected_signals.contains(&signal_id),
                                    is_can: false,
                                    ch_id,
                                    msg_id: frame.id,
                                });
                            }
                        }
                    }
                }
            }

            if channel_has_matches {
                items.push(SidebarItem::ChannelHeader {
                    name: ch_name,
                    ch_id,
                    is_can: false,
                    is_loaded: true,
                    mapping: None,
                    is_expanded,
                    selected_count: selected_by_channel.get(&ch_id).copied().unwrap_or(0),
                });
                if is_expanded {
                    items.extend(channel_items);
                }
            }
        }
    }

    // === Unloaded configured channels (only ChannelHeader, no children) ===
    let loaded_channels: std::collections::HashSet<u16> = items.iter()
        .filter_map(|i| if let SidebarItem::ChannelHeader { ch_id, .. } = i { Some(*ch_id) } else { None })
        .collect();

    for mapping in &app.app_config.mappings {
        if mapping.library_id.is_some() && mapping.version_name.is_some() {
            if !loaded_channels.contains(&mapping.channel_id) {
                let ch_id = mapping.channel_id;
                let ch_type_str = if mapping.channel_type.is_can() { "CAN" } else { "LIN" };
                let ch_name = format!("Channel {} ({}) [Unloaded]", ch_id, ch_type_str);

                if filter_text.is_empty() || ch_name.to_lowercase().contains(&filter_text) {
                    items.push(SidebarItem::ChannelHeader {
                        name: ch_name,
                        ch_id,
                        is_can: mapping.channel_type.is_can(),
                        is_loaded: false,
                        mapping: Some(mapping.clone()),
                        is_expanded: false, // Unloaded channels are never expandable
                        selected_count: selected_by_channel.get(&ch_id).copied().unwrap_or(0),
                    });
                }
            }
        }
    }

    items
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --package view --lib ui::views::plot_sidebar::tests 2>&1 | tail -30`
Expected: 5 tests PASS.

- [ ] **Step 5: Run the full test suite to confirm no regression**

Run: `cargo test --package view 2>&1 | tail -20`
Expected: ALL tests PASS.

- [ ] **Step 6: Commit**

```bash
git add src/view/src/ui/views/plot_sidebar.rs
git commit -m "feat(plot): implement extract_signal_items pure fn with fold/search tests"
```

---

## Task 5: Implement `render_signal_sidebar` (migrated from chart_view.rs, plus "Clear all" button)

**Files:**
- Modify: `src/view/src/ui/views/plot_sidebar.rs` (replace the stub `render_signal_sidebar`)

**Interfaces:**
- Consumes: `extract_signal_items` from Task 4, `render_sidebar_item` from Task 3, `CanViewApp::clear_selected_signals` from Task 2
- Produces: `plot_sidebar::render_signal_sidebar(window, app, view, cx) -> impl IntoElement` — called by `chart_view.rs::render_plot_view`

- [ ] **Step 1: Implement `render_signal_sidebar`**

Replace the stub `render_signal_sidebar` in `src/view/src/ui/views/plot_sidebar.rs` with:

```rust
/// Render the signal selection sidebar: header + search box + virtualized list
/// + bottom action bar with "Clear all" and "Plot N signals" buttons.
pub fn render_signal_sidebar(
    _window: &mut Window,
    app: &CanViewApp,
    view: Entity<CanViewApp>,
    cx: &mut Context<CanViewApp>,
) -> impl IntoElement {
    let items = extract_signal_items(app);
    let item_count = items.len();
    let selected_count = app.selected_signals.len();
    let items_arc = std::sync::Arc::new(items);

    div()
        .size_full()
        .flex()
        .flex_col()
        .bg(rgb(0x0a0a0b))
        .child(
            // Sidebar Header
            div()
                .px_4()
                .py_2()
                .bg(rgb(0x131314))
                .border_b_1()
                .border_color(rgb(0x27272a))
                .flex()
                .items_center()
                .justify_between()
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
        )
        .child(
            // Search Box
            div()
                .w_full()
                .px_4()
                .py_2()
                .border_b_1()
                .border_color(rgb(0x27272a))
                .flex()
                .items_center()
                .child(
                    if let Some(input) = &app.signal_search_input {
                        div()
                            .flex_1()
                            .h(px(32.0))
                            .flex()
                            .items_center()
                            .child(gpui_component::input::Input::new(input).appearance(true))
                            .into_any_element()
                    } else {
                        div()
                            .flex_1()
                            .h(px(32.0))
                            .flex()
                            .items_center()
                            .px_2()
                            .text_xs()
                            .text_color(rgb(0x888888))
                            .child("Search signals...")
                            .into_any_element()
                    }
                )
        )
        .child(
            // Virtualized list
            div()
                .flex_1()
                .child(
                    if item_count == 0 {
                        div()
                            .p_4()
                            .text_xs()
                            .text_color(rgb(0x52525b))
                            .text_center()
                            .child("No matches found")
                            .into_any_element()
                    } else {
                        let view_entity = view.clone();
                        gpui::uniform_list(
                            "signal-list",
                            item_count,
                            move |range, _window, _cx| {
                                let items = items_arc.clone();
                                range.map(|i| render_sidebar_item(&items[i], view_entity.clone()))
                                    .collect::<Vec<_>>()
                            }
                        )
                        .size_full()
                        .into_any_element()
                    }
                )
        )
        .child(
            // Bottom Action Bar: Clear all | Plot N signals
            if selected_count > 0 {
                div()
                    .p_2()
                    .bg(rgb(0x131314))
                    .border_t_1()
                    .border_color(rgb(0x27272a))
                    .flex()
                    .gap_2()
                    .child(
                        // Clear all button
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
                        // Plot button
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
            } else {
                div()
            }
        )
}
```

- [ ] **Step 2: Run `cargo check --package view` to confirm it compiles**

Run: `cargo check --package view 2>&1 | tail -20`
Expected: BUILD SUCCEEDED.

- [ ] **Step 3: Run all tests to confirm no regression**

Run: `cargo test --package view 2>&1 | tail -20`
Expected: ALL tests PASS.

- [ ] **Step 4: Manual smoke test — start the app and verify the sidebar renders**

Run: `cargo run --release --bin view &` (then close after viewing)
Expected: App starts, plot view shows the sidebar with fold arrows. With a BLF + DBC loaded, channels appear folded (▸), clicking expands them, "Clear all" button appears when ≥ 1 signal selected. (Manual verification — GPUI render can't be unit tested.)

- [ ] **Step 5: Commit**

```bash
git add src/view/src/ui/views/plot_sidebar.rs
git commit -m "feat(plot): render sidebar with fold/clear-all/plot buttons via extract_signal_items"
```

---

## Task 6: Change `Series.name` to full signal ID + show signal name in legend/tooltip

**Files:**
- Modify: `src/view/src/ui/views/chart_view.rs:1599` (Series name in `extract_series_data`)
- Modify: `src/view/src/ui/views/chart_view.rs:1109` (hover tooltip display)
- Modify: `src/view/src/ui/views/chart_view.rs:1254` (legend display)

**Interfaces:**
- Consumes: nothing new
- Produces: `Series.name == full signal_id` (e.g., `"CAN:1:256:EngineSpeed"`) — required by Task 7's no-data matching

- [ ] **Step 1: Write the failing test for `Series.name` format**

In `src/view/src/ui/views/chart_view.rs`, append a `#[cfg(test)] mod tests` block at the end of the file:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies that the signal ID format produced by the sidebar matches what
    /// `extract_series_data` will set as `Series.name`. This is a structural
    /// test of the ID format — it doesn't run `extract_series_data` (which
    /// needs a full message log) but locks the format convention.
    #[test]
    fn signal_id_format_matches_series_name_convention() {
        let sidebar_signal_id = "CAN:1:256:EngineSpeed";
        let parts: Vec<&str> = sidebar_signal_id.split(':').collect();
        assert!(parts.len() >= 4, "signal id must have BUS:CH:MSG:NAME structure");
        // The Series.name after this task's change will equal the full signal_id;
        // display names extract the last segment:
        let display_name = sidebar_signal_id.split(':').last().unwrap();
        assert_eq!(display_name, "EngineSpeed");
    }
}
```

- [ ] **Step 2: Run test to verify it passes (sanity check on format convention)**

Run: `cargo test --package view --lib ui::views::chart_view::tests 2>&1 | tail -20`
Expected: PASS (this test verifies the convention, not the code change). The next step changes the actual code.

- [ ] **Step 3: Change `Series.name` from `sig_name` to `sig_id`**

In `src/view/src/ui/views/chart_view.rs`, find line 1598-1604:

```rust
        all_series.push(Series {
            name: sig_name.to_string(),
            unit,
            points: points.into(),
            color: colors[idx % colors.len()],
            time_labels,
        });
```

Change `name: sig_name.to_string(),` to `name: sig_id.clone(),`.

- [ ] **Step 4: Update the legend to show the bare signal name (not the full ID)**

Find `render_legend` (around line 1227-1257). Replace the line that uses `series.name.clone()`:

```rust
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(0xa1a1aa))
                        .child(series.name.clone())
                )
```

with:

```rust
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(0xa1a1aa))
                        .child(series.name.split(':').last().unwrap_or(&series.name).to_string())
                )
```

- [ ] **Step 5: Update the hover tooltip to show the bare signal name**

Find `render_hover_tooltip` around line 1109 where it does `format!("{}: {:.2} {}", series.name, val, ...)`. Change `series.name` to `series.name.split(':').last().unwrap_or(&series.name)`:

```rust
                        .child(format!("{}: {:.2} {}", series.name.split(':').last().unwrap_or(&series.name), val, series.unit.as_deref().unwrap_or("")))
```

- [ ] **Step 6: Update `render_single_chart`'s header line for consistency**

Find line 1169-1177 (the `format!("{} {} | {} pts | ...", series.name, ...)` line). Replace `series.name` with the bare name:

```rust
                    .child(format!(
                        "{} {} | {} pts | {:.3}s-{:.3}s (span: {:.3}s)", 
                        series.name.split(':').last().unwrap_or(&series.name), 
                        series.unit.as_ref().map(|u| format!("[{}]", u)).unwrap_or_default(),
                        series.points.len(),
                        min_time,
                        max_time,
                        time_span
                    ))
```

- [ ] **Step 7: Run `cargo check --package view` to confirm it compiles**

Run: `cargo check --package view 2>&1 | tail -20`
Expected: BUILD SUCCEEDED.

- [ ] **Step 8: Run all tests**

Run: `cargo test --package view 2>&1 | tail -20`
Expected: ALL tests PASS.

- [ ] **Step 9: Commit**

```bash
git add src/view/src/ui/views/chart_view.rs
git commit -m "feat(plot): use full signal_id as Series.name; legend/tooltip show bare name"
```

---

## Task 7: Add `render_no_data_chart` and modify `render_chart_canvas` to iterate `selected_signals`

**Files:**
- Modify: `src/view/src/ui/views/chart_view.rs:611-700` (render_chart_canvas)
- Modify: `src/view/src/ui/views/chart_view.rs:1115-1137` (delete dead `points.is_empty()` branch in `render_single_chart`)
- Add new function `render_no_data_chart` in `chart_view.rs`

**Interfaces:**
- Consumes: `Series.name == full signal_id` from Task 6
- Produces: `render_chart_canvas` iterates `app.selected_signals` and renders a no-data placeholder for selected-but-missing signals

- [ ] **Step 1: Add `render_no_data_chart` function**

Add this function in `src/view/src/ui/views/chart_view.rs` near `render_single_chart` (just before `render_single_chart` at line 1115):

```rust
/// Render a placeholder card matching `render_single_chart`'s visual style
/// for a selected signal that has no data points in the current log.
fn render_no_data_chart(signal_id: &str) -> AnyElement {
    let display_name = signal_id.split(':').last().unwrap_or(signal_id);
    div()
        .flex()
        .flex_col()
        .h(px(250.0))
        .bg(rgb(0x18181b))
        .border_1()
        .border_color(rgb(0x27272a))
        .rounded_lg()
        .p_4()
        .items_center()
        .justify_center()
        .child(
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap_2()
                .child(div().text_xl().text_color(rgb(0x71717a)).child("⊘"))
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(0xa1a1aa))
                        .child(format!("No data for '{}'", display_name))
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(0x52525b))
                        .child("检查通道 ID 匹配 (DBC vs 日志) 或时间范围")
                )
        )
        .into_any_element()
}
```

- [ ] **Step 2: Modify `render_chart_canvas` to iterate `selected_signals`**

Find `render_chart_canvas` at line 611. The relevant block is around line 633-637:

```rust
                .child(render_legend(&series_data))
                .children(series_data.iter().map(|series| {
                    render_single_chart(series, start_time, show_points)
                }))
```

Replace the `.children(...)` block with:

```rust
                .child(render_legend(&series_data))
                .children(app.selected_signals.iter().map(|signal_id| {
                    let series = series_data.iter().find(|s| &s.name == signal_id);
                    match series {
                        Some(s) => render_single_chart(s, start_time, show_points).into_any_element(),
                        None => render_no_data_chart(signal_id),
                    }
                }))
```

- [ ] **Step 3: Delete the dead `points.is_empty()` branch in `render_single_chart`**

Find `render_single_chart` at line 1115-1137. Delete the early-return:

```rust
    // Safety check: ensure we have points
    if series.points.is_empty() {
        return div()
            .flex()
            .flex_col()
            .h(px(250.0))
            .bg(rgb(0x18181b))
            .border_1()
            .border_color(rgb(0x27272a))
            .rounded_lg()
            .p_4()
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(0xa1a1aa))
                    .child(format!("No data points for '{}'. Check Channel ID match (DBC vs Log) or Time Range.", series.name))
            );
    }
```

The function now assumes `series.points` is non-empty (the caller `render_chart_canvas` ensures this by routing empty series to `render_no_data_chart` instead).

- [ ] **Step 4: Run `cargo check --package view` to confirm it compiles**

Run: `cargo check --package view 2>&1 | tail -20`
Expected: BUILD SUCCEEDED.

- [ ] **Step 5: Run all tests**

Run: `cargo test --package view 2>&1 | tail -20`
Expected: ALL tests PASS.

- [ ] **Step 6: Manual smoke test — selected signal without data shows ⊘ placeholder**

Run: `cargo run --release --bin view &`
Expected: Load a BLF + DBC. In the plot sidebar, select a signal whose message ID doesn't appear in the log. Click "Plot". The right pane shows the no-data placeholder card with "⊘ No data for {signal_name}". (Manual verification — can't unit test render.)

- [ ] **Step 7: Commit**

```bash
git add src/view/src/ui/views/chart_view.rs
git commit -m "feat(plot): no-data placeholder card for selected signals missing from log"
```

---

## Task 8: Enter-key field jumping in the add-channel form

**Files:**
- Modify: `src/view/src/app/state.rs` (already added `pending_add_channel_focus` in Task 1)
- Modify: `src/view/src/ui/views/library_management.rs:1300-1349` (render_add_channel_button: add PressEnter subscriptions)
- Modify: `src/view/src/ui/views/library_management.rs:1381-1394` (render_add_channel_input_row_with_path: remove `enter` from on_key_down, add `enter` to ✓ Confirm button)
- Modify: `src/view/src/app/impls_rendering.rs:1650` (consume `pending_add_channel_focus` in render)

**Interfaces:**
- Consumes: `PendingAddChannelFocus` enum and `CanViewApp::pending_add_channel_focus` from Task 1
- Produces: Enter-key in channel_id focuses channel_name; Enter-key in channel_name focuses ✓; Enter on ✓ submits the form

- [ ] **Step 1: Add PressEnter subscriptions in `render_add_channel_button`**

In `src/view/src/ui/views/library_management.rs`, find the `render_add_channel_button` function's click handler (around lines 1316-1349). Currently:

```rust
                let id_input = cx.new(|cx| InputState::new(window, cx).placeholder("Channel ID"));
                cx.subscribe(&id_input, |this, input, event, cx| {
                    if let gpui_component::input::InputEvent::Change = event {
                        this.new_channel_id = input.read(cx).text().to_string();
                    }
                })
                .detach();
                this.channel_id_input = Some(id_input);

                let name_input =
                    cx.new(|cx| InputState::new(window, cx).placeholder("Channel name"));
                cx.subscribe(&name_input, |this, input, event, cx| {
                    if let gpui_component::input::InputEvent::Change = event {
                        this.new_channel_name = input.read(cx).text().to_string();
                    }
                })
                .detach();
                this.channel_name_input = Some(name_input);
```

Replace both `cx.subscribe` blocks to add `PressEnter` handling:

```rust
                let id_input = cx.new(|cx| InputState::new(window, cx).placeholder("Channel ID"));
                cx.subscribe(&id_input, |this, input, event, cx| {
                    match event {
                        gpui_component::input::InputEvent::Change => {
                            this.new_channel_id = input.read(cx).text().to_string();
                        }
                        gpui_component::input::InputEvent::PressEnter { .. } => {
                            // Defer focus to next render where window is available
                            this.pending_add_channel_focus =
                                Some(crate::app::state::PendingAddChannelFocus::ChannelName);
                            cx.notify();
                        }
                        _ => {}
                    }
                })
                .detach();
                this.channel_id_input = Some(id_input);

                let name_input =
                    cx.new(|cx| InputState::new(window, cx).placeholder("Channel name"));
                cx.subscribe(&name_input, |this, input, event, cx| {
                    match event {
                        gpui_component::input::InputEvent::Change => {
                            this.new_channel_name = input.read(cx).text().to_string();
                        }
                        gpui_component::input::InputEvent::PressEnter { .. } => {
                            // Defer focus to next render where window is available
                            this.pending_add_channel_focus =
                                Some(crate::app::state::PendingAddChannelFocus::ChannelConfirm);
                            cx.notify();
                        }
                        _ => {}
                    }
                })
                .detach();
                this.channel_name_input = Some(name_input);
```

- [ ] **Step 2: Modify `render_add_channel_input_row_with_path` — remove Enter from row `on_key_down`**

In `src/view/src/ui/views/library_management.rs:1381-1394`, find:

```rust
        .on_key_down(cx.listener(|this, event: &KeyDownEvent, _window, cx| {
            if event.keystroke.key == "escape" {
                // Close the input without saving
                this.show_add_channel_input = false;
                this.channel_id_input = None;
                this.channel_name_input = None;
                this.channel_db_path_input = None;
                this.new_channel_db_path.clear();
                cx.notify();
            } else if event.keystroke.key == "enter" {
                // Save the channel configuration
                this.save_channel_config(cx);
            }
        }))
```

Replace with (remove the `enter` branch):

```rust
        .on_key_down(cx.listener(|this, event: &KeyDownEvent, _window, cx| {
            if event.keystroke.key == "escape" {
                // Close the input without saving
                this.show_add_channel_input = false;
                this.channel_id_input = None;
                this.channel_name_input = None;
                this.channel_db_path_input = None;
                this.new_channel_db_path.clear();
                cx.notify();
            }
            // Enter key handling moved to per-input PressEnter subscriptions
            // (see render_add_channel_button) and the ✓ Confirm button.
        }))
```

- [ ] **Step 3: Add `on_key_down` Enter handler to the ✓ Confirm button**

In `render_add_channel_input_row_with_path` find the ✓ Confirm button `add-ch-confirm` (around line 1561). Current code:

```rust
                        .on_mouse_down(
                            gpui::MouseButton::Left,
                            cx.listener(|this, _, _, cx| {
                                eprintln!("🖱️ Confirm button clicked");
                                this.save_channel_config(cx);
                            }),
                        ),
```

Add `.on_key_down` after `.on_mouse_down`:

```rust
                        .on_mouse_down(
                            gpui::MouseButton::Left,
                            cx.listener(|this, _, _, cx| {
                                eprintln!("🖱️ Confirm button clicked");
                                this.save_channel_config(cx);
                            }),
                        )
                        .on_key_down(cx.listener(|this, event: &KeyDownEvent, _window, cx| {
                            if event.keystroke.key == "enter" {
                                this.save_channel_config(cx);
                            }
                        })),
```

Note the comma at the end of `.on_mouse_down(...)` becomes a separator; verify the chain ends with `)` after the new `.on_key_down(...)`.

- [ ] **Step 4: Consume `pending_add_channel_focus` in the top-level render**

In `src/view/src/app/impls_rendering.rs`, find the `render` function at line 1650. Right after `self.update_container_height(window);` (line 1652), add:

```rust
        // Consume pending focus from add-channel Enter-key chain (set by PressEnter subscribe)
        if let Some(target) = self.pending_add_channel_focus.take() {
            use crate::app::state::PendingAddChannelFocus;
            match target {
                // Enter on channel_id → focus the channel_name input
                PendingAddChannelFocus::ChannelName => {
                    if let Some(name_input) = &self.channel_name_input {
                        name_input.update(cx, |state, cx| state.focus(window, cx));
                    }
                }
                // Enter on channel_name → user decided "let user choose next step"
                // (no auto-focus on ✓ Confirm button — user picks Browse or ✓ manually).
                // Re-focus the name input so it's not lost; the user then either clicks
                // "Select File..." or tabs to ✓ to submit.
                PendingAddChannelFocus::ChannelConfirm => {
                    if let Some(name_input) = &self.channel_name_input {
                        name_input.update(cx, |state, cx| state.focus(window, cx));
                    }
                }
            }
        }
```

**Design note:** The `ChannelConfirm` branch re-focuses the name input (effectively a no-op since Enter on name already leaves focus there). This implements the user's decision "name 输入框之后让用户自行选择选择" — after pressing Enter on name, the user stays on the name field and manually chooses between "Select File..." (path) or ✓ (submit). A tighter UX would auto-focus the ✓ button, but that requires adding `add_ch_confirm_focus_handle: Option<gpui::FocusHandle>` to `CanViewApp` and a `.track_focus()` call on the button — out of scope for this task.

- [ ] **Step 5: Run `cargo check --package view` to confirm it compiles**

Run: `cargo check --package view 2>&1 | tail -20`
Expected: BUILD SUCCEEDED.

- [ ] **Step 6: Run all tests**

Run: `cargo test --package view 2>&1 | tail -20`
Expected: ALL tests PASS.

- [ ] **Step 7: Manual smoke test — Enter-key focus chain**

Run: `cargo run --release --bin view &`
Procedure:
1. Switch to Library tab, select a library/version
2. Click "+ Add Channel" — id input appears
3. Type a channel ID (e.g., `3`), press Enter — focus should jump to name input
4. Type a channel name (e.g., `TestChannel`), press Enter — focus should stay on name input (per the compromise above)
5. Click "Select File..." → pick a DBC → click ✓ — channel saved successfully
6. Press Escape at any step — form closes without saving

Expected: All steps work; no spurious save_channel_config calls from Enter on the id or name fields.

- [ ] **Step 8: Commit**

```bash
git add src/view/src/ui/views/library_management.rs src/view/src/app/impls_rendering.rs
git commit -m "fix(library): Enter in add-channel id/name jumps focus instead of submitting"
```

---

## Task 9: Final verification — full build, clippy, and end-to-end manual test

**Files:**
- No file changes — verification only

- [ ] **Step 1: Run the full test suite**

Run: `cargo test --workspace 2>&1 | tail -30`
Expected: ALL tests PASS across all crates (blf, parser, view).

- [ ] **Step 2: Run clippy**

Run: `cargo clippy --workspace 2>&1 | tail -30`
Expected: No new clippy warnings introduced by this plan. Existing warnings may remain.

- [ ] **Step 3: Run `cargo fmt --all`**

Run: `cargo fmt --all && git diff --name-only`
Expected: Any files modified by fmt are listed. If non-empty, stage and amend the last commit:

```bash
git add -u && git commit --amend --no-edit
```

- [ ] **Step 4: End-to-end manual test**

Run: `cargo run --release --bin view &`

Verify all 4 UX fixes:
1. **Fold**: plot sidebar shows channels with ▸ arrow; click expands (▾); expanding channel B collapses channel A; messages fold/unfold independently; search "engine" expands matching channels; clearing search restores manual fold state.
2. **Clear all**: select 3 signals → "清除全部 (3)" button appears next to "绘制 3 个信号 (Plot)"; clicking it clears selection + plot + zoom.
3. **No-data placeholder**: select a signal whose message ID is not in the loaded BLF → click Plot → ⊘ placeholder card appears for that signal alongside any signals with data.
4. **Enter-key jump**: in Library tab add-channel form, Enter on ID field → focus jumps to name field; Enter on name field → focus stays (compromise; user tabs to ✓ or clicks ✓).

- [ ] **Step 5: Final commit if anything was amended**

If step 3 amended the last commit, push (or leave for the user to push):
```bash
git log --oneline -10
```
Expected: 8-9 commits on top of `feat/ui-redesign` base, one per task (+ possibly one amend for fmt).

---

## Self-Review Notes

After completing all tasks, verify against the spec (`docs/superpowers/specs/2026-07-28-plot-ux-fixes-design.md`):

| Spec requirement | Implemented by |
|---|---|
| Channel/msg/signal 3-level fold, accordion at channel level | Tasks 1, 2, 3, 4 |
| Default all folded, session memory, no disk persistence | Task 1 (RuntimeState), Task 4 (force-expand only on search) |
| Search auto-expand matching, restore on clear | Task 4 `extract_signal_items` `force_expand` logic |
| "Clear all" button clears selected + plot + zoom | Task 2 `clear_selected_signals`, Task 5 button |
| No-data placeholder matches chart card style | Task 7 `render_no_data_chart` |
| `Series.name` = full signal_id; legend/tooltip show bare name | Task 6 |
| `render_single_chart` `points.is_empty()` branch deleted | Task 7 Step 3 |
| Enter-key chain: id → name → ✓ Confirm; ✓ + Enter submits | Task 8 |
| Existing `escape` row-level handler preserved | Task 8 Step 2 |
| `extract_signal_items` is a pure function with unit tests | Task 4 |
| `RuntimeState` carries fold state for maximize/restore | Task 1 |
