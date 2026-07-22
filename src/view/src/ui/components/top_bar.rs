//! TopBar component
//!
//! Renders the top bar: File menu button (left) + Library button (right) +
//! active library badge + window controls (Win/Linux only — macOS uses
//! system traffic lights). Data view tabs (Log/Plot) live in the StatusBar,
//! not here.

use crate::app::{AppView, CanViewApp};
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
    let in_library_view = app.current_view == AppView::LibraryView
        || app.current_view == AppView::ConfigView;

    // File menu button (left side). When the dropdown is open, use a deeper
    // background fill to indicate "pressed" state — visually distinct from
    // Library's bottom-indicator active state, so they can't be confused.
    let view_for_file = view.clone();
    let file_button = div()
        .px(spacing::SM)
        .h_full()
        .flex()
        .items_center()
        .cursor_pointer()
        .text_sm()
        .text_color(if show_file_menu {
            colors::TEXT_PRIMARY
        } else {
            colors::TEXT_MUTED
        })
        .when(show_file_menu, |el| el.bg(colors::SURFACE1))
        .hover(|s| {
            if show_file_menu {
                s
            } else {
                s.text_color(colors::TEXT_SECONDARY).bg(colors::SURFACE0)
            }
        })
        .child("File")
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            cx.stop_propagation();
            view_for_file.update(cx, |app, cx| {
                app.show_file_menu = !app.show_file_menu;
                cx.notify();
            });
        });

    // Active library badge (shown when a version is activated). Clicking it
    // jumps to the Library view.
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

    // Library button (left, next to File). Acts as the configuration entry
    // point. Active state uses a bottom 2px indicator (not background fill)
    // so it can never visually clash with the File menu's open state.
    let view_for_library = view.clone();
    let library_button = div()
        .px(spacing::SM)
        .h_full()
        .flex()
        .items_center()
        .cursor_pointer()
        .text_sm()
        .text_color(if in_library_view {
            colors::TEXT_PRIMARY
        } else {
            colors::TEXT_MUTED
        })
        .hover(|s| {
            if in_library_view {
                s
            } else {
                s.text_color(colors::TEXT_SECONDARY)
            }
        })
        .when(in_library_view, |el| {
            el.border_b_2().border_color(colors::PRIMARY)
        })
        .child("Library")
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            cx.stop_propagation();
            view_for_library.update(cx, |app, cx| {
                app.current_view = AppView::LibraryView;
                cx.notify();
            });
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
        // Left: File menu + Library button (config entry) + active library badge
        .child(file_button)
        .child(library_button)
        .when_some(badge_el, |el, b| el.child(b))
        // Center: spacer that fills remaining width so the right side sticks to the edge
        .child(div().flex_1())
        // Right: window controls (non-macOS only)
        .when(!is_macos, |el| el.child(render_window_controls(view)))
}

/// Render the Win/Linux window controls (minimize, maximize, close).
fn render_window_controls(view: Entity<CanViewApp>) -> impl IntoElement {
    let view_min = view.clone();
    let view_max = view.clone();
    let _view_close = view.clone();

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
