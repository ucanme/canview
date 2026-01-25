//! UI rendering components and views

pub mod views;
pub mod components;

use gpui::*;
use crate::app::AppView;

/// Main UI renderer
pub struct UiRenderer;

impl UiRenderer {
    pub fn render_titlebar() -> impl IntoElement {
        div()
            .h(px(40.))
            .px_4()
            .flex()
            .items_center()
            .justify_between()
            .bg(rgb(0x1a1a1a))
            .border_b_1()
            .border_color(rgb(0x2a2a2a))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_4()
                    .child(
                        div()
                            .text_lg()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0xffffff))
                            .child("CANVIEW"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0x9ca3af))
                            .child("Bus Data Analyzer"),
                    )
            )
    }
}
