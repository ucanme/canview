//! Tooltip component
//!
//! A reusable tooltip component for displaying contextual information on hover.

use gpui::{prelude::*, *};

/// Tooltip position
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TooltipPosition {
    Top,
    Bottom,
    Left,
    Right,
    Auto,
}

/// Tooltip arrow style
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TooltipArrow {
    None,
    Top,
    Bottom,
    Left,
    Right,
}

/// Tooltip configuration
#[derive(Clone, Debug)]
pub struct TooltipConfig {
    pub position: TooltipPosition,
    pub arrow: TooltipArrow,
    pub max_width: Pixels,
    pub show_delay_ms: u64,
    pub hide_delay_ms: u64,
    pub background_color: Rgba,
    pub text_color: Rgba,
}

impl Default for TooltipConfig {
    fn default() -> Self {
        Self {
            position: TooltipPosition::Auto,
            arrow: TooltipArrow::None,
            max_width: px(300.),
            show_delay_ms: 500,
            hide_delay_ms: 100,
            background_color: rgb(0x1e1e2e),
            text_color: rgb(0xcdd6f4),
        }
    }
}

/// Tooltip component
pub struct Tooltip {
    content: String,
    config: TooltipConfig,
}

impl Tooltip {
    /// Create a new tooltip
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            config: TooltipConfig::default(),
        }
    }

    /// Set the tooltip content
    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.content = content.into();
        self
    }

    /// Set the tooltip position
    pub fn position(mut self, position: TooltipPosition) -> Self {
        self.config.position = position;
        self
    }

    /// Set the tooltip arrow style
    pub fn arrow(mut self, arrow: TooltipArrow) -> Self {
        self.config.arrow = arrow;
        self
    }

    /// Set the max width
    pub fn max_width(mut self, width: Pixels) -> Self {
        self.config.max_width = width;
        self
    }

    /// Set the show delay (milliseconds)
    pub fn show_delay(mut self, delay_ms: u64) -> Self {
        self.config.show_delay_ms = delay_ms;
        self
    }

    /// Set the hide delay (milliseconds)
    pub fn hide_delay(mut self, delay_ms: u64) -> Self {
        self.config.hide_delay_ms = delay_ms;
        self
    }

    /// Set the background color
    pub fn background_color(mut self, color: Rgba) -> Self {
        self.config.background_color = color;
        self
    }

    /// Set the text color
    pub fn text_color(mut self, color: Rgba) -> Self {
        self.config.text_color = color;
        self
    }

    /// Set the tooltip configuration
    pub fn config(mut self, config: TooltipConfig) -> Self {
        self.config = config;
        self
    }

    /// Build the tooltip element
    pub fn build(self) -> Div {
        let config = self.config;

        div()
            .max_w(config.max_width)
            .px(px(4.))
            .py(px(8.))
            .bg(config.background_color)
            .border_1()
            .border_color(rgb(0x313244))
            .rounded(px(4.0))
            .shadow_xl()
            .child(
                div()
                    .text_sm()
                    .text_color(config.text_color)
                    .child(self.content),
            )
    }
}

impl Default for Tooltip {
    fn default() -> Self {
        Self::new("")
    }
}

/// Create a simple info tooltip
pub fn info_tooltip(content: impl Into<String>) -> Tooltip {
    Tooltip::new(content)
}

/// Create a simple tooltip with custom position
pub fn tooltip_with_position(content: impl Into<String>, position: TooltipPosition) -> Tooltip {
    Tooltip::new(content).position(position)
}

/// Create a tooltip with custom max width
pub fn wide_tooltip(content: impl Into<String>, max_width: Pixels) -> Tooltip {
    Tooltip::new(content).max_width(max_width)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tooltip_creation() {
        let tooltip = Tooltip::new("Test tooltip");
        assert_eq!(tooltip.content, "Test tooltip");
        assert_eq!(tooltip.config.position, TooltipPosition::Auto);
        assert!(!tooltip.config.show_delay_ms == 0);
    }

    #[test]
    fn test_tooltip_builder() {
        let tooltip = Tooltip::new("Test")
            .position(TooltipPosition::Top)
            .max_width(px(500.0))
            .show_delay_ms(1000)
            .hide_delay_ms(200);

        assert_eq!(tooltip.config.position, TooltipPosition::Top);
        assert_eq!(tooltip.config.max_width, px(500.0));
        assert_eq!(tooltip.config.show_delay_ms, 1000);
        assert_eq!(tooltip.config.hide_delay_ms, 200);
    }

    #[test]
    fn test_tooltip_config_default() {
        let config = TooltipConfig::default();
        assert_eq!(config.position, TooltipPosition::Auto);
        assert_eq!(config.arrow, TooltipArrow::None);
        assert_eq!(config.max_width, px(300.));
        assert_eq!(config.show_delay_ms, 500);
        assert_eq!(config.hide_delay_ms, 100);
    }

    #[test]
    fn test_convenience_functions() {
        let info = info_tooltip("Info");
        assert_eq!(info.content, "Info");

        let positioned = tooltip_with_position("Test", TooltipPosition::Top);
        assert_eq!(positioned.config.position, TooltipPosition::Top);

        let wide = wide_tooltip("Wide tooltip", px(500));
        assert_eq!(wide.config.max_width, px(500.0));
    }

    #[test]
    fn test_default_tooltip() {
        let tooltip = Tooltip::default();
        assert_eq!(tooltip.content, "");
        assert_eq!(tooltip.config.position, TooltipPosition::Auto);
        assert_eq!(tooltip.config.max_width, px(300.));
    }

    #[test]
    fn test_tooltip_positions() {
        assert_eq!(TooltipPosition::Top, TooltipPosition::Top);
        assert_eq!(TooltipPosition::Bottom, TooltipPosition::Bottom);
        assert_eq!(TooltipPosition::Left, TooltipPosition::Left);
        assert_eq!(TooltipPosition::Right, TooltipPosition::Right);
        assert_eq!(TooltipPosition::Auto, TooltipPosition::Auto);
    }

    #[test]
    fn test_tooltip_arrows() {
        assert_eq!(TooltipArrow::None, TooltipArrow::None);
        assert_eq!(TooltipArrow::Top, TooltipArrow::Top);
        assert_eq!(TooltipArrow::Bottom, TooltipArrow::Bottom);
        assert_eq!(TooltipArrow::Left, TooltipArrow::Left);
        assert_eq!(TooltipArrow::Right, TooltipArrow::Right);
    }
}
