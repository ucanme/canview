//! Dropdown component usage examples
//!
//! This file demonstrates how to use the Dropdown component in various scenarios.
//! It shows how to replace existing dropdown code with the new component.

use gpui::{prelude::*, *};
use crate::ui::components::dropdown::{Dropdown, DropdownItem, simple_dropdown};
use crate::app::{AppView, CanViewApp};

/// Example 1: Simple dropdown with string items
pub fn example_simple_dropdown(cx: &mut Context<CanViewApp>) -> impl IntoElement {
    let items = vec![
        DropdownItem::new("Option 1", "1"),
        DropdownItem::new("Option 2", "2"),
        DropdownItem::new("Option 3", "3"),
    ];

    simple_dropdown("Select Option", vec!["One", "Two", "Three"])
        .placeholder("Choose an option...")
        .build()
}

/// Example 2: Dropdown with custom items
pub fn example_custom_dropdown(cx: &mut Context<CanViewApp>) -> impl IntoElement {
    let items = vec![
        DropdownItem::new("🔵 CAN Channel", "can"),
        DropdownItem::new("🟨 LIN Channel", "lin"),
        DropdownItem::new("⚙️ Settings", "settings"),
    ];

    Dropdown::new("Channel Type", items)
        .placeholder("Select type...")
        .max_height(px(200.))
        .build()
}

/// Example 3: Replace existing ID filter dropdown
///
/// This shows how to replace the complex ID filter dropdown code
/// with the new Dropdown component
pub fn example_id_filter_replacement(cx: &mut Context<CanViewApp>) -> impl IntoElement {
    let view = cx.entity().clone();

    // Extract unique IDs from messages
    let mut unique_ids = std::collections::HashSet::new();
    for msg in &view.read(cx).messages {
        match msg {
            blf::LogObject::CanMessage(m) => { unique_ids.insert(m.id); }
            blf::LogObject::CanFdMessage(m) => { unique_ids.insert(m.id); }
            _ => {}
        }
    }

    let mut id_list: Vec<_> = unique_ids.into_iter().collect();
    id_list.sort_unstable();

    // Convert to dropdown items
    let items: Vec<DropdownItem> = id_list
        .iter()
        .map(|id| DropdownItem::new(format!("{:#x}", id), format!("{}", id)))
        .collect();

    // Create dropdown
    Dropdown::new("Filter by ID", items)
        .placeholder("All IDs")
        .build()
        .on_mouse_down(gpui::MouseButton::Left, {
            let view = view.clone();
            move |_event, _window, cx| {
                // Show dropdown menu (you would implement this)
                view.update(cx, |app, cx| {
                    app.show_id_filter_input = true;
                    cx.notify();
                });
            }
        })
}

/// Example 4: Replace channel filter dropdown
pub fn example_channel_filter_replacement(cx: &mut Context<CanViewApp>) -> impl IntoElement {
    let view = cx.entity().clone();

    // Extract unique channels
    let mut unique_channels = std::collections::HashSet::new();
    for msg in &view.read(cx).messages {
        match msg {
            blf::LogObject::CanMessage(m) => { unique_channels.insert(m.channel); }
            blf::LogObject::CanFdMessage(m) => { unique_channels.insert(m.channel); }
            blf::LogObject::LinMessage(m) => { unique_channels.insert(m.channel); }
            _ => {}
        }
    }

    let mut channel_list: Vec<_> = unique_channels.into_iter().collect();
    channel_list.sort_unstable();

    // Convert to dropdown items
    let items: Vec<DropdownItem> = channel_list
        .iter()
        .map(|ch| DropdownItem::new(format!("Channel {}", ch), format!("{}", ch)))
        .collect();

    Dropdown::new("Filter by Channel", items)
        .placeholder("All Channels")
        .build()
}

/// Example 5: Dropdown with async selection
pub fn example_dropdown_with_selection(cx: &mut Context<CanViewApp>) -> impl IntoElement {
    let view = cx.entity().clone();

    let items = vec![
        DropdownItem::new("Load BLF File", "load_blf"),
        DropdownItem::new("Save Config", "save_config"),
        DropdownItem::new("Import Database", "import_db"),
    ];

    Dropdown::new("File", items)
        .build()
        .on_mouse_down(gpui::MouseButton::Left, {
            let view = view.clone();
            move |_event, _window, cx| {
                view.update(cx, |app, cx| {
                    app.show_file_menu = !app.show_file_menu;
                    cx.notify();
                });
            }
        })
}

/// Example 6: Disabled dropdown
pub fn example_disabled_dropdown() -> impl IntoElement {
    let items = vec![
        DropdownItem::new("Option 1", "1"),
        DropdownItem::new("Option 2", "2"),
    ];

    Dropdown::new("Disabled Dropdown", items)
        .disabled(true)
        .placeholder("Cannot select")
        .build()
}

/// Example 7: Compact dropdown for toolbar
pub fn example_compact_dropdown() -> impl IntoElement {
    let items = vec![
        DropdownItem::new("Small", "small"),
        DropdownItem::new("Medium", "medium"),
        DropdownItem::new("Large", "large"),
    ];

    Dropdown::new("", items)
        .max_height(px(150.))
        .build()
}

/// Example 8: Before and After Comparison
///
/// BEFORE (old code with ~100 lines):
/// ```rust
/// // Complex div structure with uniform_list
/// div()
///     .absolute()
///     .top(px(36.))
///     .left(px(16.))
///     .w(px(160.))
///     .bg(rgb(0x1e1e1e))
///     .border_1()
///     .border_color(rgb(0x45475a))
///     .rounded(px(6.))
///     .shadow_lg()
///     .flex()
///     .flex_col()
///     .py_1()
///     .when(show_dropdown, |this| {
///         this.child(uniform_list(...))
///     })
/// ```
///
/// AFTER (using Dropdown component):
/// ```rust
/// let items = vec![
///     DropdownItem::new("Item 1", "1"),
///     DropdownItem::new("Item 2", "2"),
/// ];
///
/// Dropdown::new("Select", items).build()
/// ```
pub fn example_migration_guide() {
    // This is documentation only
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dropdown_item_creation() {
        let item = DropdownItem::new("Test", "test_id");
        assert_eq!(item.label, "Test");
        assert_eq!(item.id, "test_id");
    }

    #[test]
    fn test_simple_dropdown() {
        let dropdown = simple_dropdown("Test", vec!["A", "B", "C"]);
        assert_eq!(dropdown.items.len(), 3);
    }

    #[test]
    fn test_dropdown_builder() {
        let items = vec![
            DropdownItem::new("Item 1", "1"),
            DropdownItem::new("Item 2", "2"),
        ];

        let dropdown = Dropdown::new("Select", items)
            .placeholder("Choose...")
            .disabled(false)
            .max_height(px(200.));

        assert_eq!(dropdown.items.len(), 2);
        assert_eq!(dropdown.placeholder, "Choose...");
        assert!(!dropdown.disabled);
    }
}
```
