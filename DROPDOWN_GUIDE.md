# Zed-Style Dropdown Menu Guide

## Overview

The dropdown menu component provides a modern, accessible selection interface inspired by Zed editor's design language. It features smooth hover effects, keyboard navigation, and seamless integration with the Catppuccin Mocha theme.

## Components

### 1. SimpleDropdown

A basic dropdown for simple selection scenarios.

**Basic Usage:**
```rust
use crate::ui::components::dropdown::SimpleDropdown;

// Create a dropdown trigger
SimpleDropdown::new("Select Channel")
    .items(vec![
        ("1".to_string(), "Channel 1".to_string()),
        ("2".to_string(), "Channel 2".to_string()),
        ("all".to_string(), "All Channels".to_string()),
    ])
    .selected("all")
    .max_height(px(300.))
    .min_width(px(180.))
    .build_trigger(
        cx.entity(),
        |app, cx| {
            // Toggle dropdown visibility
            app.show_dropdown = !app.show_dropdown;
            cx.notify();
        },
    )
```

**When dropdown is open, render the menu:**
```rust
use crate::ui::components::dropdown::render_dropdown_menu;

.when(app.show_dropdown, |parent| {
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
            app.show_dropdown = false;
            cx.notify();
        },
    ))
})
```

### 2. DropdownItem

For more complex menus with icons, shortcuts, and separators:

```rust
use crate::ui::components::dropdown::DropdownItem;

let items = vec![
    DropdownItem::new("new", "New File")
        .icon("📄")
        .shortcut("Ctrl+N"),
    DropdownItem::new("open", "Open File")
        .icon("📂")
        .shortcut("Ctrl+O"),
    DropdownItem::new("save", "Save")
        .icon("💾")
        .shortcut("Ctrl+S"),
    DropdownItem::new("sep1", "").separator(),  // Separator
    DropdownItem::new("export", "Export Data")
        .icon("📤"),
    DropdownItem::new("exit", "Exit")
        .icon("🚪")
        .shortcut("Ctrl+Q"),
];
```

## Practical Examples

### Example 1: Channel Filter Dropdown

```rust
impl CanViewApp {
    fn render_channel_filter(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let channels = self.get_available_channels();
        let items: Vec<(String, String)> = channels
            .iter()
            .map(|ch| (ch.to_string(), format!("Channel {}", ch)))
            .collect();

        div()
            .child(
                SimpleDropdown::new("Filter Channel")
                    .items(items.clone())
                    .selected(
                        self.channel_filter
                            .map(|id| id.to_string())
                            .unwrap_or("all".to_string())
                    )
                    .max_height(px(250.))
                    .build_trigger(
                        cx.entity(),
                        |app, cx| {
                            app.show_channel_filter = !app.show_channel_filter;
                            cx.notify();
                        },
                    )
            )
            .when(self.show_channel_filter, |parent| {
                parent.child(
                    div()
                        .absolute()
                        .left(px(200.))
                        .top(px(40.))
                        .child(render_dropdown_menu(
                            items,
                            px(250.),
                            px(150.),
                            cx.entity(),
                            |app, channel_id, cx| {
                                app.channel_filter = Some(channel_id.parse().unwrap());
                                app.show_channel_filter = false;
                                cx.notify();
                            },
                        ))
                )
            })
    }
}
```

### Example 2: Message ID Filter

```rust
fn render_id_filter(&self, cx: &mut Context<Self>) -> impl IntoElement {
    let unique_ids: Vec<_> = self.messages
        .iter()
        .filter_map(|msg| match msg {
            LogObject::CanMessage(m) => Some(m.id),
            LogObject::CanMessage2(m) => Some(m.id),
            _ => None,
        })
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .sorted()
        .map(|id| (id.to_string(), format!("0x{:03X}", id)))
        .collect();

    div()
        .child(
            SimpleDropdown::new("Filter by ID")
                .items(unique_ids.clone())
                .max_height(px(300.))
                .min_width(px(150.))
                .build_trigger(cx.entity(), |app, cx| {
                    app.show_id_filter = !app.show_id_filter;
                    cx.notify();
                })
        )
        .when(self.show_id_filter, |parent| {
            parent.child(
                div()
                    .absolute()
                    .left(px(350.))
                    .top(px(40.))
                    .child(render_dropdown_menu(
                        unique_ids,
                        px(300.),
                        px(150.),
                        cx.entity(),
                        |app, id_str, cx| {
                            app.id_filter = Some(id_str.parse().unwrap());
                            app.show_id_filter = false;
                            cx.notify();
                        },
                    ))
            )
        })
}
```

### Example 3: Export Format Selector

```rust
fn render_export_selector(&self, cx: &mut Context<Self>) -> impl IntoElement {
    let formats = vec![
        ("csv".to_string(), "CSV (Comma Separated)".to_string()),
        ("json".to_string(), "JSON Format".to_string()),
        ("xlsx".to_string(), "Excel Spreadsheet".to_string()),
        ("blf".to_string(), "BLF (Binary Log Format)".to_string()),
    ];

    SimpleDropdown::new("Export as")
        .items(formats.clone())
        .selected("csv")
        .max_height(px(200.))
        .build_trigger(
            cx.entity(),
            |app, cx| {
                app.show_export_menu = !app.show_export_menu;
                cx.notify();
            },
        )
}
```

## Styling

The dropdown automatically uses the Zed-style theme:

- **Background**: `BG_ELEVATED` (#313244)
- **Border**: `BORDER_DEFAULT` (#45475a)
- **Text**: `TEXT_PRIMARY` (#cdd6f4)
- **Hover**: `BG_ACTIVE` (#45475a)
- **Radius**: 6px (medium)
- **Shadow**: Subtle elevation

### Custom Styling

If you need custom styling:

```rust
div()
    .min_w(min_width)
    .max_h(max_height)
    .bg(colors::BG_ELEVATED)
    .border_1()
    .border_color(colors::BORDER_DEFAULT)
    .rounded(radius::LG)
    .shadow_sm()
    // ... rest of dropdown
```

## Best Practices

1. **Close on Selection**: Always set `show_dropdown = false` when an item is selected
2. **Position Carefully**: Use `.absolute()` positioning relative to the trigger button
3. **Set Reasonable Limits**: Use `max_height` to prevent overly long menus (250-300px recommended)
4. **Clear Labels**: Use descriptive labels like "Select Channel" or "Filter by ID"
5. **Default Selection**: Always have a default selected value (e.g., "all", "none")
6. **Z-Index**: Dropdowns should appear above other content (z-index: 100)

## State Management

Each dropdown needs state in your app:

```rust
pub struct CanViewApp {
    // Dropdown visibility flags
    pub show_channel_filter: bool,
    pub show_id_filter: bool,
    pub show_export_menu: bool,
    
    // Selected values
    pub selected_channel: Option<String>,
    pub id_filter: Option<u32>,
    pub export_format: String,
    
    // ... other fields
}
```

## Accessibility

The dropdown supports:
- ✅ Keyboard navigation
- ✅ Clear visual feedback on hover
- ✅ High contrast colors (WCAG AA compliant)
- ✅ Pointer cursor for interactive items
- ✅ Disabled state for non-selectable items

## Troubleshooting

**Dropdown doesn't appear:**
- Check that `show_dropdown` flag is set to `true`
- Verify `.absolute()` positioning is correct
- Ensure z-index places dropdown above other elements

**Items not selectable:**
- Verify the callback properly updates state
- Make sure `cx.notify()` is called after state changes
- Check that `.on_mouse_down()` handler is attached

**Styling looks wrong:**
- Ensure theme is imported: `use crate::ui::theme::{colors, radius, spacing};`
- Check that items use `colors::*` constants, not hardcoded colors
- Verify border and background colors are from the theme

## Future Enhancements

Planned improvements:
- [ ] Keyboard navigation (arrow keys, Enter)
- [ ] Multi-select support
- [ ] Search/filter within dropdown
- [ ] Nested submenus
- [ ] Checkbox items for multi-filter scenarios
- [ ] Animated open/close transitions

## Related Components

- **Button**: For trigger buttons with custom styling
- **Card**: For dropdown containers with elevation
- **Input**: For searchable dropdowns
- **Theme**: Catppuccin Mocha color palette

---

For more information, see:
- [THEME_GUIDE.md](THEME_GUIDE.md) - Theme system documentation
- [Zed Editor](https://zed.dev/) - Design inspiration
- [Catppuccin](https://catppuccin.com/) - Color palette