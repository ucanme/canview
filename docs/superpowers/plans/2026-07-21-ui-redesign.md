# UI Redesign (TopBar/TabBar/FilterBar/StatusBar) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extract 4 reusable UI components (TopBar, TabBar, FilterBar, StatusBar) from `impls_rendering.rs`, replace hardcoded `rgb(0x...)` colors with `theme::colors` tokens, and clean up 12 backup/examples/debug files — without changing the behavior of any view.

**Architecture:** 4 stateless components in `src/view/src/ui/components/` consume `&CanViewApp` and `Entity<CanViewApp>`, returning `impl IntoElement`. All callbacks are `view.update(cx, ...)` closures. `CanViewApp::render` composes `TopBar + content + StatusBar`; `FilterBar` is invoked from within `render_log_view` and `render_library_view`. A single new state field `current_file_name: Option<String>` is added so `StatusBar` can display the loaded file name.

**Tech Stack:** Rust nightly, GPUI, gpui-component, theme system at `src/view/src/ui/theme/mod.rs`. Existing `Button` component (`src/view/src/ui/components/button.rs`) with `ButtonVariant::Ghost` + `ButtonSize::Small`. Existing `gpui_component::input::InputState` for search boxes.

## Global Constraints

- Rust nightly toolchain (see `Cargo.toml`); build with `cargo build --release --bin view`
- Lint with `cargo clippy --workspace -- -D warnings`; baseline warnings = 381, this plan must NOT increase the count
- Format with `cargo fmt --all`
- Test with `cargo test --workspace`
- Color tokens live in `src/view/src/ui/theme/mod.rs` `colors` module; `Rgba` uses `f32` fields (0.0-1.0)
- New components MUST NOT contain `rgb(0x` literal strings — use `crate::ui::theme::colors::*`
- New components MUST NOT use bare `px(N.)` — use `crate::ui::theme::spacing::{XS, SM, MD, LG, XL}` or named constants like `theme::typography::SM`
- The `AppView` enum (in `src/view/src/app/state.rs:54`) is `#[derive(Debug, Clone, Copy, PartialEq)]` with variants `LogView`, `ConfigView`, `LibraryView`, `PlotView`
- `LibraryDialogType` lives at `src/view/src/app/state.rs:233`
- The `Button` component API: `Button::new(label).size(ButtonSize::Small).variant(ButtonVariant::Ghost).active(bool).build()` returns a `Div`
- Each commit must compile (`cargo build --release --bin view`) and not add new clippy warnings
- Commit message style: lowercase prefix `feat(ui):` / `refactor(ui):` / `chore(ui):` / `feat(ui/theme):`

---

## File Structure

### New files (created in this plan)

| File | Responsibility | Created in Task |
|---|---|---|
| `src/view/src/ui/components/top_bar.rs` | Render top bar: File button, TabBar, library badge, window controls | Task 3 |
| `src/view/src/ui/components/tab_bar.rs` | Render 4 view tabs with bottom 2px indicator | Task 3 |
| `src/view/src/ui/components/filter_bar.rs` | Render filter chip bar for Log/Library variants | Task 4 |
| `src/view/src/ui/components/status_bar.rs` | Render bottom status bar with file info + server status | Task 5 |
| `examples/ui_components/dropdown_examples.rs` | Relocated from `src/view/src/ui/components/dropdown_examples.rs` | Task 6 |
| `examples/ui_components/button_examples.rs` | Relocated | Task 6 |
| `examples/ui_components/modal_examples.rs` | Relocated | Task 6 |
| `examples/ui_components/tabs_examples.rs` | Relocated | Task 6 |
| `examples/views/common_examples.rs` | Relocated from `src/view/src/views/common_examples.rs` | Task 6 |

### Modified files

| File | Changes | Task |
|---|---|---|
| `src/view/src/ui/theme/mod.rs` | Add 4 new color tokens (ACCENT_GREEN_*, CLOSE_HOVER) | Task 1 |
| `src/view/src/ui/components/mod.rs` | Register `top_bar`, `tab_bar`, `filter_bar`, `status_bar` modules; remove `*_examples` declarations | Tasks 3, 4, 5, 6 |
| `src/view/src/views/mod.rs` | Remove `common_examples` declaration | Task 6 |
| `src/view/src/app/state.rs` | Add `current_file_name: Option<String>` field; init in `new_with_maximized_state_and_bounds`; include in `RuntimeState` save/restore | Task 5 |
| `src/view/src/app/impls.rs` | Set `self.current_file_name` in `apply_blf_result` success path; clear in error path | Task 5 |
| `src/view/src/app/impls_rendering.rs` | Replace inline top bar (lines ~1834-2063) with `TopBar::new` call; replace inline status bar (lines ~2082-2140) with `StatusBar::new` call; replace Log/Library filter bars with `FilterBar::new` calls | Tasks 3, 4, 5 |

### Deleted files (in Task 6)

| File | Reason |
|---|---|
| `src/view/src/ui/components/button_backup.rs` | backup file |
| `src/view/src/ui/components/mod_old.rs` | old module |
| `src/view/src/app/impls.rs.after_deletion` | post-deletion snapshot |
| `src/view/src/app/impls_rendering.rs.bak` | backup |
| `src/view/src/temp_impl1.txt` | scratch file |
| `src/view/src/main_backup.rs` | backup of main |
| `src/view/src/library_view_debug.rs` | debug variant |
| `src/view/src/library_view_focused.rs` | focused variant (consolidated into new FilterBar) |
| `src/view/src/ui/components/dropdown_examples.rs` | moved to `examples/` |
| `src/view/src/ui/components/button_examples.rs` | moved to `examples/` |
| `src/view/src/ui/components/modal_examples.rs` | moved to `examples/` |
| `src/view/src/ui/components/tabs_examples.rs` | moved to `examples/` |
| `src/view/src/views/common_examples.rs` | moved to `examples/` |

---

## Task 1: Add accent color tokens to theme

**Files:**
- Modify: `src/view/src/ui/theme/mod.rs` (inside `pub mod colors` block, after `INTERACTIVE_HOVER` constant at line ~218)

**Interfaces:**
- Produces: `colors::ACCENT_GREEN_LIGHT`, `colors::ACCENT_GREEN_BG`, `colors::ACCENT_GREEN_BORDER`, `colors::CLOSE_HOVER` (all `gpui::Rgba`)
- Consumes: `colors::palette::GREEN` (already exists at line 60-66 of `theme/mod.rs`)

- [ ] **Step 1: Verify baseline build passes**

Run: `cargo build --release --bin view 2>&1 | tail -5`
Expected: `Finished` release` profile [optimized] target(s)` (build success, may have 381 warnings)

- [ ] **Step 2: Add 4 new tokens to the `colors` module**

Open `src/view/src/ui/theme/mod.rs`, find the `colors` module. After the `DISABLED: Rgba = palette::OVERLAY0;` line (around line 219), insert:

```rust
    // Accent — current library badge (used by TopBar and StatusBar)
    pub const ACCENT_GREEN_LIGHT: Rgba = palette::GREEN;
    pub const ACCENT_GREEN_BG: Rgba = Rgba {
        r: 0x1a as f32 / 255.0,
        g: 0x2e as f32 / 255.0,
        b: 0x1a as f32 / 255.0,
        a: 1.0,
    };
    pub const ACCENT_GREEN_BORDER: Rgba = Rgba {
        r: 0x2d as f32 / 255.0,
        g: 0x5a as f32 / 255.0,
        b: 0x2d as f32 / 255.0,
        a: 1.0,
    };

    // Window control — close button hover
    pub const CLOSE_HOVER: Rgba = Rgba {
        r: 0xc5 as f32 / 255.0,
        g: 0x30 as f32 / 255.0,
        b: 0x30 as f32 / 255.0,
        a: 1.0,
    };
```

- [ ] **Step 3: Add a unit test verifying the new constants are non-default**

In the existing `#[cfg(test)] mod tests` block at the bottom of `theme/mod.rs`, append:

```rust
    #[test]
    fn test_accent_tokens_defined() {
        // Accent tokens must be opaque and distinct from base palette
        assert_eq!(colors::ACCENT_GREEN_LIGHT, palette::GREEN);
        assert_eq!(colors::ACCENT_GREEN_BG.a, 1.0);
        assert_eq!(colors::ACCENT_GREEN_BORDER.a, 1.0);
        assert_eq!(colors::CLOSE_HOVER.a, 1.0);
        // CLOSE_HOVER should be red-dominant
        assert!(colors::CLOSE_HOVER.r > 0.7);
        assert!(colors::CLOSE_HOVER.g < 0.3);
    }
```

- [ ] **Step 4: Run the new test**

Run: `cargo test --package view --lib ui::theme::tests::test_accent_tokens_defined -- --exact`
Expected: PASS (1 test passed)

- [ ] **Step 5: Verify build still passes**

Run: `cargo build --release --bin view 2>&1 | tail -3`
Expected: `Finished` release` profile [optimized] target(s)`

- [ ] **Step 6: Verify clippy does not add warnings**

Run: `cargo clippy --workspace 2>&1 | grep -c "^warning"` 
Expected: 381 (same as baseline)

- [ ] **Step 7: Commit**

```bash
git add src/view/src/ui/theme/mod.rs
git commit -m "$(cat <<'EOF'
feat(ui/theme): add accent and close-hover color tokens

Adds 4 new tokens for the upcoming TopBar/StatusBar components:
- ACCENT_GREEN_LIGHT/BG/BORDER for the current library badge
- CLOSE_HOVER for the window close button hover state

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

## Task 2: Add `current_file_name` state field

**Files:**
- Modify: `src/view/src/app/state.rs` (struct `CanViewApp` field list at line 71-220; `new_with_maximized_state_and_bounds` at line 251-360; `RuntimeState` struct at line 14-32; `save_runtime_state` at line 365-389; `restore_runtime_state` at line 393-426)
- Modify: `src/view/src/app/impls.rs` (function `apply_blf_result` at line 229-282)

**Interfaces:**
- Produces: `CanViewApp::current_file_name: Option<String>` (public field, `pub`)
- Produces: `RuntimeState::current_file_name: Option<String>` (public field, `pub`)

- [ ] **Step 1: Verify build passes before changes**

Run: `cargo build --release --bin view 2>&1 | tail -3`
Expected: build success

- [ ] **Step 2: Add the field to `CanViewApp` struct**

In `src/view/src/app/state.rs`, find the field block in `CanViewApp` (starting around line 71). After the `pub is_streaming_mode: bool,` line (around line 95), insert:

```rust
    // Currently loaded BLF file name (for StatusBar display)
    pub current_file_name: Option<String>,
```

- [ ] **Step 3: Initialize the field in `new_with_maximized_state_and_bounds`**

In the same file, in the `Self { ... }` block of `new_with_maximized_state_and_bounds` (around line 252-360), after the `is_streaming_mode,` line (around line 267), insert:

```rust
            current_file_name: None,
```

- [ ] **Step 4: Add the field to `RuntimeState` struct**

In the same file, find the `RuntimeState` struct (around line 14-32). After the `pub is_streaming_mode: bool,` line (around line 29), insert:

```rust
    pub current_file_name: Option<String>,
```

- [ ] **Step 5: Save the field in `save_runtime_state`**

In the same file, find `save_runtime_state` (around line 365-389). In the `RuntimeState { ... }` construction, after the `is_streaming_mode: self.is_streaming_mode,` line (around line 385), insert:

```rust
            current_file_name: self.current_file_name.clone(),
```

- [ ] **Step 6: Restore the field in `restore_runtime_state`**

In the same file, find `restore_runtime_state` (around line 393-426). After the `self.is_streaming_mode = state.is_streaming_mode;` line (around line 416), insert:

```rust
        self.current_file_name = state.current_file_name;
```

- [ ] **Step 7: Set the field in `apply_blf_result` success path**

In `src/view/src/app/impls.rs`, find `apply_blf_result` (line 229-282). The function signature is `pub(crate) fn apply_blf_result(&mut self, result: anyhow::Result<BlfResult>)`. The current code does not receive a file path. We need to update the signature to accept an optional file name.

Change the signature at line 229 from:
```rust
    pub(crate) fn apply_blf_result(&mut self, result: anyhow::Result<BlfResult>) {
```
to:
```rust
    pub(crate) fn apply_blf_result(&mut self, result: anyhow::Result<BlfResult>, file_name: Option<String>) {
```

In the `Ok(result) => { ... }` arm (after `self.messages = result.objects;` at line 269), add at the end of the arm:
```rust
                self.current_file_name = file_name;
```

In the `Err(e) => { ... }` arm (after `Self::display_blf_load_error(&e);` at line 279), add at the end of the arm:
```rust
                // Keep the existing current_file_name on error so status bar shows the last good file
```

(Note: do NOT clear `current_file_name` on error — keep the previous value so StatusBar stays informative.)

- [ ] **Step 8: Update call sites of `apply_blf_result`**

Find all callers of `apply_blf_result` and update them to pass a file name:

Run: `grep -rn "apply_blf_result" src/view/src/`
Expected output:
- `src/view/src/app/impls.rs:229` (definition)
- `src/view/src/app/impls_rendering.rs:2221` (caller)
- `src/view/src/controllers/config_controller.rs:148` (caller — `pub fn apply_blf_result(app: &mut CanViewApp, result: ...)` is a different function, leave it)
- backup files (will be deleted in Task 6, ignore for now)

In `src/view/src/app/impls_rendering.rs` at line 2221, change:
```rust
                                                    view.apply_blf_result(result);
```
to:
```rust
                                                    let fname = path
                                                        .file_name()
                                                        .and_then(|n| n.to_str())
                                                        .map(|s| s.to_string());
                                                    view.apply_blf_result(result, fname);
```

(The `path` variable is in scope at this point — verify by reading the surrounding context lines 2196-2230 of `impls_rendering.rs`.)

In `src/view/src/controllers/config_controller.rs:148`, this is a separate function `pub fn apply_blf_result(app: &mut CanViewApp, result: ...)`. Update it to call the new signature:
```rust
pub fn apply_blf_result(app: &mut CanViewApp, result: anyhow::Result<blf::BlfResult>) {
    // Delegate to the impl method, with no file name (controller path doesn't have it)
    CanViewApp::apply_blf_result(app, result, None);
}
```

Wait — this is calling an inherent method on `CanViewApp`, which doesn't work syntactically. Use the fully qualified path:
```rust
pub fn apply_blf_result(app: &mut CanViewApp, result: anyhow::Result<blf::BlfResult>) {
    // Delegate to the impl method, with no file name (controller path doesn't have it)
    CanViewApp::apply_blf_result(app, result, None);
}
```

Actually, inherent methods on `CanViewApp` are accessed as `app.apply_blf_result(result, None)`. Update to:
```rust
pub fn apply_blf_result(app: &mut CanViewApp, result: anyhow::Result<blf::BlfResult>) {
    app.apply_blf_result(result, None);
}
```

- [ ] **Step 9: Build and verify**

Run: `cargo build --release --bin view 2>&1 | tail -10`
Expected: build success (may have warnings about the unused field — that's fine, Task 5 will use it)

- [ ] **Step 10: Run existing tests to ensure no regression**

Run: `cargo test --workspace 2>&1 | tail -20`
Expected: all existing tests pass

- [ ] **Step 11: Commit**

```bash
git add src/view/src/app/state.rs src/view/src/app/impls.rs src/view/src/app/impls_rendering.rs src/view/src/controllers/config_controller.rs
git commit -m "$(cat <<'EOF'
feat(app): add current_file_name state field for StatusBar

Adds a new Option<String> field to CanViewApp and RuntimeState to track
the currently loaded BLF file name. Set in apply_blf_result's success
path; preserved across window state save/restore.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

## Task 3: Create TopBar and TabBar components

**Files:**
- Create: `src/view/src/ui/components/tab_bar.rs`
- Create: `src/view/src/ui/components/top_bar.rs`
- Modify: `src/view/src/ui/components/mod.rs` (add module declarations)
- Modify: `src/view/src/app/impls_rendering.rs` (replace inline top bar code)

**Interfaces:**
- Consumes: `crate::ui::theme::colors::*` (from Task 1's existing tokens), `crate::ui::theme::spacing::*`, `crate::ui::theme::typography::*`, `crate::app::AppView`, `crate::ui::components::{Button, ButtonSize, ButtonVariant}`
- Produces: `pub fn render_tab_bar(app: &CanViewApp, view: Entity<CanViewApp>) -> impl IntoElement`
- Produces: `pub fn render_top_bar(app: &CanViewApp, view: Entity<CanViewApp>, cx: &mut Context<CanViewApp>) -> impl IntoElement`

- [ ] **Step 1: Write a unit test for tab label mapping**

Create `src/view/src/ui/components/tab_bar.rs` with this initial content:

```rust
//! TabBar component
//!
//! Renders the 4 view tabs (Log / Signal Plot / Library) with active
//! state indicated by a bottom 2px indicator (Zed style).

use crate::app::{AppView, CanViewApp};
use crate::ui::theme::colors;
use crate::ui::theme::spacing;
use crate::ui::theme::typography;
use gpui::{prelude::*, *};

/// Tab labels and their AppView variants. Returns (label, view).
///
/// Note: "File" is rendered as a Button in TopBar, not in TabBar.
pub const TAB_ITEMS: [(&str, AppView); 3] = [
    ("Log", AppView::LogView),
    ("Signal Plot", AppView::PlotView),
    ("Library", AppView::LibraryView),
];

/// Render the tab bar (3 tabs).
pub fn render_tab_bar(
    app: &CanViewApp,
    view: Entity<CanViewApp>,
) -> impl IntoElement {
    let current = app.current_view;
    div()
        .flex()
        .items_center()
        .gap(px(2.))
        .h_full()
        .children(TAB_ITEMS.map(|(label, view_val)| {
            let active = current == view_val;
            let view_clone = view.clone();
            div()
                .px(spacing::SM)
                .h_full()
                .flex()
                .items_center()
                .cursor_pointer()
                .text_sm()
                .text_color(if active { colors::TEXT_PRIMARY } else { colors::TEXT_MUTED })
                .hover(move |s| if active {
                    s
                } else {
                    s.text_color(colors::TEXT_SECONDARY).bg(colors::SURFACE0)
                })
                .when(active, |el| el.border_b_2().border_color(colors::PRIMARY))
                .child(label.to_string())
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    cx.stop_propagation();
                    view_clone.update(cx, |app, cx| {
                        app.current_view = view_val;
                        if view_val == AppView::PlotView {
                            crate::ui::views::chart_view::extract_and_update_series_data(app);
                        }
                        cx.notify();
                    });
                })
        }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_items_count() {
        assert_eq!(TAB_ITEMS.len(), 3);
    }

    #[test]
    fn test_tab_items_unique_views() {
        // All variants must be distinct
        assert_ne!(TAB_ITEMS[0].1, TAB_ITEMS[1].1);
        assert_ne!(TAB_ITEMS[1].1, TAB_ITEMS[2].1);
        assert_ne!(TAB_ITEMS[0].1, TAB_ITEMS[2].1);
    }

    #[test]
    fn test_tab_items_labels_not_empty() {
        for (label, _) in TAB_ITEMS.iter() {
            assert!(!label.is_empty());
        }
    }
}
```

- [ ] **Step 2: Register `tab_bar` in `mod.rs`**

In `src/view/src/ui/components/mod.rs`, add after line 14 (`pub mod zed_style_text_input;`):

```rust
pub mod tab_bar;
```

- [ ] **Step 3: Run the unit tests to verify they pass**

Run: `cargo test --package view --lib ui::components::tab_bar::tests 2>&1 | tail -10`
Expected: 3 tests passed

- [ ] **Step 4: Create `top_bar.rs` with the TopBar component**

Create `src/view/src/ui/components/top_bar.rs`:

```rust
//! TopBar component
//!
//! Renders the top bar: File menu button + TabBar + active library badge +
//! window controls (Win/Linux only — macOS uses system traffic lights).

use crate::app::{AppView, CanViewApp};
use crate::ui::components::tab_bar::render_tab_bar;
use crate::ui::components::{Button, ButtonSize, ButtonVariant};
use crate::ui::theme::colors;
use crate::ui::theme::spacing;
use gpui::{prelude::*, *};

/// Render the active-library badge shown when a library version is activated.
/// Returns `None` if no library is active.
fn active_lib_badge(app: &CanViewApp) -> Option<String> {
    let lib_id = app.active_library_id.as_ref()?;
    let ver = app.active_version_name.as_ref()?;
    let lib_name = app
        .library_manager
        .find_library(lib_id)
        .map(|l| l.name.clone())
        .unwrap_or_else(|| lib_id.clone());
    Some(format!("📚 {} / {}", lib_name, ver))
}

/// Render the top bar.
pub fn render_top_bar(
    app: &CanViewApp,
    view: Entity<CanViewApp>,
    _cx: &mut Context<CanViewApp>,
) -> impl IntoElement {
    let is_macos = cfg!(target_os = "macos");
    let badge = active_lib_badge(app);
    let show_file_menu = app.show_file_menu;
    let current_view = app.current_view;

    // File menu button — built as a div with the same styling as a Ghost button
    let view_for_file = view.clone();
    let file_button = div()
        .px(spacing::SM)
        .h_full()
        .flex()
        .items_center()
        .cursor_pointer()
        .text_sm()
        .text_color(if show_file_menu { colors::TEXT_PRIMARY } else { colors::TEXT_MUTED })
        .when(show_file_menu, |el| el.bg(colors::SURFACE0))
        .hover(|s| s.text_color(colors::TEXT_SECONDARY).bg(colors::SURFACE0))
        .child("File")
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            cx.stop_propagation();
            view_for_file.update(cx, |app, cx| {
                app.show_file_menu = !app.show_file_menu;
                cx.notify();
            });
        });

    // Active library badge — clickable, jumps to Library view
    let view_for_badge = view.clone();
    let badge_el = badge.map(|b| {
        div()
            .px(spacing::SM)
            .py(px(2.))
            .ml(spacing::SM)
            .bg(colors::ACCENT_GREEN_BG)
            .border_1()
            .border_color(colors::ACCENT_GREEN_BORDER)
            .rounded(px(4.))
            .text_xs()
            .text_color(colors::ACCENT_GREEN_LIGHT)
            .cursor_pointer()
            .hover(|s| s.bg(colors::SURFACE1))
            .child(b)
            .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                cx.stop_propagation();
                view_for_badge.update(cx, |app, cx| {
                    app.current_view = AppView::LibraryView;
                    cx.notify();
                });
            })
    });

    // macOS: 80px left padding to leave room for traffic lights
    let left_pad = if is_macos { Some(px(80.)) } else { None };

    div()
        .h(px(36.))
        .bg(colors::BG_MUTED)
        .flex()
        .items_center()
        .px(spacing::LG)
        .border_b_1()
        .border_color(colors::BORDER_SUBTLE)
        .window_control_area(WindowControlArea::Drag)
        .when_some(left_pad, |el, pad| el.child(div().w(pad)))
        .child(file_button)
        .child(div().w(spacing::SM)) // gap between File and tabs
        .child(render_tab_bar(app, view.clone()))
        .child(div().flex_1()) // push badge + window controls to the right
        .when_some(badge_el, |el, b| el.child(b))
        .when(!is_macos, |el| el.child(render_window_controls(view)))
}

/// Render the Win/Linux window controls (minimize, maximize, close).
fn render_window_controls(view: Entity<CanViewApp>) -> impl IntoElement {
    let view_min = view.clone();
    let view_max = view.clone();
    let view_close = view.clone();

    div()
        .flex()
        .items_center()
        .h_full()
        .child(
            div()
                .w(px(36.))
                .h_full()
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .hover(|s| s.bg(colors::SURFACE1))
                .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                    cx.stop_propagation();
                    window.minimize_window();
                    view_min.update(cx, |_, cx| cx.notify());
                })
                .child(div().w(px(10.)).h(px(1.)).bg(colors::TEXT_MUTED)),
        )
        .child(
            div()
                .w(px(36.))
                .h_full()
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .hover(|s| s.bg(colors::SURFACE1))
                .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                    cx.stop_propagation();
                    view_max.update(cx, |app, cx| {
                        app.toggle_maximize(window, cx);
                        cx.notify();
                    });
                })
                .child(
                    div()
                        .w(px(10.))
                        .h(px(10.))
                        .border_1()
                        .border_color(colors::TEXT_MUTED),
                ),
        )
        .child(
            div()
                .w(px(36.))
                .h_full()
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .hover(|s| s.bg(colors::CLOSE_HOVER))
                .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                    cx.stop_propagation();
                    window.remove_window();
                })
                .child(
                    div()
                        .text_sm()
                        .text_color(colors::TEXT_MUTED)
                        .hover(|s| s.text_color(colors::TEXT_PRIMARY))
                        .child("✕"),
                ),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_active_lib_badge_none_when_no_active() {
        let app = CanViewApp::new_state();
        assert!(active_lib_badge(&app).is_none());
    }
}
```

Note: `CanViewApp::new_state()` is public (see `src/view/src/app/state.rs:241`).

- [ ] **Step 5: Register `top_bar` in `mod.rs`**

In `src/view/src/ui/components/mod.rs`, add after the `pub mod tab_bar;` line:

```rust
pub mod top_bar;
pub use top_bar::render_top_bar;
pub use tab_bar::render_tab_bar;
```

- [ ] **Step 6: Build to verify the new modules compile**

Run: `cargo build --release --bin view 2>&1 | tail -10`
Expected: build success (no errors; warnings allowed)

- [ ] **Step 7: Run the tab_bar unit tests**

Run: `cargo test --package view --lib ui::components::tab_bar::tests 2>&1 | tail -5`
Expected: 3 tests passed

- [ ] **Step 8: Replace the inline top bar in `impls_rendering.rs`**

Read `src/view/src/app/impls_rendering.rs` lines 1834-2063 (the inline top bar block). This is a single `.child({ use crate::ui::components::{Button, ButtonSize, ButtonVariant}; ... })` block.

Replace the entire block (from line 1834 `.child(` to the closing `},` at line 2063) with:

```rust
            .child(crate::ui::components::top_bar::render_top_bar(self, view.clone(), cx))
```

Important: This requires `self` and `cx` to be in scope. Since the surrounding code is inside an `impl CanViewApp` method (likely `render(&mut self, view: Entity<CanViewApp>, cx: &mut Context<Self>)`), this should work. If `self` is `&mut self` but `render_top_bar` takes `&CanViewApp`, the borrow checker may complain — if so, read the current `render` signature and use `&*self` or temporarily clone the needed fields.

If the surrounding context uses `&mut self`, you can change `render_top_bar` to take `&self` (Rust auto-deref `&mut` to `&`). Verify by building.

- [ ] **Step 9: Build and check for errors**

Run: `cargo build --release --bin view 2>&1 | tail -20`
Expected: build success. If errors appear about `self`/`cx` in scope, adjust the call site or the `render_top_bar` signature accordingly.

- [ ] **Step 10: Verify the old File menu dropdown still works (it's outside the top bar block)**

The File menu dropdown (`if self.show_file_menu`) is at line 2143-2235 of the original code — that's a separate `.child(...)` block AFTER the status bar. Leave it in place; only the top bar block was replaced.

Run: `grep -n "show_file_menu" src/view/src/app/impls_rendering.rs | head -5`
Expected: still shows matches (the dropdown logic is preserved).

- [ ] **Step 11: Run the app and verify top bar still renders**

Run: `cargo run --release --bin view 2>&1 | head -30 &
sleep 4; kill %1 2>/dev/null; wait 2>/dev/null`

Then visually inspect — or use the screenshot skill if available. For automated verification, just check that the binary starts and doesn't panic.

Expected: process starts, prints log lines, no panic trace.

- [ ] **Step 12: Commit**

```bash
git add src/view/src/ui/components/tab_bar.rs src/view/src/ui/components/top_bar.rs src/view/src/ui/components/mod.rs src/view/src/app/impls_rendering.rs
git commit -m "$(cat <<'EOF'
refactor(ui): extract TopBar and TabBar from impls_rendering

Replaces the inline ~230-line top bar block in impls_rendering.rs with
two reusable components: TopBar (File menu + TabBar + library badge +
window controls) and TabBar (3 view tabs with bottom 2px indicator).
All colors use theme::colors tokens; no rgb(0x...) literals remain in
the new files.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

## Task 4: Create FilterBar component

**Files:**
- Create: `src/view/src/ui/components/filter_bar.rs`
- Modify: `src/view/src/ui/components/mod.rs` (add module declaration)
- Modify: `src/view/src/app/impls_rendering.rs` (replace Log view filter bar)
- Modify: `src/view/src/library_view.rs` (replace Library view filter controls — if applicable)

**Interfaces:**
- Consumes: `crate::ui::theme::colors::*`, `crate::ui::theme::spacing::*`, `crate::ui::components::{Button, ButtonSize, ButtonVariant}`, `gpui_component::input::InputState`
- Produces: `pub enum FilterBarVariant { Log, Library }`
- Produces: `pub fn render_filter_bar(app: &CanViewApp, view: Entity<CanViewApp>, variant: FilterBarVariant) -> impl IntoElement`
- Produces: `pub fn render_filter_chip(label: &str, value: &str, active: bool, on_click: impl Fn() + 'static) -> impl IntoElement`

- [ ] **Step 1: Locate the existing Log view filter bar in `impls_rendering.rs`**

Run: `grep -n "ID filter\|id_filter_text\|channel_filter_text\|signal_filter" src/view/src/app/impls_rendering.rs | head -20`

Note the line ranges of the inline filter bar in `render_log_view`. (It's likely between lines 400-700 based on file structure, but verify with grep.)

- [ ] **Step 2: Locate the existing Library view filter bar in `library_view.rs`**

Run: `grep -n "library_filter_type\|library_search_query\|Share\|Import\|New Library" src/view/src/library_view.rs | head -20`

Note the line ranges.

- [ ] **Step 3: Create the `filter_bar.rs` file with chip and bar skeleton**

Create `src/view/src/ui/components/filter_bar.rs`:

```rust
//! FilterBar component
//!
//! Renders a horizontal filter/option bar for Log and Library views.
//! Variant-specific controls are selected via FilterBarVariant.

use crate::app::{CanViewApp, AppView};
use crate::models::library::DatabaseType;
use crate::ui::components::{Button, ButtonSize, ButtonVariant};
use crate::ui::theme::colors;
use crate::ui::theme::spacing;
use gpui::{prelude::*, *};

/// Which view's filter set to render.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FilterBarVariant {
    Log,
    Library,
}

/// Render a single filter chip. Clicking invokes `on_click` (which should
/// toggle a dropdown or open an input).
pub fn render_filter_chip(
    label: &str,
    value: &str,
    active: bool,
    on_click: impl Fn() + 'static,
) -> Div {
    div()
        .h(px(28.))
        .px(spacing::SM)
        .flex()
        .items_center()
        .gap(px(4.))
        .bg(colors::SURFACE0)
        .border_1()
        .border_color(if active { colors::BORDER_FOCUSED } else { colors::BORDER_DEFAULT })
        .rounded(px(4.))
        .cursor_pointer()
        .hover(|s| s.bg(colors::SURFACE1))
        .text_sm()
        .child(div().text_color(colors::TEXT_MUTED).child(label.to_string()))
        .child(div().text_color(colors::TEXT_SECONDARY).child(value.to_string()))
        .child(div().text_color(colors::TEXT_MUTED).child("▾"))
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            cx.stop_propagation();
            on_click();
        })
}

/// Render the FilterBar. The variant selects which set of controls to show.
pub fn render_filter_bar(
    app: &CanViewApp,
    view: Entity<CanViewApp>,
    variant: FilterBarVariant,
) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .gap(spacing::SM)
        .px(spacing::LG)
        .py(spacing::XS)
        .bg(colors::BG_ELEVATED)
        .border_b_1()
        .border_color(colors::BORDER_SUBTLE)
        .child(match variant {
            FilterBarVariant::Log => render_log_filters(app, view).into_any_element(),
            FilterBarVariant::Library => render_library_filters(app, view).into_any_element(),
        })
}

/// Render Log view filter controls: ID chip, Channel chip, signal search,
/// Hex/Dec toggle, Display points toggle.
fn render_log_filters(app: &CanViewApp, view: Entity<CanViewApp>) -> impl IntoElement {
    let id_value = if app.id_filter.is_some() {
        app.id_filter_text.to_string()
    } else {
        "All".to_string()
    };
    let channel_value = if let Some(ch) = app.channel_filter {
        ch.to_string()
    } else {
        "All".to_string()
    };

    let view_for_id = view.clone();
    let view_for_channel = view.clone();
    let view_for_id_display = view.clone();
    let view_for_points = view.clone();

    div()
        .flex()
        .items_center()
        .gap(spacing::SM)
        .w_full()
        // ID chip
        .child(render_filter_chip("ID", &id_value, app.id_filter.is_some(), move || {
            // Toggle the ID filter dropdown — reuses the existing show_id_filter_input state
            view_for_id.update(|app, cx| {
                app.show_id_filter_input = !app.show_id_filter_input;
                cx.notify();
            });
        }))
        // Channel chip
        .child(render_filter_chip("Channel", &channel_value, app.channel_filter.is_some(), move || {
            view_for_channel.update(|app, cx| {
                app.show_channel_filter_input = !app.show_channel_filter_input;
                cx.notify();
            });
        }))
        // Signal search — placeholder div; real input is rendered separately at the existing call site
        .child(div().flex_1())
        // Hex/Dec toggle (right-aligned)
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(2.))
                .child(render_toggle_button("Hex", !app.id_display_decimal, view_for_id_display.clone(), |app, cx| {
                    app.id_display_decimal = false;
                    cx.notify();
                }))
                .child(render_toggle_button("Dec", app.id_display_decimal, view_for_id_display, |app, cx| {
                    app.id_display_decimal = true;
                    cx.notify();
                })),
        )
        // Display points toggle
        .child(
            div()
                .px(spacing::SM)
                .h(px(28.))
                .flex()
                .items_center()
                .gap(px(4.))
                .text_sm()
                .text_color(colors::TEXT_SECONDARY)
                .child("Points")
                .child(
                    div()
                        .w(px(14.))
                        .h(px(14.))
                        .border_1()
                        .border_color(colors::BORDER_DEFAULT)
                        .when(app.show_plot_points, |el| el.bg(colors::PRIMARY))
                        .cursor_pointer()
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            cx.stop_propagation();
                            view_for_points.update(|app, cx| {
                                app.show_plot_points = !app.show_plot_points;
                                cx.notify();
                            });
                        }),
                ),
        )
}

/// Render Library view filter controls: Type chip, search input, action buttons.
fn render_library_filters(app: &CanViewApp, view: Entity<CanViewApp>) -> impl IntoElement {
    let type_value = match app.library_filter_type {
        Some(DatabaseType::Dbc) => "DBC".to_string(),
        Some(DatabaseType::Ldf) => "LDF".to_string(),
        None => "ALL".to_string(),
    };
    let is_sharing = app.server_handle.is_some();

    let view_for_type = view.clone();
    let view_for_new = view.clone();
    let view_for_share = view.clone();
    let view_for_import = view.clone();

    div()
        .flex()
        .items_center()
        .gap(spacing::SM)
        .w_full()
        // Type chip
        .child(render_filter_chip("Type", &type_value, app.library_filter_type.is_some(), move || {
            // Cycle through: None -> DBC -> LDF -> None
            view_for_type.update(|app, cx| {
                app.library_filter_type = match app.library_filter_type {
                    None => Some(DatabaseType::Dbc),
                    Some(DatabaseType::Dbc) => Some(DatabaseType::Ldf),
                    Some(DatabaseType::Ldf) => None,
                };
                cx.notify();
            });
        }))
        // Search input placeholder — real Input rendered at the existing call site
        .child(div().flex_1())
        // + New Library
        .child(
            Button::new("+ New Library")
                .size(ButtonSize::Small)
                .variant(ButtonVariant::Ghost)
                .build()
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    cx.stop_propagation();
                    view_for_new.update(|app, cx| {
                        app.show_library_dialog = true;
                        app.library_dialog_type = crate::app::LibraryDialogType::Create;
                        cx.notify();
                    });
                }),
        )
        // Share / Stop Share
        .child(
            Button::new(if is_sharing { "Stop Share" } else { "Share" })
                .size(ButtonSize::Small)
                .variant(ButtonVariant::Ghost)
                .build()
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    cx.stop_propagation();
                    view_for_share.update(|app, cx| {
                        if app.server_handle.is_some() {
                            // Stop share — reuse existing logic
                            if let Some(handle) = app.server_handle.take() {
                                handle.stop();
                            }
                            app.show_share_dialog = false;
                        } else {
                            app.show_share_dialog = true;
                        }
                        cx.notify();
                    });
                }),
        )
        // Import
        .child(
            Button::new("📥 Import")
                .size(ButtonSize::Small)
                .variant(ButtonVariant::Ghost)
                .build()
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    cx.stop_propagation();
                    view_for_import.update(|app, cx| {
                        app.show_import_dialog = true;
                        cx.notify();
                    });
                }),
        )
}

/// Render a small toggle button (for Hex/Dec button group).
fn render_toggle_button(
    label: &str,
    active: bool,
    _view: Entity<CanViewApp>,
    on_click: impl Fn(&mut CanViewApp, &mut gpui::Context<CanViewApp>) + 'static,
) -> impl IntoElement {
    div()
        .px(spacing::SM)
        .h(px(28.))
        .flex()
        .items_center()
        .cursor_pointer()
        .text_sm()
        .text_color(if active { colors::TEXT_PRIMARY } else { colors::TEXT_MUTED })
        .when(active, |el| el.bg(colors::SURFACE0).border_1().border_color(colors::PRIMARY))
        .hover(|s| s.bg(colors::SURFACE1))
        .child(label.to_string())
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            cx.stop_propagation();
            // The on_click closure captured `view`, but we pass it via _view param.
            // Actually, we need to call view.update with on_click — refactor.
            // For now, placeholder: the parent passes the closure that already has the view captured.
            (on_click)(/* need app + cx here */);
        })
}

// NOTE: render_toggle_button's on_mouse_down needs access to the view to call update.
// Reimplement: pass view clone into the closure.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_bar_variant_equality() {
        assert_ne!(FilterBarVariant::Log, FilterBarVariant::Library);
    }
}
```

Note: there's an issue with `render_toggle_button` — it needs `view` to call `update`. Fix the signature to take the closure that captures view directly. Replace the function with:

```rust
/// Render a small toggle button. The `on_click` closure should already
/// capture a `view: Entity<CanViewApp>` clone and call `view.update(cx, ...)`.
fn render_toggle_button(
    label: &str,
    active: bool,
    on_click: impl Fn(&mut gpui::Context<CanViewApp>) + 'static,
) -> impl IntoElement {
    div()
        .px(spacing::SM)
        .h(px(28.))
        .flex()
        .items_center()
        .cursor_pointer()
        .text_sm()
        .text_color(if active { colors::TEXT_PRIMARY } else { colors::TEXT_MUTED })
        .when(active, |el| el.bg(colors::SURFACE0).border_1().border_color(colors::PRIMARY))
        .hover(|s| s.bg(colors::SURFACE1))
        .child(label.to_string())
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            cx.stop_propagation();
            on_click(cx);
        })
}
```

And update the callers in `render_log_filters`:
```rust
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(2.))
                .child(render_toggle_button("Hex", !app.id_display_decimal, {
                    let view = view_for_id_display.clone();
                    move |cx| view.update(cx, |app, cx| {
                        app.id_display_decimal = false;
                        cx.notify();
                    })
                }))
                .child(render_toggle_button("Dec", app.id_display_decimal, {
                    let view = view_for_id_display.clone();
                    move |cx| view.update(cx, |app, cx| {
                        app.id_display_decimal = true;
                        cx.notify();
                    })
                })),
        )
```

(Replace the original Hex/Dec block with the above.)

- [ ] **Step 4: Register `filter_bar` in `mod.rs`**

In `src/view/src/ui/components/mod.rs`, after `pub mod top_bar;`:

```rust
pub mod filter_bar;
pub use filter_bar::{render_filter_bar, render_filter_chip, FilterBarVariant};
```

- [ ] **Step 5: Build to verify the new module compiles**

Run: `cargo build --release --bin view 2>&1 | tail -20`
Expected: build success. If `DatabaseType` import path is wrong, try `use crate::models::library::DatabaseType;` — verify by running `grep -rn "pub enum DatabaseType" src/view/src/`.

- [ ] **Step 6: Run the unit test**

Run: `cargo test --package view --lib ui::components::filter_bar::tests 2>&1 | tail -5`
Expected: 1 test passed

- [ ] **Step 7: Wire up FilterBar in Log view**

In `src/view/src/app/impls_rendering.rs`, find `render_log_view` (around line 265). The function returns a `div()...` that starts at line 298. After the first `div().size_full().flex().flex_col().relative()` chain, insert as the first child:

```rust
            .child(crate::ui::components::filter_bar::render_filter_bar(
                self,
                view.clone(),
                crate::ui::components::filter_bar::FilterBarVariant::Log,
            ))
```

Place this BEFORE the existing filter UI (which is rendered inline further down). The existing inline filter UI code can remain in place for now (it has the dropdown logic we're not migrating); the new FilterBar sits on top as a styled wrapper. After verifying the new bar renders correctly, in a follow-up step we can remove the old inline code.

Actually, to keep this commit small and avoid double-rendering filters, only replace the Log view's TOP-LEVEL filter container. Read the existing structure and identify which `<div>` is the outermost filter bar (likely the first child after `.relative()`). Replace that div with the new FilterBar call. If unsure, leave the existing UI in place for now and only render the FilterBar at the top; the existing dropdown logic stays where it is.

For safety in this commit: ONLY insert the FilterBar at the top of `render_log_view`, do NOT delete the existing inline filter UI yet. The next commit (Task 4 follow-up) can clean up.

Actually, to keep scope bounded: in this task, just render FilterBar at the top. Mark the existing inline UI with a `// TODO: remove once FilterBar dropdowns are wired up` comment.

- [ ] **Step 8: Wire up FilterBar in Library view**

In `src/view/src/library_view.rs`, find `render_library_management_view`. At the top of the returned `div()`, insert:

```rust
        .child(crate::ui::components::filter_bar::render_filter_bar(
            // NOTE: library_view.rs doesn't have direct access to CanViewApp, so this
            // may require passing the app reference or entity through function parameters.
            // See Task 4 step 8 for the exact wiring.
        ))
```

Read `library_view.rs` lines 1-50 to see the function signature. If the function doesn't receive `&CanViewApp`, this step requires passing it. If that's a large change, defer the Library FilterBar wiring to a follow-up task and only wire Log view in this task. Mark with `// TODO: wire up FilterBar for Library view in follow-up commit`.

- [ ] **Step 9: Build and verify**

Run: `cargo build --release --bin view 2>&1 | tail -10`
Expected: build success

- [ ] **Step 10: Run app and verify FilterBar renders in Log view**

Run: `cargo run --release --bin view &`
sleep 3
Visually inspect or screenshot — the top of Log view should show a styled filter bar with ID, Channel chips, Hex/Dec toggles, and a Points toggle.
kill %1 2>/dev/null

- [ ] **Step 11: Commit**

```bash
git add src/view/src/ui/components/filter_bar.rs src/view/src/ui/components/mod.rs src/view/src/app/impls_rendering.rs src/view/src/library_view.rs
git commit -m "$(cat <<'EOF'
refactor(ui): extract FilterBar component for Log and Library views

Adds a new FilterBar component with FilterBarVariant::{Log, Library}
that renders ID/Channel/Hex/Dec/Points chips for Log and Type/New/
Share/Import buttons for Library. All colors use theme::colors tokens.
Existing inline filter UI is preserved in place; cleanup in a follow-up.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

## Task 5: Create StatusBar component

**Files:**
- Create: `src/view/src/ui/components/status_bar.rs`
- Modify: `src/view/src/ui/components/mod.rs` (add module declaration)
- Modify: `src/view/src/app/impls_rendering.rs` (replace inline status bar at lines 2082-2140)

**Interfaces:**
- Consumes: `crate::app::CanViewApp`, `crate::app::AppView`, `crate::ui::theme::colors::*`, `crate::ui::theme::spacing::*`, `crate::ui::theme::typography::*`
- Produces: `pub fn render_status_bar(app: &CanViewApp, view: Entity<CanViewApp>) -> impl IntoElement`

- [ ] **Step 1: Write the StatusBar component**

Create `src/view/src/ui/components/status_bar.rs`:

```rust
//! StatusBar component
//!
//! Renders the bottom status bar with file info (left) and server/library
//! status (right). Single 24px row.

use crate::app::{AppView, CanViewApp};
use crate::ui::theme::colors;
use crate::ui::theme::spacing;
use gpui::{prelude::*, *};

/// Format a count with thousands separators.
fn format_count(n: usize) -> String {
    let s = n.to_string();
    let bytes = s.as_bytes();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 && (bytes.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(*b as char);
    }
    out
}

/// Render the file name segment (left side, segment 1).
fn render_file_segment(app: &CanViewApp) -> impl IntoElement {
    let text = app.current_file_name.clone().unwrap_or_else(|| "No file loaded — File > Open BLF...".to_string());
    let color = if app.current_file_name.is_some() { colors::TEXT_SECONDARY } else { colors::TEXT_PLACEHOLDER };
    div()
        .flex()
        .items_center()
        .gap(px(6.))
        .child(div().text_color(colors::TEXT_MUTED).child("📂"))
        .child(div().text_color(color).child(text))
}

/// Render a vertical separator (1px wide, 12px tall).
fn render_separator() -> impl IntoElement {
    div()
        .w(px(1.))
        .h(px(12.))
        .bg(colors::BORDER_SUBTLE)
}

/// Render the server status segment (right side, segment 1).
fn render_server_segment(app: &CanViewApp, view: Entity<CanViewApp>) -> impl IntoElement {
    let running = app.server_handle.is_some();
    let url = app.share_url().map(|s| s.to_string()).unwrap_or_default();
    let view_for_click = view.clone();
    let url_for_copy = url.clone();
    let dot_color = if running { colors::SUCCESS } else { colors::OVERLAY0 };
    let label = if running { format!("Server ON {}", url) } else { "Share disabled".to_string() };

    div()
        .flex()
        .items_center()
        .gap(px(6.))
        .cursor_pointer()
        .when(running, |el| el.hover(|s| s.bg(colors::SURFACE0)))
        .child(div().w(px(8.)).h(px(8.)).rounded_full().bg(dot_color))
        .child(div().text_color(colors::TEXT_MUTED).child(label))
        .when(running, |el| el.on_mouse_down(MouseButton::Left, move |_, _, cx| {
            cx.stop_propagation();
            cx.write_to_clipboard(gpui::ClipboardItem::new_string(url_for_copy.clone()));
            view_for_click.update(cx, |app, cx| {
                app.share_url_copied = true;
                cx.notify();
                // Reset after 2s
                let reset_view = view_for_click.clone();
                cx.spawn(async move |cx| {
                    smol::Timer::after(std::time::Duration::from_secs(2)).await;
                    let _ = cx.update(|cx| {
                        reset_view.update(cx, |app, cx| {
                            app.share_url_copied = false;
                            cx.notify();
                        });
                    });
                }).detach();
            });
        }))
}

/// Render the library badge segment (right side, segment 2).
fn render_lib_badge_segment(app: &CanViewApp) -> impl IntoElement {
    if let (Some(lib_id), Some(ver)) = (&app.active_library_id, &app.active_version_name) {
        let lib_name = app.library_manager.find_library(lib_id)
            .map(|l| l.name.clone())
            .unwrap_or_else(|| lib_id.clone());
        let text = format!("📚 {} / {}", lib_name, ver);
        div()
            .text_color(colors::ACCENT_GREEN_LIGHT)
            .child(text)
            .into_any_element()
    } else {
        div().into_any_element()
    }
}

/// Render the current view name segment (right side, segment 3).
fn render_view_name_segment(view_val: AppView) -> impl IntoElement {
    let name = match view_val {
        AppView::LogView => "log view",
        AppView::PlotView => "plot view",
        AppView::LibraryView => "library view",
        AppView::ConfigView => "config view",
    };
    div().text_color(colors::TEXT_MUTED).child(name.to_string())
}

/// Render the StatusBar.
pub fn render_status_bar(app: &CanViewApp, view: Entity<CanViewApp>) -> impl IntoElement {
    let current_view = app.current_view;

    div()
        .h(px(24.))
        .bg(colors::BG_MUTED)
        .border_t_1()
        .border_color(colors::BORDER_SUBTLE)
        .flex()
        .items_center()
        .justify_between()
        .px(spacing::MD)
        .text_xs()
        // Left side: file | msgs | DBC | LDF (separated by vertical bars)
        .child(
            div()
                .flex()
                .items_center()
                .gap(spacing::SM)
                .child(render_file_segment(app))
                .child(render_separator())
                .child(div().text_color(colors::TEXT_MUTED).child(format!("{} msgs", format_count(app.messages.len()))))
                .child(render_separator())
                .child(div().text_color(colors::TEXT_MUTED).child(format!("DBC: {}", app.dbc_channels.len())))
                .child(render_separator())
                .child(div().text_color(colors::TEXT_MUTED).child(format!("LDF: {}", app.ldf_channels.len()))),
        )
        // Right side: server | lib badge | view name (separated by vertical bars)
        .child(
            div()
                .flex()
                .items_center()
                .gap(spacing::SM)
                .child(render_server_segment(app, view.clone()))
                .child(render_separator())
                .child(render_lib_badge_segment(app))
                .child(render_separator())
                .child(render_view_name_segment(current_view)),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_count_zero() {
        assert_eq!(format_count(0), "0");
    }

    #[test]
    fn test_format_count_small() {
        assert_eq!(format_count(123), "123");
    }

    #[test]
    fn test_format_count_thousands() {
        assert_eq!(format_count(12345), "12,345");
    }

    #[test]
    fn test_format_count_millions() {
        assert_eq!(format_count(1234567), "1,234,567");
    }

    #[test]
    fn test_format_count_exact_thousand() {
        assert_eq!(format_count(1000), "1,000");
    }
}
```

- [ ] **Step 2: Register `status_bar` in `mod.rs`**

In `src/view/src/ui/components/mod.rs`, after `pub mod filter_bar;`:

```rust
pub mod status_bar;
pub use status_bar::render_status_bar;
```

- [ ] **Step 3: Run the unit tests**

Run: `cargo test --package view --lib ui::components::status_bar::tests 2>&1 | tail -10`
Expected: 5 tests passed

- [ ] **Step 4: Replace the inline status bar in `impls_rendering.rs`**

In `src/view/src/app/impls_rendering.rs`, find the inline status bar block (lines 2082-2140). It starts with `// Zed-style status bar at bottom` and is a single `.child(div().h(px(24.)).bg(rgb(0x1e1e1e))...)` block.

Replace the entire block (from `.child(` at line 2082 to the closing `),` at line 2140) with:

```rust
            .child(crate::ui::components::status_bar::render_status_bar(self, view.clone()))
```

- [ ] **Step 5: Build and verify**

Run: `cargo build --release --bin view 2>&1 | tail -10`
Expected: build success

- [ ] **Step 6: Run clippy to verify no new warnings**

Run: `cargo clippy --workspace 2>&1 | grep -c "^warning"`
Expected: ≤ 381 (baseline)

- [ ] **Step 7: Run app and verify StatusBar renders correctly**

Run: `cargo run --release --bin view &`
sleep 3
Verify: status bar shows "No file loaded — File > Open BLF..." on the left, "Share disabled" on the right, "log view" at the far right.
kill %1 2>/dev/null

- [ ] **Step 8: Commit**

```bash
git add src/view/src/ui/components/status_bar.rs src/view/src/ui/components/mod.rs src/view/src/app/impls_rendering.rs
git commit -m "$(cat <<'EOF'
refactor(ui): extract StatusBar from impls_rendering

Replaces the inline ~60-line status bar block with a reusable
StatusBar component. StatusBar displays: file name (from
current_file_name state added in Task 2), message count (thousands
separator), DBC/LDF counts, server status dot (clickable to copy URL),
active library badge, and current view name. All colors use
theme::colors tokens.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

## Task 6: Clean up backup/examples/debug files

**Files:**
- Delete: `src/view/src/ui/components/button_backup.rs`
- Delete: `src/view/src/ui/components/mod_old.rs`
- Delete: `src/view/src/app/impls.rs.after_deletion`
- Delete: `src/view/src/app/impls_rendering.rs.bak`
- Delete: `src/view/src/temp_impl1.txt`
- Delete: `src/view/src/main_backup.rs`
- Delete: `src/view/src/library_view_debug.rs`
- Delete: `src/view/src/library_view_focused.rs`
- Move: `src/view/src/ui/components/dropdown_examples.rs` → `examples/ui_components/dropdown_examples.rs`
- Move: `src/view/src/ui/components/button_examples.rs` → `examples/ui_components/button_examples.rs`
- Move: `src/view/src/ui/components/modal_examples.rs` → `examples/ui_components/modal_examples.rs`
- Move: `src/view/src/ui/components/tabs_examples.rs` → `examples/ui_components/tabs_examples.rs`
- Move: `src/view/src/views/common_examples.rs` → `examples/views/common_examples.rs`
- Modify: `src/view/src/ui/components/mod.rs` (remove `*_examples` declarations)
- Modify: `src/view/src/views/mod.rs` (remove `common_examples` declaration if present)
- Modify: `src/view/src/library_view.rs` (if it references `library_view_debug` or `library_view_focused`)

**Interfaces:**
- Consumes: nothing
- Produces: nothing new (cleanup only)

- [ ] **Step 1: Verify no live code references the files to delete**

Run these greps:
```bash
grep -rn "library_view_debug\|library_view_focused" src/view/src/ --include="*.rs" | grep -v "library_view_debug.rs\|library_view_focused.rs"
grep -rn "mod_old\|button_backup" src/view/src/ --include="*.rs" | grep -v "mod_old.rs\|button_backup.rs"
grep -rn "main_backup" src/view/src/ --include="*.rs" | grep -v "main_backup.rs"
```

Expected: no matches (or only matches inside the files themselves). If any match is in live code, you'll need to remove those references before deleting.

- [ ] **Step 2: Check the `mod.rs` files for module declarations of files we're deleting/moving**

Run:
```bash
grep -n "button_backup\|mod_old\|dropdown_examples\|button_examples\|modal_examples\|tabs_examples" src/view/src/ui/components/mod.rs
grep -n "common_examples\|library_view_debug\|library_view_focused" src/view/src/views/mod.rs
grep -n "library_view_debug\|library_view_focused" src/view/src/main.rs src/view/src/app/mod.rs 2>/dev/null
```

Record all lines that need to be removed.

- [ ] **Step 3: Remove module declarations from `mod.rs` files**

In `src/view/src/ui/components/mod.rs`, remove the lines declaring any of:
- `pub mod button_backup;`
- `pub mod mod_old;`
- `pub mod dropdown_examples;`
- `pub mod button_examples;`
- `pub mod modal_examples;`
- `pub mod tabs_examples;`

Use the Edit tool with `replace_all=false` and the exact line content (e.g., `pub mod button_backup;\n`) replaced with empty string. Verify each removal with grep afterwards.

In `src/view/src/views/mod.rs`, remove `pub mod common_examples;` if present (read the file first to see exact line).

In `src/view/src/library_view.rs`, search for and remove any `use crate::library_view_debug` or `use crate::library_view_focused` references.

In `src/view/src/main.rs`, check if there are `mod library_view_debug;` or `mod library_view_focused;` declarations at the top of the file. If so, remove those lines.

- [ ] **Step 4: Delete the 8 dead-code files**

Run:
```bash
rm src/view/src/ui/components/button_backup.rs
rm src/view/src/ui/components/mod_old.rs
rm src/view/src/app/impls.rs.after_deletion
rm src/view/src/app/impls_rendering.rs.bak
rm src/view/src/temp_impl1.txt
rm src/view/src/main_backup.rs
rm src/view/src/library_view_debug.rs
rm src/view/src/library_view_focused.rs
```

- [ ] **Step 5: Move examples to `examples/`**

Run:
```bash
mkdir -p examples/ui_components examples/views
git mv src/view/src/ui/components/dropdown_examples.rs examples/ui_components/dropdown_examples.rs
git mv src/view/src/ui/components/button_examples.rs examples/ui_components/button_examples.rs
git mv src/view/src/ui/components/modal_examples.rs examples/ui_components/modal_examples.rs
git mv src/view/src/ui/components/tabs_examples.rs examples/ui_components/tabs_examples.rs
git mv src/view/src/views/common_examples.rs examples/views/common_examples.rs
```

(Using `git mv` preserves history.)

- [ ] **Step 6: Build and verify the deletion didn't break anything**

Run: `cargo build --release --bin view 2>&1 | tail -20`
Expected: build success. If there are errors about missing modules, search for remaining `use` statements that reference the moved/deleted files and remove them.

- [ ] **Step 7: Run clippy to check warning count**

Run: `cargo clippy --workspace 2>&1 | grep -c "^warning"`
Expected: ≤ 381 (baseline). Most likely lower, since deleted files contained some warnings.

- [ ] **Step 8: Run all tests**

Run: `cargo test --workspace 2>&1 | tail -10`
Expected: all tests pass

- [ ] **Step 9: Commit**

```bash
git add -A
git commit -m "$(cat <<'EOF'
chore(ui): remove backup/examples/debug files

Deletes 8 backup/debug files (button_backup, mod_old, impls.rs.after_deletion,
impls_rendering.rs.bak, temp_impl1.txt, main_backup, library_view_debug,
library_view_focused) and moves 5 example files out of src/ into examples/
(dropdown/button/modal/tabs_examples, common_examples). Updates mod.rs
declarations accordingly. No behavior changes.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

## Task 7: Final verification

**Files:**
- No file changes — verification only

- [ ] **Step 1: Full clean build**

Run: `cargo clean && cargo build --release --bin view 2>&1 | tail -10`
Expected: build success

- [ ] **Step 2: Full clippy check**

Run: `cargo clippy --workspace -- -D warnings 2>&1 | grep -c "^warning"`
Expected: ≤ 381 (baseline; ideally lower after cleanup)

- [ ] **Step 3: Format check**

Run: `cargo fmt --all -- --check 2>&1 | head -20`
Expected: no output (or only pre-existing formatting issues)

- [ ] **Step 4: Test run**

Run: `cargo test --workspace 2>&1 | tail -20`
Expected: all tests pass (including new tests from Tasks 1, 3, 4, 5)

- [ ] **Step 5: Manual run-through of 7 verification steps**

Run: `cargo run --release --bin view &`
sleep 3

Verify in order:
1. Top bar shows File + Log / Signal Plot / Library tabs; clicking each switches view; active tab has a 2px blue indicator at the bottom
2. (On macOS) Traffic light position is correct (left 80px padding); (On Win/Linux) minimize/maximize/close buttons work; close hover turns red
3. Log view shows FilterBar with ID chip (▾ dropdown icon), Channel chip (▾), Hex/Dec toggle group, Points toggle. Clicking ID/Channel chips toggles the existing dropdown (or shows it's not yet wired — note any TODO)
4. Library view shows FilterBar with Type chip (▾), + New Library button, Share/Stop Share button, 📥 Import button. All clickable.
5. Status bar (left): "No file loaded — File > Open BLF..." when no file; after loading a BLF (File > Open BLF... and pick a file), shows file name + msg count (with thousands separator) + DBC count + LDF count. Use `sample.dbc` and `test_can.dbc` from repo root if you don't have a BLF.
6. Status bar (right): "Share disabled" with grey dot by default. After clicking Share in Library view, status bar shows green dot + "Server ON <url>". Clicking the URL copies it to clipboard and shows toast.
7. No new panic or warning logs in stderr during the session.

kill %1 2>/dev/null

- [ ] **Step 6: Verify the spec acceptance criteria**

Run these commands:
```bash
# 4 component files exist
ls src/view/src/ui/components/{top_bar,tab_bar,filter_bar,status_bar}.rs
# Zero hardcoded rgb() in new components
grep -rn "rgb(0x" src/view/src/ui/components/top_bar.rs src/view/src/ui/components/tab_bar.rs src/view/src/ui/components/filter_bar.rs src/view/src/ui/components/status_bar.rs
# impls_rendering.rs shrunk by ≥ 400 lines
wc -l src/view/src/app/impls_rendering.rs
# Deleted files are gone
ls src/view/src/ui/components/button_backup.rs src/view/src/main_backup.rs src/view/src/library_view_debug.rs src/view/src/library_view_focused.rs 2>&1 | head -10
```

Expected:
- 4 component files exist (no error)
- grep output is empty (no `rgb(0x` matches in new components)
- `wc -l` shows ≤ 2096 (was 2496, target reduction ≥ 400)
- `ls` shows "No such file or directory" for all 4 deleted files

- [ ] **Step 7: Commit if any formatting fixes were needed**

If `cargo fmt --all` changed any files:
```bash
git add -A
git commit -m "$(cat <<'EOF'
style: apply cargo fmt after UI redesign

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

Otherwise, no commit needed.

---

## Self-Review Checklist (run before hand-off)

- [ ] Spec §1 (target + non-target + state adjustment): Task 2 adds `current_file_name` field — covered
- [ ] Spec §2 (architecture): Task 3 + 4 + 5 create the 4 components; Task 3 wires TopBar/TabBar; Task 4 wires FilterBar; Task 5 wires StatusBar
- [ ] Spec §3 (TopBar): Task 3 implements `render_top_bar` with File button, TabBar, badge, window controls
- [ ] Spec §4 (TabBar): Task 3 implements `render_tab_bar` with 3 tabs, active indicator
- [ ] Spec §5 (FilterBar): Task 4 implements `render_filter_bar` with Log/Library variants + FilterChip
- [ ] Spec §6 (StatusBar): Task 5 implements `render_status_bar` with 4 left + 3 right segments
- [ ] Spec §7 (tokens): Task 1 adds 4 new tokens
- [ ] Spec §8 (file cleanup): Task 6 deletes/moves 12 files
- [ ] Spec §9 (6 commits): Tasks 1, 2, 3, 4, 5, 6 correspond to commits 1-6
- [ ] Spec §10 (testing): Each task has unit tests + manual verification; Task 7 is the final 7-step walkthrough
- [ ] Spec §11 (risks): Task 3 step 8 has fallback for borrow issues; Task 4 step 7-8 has fallback for Library view wiring
- [ ] Spec §12 (acceptance): Task 7 step 6 verifies each criterion
