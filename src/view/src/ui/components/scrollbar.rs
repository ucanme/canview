//! Scrollbar component
//!
//! A simplified reusable custom scrollbar component for lists.
//! This version focuses on visual rendering without complex drag handling.

use gpui::{prelude::*, *};

/// Scrollbar configuration
#[derive(Clone, Copy)]
pub struct ScrollbarConfig {
    pub width: Pixels,
    pub thumb_width: Pixels,
    pub min_thumb_size: f32,
    pub track_color: u32,
    pub thumb_color: u32,
    pub thumb_hover_color: u32,
}

impl Default for ScrollbarConfig {
    fn default() -> Self {
        Self {
            width: px(12.),
            thumb_width: px(8.),
            min_thumb_size: 15.0,
            track_color: 0x1a1a1a,
            thumb_color: 0x6a6a6a,
            thumb_hover_color: 0x7a7a7a,
        }
    }
}

/// Scrollbar component
pub struct Scrollbar {
    total_items: usize,
    visible_items: usize,
    current_index: usize,
    row_height: f32,
    container_height: f32,
    config: ScrollbarConfig,
}

impl Scrollbar {
    /// Create a new scrollbar
    pub fn new() -> Self {
        Self {
            total_items: 0,
            visible_items: 20,
            current_index: 0,
            row_height: 20.0,
            container_height: 300.0,
            config: ScrollbarConfig::default(),
        }
    }

    /// Set the total number of items
    pub fn total_items(mut self, count: usize) -> Self {
        self.total_items = count;
        self
    }

    /// Set the number of visible items
    pub fn visible_items(mut self, count: usize) -> Self {
        self.visible_items = count;
        self
    }

    /// Set the current scroll index
    pub fn current_index(mut self, index: usize) -> Self {
        self.current_index = index;
        self
    }

    /// Set the row height
    pub fn row_height(mut self, height: f32) -> Self {
        self.row_height = height;
        self
    }

    /// Set the container height
    pub fn container_height(mut self, height: f32) -> Self {
        self.container_height = height;
        self
    }

    /// Set the scrollbar configuration
    pub fn config(mut self, config: ScrollbarConfig) -> Self {
        self.config = config;
        self
    }

    /// Calculate the thumb height
    pub fn thumb_height(&self) -> Pixels {
        let total_content_height = self.total_items as f32 * self.row_height;
        let thumb_ratio = (self.container_height / total_content_height).min(1.0);

        // Calculate minimum thumb size based on item count
        let min_thumb_size = if self.total_items <= 10 {
            self.container_height
        } else if self.total_items <= 50 {
            self.container_height * 0.5
        } else if self.total_items <= 200 {
            40.0
        } else {
            15.0
        };

        let thumb_h = (thumb_ratio * self.container_height)
            .max(min_thumb_size)
            .min(self.container_height);
        px(thumb_h)
    }

    /// Calculate the thumb top position
    pub fn thumb_top(&self) -> Pixels {
        let max_index = self.total_items.saturating_sub(self.visible_items);
        if max_index == 0 {
            return px(0.0);
        }

        let thumb_height_f = f32::from(self.thumb_height());
        let track_height = self.container_height - thumb_height_f;
        let thumb_top = (self.current_index as f32 / max_index as f32) * track_height;
        px(thumb_top)
    }

    /// Build the scrollbar element
    pub fn build(self) -> Div {
        let thumb_height_px = self.thumb_height();
        let thumb_top_px = self.thumb_top();
        let width = self.config.width;
        let thumb_width = self.config.thumb_width;
        let track_color = self.config.track_color;
        let thumb_color = self.config.thumb_color;
        let thumb_hover_color = self.config.thumb_hover_color;

        div()
            .w(width)
            .flex()
            .items_center()
            .justify_center()
            .bg(rgb(track_color))
            .child(
                div().size_full().relative().child(
                    // Scrollbar thumb (visual only)
                    div()
                        .w(thumb_width)
                        .h(thumb_height_px)
                        .top(thumb_top_px)
                        .absolute()
                        .bg(rgb(thumb_color))
                        .rounded(px(4.))
                        .hover(|style| style.bg(rgb(thumb_hover_color)))
                        .cursor_grab(),
                ),
            )
    }
}

impl Default for Scrollbar {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function to create a vertical scrollbar
pub fn vertical_scrollbar(total_items: usize, current_index: usize) -> Scrollbar {
    Scrollbar::new()
        .total_items(total_items)
        .current_index(current_index)
        .row_height(20.0)
        .container_height(300.0)
        .visible_items(20)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scrollbar_config() {
        let config = ScrollbarConfig::default();
        assert_eq!(config.width, px(12.));
        assert_eq!(config.min_thumb_size, 15.0);
    }

    #[test]
    fn test_scrollbar_creation() {
        let scrollbar = Scrollbar::new();
        assert_eq!(scrollbar.total_items, 0);
        assert_eq!(scrollbar.row_height, 20.0);
    }

    #[test]
    fn test_thumb_height() {
        let scrollbar = Scrollbar::new()
            .total_items(10)
            .container_height(300.0)
            .row_height(20.0);

        let h = scrollbar.thumb_height();
        // With 10 items, thumb should be large
        assert!(h.as_f32() > 200.0);
    }

    #[test]
    fn test_thumb_top() {
        let scrollbar = Scrollbar::new().total_items(100).current_index(50);

        let top = scrollbar.thumb_top();
        // At index 50 (middle), thumb should be in the middle
        assert!(top.as_f32() > 100.0 && top.as_f32() < 200.0);
    }

    #[test]
    fn test_convenience_function() {
        let scrollbar = vertical_scrollbar(100, 25);
        assert_eq!(scrollbar.total_items, 100);
    }
}
