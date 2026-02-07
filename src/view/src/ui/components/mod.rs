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

// Re-exports for working components
pub use button::{Button, ButtonSize, ButtonVariant};
pub use button::{danger_button, ghost_button, primary_button, secondary_button};
pub use dropdown::{Dropdown, DropdownItem, simple_dropdown};
pub use scrollbar::{Scrollbar, ScrollbarConfig, vertical_scrollbar};
pub use simple_text_input::SimpleTextInputBuilder;
pub use text_input::{TextInputBuilder, TextInputValidation};
pub use zed_style_text_input::{ZedStyleTextInputBuilder, ZedStyleTextInputState};
