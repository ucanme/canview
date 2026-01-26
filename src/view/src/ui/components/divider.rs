//! Divider component
//!
//! Provides horizontal and vertical separator lines.

use gpui::{prelude::*, *};

/// Divider orientation
#[derive(Clone, Copy)]
pub enum DividerOrientation {
    Horizontal,
    Vertical,
}

/// Divider component
pub struct Divider {
    orientation: DividerOrientation,
    color: u32,
    thickness: f32,
}

impl Divider {
    /// Create a new horizontal divider
    pub fn horizontal() -> Self {
        Self {
            orientation: DividerOrientation::Horizontal,
            color: 0x313131,
            thickness: 1.0,
        }
    }

    /// Create a new vertical divider
    pub fn vertical() -> Self {
        Self {
            orientation: DividerOrientation::Vertical,
            color: 0x313131,
            thickness: 1.0,
        }
    }

    /// Set divider color
    pub fn color(mut self, color: u32) -> Self {
        self.color = color;
        self
    }

    /// Set thickness in pixels
    pub fn thickness(mut self, thickness: f32) -> Self {
        self.thickness = thickness;
        self
    }

    /// Build the divider element
    pub fn build(self) -> impl IntoElement {
        match self.orientation {
            DividerOrientation::Horizontal => {
                div().h(px(self.thickness)).w_full().bg(rgb(self.color))
            }
            DividerOrientation::Vertical => {
                div().w(px(self.thickness)).h_full().bg(rgb(self.color))
            }
        }
    }
}

impl Default for Divider {
    fn default() -> Self {
        Self::horizontal()
    }
}
