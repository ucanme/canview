//! Common components usage examples
//!
//! This file demonstrates how to use the reusable components from `views/common.rs`
//! in various scenarios. Each example shows the component API and practical usage.

use crate::app::CanViewApp;
use crate::views::common::*;
use gpui::{prelude::*, *};

// ============================================================================
// Table Component Examples
// ============================================================================

/// Example 1: Simple table header
///
/// Demonstrates the most basic table header with fixed-width columns.
pub fn example_simple_table_header() -> Div {
    let columns = vec![
        TableColumn::fixed("#", px(60.)),
        TableColumn::fixed("Name", px(150.)),
        TableColumn::fixed("Value", px(100.)),
        TableColumn::flex("Description"),
    ];

    render_table_header(columns, None)
}

/// Example 2: Table header with mixed column types
///
/// Shows how to combine fixed, flex, and auto-width columns.
pub fn example_mixed_table_header() -> Div {
    let columns = vec![
        TableColumn::fixed("ID", px(60.)),
        TableColumn::fixed("Time", px(120.)),
        TableColumn::new("Channel", TableColumnWidth::Auto),
        TableColumn::flex("Data"),
    ];

    render_table_header(columns, None)
}

/// Example 3: Table header with extra actions
///
/// Demonstrates adding a search button or other actions to the header.
pub fn example_table_header_with_actions(view: Entity<CanViewApp>) -> Div {
    let columns = vec![
        TableColumn::fixed("#", px(60.)),
        TableColumn::fixed("Message", px(200.)),
        TableColumn::flex("Details"),
    ];

    let extra_actions = div()
        .flex()
        .items_center()
        .gap_2()
        .child(
            div()
                .text_xs()
                .cursor_pointer()
                .text_color(rgb(0x9ca3af))
                .hover(|style| style.text_color(rgb(0xffffff)))
                .child("🔍"),
        )
        .child(
            div()
                .text_xs()
                .cursor_pointer()
                .text_color(rgb(0x9ca3af))
                .hover(|style| style.text_color(rgb(0xffffff)))
                .child("⚙️"),
        );

    render_table_header(columns, Some(extra_actions))
}

// ============================================================================
// Filter Component Examples
// ============================================================================

/// Example 4: Filter toggle button (inactive state)
///
/// Shows a filter toggle button that is not currently active.
pub fn example_filter_toggle_inactive(view: Entity<CanViewApp>) -> Div {
    render_filter_toggle(FilterState::inactive(), None, move |_event, _window, cx| {
        view.update(cx, |app, cx| {
            // Toggle filter visibility
            cx.notify();
        });
    })
}

/// Example 5: Filter toggle button (active state)
///
/// Shows a filter toggle button with an active filter.
pub fn example_filter_toggle_active(view: Entity<CanViewApp>) -> Div {
    render_filter_toggle(
        FilterState::with_value(true),
        Some("Filter active"),
        move |_event, _window, cx| {
            view.update(cx, |app, cx| {
                // Clear filter
                app.id_filter = None;
                cx.notify();
            });
        },
    )
}

/// Example 6: Filter dropdown with items
///
/// Demonstrates a filter dropdown with multiple selectable items.
pub fn example_filter_dropdown(view: Entity<CanViewApp>) -> Div {
    let items = vec![
        FilterDropdownItem::new("Channel 1", "1"),
        FilterDropdownItem::new("Channel 2", "2"),
        FilterDropdownItem::new("Channel 3", "3"),
        FilterDropdownItem::simple("All Channels"),
    ];

    let config = FilterDropdownConfig::new()
        .items(items)
        .selected(Some(0))
        .width(px(150.))
        .max_height(px(200.))
        .on_select({
            let view = view.clone();
            move |index| {
                view.update(&mut |app, cx| {
                    // Handle selection
                    println!("Selected item: {}", index);
                    cx.notify();
                });
            }
        });

    div()
        .relative()
        .size_full()
        .child(
            div()
                .w(px(100.))
                .px_3()
                .py_2()
                .bg(rgb(0x374151))
                .rounded(px(4.))
                .cursor_pointer()
                .child("Show Dropdown"),
        )
        .child(render_filter_dropdown(px(0.), px(40.), config))
}

// ============================================================================
// Status Component Examples
// ============================================================================

/// Example 7: Status badges for different states
///
/// Shows all available status badge variants.
pub fn example_status_badges() -> Div {
    div()
        .flex()
        .flex_col()
        .gap_3()
        .p_4()
        .child(
            div()
                .flex()
                .items_center()
                .gap_3()
                .child("Success:")
                .child(render_status_badge("Connected", StatusVariant::Success)),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap_3()
                .child("Warning:")
                .child(render_status_badge("Unstable", StatusVariant::Warning)),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap_3()
                .child("Error:")
                .child(render_status_badge("Disconnected", StatusVariant::Error)),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap_3()
                .child("Info:")
                .child(render_status_badge("Processing", StatusVariant::Info)),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap_3()
                .child("Neutral:")
                .child(render_status_badge("Idle", StatusVariant::Neutral)),
        )
}

/// Example 8: Status indicators with dots
///
/// Shows status indicators with colored dots.
pub fn example_status_indicators() -> Div {
    div()
        .flex()
        .flex_col()
        .gap_3()
        .p_4()
        .child(render_status_indicator("System OK", StatusVariant::Success))
        .child(render_status_indicator(
            "High CPU usage",
            StatusVariant::Warning,
        ))
        .child(render_status_indicator(
            "Connection lost",
            StatusVariant::Error,
        ))
        .child(render_status_indicator(
            "Receiving data...",
            StatusVariant::Info,
        ))
}

// ============================================================================
// Card Component Examples
// ============================================================================

/// Example 9: Simple card
///
/// Shows a basic card with default styling.
pub fn example_simple_card() -> Div {
    render_card(CardStyle::new(), |card| {
        card.child(
            div()
                .text_lg()
                .font_weight(FontWeight::BOLD)
                .child("Card Title"),
        )
        .child(
            div()
                .text_sm()
                .text_color(rgb(0x9ca3af))
                .child("Card content goes here..."),
        )
    })
}

/// Example 10: Bordered card
///
/// Shows a card with a blue border for emphasis.
pub fn example_bordered_card() -> Div {
    render_card(CardStyle::new().bordered(), |card| {
        card.child(
            div()
                .text_lg()
                .font_weight(FontWeight::BOLD)
                .child("Important Card"),
        )
        .child(
            div()
                .text_sm()
                .text_color(rgb(0x9ca3af))
                .child("This card has a blue border"),
        )
    })
}

/// Example 11: Card with custom padding
///
/// Shows a card with reduced padding.
pub fn example_compact_card() -> Div {
    render_card(CardStyle::new().padding(px(8.)), |card| {
        card.child(div().text_sm().child("Compact card content"))
    })
}

/// Example 12: Multiple cards in a grid
///
/// Shows how to layout multiple cards.
pub fn example_card_grid() -> Div {
    div()
        .flex()
        .gap_4()
        .p_4()
        .child(example_simple_card())
        .child(example_bordered_card())
        .child(example_compact_card())
}

// ============================================================================
// Section Header Example
// ============================================================================

/// Example 13: Section header with actions
///
/// Shows a section header with title and action buttons.
pub fn example_section_header(view: Entity<CanViewApp>) -> Div {
    let actions = div()
        .flex()
        .items_center()
        .gap_2()
        .child(
            div()
                .text_sm()
                .cursor_pointer()
                .text_color(rgb(0xffffff))
                .child("Edit"),
        )
        .child(
            div()
                .text_sm()
                .cursor_pointer()
                .text_color(rgb(0x9ca3af))
                .child("Delete"),
        );

    render_section_header("Configuration", Some(actions))
}

// ============================================================================
// Empty State Examples
// ============================================================================

/// Example 14: Empty state with icon
///
/// Shows an empty state with an emoji icon.
pub fn example_empty_state_with_icon() -> Div {
    render_empty_state(
        EmptyStateConfig::new("No messages loaded")
            .icon("📭")
            .description("Click 'Open BLF' to load a log file"),
    )
}

/// Example 15: Empty state with action
///
/// Shows an empty state with a call-to-action button.
pub fn example_empty_state_with_action(view: Entity<CanViewApp>) -> Div {
    render_empty_state(
        EmptyStateConfig::new("No libraries configured")
            .icon("📚")
            .description("Create a library to get started")
            .action("Create Library", || {
                div()
                    .px_4()
                    .py_2()
                    .bg(rgb(0x3b82f6))
                    .rounded(px(4.))
                    .cursor_pointer()
                    .text_color(rgb(0xffffff))
                    .child("Create Library")
            }),
    )
}

// ============================================================================
// Conditional Rendering Examples
// ============================================================================

/// Example 16: Conditional rendering with render_when
///
/// Shows how to conditionally render content based on a boolean.
pub fn example_conditional_rendering(has_data: bool) -> Div {
    div()
        .flex()
        .flex_col()
        .gap_2()
        .child(render_when(has_data, |div| div.child("Data is available")))
        .child(render_when(!has_data, |div| div.child("No data available")))
}

/// Example 17: Option rendering with render_option
///
/// Shows how to render content that may or may not exist.
pub fn example_option_rendering(maybe_value: Option<String>) -> Div {
    div()
        .flex()
        .flex_col()
        .gap_2()
        .child(render_option(maybe_value.clone(), |value| {
            div().child(format!("Value: {}", value))
        }))
        .child(render_option(maybe_value, |_| {
            div().text_color(rgb(0x6b7280)).child("No value set")
        }))
}

// ============================================================================
// Scrollable Content Example
// ============================================================================

/// Example 18: Scrollable content container
///
/// Shows how to wrap content in a scrollable container.
pub fn example_scrollable_content() -> Div {
    render_scrollable(|scroll| {
        scroll.child(div().flex().flex_col().gap_2().children((0..50).map(|i| {
            div()
                .px_3()
                .py_2()
                .bg(rgb(0x1f1f1f))
                .child(format!("Item {}", i))
        })))
    })
}

// ============================================================================
// Complete Example: Combined Components
// ============================================================================

/// Example 19: Complete view using multiple components
///
/// Shows how to combine multiple common components into a complete view.
pub fn example_complete_view(view: Entity<CanViewApp>) -> Div {
    div()
        .size_full()
        .flex()
        .flex_col()
        .bg(rgb(0x0d0d0d))
        .child(example_section_header(view.clone()))
        .child(
            div()
                .flex_1()
                .flex()
                .flex_col()
                .gap_4()
                .p_4()
                .child(render_card(CardStyle::new(), |card| {
                    card.child(example_status_indicators())
                }))
                .child(
                    div()
                        .flex()
                        .gap_4()
                        .child(example_simple_card())
                        .child(example_bordered_card()),
                )
                .child(render_card(CardStyle::new(), |card| {
                    card.child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(example_simple_table_header())
                            .child(render_scrollable(|scroll| {
                                scroll.child(example_empty_state_with_icon())
                            })),
                    )
                })),
        )
}

// ============================================================================
// Migration Examples
// ============================================================================

/// Example 20: Before and after migration
///
/// Shows how to migrate existing code to use common components.

// BEFORE: Inline rendering (verbose, repetitive)
#[allow(dead_code)]
fn before_migration() -> Div {
    div()
        .w_full()
        .px_4()
        .py_3()
        .bg(rgb(0x1f1f1f))
        .border_b_1()
        .border_color(rgb(0x2a2a2a))
        .flex()
        .items_center()
        .justify_between()
        .child(
            div()
                .text_lg()
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(0xffffff))
                .child("Title"),
        )
}

// AFTER: Using common components (clean, reusable)
#[allow(dead_code)]
fn after_migration() -> Div {
    render_section_header("Title", None)
}

/// Example 21: Table header migration
///
/// Before: Manual table header construction (~50 lines)
/// After: Using render_table_header (~5 lines)

// BEFORE
#[allow(dead_code)]
fn table_header_before(time_width: Pixels) -> Div {
    div()
        .w_full()
        .h(px(28.))
        .bg(rgb(0x1f1f1f))
        .border_b_1()
        .border_color(rgb(0x2a2a2a))
        .flex()
        .items_center()
        .text_xs()
        .child(div().w(px(60.)).child("#"))
        .child(div().w(time_width).child("TIME"))
        .child(div().flex_1().child("DATA"))
}

// AFTER
#[allow(dead_code)]
fn table_header_after(time_width: Pixels) -> Div {
    let columns = vec![
        TableColumn::fixed("#", px(60.)),
        TableColumn::fixed("TIME", time_width),
        TableColumn::flex("DATA"),
    ];
    render_table_header(columns, None)
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_state_creation() {
        let inactive = FilterState::inactive();
        assert!(!inactive.is_active);
        assert!(!inactive.has_value);

        let active = FilterState::active();
        assert!(active.is_active);
        assert!(active.has_value);

        let with_value = FilterState::with_value(true);
        assert!(with_value.has_value);
    }

    #[test]
    fn test_table_column_creation() {
        let fixed = TableColumn::fixed("Test", px(100.));
        let flex = TableColumn::flex("Data");
        let auto = TableColumn::auto("Auto");

        match fixed.width {
            TableColumnWidth::Fixed(w) => assert_eq!(w, px(100.)),
            _ => panic!("Expected Fixed width"),
        }
        match flex.width {
            TableColumnWidth::Flex => {}
            _ => panic!("Expected Flex width"),
        }
        match auto.width {
            TableColumnWidth::Auto => {}
            _ => panic!("Expected Auto width"),
        }
    }

    #[test]
    fn test_empty_state_config_builder() {
        let config = EmptyStateConfig::new("No data")
            .icon("📭")
            .description("Test description");

        assert_eq!(config.title, "No data");
        assert_eq!(config.icon, Some("📭"));
        assert_eq!(config.description, Some("Test description".to_string()));
    }

    #[test]
    fn test_card_style_builder() {
        let default = CardStyle::new();
        assert_eq!(default.padding, px(16.));

        let custom = CardStyle::new().padding(px(32.));
        assert_eq!(custom.padding, px(32.));

        let bordered = CardStyle::new().bordered();
        assert_eq!(bordered.border_color, rgb(0x3b82f6));
    }
}

// ============================================================================
// Integration Example
// ============================================================================

/// Example 22: Real-world integration - Log view header
///
/// Shows how the common components can be used to build the actual log view header.
pub fn example_log_view_header_integration(
    view: Entity<CanViewApp>,
    time_width: Pixels,
    ch_width: Pixels,
    has_channel_filter: bool,
) -> Div {
    let columns = vec![
        TableColumn::fixed("#", px(60.)),
        TableColumn::fixed("TIME", time_width),
        TableColumn::new("CH", TableColumnWidth::Fixed(ch_width)),
        TableColumn::fixed("TYPE", px(50.)),
        TableColumn::fixed("ID", px(60.)),
        TableColumn::fixed("DLC", px(30.)),
        TableColumn::flex("DATA"),
        TableColumn::flex("SIGNALS"),
    ];

    let filter_toggle = render_filter_toggle(
        FilterState::with_value(has_channel_filter),
        None,
        move |_event, _window, cx| {
            view.update(cx, |app, cx| {
                // Toggle channel filter
                app.show_channel_filter_input = !app.show_channel_filter_input;
                cx.notify();
            });
        },
    );

    // Add filter toggle after CH column
    let extra = div().child(filter_toggle);

    render_table_header(columns, Some(extra))
}
