# Zed-Style Theme Guide

## Overview

This project now includes a comprehensive theme system inspired by [Zed editor](https://zed.dev/) and the [Catppuccin Mocha](https://catppuccin.com/) color palette. The theme provides a cohesive, modern aesthetic with excellent accessibility and visual consistency.

## Color Palette

### Catppuccin Mocha Base Colors

The theme uses the Catppuccin Mocha palette, which provides a warm, dark color scheme with excellent contrast:

- **Rosewater** `#f5e0dc` - Soft accent
- **Flamingo** `#f2cdcd` - Warm highlight
- **Pink** `#f5c2e7` - Feminine accent
- **Mauve** `#cba6f7` - Purple accent
- **Red** `#f38ba8` - Error states
- **Maroon** `#eba0ac` - Secondary error
- **Peach** `#fab387` - Warm warning
- **Yellow** `#f9e2af` - Warning states
- **Green** `#a6e3a1` - Success states
- **Teal** `#94e2d5` - Cyan accent
- **Sky** `#89dceb` - Light blue
- **Sapphire** `#74c7ec` - Interactive hover
- **Blue** `#89b4fa` - Primary color
- **Lavender** `#b4befe` - Active state

### Surface Colors

- **Text** `#cdd6f4` - Primary text
- **Subtext1** `#bac2de` - Secondary text
- **Subtext0** `#a6adc8` - Tertiary text
- **Overlay2** `#9399b2` - Dimmed text
- **Overlay1** `#7f849c` - Placeholder text
- **Overlay0** `#6c7086` - Disabled text
- **Surface2** `#585b70` - Active surface
- **Surface1** `#45475a` - Elevated surface
- **Surface0** `#313244` - Background surface

### Background Colors

- **Base** `#1e1e2e` - Main background
- **Mantle** `#181825` - Secondary background
- **Crust** `#11111b` - Deepest background

## Usage

### Import the Theme

```rust
use crate::ui::theme::{colors, palette, radius, spacing, typography};
```

### Using Colors

```rust
// Semantic colors (recommended)
div()
    .bg(colors::BG_DEFAULT)
    .text_color(colors::TEXT_PRIMARY)
    .border_1()
    .border_color(colors::BORDER_DEFAULT);

// Direct palette colors
div()
    .bg(palette::BLUE)
    .text_color(palette::CRUST);
```

### Using Spacing

```rust
div()
    .px(spacing::LG)      // 16px horizontal padding
    .py(spacing::MD)      // 12px vertical padding
    .gap(spacing::SM);    // 8px gap
```

Spacing scale:
- `XS` - 4px
- `SM` - 8px
- `MD` - 12px
- `LG` - 16px
- `XL` - 20px
- `XXL` - 24px

### Using Border Radius

```rust
div()
    .rounded(radius::MD);  // 4px radius
```

Radius scale:
- `SM` - 2px
- `MD` - 4px
- `LG` - 6px
- `XL` - 8px
- `FULL` - 999px (pill shape)

### Using Typography

```rust
div()
    .font_size(typography::BASE);  // 15px
```

Typography scale:
- `XS` - 11px
- `SM` - 13px
- `BASE` - 15px
- `MD` - 16px
- `LG` - 18px
- `XL` - 20px
- `XXL` - 24px

## Component Examples

### Styled Button

```rust
use crate::ui::components::Button;

Button::new("Click Me")
    .primary()
    .large()
    .icon("→")
    .build(|_event, _window, cx| {
        // Handle click
    });
```

### Styled Card

```rust
use crate::ui::components::{Card, CardPadding};

Card::new()
    .elevated()
    .padding(CardPadding::Relaxed)
    .width(px(400.0))
    .build()
    .child(content);
```

### Dropdown Menu

The dropdown menu component provides a Zed-style selection interface:

```rust
use crate::ui::components::dropdown::{SimpleDropdown, render_dropdown_menu};

// Create a dropdown trigger button
SimpleDropdown::new("Select Channel")
    .items(vec![
        ("1".to_string(), "Channel 1".to_string()),
        ("2".to_string(), "Channel 2".to_string()),
        ("all".to_string(), "All Channels".to_string()),
    ])
    .selected("all")
    .build_trigger(
        cx.entity(),
        |app, cx| {
            // Toggle dropdown visibility
            app.show_channel_dropdown = !app.show_channel_dropdown;
            cx.notify();
        },
    )

// Render the dropdown menu when visible
.when(app.show_channel_dropdown, |parent| {
    parent.child(render_dropdown_menu(
        vec![
            ("1".to_string(), "Channel 1".to_string()),
            ("2".to_string(), "Channel 2".to_string()),
            ("all".to_string(), "All Channels".to_string()),
        ],
        px(300.),  // max_height
        px(180.),  // min_width
        cx.entity(),
        |app, channel_id, cx| {
            // Handle selection
            app.selected_channel = Some(channel_id.clone());
            app.show_channel_dropdown = false;
            cx.notify();
        },
    ))
})
```

**Dropdown Features:**
- Smooth hover effects with Zed-style colors
- Keyboard accessible navigation
- Customizable height and width
- Separator support for grouping items
- Disabled state for non-selectable items
- Optional icons and keyboard shortcuts

## Best Practices

1. **Use Semantic Colors**: Prefer `colors::*` constants over direct `palette::*` colors for better maintainability

2. **Maintain Contrast**: The theme is designed with WCAG AA accessibility standards in mind

3. **Consistent Spacing**: Always use spacing constants instead of arbitrary pixel values

4. **Interactive States**: Use the predefined hover/active colors for consistent feedback

5. **Visual Hierarchy**: 
   - Primary: `colors::PRIMARY` for main actions
   - Secondary: `colors::BG_ELEVATED` for secondary elements
   - Muted: `colors::BG_MUTED` for disabled states

## Color Meanings

- **Blue (PRIMARY)**: Primary actions, links, focus states
- **Green (SUCCESS)**: Success messages, positive confirmations
- **Yellow (WARNING)**: Warning states, caution
- **Red (ERROR)**: Error states, destructive actions
- **Gray surfaces**: Background layers and depth

## Migration Guide

### Old Style

```rust
div()
    .bg(rgb(0x1a1a1a))
    .border_color(rgb(0x2a2a2a))
    .text_color(rgb(0xcdd6f4))
    .px(px(16.0))
    .py(px(12.0))
    .rounded(px(4.0));
```

### New Style

```rust
div()
    .bg(colors::BG_ELEVATED)
    .border_color(colors::BORDER_DEFAULT)
    .text_color(colors::TEXT_PRIMARY)
    .px(spacing::LG)
    .py(spacing::MD)
    .rounded(radius::MD);
```

## Zed Design Principles

This theme follows Zed editor's design philosophy:

1. **Clarity Over Decoration**: Every visual element should have a purpose
2. **Subtle Elevation**: Use shadows and borders sparingly for depth
3. **Smooth Interactions**: Hover and active states should feel natural
4. **High Contrast**: Ensure text is always readable against backgrounds
5. **Consistent Spacing**: Use the 8px grid system (spacing scale)

## Future Enhancements

Potential improvements to consider:

- [ ] Light theme variant
- [ ] Custom accent color support
- [ ] Animation/transition utilities
- [ ] Additional component presets
- [ ] Typography scale refinement
- [ ] Color contrast checker tool

## Resources

- [Zed Editor](https://zed.dev/)
- [Catppuccin Color Palette](https://catppuccin.com/)
- [GPUI Documentation](https://github.com/zed-industries/zed)
- [WCAG Accessibility Guidelines](https://www.w3.org/WAI/WCAG21/quickref/)

## Contributing

When adding new components or UI elements:

1. Use theme constants for all colors, spacing, and sizing
2. Ensure proper contrast ratios (4.5:1 for text)
3. Test in both light and dark contexts (if light theme is added)
4. Follow the established visual hierarchy
5. Document any new theme constants added

---

**Note**: This theme system is built on top of GPUI (Zed's UI framework) and is designed to work seamlessly with GPUI components.