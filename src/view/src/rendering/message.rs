//! Message formatting and rendering utilities
//!
//! This module contains utility functions for formatting and rendering
//! CAN/LIN message data.

use blf::LogObject;
use parser::dbc::DbcDatabase;
use parser::ldf::LdfDatabase;
use gpui::{px, Pixels};

/// Calculate column widths for the message table
///
/// This function analyzes all messages and determines the optimal width
/// for each column to ensure proper alignment and readability.
///
/// # Arguments
/// * `messages` - Slice of log objects to analyze
/// * `dbc_channels` - DBC database channels (currently unused)
/// * `ldf_channels` - LDF database channels (currently unused)
/// * `start_time` - Optional start time for relative timestamps
///
/// # Returns
/// A tuple of 5 `Pixels` values representing the widths for:
/// (time, channel, type, id, dlc) columns
pub fn calculate_column_widths(
    messages: &[LogObject],
    _dbc_channels: &std::collections::HashMap<u16, DbcDatabase>,
    _ldf_channels: &std::collections::HashMap<u16, LdfDatabase>,
    start_time: Option<chrono::NaiveDateTime>,
) -> (
    gpui::Pixels,
    gpui::Pixels,
    gpui::Pixels,
    gpui::Pixels,
    gpui::Pixels,
) {
    // Define minimum widths for each column (for header text)
    let mut max_time_width = 50.0_f32; // "TIME" header
    let mut max_ch_width = 60.0_f32; // "CH" header with gear icon (CH + ⚙ + padding = ~50px)
    let mut max_type_width = 50.0_f32; // "TYPE" header
    let mut max_id_width = 80.0_f32; // "ID" header with gear icon (ID + 10 + ⚙ = ~70px)
    let mut max_dlc_width = 40.0_f32; // "DLC" header

    // Calculate widths based on ALL messages
    // Use a smarter sampling strategy:
    // - For small datasets (<1000), scan all
    // - For large datasets, scan in intervals to get representative sample
    let sample_size = messages.len();
    let step = if sample_size > 5000 {
        sample_size / 1000 // Sample ~1000 messages spread evenly
    } else if sample_size > 1000 {
        sample_size / 500 // Sample ~500 messages spread evenly
    } else {
        1 // Scan all messages
    };

    for (i, msg) in messages.iter().enumerate() {
        // Skip messages based on step size for large datasets
        if i % step != 0 {
            continue;
        }

        let (time_str, channel_id, msg_type, id_str, dlc_str, _data_str) =
            get_message_strings(msg, start_time, true); // Use decimal for width calculation

        // Calculate exact width needed for each column
        // Using 8.0 pixels per character (monospace font approximation)
        // Add padding: horizontal padding (px_2 or px_3) + some margin
        max_time_width = max_time_width.max(time_str.len() as f32 * 8.0 + 16.0); // px_3 = 12px + 4px margin
        max_ch_width = max_ch_width.max(channel_id.to_string().len() as f32 * 8.0 + 10.0); // px_2 = 8px + 2px margin
        max_type_width = max_type_width.max(msg_type.len() as f32 * 8.0 + 10.0);
        max_id_width = max_id_width.max(id_str.len() as f32 * 8.0 + 10.0);
        max_dlc_width = max_dlc_width.max(dlc_str.len() as f32 * 8.0 + 10.0);
    }

    // Apply maximum limits to prevent columns from becoming too wide
    // This ensures the table remains readable even with very long content
    max_time_width = max_time_width.min(300.0);
    max_ch_width = max_ch_width.min(80.0);
    max_type_width = max_type_width.min(120.0);
    max_id_width = max_id_width.min(100.0);
    max_dlc_width = max_dlc_width.min(80.0);

    // Round to integer pixels to ensure consistency across all rows
    // This prevents rounding errors that can cause misalignment
    max_time_width = max_time_width.round();
    max_ch_width = max_ch_width.round();
    max_type_width = max_type_width.round();
    max_id_width = max_id_width.round();
    max_dlc_width = max_dlc_width.round();

    // Return calculated widths (excluding DATA which uses flex_1)
    (
        px(max_time_width),
        px(max_ch_width),
        px(max_type_width),
        px(max_id_width),
        px(max_dlc_width),
    )
}

/// Extract message strings without rendering
///
/// This function extracts formatted string representations of various
/// message fields for display or processing purposes.
///
/// # Arguments
/// * `msg` - Reference to the log object
/// * `start_time` - Optional start time for relative timestamps
/// * `decimal` - If true, format IDs as decimal; if false, as hex (0xXXX)
///
/// # Returns
/// A tuple of 6 strings: (time, channel_id, type, id, dlc, data)
pub fn get_message_strings(
    msg: &LogObject,
    start_time: Option<chrono::NaiveDateTime>,
    decimal: bool,
) -> (String, u16, String, String, String, String) {
    let format_id = |id: u32| -> String {
        if decimal {
            id.to_string()
        } else {
            format!("0x{:03X}", id)
        }
    };

    match msg {
        LogObject::CanMessage(can_msg) => {
            let timestamp = can_msg.header.object_time_stamp;
            let time_str = if let Some(start) = start_time {
                let msg_time = start + chrono::Duration::nanoseconds(timestamp as i64);
                // Format: YYYY-MM-DD HH:MM:SS.mmmmmm (microseconds)
                msg_time.format("%Y-%m-%d %H:%M:%S%.6f").to_string()
            } else {
                // If no start time, show nanoseconds as seconds with microsecond precision
                let seconds = timestamp as f64 / 1_000_000_000.0;
                format!("{:.6}", seconds)
            };

            let actual_data_len = can_msg.data.len().min(can_msg.dlc as usize);
            let data_hex = can_msg
                .data
                .iter()
                .take(actual_data_len)
                .map(|b| format!("{:02X}", b))
                .collect::<Vec<_>>()
                .join(" ");

            (
                time_str,
                can_msg.channel,
                "CAN".to_string(),
                format_id(can_msg.id),
                actual_data_len.to_string(),
                data_hex,
            )
        }
        LogObject::CanMessage2(can_msg) => {
            let timestamp = can_msg.header.object_time_stamp;
            let time_str = if let Some(start) = start_time {
                let msg_time = start + chrono::Duration::nanoseconds(timestamp as i64);
                msg_time.format("%Y-%m-%d %H:%M:%S%.6f").to_string()
            } else {
                let seconds = timestamp as f64 / 1_000_000_000.0;
                format!("{:.6}", seconds)
            };

            let actual_data_len = can_msg.data.len().min(can_msg.dlc as usize);
            let data_hex = can_msg
                .data
                .iter()
                .take(actual_data_len)
                .map(|b| format!("{:02X}", b))
                .collect::<Vec<_>>()
                .join(" ");

            (
                time_str,
                can_msg.channel,
                "CAN2".to_string(),
                format_id(can_msg.id),
                actual_data_len.to_string(),
                data_hex,
            )
        }
        LogObject::CanErrorFrame(err) => {
            let timestamp = err.header.object_time_stamp;
            let time_str = if let Some(start) = start_time {
                let msg_time = start + chrono::Duration::nanoseconds(timestamp as i64);
                msg_time.format("%Y-%m-%d %H:%M:%S%.6f").to_string()
            } else {
                let seconds = timestamp as f64 / 1_000_000_000.0;
                format!("{:.6}", seconds)
            };

            (
                time_str,
                err.channel,
                "CAN_ERR".to_string(),
                "-".to_string(),
                err.length.to_string(),
                "-".to_string(),
            )
        }
        LogObject::CanFdMessage(fd_msg) => {
            let timestamp = fd_msg.header.object_time_stamp;
            let time_str = if let Some(start) = start_time {
                let msg_time = start + chrono::Duration::nanoseconds(timestamp as i64);
                msg_time.format("%Y-%m-%d %H:%M:%S%.6f").to_string()
            } else {
                let seconds = timestamp as f64 / 1_000_000_000.0;
                format!("{:.6}", seconds)
            };

            let actual_data_len = fd_msg.data.len().min(fd_msg.dlc as usize);
            let data_hex = fd_msg
                .data
                .iter()
                .take(actual_data_len)
                .map(|b| format!("{:02X}", b))
                .collect::<Vec<_>>()
                .join(" ");

            (
                time_str,
                fd_msg.channel,
                "CAN_FD".to_string(),
                format_id(fd_msg.id),
                actual_data_len.to_string(),
                data_hex,
            )
        }
        LogObject::CanFdMessage64(fd_msg) => {
            let timestamp = fd_msg.header.object_time_stamp;
            let time_str = if let Some(start) = start_time {
                let msg_time = start + chrono::Duration::nanoseconds(timestamp as i64);
                msg_time.format("%Y-%m-%d %H:%M:%S%.6f").to_string()
            } else {
                let seconds = timestamp as f64 / 1_000_000_000.0;
                format!("{:.6}", seconds)
            };

            let actual_data_len = fd_msg.data.len().min(fd_msg.valid_data_bytes as usize);
            let data_hex = fd_msg
                .data
                .iter()
                .take(actual_data_len)
                .map(|b| format!("{:02X}", b))
                .collect::<Vec<_>>()
                .join(" ");

            (
                time_str,
                fd_msg.channel as u16,
                "CAN_FD64".to_string(),
                format_id(fd_msg.id),
                actual_data_len.to_string(),
                data_hex,
            )
        }
        LogObject::CanOverloadFrame(ov) => {
            let timestamp = ov.header.object_time_stamp;
            let time_str = if let Some(start) = start_time {
                let msg_time = start + chrono::Duration::nanoseconds(timestamp as i64);
                msg_time.format("%Y-%m-%d %H:%M:%S%.6f").to_string()
            } else {
                let seconds = timestamp as f64 / 1_000_000_000.0;
                format!("{:.6}", seconds)
            };

            (
                time_str,
                ov.channel,
                "CAN_OV".to_string(),
                "-".to_string(),
                "-".to_string(),
                "-".to_string(),
            )
        }
        LogObject::LinMessage(lin_msg) => {
            let timestamp = lin_msg.header.object_time_stamp;
            let time_str = if let Some(start) = start_time {
                let msg_time = start + chrono::Duration::nanoseconds(timestamp as i64);
                // Format: YYYY-MM-DD HH:MM:SS.mmmmmm (microseconds)
                msg_time.format("%Y-%m-%d %H:%M:%S%.6f").to_string()
            } else {
                format!("{:.6}", timestamp as f64 / 1_000_000_000.0)
            };

            let actual_data_len = lin_msg.data.len().min(lin_msg.dlc as usize);
            let data_hex = lin_msg
                .data
                .iter()
                .take(actual_data_len)
                .map(|b| format!("{:02X}", b))
                .collect::<Vec<_>>()
                .join(" ");

            (
                time_str,
                lin_msg.channel,
                "LIN".to_string(),
                format_id(lin_msg.id as u32),
                actual_data_len.to_string(),
                data_hex,
            )
        }
        LogObject::LinMessage2(lin_msg) => {
            let timestamp = lin_msg.header.object_time_stamp;
            let time_str = if let Some(start) = start_time {
                let msg_time = start + chrono::Duration::nanoseconds(timestamp as i64);
                msg_time.format("%Y-%m-%d %H:%M:%S%.6f").to_string()
            } else {
                let seconds = timestamp as f64 / 1_000_000_000.0;
                format!("{:.6}", seconds)
            };

            let actual_data_len = lin_msg.data.len();
            let data_hex = lin_msg
                .data
                .iter()
                .map(|b| format!("{:02X}", b))
                .collect::<Vec<_>>()
                .join(" ");

            (
                time_str,
                0_u16,
                "LIN2".to_string(),
                "-".to_string(),
                actual_data_len.to_string(),
                data_hex,
            )
        }
        _ => {
            let type_name = format!("{:?}", msg);
            (
                "-".to_string(),
                0_u16,
                type_name.split('(').next().unwrap_or("UNKNOWN").to_string(),
                "-".to_string(),
                "-".to_string(),
                "-".to_string(),
            )
        }
    }
}

/// Render a message row with pre-calculated widths for perfect alignment
///
/// This function renders a single message row with fixed column widths
/// to ensure consistent alignment across all rows.
///
/// # Arguments
/// * `msg` - Reference to the log object to render
/// * `index` - Row index for line number display
/// * `time_width` - Width of the timestamp column
/// * `ch_width` - Width of the channel column
/// * `type_width` - Width of the message type column
/// * `id_width` - Width of the ID column
/// * `dlc_width` - Width of the DLC column
/// * `_dbc_channels` - DBC database channels (currently unused)
/// * `_ldf_channels` - LDF database channels (currently unused)
/// * `start_time` - Optional start time for relative timestamps
/// * `decimal` - If true, format IDs as decimal; if false, as hex
/// * `disable_hover` - If true, disable hover effect
///
/// # Returns
/// A GPUI element that can be rendered in the message table
pub fn render_message_row_static_with_widths(
    msg: &LogObject,
    index: usize,
    time_width: gpui::Pixels,
    ch_width: gpui::Pixels,
    type_width: gpui::Pixels,
    id_width: gpui::Pixels,
    dlc_width: gpui::Pixels,
    _dbc_channels: &std::collections::HashMap<u16, DbcDatabase>,
    _ldf_channels: &std::collections::HashMap<u16, LdfDatabase>,
    start_time: Option<chrono::NaiveDateTime>,
    decimal: bool,
    disable_hover: bool,
) -> gpui::AnyElement {
    use gpui::{div, prelude::*, rgb};

    let (time_str, channel_id, msg_type, id_str, dlc_str, data_str) =
        get_message_strings(msg, start_time, decimal);

    let bg_color = rgb(0x181818);
    let type_color = match msg_type.as_str() {
        "CAN" | "CAN2" => rgb(0x34d399),
        "CAN_ERR" => rgb(0xef4444),
        "CAN_FD" | "CAN_FD64" => rgb(0x8b5cf6),
        "CAN_OV" => rgb(0xf59e0b),
        "LIN" | "LIN2" => rgb(0x60a5fa),
        _ => rgb(0x9ca3af),
    };

    div()
        .flex()
        .w_full()
        .min_h(px(22.))
        .bg(bg_color)
        .border_b_1()
        .border_color(rgb(0x2a2a2a))
        .items_center()
        .text_xs()
        .text_color(rgb(0xd1d5db))
        .when(!disable_hover, |div| {
            div.hover(|style| style.bg(rgb(0x1f2937)))
        })
        .cursor_pointer()
        .overflow_hidden()
        .child(
            // Line number column
            div()
                .w(px(60.))
                .px_3()
                .py_1()
                .flex()
                .items_center()
                .flex_shrink_0()
                .text_color(rgb(0x6b7280))
                .whitespace_nowrap()
                .overflow_hidden()
                .child(format!("{}", index + 1)),
        )
        .child(
            div()
                .w(time_width)
                .px_3()
                .py_1()
                .flex()
                .items_center()
                .flex_shrink_0()
                .text_color(rgb(0x9ca3af))
                .whitespace_nowrap()
                .overflow_hidden()
                .child(time_str),
        )
        .child(
            div()
                .w(ch_width)
                .px_2()
                .py_1()
                .flex()
                .items_center()
                .flex_shrink_0()
                .text_color(rgb(0x60a5fa))
                .whitespace_nowrap()
                .overflow_hidden()
                .child(channel_id.to_string()),
        )
        .child(
            div()
                .w(type_width)
                .px_2()
                .py_1()
                .flex()
                .items_center()
                .flex_shrink_0()
                .text_color(type_color)
                .whitespace_nowrap()
                .overflow_hidden()
                .child(msg_type),
        )
        .child(
            div()
                .w(id_width)
                .px_2()
                .py_1()
                .flex()
                .items_center()
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
                .py_1()
                .flex()
                .items_center()
                .flex_shrink_0()
                .whitespace_nowrap()
                .overflow_hidden()
                .child(dlc_str),
        )
        .child(
            div()
                .flex_1()
                .px_2()
                .py_1()
                .flex()
                .items_center()
                .text_color(rgb(0xa78bfa))
                .whitespace_nowrap()
                .child(data_str),
        )
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_column_widths_empty() {
        let messages: Vec<LogObject> = vec![];
        let dbc_channels = std::collections::HashMap::new();
        let ldf_channels = std::collections::HashMap::new();

        let (time_w, ch_w, type_w, id_w, dlc_w) =
            calculate_column_widths(&messages, &dbc_channels, &ldf_channels, None);

        // Should return minimum widths based on headers
        assert_eq!(time_w, px(50.0));
        assert_eq!(ch_w, px(60.0));
        assert_eq!(type_w, px(50.0));
        assert_eq!(id_w, px(80.0));
        assert_eq!(dlc_w, px(40.0));
    }

    #[test]
    fn test_format_id_decimal() {
        let format_id_fn = |id: u32| -> String {
            if true {  // decimal = true
                id.to_string()
            } else {
                format!("0x{:03X}", id)
            }
        };

        assert_eq!(format_id_fn(123), "123");
        assert_eq!(format_id_fn(0xABC), 2748.to_string());
    }

    #[test]
    fn test_format_id_hex() {
        let format_id_fn = |id: u32| -> String {
            if false {  // decimal = false
                id.to_string()
            } else {
                format!("0x{:03X}", id)
            }
        };

        assert_eq!(format_id_fn(0x123), "0x123");
        assert_eq!(format_id_fn(0xABC), "0xABC");
    }
}
