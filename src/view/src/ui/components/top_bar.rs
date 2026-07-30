//! TopBar component
//!
//! Renders the top bar: File menu button (left) + Library button (right) +
//! window controls (Win/Linux only — macOS uses system traffic lights).
//! Data view tabs (Log/Plot) and the active-library badge live in the
//! StatusBar, not here.

use crate::app::{AppView, CanViewerApp};
use crate::ui::theme::colors;
use crate::ui::theme::spacing;
use gpui::{prelude::*, *};

/// Render the top bar.
pub fn render_top_bar(
    app: &CanViewerApp,
    view: Entity<CanViewerApp>,
    _cx: &mut Context<CanViewerApp>,
) -> impl IntoElement {
    let is_macos = cfg!(target_os = "macos");
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

    // Active library badge moved to the status bar (bottom) to avoid
    // showing two library references simultaneously. See render_lib_badge_segment.

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
                app.library_picker_dismissed = false;
                cx.notify();
            });
        });

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
        // Left: File menu + Library button (config entry)
        .child(file_button)
        .child(library_button)
        // Center: spacer that fills remaining width so the right side sticks to the edge
        .child(div().flex_1())
        // Right: Help button (all platforms) + window controls (non-macOS only)
        .child(help_button)
        .when(!is_macos, |el| el.child(render_window_controls(view)))
}

/// Render the Win/Linux window controls (minimize, maximize, close).
fn render_window_controls(view: Entity<CanViewerApp>) -> impl IntoElement {
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
    #[test]
    fn placeholder() {}
}
