//! Modal component usage examples
//!
//! This file demonstrates various ways to use the Modal component.

use crate::ui::components::modal::*;
use gpui::{prelude::*, *};

/// Example 1: Simple info modal
///
/// This is the simplest way to show a modal with just a title and text content.
pub fn example_simple_info_modal() -> Div {
    info_modal("Information")
        .size(ModalSize::Medium)
        .build_simple("This is an informational message.")
}

/// Example 2: Warning modal
///
/// Shows a warning modal with custom styling.
pub fn example_warning_modal() -> Div {
    warning_modal("Warning")
        .size(ModalSize::Small)
        .build_simple("Are you sure you want to proceed?")
}

/// Example 3: Error modal
///
/// Shows an error modal for critical issues.
pub fn example_error_modal() -> Div {
    error_modal("Error")
        .size(ModalSize::Medium)
        .build_simple("Failed to load configuration file.")
}

/// Example 4: Success modal
///
/// Shows a success modal after completing an operation.
pub fn example_success_modal() -> Div {
    success_modal("Success")
        .size(ModalSize::Small)
        .build_simple("Your changes have been saved successfully.")
}

/// Example 5: Custom content modal
///
/// Shows a modal with custom rich content including action buttons.
pub fn example_custom_content_modal() -> Div {
    Modal::new("Confirm Action")
        .size(ModalSize::Medium)
        .variant(ModalType::Warning)
        .build(
            div()
                .flex()
                .flex_col()
                .gap_3()
                .child(
                    div()
                        .text_color(rgb(0xcdd6f4))
                        .child("Do you want to delete this item? This action cannot be undone."),
                )
                .child(
                    div()
                        .flex()
                        .gap_2()
                        .justify_end()
                        .child(
                            div()
                                .px_4()
                                .py_2()
                                .bg(rgb(0x45475a))
                                .rounded(px(4.0))
                                .cursor_pointer()
                                .child("Cancel"),
                        )
                        .child(
                            div()
                                .px_4()
                                .py_2()
                                .bg(rgb(0xf38ba8))
                                .rounded(px(4.0))
                                .cursor_pointer()
                                .child("Delete"),
                        ),
                ),
        )
}

/// Example 6: Large modal for complex forms
///
/// Shows a large modal suitable for forms with many fields.
pub fn example_form_modal() -> Div {
    Modal::new("Add New Channel")
        .size(ModalSize::Large)
        .variant(ModalType::Info)
        .build(
            div()
                .flex()
                .flex_col()
                .gap_4()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(rgb(0x9399b2))
                                .child("Channel ID"),
                        )
                        .child(
                            div()
                                .w_full()
                                .px_3()
                                .py_2()
                                .bg(rgb(0x1e1e2e))
                                .border_1()
                                .border_color(rgb(0x45475a))
                                .rounded(px(4.0))
                                .child("1"),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(rgb(0x9399b2))
                                .child("Channel Name"),
                        )
                        .child(
                            div()
                                .w_full()
                                .px_3()
                                .py_2()
                                .bg(rgb(0x1e1e2e))
                                .border_1()
                                .border_color(rgb(0x45475a))
                                .rounded(px(4.0))
                                .child("Engine CAN"),
                        ),
                )
                .child(
                    div().flex().gap_2().justify_end().child(
                        div()
                            .px_4()
                            .py_2()
                            .bg(rgb(0x89b4fa))
                            .rounded(px(4.0))
                            .cursor_pointer()
                            .child("Save"),
                    ),
                ),
        )
}

/// Example 7: Modal without backdrop
///
/// Shows a modal without the semi-transparent backdrop.
pub fn example_no_backdrop_modal() -> Div {
    info_modal("No Backdrop")
        .size(ModalSize::Medium)
        .show_backdrop(false)
        .build_simple("This modal has no backdrop.")
}

/// Example 8: Modal without close button
///
/// Shows a modal without the close button in the header (useful for loading states).
pub fn example_no_close_button_modal() -> Div {
    info_modal("Processing...")
        .size(ModalSize::Small)
        .show_close_button(false)
        .build_simple("Please wait while we process your request.")
}

/// Example 9: Small modal for quick messages
///
/// Shows a small modal for brief notifications.
pub fn example_small_modal() -> Div {
    success_modal("Done!")
        .size(ModalSize::Small)
        .build_simple("Operation completed successfully.")
}

/// Example 10: Large modal for detailed content
///
/// Shows a large modal for displaying detailed information.
pub fn example_large_content_modal() -> Div {
    Modal::new("Data Export Details")
        .size(ModalSize::Large)
        .variant(ModalType::Info)
        .build(
            div()
                .flex()
                .flex_col()
                .gap_4()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(rgb(0x9399b2))
                                .child("Export Summary"),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(rgb(0xcdd6f4))
                                .child("Total messages: 15,432"),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(rgb(0xcdd6f4))
                                .child("Date range: 2025-01-01 to 2025-01-19"),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(rgb(0xcdd6f4))
                                .child("File size: 2.4 MB"),
                        ),
                )
                .child(
                    div().flex().gap_2().justify_end().child(
                        div()
                            .px_4()
                            .py_2()
                            .bg(rgb(0x89b4fa))
                            .rounded(px(4.0))
                            .cursor_pointer()
                            .child("Close"),
                    ),
                ),
        )
}

/// Example 11: Integration with app state
///
/// Shows how to integrate a modal with application state.
pub struct ExampleModalState {
    pub show_modal: bool,
    pub modal_type: Option<ModalType>,
    pub modal_title: String,
    pub modal_content: String,
}

impl ExampleModalState {
    pub fn new() -> Self {
        Self {
            show_modal: false,
            modal_type: None,
            modal_title: String::new(),
            modal_content: String::new(),
        }
    }

    pub fn show_info(&mut self, title: impl Into<String>, content: impl Into<String>) {
        self.show_modal = true;
        self.modal_type = Some(ModalType::Info);
        self.modal_title = title.into();
        self.modal_content = content.into();
    }

    pub fn show_warning(&mut self, title: impl Into<String>, content: impl Into<String>) {
        self.show_modal = true;
        self.modal_type = Some(ModalType::Warning);
        self.modal_title = title.into();
        self.modal_content = content.into();
    }

    pub fn show_error(&mut self, title: impl Into<String>, content: impl Into<String>) {
        self.show_modal = true;
        self.modal_type = Some(ModalType::Error);
        self.modal_title = title.into();
        self.modal_content = content.into();
    }

    pub fn show_success(&mut self, title: impl Into<String>, content: impl Into<String>) {
        self.show_modal = true;
        self.modal_type = Some(ModalType::Success);
        self.modal_title = title.into();
        self.modal_content = content.into();
    }

    pub fn close(&mut self) {
        self.show_modal = false;
        self.modal_type = None;
    }

    pub fn render(&self) -> Div {
        if !self.show_modal {
            return div();
        }

        let modal_type = self.modal_type.unwrap_or(ModalType::Info);
        let title = self.modal_title.clone();
        let content = self.modal_content.clone();

        Modal::new(title)
            .size(ModalSize::Medium)
            .variant(modal_type)
            .build_simple(content)
    }
}

impl Default for ExampleModalState {
    fn default() -> Self {
        Self::new()
    }
}

/// Example 12: Confirmation dialog state
///
/// A common pattern for confirming destructive actions.
pub struct ConfirmDialogState {
    pub show: bool,
    pub title: String,
    pub message: String,
}

impl ConfirmDialogState {
    pub fn new() -> Self {
        Self {
            show: false,
            title: String::new(),
            message: String::new(),
        }
    }

    pub fn confirm(&mut self, title: impl Into<String>, message: impl Into<String>) {
        self.show = true;
        self.title = title.into();
        self.message = message.into();
    }

    pub fn close(&mut self) {
        self.show = false;
    }

    pub fn is_visible(&self) -> bool {
        self.show
    }

    pub fn render(&self) -> Div {
        if !self.show {
            return div();
        }

        let title = self.title.clone();
        let message = self.message.clone();

        Modal::new(title)
            .size(ModalSize::Medium)
            .variant(ModalType::Error)
            .build(
                div()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(div().text_color(rgb(0xcdd6f4)).child(message))
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .justify_end()
                            .child(
                                div()
                                    .px_4()
                                    .py_2()
                                    .bg(rgb(0x45475a))
                                    .rounded(px(4.0))
                                    .cursor_pointer()
                                    .child("Cancel"),
                            )
                            .child(
                                div()
                                    .px_4()
                                    .py_2()
                                    .bg(rgb(0xf38ba8))
                                    .rounded(px(4.0))
                                    .cursor_pointer()
                                    .child("Confirm"),
                            ),
                    ),
            )
    }
}

impl Default for ConfirmDialogState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modal_state_info() {
        let mut state = ExampleModalState::new();
        state.show_info("Test", "Content");

        assert!(state.show_modal);
        assert_eq!(state.modal_type, Some(ModalType::Info));
        assert_eq!(state.modal_title, "Test");
        assert_eq!(state.modal_content, "Content");
    }

    #[test]
    fn test_modal_state_warning() {
        let mut state = ExampleModalState::new();
        state.show_warning("Warning", "Content");

        assert!(state.show_modal);
        assert_eq!(state.modal_type, Some(ModalType::Warning));
    }

    #[test]
    fn test_modal_state_error() {
        let mut state = ExampleModalState::new();
        state.show_error("Error", "Content");

        assert!(state.show_modal);
        assert_eq!(state.modal_type, Some(ModalType::Error));
    }

    #[test]
    fn test_modal_state_success() {
        let mut state = ExampleModalState::new();
        state.show_success("Success", "Content");

        assert!(state.show_modal);
        assert_eq!(state.modal_type, Some(ModalType::Success));
    }

    #[test]
    fn test_modal_state_close() {
        let mut state = ExampleModalState::new();
        state.show_info("Test", "Content");
        assert!(state.show_modal);

        state.close();
        assert!(!state.show_modal);
        assert!(state.modal_type.is_none());
    }

    #[test]
    fn test_confirm_dialog_state() {
        let mut state = ConfirmDialogState::new();
        assert!(!state.show);

        state.confirm("Delete Item", "Are you sure?");
        assert!(state.show);
        assert_eq!(state.title, "Delete Item");
        assert_eq!(state.message, "Are you sure?");
        assert!(state.is_visible());

        state.close();
        assert!(!state.show);
        assert!(!state.is_visible());
    }

    #[test]
    fn test_modal_builder_patterns() {
        let modal = Modal::new("Test")
            .size(ModalSize::Large)
            .variant(ModalType::Warning)
            .show_close_button(true)
            .show_backdrop(true);

        assert_eq!(modal.title, "Test");
        assert_eq!(modal.config.size, ModalSize::Large);
        assert_eq!(modal.config.modal_type, ModalType::Warning);
        assert!(modal.config.show_close_button);
        assert!(modal.config.show_backdrop);
    }

    #[test]
    fn test_modal_type_convenience_functions() {
        let info = info_modal("Info");
        assert_eq!(info.config.modal_type, ModalType::Info);

        let warning = warning_modal("Warning");
        assert_eq!(warning.config.modal_type, ModalType::Warning);

        let error = error_modal("Error");
        assert_eq!(error.config.modal_type, ModalType::Error);

        let success = success_modal("Success");
        assert_eq!(success.config.modal_type, ModalType::Success);
    }
}
