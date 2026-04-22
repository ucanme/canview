//! Tabs component usage examples
//!
//! This file demonstrates various ways to use the Tabs component.

use crate::ui::components::tabs::*;
use gpui::{prelude::*, *};

/// Example 1: Simple tabs with string values
///
/// This is the simplest way to create tabs.
pub fn example_simple_tabs() -> Tabs<String> {
    simple_tabs(
        vec![
            ("Log View".to_string(), "log".to_string()),
            ("Config View".to_string(), "config".to_string()),
            ("Library View".to_string(), "library".to_string()),
        ],
        "log".to_string(),
    )
}

/// Example 2: Tabs with custom configuration
///
/// Shows how to customize the appearance of tabs.
pub fn example_custom_tabs() -> Tabs<&'static str> {
    let tabs = vec![
        TabItem::new("Dashboard", "dashboard"),
        TabItem::new("Analytics", "analytics"),
        TabItem::new("Settings", "settings"),
    ];

    Tabs::new(tabs, "dashboard")
        .alignment(TabAlignment::Center)
        .show_divider(true)
        .indicator_color(0x89b4fa)
        .active_color(0xcdd6f4)
        .inactive_color(0x6c7086)
}

/// Example 3: Tabs with icons
///
/// Shows how to add icons to tabs.
pub fn example_tabs_with_icons() -> Tabs<&'static str> {
    let tabs = vec![
        TabItem::new("Home", "home").icon("🏠"),
        TabItem::new("Messages", "messages").icon("💬"),
        TabItem::new("Settings", "settings").icon("⚙️"),
    ];

    Tabs::new(tabs, "home").alignment(TabAlignment::Start)
}

/// Example 4: Center-aligned tabs
///
/// Shows tabs centered in their container.
pub fn example_centered_tabs() -> Tabs<&'static str> {
    let tabs = vec![
        TabItem::new("Day", "day"),
        TabItem::new("Week", "week"),
        TabItem::new("Month", "month"),
        TabItem::new("Year", "year"),
    ];

    Tabs::new(tabs, "week")
        .alignment(TabAlignment::Center)
        .show_divider(false)
}

/// Example 5: End-aligned tabs
///
/// Shows tabs aligned to the right/end.
pub fn example_end_aligned_tabs() -> Tabs<&'static str> {
    let tabs = vec![
        TabItem::new("List", "list"),
        TabItem::new("Grid", "grid"),
        TabItem::new("Tree", "tree"),
    ];

    Tabs::new(tabs, "list").alignment(TabAlignment::End)
}

/// Example 6: Tabs without divider
///
/// Shows tabs without the bottom divider line.
pub fn example_tabs_without_divider() -> Tabs<&'static str> {
    let tabs = vec![
        TabItem::new("Overview", "overview"),
        TabItem::new("Details", "details"),
    ];

    Tabs::new(tabs, "overview").show_divider(false)
}

/// Example 7: Custom colored tabs
///
/// Shows tabs with custom colors for different states.
pub fn example_custom_colored_tabs() -> Tabs<&'static str> {
    let tabs = vec![
        TabItem::new("Active", "active"),
        TabItem::new("Pending", "pending"),
        TabItem::new("Completed", "completed"),
    ];

    Tabs::new(tabs, "active")
        .indicator_color(0x10b981) // Green indicator
        .active_color(0x10b981) // Green active text
        .inactive_color(0x6c7086) // Gray inactive text
        .hover_color(0x34d399) // Light green hover
}

/// Example 8: Integration with app state
///
/// Shows how to integrate tabs with application view state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AppView {
    LogView,
    ConfigView,
    LibraryView,
    ChartView,
}

pub struct AppTabsState {
    pub current_view: AppView,
}

impl AppTabsState {
    pub fn new() -> Self {
        Self {
            current_view: AppView::LogView,
        }
    }

    pub fn render_tabs<App>(&self, cx: &mut Context<App>) -> Div
    where
        App: 'static,
    {
        let tabs = vec![
            TabItem::new("Log View", AppView::LogView),
            TabItem::new("Config", AppView::ConfigView),
            TabItem::new("Library", AppView::LibraryView),
            TabItem::new("Charts", AppView::ChartView),
        ];

        Tabs::new(tabs, self.current_view)
            .alignment(TabAlignment::Start)
            .show_divider(true)
            .build(cx.listener(|state, tab_value, _window, cx| {
                state.current_view = *tab_value;
                cx.notify();
            }))
    }

    pub fn render_content<App>(&self, cx: &mut Context<App>) -> Div
    where
        App: 'static,
    {
        match self.current_view {
            AppView::LogView => div().child("Log View Content"),
            AppView::ConfigView => div().child("Configuration Content"),
            AppView::LibraryView => div().child("Library Management Content"),
            AppView::ChartView => div().child("Chart Visualization Content"),
        }
    }
}

impl Default for AppTabsState {
    fn default() -> Self {
        Self::new()
    }
}

/// Example 9: Filter tabs
///
/// Shows tabs used for filtering content.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FilterType {
    All,
    Can,
    Lin,
    Error,
}

pub struct FilterTabsState {
    pub current_filter: FilterType,
}

impl FilterTabsState {
    pub fn new() -> Self {
        Self {
            current_filter: FilterType::All,
        }
    }

    pub fn render<App>(&self, cx: &mut Context<App>) -> Div
    where
        App: 'static,
    {
        let tabs = vec![
            TabItem::new("All", FilterType::All),
            TabItem::new("CAN", FilterType::Can),
            TabItem::new("LIN", FilterType::Lin),
            TabItem::new("Errors", FilterType::Error),
        ];

        Tabs::new(tabs, self.current_filter)
            .alignment(TabAlignment::Center)
            .show_divider(false)
            .build(cx.listener(|state, tab_value, _window, cx| {
                state.current_filter = *tab_value;
                cx.notify();
            }))
    }

    pub fn should_show_message(&self, is_can: bool, is_lin: bool, is_error: bool) -> bool {
        match self.current_filter {
            FilterType::All => true,
            FilterType::Can => is_can,
            FilterType::Lin => is_lin,
            FilterType::Error => is_error,
        }
    }
}

impl Default for FilterTabsState {
    fn default() -> Self {
        Self::new()
    }
}

/// Example 10: Time range tabs
///
/// Shows tabs for selecting time ranges in charts.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TimeRange {
    OneMinute,
    FiveMinutes,
    FifteenMinutes,
    OneHour,
}

pub fn example_time_range_tabs() -> Tabs<TimeRange> {
    let tabs = vec![
        TabItem::new("1m", TimeRange::OneMinute),
        TabItem::new("5m", TimeRange::FiveMinutes),
        TabItem::new("15m", TimeRange::FifteenMinutes),
        TabItem::new("1h", TimeRange::OneHour),
    ];

    Tabs::new(tabs, TimeRange::FiveMinutes)
        .alignment(TabAlignment::End)
        .show_divider(false)
}

/// Example 11: Channel tabs
///
/// Shows tabs for selecting different channels.
pub fn example_channel_tabs() -> Tabs<usize> {
    let tabs = vec![
        TabItem::new("CH 1", 1).icon("1️⃣"),
        TabItem::new("CH 2", 2).icon("2️⃣"),
        TabItem::new("CH 3", 3).icon("3️⃣"),
        TabItem::new("CH 4", 4).icon("4️⃣"),
    ];

    Tabs::new(tabs, 1)
        .alignment(TabAlignment::Start)
        .show_divider(true)
}

/// Example 12: Compact tabs for toolbars
///
/// Shows a compact tab layout suitable for toolbars.
pub fn example_compact_tabs() -> Tabs<&'static str> {
    let tabs = vec![
        TabItem::new("Edit", "edit"),
        TabItem::new("View", "view"),
        TabItem::new("Help", "help"),
    ];

    Tabs::new(tabs, "edit")
        .alignment(TabAlignment::Start)
        .show_divider(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_view_tabs() {
        let state = AppTabsState::new();
        assert_eq!(state.current_view, AppView::LogView);

        let tabs = vec![
            TabItem::new("Log View", AppView::LogView),
            TabItem::new("Config", AppView::ConfigView),
            TabItem::new("Library", AppView::LibraryView),
            TabItem::new("Charts", AppView::ChartView),
        ];

        let tabs_component = Tabs::new(tabs, state.current_view);
        assert_eq!(tabs_component.selected, AppView::LogView);
        assert_eq!(tabs_component.tabs.len(), 4);
    }

    #[test]
    fn test_filter_tabs_logic() {
        let mut state = FilterTabsState::new();
        assert_eq!(state.current_filter, FilterType::All);

        // Test "All" filter - shows everything
        assert!(state.should_show_message(true, false, false));
        assert!(state.should_show_message(false, true, false));
        assert!(state.should_show_message(false, false, true));

        state.current_filter = FilterType::Can;

        // Test "CAN" filter - only shows CAN messages
        assert!(state.should_show_message(true, false, false));
        assert!(!state.should_show_message(false, true, false));
        assert!(!state.should_show_message(false, false, true));

        state.current_filter = FilterType::Error;

        // Test "Error" filter - only shows errors
        assert!(!state.should_show_message(true, false, false));
        assert!(!state.should_show_message(false, true, false));
        assert!(state.should_show_message(false, false, true));
    }

    #[test]
    fn test_time_range_tabs() {
        let tabs = example_time_range_tabs();
        assert_eq!(tabs.selected, TimeRange::FiveMinutes);
        assert_eq!(tabs.config.alignment, TabAlignment::End);
        assert!(!tabs.config.show_divider);
    }

    #[test]
    fn test_channel_tabs() {
        let tabs = example_channel_tabs();
        assert_eq!(tabs.selected, 1);
        assert_eq!(tabs.tabs.len(), 4);
        assert_eq!(tabs.tabs[0].label, "CH 1");
        assert_eq!(tabs.tabs[0].icon, Some("1️⃣".to_string()));
    }

    #[test]
    fn test_tab_building_patterns() {
        // Test with custom config
        let tabs = vec![
            TabItem::new("A", 1),
            TabItem::new("B", 2),
            TabItem::new("C", 3),
        ];

        let tabs_component = Tabs::new(tabs, 2)
            .alignment(TabAlignment::Center)
            .show_divider(false)
            .indicator_color(xff0000)
            .active_color(x00ff00)
            .inactive_color(x0000ff)
            .hover_color(xffff00);

        assert_eq!(tabs_component.config.alignment, TabAlignment::Center);
        assert!(!tabs_component.config.show_divider);
        assert_eq!(tabs_component.config.indicator_color, 0xff0000);
        assert_eq!(tabs_component.config.active_color, 0x00ff00);
        assert_eq!(tabs_component.config.inactive_color, 0x0000ff);
        assert_eq!(tabs_component.config.hover_color, 0xffff00);
    }

    #[test]
    fn test_simple_tabs_convenience() {
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
        assert!(tabs_component.tabs[0].icon.is_none());
    }
}
