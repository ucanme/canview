# Help Button Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a Help button to the top bar with "View on GitHub" and "Send Feedback" entries (opening the GitHub repo URL and a `mailto:` link), and mirror these entries in the native macOS app menu.

**Architecture:** New `show_help_menu: bool` state field on `CanViewApp` toggles a top-bar dropdown rendered in `impls_rendering.rs` alongside the existing File dropdown. The dropdown items call `cx.open_url(...)`. Separately, `main.rs` calls `cx.set_menus(...)` once at startup with three GPUI actions (`OpenGitHubUrl`, `SendFeedbackEmail`, `QuitApp`); on macOS these appear in the native app menu next to the Apple menu, on Windows/Linux `set_menus` is a no-op visually. The same `cx.open_url(...)` calls service both entry points.

**Tech Stack:** Rust, GPUI 0.2.2 (`gpui::Menu`, `gpui::MenuItem`, `gpui::actions!`, `cx.set_menus`, `cx.on_action`, `cx.open_url`, `cx.quit`), Catppuccin Mocha theme via `crate::ui::theme::colors`.

## Global Constraints

- All new code must compile on macOS, Windows, and Linux. Platform-specific behavior is gated by `cfg!(target_os = ...)` and `cfg!` attributes — never `#[cfg]`-out a function body without a stub on other platforms.
- Theme colors come from `crate::ui::theme::colors::*` (e.g., `colors::TEXT_MUTED`, `colors::SURFACE0`, `colors::SURFACE1`, `colors::TEXT_PRIMARY`, `colors::TEXT_SECONDARY`). Spacing comes from `crate::ui::theme::spacing::*` (e.g., `spacing::SM`, `spacing::LG`).
- The two URLs are exact: `https://github.com/ucanme/canview` and `mailto:admin@ucan.me?subject=CANVIEW%20Feedback`. Do not change them.
- Styling of the Help button and dropdown must visually match the existing File menu button (`top_bar.rs:28-55`) and the File dropdown (`impls_rendering.rs:1899-2097`) — same colors, same padding, same border, same shadow.
- No emojis, no extra comments beyond what the existing code style uses (one-line `//` comments for section dividers).
- Do not skip hooks. Do not use `--amend` for commits — create new commits.

---

## File Structure

| File | Change | Responsibility |
|---|---|---|
| `src/view/src/app/state.rs` | Modify | Add `show_help_menu: bool` field declaration + initialization |
| `src/view/src/app/impls.rs` | Modify | Initialize `show_help_menu` in the two state constructors (lines 144 and 889) |
| `src/view/src/ui/components/top_bar.rs` | Modify | Add Help button on the right side of the top bar |
| `src/view/src/app/impls_rendering.rs` | Modify | Add Help dropdown + click-outside overlay next to the File dropdown |
| `src/view/src/main.rs` | Modify | Declare `help` actions, call `cx.set_menus(...)`, register `cx.on_action(...)` handlers |

No new files are created — all changes fold into existing modules following established patterns.

---

## Task 1: Add `show_help_menu` state field

**Files:**
- Modify: `src/view/src/app/state.rs:239` (add field after `show_file_menu`)
- Modify: `src/view/src/app/state.rs:391` (add initialization after `show_file_menu: false`)
- Modify: `src/view/src/app/impls.rs:144` (add initialization in first constructor)
- Modify: `src/view/src/app/impls.rs:889` (add initialization in second constructor)

**Interfaces:**
- Produces: `pub show_help_menu: bool` field on `CanViewApp` — used by Tasks 3 and 4
- Consumes: nothing (Task 1 is the foundation)

- [ ] **Step 1: Add the field declaration in `state.rs`**

In `src/view/src/app/state.rs`, locate lines 238-239 (the `// File menu dropdown state` block) and replace:

```rust
    // File menu dropdown state
    pub show_file_menu: bool,
```

with:

```rust
    // File menu dropdown state
    pub show_file_menu: bool,
    // Help menu dropdown state
    pub show_help_menu: bool,
```

- [ ] **Step 2: Initialize the field in `state.rs`**

In the same file, locate lines 390-391 (the `// File menu dropdown state` block inside `new_with_maximized_state_and_bounds`) and replace:

```rust
            // File menu dropdown state
            show_file_menu: false,
```

with:

```rust
            // File menu dropdown state
            show_file_menu: false,
            // Help menu dropdown state
            show_help_menu: false,
```

- [ ] **Step 3: Initialize the field in the first constructor in `impls.rs`**

In `src/view/src/app/impls.rs`, locate lines 143-144 (the `// File menu dropdown state` block) and replace:

```rust
            // File menu dropdown state
            show_file_menu: false,
```

with:

```rust
            // File menu dropdown state
            show_file_menu: false,
            // Help menu dropdown state
            show_help_menu: false,
```

- [ ] **Step 4: Initialize the field in the second constructor in `impls.rs`**

In the same file, locate lines 888-889 (the second `// File menu dropdown state` block) and apply the same replacement as Step 3.

- [ ] **Step 5: Verify the build still compiles**

Run: `cargo check --workspace`
Expected: PASS with no errors. (The new field is initialized everywhere `CanViewApp` is constructed; no other code references it yet, so this is a no-op change behaviorally.)

- [ ] **Step 6: Commit**

```bash
git add src/view/src/app/state.rs src/view/src/app/impls.rs
git commit -m "feat(help): add show_help_menu state field to CanViewApp"
```

---

## Task 2: Declare `help` actions and wire up the macOS native app menu

**Files:**
- Modify: `src/view/src/main.rs:1-49` (add `gpui::actions!` declaration after `parse_file_urls`)
- Modify: `src/view/src/main.rs:103-132` (replace `app.run` closure body to add `set_menus` and `on_action` handlers)

**Interfaces:**
- Produces: three GPUI actions `help::OpenGitHubUrl`, `help::SendFeedbackEmail`, `help::QuitApp` available at module scope in `main.rs`
- Produces: native app menu wired up with two URL actions and one quit action; the menu is set once at startup
- Consumes: `gpui::Menu`, `gpui::MenuItem`, `gpui::Application`, `cx.set_menus`, `cx.on_action`, `cx.open_url`, `cx.quit`

- [ ] **Step 1: Add the `help` actions declaration**

In `src/view/src/main.rs`, locate the end of the `parse_file_urls` function (line 47, the closing `}`) and insert immediately after it (before `fn main() {`):

```rust
// Actions for the Help menu (top-bar dropdown + native macOS app menu).
// Declared at module scope so the `set_menus` call and `on_action` handlers
// can reference them by name.
gpui::actions!(help, [OpenGitHubUrl, SendFeedbackEmail, QuitApp]);
```

- [ ] **Step 2: Wire up `set_menus` and `on_action` handlers in `app.run`**

In `src/view/src/main.rs`, locate the `app.run(move |cx| { ... })` body (lines 103-132). Replace the entire `app.run(move |cx| { ... })` call with:

```rust
    app.run(move |cx| {
        // This must be called before using any GPUI Component features
        gpui_component::init(cx);

        // Wire up the native app menu. On macOS this populates the menu bar
        // next to the Apple menu; on Windows/Linux GPUI stores the menus but
        // does not render them visually (the top-bar Help dropdown covers
        // those platforms — see top_bar.rs + impls_rendering.rs).
        cx.set_menus(vec![
            gpui::Menu {
                name: "CANVIEW".into(),
                items: vec![
                    gpui::MenuItem::action("View on GitHub", help::OpenGitHubUrl),
                    gpui::MenuItem::separator(),
                    gpui::MenuItem::action("Send Feedback", help::SendFeedbackEmail),
                    gpui::MenuItem::separator(),
                    gpui::MenuItem::action("Quit CANVIEW", help::QuitApp),
                ],
            },
        ]);
        cx.on_action(|_action: &help::OpenGitHubUrl, cx| {
            cx.open_url("https://github.com/ucanme/canview");
        });
        cx.on_action(|_action: &help::SendFeedbackEmail, cx| {
            cx.open_url("mailto:admin@ucan.me?subject=CANVIEW%20Feedback");
        });
        cx.on_action(|_action: &help::QuitApp, cx| {
            cx.quit();
        });

        cx.spawn(async move |cx| {
            let options = WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point::new(px(200.0), px(150.0)),
                    size: gpui::Size {
                        width: px(1600.0),
                        height: px(1000.0),
                    },
                })),
                titlebar: Some(TitlebarOptions {
                    title: Some("CANVIEW - Bus Data Analyzer".into()),
                    appears_transparent: true,
                    traffic_light_position: None,
                }),
                kind: gpui::WindowKind::Normal,
                ..Default::default()
            };
            cx.open_window(options, |window, cx| {
                let view = cx.new(|_cx| CanViewApp::new());
                // This first level on the window should be a Root for gpui-component
                cx.new(|cx| gpui_component::Root::new(view, window, cx))
            })?;
            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
```

Note: the `cx.spawn(async move |cx| { ... })` block inside `app.run` is preserved verbatim from the existing code — only the lines before it (between `gpui_component::init(cx);` and `cx.spawn(...)`) are new.

- [ ] **Step 3: Verify the build compiles**

Run: `cargo check --workspace`
Expected: PASS. If you see "cannot find type `help` in this scope" or similar, the `gpui::actions!` macro needs to be reachable — verify Step 1 placed the declaration at module scope (outside any `fn`).

- [ ] **Step 4: Run the app on macOS and verify the native menu appears**

Run: `cargo run --release --bin view`
Expected: App opens. In the macOS menu bar at the top of the screen, the bold "CANVIEW" menu appears next to the Apple () menu. Clicking it reveals "View on GitHub", a separator, "Send Feedback", a separator, and "Quit CANVIEW". Clicking "View on GitHub" opens `https://github.com/ucanme/canview` in the default browser. Clicking "Send Feedback" opens a new email in the default mail client with the subject "CANVIEW Feedback" pre-filled. Clicking "Quit CANVIEW" quits the app.

If you are not on macOS, skip this step and note it in the commit message — manual macOS verification is required before merging.

- [ ] **Step 5: Commit**

```bash
git add src/view/src/main.rs
git commit -m "feat(help): wire up native macOS app menu with GitHub/Feedback/Quit"
```

---

## Task 3: Add the Help button to the top bar

**Files:**
- Modify: `src/view/src/ui/components/top_bar.rs:14-116` (extend `render_top_bar` to add a Help button on the right)

**Interfaces:**
- Consumes: `app.show_help_menu: bool` (from Task 1)
- Produces: a Help button in the top bar that toggles `app.show_help_menu`
- Consumes: `crate::ui::theme::{colors, spacing}` (already imported at `top_bar.rs:9-11`)

- [ ] **Step 1: Add the Help button closure inside `render_top_bar`**

In `src/view/src/ui/components/top_bar.rs`, locate the `render_top_bar` function body. Find the `let left_pad = ...` line (line 97) and insert the following closure definition immediately **before** it:

```rust
    // Help button (right side). Toggles the help dropdown (GitHub / Feedback).
    // Styling mirrors the File menu button so the top bar reads consistently.
    let show_help_menu = app.show_help_menu;
    let view_for_help = view.clone();
    let help_button = div()
        .px(spacing::SM)
        .h_full()
        .flex()
        .items_center()
        .cursor_pointer()
        .text_sm()
        .text_color(if show_help_menu {
            colors::TEXT_PRIMARY
        } else {
            colors::TEXT_MUTED
        })
        .when(show_help_menu, |el| el.bg(colors::SURFACE1))
        .hover(|s| {
            if show_help_menu {
                s
            } else {
                s.text_color(colors::TEXT_SECONDARY).bg(colors::SURFACE0)
            }
        })
        .child("Help")
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            cx.stop_propagation();
            view_for_help.update(cx, |app, cx| {
                app.show_help_menu = !app.show_help_menu;
                cx.notify();
            });
        });
```

- [ ] **Step 2: Insert the Help button into the top-bar layout, on the right side**

In the same file, locate the `div()` builder that starts at line 99 and assembles the top bar. The current structure is:

```rust
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
        // Left: File menu + Library button (config entry)
        .child(file_button)
        .child(library_button)
        // Center: spacer that fills remaining width so the right side sticks to the edge
        .child(div().flex_1())
        // Right: window controls (non-macOS only)
        .when(!is_macos, |el| el.child(render_window_controls(view)))
```

Replace the section from `.child(div().flex_1())` through the end with:

```rust
        // Center: spacer that fills remaining width so the right side sticks to the edge
        .child(div().flex_1())
        // Right: Help button (all platforms) + window controls (non-macOS only)
        .child(help_button)
        .when(!is_macos, |el| el.child(render_window_controls(view)))
```

- [ ] **Step 3: Verify the build compiles**

Run: `cargo check --workspace`
Expected: PASS. If you see "use of moved value: `view`", verify you added `let view_for_help = view.clone();` before the `help_button` closure and that you did not also move `view` into `help_button` twice.

- [ ] **Step 4: Run the app and verify the Help button appears**

Run: `cargo run --release --bin view`
Expected: App opens. The top bar shows "File" and "Library" buttons on the left, and a "Help" button on the right (flush right on macOS; just left of the minimize/maximize/close controls on Windows/Linux). Hover changes the text color; clicking toggles a darker "pressed" background (but nothing opens yet — the dropdown is wired in Task 4).

- [ ] **Step 5: Commit**

```bash
git add src/view/src/ui/components/top_bar.rs
git commit -m "feat(help): add Help button to top bar"
```

---

## Task 4: Render the Help dropdown and click-outside overlay

**Files:**
- Modify: `src/view/src/app/impls_rendering.rs:1899` (add a new `.child(...)` block immediately after the File dropdown block ends at line 2097)

**Interfaces:**
- Consumes: `self.show_help_menu: bool` (from Task 1), `view: Entity<CanViewApp>` (already in scope at this point in the file)
- Produces: a Help dropdown with two items that call `cx.open_url(...)` and a full-screen click-outside overlay

- [ ] **Step 1: Locate the insertion point**

In `src/view/src/app/impls_rendering.rs`, locate line 2097 (the closing `})` of the File dropdown `.child({...})` block — the line right after the `}` that closes the `else { div().hidden() }` branch of the File menu). The next line is `// Share dialog overlay` at line 2098-2099.

- [ ] **Step 2: Insert the Help dropdown overlay (click-outside)**

Immediately after line 2097 (the File dropdown's closing `})`) and before the `// Share dialog overlay` comment, insert:

```rust
            .child({
                // Full-screen overlay to catch clicks outside help dropdown
                if self.show_help_menu {
                    let view_for_overlay = view.clone();
                    div()
                        .absolute()
                        .top_0()
                        .left_0()
                        .w_full()
                        .h_full()
                        .bg(rgba(0x00000033))
                        .on_mouse_down(gpui::MouseButton::Left, move |_event, _window, cx| {
                            view_for_overlay.update(cx, |app, cx| {
                                app.show_help_menu = false;
                                cx.notify();
                            });
                        })
                } else {
                    div().hidden()
                }
            })
```

- [ ] **Step 3: Insert the Help dropdown menu itself**

Immediately after the overlay block from Step 2 (still before the `// Share dialog overlay` comment), insert:

```rust
            .child({
                // Help dropdown menu — GitHub + Feedback
                if self.show_help_menu {
                    let view_for_github = view.clone();
                    let view_for_feedback = view.clone();
                    div()
                        .absolute()
                        .top(px(36.))
                        .right(px(16.))
                        .w(px(200.))
                        .bg(rgb(0x313244))
                        .border_1()
                        .border_color(rgb(0x45475a))
                        .rounded(px(6.))
                        .shadow_lg()
                        .flex()
                        .flex_col()
                        .py_1()
                        .on_mouse_down(gpui::MouseButton::Left, |_event, _window, cx| {
                            cx.stop_propagation();
                        })
                        // View on GitHub
                        .child(
                            div()
                                .px_3()
                                .py_1()
                                .text_xs()
                                .text_color(rgb(0xcdd6f4))
                                .hover(|style| style.bg(rgb(0x45475a)))
                                .cursor_pointer()
                                .on_mouse_down(gpui::MouseButton::Left, move |_event, _window, cx| {
                                    cx.stop_propagation();
                                    cx.open_url("https://github.com/ucanme/canview");
                                    view_for_github.update(cx, |app, cx| {
                                        app.show_help_menu = false;
                                        cx.notify();
                                    });
                                })
                                .child("View on GitHub"),
                        )
                        // Send Feedback
                        .child(
                            div()
                                .px_3()
                                .py_1()
                                .text_xs()
                                .text_color(rgb(0xcdd6f4))
                                .hover(|style| style.bg(rgb(0x45475a)))
                                .cursor_pointer()
                                .on_mouse_down(gpui::MouseButton::Left, move |_event, _window, cx| {
                                    cx.stop_propagation();
                                    cx.open_url("mailto:admin@ucan.me?subject=CANVIEW%20Feedback");
                                    view_for_feedback.update(cx, |app, cx| {
                                        app.show_help_menu = false;
                                        cx.notify();
                                    });
                                })
                                .child("Send Feedback"),
                        )
                } else {
                    div().hidden()
                }
            })
```

- [ ] **Step 4: Verify the build compiles**

Run: `cargo check --workspace`
Expected: PASS. If `cx.open_url` is not found, verify the closure argument is named `cx` (not `_cx`); `App` methods require the un-prefixed binding.

- [ ] **Step 5: Run the app and verify the dropdown works**

Run: `cargo run --release --bin view`
Expected:
1. Click "Help" in the top bar — a dropdown appears at the top-right with two items: "View on GitHub" and "Send Feedback".
2. Click anywhere outside the dropdown — the dropdown closes (the dark overlay catches the click).
3. Click "Help" again, then click "View on GitHub" — the default browser opens `https://github.com/ucanme/canview` and the dropdown closes.
4. Click "Help" again, then click "Send Feedback" — the default mail client opens a new message addressed to `admin@ucan.me` with the subject "CANVIEW Feedback" pre-filled, and the dropdown closes.
5. Verify the File menu and Help menu are independent: opening one does not open the other; opening one while the other is open leaves the other alone (each has its own state field and its own overlay).

- [ ] **Step 6: Commit**

```bash
git add src/view/src/app/impls_rendering.rs
git commit -m "feat(help): render Help dropdown with GitHub and Feedback entries"
```

---

## Task 5: Final cross-platform build verification and self-review

**Files:**
- No file changes — verification only

**Interfaces:**
- Consumes: all prior tasks

- [ ] **Step 1: Run clippy on the workspace**

Run: `cargo clippy --workspace --all-targets -- -D warnings`
Expected: PASS with no new warnings. If clippy flags something introduced by this plan, fix it and amend the relevant task's commit (or create a follow-up commit — do not use `--amend`).

- [ ] **Step 2: Verify macOS run**

Run: `cargo run --release --bin view`
Expected:
- Top bar shows "Help" button on the right.
- Clicking it opens the dropdown with the two items.
- Both items trigger the expected URL/email.
- The native macOS app menu (next to the Apple menu) shows "View on GitHub", "Send Feedback", and "Quit CANVIEW" under the "CANVIEW" submenu.
- "Quit CANVIEW" quits the app.

- [ ] **Step 3: Verify Windows or Linux run (whichever is available)**

If on Linux:
Run: `cargo run --release --bin view`
Expected: Top bar shows "Help" button on the right (just left of the minimize/maximize/close controls). Dropdown works. Native app menu is not visible (accepted — GPUI on Linux stores but does not render the menu bar).

If on Windows:
Run: `cargo run --release --bin view.exe`
Expected: Same as Linux — top-bar dropdown works; native menu bar is not visible.

If neither Windows nor Linux is available, skip this step and note "Cross-platform compile verified via `cargo check`; manual Windows/Linux run pending" in the PR description.

- [ ] **Step 4: Run the test suite**

Run: `cargo test --workspace`
Expected: PASS with no test failures. (This plan does not add unit tests — the feature is purely UI and is verified manually. Existing tests should not regress.)

- [ ] **Step 5: Final commit (if any cleanup was needed)**

If clippy or tests required changes:
```bash
git add <changed files>
git commit -m "chore(help): post-merge cleanup from clippy/test review"
```

If no cleanup was needed, this step is a no-op — do not create an empty commit.

- [ ] **Step 6: Open a PR**

Use `gh pr create` with a body summarizing:
- What was added (Help button + dropdown + macOS app menu)
- How it was tested (manual run on each available platform, `cargo check`/`cargo clippy`/`cargo test` results)
- Any platform that could not be manually tested

Reference the design spec: `docs/superpowers/specs/2026-07-27-help-button-design.md`.
