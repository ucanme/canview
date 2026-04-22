//! Zed-style Button component
//!
//! Provides modern, accessible button styling with smooth interactions.

use crate::ui::theme::{colors, palette, radius, spacing, typography};
use gpui::{prelude::*, *};

/// Button size variants
#[derive(Clone, Copy, PartialEq)]
pub enum ButtonSize {
    Small,
    Medium,
    Large,
}

impl ButtonSize {
    fn padding(&self) -> (gpui::Pixels, gpui::Pixels) {
        match self {
            ButtonSize::Small => (spacing::XS, spacing::SM),
            ButtonSize::Medium => (spacing::SM, spacing::MD),
            ButtonSize::Large => (spacing::MD, spacing::LG),
        }
    }

    fn font_size(&self) -> gpui::Pixels {
        match self {
            ButtonSize::Small => typography::XS,
            ButtonSize::Medium => typography::SM,
            ButtonSize::Large => typography::BASE,
        }
    }
}

/// Button color variants
#[derive(Clone, Copy)]
pub enum ButtonColor {
    Primary,   // Blue accent
    Secondary, // Gray neutral
    Danger,    // Red for destructive actions
    Ghost,     // Transparent with accent color
    Success,   // Green for positive actions
}

impl ButtonColor {
    fn bg(&self) -> Rgba {
        match self {
            ButtonColor::Primary => colors::PRIMARY,
            ButtonColor::Secondary => colors::BG_ELEVATED,
            ButtonColor::Danger => colors::ERROR,
            ButtonColor::Ghost => colors::BG_DEFAULT,
            ButtonColor::Success => colors::SUCCESS,
        }
    }

    fn text(&self) -> Rgba {
        match self {
            ButtonColor::Primary => colors::BG_DEFAULT,
            ButtonColor::Secondary => colors::TEXT_PRIMARY,
            ButtonColor::Danger => colors::BG_DEFAULT,
            ButtonColor::Ghost => colors::PRIMARY,
            ButtonColor::Success => colors::BG_DEFAULT,
        }
    }

    fn border(&self) -> Rgba {
        match self {
            ButtonColor::Ghost => colors::BORDER_DEFAULT,
            _ => colors::BG_DEFAULT, // Transparent border for solid buttons
        }
    }

    fn hover_bg(&self) -> Rgba {
        match self {
            ButtonColor::Primary => colors::PRIMARY_HOVER,
            ButtonColor::Secondary => colors::BG_ACTIVE,
            ButtonColor::Danger => colors::ERROR,
            ButtonColor::Ghost => colors::BG_ELEVATED,
            ButtonColor::Success => colors::SUCCESS,
        }
    }

    fn active_bg(&self) -> Rgba {
        match self {
            ButtonColor::Primary => colors::PRIMARY_ACTIVE,
            ButtonColor::Secondary => palette::SURFACE2,
            ButtonColor::Danger => palette::MAROON,
            ButtonColor::Ghost => colors::BG_ACTIVE,
            ButtonColor::Success => palette::TEAL,
        }
    }

    fn has_border(&self) -> bool {
        matches!(self, ButtonColor::Ghost)
    }
}

/// Modern button component builder
pub struct Button {
    label: String,
    color: ButtonColor,
    size: ButtonSize,
    disabled: bool,
    icon: Option<String>,
    icon_position: IconPosition,
}

/// Icon position within button
#[derive(Clone, Copy, PartialEq)]
pub enum IconPosition {
    Left,
    Right,
}

impl Button {
    /// Create a new button with the given label
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            color: ButtonColor::Secondary,
            size: ButtonSize::Medium,
            disabled: false,
            icon: None,
            icon_position: IconPosition::Left,
        }
    }

    /// Set button size
    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    /// Small button variant
    pub fn small(mut self) -> Self {
        self.size = ButtonSize::Small;
        self
    }

    /// Large button variant
    pub fn large(mut self) -> Self {
        self.size = ButtonSize::Large;
        self
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

    /// Set button color to success (green)
    pub fn success(mut self) -> Self {
        self.color = ButtonColor::Success;
        self
    }

    /// Disable the button
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Add an icon to the button
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Set icon position (left or right)
    pub fn icon_position(mut self, position: IconPosition) -> Self {
        self.icon_position = position;
        self
    }

    /// Build the button element with an on-click handler
    pub fn build<App>(
        self,
        on_click: impl FnMut(&MouseEvent, &mut Window, &mut Context<App>) + 'static,
    ) -> impl IntoElement
    where
        App: 'static,
    {
        let bg = self.color.bg();
        let text = self.color.text();
        let border = self.color.border();
        let hover_bg = self.color.hover_bg();
        let active_bg = self.color.active_bg();
        let label = self.label;
        let disabled = self.disabled;
        let has_border = self.color.has_border();
        let (px, py) = self.size.padding();
        let font_size = self.size.font_size();
        let icon = self.icon;
        let icon_position = self.icon_position;

        let disabled_bg = colors::BG_MUTED;
        let disabled_text = colors::DISABLED;

        div()
            .px(px)
            .py(py)
            .rounded(radius::MD)
            .cursor_pointer()
            .when(disabled, |div| div.cursor_not_allowed())
            .bg(if disabled { disabled_bg } else { bg })
            .when(has_border && !disabled, |div| {
                div.border_1().border_color(border)
            })
            .text_color(if disabled { disabled_text } else { text })
            .hover(|style| if !disabled { style.bg(hover_bg) } else { style })
            .active(|style| {
                if !disabled {
                    style.bg(active_bg)
                } else {
                    style
                }
            })
            .when(!disabled, |div| {
                div.on_mouse_down(gpui::MouseButton::Left, on_click)
            })
            .flex()
            .items_center()
            .gap(spacing::XS)
            .child({
                let mut content = div().flex().items_center().gap(spacing::XS);

                // Add icon if present
                if let Some(icon_text) = icon {
                    if icon_position == IconPosition::Left {
                        content = content.child(
                            div()
                                .text_color(if disabled { disabled_text } else { text })
                                .child(icon_text),
                        );
                    }
                }

                // Add label
                content = content.child(
                    div()
                        .font_size(font_size)
                        .text_color(if disabled { disabled_text } else { text })
                        .child(label),
                );

                // Add icon if present (right side)
                if let Some(icon_text) = icon {
                    if icon_position == IconPosition::Right {
                        content = content.child(
                            div()
                                .text_color(if disabled { disabled_text } else { text })
                                .child(icon_text),
                        );
                    }
                }

                content
            })
    }
}

impl Default for Button {
    fn default() -> Self {
        Self::new("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_button_creation() {
        let button = Button::new("Click me");
        assert_eq!(button.label, "Click me");
    }

    #[test]
    fn test_button_variants() {
        let primary = Button::new("Primary").primary();
        assert!(matches!(primary.color, ButtonColor::Primary));

        let danger = Button::new("Delete").danger();
        assert!(matches!(danger.color, ButtonColor::Danger));

        let ghost = Button::new("Cancel").ghost();
        assert!(matches!(ghost.color, ButtonColor::Ghost));
    }

    #[test]
    fn test_button_sizes() {
        let small = Button::new("Small").small();
        assert_eq!(small.size, ButtonSize::Small);

        let large = Button::new("Large").large();
        assert_eq!(large.size, ButtonSize::Large);
    }

    #[test]
    fn test_button_builder_pattern() {
        let button = Button::new("Save")
            .primary()
            .large()
            .icon("💾")
            .disabled(false);

        assert_eq!(button.label, "Save");
        assert!(matches!(button.color, ButtonColor::Primary));
        assert_eq!(button.size, ButtonSize::Large);
        assert_eq!(button.icon, Some("💾".to_string()));
        assert!(!button.disabled);
    }
}
