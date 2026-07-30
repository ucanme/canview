//! Simple status view example
//!
//! This module demonstrates how to extract view rendering logic from impls.rs
//! into a separate module. This is a simplified example that can be used as
//! a reference for extracting more complex views.

use crate::app::CanViewerApp;
use gpui::{prelude::*, rgb, *};

/// Render a simple status view
///
/// This demonstrates the pattern for extracting views:
/// 1. Take a reference to app state (or Entity for mutable access)
/// 2. Take GPUI context
/// 3. Return an impl IntoElement
///
/// # Example Usage in impls.rs
/// ```rust
/// impl CanViewerApp {
///     fn render_status_view(&self, cx: &mut Context<Self>) -> impl IntoElement {
///         views::status_view::render(self, cx)
///     }
/// }
/// ```
pub fn render_status_view(
    app: &CanViewerApp,
    _cx: &mut Context<CanViewerApp>,
) -> impl IntoElement {
    div()
        .size_full()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap_4()
        .child(
            div()
                .text_xl()
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(0xffffff))
                .child("canview Status"),
        )
        .child(
            div()
                .text_lg()
                .text_color(rgb(0xd1d5db))
                .child(app.status_msg.clone()),
        )
        .child(render_status_badges(app))
}

/// Render status badges showing system information
fn render_status_badges(app: &CanViewerApp) -> Div {
    let message_count = app.messages.len();
    let dbc_count = app.dbc_channels.len();
    let ldf_count = app.ldf_channels.len();

    div()
        .flex()
        .gap_3()
        .child(render_badge("Messages", &message_count.to_string(), gpui::rgba(0x3b82f6ff)))
        .child(render_badge("DBC Files", &dbc_count.to_string(), gpui::rgba(0x10b981ff)))
        .child(render_badge("LDF Files", &ldf_count.to_string(), gpui::rgba(0xf59e0bff)))
}

/// Render a single status badge
fn render_badge(label: &str, value: &str, color: gpui::Rgba) -> Div {
    div()
        .flex()
        .flex_col()
        .items_center()
        .gap_1()
        .child(
            div()
                .text_xs()
                .text_color(rgb(0x9ca3af))
                .child(label.to_string()),
        )
        .child(
            div()
                .px_3()
                .py_1()
                .bg(color)
                .rounded(px(4.))
                .text_sm()
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgb(0xffffff))
                .child(value.to_string()),
        )
}

/// Example: How to create a simple info card
pub fn render_info_card(
    title: &str,
    content: &str,
    icon_color: gpui::Rgba,
) -> impl IntoElement {
    div()
        .px_4()
        .py_3()
        .bg(rgb(0x1f2937))
        .border_1()
        .border_color(rgb(0x374151))
        .rounded(px(8.))
        .flex()
        .items_center()
        .gap_3()
        .child(
            div()
                .w_4()
                .h_4()
                .rounded(px(2.))
                .bg(icon_color),
        )
        .child(
            div()
                .flex_1()
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(rgb(0xffffff))
                        .child(title.to_string()),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(0x9ca3af))
                        .child(content.to_string()),
                ),
        )
}
