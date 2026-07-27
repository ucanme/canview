# Help Button — Design

**Date:** 2026-07-27
**Branch:** `feat/ui-redesign`

## Goal

Add a Help button to the top bar with two entries — "View on GitHub" (opens `https://github.com/ucanme/canview`) and "Send Feedback" (opens `mailto:admin@ucan.me`). On macOS, also expose these as items in the native app menu (next to the Apple menu, under the "CANVIEW" application menu). Windows/Linux use the top-bar dropdown only.

## Background

The top bar lives in `src/view/src/ui/components/top_bar.rs` — it currently has File + Library buttons on the left, and window controls on the right (Win/Linux only). The File menu dropdown is rendered inline in `src/view/src/app/impls_rendering.rs` triggered by `show_file_menu: bool` in `CanViewApp` state (`src/view/src/app/state.rs`). URL opening uses `cx.open_url(&str)` (already used by the share dialog at `impls_rendering.rs:2221`).

GPUI exposes native menus via `cx.set_menus(Vec<Menu>)` (`gpui::app.rs:1840`). On macOS this calls `app.setMainMenu_(menu)` and the menu appears next to the Apple menu. On Windows and Linux, `set_menus` only stores the menus — there is no native menu bar rendering. The top-bar dropdown is therefore the universal fallback.

## Design

### Placement

- **Top bar**: Help button on the right side, flush right on macOS (no window controls), to the left of the window controls on Win/Linux. Same height and padding as the File/Library buttons (`h_full`, `px(spacing::SM)`).
- **macOS native app menu**: A `Menu { name: "CANVIEW", items: [...] }` with the two actions, set once at startup via `cx.set_menus()`. Appears as the bold app menu next to the Apple menu.
- **Windows/Linux**: Top-bar dropdown only. `set_menus()` is still called (cheap, stored) but produces no visible menu bar — accepted as a no-op.

### Top Bar Help Button

Styling mirrors the existing File button exactly so the bar reads consistently:

- Default: `text_color(colors::TEXT_MUTED)`, no background
- Hover: `text_color(colors::TEXT_SECONDARY)`, `bg(colors::SURFACE0)`
- Open (pressed): `text_color(colors::TEXT_PRIMARY)`, `bg(colors::SURFACE1)` — same "pressed" fill the File menu uses (`top_bar.rs:40`)

Toggle on `on_mouse_down(MouseButton::Left)` with `cx.stop_propagation()` and `app.show_help_menu = !app.show_help_menu`.

### Help Dropdown

Rendered in `impls_rendering.rs` as a sibling of the File menu dropdown (around line 1899). Same popover style:

- `absolute()`, `top(px(36.))`, `right(px(16.))`, `w(px(200.))`
- `bg(rgb(0x313244))`, `border_1()`, `border_color(rgb(0x45475a))`, `rounded(px(6.))`, `shadow_lg()`
- `flex().flex_col().py_1()`
- `on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())` — keep clicks inside from closing

Two items, each `px_3().py_1().text_xs().text_color(rgb(0xcdd6f4)).hover(|s| s.bg(rgb(0x45475a))).cursor_pointer()`:

1. **"View on GitHub"** — `cx.open_url("https://github.com/ucanme/canview")`
2. **"Send Feedback"** — `cx.open_url("mailto:admin@ucan.me?subject=CANVIEW%20Feedback")`

Both close the dropdown (`app.show_help_menu = false; cx.notify();`) on click.

### Click-Outside Overlay

A full-screen `div().absolute().top_0().left_0().w_full().h_full().bg(rgba(0x00000033))` rendered when `show_help_menu` is true, with an `on_mouse_down` handler that sets `app.show_help_menu = false`. Identical pattern to the File menu overlay at `impls_rendering.rs:1878-1898`.

### State

Add `pub show_help_menu: bool` to `CanViewApp` in `src/view/src/app/state.rs`, next to `show_file_menu` (line 239). Initialize to `false`:

- `state.rs:391` (in `new_with_maximized_state_and_bounds`)
- `impls.rs:144` (in any reset path that re-initializes state)
- `impls.rs:889` (second reset path)

### Native macOS App Menu

In `src/view/src/main.rs`, inside `app.run(move |cx| { ... })` after `gpui_component::init(cx)`:

1. Define two GPUI actions near the top of the file (next to the existing `library_input` actions at `app/mod.rs`):
   ```rust
   gpui::actions!(help, [OpenGitHubUrl, SendFeedbackEmail]);
   ```
2. Call `cx.set_menus(vec![Menu { name: "CANVIEW".into(), items: vec![
       MenuItem::action("View on GitHub", OpenGitHubUrl),
       MenuItem::separator(),
       MenuItem::action("Send Feedback", SendFeedbackEmail),
   ] }])`.
   - Note: GPUI on macOS automatically adds standard items (About, Hide, Quit) to the app menu. We only need to add our custom entries.
3. Register `cx.on_action(|_: &OpenGitHubUrl, cx| { cx.open_url("https://github.com/ucanme/canview"); })` and the same for `SendFeedbackEmail` with the `mailto:` URL.

The menus are set once at startup. They live alongside whatever default menus GPUI adds (Quit, About, etc. on macOS). On Windows/Linux, `set_menus` is a no-op visually.

### URL Schemes

- `https://github.com/ucanme/canview` — opens in the system browser
- `mailto:admin@ucan.me?subject=CANVIEW%20Feedback` — opens in the user's default mail client with the subject pre-filled

`cx.open_url` is the only API used. Both schemes are passed through; GPUI dispatches to the OS handler (`NSWorkspace.openURL` on macOS, `ShellExecuteW` on Windows, `xdg-open` on Linux — handled internally by GPUI).

## Files Touched

- `src/view/src/ui/components/top_bar.rs` — add Help button on the right
- `src/view/src/app/impls_rendering.rs` — add dropdown menu + click-outside overlay (next to File menu dropdown at line 1899)
- `src/view/src/app/state.rs` — add `show_help_menu: bool` field (line 239), init to `false` (line 391)
- `src/view/src/app/impls.rs` — initialize the new field at lines 144 and 889
- `src/view/src/main.rs` — add `gpui::actions!(help, [...])`, `cx.set_menus(...)`, and `cx.on_action(...)` handlers inside `app.run`

## Testing

- **cargo check** on macOS, Windows, Linux (at minimum `cargo check` on macOS; cross-platform compile guarantees via `cfg`-gated code paths)
- **macOS manual**:
  - Top bar: Help button opens dropdown, click-outside closes, both items fire `cx.open_url` (verify browser/mail client opens)
  - App menu: items appear under the bold "CANVIEW" menu next to the Apple menu, clicking fires `cx.open_url`
- **Windows/Linux manual**:
  - Top-bar dropdown opens/closes, both items fire `cx.open_url`
  - Native menu bar is not expected to render (accepted limitation)

## Out of Scope

- No "About" dialog (the native macOS app menu already provides About)
- No keyboard shortcuts for the help actions (can be added later via `keymap`)
- No localization — labels are English-only (matches the rest of the top bar)
- No icon for the Help button (matches File/Library which are text-only)
