//! Chart view rendering
//!
//! This module contains the chart view rendering functionality.

use gpui::{prelude::*, *};

/// Render the chart view
///
/// Currently shows a placeholder message indicating this feature is coming soon.
pub fn render_chart_view() -> impl IntoElement {
    div()
        .size_full()
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .text_lg()
                .text_color(rgb(0x9ca3af))
                .child("Chart view - Feature coming soon")
        )
}
