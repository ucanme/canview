//! UI controller
//!
//! Handles UI rendering logic and view composition.

use crate::app::{CanViewApp, LibraryDialogType};
use crate::models::ChannelMapping;
use crate::views::log_view::render_message_row_static_with_widths;
use blf::LogObject;
use gpui::{prelude::*, *};
use parser::dbc::DbcDatabase;
use parser::ldf::LdfDatabase;
use std::collections::HashMap;
use std::rc::Rc;

// ========== Filter Dropdown Types ==========

/// Generic filter dropdown configuration
pub enum FilterType {
    IdFilter(Vec<u32>),
    ChannelFilter(Vec<u16>),
}

// ========== Message Filtering ==========

/// Apply message filters based on current filter state
pub fn apply_message_filters(app: &CanViewApp) -> Vec<LogObject> {
    match (app.id_filter, app.channel_filter) {
        (None, None) => app.messages.clone(),
        (Some(filter_id), None) => app
            .messages
            .iter()
            .filter(|msg| matches!(msg, LogObject::CanMessage(m) if m.id == filter_id)
                || matches!(msg, LogObject::CanMessage2(m) if m.id == filter_id)
                || matches!(msg, LogObject::CanFdMessage(m) if m.id == filter_id)
                || matches!(msg, LogObject::CanFdMessage64(m) if m.id == filter_id)
                || matches!(msg, LogObject::LinMessage(m) if m.id as u32 == filter_id))
            .cloned()
            .collect(),
        (None, Some(filter_ch)) => app
            .messages
            .iter()
            .filter(|msg| matches!(msg, LogObject::CanMessage(m) if m.channel == filter_ch)
                || matches!(msg, LogObject::CanMessage2(m) if m.channel == filter_ch)
                || matches!(msg, LogObject::CanFdMessage(m) if m.channel == filter_ch)
                || matches!(msg, LogObject::CanFdMessage64(m) if m.channel as u16 == filter_ch)
                || matches!(msg, LogObject::LinMessage(m) if m.channel == filter_ch))
            .cloned()
            .collect(),
        (Some(filter_id), Some(filter_ch)) => app
            .messages
            .iter()
            .filter(|msg| {
                (matches!(msg, LogObject::CanMessage(m) if m.id == filter_id && m.channel == filter_ch)
                    || matches!(msg, LogObject::CanMessage2(m) if m.id == filter_id && m.channel == filter_ch)
                    || matches!(msg, LogObject::CanFdMessage(m) if m.id == filter_id && m.channel == filter_ch)
                    || matches!(msg, LogObject::CanFdMessage64(m) if m.id == filter_id && m.channel as u16 == filter_ch)
                    || matches!(msg, LogObject::LinMessage(m) if m.id as u32 == filter_id && m.channel == filter_ch))
            })
            .cloned()
            .collect(),
    }
}

// ========== Log View Rendering ==========

/// Render log view header with column labels and filter controls
pub fn render_log_header(
    app: &CanViewApp,
    view: Entity<CanViewApp>,
    time_width: gpui::Pixels,
    ch_width: gpui::Pixels,
    type_width: gpui::Pixels,
    id_width: gpui::Pixels,
    dlc_width: gpui::Pixels,
) -> Div {
    let id_filter = app.id_filter;
    let id_display_decimal = app.id_display_decimal;
    let view_clone = view.clone();  // Clone for later use in dropdown

    // Extract unique channels for dropdown
    let mut unique_channels: std::collections::HashSet<u16> = std::collections::HashSet::new();
    for msg in app.messages.iter() {
        match msg {
            blf::LogObject::CanMessage(m) => { unique_channels.insert(m.channel); }
            blf::LogObject::CanMessage2(m) => { unique_channels.insert(m.channel); }
            blf::LogObject::CanFdMessage(m) => { unique_channels.insert(m.channel); }
            blf::LogObject::CanFdMessage64(m) => { unique_channels.insert(m.channel as u16); }
            _ => {}
        }
    }
    let mut channel_list: Vec<u16> = unique_channels.into_iter().collect();
    channel_list.sort();

    let ch_filter_left = 80.0 + f32::from(time_width) - 8.0;

    div()
        .relative()  // Make it relative for absolute dropdown positioning
        .w_full()
        .flex()
        .flex_col()
        .child(
            // Header row
            div()
                .w_full()
                .h(px(28.))
                .bg(rgb(0x1f1f1f))
                .border_b_1()
                .border_color(rgb(0x2a2a2a))
                .flex()
                .items_center()
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgb(0x9ca3af))
                .child(
                    div()
                        .w(px(80.))
                .px_3()
                .py_1()
                .flex()
                .items_center()
                .flex_shrink_0()
                .whitespace_nowrap()
                .overflow_hidden()
                .child("#")
        )
        .child(
            div()
                .w(time_width)
                .px_3()
                .py_1()
                .flex()
                .items_center()
                .flex_shrink_0()
                .whitespace_nowrap()
                .overflow_hidden()
                .child("TIME")
        )
        .child(render_header_channel_column(app, view.clone(), ch_width))
        .child(
            div()
                .w(type_width)
                .px_2()
                .py_1()
                .flex()
                .items_center()
                .flex_shrink_0()
                .whitespace_nowrap()
                .overflow_hidden()
                .child("TYPE")
        )
        .child(render_header_id_column(
            app,
            view,
            id_width,
            id_filter,
            id_display_decimal,
        ))
        .child(
            div()
                .w(dlc_width)
                .px_2()
                .py_1()
                .flex()
                .items_center()
                .flex_shrink_0()
                .whitespace_nowrap()
                .overflow_hidden()
                .child("DLC")
        )
        .child(
            div()
                .w(px(60.))
                .px_2()
                .py_1()
                .flex()
                .items_center()
                .flex_shrink_0()
                .whitespace_nowrap()
                .overflow_hidden()
                .child("LENGTH")
        )
        .child(
            div()
                .flex_1()
                .px_2()
                .py_1()
                .flex()
                .items_center()
                .whitespace_nowrap()
                .child("DATA")
        )
        .child(
            div()
                .flex_1()
                .px_2()
                .py_1()
                .flex()
                .items_center()
                .whitespace_nowrap()
                .child("SIGNALS")
        )
        )
        // Channel filter dropdown (as a sibling to header)
        .when(app.show_channel_filter_input, |parent| {
            eprintln!("Rendering channel filter dropdown in header container");
            parent.child({
                use crate::app::FilterType;
                CanViewApp::render_filter_dropdown(
                    app,
                    view_clone,
                    FilterType::ChannelFilter(channel_list),
                    ch_filter_left,
                )
            })
        })
}

/// Render channel column with filter control
pub fn render_header_channel_column(
    app: &CanViewApp,
    view: Entity<CanViewApp>,
    ch_width: gpui::Pixels,
) -> Div {
    div()
        .w(ch_width)
        .px_2()
        .py_1()
        .flex()
        .items_center()
        .flex_shrink_0()
        .child("CH")
        .child(
            div()
                .text_xs()
                .cursor_pointer()
                .text_color(if app.channel_filter.is_some() {
                    rgb(0x60a5fa)
                } else if app.show_channel_filter_input {
                    rgb(0xf59e0b)  // Amber color when dropdown is open
                } else {
                    rgb(0x4b5563)
                })
                .hover(|style| style.bg(rgb(0x374151)))
                .rounded(px(2.))
                .ml_1()
                .px_1()
                .py_0p5()
                .on_mouse_down(gpui::MouseButton::Left, {
                    let view = view.clone();
                    move |_event, _window, cx| {
                        eprintln!("=== Channel filter button clicked ===");
                        cx.stop_propagation();
                        view.update(cx, |app, cx| {
                            eprintln!("Before: show_channel_filter_input={}, channel_filter={:?}",
                                app.show_channel_filter_input, app.channel_filter);
                            if app.channel_filter.is_some() {
                                app.channel_filter = None;
                                app.channel_filter_text = "".into();
                                app.show_channel_filter_input = false;
                            } else {
                                app.show_channel_filter_input = !app.show_channel_filter_input;
                            }
                            eprintln!("After: show_channel_filter_input={}, channel_filter={:?}",
                                app.show_channel_filter_input, app.channel_filter);
                            cx.notify();
                        });
                    }
                })
                .child(if app.channel_filter.is_some() { "✓" } else if app.show_channel_filter_input { "▼" } else { "⚙" })
        )
}

/// Render ID column with display mode toggle and filter control
pub fn render_header_id_column(
    app: &CanViewApp,
    view: Entity<CanViewApp>,
    id_width: gpui::Pixels,
    id_filter: Option<u32>,
    id_display_decimal: bool,
) -> Div {
    div()
        .w(id_width)
        .pl_2()
        .pr_0()
        .py_1()
        .flex()
        .items_center()
        .flex_shrink_0()
        .child(
            div()
                .flex()
                .items_center()
                .child(
                    div()
                        .cursor_pointer()
                        .rounded(px(2.))
                        .pl_1()
                        .pr_0()
                        .py_0p5()
                        .hover(|style| style.bg(rgb(0x374151)))
                        .on_mouse_down(gpui::MouseButton::Left, {
                            let view = view.clone();
                            move |_, _, cx| {
                                view.update(cx, |app, cx| {
                                    app.id_display_decimal = !app.id_display_decimal;
                                    cx.notify();
                                });
                            }
                        })
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_0p5()
                                .child("ID")
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(0x6b7280))
                                        .child(if id_display_decimal { "10" } else { "16" }),
                                ),
                        )
                )
                .child(
                    div()
                        .text_xs()
                        .cursor_pointer()
                        .text_color(if id_filter.is_some() {
                            rgb(0x60a5fa)
                        } else {
                            rgb(0x4b5563)
                        })
                        .hover(|style| style.bg(rgb(0x374151)))
                        .rounded(px(2.))
                        .pl_1()
                        .pr_0()
                        .py_0p5()
                        .on_mouse_down(gpui::MouseButton::Left, {
                            let view = view.clone();
                            move |event, _, cx| {
                                eprintln!("Gear clicked! Position: {:?}", event.position);
                                view.update(cx, |app, cx| {
                                    if app.id_filter.is_some() {
                                        eprintln!("Clearing filter");
                                        app.id_filter = None;
                                        app.id_filter_text = "".into();
                                        app.show_id_filter_input = false;
                                    } else {
                                        eprintln!(
                                            "Before: show_id_filter_input={}",
                                            app.show_id_filter_input
                                        );
                                        app.show_id_filter_input = !app.show_id_filter_input;
                                        eprintln!(
                                            "After: show_id_filter_input={}",
                                            app.show_id_filter_input
                                        );
                                    }
                                    cx.notify();
                                });
                            }
                        })
                        .child(if id_filter.is_some() { "✓" } else { "⚙" }),
                )
        )
}

/// Render message list with placeholder or uniform_list
pub fn render_message_list(
    app: &CanViewApp,
    view: Entity<CanViewApp>,
    filtered_messages: Vec<LogObject>,
    time_width: gpui::Pixels,
    ch_width: gpui::Pixels,
    type_width: gpui::Pixels,
    id_width: gpui::Pixels,
    dlc_width: gpui::Pixels,
    dbc_channels: HashMap<u16, DbcDatabase>,
    ldf_channels: HashMap<u16, LdfDatabase>,
    start_time: Option<chrono::NaiveDateTime>,
    id_display_decimal: bool,
    scroll_handle: gpui::UniformListScrollHandle,
) -> Div {
    let filtered_count = filtered_messages.len();
    let container_height = app.list_container_height;

    div()
        .flex_1()
        .flex()
        .flex_col()
        .relative()
        .h(px(container_height))  // Use explicit height to ensure uniform_list fills the space
        // Show placeholder when no messages
        .when(app.messages.is_empty(), |parent| {
            parent.child(
                div()
                    .flex_1()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        div()
                            .text_lg()
                            .text_color(rgb(0x6b7280))
                            .child("No messages loaded. Click '📂 Open BLF' to load a file."),
                    ),
            )
        })
        // Show messages - always use uniform_list for better performance
        .when(!filtered_messages.is_empty(), |parent| {
            let display_count = filtered_messages.len();
            let view_entity = view.clone();
            let filtered_msgs_clone = filtered_messages.clone();
            let dbc_channels_rc = Rc::new(dbc_channels);
            let ldf_channels_rc = Rc::new(ldf_channels);
            // Convert NaiveDateTime to DateTime<Utc>
            let start_time_utc = start_time.map(|dt| chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(dt, chrono::Utc));
            // Capture column widths for use in closure
            let time_width_cp = time_width;
            let ch_width_cp = ch_width;
            let type_width_cp = type_width;
            let id_width_cp = id_width;
            let dlc_width_cp = dlc_width;

            // Uniform list with explicit height
            let container_height = app.list_container_height;
            parent.child(
                gpui::uniform_list(
                    "message-list",
                    display_count,
                    move |range: std::ops::Range<usize>, _window: &mut gpui::Window, cx: &mut gpui::App| {
                        // Debug: log the range being rendered (less frequent for performance)
                        if range.start > 0 && range.start % 5000 == 0 {
                            eprintln!("Rendering range: {}..{} (total: {})", range.start, range.end, display_count);
                        }

                        // Track scroll position by observing the visible range
                        // Only update if significantly changed to improve performance
                        let first_visible = range.start;
                        let new_offset = px(first_visible as f32 * 22.0);
                        view_entity.update(cx, |v, cx| {
                            // Only update if offset changed significantly (> 1 pixel)
                            let current_offset = f32::from(v.scroll_offset);
                            let target_offset = f32::from(new_offset);
                            if (current_offset - target_offset).abs() > 1.0 {
                                v.scroll_offset = new_offset;
                                cx.notify(); // Only notify if actually changed
                            }
                        });

                        range
                            .map(|index| {
                                if let Some(msg) = filtered_msgs_clone.get(index) {
                                    render_message_row_static_with_widths(
                                        msg,
                                        index,
                                        dbc_channels_rc.clone(),
                                        ldf_channels_rc.clone(),
                                        start_time_utc,
                                        id_display_decimal,
                                        time_width_cp,
                                        ch_width_cp,
                                        type_width_cp,
                                        id_width_cp,
                                        dlc_width_cp,
                                    ).into_any_element()
                                } else {
                                    div().into_any_element()
                                }
                            })
                            .collect::<Vec<_>>()
                    },
                )
                .track_scroll(&scroll_handle)
                .flex_1()
                .h(px(container_height))  // Explicit height for proper virtual scrolling
            )
        })
}

/// Render overlay and filter dropdowns
pub fn render_overlays_and_dropdowns(
    app: &CanViewApp,
    view: Entity<CanViewApp>,
    time_width: gpui::Pixels,
    ch_width: gpui::Pixels,
    type_width: gpui::Pixels,
    id_width: gpui::Pixels,
) -> Div {
    use blf::LogObject;

    // Extract unique IDs and channels
    let mut unique_ids = std::collections::HashSet::new();
    let mut unique_channels = std::collections::HashSet::new();

    eprintln!("=== Extracting unique IDs and channels from {} messages ===", app.messages.len());

    for msg in app.messages.iter() {
        match msg {
            LogObject::CanMessage(m) => {
                unique_ids.insert(m.id);
                unique_channels.insert(m.channel);
            }
            LogObject::CanMessage2(m) => {
                unique_ids.insert(m.id);
                unique_channels.insert(m.channel);
            }
            LogObject::CanFdMessage(m) => {
                unique_ids.insert(m.id);
                unique_channels.insert(m.channel);
            }
            LogObject::CanFdMessage64(m) => {
                unique_ids.insert(m.id);
                unique_channels.insert(m.channel as u16);
            }
            LogObject::LinMessage(m) => {
                unique_ids.insert(m.id as u32);
                unique_channels.insert(m.channel);
            }
            _ => {}
        }
    }

    let mut id_list: Vec<u32> = unique_ids.into_iter().collect();
    id_list.sort();

    let mut channel_list: Vec<u16> = unique_channels.into_iter().collect();
    channel_list.sort();

    eprintln!("Found {} unique IDs and {} unique channels", id_list.len(), channel_list.len());

    // Calculate filter dropdown position (aligned with respective column)
    let id_filter_left = 80.0 + f32::from(time_width) + f32::from(ch_width) + f32::from(type_width) - 8.0;
    let ch_filter_left = 80.0 + f32::from(time_width) - 8.0;

    div()
        // Full-screen overlay to catch clicks outside dropdown
        .when(app.show_id_filter_input || app.show_channel_filter_input, |parent| {
            let view_for_overlay = view.clone();
            eprintln!("Rendering overlay - show_id={}, show_channel={}",
                app.show_id_filter_input, app.show_channel_filter_input);
            parent.child(
                div()
                    .absolute()
                    .inset_0()
                    .on_mouse_down(gpui::MouseButton::Left, move |_event, _window, cx| {
                        eprintln!("Overlay clicked");
                        view_for_overlay.update(cx, |app, cx| {
                            app.show_id_filter_input = false;
                            app.show_channel_filter_input = false;
                            cx.notify();
                        });
                    }),
            )
        })
        // ID filter dropdown (still rendered in main container for now)
        .when(app.show_id_filter_input, |parent| {
            eprintln!("Rendering ID filter dropdown with {} items", id_list.len());
            parent.child(render_id_filter_dropdown(
                app,
                view.clone(),
                id_list,
                id_filter_left,
            ))
        })
        // Note: Channel filter dropdown is now rendered in header container
}

/// Render ID filter dropdown
fn render_id_filter_dropdown(
    app: &CanViewApp,
    view: Entity<CanViewApp>,
    id_list: Vec<u32>,
    filter_left: f32,
) -> Div {
    eprintln!("render_id_filter_dropdown: id_list has {} items", id_list.len());

    // Use the old proven implementation from CanViewApp
    use crate::app::FilterType;
    CanViewApp::render_filter_dropdown(
        app,
        view,
        FilterType::IdFilter(id_list),
        filter_left,
    )
}

/// Render channel filter dropdown
fn render_channel_filter_dropdown(
    app: &CanViewApp,
    view: Entity<CanViewApp>,
    channel_list: Vec<u16>,
    filter_left: f32,
) -> Div {
    eprintln!("render_channel_filter_dropdown: channel_list has {} items", channel_list.len());

    // Use the old proven implementation from CanViewApp
    use crate::app::FilterType;
    CanViewApp::render_filter_dropdown(
        app,
        view,
        FilterType::ChannelFilter(channel_list),
        filter_left,
    )
}

// ========== Config View Rendering ==========

/// Render a single channel mapping card
pub fn render_channel_mapping_card(mapping: &ChannelMapping) -> Div {
    let channel_type_label = if mapping.channel_type == crate::models::ChannelType::CAN {
        "CAN"
    } else {
        "LIN"
    };

    div()
        .p_3()
        .bg(rgb(0x374151))
        .rounded(px(4.))
        .flex()
        .items_center()
        .justify_between()
        .child(
            div()
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(rgb(0xffffff))
                        .child(format!("Channel {} ({})", mapping.channel_id, channel_type_label)),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(0x9ca3af))
                        .child(mapping.path.clone()),
                ),
        )
}

/// Render system status section
pub fn render_system_status_section(
    messages_count: usize,
    dbc_count: usize,
    ldf_count: usize,
) -> Div {
    div()
        .p_4()
        .bg(rgb(0x1f1f1f))
        .border_1()
        .border_color(rgb(0x2a2a2a))
        .rounded(px(8.))
        .flex()
        .flex_col()
        .gap_2()
        .child(
            div()
                .text_sm()
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgb(0xffffff))
                .child("System Status"),
        )
        .child(
            div()
                .flex()
                .gap_4()
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(0x9ca3af))
                        .child(format!("Messages: {}", messages_count)),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(0x9ca3af))
                        .child(format!("DBC: {}", dbc_count)),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(0x9ca3af))
                        .child(format!("LIN: {}", ldf_count)),
                ),
        )
}

// ========== UI Interaction Functions ==========

/// Show library dialog
pub fn show_library_dialog(app: &mut CanViewApp, dialog_type: LibraryDialogType, cx: &mut Context<CanViewApp>) {
    app.library_dialog_type = dialog_type;
    app.show_library_dialog = true;
    cx.notify();
}

/// Hide library dialog
pub fn hide_library_dialog(app: &mut CanViewApp, cx: &mut Context<CanViewApp>) {
    app.show_library_dialog = false;
    app.new_library_name.clear();
    app.new_version_name.clear();
    cx.notify();
}

/// Quick import database
pub fn quick_import_database(app: &mut CanViewApp, cx: &mut Context<CanViewApp>) {
    // TODO: Implement file dialog for quick import
    app.status_msg = "Quick import - file dialog not yet implemented".into();
    eprintln!("⚠️  Quick import: file dialog not yet implemented");
    cx.notify();
}

/// Show channel input for adding a new channel (inline)
pub fn show_add_channel_dialog(app: &mut CanViewApp, cx: &mut Context<CanViewApp>) {
    app.show_add_channel_input = true;
    cx.notify();
}

/// Hide channel input and clear values
pub fn hide_add_channel_input(app: &mut CanViewApp, cx: &mut Context<CanViewApp>) {
    app.show_add_channel_input = false;
    app.new_channel_id.clear();
    app.new_channel_name.clear();
    app.new_channel_db_path.clear();
    app.editing_channel_index = None;
    cx.notify();
}

/// Save channel configuration
pub fn save_channel_config(app: &mut CanViewApp, cx: &mut Context<CanViewApp>) {
    // Validate channel ID
    let channel_id = match app.new_channel_id.trim().parse::<u16>() {
        Ok(id) => id,
        Err(_) => {
            app.status_msg = "Invalid channel ID".into();
            cx.notify();
            return;
        }
    };

    // Check if editing or adding
    if let Some(index) = app.editing_channel_index {
        // Update existing channel
        if let Some(mapping) = app.app_config.mappings.get_mut(index) {
            mapping.channel_id = channel_id;
            mapping.channel_type = app.new_channel_type;
            mapping.description = app.new_channel_name.clone();
            mapping.path = app.new_channel_db_path.clone();
        }
        app.status_msg = format!("Channel {} updated", channel_id).into();
    } else {
        // Add new channel
        // Check for duplicate
        if app
            .app_config
            .mappings
            .iter()
            .any(|m| m.channel_id == channel_id)
        {
            app.status_msg = format!("Channel {} already exists", channel_id).into();
            cx.notify();
            return;
        }

        app.app_config.mappings.push(crate::models::ChannelMapping {
            channel_id,
            channel_type: app.new_channel_type,
            library_id: None,
            version_name: None,
            path: app.new_channel_db_path.clone(),
            description: app.new_channel_name.clone(),
        });
        app.status_msg = format!("Channel {} added", channel_id).into();
    }

    // Clear input and hide dialog
    hide_add_channel_input(app, cx);

    // Save configuration
    // Note: need to call save_config which requires &mut self, but we have &mut self
    // For now, just notify
    cx.notify();
}

/// Delete a channel
pub fn delete_channel(app: &mut CanViewApp, channel_id: u16, cx: &mut Context<CanViewApp>) {
    // Find and remove the channel
    if let Some(pos) = app
        .app_config
        .mappings
        .iter()
        .position(|m| m.channel_id == channel_id)
    {
        app.app_config.mappings.remove(pos);
        app.status_msg = format!("Channel {} deleted", channel_id).into();
    } else {
        app.status_msg = format!("Channel {} not found", channel_id).into();
    }
    cx.notify();
}

/// Cancel channel configuration
pub fn cancel_channel_config(app: &mut CanViewApp, cx: &mut Context<CanViewApp>) {
    hide_add_channel_input(app, cx);
    app.status_msg = "Channel configuration cancelled".into();
    cx.notify();
}
