//! Panel component
//!
//! Provides a styled container for grouping related content.

use gpui::{prelude::*, *};

/// Panel style variants
#[derive(Clone, Copy)]
pub enum PanelStyle {
    Default,   // Dark panel (0x0c0c0e)
    Elevated,  // Lighter panel (0x1a1a1a)
    Subtle,    // Very subtle panel (0x1e1e2e)
}

impl PanelStyle {
    fn bg(&self) -> u32 {
        match self {
            PanelStyle::Default => 0x0c0c0e,
            PanelStyle::Elevated => 0x1a1a1a,
            PanelStyle::Subtle => 0x1e1e2e,
        }
    }
}

/// Panel component builder
pub struct Panel {
    style: PanelStyle,
    padding: Option<(f32, f32)>, // (horizontal, vertical)
    rounded: bool,
}

impl Panel {
    /// Create a new panel with default styling
    pub fn new() -> Self {
        Self {
            style: PanelStyle::Default,
            padding: None,
            rounded: true,
        }
    }

    /// Set panel style
    pub fn style(mut self, style: PanelStyle) -> Self {
        self.style = style;
        self
    }

    /// Set padding (horizontal, vertical) in pixels
    pub fn padding(mut self, horizontal: f32, vertical: f32) -> Self {
        self.padding = Some((horizontal, vertical));
        self
    }

    /// Disable rounded corners
    pub fn no_round(mut self) -> Self {
        self.rounded = false;
        self
    }

    /// Build the panel element
    pub fn build(self) -> Div {
        let bg = self.style.bg();
        let mut panel = div().bg(rgb(bg));

        // Apply padding
        if let Some((h, v)) = self.padding {
            panel = panel.px(h).py(v);
        } else {
            panel = panel.px_4().py_3();
        }

        // Apply rounded corners
        if self.rounded {
            panel = panel.rounded(px(6.));
        }

        panel
    }
}

impl Default for Panel {
    fn default() -> Self {
        Self::new()
    }
}
