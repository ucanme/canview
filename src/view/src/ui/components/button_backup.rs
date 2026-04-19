//! Button component
//!
//! A simple, reusable button component with multiple variants.

use gpui::{prelude::*, *};

/// Button size variant
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ButtonSize {
    Small,
    Medium,
    Large,
}

/// Button visual variant
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Ghost,
    Danger,
}

/// Simple button component
pub struct Button {
    label: String,
    size: ButtonSize,
    variant: ButtonVariant,
    disabled: bool,
    active: bool,
}

impl Button {
    /// Create a new button
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            size: ButtonSize::Medium,
            variant: ButtonVariant::Secondary,
            disabled: false,
            active: false,
        }
    }

    /// Set button size
    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    /// Set button variant
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Set disabled state
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set active state (for tabs)
    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    /// Build the button - returns the div for further chaining
    pub fn build(self) -> Div {
        let label = self.label;
        let disabled = self.disabled;
        let active = self.active;

        // Get styling based on size
        let (height, padding_x, padding_y) = match self.size {
            ButtonSize::Small => (px(24.), px(12.), px(4.)),
            ButtonSize::Medium => (px(32.), px(16.), px(8.)),
            ButtonSize::Large => (px(40.), px(20.), px(12.)),
        };

        // Get colors based on variant
        let (bg, text_color, hover_bg) = match self.variant {
            ButtonVariant::Primary => (Some(rgb(0x89b4fa)), rgb(0x1e1e2e), rgb(0x74c7ec)),
            ButtonVariant::Secondary => (Some(rgb(0x45475a)), rgb(0xcdd6f4), rgb(0x585b70)),
            ButtonVariant::Ghost => (None, rgb(0xcdd6f4), rgb(0x313244)),
            ButtonVariant::Danger => (Some(rgb(0xf38ba8)), rgb(0x1e1e2e), rgb(0xba2b59)),
        };

        // Active state
        let (bg, text_color) = if active {
            (Some(rgb(0x89b4fa)), rgb(0x1e1e2e))
        } else {
            (bg, text_color)
        };

        // Build button
        let mut button = div()
            .px(padding_x)
            .py(padding_y)
            .h(height)
            .min_w(px(80.))
            .flex()
            .items_center()
            .justify_center()
            .rounded(px(4.))
            .text_sm()
            .font_weight(FontWeight::MEDIUM)
            .text_color(text_color)
            .cursor_pointer()
            .when(disabled, |el| el.opacity(0.5).cursor_default());

        // Apply background
        if let Some(bg) = bg {
            button = button.bg(bg);
        }

        // Apply hover effect
        if !disabled && !active {
            button = button.hover(|style| style.bg(hover_bg));
        }

        // Add label
        button.child(div().child(label))
    }
}

/// Convenience functions
pub fn primary_button(label: impl Into<String>) -> Button {
    Button::new(label).variant(ButtonVariant::Primary)
}

pub fn secondary_button(label: impl Into<String>) -> Button {
    Button::new(label).variant(ButtonVariant::Secondary)
}

pub fn ghost_button(label: impl Into<String>) -> Button {
    Button::new(label).variant(ButtonVariant::Ghost)
}

pub fn danger_button(label: impl Into<String>) -> Button {
    Button::new(label).variant(ButtonVariant::Danger)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_button_creation() {
        let button = Button::new("Test");
        assert_eq!(button.label, "Test");
        assert_eq!(button.size, ButtonSize::Medium);
        assert_eq!(button.variant, ButtonVariant::Secondary);
        assert!(!button.disabled);
        assert!(!button.active);
    }

    #[test]
    fn test_button_builder() {
        let button = Button::new("Test")
            .size(ButtonSize::Small)
            .variant(ButtonVariant::Primary)
            .disabled(true)
            .active(true);

        assert_eq!(button.size, ButtonSize::Small);
        assert_eq!(button.variant, ButtonVariant::Primary);
        assert!(button.disabled);
        assert!(button.active);
    }

    #[test]
    fn test_convenience_functions() {
        let primary = primary_button("Primary");
        assert_eq!(primary.variant, ButtonVariant::Primary);

        let secondary = secondary_button("Secondary");
        assert_eq!(secondary.variant, ButtonVariant::Secondary);

        let ghost = ghost_button("Ghost");
        assert_eq!(ghost.variant, ButtonVariant::Ghost);

        let danger = danger_button("Danger");
        assert_eq!(danger.variant, ButtonVariant::Danger);
    }
}
