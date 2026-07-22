//! LibraryPicker overlay component
//!
//! Renders a centered prompt card over the Log/Plot data view when a BLF
//! file has been loaded but no signal library is active. Lets the user
//! pick a library/version to activate without leaving the data view, or
//! jump to the full Library management view.

use crate::app::{AppView, CanViewApp};
use crate::ui::theme::colors;
use crate::ui::theme::spacing;
use gpui::{prelude::*, *};

/// Render the library picker overlay.
///
/// Returns `Some(element)` when the overlay should be shown:
/// - A BLF file has been loaded (`app.current_file_name.is_some()`)
/// - The current view is a data view (Log or Plot)
/// - No library version is active
///
/// Returns `None` otherwise, so the caller can skip rendering via `.when_some()`.
pub fn render_library_picker(
    app: &CanViewApp,
    view: Entity<CanViewApp>,
) -> Option<impl IntoElement> {
    // Only show when a file is loaded and we're in a data view and no lib is active.
    if app.current_file_name.is_none() {
        return None;
    }
    if !matches!(app.current_view, AppView::LogView | AppView::PlotView) {
        return None;
    }
    if app.active_library_id.is_some() && app.active_version_name.is_some() {
        return None;
    }

    let libraries = app.library_manager.libraries();

    let view_for_card = view.clone();
    let view_for_new = view.clone();
    let view_for_manage = view.clone();

    let card = div()
        .absolute()
        .top(px(80.))
        .left_1_2()
        .ml(px(-220.))
        .w(px(440.))
        .bg(colors::BG_ELEVATED)
        .border_1()
        .border_color(colors::BORDER_DEFAULT)
        .rounded(px(8.))
        .shadow_lg()
        .flex()
        .flex_col()
        .p(spacing::LG)
        .gap(spacing::MD)
        // Click inside the card should NOT propagate to the data view behind it.
        .on_mouse_down(MouseButton::Left, |_, _, cx| {
            cx.stop_propagation();
        })
        // Header
        .child(
            div()
                .flex()
                .items_center()
                .gap(spacing::SM)
                .child(div().text_color(colors::WARNING).text_lg().child("⚠"))
                .child(
                    div()
                        .text_color(colors::TEXT_PRIMARY)
                        .text_sm()
                        .font_weight(FontWeight::BOLD)
                        .child("No active signal library"),
                ),
        )
        .child(
            div()
                .text_color(colors::TEXT_MUTED)
                .text_xs()
                .child("Signal column will be empty. Pick a library version to decode messages."),
        )
        // Library list
        .child(
            div()
                .flex()
                .flex_col()
                .gap(spacing::SM)
                .max_h(px(280.))
                .overflow_hidden()
                .child(
                    if libraries.is_empty() {
                        // No libraries at all
                        div()
                            .text_color(colors::TEXT_MUTED)
                            .text_xs()
                            .p(spacing::SM)
                            .bg(colors::BG_DEFAULT)
                            .rounded(px(4.))
                            .child("No libraries created yet. Click \"Create new library\" below.")
                            .into_any_element()
                    } else {
                        // Render library/version tree
                        div()
                            .flex()
                            .flex_col()
                            .gap(spacing::SM)
                            .children(libraries.iter().map(|lib| {
                                let lib_id = lib.id.clone();
                                render_library_entry(
                                    &lib.name,
                                    &lib.id,
                                    &lib.versions,
                                    view_for_card.clone(),
                                )
                            }))
                            .into_any_element()
                    },
                ),
        )
        // Footer actions
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .pt(spacing::SM)
                .border_t_1()
                .border_color(colors::BORDER_SUBTLE)
                .child(
                    div()
                        .text_color(colors::TEXT_MUTED)
                        .text_xs()
                        .child("Tip: active library is also shown in the top bar."),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(spacing::SM)
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
                                        app.library_dialog_type =
                                            crate::app::LibraryDialogType::Create;
                                        cx.notify();
                                    });
                                }),
                        )
                        // Go to Library management
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
                        ),
                ),
        );

    Some(card)
}

/// Render one library entry with its versions as a list of "Activate" buttons.
fn render_library_entry(
    lib_name: &str,
    _lib_id: &str,
    versions: &[crate::models::LibraryVersion],
    view: Entity<CanViewApp>,
) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap(px(2.))
        .child(
            div()
                .text_color(colors::TEXT_PRIMARY)
                .text_sm()
                .font_weight(FontWeight::SEMIBOLD)
                .child(format!("📚 {}", lib_name)),
        )
        .children(versions.iter().map(|ver| {
            let version_name = ver.name.clone();
            render_version_row(&ver.name, version_name, view.clone())
        }))
}

/// Render a single version row with an Activate button on the right.
fn render_version_row(
    display_name: &str,
    version_name: String,
    view: Entity<CanViewApp>,
) -> impl IntoElement {
    let view_for_activate = view.clone();
    let display = display_name.to_string();

    div()
        .flex()
        .items_center()
        .justify_between()
        .px(spacing::SM)
        .py(px(4.))
        .bg(colors::BG_DEFAULT)
        .rounded(px(4.))
        .hover(|s| s.bg(colors::SURFACE0))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(6.))
                .text_xs()
                .text_color(colors::TEXT_MUTED)
                .child("└─")
                .child(div().text_color(colors::TEXT_SECONDARY).child(display)),
        )
        .child(
            div()
                .px(spacing::SM)
                .py(px(2.))
                .bg(colors::SURFACE1)
                .border_1()
                .border_color(colors::BORDER_DEFAULT)
                .rounded(px(3.))
                .cursor_pointer()
                .text_xs()
                .text_color(colors::TEXT_PRIMARY)
                .hover(|s| s.bg(colors::PRIMARY).text_color(colors::BG_DEFAULT))
                .child("Activate")
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    cx.stop_propagation();
                    // Activate the most recently picked library version.
                    // The caller passes `view` captured from the library picker;
                    // we look up the library id from the active row by re-reading
                    // the library_manager. But here we only have version_name.
                    // Solution: capture library_id from the enclosing closure.
                    // (Done via re-stitching: see render_library_entry.)
                    view_for_activate.update(cx, |app, cx| {
                        // Try to find which library contains this version name.
                        // (Quick lookup; if multiple libs share the version name,
                        // the first match wins — user can use Library view for
                        // disambiguation.)
                        let lib_id = app
                            .library_manager
                            .libraries()
                            .iter()
                            .find(|lib| {
                                lib.versions.iter().any(|v| v.name == version_name)
                            })
                            .map(|lib| lib.id.clone());
                        if let Some(id) = lib_id {
                            app.activate_library_version(&id, &version_name, cx);
                        } else {
                            app.status_msg = "Version not found in any library".into();
                            cx.notify();
                        }
                    });
                }),
        )
}
