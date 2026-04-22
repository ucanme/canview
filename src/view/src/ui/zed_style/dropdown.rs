//! Zed-style Dropdown Menu Component
//!
//! Provides a modern, accessible dropdown menu with smooth interactions
//! inspired by Zed editor's design language.

use crate::ui::theme::{colors, radius, spacing};
use gpui::{prelude::*, *};

/// Dropdown menu item
#[derive(Clone)]
pub struct DropdownItem {
    pub id: String,
    pub label: String,
    pub icon: Option<String>,
    pub shortcut: Option<String>,
    pub disabled: bool,
    pub separator: bool,
}

impl DropdownItem {
    /// Create a new dropdown item
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            shortcut: None,
            disabled: false,
            separator: false,
        }
    }

    /// Add an icon to the item
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Add a keyboard shortcut hint
    pub fn shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    /// Mark item as disabled
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Mark as separator item
    pub fn separator(mut self) -> Self {
        self.separator = true;
        self
    }
}

/// Dropdown menu state
#[derive(Clone)]
pub struct DropdownState {
    pub is_open: bool,
    pub selected_index: Option<usize>,
    pub hovered_index: Option<usize>,
}

impl Default for DropdownState {
    fn default() -> Self {
        Self {
            is_open: false,
            selected_index: None,
            hovered_index: None,
        }
    }
}

/// Simple dropdown builder for common use cases
pub struct SimpleDropdown {
    label: String,
    items: Vec<(String, String)>, // (id, label)
    selected_id: Option<String>,
    max_height: Option<Pixels>,
    min_width: Option<Pixels>,
}

impl SimpleDropdown {
    /// Create a new simple dropdown
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            items: Vec::new(),
            selected_id: None,
            max_height: Some(px(300.)),
            min_width: Some(px(180.)),
        }
    }

    /// Set dropdown items
    pub fn items(mut self, items: Vec<(String, String)>) -> Self {
        self.items = items;
        self
    }

    /// Set selected item ID
    pub fn selected(mut self, id: impl Into<String>) -> Self {
        self.selected_id = Some(id.into());
        self
    }

    /// Set maximum height
    pub fn max_height(mut self, height: Pixels) -> Self {
        self.max_height = Some(height);
        self
    }

    /// Set minimum width
    pub fn min_width(mut self, width: Pixels) -> Self {
        self.min_width = Some(width);
        self
    }

    /// Build the simple dropdown trigger button (menu needs to be rendered separately)
    pub fn build_trigger<App>(
        self,
        view: Entity<App>,
        on_toggle: impl Fn(&mut App, &mut Context<App>) + 'static,
    ) -> impl IntoElement
    where
        App: 'static,
    {
        let label = self.label;
        let selected_id = self.selected_id;
        let min_width = self.min_width.unwrap_or(px(180.));

        let display_text = selected_id.clone().unwrap_or_else(|| label.clone());

        div()
            .px(spacing::SM)
            .py(spacing::XS)
            .min_w(min_width)
            .bg(colors::BG_ELEVATED)
            .border_1()
            .border_color(colors::BORDER_DEFAULT)
            .rounded(radius::MD)
            .flex()
            .items_center()
            .justify_between()
            .gap(spacing::SM)
            .cursor_pointer()
            .hover(|style| style.border_color(colors::BORDER_HOVER))
            .child(
                div()
                    .text_sm()
                    .text_color(colors::TEXT_PRIMARY)
                    .child(display_text),
            )
            .child(div().text_sm().text_color(colors::TEXT_MUTED).child("▼"))
            .on_mouse_down(MouseButton::Left, move |_event, _window, cx| {
                view.update(cx, |app, cx| {
                    on_toggle(app, cx);
                });
            })
    }
}

impl Default for SimpleDropdown {
    fn default() -> Self {
        Self::new("Select...")
    }
}

/// Dropdown menu content (popup menu)
pub fn render_dropdown_menu<App>(
    items: Vec<(String, String)>,
    max_height: Pixels,
    min_width: Pixels,
    view: Entity<App>,
    on_select: impl Fn(&mut App, String, &mut Context<App>) + 'static,
) -> impl IntoElement
where
    App: 'static,
{
    div()
        .min_w(min_width)
        .max_h(max_height)
        .bg(colors::BG_ELEVATED)
        .border_1()
        .border_color(colors::BORDER_DEFAULT)
        .rounded(radius::LG)
        .overflow_hidden()
        .flex()
        .flex_col()
        .py(spacing::XS)
        .children(
            items
                .iter()
                .enumerate()
                .map(|(index, (id, label))| {
                    let id = id.clone();
                    let label = label.clone();
                    let view = view.clone();

                    div()
                        .w_full()
                        .min_h(px(32.))
                        .px(spacing::SM)
                        .flex()
                        .items_center()
                        .gap(spacing::SM)
                        .rounded(radius::SM)
                        .mx(spacing::XS)
                        .cursor_pointer()
                        .text_color(colors::TEXT_PRIMARY)
                        .hover(|style| style.bg(colors::BG_ACTIVE))
                        .child(div().text_sm().child(label))
                        .on_mouse_down(MouseButton::Left, move |_event, _window, cx| {
                            view.update(cx, |app, cx| {
                                on_select(app, id.clone(), cx);
                            });
                        })
                        .into_any_element()
                })
                .collect::<Vec<_>>(),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dropdown_item_creation() {
        let item = DropdownItem::new("1", "Item 1")
            .icon("📄")
            .shortcut("Ctrl+S")
            .disabled(false);

        assert_eq!(item.id, "1");
        assert_eq!(item.label, "Item 1");
        assert_eq!(item.icon, Some("📄".to_string()));
        assert_eq!(item.shortcut, Some("Ctrl+S".to_string()));
        assert!(!item.disabled);
    }

    #[test]
    fn test_dropdown_item_separator() {
        let item = DropdownItem::new("sep", "").separator();
        assert!(item.separator);
    }

    #[test]
    fn test_simple_dropdown() {
        let dropdown = SimpleDropdown::new("Choose option")
            .items(vec![
                ("1".to_string(), "Option 1".to_string()),
                ("2".to_string(), "Option 2".to_string()),
            ])
            .selected("1")
            .max_height(px(200.));

        assert_eq!(dropdown.label, "Choose option");
        assert_eq!(dropdown.items.len(), 2);
        assert_eq!(dropdown.selected_id, Some("1".to_string()));
    }

    #[test]
    fn test_dropdown_state_default() {
        let state = DropdownState::default();
        assert!(!state.is_open);
        assert!(state.selected_index.is_none());
        assert!(state.hovered_index.is_none());
    }
}
