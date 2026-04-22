//! UI components
//!
//! Reusable UI components for the application.

// Core working components
pub mod button;
pub mod divider;
pub mod dropdown;
pub mod scrollbar;
pub mod simple_text_input;
pub mod text_input;
pub mod zed_style_text_input;

// New Zed-style components (being developed)
// Temporarily disabled due to compilation issues
// pub mod card;
// pub mod dropdown;
// pub mod enhanced_text_input;
// pub mod ime_text_input;

// Re-exports for working components
pub use button::{Button, ButtonSize, ButtonVariant};
pub use button::{danger_button, ghost_button, primary_button, secondary_button};
pub use dropdown::{Dropdown, DropdownItem, simple_dropdown};
pub use scrollbar::{Scrollbar, ScrollbarConfig, ScrollbarDragState, vertical_scrollbar};
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
/// Now using the Button component for consistent styling
pub fn render_view_button(
    label: &str,
    view: AppView,
    current_view: AppView,
    cx: &mut Context<CanViewApp>,
) -> impl IntoElement {
    let is_active = current_view == view;
    let view_clone = view;
    let label = label.to_string();

    Button::new(label.clone())
        .size(ButtonSize::Medium)
        .variant(ButtonVariant::Ghost)
        .active(is_active)
        .build()
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
