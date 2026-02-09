//! Badge component
//!
//! A reusable badge component for status indicators, labels, and tags.

use gpui::{prelude::*, *};

/// Badge size variant
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BadgeSize {
    Small,
    Medium,
    Large,
}

impl BadgeSize {
    pub fn font_size(&self) -> Pixels {
        match self {
            BadgeSize::Small => px(10.0),
            BadgeSize::Medium => px(12.0),
            BadgeSize::Large => px(14.0),
        }
    }

    pub fn padding(&self) -> (Pixels, Pixels) {
        match self {
            BadgeSize::Small => (px(4.0), px(8.0)),
            BadgeSize::Medium => (px(6.0), px(10.0)),
            BadgeSize::Large => (px(8.0), px(12.0)),
        }
    }

    pub fn border_radius(&self) -> Pixels {
        match self {
            BadgeSize::Small => px(4.0),
            BadgeSize::Medium => px(6.0),
            BadgeSize::Large => px(8.0),
        }
    }
}

/// Badge variant/color scheme
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BadgeVariant {
    Gray,
    Blue,
    Green,
    Yellow,
    Red,
    Purple,
    Pink,
    Cyan,
    Orange,
}

impl BadgeVariant {
    pub fn bg_color(&self) -> Rgba {
        match self {
            BadgeVariant::Gray => rgb(0x6b7280),
            BadgeVariant::Blue => rgb(0x3b82f6),
            BadgeVariant::Green => rgb(0x10b981),
            BadgeVariant::Yellow => rgb(0xf59e0b),
            BadgeVariant::Red => rgb(0xef4444),
            BadgeVariant::Purple => rgb(0x8b5cf6),
            BadgeVariant::Pink => rgb(0xec4899),
            BadgeVariant::Cyan => rgb(0x06b6d4),
            BadgeVariant::Orange => rgb(0xf97316),
        }
    }

    pub fn text_color(&self) -> Rgba {
        match self {
            BadgeVariant::Gray => rgb(0x9ca3af),
            BadgeVariant::Blue => rgb(0xffffff),
            BadgeVariant::Green => rgb(0xffffff),
            BadgeVariant::Yellow => rgb(0x000000),
            BadgeVariant::Red => rgb(0xffffff),
            BadgeVariant::Purple => rgb(0xffffff),
            BadgeVariant::Pink => rgb(0xffffff),
            BadgeVariant::Cyan => rgb(0x000000),
            BadgeVariant::Orange => rgb(0x000000),
        }
    }
}

impl Default for BadgeVariant {
    fn default() -> Self {
        Self::Gray
    }
}

/// Badge component
pub struct Badge {
    label: String,
    size: BadgeSize,
    variant: BadgeVariant,
}

impl Badge {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            size: BadgeSize::Medium,
            variant: BadgeVariant::default(),
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    pub fn size(mut self, size: BadgeSize) -> Self {
        self.size = size;
        self
    }

    pub fn variant(mut self, variant: BadgeVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn build(self) -> Div {
        let (padding_x, padding_y) = self.size.padding();
        let border_radius = self.size.border_radius();
        let bg_color = self.variant.bg_color();
        let text_color = self.variant.text_color();

        div()
            .flex()
            .items_center()
            .px(padding_x)
            .py(padding_y)
            .bg(bg_color)
            .rounded(border_radius)
            .child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(text_color)
                    .child(self.label),
            )
    }
}

impl Default for Badge {
    fn default() -> Self {
        Self::new("")
    }
}

// Convenience functions

pub fn gray_badge(label: impl Into<String>) -> Badge {
    Badge::new(label).variant(BadgeVariant::Gray)
}

pub fn blue_badge(label: impl Into<String>) -> Badge {
    Badge::new(label).variant(BadgeVariant::Blue)
}

pub fn green_badge(label: impl Into<String>) -> Badge {
    Badge::new(label).variant(BadgeVariant::Green)
}

pub fn yellow_badge(label: impl Into<String>) -> Badge {
    Badge::new(label).variant(BadgeVariant::Yellow)
}

pub fn red_badge(label: impl Into<String>) -> Badge {
    Badge::new(label).variant(BadgeVariant::Red)
}

pub fn purple_badge(label: impl Into<String>) -> Badge {
    Badge::new(label).variant(BadgeVariant::Purple)
}

pub fn status_badge(status: &str) -> Badge {
    let variant = match status.to_lowercase().as_str() {
        "success" | "ok" | "done" | "completed" => BadgeVariant::Green,
        "error" | "fail" | "failed" => BadgeVariant::Red,
        "warning" | "warn" => BadgeVariant::Yellow,
        "info" => BadgeVariant::Blue,
        _ => BadgeVariant::Gray,
    };
    Badge::new(status).variant(variant)
}

pub fn count_badge(count: usize) -> Badge {
    Badge::new(count.to_string())
        .variant(BadgeVariant::Gray)
        .size(BadgeSize::Small)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_badge_creation() {
        let badge = Badge::new("Test");
        assert_eq!(badge.label, "Test");
        assert_eq!(badge.size, BadgeSize::Medium);
        assert_eq!(badge.variant, BadgeVariant::Gray);
    }

    #[test]
    fn test_badge_builder() {
        let badge = Badge::new("Test")
            .size(BadgeSize::Small)
            .variant(BadgeVariant::Green);

        assert_eq!(badge.label, "Test");
        assert_eq!(badge.size, BadgeSize::Small);
        assert_eq!(badge.variant, BadgeVariant::Green);
    }

    #[test]
    fn test_badge_sizes() {
        assert_eq!(BadgeSize::Small.font_size(), px(10.0));
        assert_eq!(BadgeSize::Small.padding(), (px(4.0), px(8.0)));
        assert_eq!(BadgeSize::Large.font_size(), px(14.0));
        assert_eq!(BadgeSize::Large.padding(), (px(8.0), px(12.0)));
    }

    #[test]
    fn test_badge_variants() {
        assert_eq!(BadgeVariant::Gray.bg_color(), rgb(0x6b7280));
        assert_eq!(BadgeVariant::Blue.bg_color(), rgb(0x3b82f6));
        assert_eq!(BadgeVariant::Green.bg_color(), rgb(0x10b981));
        assert_eq!(BadgeVariant::Red.bg_color(), rgb(0xef4444));
    }

    #[test]
    fn test_convenience_functions() {
        assert_eq!(gray_badge("Gray").variant, BadgeVariant::Gray);
        assert_eq!(blue_badge("Blue").variant, BadgeVariant::Blue);
        assert_eq!(green_badge("Green").variant, BadgeVariant::Green);
        assert_eq!(yellow_badge("Yellow").variant, BadgeVariant::Yellow);
        assert_eq!(red_badge("Red").variant, BadgeVariant::Red);
    }

    #[test]
    fn test_status_badge() {
        assert_eq!(status_badge("success").variant, BadgeVariant::Green);
        assert_eq!(status_badge("error").variant, BadgeVariant::Red);
        assert_eq!(status_badge("warning").variant, BadgeVariant::Yellow);
        assert_eq!(status_badge("info").variant, BadgeVariant::Blue);
        assert_eq!(status_badge("unknown").variant, BadgeVariant::Gray);
    }

    #[test]
    fn test_count_badge() {
        let badge = count_badge(42);
        assert_eq!(badge.label, "42");
        assert_eq!(badge.size, BadgeSize::Small);
    }
}
