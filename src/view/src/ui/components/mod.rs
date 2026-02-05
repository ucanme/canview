//! UI components
//!
//! Reusable UI components for the application.

// Core working components
pub mod divider;
pub mod simple_text_input;
pub mod text_input;
pub mod zed_style_text_input;

// New Zed-style components (being developed)
// Temporarily disabled due to compilation issues
// pub mod button;
// pub mod card;
// pub mod dropdown;
// pub mod enhanced_text_input;
// pub mod ime_text_input;

// Re-exports for working components
pub use simple_text_input::SimpleTextInputBuilder;
pub use text_input::{TextInputBuilder, TextInputValidation};
pub use zed_style_text_input::{ZedStyleTextInputBuilder, ZedStyleTextInputState};

use crate::CanViewApp;
use crate::app::AppView;
use gpui::prelude::*;
use gpui::*;

/// View button component for navigation
///
/// This is a simple navigation button that switches between different views
/// (LogView, ConfigView, LibraryView, PlotView)
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
        .when(is_active, |el| {
            el.bg(rgb(0x89b4fa)) // Zed blue for active state
        })
        .when(!is_active, |el| {
            el.hover(|style| style.bg(rgb(0x313244))) // Catppuccin surface0 for hover
        })
        .child(
            div()
                .text_sm()
                .font_weight(FontWeight::MEDIUM)
                .when(is_active, |el| el.text_color(rgb(0x1e1e2e))) // Dark text on active
                .when(!is_active, |el| el.text_color(rgb(0xcdd6f4))) // Light text on inactive
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
