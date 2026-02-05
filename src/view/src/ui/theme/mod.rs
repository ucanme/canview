//! Zed-inspired theme module with Catppuccin Mocha colors
//!
//! This module provides a cohesive color palette and styling utilities
//! inspired by Zed editor's modern, clean aesthetic.

use gpui::Rgba;

/// Catppuccin Mocha color palette
pub mod palette {
    use gpui::Rgba;

    // Base colors
    pub const ROSEWATER: Rgba = Rgba {
        r: 0xf5 as f32 / 255.0,
        g: 0xe0 as f32 / 255.0,
        b: 0xdc as f32 / 255.0,
        a: 1.0,
    };
    pub const FLAMINGO: Rgba = Rgba {
        r: 0xf2 as f32 / 255.0,
        g: 0xcd as f32 / 255.0,
        b: 0xcd as f32 / 255.0,
        a: 1.0,
    };
    pub const PINK: Rgba = Rgba {
        r: 0xf5 as f32 / 255.0,
        g: 0xc2 as f32 / 255.0,
        b: 0xe7 as f32 / 255.0,
        a: 1.0,
    };
    pub const MAUVE: Rgba = Rgba {
        r: 0xcb as f32 / 255.0,
        g: 0xa6 as f32 / 255.0,
        b: 0xf7 as f32 / 255.0,
        a: 1.0,
    };
    pub const RED: Rgba = Rgba {
        r: 0xf3 as f32 / 255.0,
        g: 0x8b as f32 / 255.0,
        b: 0xa8 as f32 / 255.0,
        a: 1.0,
    };
    pub const MAROON: Rgba = Rgba {
        r: 0xeb as f32 / 255.0,
        g: 0xa0 as f32 / 255.0,
        b: 0xac as f32 / 255.0,
        a: 1.0,
    };
    pub const PEACH: Rgba = Rgba {
        r: 0xfa as f32 / 255.0,
        g: 0xb3 as f32 / 255.0,
        b: 0x87 as f32 / 255.0,
        a: 1.0,
    };
    pub const YELLOW: Rgba = Rgba {
        r: 0xf9 as f32 / 255.0,
        g: 0xe2 as f32 / 255.0,
        b: 0xaf as f32 / 255.0,
        a: 1.0,
    };
    pub const GREEN: Rgba = Rgba {
        r: 0xa6 as f32 / 255.0,
        g: 0xe3 as f32 / 255.0,
        b: 0xa1 as f32 / 255.0,
        a: 1.0,
    };
    pub const TEAL: Rgba = Rgba {
        r: 0x94 as f32 / 255.0,
        g: 0xe2 as f32 / 255.0,
        b: 0xd5 as f32 / 255.0,
        a: 1.0,
    };
    pub const SKY: Rgba = Rgba {
        r: 0x89 as f32 / 255.0,
        g: 0xdc as f32 / 255.0,
        b: 0xeb as f32 / 255.0,
        a: 1.0,
    };
    pub const SAPPHIRE: Rgba = Rgba {
        r: 0x74 as f32 / 255.0,
        g: 0xc7 as f32 / 255.0,
        b: 0xec as f32 / 255.0,
        a: 1.0,
    };
    pub const BLUE: Rgba = Rgba {
        r: 0x89 as f32 / 255.0,
        g: 0xb4 as f32 / 255.0,
        b: 0xfa as f32 / 255.0,
        a: 1.0,
    };
    pub const LAVENDER: Rgba = Rgba {
        r: 0xb4 as f32 / 255.0,
        g: 0xbe as f32 / 255.0,
        b: 0xfe as f32 / 255.0,
        a: 1.0,
    };

    // Surface colors
    pub const TEXT: Rgba = Rgba {
        r: 0xcd as f32 / 255.0,
        g: 0xd6 as f32 / 255.0,
        b: 0xf4 as f32 / 255.0,
        a: 1.0,
    };
    pub const SUBTEXT1: Rgba = Rgba {
        r: 0xba as f32 / 255.0,
        g: 0xc2 as f32 / 255.0,
        b: 0xde as f32 / 255.0,
        a: 1.0,
    };
    pub const SUBTEXT0: Rgba = Rgba {
        r: 0xa6 as f32 / 255.0,
        g: 0xad as f32 / 255.0,
        b: 0xc8 as f32 / 255.0,
        a: 1.0,
    };
    pub const OVERLAY2: Rgba = Rgba {
        r: 0x93 as f32 / 255.0,
        g: 0x99 as f32 / 255.0,
        b: 0xb2 as f32 / 255.0,
        a: 1.0,
    };
    pub const OVERLAY1: Rgba = Rgba {
        r: 0x7f as f32 / 255.0,
        g: 0x84 as f32 / 255.0,
        b: 0x9c as f32 / 255.0,
        a: 1.0,
    };
    pub const OVERLAY0: Rgba = Rgba {
        r: 0x6c as f32 / 255.0,
        g: 0x70 as f32 / 255.0,
        b: 0x86 as f32 / 255.0,
        a: 1.0,
    };
    pub const SURFACE2: Rgba = Rgba {
        r: 0x58 as f32 / 255.0,
        g: 0x5b as f32 / 255.0,
        b: 0x70 as f32 / 255.0,
        a: 1.0,
    };
    pub const SURFACE1: Rgba = Rgba {
        r: 0x45 as f32 / 255.0,
        g: 0x47 as f32 / 255.0,
        b: 0x5a as f32 / 255.0,
        a: 1.0,
    };
    pub const SURFACE0: Rgba = Rgba {
        r: 0x31 as f32 / 255.0,
        g: 0x32 as f32 / 255.0,
        b: 0x44 as f32 / 255.0,
        a: 1.0,
    };

    // Background colors
    pub const BASE: Rgba = Rgba {
        r: 0x1e as f32 / 255.0,
        g: 0x1e as f32 / 255.0,
        b: 0x2e as f32 / 255.0,
        a: 1.0,
    };
    pub const MANTLE: Rgba = Rgba {
        r: 0x18 as f32 / 255.0,
        g: 0x18 as f32 / 255.0,
        b: 0x25 as f32 / 255.0,
        a: 1.0,
    };
    pub const CRUST: Rgba = Rgba {
        r: 0x11 as f32 / 255.0,
        g: 0x11 as f32 / 255.0,
        b: 0x1b as f32 / 255.0,
        a: 1.0,
    };
}

/// Semantic color tokens for UI elements
pub mod colors {
    use super::palette;
    use gpui::Rgba;

    // Primary colors
    pub const PRIMARY: Rgba = palette::BLUE;
    pub const PRIMARY_HOVER: Rgba = palette::SAPPHIRE;
    pub const PRIMARY_ACTIVE: Rgba = palette::LAVENDER;

    // Background surfaces
    pub const BG_DEFAULT: Rgba = palette::BASE;
    pub const BG_ELEVATED: Rgba = palette::SURFACE0;
    pub const BG_MUTED: Rgba = palette::MANTLE;
    pub const BG_ACTIVE: Rgba = palette::SURFACE1;

    // Borders
    pub const BORDER_DEFAULT: Rgba = palette::SURFACE1;
    pub const BORDER_FOCUSED: Rgba = palette::BLUE;
    pub const BORDER_HOVER: Rgba = palette::OVERLAY0;
    pub const BORDER_SUBTLE: Rgba = palette::SURFACE0;

    // Text
    pub const TEXT_PRIMARY: Rgba = palette::TEXT;
    pub const TEXT_SECONDARY: Rgba = palette::SUBTEXT1;
    pub const TEXT_MUTED: Rgba = palette::SUBTEXT0;
    pub const TEXT_PLACEHOLDER: Rgba = palette::OVERLAY1;

    // Status colors
    pub const SUCCESS: Rgba = palette::GREEN;
    pub const WARNING: Rgba = palette::YELLOW;
    pub const ERROR: Rgba = palette::RED;
    pub const INFO: Rgba = palette::BLUE;

    // Additional surface colors for interactive states
    pub const SURFACE0: Rgba = palette::SURFACE0;
    pub const SURFACE1: Rgba = palette::SURFACE1;
    pub const SURFACE2: Rgba = palette::SURFACE2;
    pub const MAROON: Rgba = palette::MAROON;
    pub const TEAL: Rgba = palette::TEAL;

    // Interactive states
    pub const INTERACTIVE: Rgba = palette::BLUE;
    pub const INTERACTIVE_HOVER: Rgba = palette::SAPPHIRE;
    pub const DISABLED: Rgba = palette::OVERLAY0;

    // Shadows (for depth)
    pub const SHADOW_SM: Rgba = Rgba {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.1,
    };
    pub const SHADOW_MD: Rgba = Rgba {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.2,
    };
    pub const SHADOW_LG: Rgba = Rgba {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.3,
    };
}

/// Spacing constants (in pixels)
pub mod spacing {
    use gpui::px;

    pub const XS: gpui::Pixels = px(4.0);
    pub const SM: gpui::Pixels = px(8.0);
    pub const MD: gpui::Pixels = px(12.0);
    pub const LG: gpui::Pixels = px(16.0);
    pub const XL: gpui::Pixels = px(20.0);
    pub const XXL: gpui::Pixels = px(24.0);
}

/// Border radius constants
pub mod radius {
    use gpui::px;

    pub const SM: gpui::Pixels = px(2.0);
    pub const MD: gpui::Pixels = px(4.0);
    pub const LG: gpui::Pixels = px(6.0);
    pub const XL: gpui::Pixels = px(8.0);
    pub const FULL: gpui::Pixels = px(999.0);
}

/// Typography scale
pub mod typography {
    use gpui::px;

    pub const XS: gpui::Pixels = px(11.0);
    pub const SM: gpui::Pixels = px(13.0);
    pub const BASE: gpui::Pixels = px(15.0);
    pub const MD: gpui::Pixels = px(16.0);
    pub const LG: gpui::Pixels = px(18.0);
    pub const XL: gpui::Pixels = px(20.0);
    pub const XXL: gpui::Pixels = px(24.0);
}

/// Z-index layers
pub mod z_index {
    pub const DROPDOWN: u32 = 100;
    pub const MODAL: u32 = 200;
    pub const TOOLTIP: u32 = 300;
    pub const TOAST: u32 = 400;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_accessibility() {
        // Ensure high contrast ratios for accessibility
        // This is a placeholder - actual contrast calculations would require
        // a more sophisticated testing framework
        assert!(colors::TEXT_PRIMARY.a > 0.8);
        assert!(colors::BG_DEFAULT.a == 1.0);
    }

    #[test]
    fn test_spacing_consistency() {
        // Spacing should follow a consistent scale
        assert!(spacing::SM.0 < spacing::MD.0);
        assert!(spacing::MD.0 < spacing::LG.0);
        assert!(spacing::LG.0 < spacing::XL.0);
    }
}
