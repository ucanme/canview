//! Simplified Text Input Component (Visual Only)
//!
//! This component only renders the text input UI, without handling keyboard events.
//! Keyboard events should be handled by the parent component's global event handler.
//!
//! # Example
//!
//! ```rust
//! use crate::ui::components::SimpleTextInputBuilder;
//!
//! // In render function
//! let input = SimpleTextInputBuilder::new()
//!     .text(state.my_text.clone())
//!     .placeholder("Enter text...")
//!     .focused(state.is_focused)
//!     .build("my_input_id");
//!
//! // In global key_down handler
//! if let Some("my_input_id") = state.focused_input.as_deref() {
//!     match keystroke.as_str() {
//!         "backspace" => { /* handle */ }
//!         "enter" => { /* handle */ }
//!         _ => { /* handle character input */ }
//!     }
//! }
//! ```

use gpui::prelude::FluentBuilder;
use gpui::*;

/// Builder for creating simple text input elements (visual only)
pub struct SimpleTextInputBuilder {
    text: String,
    placeholder: String,
    focused: bool,
    max_width: Option<Pixels>,
    min_width: Option<Pixels>,
}

impl SimpleTextInputBuilder {
    /// Create a new text input builder
    pub fn new() -> Self {
        Self {
            text: String::new(),
            placeholder: String::new(),
            focused: false,
            max_width: None,
            min_width: None,
        }
    }

    /// Set the initial text content
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }

    /// Set the placeholder text shown when input is empty
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Set whether the input is focused
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Set maximum width
    pub fn max_width(mut self, width: Pixels) -> Self {
        self.max_width = Some(width);
        self
    }

    /// Set minimum width
    pub fn min_width(mut self, width: Pixels) -> Self {
        self.min_width = Some(width);
        self
    }

    /// Build the text input element (visual only)
    ///
    /// Note: This component does NOT handle keyboard events.
    /// The parent component must handle keyboard events in its global event handler.
    pub fn build(self, id: impl Into<String>) -> impl IntoElement {
        let text = self.text.clone();
        let placeholder = self.placeholder.clone();
        let input_id = id.into();
        let max_width = self.max_width.unwrap_or(px(200.));
        let min_width = self.min_width.unwrap_or(px(100.));

        let border_color = if self.focused {
            rgb(0x89b4fa)
        } else {
            rgb(0x2a2a2a)
        };

        let is_empty = text.trim().is_empty();
        let text_color = if is_empty {
            rgb(0x646473)
        } else {
            rgb(0xcdd6f4)
        };

        let display_text = if is_empty {
            if placeholder.is_empty() {
                String::from("Type here...")
            } else {
                placeholder
            }
        } else {
            text.clone()
        };

        div()
            .px_2()
            .py_1()
            .bg(rgb(0x1a1a1a))
            .border_1()
            .border_color(border_color)
            .rounded(px(2.))
            .flex()
            .items_center()
            .min_w(min_width)
            .max_w(max_width)
            .cursor_text()
            .id(input_id)
            .focusable()
            .relative()
            .child(
                div()
                    .text_xs()
                    .text_color(text_color)
                    .child(display_text.clone()),
            )
            // Render cursor overlay when focused
            .when(self.focused, |this_div| {
                this_div.child(
                    div()
                        .absolute()
                        .left(px(4.))
                        .top(px(2.))
                        .bottom(px(2.))
                        .w(px(1.5))
                        .bg(rgb(0xcdd6f4))
                )
            })
    }
}

impl Default for SimpleTextInputBuilder {
    fn default() -> Self {
        Self::new()
    }
}
