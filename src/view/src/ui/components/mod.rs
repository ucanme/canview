//! UI components
//!
//! Reusable UI components for the application.

// Temporarily disabled - these components have compilation errors and are not used
// pub mod button;
// pub mod card;
pub mod divider;
// pub mod label;
// pub mod panel;
pub mod text_input;
pub mod simple_text_input;  // New simplified version
pub mod enhanced_text_input;
pub mod zed_style_text_input;
pub mod ime_text_input;

// Re-export for convenience
// pub use button::{Button, ButtonColor};
// pub use card::{Card, CardStyle};
pub use divider::{Divider, DividerOrientation};
// pub use label::{Label, LabelColor, LabelSize};
// pub use panel::{Panel, PanelStyle};
pub use text_input::{TextInputBuilder, TextInputValidation};
pub use simple_text_input::SimpleTextInputBuilder;  // Simple version, no internal event handling
pub use enhanced_text_input::{EnhancedTextInputBuilder, EnhancedTextInputState, TextSelection};
pub use zed_style_text_input::{ZedStyleTextInputBuilder, ZedStyleTextInputState};
pub use ime_text_input::{ImeTextInputState};

use crate::app::AppView;
use crate::CanViewApp;
use gpui::prelude::*;
use gpui::*;

/// View button component for navigation
pub fn render_view_button(
    label: &str,
    view: AppView,
    current_view: AppView,
    cx: &mut Context<CanViewApp>,
) -> impl IntoElement {
    let is_active = current_view == view;
    let view_clone = view;
    let label = label.to_string();

    div()
        .px_4()
        .py_2()
        .rounded(px(4.))
        .cursor_pointer()
        .when(is_active, |el| el.bg(rgb(0x3b82f6)))
        .when(!is_active, |el| el.hover(|style| style.bg(rgb(0x374151))))
        .child(
            div()
                .text_sm()
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgb(0xffffff))
                .child(label),
        )
        .on_mouse_down(gpui::MouseButton::Left, {
            let view = cx.entity().clone();
            move |_event, _window, cx| {
                view.update(cx, |this, cx| {
                    this.current_view = view_clone;
                    cx.notify();
                });
            }
        })
}
