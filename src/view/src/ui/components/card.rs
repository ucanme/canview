//! Card component
//!
//! Provides an elevated card container for grouping content.

use gpui::{prelude::*, *};

/// Card style variants
#[derive(Clone, Copy)]
pub enum CardStyle {
    Default,  // Default card
    Hover,    // Card with hover effect
    Clickable, // Card that appears clickable
}

impl CardStyle {
    fn bg(&self) -> u32 {
        match self {
            CardStyle::Default => 0x1a1a1a,
            CardStyle::Hover => 0x1a1a1a,
            CardStyle::Clickable => 0x1a1a1a,
        }
    }

    fn hover_bg(&self) -> Option<u32> {
        match self {
            CardStyle::Default => None,
            CardStyle::Hover => Some(0x252525),
            CardStyle::Clickable => Some(0x252525),
        }
    }
}

/// Card component builder
pub struct Card {
    style: CardStyle,
    padding: Option<(f32, f32)>, // (horizontal, vertical)
    on_click: Option<Box<dyn FnMut(&MouseEvent, &mut Window, &mut Context<Entity<Any>>) + 'static>>,
}

impl Card {
    /// Create a new card with default styling
    pub fn new() -> Self {
        Self {
            style: CardStyle::Default,
            padding: None,
            on_click: None,
        }
    }

    /// Set card style
    pub fn style(mut self, style: CardStyle) -> Self {
        self.style = style;
        self
    }

    /// Set padding (horizontal, vertical) in pixels
    pub fn padding(mut self, horizontal: f32, vertical: f32) -> Self {
        self.padding = Some((horizontal, vertical));
        self
    }

    /// Make card clickable with hover effect
    pub fn clickable(
        mut self,
        on_click: impl FnMut(&MouseEvent, &mut Window, &mut Context<Entity<Any>>) + 'static,
    ) -> Self {
        self.style = CardStyle::Clickable;
        self.on_click = Some(Box::new(on_click));
        self
    }

    /// Build the card element
    pub fn build(self) -> Div {
        let bg = self.style.bg();
        let hover_bg = self.style.hover_bg();
        let (h_pad, v_pad) = self.padding.unwrap_or((16.0, 12.0));

        let mut card = div()
            .bg(rgb(bg))
            .px(h_pad)
            .py(v_pad)
            .rounded(px(8.))
            .border_1()
            .border_color(rgb(0x2a2a2a));

        // Add hover effect if applicable
        if let Some(hover_bg) = hover_bg {
            card = card.hover(|style| style.bg(rgb(hover_bg)));
        }

        // Add click handler if present
        if let Some(on_click) = self.on_click {
            card = card.cursor_pointer().on_mouse_down(gpui::MouseButton::Left, on_click);
        }

        card
    }
}

impl Default for Card {
    fn default() -> Self {
        Self::new()
    }
}
