//! Reusable Button component
//!
//! Provides consistent button styling and behavior.

use gpui::{prelude::*, *};

/// Button color variants
#[derive(Clone, Copy)]
pub enum ButtonColor {
    Primary,   // Blue
    Secondary, // Gray
    Danger,    // Red
    Ghost,     // Transparent background
}

impl ButtonColor {
    fn bg(&self) -> u32 {
        match self {
            ButtonColor::Primary => 0x89b4fa,
            ButtonColor::Secondary => 0x2a2a2a,
            ButtonColor::Danger => 0xf38ba8,
            ButtonColor::Ghost => 0x000000,
        }
    }

    fn text(&self) -> u32 {
        match self {
            ButtonColor::Primary => 0x1a1a1a,
            ButtonColor::Secondary => 0xcdd6f4,
            ButtonColor::Danger => 0x1a1a1a,
            ButtonColor::Ghost => 0x89b4fa,
        }
    }

    fn hover_bg(&self) -> u32 {
        match self {
            ButtonColor::Primary => 0x6a9cda,
            ButtonColor::Secondary => 0x3a3a3a,
            ButtonColor::Danger => 0xe78aa7,
            ButtonColor::Ghost => 0x000000,
        }
    }

    fn hover_text(&self) -> u32 {
        match self {
            ButtonColor::Primary => 0x1a1a1a,
            ButtonColor::Secondary => 0xcdd6f4,
            ButtonColor::Danger => 0x1a1a1a,
            ButtonColor::Ghost => 0x74c7ec,
        }
    }
}

/// Simple button component builder
pub struct Button {
    label: String,
    color: ButtonColor,
}

impl Button {
    /// Create a new button with the given label
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            color: ButtonColor::Secondary,
        }
    }

    /// Set button color to primary (blue)
    pub fn primary(mut self) -> Self {
        self.color = ButtonColor::Primary;
        self
    }

    /// Set button color to danger (red)
    pub fn danger(mut self) -> Self {
        self.color = ButtonColor::Danger;
        self
    }

    /// Set button color to ghost (transparent)
    pub fn ghost(mut self) -> Self {
        self.color = ButtonColor::Ghost;
        self
    }

    /// Build the button element with an on-click handler
    pub fn build(
        self,
        on_click: impl FnMut(&MouseEvent, &mut Window, &mut Context<gpui::Entity<gpui::Any>>) + 'static,
    ) -> impl IntoElement {
        let bg = self.color.bg();
        let text = self.color.text();
        let hover_bg = self.color.hover_bg();
        let hover_text = self.color.hover_text();
        let label = self.label;

        div()
            .px_3()
            .py_1()
            .rounded(px(2.))
            .cursor_pointer()
            .bg(rgb(bg))
            .text_color(rgb(text))
            .hover(move |style| {
                style.bg(rgb(hover_bg)).text_color(rgb(hover_text))
            })
            .on_mouse_down(gpui::MouseButton::Left, on_click)
            .child(div().text_sm().text_color(rgb(text)).child(label))
    }
}
