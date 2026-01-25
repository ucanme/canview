//! Label component
//!
//! Provides styled text labels with size and color variants.

use gpui::{prelude::*, *};

/// Label size variants
#[derive(Clone, Copy)]
pub enum LabelSize {
    XS,   // Extra small
    SM,   // Small
    Base, // Base (default)
    LG,   // Large
    XL,   // Extra large
}

/// Label color variants
#[derive(Clone, Copy)]
pub enum LabelColor {
    Default,  // #cdd6f4 (light gray)
    Muted,    // #646473 (muted gray)
    Accent,   // #89b4fa (blue)
    Success,  // #a6e3a1 (green)
    Warning,  // #f9e2af (yellow)
    Error,    // #f38ba8 (red)
}

impl LabelSize {
    fn font_size(&self) -> impl Fn(gpui::TextStyle) -> gpui::TextStyle {
        match self {
            LabelSize::XS => |style| style.text_size(px(11.)),
            LabelSize::SM => |style| style.text_size(px(13.)),
            LabelSize::Base => |style| style.text_size(px(14.)),
            LabelSize::LG => |style| style.text_size(px(16.)),
            LabelSize::XL => |style| style.text_size(px(18.)),
        }
    }
}

impl LabelColor {
    fn color(&self) -> u32 {
        match self {
            LabelColor::Default => 0xcdd6f4,
            LabelColor::Muted => 0x646473,
            LabelColor::Accent => 0x89b4fa,
            LabelColor::Success => 0xa6e3a1,
            LabelColor::Warning => 0xf9e2af,
            LabelColor::Error => 0xf38ba8,
        }
    }
}

/// Label component builder
pub struct Label {
    text: String,
    size: LabelSize,
    color: LabelColor,
    font_weight: Option<FontWeight>,
}

impl Label {
    /// Create a new label with text
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            size: LabelSize::Base,
            color: LabelColor::Default,
            font_weight: None,
        }
    }

    /// Set label size
    pub fn size(mut self, size: LabelSize) -> Self {
        self.size = size;
        self
    }

    /// Set label color
    pub fn color(mut self, color: LabelColor) -> Self {
        self.color = color;
        self
    }

    /// Set font weight
    pub fn font_weight(mut self, weight: FontWeight) -> Self {
        self.font_weight = Some(weight);
        self
    }

    /// Build the label element
    pub fn build(self) -> impl IntoElement {
        let color = self.color.color();
        let mut label = div()
            .text_color(rgb(color))
            .when(self.size == LabelSize::XS, |div| div.text_xs())
            .when(self.size == LabelSize::SM, |div| div.text_sm())
            .when(self.size == LabelSize::Base, |div| div.text_base())
            .when(self.size == LabelSize::LG, |div| div.text_lg())
            .when(self.size == LabelSize::XL, |div| div.text_xl());

        if let Some(weight) = self.font_weight {
            label = label.font_weight(weight);
        }

        label.child(self.text)
    }

    /// Convenience method for small muted text
    pub fn small_muted(text: impl Into<String>) -> impl IntoElement {
        Self::new(text).size(LabelSize::SM).color(LabelColor::Muted).build()
    }

    /// Convenience method for accent label
    pub fn accent(text: impl Into<String>) -> impl IntoElement {
        Self::new(text).color(LabelColor::Accent).build()
    }
}
