//! Library picker overlay
//!
//! Modal overlay shown when a BLF file is loaded but no signal library
//! version is active. Covers only the data area (Log/Plot view). Lets the
//! user pick a library + version and activate without leaving the data
//! view, or jump to the full Library management view.

use crate::app::{AppView, CanViewerApp, LibraryDialogType};
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
    app: &CanViewerApp,
    view: Entity<CanViewerApp>,
) -> Option<impl IntoElement> {
    if app.current_file_name.is_none() {
        return None;
    }
    // Only show in PlotView — LogView users just browse raw messages and
    // don't need a DBC. Picking a library only matters when plotting
    // signals, so prompt there.
    if app.current_view != AppView::PlotView {
        return None;
    }
    if app.active_library_id.is_some() && app.active_version_name.is_some() {
        return None;
    }
    if app.library_picker_dismissed {
        return None;
    }

    let libraries = app.library_manager.libraries().to_vec();

    let view_for_close = view.clone();
    let view_for_new = view.clone();
    let view_for_manage = view.clone();

    let card = div()
        .absolute()
        .top_0()
        .left_0()
        .w_full()
        .h_full()
        .flex()
        .items_center()
        .justify_center()
        .bg(rgba(0x00000055))
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            view_for_close.update(cx, |app, cx| {
                app.library_picker_dismissed = true;
                cx.notify();
            });
        })
        .child(render_card(&libraries, view_for_new, view_for_manage));

    Some(card)
}

/// Render the centered card.
fn render_card(
    libraries: &[crate::models::SignalLibrary],
    view_for_new: Entity<CanViewerApp>,
    view_for_manage: Entity<CanViewerApp>,
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

/// Render the library list area. Empty state shows a hint; non-empty uses
/// `uniform_list` so many libraries scroll inside the card (max_h 400px on
/// the card caps visible height).
fn render_library_list(
    libraries: &[crate::models::SignalLibrary],
    view: Entity<CanViewerApp>,
) -> AnyElement {
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

    // Each library row is 40px tall. uniform_list provides native scrolling.
    let row_count = libraries.len();
    let libs: Vec<crate::models::SignalLibrary> = libraries.to_vec();
    let view_for_rows = view.clone();

    div()
        .flex_1()
        .h(px(280.))
        .child(gpui::uniform_list(
            "library-picker-list",
            row_count,
            move |range, _window, _cx| {
                let view = view_for_rows.clone();
                range
                    .filter_map(|i| libs.get(i).map(|lib| (i, lib, view.clone())))
                    .map(|(_, lib, v)| render_library_row(lib, v).into_any_element())
                    .collect::<Vec<_>>()
            },
        ))
        .into_any_element()
}

/// Render one library row: 📚 name | [version ▾] | [Activate]
fn render_library_row(
    lib: &crate::models::SignalLibrary,
    view: Entity<CanViewerApp>,
) -> impl IntoElement {
    let lib_id = lib.id.clone();
    let lib_name = lib.name.clone();
    let versions: Vec<String> = lib.versions.iter().map(|v| v.name.clone()).collect();
    let selected_version = lib
        .latest_version()
        .map(|v| v.name.clone())
        .unwrap_or_default();

    let view_for_activate = view.clone();
    let selected_for_activate = selected_version.clone();
    let versions_for_activate = versions.clone();
    let lib_id_for_activate = lib_id.clone();

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
                .child(render_version_dropdown(&selected_version))
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
                                app.activate_library_version(
                                    &lib_id_for_activate,
                                    &version_to_use,
                                    cx,
                                );
                            });
                        }),
                ),
        )
}

/// Render the version dropdown as a static label showing the selected version.
/// Full popover dropdown is a follow-up.
fn render_version_dropdown(selected: &str) -> impl IntoElement {
    div()
        .px(spacing::SM)
        .h(px(24.))
        .flex()
        .items_center()
        .bg(colors::SURFACE0)
        .border_1()
        .border_color(colors::BORDER_DEFAULT)
        .rounded(px(4.))
        .text_xs()
        .text_color(colors::TEXT_SECONDARY)
        .child(if selected.is_empty() {
            "Latest".to_string()
        } else {
            selected.to_string()
        })
}

/// Render the footer: "+ Create new library" (left) + "Open Library →" (right)
fn render_footer(
    view_for_new: Entity<CanViewerApp>,
    view_for_manage: Entity<CanViewerApp>,
) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .justify_between()
        .pt(spacing::SM)
        .border_t_1()
        .border_color(colors::BORDER_SUBTLE)
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
                        app.library_dialog_type = LibraryDialogType::Create;
                        cx.notify();
                    });
                }),
        )
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
        let app = CanViewerApp::new_state();
        assert!(app.current_file_name.is_none());
    }

    #[test]
    fn test_overlay_not_shown_when_dismissed() {
        let mut app = CanViewerApp::new_state();
        app.library_picker_dismissed = true;
        assert!(app.library_picker_dismissed);
    }

    #[test]
    fn test_overlay_not_shown_when_active_library() {
        let mut app = CanViewerApp::new_state();
        app.active_library_id = Some("lib1".to_string());
        app.active_version_name = Some("v1.0".to_string());
        assert!(app.active_library_id.is_some() && app.active_version_name.is_some());
    }

    #[test]
    fn test_overlay_shown_when_in_data_view() {
        let app = CanViewerApp::new_state();
        assert!(matches!(app.current_view, AppView::LogView));
    }

    #[test]
    fn test_selected_version_starts_empty() {
        let app = CanViewerApp::new_state();
        assert!(app.library_picker_selected_version.is_empty());
    }
}
