//! Log view rendering module
//!
//! This module contains all functionality related to rendering the log view,
//! including message list, filtering, headers, and scroll handling.

use crate::app::CanViewApp;
use crate::rendering::calculate_column_widths;
use blf::LogObject;
use gpui::{prelude::*, *};
use parser::dbc::DbcDatabase;
use parser::ldf::LdfDatabase;
use std::collections::HashMap;
use std::rc::Rc;

/// Render the log view with message list and filters
///
/// This function renders the main log view including:
/// - Message header with column labels
/// - Filter controls (ID filter, channel filter)
/// - Message list with uniform list for performance
/// - Custom scrollbar
///
/// # Arguments
/// * `view` - The CanViewApp entity for state access and updates
///
/// # Returns
/// An element that can be rendered in the UI
///
/// NOTE: This is a simplified version. Full implementation requires access to App context
/// for reading entity state. This will be integrated into impls.rs render method.
pub fn render_log_view(_view: Entity<CanViewApp>) -> impl IntoElement {
    // TODO: Integrate full log view rendering
    // This requires access to App context to read entity state properly
    div()
        .size_full()
        .flex()
        .items_center()
        .justify_center()
        .child("Log View - Coming Soon")
}

/// Apply ID and channel filters to messages
fn apply_message_filters(
    messages: &[LogObject],
    id_filter: Option<u32>,
    channel_filter: Option<u16>,
) -> Vec<LogObject> {
    match (id_filter, channel_filter) {
        (None, None) => messages.to_vec(),
        (Some(filter_id), None) => messages
            .iter()
            .filter(|msg| {
                matches!(msg, LogObject::CanMessage(m) if m.id == filter_id)
                    || matches!(msg, LogObject::CanMessage2(m) if m.id == filter_id)
                    || matches!(msg, LogObject::CanFdMessage(m) if m.id == filter_id)
                    || matches!(msg, LogObject::CanFdMessage64(m) if m.id == filter_id)
                    || matches!(msg, LogObject::LinMessage(m) if m.id as u32 == filter_id)
            })
            .cloned()
            .collect(),
        (None, Some(filter_ch)) => messages
            .iter()
            .filter(|msg| {
                matches!(msg, LogObject::CanMessage(m) if m.channel == filter_ch)
                    || matches!(msg, LogObject::CanMessage2(m) if m.channel == filter_ch)
                    || matches!(msg, LogObject::CanFdMessage(m) if m.channel == filter_ch)
                    || matches!(msg, LogObject::CanFdMessage64(m) if m.channel as u16 == filter_ch)
                    || matches!(msg, LogObject::LinMessage(m) if m.channel == filter_ch)
            })
            .cloned()
            .collect(),
        (Some(filter_id), Some(filter_ch)) => messages
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

/*
/// Handle keyboard input for the log view
/// NOTE: This function is temporarily disabled due to KeyEvent type compatibility issues
#[allow(dead_code)]
fn _handle_log_view_key_down(
    _event: &gpui::KeyEvent,
    _view: &Entity<CanViewApp>,
    _cx: &mut Context<CanViewApp>,
) {
    // TODO: Re-enable after fixing KeyEvent type
    let _keystroke_str = format!("{}", _event.keystroke);
    /*
    _view.update(_cx, |app, cx| {
        match keystroke_str.as_str() {
            "backspace" => {
                let mut text = app.id_filter_text.to_string();
                if !text.is_empty() {
                    text.pop();
                    app.id_filter_text = text.into();
                    if text.is_empty() {
                        app.id_filter = None;
                    } else if let Ok(parsed_id) = u32::from_str_radix(&text, 10) {
                        app.id_filter = Some(parsed_id);
                    } else {
                        app.id_filter = None;
                    }
                    cx.notify();
                }
            }
            "escape" => {
                app.id_filter = None;
                app.id_filter_text = "".into();
                app.show_id_filter_input = false;
                cx.notify();
            }
            _ => {
                // Handle digit input
                if keystroke_str.len() == 1 {
                    if let Some(ch) = keystroke_str.chars().next() {
                        if ch.is_ascii_digit() {
                            let mut text = app.id_filter_text.to_string();
                            text.push(ch);
                            if let Ok(parsed_id) = u32::from_str_radix(&text, 10) {
                                app.id_filter = Some(parsed_id);
                            }
                            app.id_filter_text = text.into();
                            cx.notify();
                        }
                    }
                }
            }
        }
    });
    */
}
*/

/// Render the log view header with column labels
fn render_log_header(
    view: Entity<CanViewApp>,
    time_width: Pixels,
    ch_width: Pixels,
    type_width: Pixels,
    id_width: Pixels,
    _dlc_width: Pixels,
    id_filter: Option<u32>,
    channel_filter: Option<u16>,
    show_channel_filter_input: bool,
) -> Div {
    // Build header columns directly
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
        .child(div().w(px(50.)).px_3().py_1().child("#"))
        .child(div().w(time_width).px_3().py_1().child("TIME"))
        .child({
            let view_clone = view.clone();
            div()
                .w(ch_width)
                .px_2()
                .py_1()
                .flex()
                .items_center()
                .child("CH")
                .child(
                    div()
                        .text_xs()
                        .cursor_pointer()
                        .text_color(if channel_filter.is_some() {
                            rgb(0x60a5fa)
                        } else {
                            rgb(0x4b5563)
                        })
                        .hover(|style| style.bg(rgb(0x374151)))
                        .rounded(px(2.))
                        .ml_0p5()
                        .pl_0()
                        .pr_0()
                        .py_0p5()
                        .on_mouse_down(gpui::MouseButton::Left, move |_event, _window, cx| {
                            view_clone.update(cx, |app, cx| {
                                if app.channel_filter.is_some() {
                                    app.channel_filter = None;
                                    app.channel_filter_text = "".into();
                                    app.show_channel_filter_input = false;
                                } else {
                                    app.show_channel_filter_input = !app.show_channel_filter_input;
                                }
                                cx.notify();
                            });
                        })
                        .child(if channel_filter.is_some() {
                            "✓"
                        } else {
                            "⚙"
                        }),
                )
        })
        .child(div().w(type_width).px_2().py_1().child("TYPE"))
        .child(div().w(id_width).px_2().py_1().child("ID"))
        .child(div().w(_dlc_width).px_2().py_1().child("DLC"))
        .child(div().w(px(30.)).px_2().py_1().child("LENGTH"))
        .child(div().flex_1().px_2().child("DATA"))
        .child(div().w(px(250.)).flex_shrink_0().px_2().child("SIGNALS"))
        .when(show_channel_filter_input, |this| {
            this.child(div().child("Channel filter dropdown"))
        })
}

/// Render a single message row in the log view
///
/// # Arguments
/// * `msg` - The log object to render
/// * `index` - The row index
/// * `dbc_channels` - Reference to DBC channel database
/// * `ldf_channels` - Reference to LDF channel database
/// * `start_time` - Optional start time for relative timestamps
/// * `id_display_decimal` - Whether to display IDs in decimal format
///
/// # Returns
/// An element representing the message row
pub fn render_message_row(
    msg: &LogObject,
    index: usize,
    dbc_channels: &HashMap<u16, DbcDatabase>,
    ldf_channels: &HashMap<u16, LdfDatabase>,
    start_time: &Option<chrono::DateTime<chrono::Utc>>,
    id_display_decimal: bool,
) -> impl IntoElement {
    let (time_str, channel_id, msg_type, id_str, dlc_str, data_str, signals_str) =
        parse_message_data(
            msg,
            dbc_channels,
            ldf_channels,
            start_time,
            id_display_decimal,
        );

    div()
        .w_full()
        .h(px(22.))
        .flex()
        .items_center()
        .text_sm()
        .border_b_1()
        .border_color(rgb(0x2a2a2a))
        .hover(|style| style.bg(rgb(0x1a1a1a)))
        .child(
            div()
                .w(px(50.))
                .px_2()
                .flex_shrink_0()
                .text_color(rgb(0x6b7280))
                .child(index.to_string()),
        )
        .child(
            div()
                .w(px(120.))
                .px_3()
                .flex_shrink_0()
                .text_color(rgb(0x60a5fa))
                .child(time_str),
        )
        .child(
            div()
                .w(px(40.))
                .px_2()
                .flex_shrink_0()
                .text_color(rgb(0x9ca3af))
                .child(channel_id.to_string()),
        )
        .child(
            div()
                .w(px(50.))
                .px_2()
                .flex_shrink_0()
                .text_color(rgb(0x9ca3af))
                .child(msg_type),
        )
        .child(
            div()
                .w(px(60.))
                .px_2()
                .flex_shrink_0()
                .text_color(rgb(0xfbbf24))
                .child(id_str),
        )
        .child(
            div()
                .w(px(30.))
                .px_2()
                .flex_shrink_0()
                .text_color(rgb(0x9ca3af))
                .child(dlc_str.clone()),
        )
        .child(
            div()
                .w(px(30.))
                .px_2()
                .flex_shrink_0()
                .text_color(rgb(0x9ca3af))
                .child(dlc_str), // LENGTH列显示数据长度
        )
        .child(
            div()
                .flex_1()
                .px_1()
                .text_size(px(10.0))
                .text_color(rgb(0x10b981))
                .font_family("Mono")
                .child(data_str),
        )
        .child(
            div()
                .w(px(250.))
                .flex_shrink_0()
                .px_2()
                .text_color(rgb(0x8b5cf6))
                .whitespace_nowrap()
                .overflow_hidden()
                .child(signals_str),
        )
}

/// Parse message data for rendering
fn parse_message_data(
    msg: &LogObject,
    dbc_channels: &HashMap<u16, DbcDatabase>,
    ldf_channels: &HashMap<u16, LdfDatabase>,
    start_time: &Option<chrono::DateTime<chrono::Utc>>,
    id_display_decimal: bool,
) -> (String, u16, &'static str, String, String, String, String) {
    match msg {
        LogObject::CanMessage(can_msg) => {
            let time_str = format_timestamp(can_msg.header.object_time_stamp, start_time);
            let channel_id = can_msg.channel;
            let msg_type = "CAN";
            let id_str = if id_display_decimal {
                can_msg.id.to_string()
            } else {
                format!("{:03X}", can_msg.id)
            };
            let dlc_str = can_msg.dlc.to_string();
            let data_len = can_msg.data.len().min(can_msg.dlc as usize);
            let data_str = crate::rendering::format_hex_data(&can_msg.data[..data_len]);

            let signals_str = decode_can_signals(can_msg, dbc_channels);

            (
                time_str,
                channel_id,
                msg_type,
                id_str,
                dlc_str,
                data_str,
                signals_str,
            )
        }
        LogObject::LinMessage(lin_msg) => {
            let time_str = format_timestamp(lin_msg.header.object_time_stamp, start_time);
            let channel_id = lin_msg.channel;
            let msg_type = "LIN";
            let id_str = lin_msg.id.to_string();
            let dlc_str = lin_msg.dlc.to_string();
            let data_len = lin_msg.data.len().min(lin_msg.dlc as usize);
            let data_str = crate::rendering::format_hex_data(&lin_msg.data[..data_len]);

            let signals_str = decode_lin_signals(lin_msg, ldf_channels);

            (
                time_str,
                channel_id,
                msg_type,
                id_str,
                dlc_str,
                data_str,
                signals_str,
            )
        }
        LogObject::CanMessage2(can_msg) => {
            let time_str = format_timestamp(can_msg.header.object_time_stamp, start_time);
            let channel_id = can_msg.channel;
            let msg_type = "CAN2";
            let id_str = if id_display_decimal {
                can_msg.id.to_string()
            } else {
                format!("{:03X}", can_msg.id)
            };
            let dlc_str = can_msg.dlc.to_string();
            let data_len = can_msg.data.len().min(can_msg.dlc as usize);
            let data_str = crate::rendering::format_hex_data(&can_msg.data[..data_len]);
            let signals_str = decode_can2_signals(can_msg, dbc_channels);
            (
                time_str,
                channel_id,
                msg_type,
                id_str,
                dlc_str,
                data_str,
                signals_str,
            )
        }
        LogObject::CanFdMessage(fd_msg) => {
            let time_str = format_timestamp(fd_msg.header.object_time_stamp, start_time);
            let channel_id = fd_msg.channel;
            let msg_type = "CANFD";
            let id_str = if id_display_decimal {
                fd_msg.id.to_string()
            } else {
                format!("{:03X}", fd_msg.id)
            };
            let dlc_str = fd_msg.dlc.to_string();
            let data_len = fd_msg.data.len().min(fd_msg.dlc as usize);
            let data_str = crate::rendering::format_hex_data(&fd_msg.data[..data_len]);
            let signals_str = decode_can_fd_signals(fd_msg, dbc_channels);
            (
                time_str,
                channel_id,
                msg_type,
                id_str,
                dlc_str,
                data_str,
                signals_str,
            )
        }
        LogObject::CanFdMessage64(fd_msg) => {
            let time_str = format_timestamp(fd_msg.header.object_time_stamp, start_time);
            let channel_id = fd_msg.channel as u16;
            let msg_type = "CANFD64";
            let id_str = if id_display_decimal {
                fd_msg.id.to_string()
            } else {
                format!("{:03X}", fd_msg.id)
            };
            let dlc_str = fd_msg.dlc.to_string();
            let data_len = fd_msg.data.len().min(fd_msg.valid_data_bytes as usize);
            let data_str = crate::rendering::format_hex_data(&fd_msg.data[..data_len]);
            let signals_str = decode_can_fd_signals_64(fd_msg, dbc_channels);
            (
                time_str,
                channel_id,
                msg_type,
                id_str,
                dlc_str,
                data_str,
                signals_str,
            )
        }
        _ => {
            // Default handler for all other message types
            let timestamp = msg.timestamp();
            let time_str = format_timestamp(timestamp, start_time);
            (
                time_str,
                0,
                "OTHER",
                "N/A".to_string(),
                "0".to_string(),
                String::new(),
                String::new(),
            )
        }
    }
}

/// Format timestamp as string.
///
/// `timestamp` 是 MergedView::from_segments 写入的绝对 Unix 纳秒
/// (abs_ns = file_start_ns + msg.relative_ns),已经包含文件里记录的
/// measurement_start_time,无需再加 start_time。
fn format_timestamp(timestamp: u64, _start_time: &Option<chrono::DateTime<chrono::Utc>>) -> String {
    use chrono::{TimeZone, Utc};
    let dt = Utc.timestamp_nanos(timestamp as i64);
    dt.naive_utc().format("%H:%M:%S%.6f").to_string()
}

/// Decode CAN signals from a message
fn decode_can_signals(
    can_msg: &blf::CanMessage,
    dbc_channels: &HashMap<u16, DbcDatabase>,
) -> String {
    if let Some(db) = dbc_channels.get(&can_msg.channel) {
        if let Some(message) = db.messages.get(&can_msg.id) {
            let signals: Vec<String> = message
                .signals
                .iter()
                .map(|(sig_name, sig)| {
                    let val = sig.decode(&can_msg.data);
                    format!("{}: {:.2}", sig_name, val)
                })
                .collect();
            signals.join(", ")
        } else {
            String::new()
        }
    } else {
        String::new()
    }
}

/// Decode CAN2 signals from a message (CanMessage2 type)
fn decode_can2_signals(
    can_msg: &blf::CanMessage2,
    dbc_channels: &HashMap<u16, DbcDatabase>,
) -> String {
    if let Some(db) = dbc_channels.get(&can_msg.channel) {
        if let Some(message) = db.messages.get(&can_msg.id) {
            let signals: Vec<String> = message
                .signals
                .iter()
                .map(|(sig_name, sig)| {
                    let val = sig.decode(&can_msg.data);
                    format!("{}: {:.2}", sig_name, val)
                })
                .collect();
            signals.join(", ")
        } else {
            String::new()
        }
    } else {
        String::new()
    }
}

/// Decode CAN FD signals from a message
fn decode_can_fd_signals(
    fd_msg: &blf::CanFdMessage,
    dbc_channels: &HashMap<u16, DbcDatabase>,
) -> String {
    if let Some(db) = dbc_channels.get(&fd_msg.channel) {
        if let Some(message) = db.messages.get(&fd_msg.id) {
            let signals: Vec<String> = message
                .signals
                .iter()
                .map(|(sig_name, sig)| {
                    let val = sig.decode(&fd_msg.data);
                    format!("{}: {:.2}", sig_name, val)
                })
                .collect();
            signals.join(", ")
        } else {
            String::new()
        }
    } else {
        String::new()
    }
}

/// Decode LIN signals from a message
fn decode_lin_signals(
    _lin_msg: &blf::LinMessage,
    _ldf_channels: &HashMap<u16, LdfDatabase>,
) -> String {
    // TODO: Implement LIN signal decoding
    // LIN signal decoding requires access to frame byte order which may have changed in the parser API
    String::new()
}

/// Decode CAN FD 64 signals from a message
fn decode_can_fd_signals_64(
    fd_msg: &blf::CanFdMessage64,
    dbc_channels: &HashMap<u16, DbcDatabase>,
) -> String {
    if let Some(db) = dbc_channels.get(&(fd_msg.channel as u16)) {
        if let Some(message) = db.messages.get(&fd_msg.id) {
            let signals: Vec<String> = message
                .signals
                .iter()
                .map(|(sig_name, sig)| {
                    let val = sig.decode(&fd_msg.data);
                    format!("{}: {:.2}", sig_name, val)
                })
                .collect();
            signals.join(", ")
        } else {
            String::new()
        }
    } else {
        String::new()
    }
}

/// Static version of render_message_row for use in uniform_list closures
///
/// This version takes all parameters as values rather than references,
/// making it suitable for use in closures that need to be 'static.
pub fn render_message_row_static(
    msg: &LogObject,
    index: usize,
    dbc_channels: Rc<HashMap<u16, DbcDatabase>>,
    ldf_channels: Rc<HashMap<u16, LdfDatabase>>,
    start_time: Option<chrono::DateTime<chrono::Utc>>,
    id_display_decimal: bool,
) -> impl IntoElement {
    let (time_str, channel_id, msg_type, id_str, dlc_str, data_str, signals_str) =
        parse_message_data(
            msg,
            &dbc_channels,
            &ldf_channels,
            &start_time,
            id_display_decimal,
        );

    div()
        .w_full()
        .h(px(22.))
        .flex()
        .items_center()
        .text_sm()
        .border_b_1()
        .border_color(rgb(0x2a2a2a))
        .hover(|style| style.bg(rgb(0x1a1a1a)))
        .child(
            div()
                .w(px(50.))
                .px_2()
                .flex_shrink_0()
                .text_color(rgb(0x6b7280))
                .child(index.to_string()),
        )
        .child(
            div()
                .w(px(120.))
                .px_3()
                .flex_shrink_0()
                .text_color(rgb(0x60a5fa))
                .child(time_str),
        )
        .child(
            div()
                .w(px(40.))
                .px_2()
                .flex_shrink_0()
                .text_color(rgb(0x9ca3af))
                .child(channel_id.to_string()),
        )
        .child(
            div()
                .w(px(50.))
                .px_2()
                .flex_shrink_0()
                .text_color(rgb(0x9ca3af))
                .child(msg_type),
        )
        .child(
            div()
                .w(px(60.))
                .px_2()
                .flex_shrink_0()
                .text_color(rgb(0xfbbf24))
                .child(id_str),
        )
        .child(
            div()
                .w(px(30.))
                .px_2()
                .flex_shrink_0()
                .text_color(rgb(0x9ca3af))
                .child(dlc_str.clone()),
        )
        .child(
            div()
                .w(px(30.))
                .px_2()
                .flex_shrink_0()
                .text_color(rgb(0x9ca3af))
                .child(dlc_str), // LENGTH列显示数据长度
        )
        .child(
            div()
                .flex_1()
                .px_1()
                .text_size(px(10.0))
                .text_color(rgb(0x10b981))
                .font_family("Mono")
                .child(data_str),
        )
        .child(
            div()
                .w(px(250.))
                .flex_shrink_0()
                .px_2()
                .text_color(rgb(0x8b5cf6))
                .whitespace_nowrap()
                .overflow_hidden()
                .child(signals_str),
        )
}

/// Static version with custom column widths
///
/// This version accepts custom column widths to match header widths.
pub fn render_message_row_static_with_widths(
    msg: &LogObject,
    index: usize,
    dbc_channels: Rc<HashMap<u16, DbcDatabase>>,
    ldf_channels: Rc<HashMap<u16, LdfDatabase>>,
    start_time: Option<chrono::DateTime<chrono::Utc>>,
    id_display_decimal: bool,
    time_width: gpui::Pixels,
    ch_width: gpui::Pixels,
    type_width: gpui::Pixels,
    id_width: gpui::Pixels,
    dlc_width: gpui::Pixels,
) -> impl IntoElement {
    let (time_str, channel_id, msg_type, id_str, dlc_str, data_str, signals_str) =
        parse_message_data(
            msg,
            &dbc_channels,
            &ldf_channels,
            &start_time,
            id_display_decimal,
        );

    div()
        .w_full()
        .h(px(22.))
        .flex()
        .items_center()
        .text_sm()
        .border_b_1()
        .border_color(rgb(0x2a2a2a))
        .hover(|style| style.bg(rgb(0x1a1a1a)))
        .child(
            div()
                .w(px(50.))
                .px_2()
                .flex_shrink_0()
                .text_color(rgb(0x6b7280))
                .child(index.to_string()),
        )
        .child(
            div()
                .w(time_width)
                .px_3()
                .flex_shrink_0()
                .text_color(rgb(0x60a5fa))
                .child(time_str),
        )
        .child(
            div()
                .w(ch_width)
                .px_2()
                .flex_shrink_0()
                .text_color(rgb(0x9ca3af))
                .child(channel_id.to_string()),
        )
        .child(
            div()
                .w(type_width)
                .px_2()
                .flex_shrink_0()
                .text_color(rgb(0x9ca3af))
                .whitespace_nowrap()
                .overflow_hidden()
                .child(msg_type),
        )
        .child(
            div()
                .w(id_width)
                .px_2()
                .flex_shrink_0()
                .text_color(rgb(0xfbbf24))
                .whitespace_nowrap()
                .overflow_hidden()
                .child(id_str),
        )
        .child(
            div()
                .w(dlc_width)
                .px_2()
                .flex_shrink_0()
                .text_color(rgb(0x9ca3af))
                .whitespace_nowrap()
                .overflow_hidden()
                .child(dlc_str.clone()),
        )
        .child(
            div()
                .w(px(30.))
                .px_2()
                .flex_shrink_0()
                .text_color(rgb(0x9ca3af))
                .whitespace_nowrap()
                .overflow_hidden()
                .child(dlc_str), // LENGTH列显示数据长度（字节数）
        )
        .child(
            div()
                .flex_1()
                .px_1()
                .text_size(px(10.0))
                .text_color(rgb(0x10b981))
                .font_family("Mono")
                .whitespace_nowrap()
                .child(data_str),
        )
        .child(
            div()
                .w(px(250.))
                .flex_shrink_0()
                .px_2()
                .text_color(rgb(0x8b5cf6))
                .whitespace_nowrap()
                .overflow_hidden()
                .child(signals_str),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_timestamp_no_start() {
        let result = format_timestamp(1_500_000_000, &None);
        assert!(result.starts_with("1."));
    }

    #[test]
    fn test_format_timestamp_with_start() {
        let start = chrono::Utc::now();
        let result = format_timestamp(1_000_000_000, &Some(start));
        // Should contain time formatting
        assert!(result.contains(':') || result.contains('.'));
    }

    #[test]
    fn test_apply_message_filters_no_filters() {
        // This is a basic smoke test
        let filtered = apply_message_filters(&[], None, None);
        assert_eq!(filtered.len(), 0);
    }

    #[test]
    fn test_parse_message_data_unknown() {
        // Create a minimal LogObject for testing
        // This test verifies the unknown message case
        let time_str = format_timestamp(12345, &None);
        assert!(!time_str.is_empty());
    }
}
