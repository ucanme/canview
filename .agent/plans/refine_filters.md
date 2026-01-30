# Implementation Plan - Refine Channel Filter and Fix Event Leakage

The user wants the channel filter to match the ID filter implementation style and resolve the issue where hovering over the dropdown causes background elements (message rows) to be highlighted or selected.

## Proposed Changes

### 1. Unified Filter Implementation in `render_log_view`
- Ensure both ID and Channel filters use the same pattern:
    - Inline implementation within `.when(...)` blocks in `render_log_view`.
    - `absolute()` positioning.
    - `uniform_list` for items.
    - `cx.stop_propagation()` in all mouse event handlers (`on_mouse_move`, `on_mouse_down`, `on_mouse_up`).

### 2. Fix Event Leakage (Hovering/Clicking through Dropdown)
- Add `on_mouse_move`, `on_mouse_down`, and `on_mouse_up` with `cx.stop_propagation()` to the dropdown container.
- Use explicit type annotations for `cx` to avoid compilation errors (e.g., `cx: &mut AppContext`).

### 3. Disable Background Hover Highlights
- Ensure `render_message_row_static_with_widths` receives a `disable_hover` flag that is true if ANY filter dropdown is open.

## Verification Plan

### Automated Tests
- Run `cargo check -p view` to ensure no regression or type inference issues.

### Manual Verification
- Verify that hovering over the dropdown does not highlight the message rows beneath it.
- Verify that clicking a channel/ID in the dropdown works correctly and closes the dropdown.
- Verify that clicking outside the dropdown closes it via the overlay.
