//! Common rendering utilities and abstract components
//!
//! This module contains highly reusable, abstract rendering components
//! used across different views. Each component is designed to be flexible
//! and composable.

use gpui::{prelude::*, *};
use std::rc::Rc;

// ============================================================================
// Table Components
// ============================================================================

/// Table column configuration
pub struct TableColumn {
    pub label: String,
    pub width: TableColumnWidth,
    pub align: ColumnAlign,
}

impl TableColumn {
    pub fn new(label: impl Into<String>, width: TableColumnWidth) -> Self {
        Self {
            label: label.into(),
            width,
            align: ColumnAlign::Left,
        }
    }

    pub fn align(mut self, align: ColumnAlign) -> Self {
        self.align = align;
        self
    }

    pub fn fixed(label: impl Into<String>, width: Pixels) -> Self {
        Self::new(label, TableColumnWidth::Fixed(width))
    }

    pub fn flex(label: impl Into<String>) -> Self {
        Self::new(label, TableColumnWidth::Flex)
    }

    pub fn auto(label: impl Into<String>) -> Self {
        Self::new(label, TableColumnWidth::Auto)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TableColumnWidth {
    Fixed(Pixels),
    Flex,
    Auto,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ColumnAlign {
    Left,
    Center,
    Right,
}

/// Render a table header with columns
///
/// # Example
/// ```rust
/// let columns = vec![
///     TableColumn::fixed("#", px(60.)),
///     TableColumn::fixed("TIME", px(120.)),
///     TableColumn::flex("DATA"),
/// ];
/// render_table_header(columns, None)
/// ```
pub fn render_table_header(columns: Vec<TableColumn>, extra: Option<Div>) -> Div {
    let mut header = div()
        .w_full()
        .h(px(28.))
        .bg(rgb(0x1f1f1f))
        .border_b_1()
        .border_color(rgb(0x2a2a2a))
        .flex()
        .items_center()
        .text_xs()
        .font_weight(FontWeight::MEDIUM)
        .text_color(rgb(0x9ca3af));

    for column in columns {
        header = match column.width {
            TableColumnWidth::Fixed(width) => header.child(render_header_cell(
                column.label.clone(),
                width,
                column.align,
                None,
            )),
            TableColumnWidth::Flex => header.child(
                div()
                    .flex_1()
                    .px_2()
                    .py_1()
                    .flex()
                    .items_center()
                    .whitespace_nowrap()
                    .overflow_hidden()
                    .when(column.align == ColumnAlign::Center, |div| {
                        div.justify_center()
                    })
                    .child(column.label),
            ),
            TableColumnWidth::Auto => header.child(
                div()
                    .px_2()
                    .py_1()
                    .flex()
                    .items_center()
                    .flex_shrink_0()
                    .whitespace_nowrap()
                    .overflow_hidden()
                    .child(column.label),
            ),
        };
    }

    if let Some(extra_div) = extra {
        header = header.child(extra_div);
    }

    header
}

fn render_header_cell(
    label: String,
    width: Pixels,
    align: ColumnAlign,
    action: Option<Div>,
) -> Div {
    let mut cell = div()
        .w(width)
        .px_2()
        .py_1()
        .flex()
        .items_center()
        .flex_shrink_0()
        .whitespace_nowrap()
        .overflow_hidden();

    cell = match align {
        ColumnAlign::Left => cell,
        ColumnAlign::Center => cell.justify_center(),
        ColumnAlign::Right => cell.justify_end(),
    };

    let content = if let Some(action_div) = action {
        div()
            .flex()
            .items_center()
            .gap_1()
            .child(label.clone())
            .child(action_div)
    } else {
        div().child(label.clone())
    };

    cell.child(content)
}

// ============================================================================
// Filter Components
// ============================================================================

/// Filter toggle button state
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FilterState {
    pub is_active: bool,
    pub has_value: bool,
}

impl FilterState {
    pub fn inactive() -> Self {
        Self {
            is_active: false,
            has_value: false,
        }
    }

    pub fn active() -> Self {
        Self {
            is_active: true,
            has_value: true,
        }
    }

    pub fn with_value(has_value: bool) -> Self {
        Self {
            is_active: false,
            has_value,
        }
    }
}

/// Render a compact filter toggle button (gear icon style)
///
/// This is a simplified version that returns a Div builder
/// The caller is responsible for attaching the event handler
pub fn render_filter_toggle_base(state: FilterState, tooltip: Option<String>) -> Div {
    let icon = if state.has_value { "✓" } else { "⚙" };
    let color = if state.has_value {
        rgb(0x60a5fa)
    } else {
        rgb(0x4b5563)
    };

    let mut button = div()
        .text_xs()
        .cursor_pointer()
        .text_color(color)
        .hover(|style| style.bg(rgb(0x374151)))
        .rounded(px(2.))
        .ml_0p5()
        .pl_0()
        .pr_0()
        .py_0p5()
        .child(icon);

    if let Some(text) = tooltip {
        button = button.child(
            div()
                .text_xs()
                .text_color(rgb(0x6b7280))
                .child(text.clone()),
        );
    }

    button
}

/// Filter dropdown configuration
pub struct FilterDropdownConfig {
    pub items: Vec<FilterDropdownItem>,
    pub selected_index: Option<usize>,
    pub on_select: Rc<dyn Fn(usize) + 'static>,
    pub width: Pixels,
    pub max_height: Pixels,
}

impl Default for FilterDropdownConfig {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            selected_index: None,
            on_select: Rc::new(|_| {}),
            width: px(150.),
            max_height: px(300.),
        }
    }
}

impl FilterDropdownConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn items(mut self, items: Vec<FilterDropdownItem>) -> Self {
        self.items = items;
        self
    }

    pub fn selected(mut self, index: Option<usize>) -> Self {
        self.selected_index = index;
        self
    }

    pub fn on_select<F>(mut self, f: F) -> Self
    where
        F: Fn(usize) + 'static,
    {
        self.on_select = Rc::new(f);
        self
    }

    pub fn width(mut self, width: Pixels) -> Self {
        self.width = width;
        self
    }

    pub fn max_height(mut self, height: Pixels) -> Self {
        self.max_height = height;
        self
    }
}

#[derive(Clone, Debug)]
pub struct FilterDropdownItem {
    pub label: String,
    pub value: String,
}

impl FilterDropdownItem {
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
        }
    }

    pub fn simple(value: impl Into<String>) -> Self {
        let v = value.into();
        Self::new(v.clone(), v)
    }
}

/// Render a filter dropdown overlay
///
/// # Example
/// ```rust
/// let config = FilterDropdownConfig::new()
///     .items(vec![
///         FilterDropdownItem::simple("Option 1"),
///         FilterDropdownItem::simple("Option 2"),
///     ])
///     .selected(Some(0))
///     .on_select(|index| {
///         println!("Selected: {}", index);
///     });
/// render_filter_dropdown(px(100.), px(50.), config)
/// ```
pub fn render_filter_dropdown(left: Pixels, top: Pixels, config: FilterDropdownConfig) -> Div {
    let items = config.items.clone();
    let selected = config.selected_index;
    let on_select = config.on_select.clone();

    div()
        .absolute()
        .left(left)
        .top(top)
        .w(config.width)
        .h(config.max_height)
        .bg(rgb(0x1f2937))
        .border_1()
        .border_color(rgb(0x3b82f6))
        .rounded(px(4.))
        .shadow_lg()
        .flex()
        .flex_col()
        .overflow_hidden()
        .child(
            div().w_full().h_full().child(
                div().w_full().flex().flex_col().children(
                    items
                        .iter()
                        .enumerate()
                        .map(|(index, item)| {
                            let is_selected = selected == Some(index);
                            let on_select = on_select.clone();
                            div()
                                .w_full()
                                .px_3()
                                .py_2()
                                .text_sm()
                                .text_color(rgb(0xffffff))
                                .hover(|style| style.bg(rgb(0x374151)))
                                .when(is_selected, |div| div.bg(rgb(0x3b82f6)))
                                .cursor_pointer()
                                .on_mouse_down(MouseButton::Left, move |_event, _window, _cx| {
                                    on_select(index);
                                })
                                .child(item.label.clone())
                        })
                        .collect::<Vec<_>>(),
                ),
            ),
        )
}

// ============================================================================
// Status & Badge Components
// ============================================================================

/// Status indicator variant
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StatusVariant {
    Success,
    Warning,
    Error,
    Info,
    Neutral,
}

impl StatusVariant {
    fn color(self) -> Rgba {
        match self {
            StatusVariant::Success => rgb(0x10b981),
            StatusVariant::Warning => rgb(0xf59e0b),
            StatusVariant::Error => rgb(0xef4444),
            StatusVariant::Info => rgb(0x3b82f6),
            StatusVariant::Neutral => rgb(0x6b7280),
        }
    }

    fn bg_color(self) -> Rgba {
        match self {
            StatusVariant::Success => rgb(0x064e3b),
            StatusVariant::Warning => rgb(0x78350f),
            StatusVariant::Error => rgb(0x7f1d1d),
            StatusVariant::Info => rgb(0x1e3a8a),
            StatusVariant::Neutral => rgb(0x374151),
        }
    }
}

/// Render a status badge
///
/// # Example
/// ```rust
/// render_status_badge("Active", StatusVariant::Success)
/// render_status_badge("Warning", StatusVariant::Warning)
/// ```
pub fn render_status_badge(label: String, variant: StatusVariant) -> Div {
    div()
        .px_2()
        .py_0p5()
        .rounded(px(4.))
        .text_xs()
        .font_weight(FontWeight::MEDIUM)
        .bg(variant.bg_color())
        .text_color(variant.color())
        .child(label.clone())
}

/// Render a compact status indicator (dot + label)
pub fn render_status_indicator(label: String, variant: StatusVariant) -> Div {
    div()
        .flex()
        .items_center()
        .gap_1()
        .child(div().w(px(6.)).h(px(6.)).rounded_full().bg(variant.color()))
        .child(
            div()
                .text_sm()
                .text_color(rgb(0x9ca3af))
                .child(label.clone()),
        )
}

// ============================================================================
// Layout Components
// ============================================================================

/// Card container style
pub struct CardStyle {
    pub background: Rgba,
    pub border_color: Rgba,
    pub border_radius: Pixels,
    pub padding: Pixels,
}

impl Default for CardStyle {
    fn default() -> Self {
        Self {
            background: rgb(0x1f1f1f),
            border_color: rgb(0x2a2a2a),
            border_radius: px(8.),
            padding: px(16.),
        }
    }
}

impl CardStyle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bordered(mut self) -> Self {
        self.border_color = rgb(0x3b82f6);
        self
    }

    pub fn flat(mut self) -> Self {
        self.border_radius = px(0.);
        self
    }

    pub fn padding(mut self, padding: Pixels) -> Self {
        self.padding = padding;
        self
    }
}

/// Render a card container
///
/// # Example
/// ```rust
/// render_card(CardStyle::new(), |card| {
///     card.child("Content inside card")
/// })
/// ```
pub fn render_card<F>(style: CardStyle, content: F) -> Div
where
    F: FnOnce(Div) -> Div,
{
    let card = div()
        .bg(style.background)
        .border_1()
        .border_color(style.border_color)
        .rounded(style.border_radius)
        .px(style.padding)
        .py(style.padding);

    content(card)
}

/// Section header with title and optional actions
///
/// # Example
/// ```rust
/// render_section_header("Settings", Some(
///     div().child("Actions")
/// ))
/// ```
pub fn render_section_header(title: String, actions: Option<Div>) -> Div {
    let mut header = div()
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
                .child(title.clone()),
        );

    if let Some(actions_div) = actions {
        header = header.child(actions_div);
    }

    header
}

// ============================================================================
// Empty State Components
// ============================================================================

/// Empty state configuration
pub struct EmptyStateConfig {
    pub icon: Option<&'static str>,
    pub title: String,
    pub description: Option<String>,
    pub action: Option<(&'static str, Box<dyn Fn() -> Div>)>,
}

impl EmptyStateConfig {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            icon: None,
            title: title.into(),
            description: None,
            action: None,
        }
    }

    pub fn icon(mut self, icon: &'static str) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn action<F>(mut self, label: &'static str, render: F) -> Self
    where
        F: Fn() -> Div + 'static,
    {
        self.action = Some((label, Box::new(render)));
        self
    }
}

/// Render an empty state placeholder
///
/// # Example
/// ```rust
/// render_empty_state(EmptyStateConfig::new("No messages")
///     .icon("📭")
///     .description("Load a BLF file to view messages")
/// )
/// ```
pub fn render_empty_state(config: EmptyStateConfig) -> Div {
    let mut content = div()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap_3();

    if let Some(icon) = config.icon {
        content = content.child(div().text_3xl().child(icon));
    }

    content = content.child(
        div()
            .text_lg()
            .text_color(rgb(0x6b7280))
            .child(config.title),
    );

    if let Some(description) = config.description {
        content = content.child(div().text_sm().text_color(rgb(0x4b5563)).child(description));
    }

    if let Some((label, render)) = config.action {
        content = content.child(render());
    }

    div()
        .flex_1()
        .flex()
        .items_center()
        .justify_center()
        .child(content)
}

// ============================================================================
// Loading & Skeleton Components
// ============================================================================

/// Render a skeleton loader (placeholder while content loads)
///
/// # Example
/// ```rust
/// render_skeleton_row(px(400.), px(22.))
/// ```
pub fn render_skeleton_row(width: Pixels, height: Pixels) -> Div {
    div().w(width).h(height).bg(rgb(0x374151)).rounded(px(4.))
}

/// Render a loading spinner
pub fn render_loading_spinner(size: Pixels) -> Div {
    div()
        .w(size)
        .h(size)
        .rounded_full()
        .border_2()
        .border_color(rgb(0x3b82f6))
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Conditional rendering helper
///
/// # Example
/// ```rust
/// render_when(some_condition.is_some(), |div| {
///     div.child("Show when condition is true")
/// })
/// ```
pub fn render_when<F>(condition: bool, builder: F) -> Option<Div>
where
    F: FnOnce(Div) -> Div,
{
    if condition {
        Some(builder(div()))
    } else {
        None
    }
}

/// Map over an optional value and render
///
/// # Example
/// ```rust
/// render_option(maybe_value, |value| {
///     div().child(value)
/// })
/// ```
pub fn render_option<T, F>(value: Option<T>, builder: F) -> Option<Div>
where
    F: FnOnce(T) -> Div,
{
    value.map(builder)
}

/// Wrap content in a scrollable container
///
/// # Example
/// ```rust
/// render_scrollable(|scroll| {
///     scroll.child("Long content")
/// })
/// ```
pub fn render_scrollable<F>(content: F) -> Div
where
    F: FnOnce(Div) -> Div,
{
    content(div().flex_1())
}
