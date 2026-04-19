//! Tabs component
//!
//! A reusable tab navigation component for switching between views.

use gpui::{prelude::*, *};

/// Tab alignment
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TabAlignment {
    Start,
    Center,
    End,
}

/// Tab item representation
#[derive(Clone, Debug)]
pub struct TabItem<T> {
    pub label: String,
    pub value: T,
    pub icon: Option<String>,
}

impl<T> TabItem<T> {
    /// Create a new tab item
    pub fn new(label: impl Into<String>, value: T) -> Self {
        Self {
            label: label.into(),
            value,
            icon: None,
        }
    }

    /// Set the tab icon
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }
}

/// Tabs configuration
#[derive(Clone, Copy, Debug)]
pub struct TabsConfig {
    pub alignment: TabAlignment,
    pub show_divider: bool,
    pub indicator_color: u32,
    pub active_color: u32,
    pub inactive_color: u32,
}

impl Default for TabsConfig {
    fn default() -> Self {
        Self {
            alignment: TabAlignment::Start,
            show_divider: true,
            indicator_color: 0x89b4fa, // Blue
            active_color: 0xcdd6f4,    // Light text
            inactive_color: 0x6c7086,  // Muted text
        }
    }
}

/// Tabs component
pub struct Tabs<T> {
    tabs: Vec<TabItem<T>>,
    selected: T,
    config: TabsConfig,
}

impl<T: Clone + PartialEq + 'static> Tabs<T> {
    /// Create a new tabs component
    pub fn new(tabs: Vec<TabItem<T>>, selected: T) -> Self {
        Self {
            tabs,
            selected,
            config: TabsConfig::default(),
        }
    }

    /// Set the tab alignment
    pub fn alignment(mut self, alignment: TabAlignment) -> Self {
        self.config.alignment = alignment;
        self
    }

    /// Show/hide the divider
    pub fn show_divider(mut self, show: bool) -> Self {
        self.config.show_divider = show;
        self
    }

    /// Set the indicator color
    pub fn indicator_color(mut self, color: u32) -> Self {
        self.config.indicator_color = color;
        self
    }

    /// Set the active color
    pub fn active_color(mut self, color: u32) -> Self {
        self.config.active_color = color;
        self
    }

    /// Set the inactive color
    pub fn inactive_color(mut self, color: u32) -> Self {
        self.config.inactive_color = color;
        self
    }

    /// Set the tabs configuration
    pub fn config(mut self, config: TabsConfig) -> Self {
        self.config = config;
        self
    }

    /// Build the tabs element (visual only, no click handlers)
    pub fn build(self) -> Div {
        let config = self.config;
        let selected = self.selected;
        let indicator_color = config.indicator_color;
        let active_color = config.active_color;
        let inactive_color = config.inactive_color;

        let tabs_container = div()
            .flex()
            .flex_row()
            .when(config.alignment == TabAlignment::Center, |div| {
                div.justify_center()
            })
            .when(config.alignment == TabAlignment::End, |div| {
                div.justify_end()
            })
            .gap_2()
            .px_4()
            .py_2()
            .w_full();

        let tabs_with_divider = tabs_container.children(self.tabs.into_iter().map(|tab| {
            let tab_value = tab.value.clone();
            let tab_label = tab.label.clone();
            let tab_icon = tab.icon.clone();
            let is_selected = tab_value == selected;

            // Build icon element if present
            let icon_element = if let Some(icon) = tab_icon {
                Some(div().text_sm().child(icon).into_any())
            } else {
                None
            };

            // Build label element
            let label_color = if is_selected {
                rgb(active_color)
            } else {
                rgb(inactive_color)
            };

            let label_element = div()
                .text_sm()
                .font_weight(FontWeight::MEDIUM)
                .text_color(label_color)
                .child(tab_label)
                .into_any();

            // Combine children
            let children: Vec<AnyElement> = [icon_element, Some(label_element)]
                .into_iter()
                .filter_map(|x| x)
                .collect();

            // Build indicator
            let indicator = if is_selected {
                Some(
                    div()
                        .absolute()
                        .bottom_0()
                        .left_0()
                        .w_full()
                        .h(px(2.0))
                        .bg(rgb(indicator_color))
                        .rounded_b_lg()
                        .into_any(),
                )
            } else {
                None
            };

            // Build tab element
            let mut tab_div = div()
                .flex()
                .items_center()
                .gap_2()
                .px_4()
                .py_2()
                .rounded_lg()
                .cursor_pointer()
                .relative()
                .hover(|style| style.bg(rgba(0x00000008)));

            // Add indicator if present
            if let Some(ind) = indicator {
                tab_div = tab_div.child(ind);
            }

            // Add all children
            for child in children {
                tab_div = tab_div.child(child);
            }

            tab_div.into_any()
        }));

        if config.show_divider {
            tabs_with_divider.border_b_1().border_color(rgb(0x313244))
        } else {
            tabs_with_divider
        }
    }

    /// Get the current selected value
    pub fn selected(&self) -> &T {
        &self.selected
    }

    /// Get all tabs
    pub fn tabs(&self) -> &[TabItem<T>] {
        &self.tabs
    }
}

/// Convenience function to create simple tabs
pub fn simple_tabs<T: Clone + PartialEq + 'static>(tabs: Vec<(String, T)>, selected: T) -> Tabs<T> {
    let tab_items: Vec<TabItem<T>> = tabs
        .into_iter()
        .map(|(label, value)| TabItem::new(label, value))
        .collect();
    Tabs::new(tab_items, selected)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_item_creation() {
        let tab = TabItem::new("Test", "value");
        assert_eq!(tab.label, "Test");
        assert_eq!(tab.value, "value");
        assert!(tab.icon.is_none());
    }

    #[test]
    fn test_tab_item_with_icon() {
        let tab = TabItem::new("Test", "value").icon("🔥");
        assert_eq!(tab.icon, Some("🔥".to_string()));
    }

    #[test]
    fn test_tabs_config_default() {
        let config = TabsConfig::default();
        assert_eq!(config.alignment, TabAlignment::Start);
        assert!(config.show_divider);
        assert_eq!(config.indicator_color, 0x89b4fa);
    }

    #[test]
    fn test_tabs_creation() {
        let tabs = vec![
            TabItem::new("Tab 1", "1"),
            TabItem::new("Tab 2", "2"),
            TabItem::new("Tab 3", "3"),
        ];
        let tabs_component = Tabs::new(tabs, "2");
        assert_eq!(tabs_component.selected, "2");
        assert_eq!(tabs_component.tabs.len(), 3);
    }

    #[test]
    fn test_tabs_builder() {
        let tabs = vec![TabItem::new("Tab 1", "1"), TabItem::new("Tab 2", "2")];
        let tabs_component = Tabs::new(tabs, "1")
            .alignment(TabAlignment::Center)
            .show_divider(false)
            .indicator_color(0xff0000);

        assert_eq!(tabs_component.config.alignment, TabAlignment::Center);
        assert!(!tabs_component.config.show_divider);
        assert_eq!(tabs_component.config.indicator_color, 0xff0000);
    }

    #[test]
    fn test_simple_tabs() {
        let tabs = vec![
            ("One".to_string(), 1),
            ("Two".to_string(), 2),
            ("Three".to_string(), 3),
        ];
        let tabs_component = simple_tabs(tabs, 2);
        assert_eq!(tabs_component.selected, 2);
        assert_eq!(tabs_component.tabs.len(), 3);
        assert_eq!(tabs_component.tabs[0].label, "One");
        assert_eq!(tabs_component.tabs[0].value, 1);
    }
}
