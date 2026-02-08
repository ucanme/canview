//! Modal component
//!
//! A reusable modal dialog component for showing dialogs and prompts.

use gpui::{prelude::*, *};

/// Modal size variant
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ModalSize {
    Small,
    Medium,
    Large,
}

impl ModalSize {
    /// Get the modal width in pixels
    pub fn width(&self) -> f32 {
        match self {
            ModalSize::Small => 400.0,
            ModalSize::Medium => 600.0,
            ModalSize::Large => 800.0,
        }
    }

    /// Get the modal max height in pixels
    pub fn max_height(&self) -> f32 {
        match self {
            ModalSize::Small => 300.0,
            ModalSize::Medium => 500.0,
            ModalSize::Large => 700.0,
        }
    }
}

/// Modal type/variant
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ModalType {
    Info,
    Warning,
    Error,
    Success,
}

impl ModalType {
    /// Get the accent color for this modal type
    pub fn accent_color(&self) -> u32 {
        match self {
            ModalType::Info => 0x89b4fa,    // Blue
            ModalType::Warning => 0xf9e2af, // Yellow
            ModalType::Error => 0xf38ba8,   // Red
            ModalType::Success => 0xa6e3a1, // Green
        }
    }
}

/// Modal configuration
#[derive(Clone, Debug)]
pub struct ModalConfig {
    pub size: ModalSize,
    pub modal_type: ModalType,
    pub show_close_button: bool,
    pub show_backdrop: bool,
    pub close_on_backdrop_click: bool,
}

impl Default for ModalConfig {
    fn default() -> Self {
        Self {
            size: ModalSize::Medium,
            modal_type: ModalType::Info,
            show_close_button: true,
            show_backdrop: true,
            close_on_backdrop_click: true,
        }
    }
}

/// Modal component
pub struct Modal {
    pub title: String,
    config: ModalConfig,
}

impl Modal {
    /// Create a new modal with a title
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            config: ModalConfig::default(),
        }
    }

    /// Set the modal title
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Set the modal size
    pub fn size(mut self, size: ModalSize) -> Self {
        self.config.size = size;
        self
    }

    /// Set the modal type/variant
    pub fn variant(mut self, variant: ModalType) -> Self {
        self.config.modal_type = variant;
        self
    }

    /// Set whether to show the close button
    pub fn show_close_button(mut self, show: bool) -> Self {
        self.config.show_close_button = show;
        self
    }

    /// Set whether to show the backdrop
    pub fn show_backdrop(mut self, show: bool) -> Self {
        self.config.show_backdrop = show;
        self
    }

    /// Set whether clicking the backdrop closes the modal
    pub fn close_on_backdrop_click(mut self, close: bool) -> Self {
        self.config.close_on_backdrop_click = close;
        self
    }

    /// Build the modal with custom content
    ///
    /// # Arguments
    /// * `content` - The content div to render inside the modal
    pub fn build(self, content: Div) -> Div {
        let title = self.title;
        let config = self.config.clone();
        let accent_color = rgb(self.config.modal_type.accent_color());
        let width = self.config.size.width();
        let max_height = self.config.size.max_height();

        let modal_content = div()
            .w(px(width))
            .max_h(px(max_height))
            .bg(rgb(0x1e1e2e))
            .border_1()
            .border_color(accent_color)
            .rounded(px(8.0))
            .shadow_xl()
            .flex()
            .flex_col()
            .overflow_hidden()
            .child(
                // Header
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_4()
                    .py_3()
                    .bg(rgb(0x181825))
                    .border_b_1()
                    .border_color(rgb(0x313244))
                    .child(
                        div()
                            .text_lg()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(accent_color)
                            .child(title),
                    )
                    .when(config.show_close_button, |this| {
                        this.child(
                            div()
                                .cursor_pointer()
                                .px_2()
                                .py_1()
                                .rounded(px(4.0))
                                .hover(|style| style.bg(rgb(0x313244)))
                                .child(div().text_sm().text_color(rgb(0xcdd6f4)).child("×")),
                        )
                    }),
            )
            .child(
                // Content area
                div().flex_1().p_4().min_h(px(100.0)).child(content),
            );

        if config.show_backdrop {
            div()
                .absolute()
                .top_0()
                .left_0()
                .w_full()
                .h_full()
                .flex()
                .items_center()
                .justify_center()
                .bg(rgba(0x000000cc))
                .child(modal_content)
        } else {
            div()
                .absolute()
                .top_0()
                .left_0()
                .w_full()
                .h_full()
                .flex()
                .items_center()
                .justify_center()
                .child(modal_content)
        }
    }

    /// Build the modal with simple text content
    pub fn build_simple(self, content_text: impl Into<String>) -> Div {
        let content = content_text.into();
        self.build(div().text_sm().text_color(rgb(0xcdd6f4)).child(content))
    }
}

impl Default for Modal {
    fn default() -> Self {
        Self::new("")
    }
}

/// Convenience function to create an info modal
pub fn info_modal(title: impl Into<String>) -> Modal {
    Modal::new(title).variant(ModalType::Info)
}

/// Convenience function to create a warning modal
pub fn warning_modal(title: impl Into<String>) -> Modal {
    Modal::new(title).variant(ModalType::Warning)
}

/// Convenience function to create an error modal
pub fn error_modal(title: impl Into<String>) -> Modal {
    Modal::new(title).variant(ModalType::Error)
}

/// Convenience function to create a success modal
pub fn success_modal(title: impl Into<String>) -> Modal {
    Modal::new(title).variant(ModalType::Success)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modal_size() {
        assert_eq!(ModalSize::Small.width(), 400.0);
        assert_eq!(ModalSize::Small.max_height(), 300.0);
        assert_eq!(ModalSize::Medium.width(), 600.0);
        assert_eq!(ModalSize::Medium.max_height(), 500.0);
        assert_eq!(ModalSize::Large.width(), 800.0);
        assert_eq!(ModalSize::Large.max_height(), 700.0);
    }

    #[test]
    fn test_modal_type_colors() {
        assert_eq!(ModalType::Info.accent_color(), 0x89b4fa);
        assert_eq!(ModalType::Warning.accent_color(), 0xf9e2af);
        assert_eq!(ModalType::Error.accent_color(), 0xf38ba8);
        assert_eq!(ModalType::Success.accent_color(), 0xa6e3a1);
    }

    #[test]
    fn test_modal_config_default() {
        let config = ModalConfig::default();
        assert_eq!(config.size, ModalSize::Medium);
        assert_eq!(config.modal_type, ModalType::Info);
        assert!(config.show_close_button);
        assert!(config.show_backdrop);
        assert!(config.close_on_backdrop_click);
    }

    #[test]
    fn test_modal_builder() {
        let modal = Modal::new("Test Title")
            .size(ModalSize::Large)
            .variant(ModalType::Warning)
            .show_close_button(false)
            .show_backdrop(false)
            .close_on_backdrop_click(false);

        assert_eq!(modal.title, "Test Title");
        assert_eq!(modal.config.size, ModalSize::Large);
        assert_eq!(modal.config.modal_type, ModalType::Warning);
        assert!(!modal.config.show_close_button);
        assert!(!modal.config.show_backdrop);
        assert!(!modal.config.close_on_backdrop_click);
    }

    #[test]
    fn test_modal_default() {
        let modal = Modal::default();
        assert_eq!(modal.title, "");
        assert_eq!(modal.config.size, ModalSize::Medium);
    }

    #[test]
    fn test_convenience_functions() {
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
