//! FilterBar component
//!
//! Renders a horizontal filter/option bar for Log and Library views.
//! Variant-specific controls are selected via FilterBarVariant.

use crate::app::CanViewApp;
use crate::models::library::DatabaseType;
use crate::ui::components::{Button, ButtonSize, ButtonVariant};
use crate::ui::theme::colors;
use crate::ui::theme::spacing;
use gpui::{prelude::*, *};

/// Which view's filter set to render.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FilterBarVariant {
    Log,
    /// Library view — not yet wired up. See
    /// `// TODO: wire up FilterBar for Library view in follow-up commit`
    /// in `app/impls_rendering.rs::render_library_view`.
    #[allow(dead_code)]
    Library,
}

/// Render a single filter chip. Clicking invokes `on_click` (which should
/// toggle a dropdown or open an input). The `on_click` closure receives the
/// app context so it can call `view.update(cx, ...)`.
pub fn render_filter_chip(
    label: &str,
    value: &str,
    active: bool,
    on_click: impl Fn(&mut App) + 'static,
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
            on_click(cx);
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
        .child(render_filter_chip("ID", &id_value, app.id_filter.is_some(), move |cx| {
            // Toggle the ID filter dropdown — reuses the existing show_id_filter_input state
            view_for_id.update(cx, |app, cx| {
                app.show_id_filter_input = !app.show_id_filter_input;
                cx.notify();
            });
        }))
        // Channel chip
        .child(render_filter_chip("Channel", &channel_value, app.channel_filter.is_some(), move |cx| {
            view_for_channel.update(cx, |app, cx| {
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
                            view_for_points.update(cx, |app, cx| {
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
        Some(DatabaseType::DBC) => "DBC".to_string(),
        Some(DatabaseType::LDF) => "LDF".to_string(),
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
        .child(render_filter_chip("Type", &type_value, app.library_filter_type.is_some(), move |cx| {
            // Cycle through: None -> DBC -> LDF -> None
            view_for_type.update(cx, |app, cx| {
                app.library_filter_type = match app.library_filter_type {
                    None => Some(DatabaseType::DBC),
                    Some(DatabaseType::DBC) => Some(DatabaseType::LDF),
                    Some(DatabaseType::LDF) => None,
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
                    view_for_new.update(cx, |app, cx| {
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
                    view_for_share.update(cx, |app, cx| {
                        if app.server_handle.is_some() {
                            // Stop share — reuse existing logic
                            if let Some(mut handle) = app.server_handle.take() {
                                handle.shutdown();
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
                    view_for_import.update(cx, |app, cx| {
                        app.show_import_dialog = true;
                        cx.notify();
                    });
                }),
        )
}

/// Render a small toggle button. The `on_click` closure should already
/// capture a `view: Entity<CanViewApp>` clone and call `view.update(cx, ...)`.
fn render_toggle_button(
    label: &str,
    active: bool,
    on_click: impl Fn(&mut App) + 'static,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_bar_variant_equality() {
        assert_ne!(FilterBarVariant::Log, FilterBarVariant::Library);
    }
}
