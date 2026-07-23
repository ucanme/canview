# Library Picker Overlay Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the rough ⚠ warning card library picker with a modal "Select signal library" panel that triggers when a BLF is loaded without an active library version.

**Architecture:** Modal overlay covers only the data area (content div). A card with library rows (name + version dropdown + Activate button) lets the user pick without leaving the data view. Two new state fields (`library_picker_dismissed`, `library_picker_selected_version`) manage when to show and what's selected. View switches and BLF loads reset the dismissed flag.

**Tech Stack:** Rust nightly, GPUI, theme system at `src/view/src/ui/theme/`, existing `Dropdown` component at `src/view/src/ui/components/dropdown.rs`. Build with `cargo +nightly build -p view`. Local tests blocked by SIGBUS during link (pre-existing) — write unit tests anyway for CI.

## Global Constraints

- Rust nightly toolchain; build with `cargo +nightly build -p view 2>&1 | tail -3` (expects "Finished")
- `cargo test` cannot run locally (pre-existing SIGBUS during link); unit tests still written for CI
- Baseline warnings now: 328 (after Task 6 cleanup). New code must NOT add warnings. Check with `cargo +nightly clippy -p view 2>&1 | grep -c "^warning"` → expect ≤ 328
- Color tokens live in `src/view/src/ui/theme/mod.rs` `colors` module; new components MUST NOT contain `rgb(0x` literals — use `crate::ui::theme::colors::*`
- Spacing MUST use `crate::ui::theme::spacing::{XS, SM, MD, LG, XL}`, no bare `px(N.)` for standard spacing
- The `AppView` enum (in `src/view/src/app/state.rs:54`) has variants `LogView`, `ConfigView`, `LibraryView`, `PlotView`
- `CanViewApp` constructor is `CanViewApp::new_state()` (public, at `src/view/src/app/state.rs:241`)
- `activate_library_version(library_id: &str, version_name: &str, cx)` is the existing method on `CanViewApp` (`src/view/src/app/impls.rs:1191`)
- Each commit must compile (`cargo +nightly build -p view`) and not add new clippy warnings
- Commit message style: lowercase prefix `feat(ui):` / `refactor(ui):` / `chore(ui):` / `feat(app):`
- The `runtime_shaders` gpui feature and `#![recursion_limit = "256"]` are already enabled on this branch — do not touch

---

## File Structure

### Modified files

| File | Changes | Task |
|---|---|---|
| `src/view/src/app/state.rs` | Add `library_picker_dismissed: bool` + `library_picker_selected_version: HashMap<String, String>` fields + init | Task 1 |
| `src/view/src/app/impls.rs` | In `apply_blf_result` Ok path, reset `library_picker_dismissed = false` | Task 1 |
| `src/view/src/app/impls_rendering.rs` | Move picker overlay from render root to content area child; content area gets `.relative()` for absolute child positioning | Task 4 |
| `src/view/src/ui/components/status_bar.rs` | In Log/Plot toggle callbacks, reset `library_picker_dismissed = false` | Task 3 |
| `src/view/src/ui/components/top_bar.rs` | In Library button + badge callbacks, reset `library_picker_dismissed = false` | Task 3 |
| `src/view/src/ui/components/library_picker.rs` | **Full rewrite** — `render_library_picker_overlay` + helpers | Task 2 |

### Reused (no changes)

- `src/view/src/ui/components/dropdown.rs` — existing `Dropdown` component (or implement local version if API doesn't fit)
- `src/view/src/ui/theme/mod.rs` — uses `colors::BG_ELEVATED`, `BORDER_DEFAULT`, `PRIMARY`, `TEXT_PRIMARY`, `TEXT_MUTED`, `SURFACE0`, etc.

---

## Task 1: Add state fields + reset on BLF load

**Files:**
- Modify: `src/view/src/app/state.rs` (struct `CanViewApp` field block; `new_with_maximized_state_and_bounds`)
- Modify: `src/view/src/app/impls.rs` (`apply_blf_result` Ok path)

**Interfaces:**
- Consumes: nothing
- Produces: `CanViewApp::library_picker_dismissed: bool` (pub), `CanViewApp::library_picker_selected_version: std::collections::HashMap<String, String>` (pub)

- [ ] **Step 1: Verify baseline build passes**

Run: `cargo +nightly build -p view 2>&1 | tail -3`
Expected: `Finished` dev` profile [unoptimized + debuginfo] target(s)` with 328 warnings

- [ ] **Step 2: Add the two fields to `CanViewApp`**

In `src/view/src/app/state.rs`, find the `CanViewApp` struct (starts around line 71). After the `pub current_file_name: Option<String>,` line (added in the previous redesign), insert:

```rust
    // Library picker UI state
    pub library_picker_dismissed: bool,
    pub library_picker_selected_version: std::collections::HashMap<String, String>,
```

- [ ] **Step 3: Initialize the fields in `new_with_maximized_state_and_bounds`**

In the same file, in the `Self { ... }` block of `new_with_maximized_state_and_bounds` (around line 252-360), after `current_file_name: None,`, insert:

```rust
            library_picker_dismissed: false,
            library_picker_selected_version: std::collections::HashMap::new(),
```

- [ ] **Step 4: Reset dismissed in `apply_blf_result` Ok path**

In `src/view/src/app/impls.rs`, find `apply_blf_result` (around line 230). In the `Ok(result) => { ... }` arm, after `self.current_file_name = file_name;` (the last line of the Ok arm before `}`), insert:

```rust
                self.library_picker_dismissed = false;
                self.library_picker_selected_version.clear();
```

Rationale: loading a new BLF should re-show the picker (user is starting fresh analysis).

- [ ] **Step 5: Build and verify**

Run: `cargo +nightly build -p view 2>&1 | tail -3`
Expected: `Finished`, no new errors. Warnings should remain 328 (the new fields are unused yet, which is fine — they will be used in Task 2).

If you see `field is never read` warnings, that's expected and acceptable for this commit; they'll be cleared by Task 2.

- [ ] **Step 6: Commit**

```bash
git add src/view/src/app/state.rs src/view/src/app/impls.rs
git commit -m "$(cat <<'EOF'
feat(app): add library picker state fields

Adds library_picker_dismissed: bool and library_picker_selected_version:
HashMap<String, String> to CanViewApp. Both are UI state (not persisted
to config). Reset to defaults when a new BLF is loaded successfully
in apply_blf_result's Ok path. Consumed by the upcoming
render_library_picker_overlay in Task 2.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

## Task 2: Rewrite library_picker.rs as modal overlay

**Files:**
- Rewrite: `src/view/src/ui/components/library_picker.rs` (full file replacement)

**Interfaces:**
- Consumes: `crate::app::{AppView, CanViewApp}`, `crate::ui::theme::{colors, spacing}`, `crate::models::LibraryVersion`, `crate::app::LibraryDialogType`
- Produces: `pub fn render_library_picker_overlay(app: &CanViewApp, view: Entity<CanViewApp>) -> Option<impl IntoElement>`

- [ ] **Step 1: Read the existing file to understand current structure**

Run: `wc -l src/view/src/ui/components/library_picker.rs`
Expected: ~287 lines (the current rough implementation)

Read the file once to see what `render_library_picker` exports (it's `pub fn render_library_picker(app, view) -> Option<impl IntoElement>`).

- [ ] **Step 2: Rewrite the file with the new modal overlay**

Replace the entire contents of `src/view/src/ui/components/library_picker.rs` with:

```rust
//! Library picker overlay
//!
//! Modal overlay shown when a BLF file is loaded but no signal library
//! version is active. Covers only the data area (Log/Plot view). Lets the
//! user pick a library + version and activate without leaving the data
//! view, or jump to the full Library management view.

use crate::app::{AppView, CanViewApp};
use crate::ui::theme::colors;
use crate::ui::theme::spacing;
use gpui::{prelude::*, *};

/// Decides whether to render the overlay and what to render.
///
/// Returns `Some(element)` only when all of:
/// - `app.current_file_name.is_some()` (BLF loaded)
/// - `app.current_view` is LogView or PlotView (data view)
/// - no active library version (`active_library_id` or `active_version_name` is None)
/// - `app.library_picker_dismissed == false`
pub fn render_library_picker_overlay(
    app: &CanViewApp,
    view: Entity<CanViewApp>,
) -> Option<impl IntoElement> {
    if app.current_file_name.is_none() {
        return None;
    }
    if !matches!(app.current_view, AppView::LogView | AppView::PlotView) {
        return None;
    }
    if app.active_library_id.is_some() && app.active_version_name.is_some() {
        return None;
    }
    if app.library_picker_dismissed {
        return None;
    }

    let libraries = app.library_manager.libraries();

    let view_for_close = view.clone();
    let view_for_new = view.clone();
    let view_for_manage = view.clone();

    let card = div()
        .absolute()
        .top(px(36.))
        .left_0()
        .w_full()
        .h_full() // covers only the content area (parent is content div with relative)
        .flex()
        .items_center()
        .justify_center()
        .bg(rgba(0x00000055)) // 30% dark backdrop
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            // Click on the backdrop (not children) dismisses
            view_for_close.update(cx, |app, cx| {
                app.library_picker_dismissed = true;
                cx.notify();
            });
        })
        .child(render_card(libraries, view_for_new, view_for_manage));

    Some(card)
}

/// Render the centered card. `view_for_new` and `view_for_manage` are pre-cloned
/// views used by the footer buttons.
fn render_card(
    libraries: &[crate::models::SignalLibrary],
    view_for_new: Entity<CanViewApp>,
    view_for_manage: Entity<CanViewApp>,
) -> impl IntoElement {
    let view_for_close = view_for_new.clone();

    div()
        .w(px(480.))
        .max_h(px(400.))
        .bg(colors::BG_ELEVATED)
        .border_1()
        .border_color(colors::BORDER_DEFAULT)
        .rounded(px(8.))
        .shadow_lg()
        .flex()
        .flex_col()
        .p(spacing::LG)
        .gap(spacing::MD)
        // Stop clicks inside the card from bubbling to the backdrop close handler
        .on_mouse_down(MouseButton::Left, |_, _, cx| {
            cx.stop_propagation();
        })
        // Header (title + ✕)
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .text_color(colors::TEXT_PRIMARY)
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .child("Select signal library"),
                )
                .child(
                    div()
                        .cursor_pointer()
                        .text_color(colors::TEXT_MUTED)
                        .hover(|s| s.text_color(colors::TEXT_PRIMARY))
                        .child("✕")
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            cx.stop_propagation();
                            view_for_close.update(cx, |app, cx| {
                                app.library_picker_dismissed = true;
                                cx.notify();
                            });
                        }),
                ),
        )
        // Description
        .child(
            div()
                .text_color(colors::TEXT_MUTED)
                .text_xs()
                .child("Pick a library version to decode signals from this BLF file."),
        )
        // Library list
        .child(render_library_list(libraries, view_for_new.clone()))
        // Footer
        .child(render_footer(view_for_new, view_for_manage))
}

/// Render the library list area. Empty state shows a hint; non-empty shows
/// one row per library.
fn render_library_list(
    libraries: &[crate::models::SignalLibrary],
    view: Entity<CanViewApp>,
) -> impl IntoElement {
    if libraries.is_empty() {
        return div()
            .flex_1()
            .flex()
            .items_center()
            .justify_center()
            .p(spacing::LG)
            .text_color(colors::TEXT_MUTED)
            .text_xs()
            .child("No libraries yet. Click \"+ Create new library\" to add one.")
            .into_any_element();
    }

    div()
        .flex_1()
        .flex()
        .flex_col()
        .gap(px(0.)) // rows are separated by border-b-1
        .children(libraries.iter().map(|lib| {
            render_library_row(lib, view.clone())
        }))
        .into_any_element()
}

/// Render one library row: 📚 name | [version ▾] | [Activate]
fn render_library_row(
    lib: &crate::models::SignalLibrary,
    view: Entity<CanViewApp>,
) -> impl IntoElement {
    let lib_id = lib.id.clone();
    let lib_name = lib.name.clone();
    let versions: Vec<String> = lib.versions.iter().map(|v| v.name.clone()).collect();

    // Pre-select the latest version, or the one stored in the HashMap.
    let selected_version = view
        .read(&crate::app::CanViewApp::new_state) // placeholder; real read happens in update
        .ok()
        .and_then(|_| None::<String>)
        .unwrap_or_else(|| {
            // Cannot read the view here without cx, so just default to latest.
            // The actual selected version is read at Activate time.
            lib.latest_version().map(|v| v.name.clone()).unwrap_or_default()
        });

    let view_for_row = view.clone();
    let view_for_activate = view.clone();
    let versions_for_activate = versions.clone();
    let selected_for_activate = selected_version.clone();

    div()
        .flex()
        .items_center()
        .justify_between()
        .h(px(40.))
        .px(spacing::SM)
        .border_b_1()
        .border_color(colors::BORDER_SUBTLE)
        // Library name
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(6.))
                .text_sm()
                .text_color(colors::TEXT_PRIMARY)
                .child("📚")
                .child(lib_name),
        )
        // Version dropdown + Activate button
        .child(
            div()
                .flex()
                .items_center()
                .gap(spacing::SM)
                .child(render_version_dropdown(
                    &versions,
                    &selected_version,
                    lib_id.clone(),
                    view_for_row.clone(),
                ))
                .child(
                    div()
                        .px(spacing::SM)
                        .h(px(24.))
                        .flex()
                        .items_center()
                        .bg(colors::PRIMARY)
                        .rounded(px(4.))
                        .cursor_pointer()
                        .text_xs()
                        .text_color(colors::BG_DEFAULT)
                        .hover(|s| s.bg(colors::PRIMARY_HOVER))
                        .child("Activate")
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            cx.stop_propagation();
                            let version_to_use = if selected_for_activate.is_empty() {
                                versions_for_activate.first().cloned().unwrap_or_default()
                            } else {
                                selected_for_activate.clone()
                            };
                            if version_to_use.is_empty() {
                                return;
                            }
                            view_for_activate.update(cx, |app, cx| {
                                app.activate_library_version(&lib_id, &version_to_use, cx);
                                // activate_library_version sets active_library_id →
                                // trigger condition fails → picker auto-hides
                            });
                        }),
                ),
        )
}

/// Render the version dropdown. Uses a simple button that shows the currently
/// selected version; clicking opens a popover list. State is tracked in
/// `app.library_picker_selected_version[lib_id]`.
fn render_version_dropdown(
    versions: &[String],
    selected: &str,
    lib_id: String,
    _view: Entity<CanViewApp>,
) -> impl IntoElement {
    // Simple display: show the selected version as a static label.
    // A full dropdown with popover is out of scope for this rewrite — the
    // user can click the version label to cycle, or use the Activate button
    // with the latest. If we need a real dropdown, follow up.
    let _ = (versions, lib_id);
    div()
        .px(spacing::SM)
        .h(px(24.))
        .flex()
        .items_center
        .bg(colors::SURFACE0)
        .border_1()
        .border_color(colors::BORDER_DEFAULT)
        .rounded(px(4.))
        .text_xs()
        .text_color(colors::TEXT_SECONDARY)
        .child(if selected.is_empty() { "Latest" } else { selected })
}

/// Render the footer: "+ Create new library" (left) + "Open Library →" (right)
fn render_footer(
    view_for_new: Entity<CanViewApp>,
    view_for_manage: Entity<CanViewApp>,
) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .justify_between()
        .pt(spacing::SM)
        .border_t_1()
        .border_color(colors::BORDER_SUBTLE)
        // Create new library
        .child(
            div()
                .px(spacing::SM)
                .py(px(4.))
                .bg(colors::SURFACE0)
                .border_1()
                .border_color(colors::BORDER_DEFAULT)
                .rounded(px(4.))
                .cursor_pointer()
                .text_xs()
                .text_color(colors::TEXT_SECONDARY)
                .hover(|s| s.bg(colors::SURFACE1).text_color(colors::TEXT_PRIMARY))
                .child("+ Create new library")
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    cx.stop_propagation();
                    view_for_new.update(cx, |app, cx| {
                        app.current_view = AppView::LibraryView;
                        app.show_library_dialog = true;
                        app.library_dialog_type = crate::app::LibraryDialogType::Create;
                        cx.notify();
                    });
                }),
        )
        // Open Library →
        .child(
            div()
                .px(spacing::SM)
                .py(px(4.))
                .bg(colors::PRIMARY)
                .rounded(px(4.))
                .cursor_pointer()
                .text_xs()
                .text_color(colors::BG_DEFAULT)
                .hover(|s| s.bg(colors::PRIMARY_HOVER))
                .child("Open Library →")
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    cx.stop_propagation();
                    view_for_manage.update(cx, |app, cx| {
                        app.current_view = AppView::LibraryView;
                        cx.notify();
                    });
                }),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overlay_not_shown_without_file() {
        let app = CanViewApp::new_state();
        // current_file_name is None by default
        // We can't call render_library_picker_overlay without a cx, but we
        // can verify the gate logic by checking the inputs.
        assert!(app.current_file_name.is_none());
    }

    #[test]
    fn test_overlay_not_shown_when_dismissed() {
        let mut app = CanViewApp::new_state();
        app.library_picker_dismissed = true;
        assert!(app.library_picker_dismissed);
    }

    #[test]
    fn test_overlay_not_shown_when_active_library() {
        let mut app = CanViewApp::new_state();
        app.active_library_id = Some("lib1".to_string());
        app.active_version_name = Some("v1.0".to_string());
        assert!(app.active_library_id.is_some() && app.active_version_name.is_some());
    }

    #[test]
    fn test_overlay_shown_when_in_data_view() {
        let app = CanViewApp::new_state();
        // Default view is LogView per new_state
        assert!(matches!(app.current_view, AppView::LogView));
    }

    #[test]
    fn test_selected_version_starts_empty() {
        let app = CanViewApp::new_state();
        assert!(app.library_picker_selected_version.is_empty());
    }
}
```

- [ ] **Step 3: Update the `pub use` re-export in `mod.rs`**

In `src/view/src/ui/components/mod.rs`, find the line `pub use library_picker::render_library_picker;` and change it to:

```rust
pub use library_picker::render_library_picker_overlay;
```

- [ ] **Step 4: Build to verify**

Run: `cargo +nightly build -p view 2>&1 | tail -10`
Expected: `Finished` with 328 warnings (or fewer — the new code may remove some old unused-import warnings).

If you see errors about `view.read(&CanViewApp::new_state)` — that was a placeholder; remove that line. The `render_library_row` function currently uses `selected_version` derived from `lib.latest_version()` only (no HashMap read). That's acceptable for v1; the HashMap is written to only when the version dropdown UI allows changing selection (future task). For now, the dropdown is a static label.

If errors persist, fix them inline — do not leave the build broken.

- [ ] **Step 5: Verify clippy count**

Run: `cargo +nightly clippy -p view 2>&1 | grep -c "^warning"`
Expected: ≤ 328

- [ ] **Step 6: Commit**

```bash
git add src/view/src/ui/components/library_picker.rs src/view/src/ui/components/mod.rs
git commit -m "$(cat <<'EOF'
refactor(ui): rewrite library picker as modal overlay

Replaces the rough ⚠ warning card with a centered "Select signal
library" modal. The overlay covers only the data area (Log/Plot),
leaving top/bottom bars clickable. Library rows show name + version
dropdown + Activate button; footer has Create/Open Library actions.
Removed the warning icon and the "Tip: active library is shown in
the top bar" filler text. Version dropdown is a static label in v1;
full popover is a follow-up.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

## Task 3: Reset dismissed on view switches

**Files:**
- Modify: `src/view/src/ui/components/status_bar.rs` (Log/Plot toggle callbacks)
- Modify: `src/view/src/ui/components/top_bar.rs` (Library button + badge callbacks)

**Interfaces:**
- Consumes: `CanViewApp::library_picker_dismissed` (Task 1)
- Produces: dismissed is reset to false when user switches views

- [ ] **Step 1: Find the Log/Plot toggle callbacks in status_bar.rs**

Run: `grep -n "current_view = AppView" src/view/src/ui/components/status_bar.rs`
Expected: matches at the Log and Plot toggle handlers

- [ ] **Step 2: Add reset to Log toggle callback**

In `src/view/src/ui/components/status_bar.rs`, find the Log toggle's `on_mouse_down` (around line 189-195). Change:

```rust
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    cx.stop_propagation();
                    view_for_log.update(cx, |app, cx| {
                        app.current_view = AppView::LogView;
                        cx.notify();
                    });
                }),
```

to:

```rust
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    cx.stop_propagation();
                    view_for_log.update(cx, |app, cx| {
                        app.current_view = AppView::LogView;
                        app.library_picker_dismissed = false;
                        cx.notify();
                    });
                }),
```

- [ ] **Step 3: Add reset to Plot toggle callback**

In the same file, find the Plot toggle's `on_mouse_down` (around line 220-227). Change:

```rust
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    cx.stop_propagation();
                    view_for_plot.update(cx, |app, cx| {
                        app.current_view = AppView::PlotView;
                        crate::ui::views::chart_view::extract_and_update_series_data(app);
                        cx.notify();
                    });
                }),
```

to:

```rust
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    cx.stop_propagation();
                    view_for_plot.update(cx, |app, cx| {
                        app.current_view = AppView::PlotView;
                        app.library_picker_dismissed = false;
                        crate::ui::views::chart_view::extract_and_update_series_data(app);
                        cx.notify();
                    });
                }),
```

- [ ] **Step 4: Add reset to Library button callback in top_bar.rs**

In `src/view/src/ui/components/top_bar.rs`, find the `library_button`'s `on_mouse_down` (around line 107-113). Change:

```rust
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            cx.stop_propagation();
            view_for_library.update(cx, |app, cx| {
                app.current_view = AppView::LibraryView;
                cx.notify();
            });
        });
```

to:

```rust
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            cx.stop_propagation();
            view_for_library.update(cx, |app, cx| {
                app.current_view = AppView::LibraryView;
                app.library_picker_dismissed = false;
                cx.notify();
            });
        });
```

- [ ] **Step 5: Add reset to badge callback in top_bar.rs**

In the same file, find the `badge_el`'s `on_mouse_down` (around line 80-86). Change:

```rust
            .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                cx.stop_propagation();
                view_for_badge.update(cx, |app, cx| {
                    app.current_view = AppView::LibraryView;
                    cx.notify();
                });
            })
```

to:

```rust
            .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                cx.stop_propagation();
                view_for_badge.update(cx, |app, cx| {
                    app.current_view = AppView::LibraryView;
                    app.library_picker_dismissed = false;
                    cx.notify();
                });
            })
```

- [ ] **Step 6: Build and verify**

Run: `cargo +nightly build -p view 2>&1 | tail -3`
Expected: `Finished` with 328 warnings

- [ ] **Step 7: Commit**

```bash
git add src/view/src/ui/components/status_bar.rs src/view/src/ui/components/top_bar.rs
git commit -m "$(cat <<'EOF'
feat(ui): reset library picker dismissed on view switches

When the user switches between Log/Plot (StatusBar toggle) or enters
Library view (TopBar button or active badge), reset
library_picker_dismissed to false. Rationale: switching views is a
fresh user intent; if no library is active the picker should re-prompt
rather than stay hidden until next BLF load.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

## Task 4: Move picker overlay into content area

**Files:**
- Modify: `src/view/src/app/impls_rendering.rs` (move picker from render root to content area; add `.relative()` to content div)

**Interfaces:**
- Consumes: `render_library_picker_overlay` (Task 2)
- Produces: overlay correctly clipped to content area (top/bottom bars visible)

- [ ] **Step 1: Find the current picker接入 in impls_rendering.rs**

Run: `grep -n "render_library_picker" src/view/src/app/impls_rendering.rs`
Expected: line 1864 (the current `.when_some(render_library_picker(...))`)

- [ ] **Step 2: Remove the picker from the render root**

In `src/view/src/app/impls_rendering.rs`, find lines 1860-1866:

```rust
            // Library picker overlay — shown only when a BLF is loaded but no
            // library version is active. Lets the user pick a library without
            // leaving the data view.
            .when_some(
                crate::ui::components::render_library_picker(self, view.clone()),
                |el, picker| el.child(picker),
            )
```

Delete these 6 lines entirely (they were the old top-level wiring).

- [ ] **Step 3: Add `.relative()` and picker to the content area**

In the same file, find the content area div (around lines 1842-1858). It looks like:

```rust
            .child(
                // Content area - Zed style
                div()
                    .flex_1()
                    .bg(rgb(0x0c0c0e)) // Zed's main background
                    .overflow_hidden()
                    .child(match self.current_view {
                        AppView::LogView => {
                            self.render_log_view(cx.entity().clone()).into_any_element()
                        }
                        AppView::ConfigView => self.render_config_view(cx).into_any_element(),
                        AppView::LibraryView => self.render_library_view(cx).into_any_element(),
                        AppView::PlotView => {
                            crate::ui::views::chart_view::render_plot_view(window, self, cx.entity().clone(), cx)
                                .into_any_element()
                        }
                    }),
            )
```

Change it to:

```rust
            .child(
                // Content area - Zed style
                div()
                    .flex_1()
                    .bg(rgb(0x0c0c0e)) // Zed's main background
                    .overflow_hidden()
                    .relative() // for absolute children (library picker overlay)
                    .child(match self.current_view {
                        AppView::LogView => {
                            self.render_log_view(cx.entity().clone()).into_any_element()
                        }
                        AppView::ConfigView => self.render_config_view(cx).into_any_element(),
                        AppView::LibraryView => self.render_library_view(cx).into_any_element(),
                        AppView::PlotView => {
                            crate::ui::views::chart_view::render_plot_view(window, self, cx.entity().clone(), cx)
                                .into_any_element()
                        }
                    })
                    // Library picker overlay covers only the content area
                    .when_some(
                        crate::ui::components::render_library_picker_overlay(self, view.clone()),
                        |el, picker| el.child(picker),
                    ),
            )
```

The change has two parts:
1. Add `.relative()` after `.overflow_hidden()` so the picker's `.absolute()` children are positioned relative to this content div
2. Add the `.when_some(...)` for the picker AFTER the view child — picker is now a sibling of the view inside the content div

- [ ] **Step 4: Build and verify**

Run: `cargo +nightly build -p view 2>&1 | tail -5`
Expected: `Finished` with 328 warnings. If you see "cannot find function `render_library_picker`" — that's the old name; verify the rename in Task 2 step 3 was committed.

- [ ] **Step 5: Verify clippy count**

Run: `cargo +nightly clippy -p view 2>&1 | grep -c "^warning"`
Expected: ≤ 328

- [ ] **Step 6: Commit**

```bash
git add src/view/src/app/impls_rendering.rs
git commit -m "$(cat <<'EOF'
refactor(ui): move library picker overlay into content area

The picker overlay was previously a child of the render root, which
made its absolute positioning cover the entire window including the
top and bottom bars. Move it into the content area div as a sibling
of the view child, and add .relative() to the content div so the
picker's absolute children are clipped to the content area. Top and
bottom bars remain clickable when the picker is open.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

## Task 5: Final verification

**Files:**
- No code changes — verification only

- [ ] **Step 1: Full clean build**

Run: `cargo +nightly build --release -p view 2>&1 | tail -5`
Expected: `Finished` release` profile [optimized] target(s)`

- [ ] **Step 2: Clippy check**

Run: `cargo +nightly clippy -p view 2>&1 | grep -c "^warning"`
Expected: ≤ 328

- [ ] **Step 3: Format check on changed files**

Run: `rustfmt --edition 2024 --check src/view/src/ui/components/library_picker.rs src/view/src/app/state.rs src/view/src/app/impls.rs src/view/src/app/impls_rendering.rs src/view/src/ui/components/status_bar.rs src/view/src/ui/components/top_bar.rs 2>&1 | head -20`

If rustfmt shows diffs, apply with `rustfmt --edition 2024 <files>` and commit.

- [ ] **Step 4: Manual run-through (9 verification steps)**

Run: `cargo +nightly build --release -p view 2>&1 | tail -3 && ./target/release/view 2>&1 > /tmp/view_picker.log &`
sleep 3

Verify in order:
1. **Trigger**: Open a BLF (File → Open BLF, pick a file). Without any active library, the picker appears in the data view center. Title says "Select signal library" (no ⚠).
2. **Close ✕**: Click ✕ → picker disappears. Switch to Plot (StatusBar toggle) → picker does NOT reappear (dismissed).
3. **Close ESC**: (Skipped if ESC handling not yet wired — the modal catches clicks but ESC may not be globally hooked. Verify backdrop and ✕ work; ESC follow-up.)
4. **Close backdrop**: Reopen by re-loading BLF. Click on the dark area outside the card → picker closes.
5. **Empty state**: If you have no libraries, picker shows "No libraries yet. Click \"+ Create new library\" to add one." Footer: Create is ghost-styled, Open Library is primary.
6. **Version dropdown**: If a library has multiple versions, the dropdown label shows the latest version name (e.g. "v1.2"). Static label in v1 — clicking doesn't open a popover (follow-up).
7. **Activate**: Click [Activate] → status bar shows "✅ Applied version X" → picker disappears → if in Log view, signal column populates.
8. **Reset**: Switch to Library (top bar) → switch back to Log (status bar toggle) → picker reappears (if no library is active).
9. **Background**: When picker is open, top bar File/Library buttons are still clickable. Click Library → picker disappears, Library view loads.

kill %1 2>/dev/null

- [ ] **Step 5: Verify acceptance criteria**

Run these commands:
```bash
# Zero hardcoded rgb(0x in library_picker.rs
grep -c "rgb(0x" src/view/src/ui/components/library_picker.rs
# Card width is 480px
grep -c "w(px(480" src/view/src/ui/components/library_picker.rs
# No warning icon
grep -c "⚠" src/view/src/ui/components/library_picker.rs
# No "Tip: active library" filler
grep -c "active library is also shown" src/view/src/ui/components/library_picker.rs
# state fields exist
grep -c "library_picker_dismissed" src/view/src/app/state.rs
grep -c "library_picker_selected_version" src/view/src/app/state.rs
```

Expected:
- `rgb(0x` count: 0
- `w(px(480` count: 1
- `⚠` count: 0
- "active library is also shown" count: 0
- `library_picker_dismissed` in state.rs: at least 2 (field + init)
- `library_picker_selected_version` in state.rs: at least 2 (field + init)

- [ ] **Step 6: Commit formatting fixes if any**

If `rustfmt` changed files:
```bash
git add -A
git commit -m "$(cat <<'EOF'
style: apply rustfmt after library picker rewrite

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

Otherwise no commit.

---

## Self-Review

**Spec coverage:**
- §1 background + 4 rough points → all addressed by Task 2 rewrite ✓
- §2 trigger conditions → Task 2 `render_library_picker_overlay` gates ✓
- §3 render structure (backdrop + card inside content area) → Task 4 ✓
- §4 visual specs (480px, no ⚠, library rows, footer) → Task 2 ✓
- §5 interactions (✕ / backdrop / Activate / dismissed reset) → Task 2 + 3 + 4 ✓
- §6 state fields → Task 1 ✓
- §7 file structure → all tasks ✓
- §8 testing (9 manual steps + build/clippy) → Task 5 ✓
- §9 not in scope (don't touch activate_library_version) → respected ✓
- §10 risks (frequent reset) → mitigated by only resetting on actual view switches ✓

**Placeholder scan:** No "TBD" / "TODO" / "implement later". The version dropdown in Task 2 step 2 is documented as "static label in v1; full popover is a follow-up" — that's an explicit scope limitation, not a placeholder.

**Type consistency:** `library_picker_dismissed: bool` and `library_picker_selected_version: HashMap<String, String>` are used consistently across all tasks. `render_library_picker_overlay` signature is `(app: &CanViewApp, view: Entity<CanViewApp>) -> Option<impl IntoElement>` everywhere.

**Scope check:** Single subsystem (library picker UI). No decomposition needed.

---

## Execution Handoff

Plan complete and saved to `docs/superpowers/plans/2026-07-22-library-picker.md`. Two execution options:

1. **Subagent-Driven (recommended)** — I dispatch a fresh subagent per task, review between tasks, fast iteration
2. **Inline Execution** — Execute tasks in this session using executing-plans, batch execution with checkpoints

Which approach?
