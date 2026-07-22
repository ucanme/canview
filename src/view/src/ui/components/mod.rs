//! UI components
//!
//! Reusable UI components for the application.

// Core working components
pub mod button;
pub mod divider;
pub mod dropdown;
pub mod modal;
pub mod scrollbar;
pub mod simple_text_input;
pub mod tabs;
pub mod text_input;
pub mod zed_style_text_input;
pub mod top_bar;
pub mod status_bar;
pub mod library_picker;
pub use top_bar::render_top_bar;
pub use status_bar::render_status_bar;
pub use library_picker::render_library_picker;

// Re-exports for working components
pub use button::{Button, ButtonSize, ButtonVariant};
pub use button::{danger_button, ghost_button, primary_button, secondary_button};
pub use dropdown::{Dropdown, DropdownItem, simple_dropdown};
pub use modal::{Modal, ModalConfig, ModalSize, ModalType};
pub use modal::{error_modal, info_modal, success_modal, warning_modal};
pub use scrollbar::{Scrollbar, ScrollbarConfig, vertical_scrollbar};
pub use simple_text_input::SimpleTextInputBuilder;
pub use tabs::simple_tabs;
pub use tabs::{TabAlignment, TabItem, Tabs, TabsConfig};
pub use text_input::{TextInputBuilder, TextInputValidation};
pub use zed_style_text_input::{ZedStyleTextInputBuilder, ZedStyleTextInputState};
