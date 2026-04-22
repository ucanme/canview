//! Zed-style Card component
//!
//! Provides elegant, elevated containers for grouping content with subtle shadows and smooth interactions.

use crate::ui::theme::{colors, palette, radius, spacing};
use gpui::{prelude::*, *};

/// Card style variants
#[derive(Clone, Copy, PartialEq)]
pub enum CardStyle {
    Default,     // Standard card with subtle elevation
    Elevated,    // More prominent elevation
    Bordered,    // Card with visible border
    Ghost,       // Minimal styling, transparent background
    Interactive, // Appears clickable with stronger feedback
}

impl CardStyle {
    fn bg(&self) -> Rgba {
        match self {
            CardStyle::Default => colors::BG_ELEVATED,
            CardStyle::Elevated => colors::BG_ELEVATED,
            CardStyle::Bordered => colors::BG_ELEVATED,
            CardStyle::Ghost => colors::BG_DEFAULT,
            CardStyle::Interactive => colors::BG_ELEVATED,
        }
    }

    fn border(&self) -> Rgba {
        match self {
            CardStyle::Default => colors::BORDER_SUBTLE,
            CardStyle::Elevated => colors::BORDER_SUBTLE,
            CardStyle::Bordered => colors::BORDER_DEFAULT,
            CardStyle::Ghost => colors::BORDER_SUBTLE,
            CardStyle::Interactive => colors::BORDER_DEFAULT,
        }
    }

    fn has_border(&self) -> bool {
        matches!(self, CardStyle::Bordered | CardStyle::Interactive)
    }

    fn hover_bg(&self) -> Option<Rgba> {
        match self {
            CardStyle::Ghost => Some(colors::BG_ELEVATED),
            CardStyle::Interactive => Some(colors::BG_ACTIVE),
            _ => None,
        }
    }

    fn active_bg(&self) -> Option<Rgba> {
        match self {
            CardStyle::Interactive => Some(palette::SURFACE1),
            _ => None,
        }
    }

    fn cursor(&self) -> Option<CursorStyle> {
        match self {
            CardStyle::Interactive => Some(CursorStyle::Pointer),
            _ => None,
        }
    }

    fn shadow(&self) -> Rgba {
        match self {
            CardStyle::Elevated => colors::SHADOW_MD,
            CardStyle::Interactive => colors::SHADOW_SM,
            _ => colors::SHADOW_SM,
        }
    }
}

/// Card padding preset
#[derive(Clone, Copy)]
pub enum CardPadding {
    None,
    Tight,
    Normal,
    Relaxed,
    Spacious,
}

impl CardPadding {
    fn values(&self) -> (gpui::Pixels, gpui::Pixels) {
        match self {
            CardPadding::None => (px(0.), px(0.)),
            CardPadding::Tight => (spacing::SM, spacing::SM),
            CardPadding::Normal => (spacing::LG, spacing::MD),
            CardPadding::Relaxed => (spacing::XL, spacing::LG),
            CardPadding::Spacious => (spacing::XXL, spacing::XL),
        }
    }
}

/// Modern card component builder
pub struct Card {
    style: CardStyle,
    padding: CardPadding,
    width: Option<gpui::Pixels>,
    height: Option<gpui::Pixels>,
    min_width: Option<gpui::Pixels>,
    min_height: Option<gpui::Pixels>,
    max_width: Option<gpui::Pixels>,
    max_height: Option<gpui::Pixels>,
    rounded: Option<gpui::Pixels>,
    on_click: Option<Box<dyn FnMut(&MouseEvent, &mut Window, &mut Context<Entity<Any>>) + 'static>>,
    disabled: bool,
}

impl Card {
    /// Create a new card with default styling
    pub fn new() -> Self {
        Self {
            style: CardStyle::Default,
            padding: CardPadding::Normal,
            width: None,
            height: None,
            min_width: None,
            min_height: None,
            max_width: None,
            max_height: None,
            rounded: None,
            on_click: None,
            disabled: false,
        }
    }

    /// Set card style variant
    pub fn style(mut self, style: CardStyle) -> Self {
        self.style = style;
        self
    }

    /// Elevated card with more prominent shadow
    pub fn elevated(mut self) -> Self {
        self.style = CardStyle::Elevated;
        self
    }

    /// Bordered card with visible border
    pub fn bordered(mut self) -> Self {
        self.style = CardStyle::Bordered;
        self
    }

    /// Ghost card with minimal styling
    pub fn ghost(mut self) -> Self {
        self.style = CardStyle::Ghost;
        self
    }

    /// Interactive card with click feedback
    pub fn interactive(mut self) -> Self {
        self.style = CardStyle::Interactive;
        self
    }

    /// Set padding preset
    pub fn padding(mut self, padding: CardPadding) -> Self {
        self.padding = padding;
        self
    }

    /// Set explicit width
    pub fn width(mut self, width: gpui::Pixels) -> Self {
        self.width = Some(width);
        self
    }

    /// Set explicit height
    pub fn height(mut self, height: gpui::Pixels) -> Self {
        self.height = Some(height);
        self
    }

    /// Set minimum width
    pub fn min_width(mut self, width: gpui::Pixels) -> Self {
        self.min_width = Some(width);
        self
    }

    /// Set minimum height
    pub fn min_height(mut self, height: gpui::Pixels) -> Self {
        self.min_height = Some(height);
        self
    }

    /// Set maximum width
    pub fn max_width(mut self, width: gpui::Pixels) -> Self {
        self.max_width = Some(width);
        self
    }

    /// Set maximum height
    pub fn max_height(mut self, height: gpui::Pixels) -> Self {
        self.max_height = Some(height);
        self
    }

    /// Set border radius (overrides default)
    pub fn rounded(mut self, radius: gpui::Pixels) -> Self {
        self.rounded = Some(radius);
        self
    }

    /// Make card clickable
    pub fn clickable(
        mut self,
        on_click: impl FnMut(&MouseEvent, &mut Window, &mut Context<Entity<Any>>) + 'static,
    ) -> Self {
        self.style = CardStyle::Interactive;
        self.on_click = Some(Box::new(on_click));
        self
    }

    /// Disable card interaction
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Build the card element
    pub fn build(self) -> Div {
        let bg = if self.disabled {
            colors::BG_MUTED
        } else {
            self.style.bg()
        };
        let border = self.style.border();
        let hover_bg = self.style.hover_bg();
        let active_bg = self.style.active_bg();
        let cursor = self.style.cursor();
        let has_border = self.style.has_border() || self.disabled;
        let shadow = self.style.shadow();
        let rounded = self.rounded.unwrap_or(match self.style {
            CardStyle::Elevated => radius::LG,
            _ => radius::MD,
        });
        let (h_pad, v_pad) = self.padding.values();

        let mut card = div()
            .bg(bg)
            .px(h_pad)
            .py(v_pad)
            .rounded(rounded)
            .when(has_border, |div| div.border_1().border_color(border))
            .shadow_2xl(shadow);

        // Apply size constraints
        if let Some(w) = self.width {
            card = card.w(w);
        }
        if let Some(h) = self.height {
            card = card.h(h);
        }
        if let Some(w) = self.min_width {
            card = card.min_w(w);
        }
        if let Some(h) = self.min_height {
            card = card.min_h(h);
        }
        if let Some(w) = self.max_width {
            card = card.max_w(w);
        }
        if let Some(h) = self.max_height {
            card = card.max_h(h);
        }

        // Add cursor style
        if let Some(cursor_style) = cursor {
            card = if !self.disabled {
                card.cursor_pointer()
            } else {
                card.cursor_not_allowed()
            };
        }

        // Add hover effect if applicable
        if !self.disabled {
            if let Some(hover) = hover_bg {
                card = card.hover(move |style| style.bg(hover));
            }

            if let Some(active) = active_bg {
                card = card.active(move |style| style.bg(active));
            }
        }

        // Add click handler if present
        if let Some(on_click) = self.on_click {
            if !self.disabled {
                card = card.on_mouse_down(gpui::MouseButton::Left, on_click);
            }
        }

        card
    }
}

impl Default for Card {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_card_creation() {
        let card = Card::new();
        assert_eq!(card.style, CardStyle::Default);
        assert_eq!(card.padding, CardPadding::Normal);
    }

    #[test]
    fn test_card_variants() {
        let elevated = Card::new().elevated();
        assert_eq!(elevated.style, CardStyle::Elevated);

        let bordered = Card::new().bordered();
        assert_eq!(bordered.style, CardStyle::Bordered);

        let ghost = Card::new().ghost();
        assert_eq!(ghost.style, CardStyle::Ghost);

        let interactive = Card::new().interactive();
        assert_eq!(interactive.style, CardStyle::Interactive);
    }

    #[test]
    fn test_card_padding() {
        let tight = Card::new().padding(CardPadding::Tight);
        assert_eq!(tight.padding, CardPadding::Tight);

        let spacious = Card::new().padding(CardPadding::Spacious);
        assert_eq!(spacious.padding, CardPadding::Spacious);
    }

    #[test]
    fn test_card_sizes() {
        let card = Card::new()
            .width(px(300.))
            .height(px(200.))
            .min_width(px(100.))
            .max_width(px(500.));

        assert_eq!(card.width, Some(px(300.)));
        assert_eq!(card.height, Some(px(200.)));
        assert_eq!(card.min_width, Some(px(100.)));
        assert_eq!(card.max_width, Some(px(500.)));
    }

    #[test]
    fn test_card_builder_pattern() {
        let card = Card::new()
            .interactive()
            .padding(CardPadding::Relaxed)
            .width(px(400.))
            .rounded(radius::XL)
            .disabled(false);

        assert_eq!(card.style, CardStyle::Interactive);
        assert_eq!(card.padding, CardPadding::Relaxed);
        assert_eq!(card.width, Some(px(400.)));
        assert_eq!(card.rounded, Some(radius::XL));
        assert!(!card.disabled);
    }

    #[test]
    fn test_style_borders() {
        assert!(!CardStyle::Default.has_border());
        assert!(!CardStyle::Elevated.has_border());
        assert!(CardStyle::Bordered.has_border());
        assert!(!CardStyle::Ghost.has_border());
        assert!(CardStyle::Interactive.has_border());
    }
}
