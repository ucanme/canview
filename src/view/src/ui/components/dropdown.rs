//! Dropdown component
//!
//! A reusable dropdown menu component for displaying lists of items.
//! Based on the existing ID and channel filter dropdowns.

use gpui::{prelude::*, *};
use std::rc::Rc;

/// Dropdown menu component
///
/// # Example
/// ```rust
/// use crate::ui::components::dropdown::{Dropdown, DropdownItem};
///
/// let items = vec![
///     DropdownItem::new("Item 1", 1),
///     DropdownItem::new("Item 2", 2),
/// ];
///
/// Dropdown::new("Select", items)
///     .build()
///     .on_select(|item| {
///         println!("Selected: {:?}", item);
///     })
/// ```

/// Dropdown item
#[derive(Clone, Debug)]
pub struct DropdownItem {
    /// Display label
    pub label: String,
    /// Optional value/id
    pub id: String,
}

impl DropdownItem {
    /// Create a new dropdown item
    pub fn new(label: impl Into<String>, id: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            id: id.into(),
        }
    }

    /// Create a dropdown item with label as id
    pub fn from_label(label: impl Into<String>) -> Self {
        let label = label.into();
        Self {
            label: label.clone(),
            id: label,
        }
    }
}

/// Dropdown component builder
pub struct Dropdown {
    label: String,
    items: Vec<DropdownItem>,
    placeholder: String,
    disabled: bool,
    max_height: Pixels,
}

impl Dropdown {
    /// Create a new dropdown
    pub fn new(label: impl Into<String>, items: Vec<DropdownItem>) -> Self {
        Self {
            label: label.into(),
            items,
            placeholder: String::new(),
            disabled: false,
            max_height: px(300.),
        }
    }

    /// Set the placeholder text
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Set the disabled state
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set the maximum height
    pub fn max_height(mut self, height: Pixels) -> Self {
        self.max_height = height;
        self
    }

    /// Build the dropdown - returns the trigger element
    pub fn build(self) -> Div {
        let label = self.label;
        let disabled = self.disabled;
        let item_count = self.items.len();

        div()
            .px_3()
            .py_2()
            .min_w(px(120.))
            .border_1()
            .border_color(rgb(0x45475a))
            .rounded(px(4.))
            .cursor_pointer()
            .when(disabled, |el| el.opacity(0.5).cursor_default())
            .when(!disabled, |el| el.hover(|style| style.bg(rgb(0x313244))))
            .flex()
            .items_center()
            .justify_between()
            .child(
                div()
                    .text_sm()
                    .text_color(if disabled {
                        rgb(0x6c7086)
                    } else {
                        rgb(0xcdd6f4)
                    })
                    .child(if label.is_empty() {
                        self.placeholder.clone()
                    } else {
                        label
                    }),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(0x6c7086))
                    .child(if item_count > 0 { "▼" } else { "" }),
            )
    }
}

/// Convenience function to create a dropdown from a simple list of strings
pub fn simple_dropdown(label: impl Into<String>, items: Vec<impl Into<String>>) -> Dropdown {
    let items = items
        .into_iter()
        .enumerate()
        .map(|(i, label)| DropdownItem {
            label: label.into(),
            id: i.to_string(),
        })
        .collect();

    Dropdown::new(label, items)
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
    fn test_dropdown_item_from_label() {
        let item = DropdownItem::from_label("Single");
        assert_eq!(item.label, "Single");
        assert_eq!(item.id, "Single");
    }

    #[test]
    fn test_dropdown_creation() {
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

    #[test]
    fn test_simple_dropdown() {
        let dropdown = simple_dropdown("Numbers", vec!["One", "Two", "Three"]);
        assert_eq!(dropdown.items.len(), 3);
        assert_eq!(dropdown.items[0].label, "One");
        assert_eq!(dropdown.items[0].id, "0");
    }
}
