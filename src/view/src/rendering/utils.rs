//! Rendering utility functions
//!
//! This module contains pure utility functions for formatting and data transformation
//! used in rendering UI elements.

/// Format a timestamp into a human-readable string
///
/// # Arguments
/// * `timestamp` - The timestamp in nanoseconds
/// * `start_time` - Optional start time for relative timestamps
///
/// # Returns
/// A formatted timestamp string in the format "YYYY-MM-DD HH:MM:SS.mmmmmm"
/// If no start time is provided, returns the timestamp as seconds with microsecond precision
///
/// # Examples
/// ```
/// let timestamp = 1_500_000_000; // 1.5 seconds
/// let formatted = format_timestamp(timestamp, None);
/// assert_eq!(formatted, "1.500000");
/// ```
#[allow(dead_code)]
pub fn format_timestamp(timestamp: u64, _start_time: Option<chrono::NaiveDateTime>) -> String {
    // `timestamp` 是 MergedView::from_segments 写入的绝对 Unix 纳秒
    // (abs_ns = file_start_ns + msg.relative_ns),已经包含文件里记录的
    // measurement_start_time,无需再加 start_time。
    use chrono::{TimeZone, Utc};
    let dt = Utc.timestamp_nanos(timestamp as i64);
    dt.naive_utc().format("%Y-%m-%d %H:%M:%S%.6f").to_string()
}

/// Format a byte array as a hexadecimal string with space separators
///
/// # Arguments
/// * `data` - Slice of bytes to format
///
/// # Returns
/// A string with each byte formatted as two uppercase hex digits, separated by spaces
///
/// # Examples
/// ```
/// let data = vec![0x12, 0x34, 0xAB];
/// let formatted = format_hex_data(&data);
/// assert_eq!(formatted, "12 34 AB");
/// ```
pub fn format_hex_data(data: &[u8]) -> String {
    data.iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Format a CAN ID as a hexadecimal string
///
/// # Arguments
/// * `id` - The CAN ID
///
/// # Returns
/// A string with the ID formatted as "0xXXX"
pub fn format_can_id(id: u32) -> String {
    format!("0x{:03X}", id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_timestamp_without_start() {
        let result = format_timestamp(1_500_000_000, None);
        assert_eq!(result, "1.500000");
    }

    #[test]
    fn test_format_hex_data() {
        let data = vec![0x12, 0x34, 0xAB, 0xFF];
        let result = format_hex_data(&data);
        assert_eq!(result, "12 34 AB FF");
    }

    #[test]
    fn test_format_can_id() {
        assert_eq!(format_can_id(0x123), "0x123");
        assert_eq!(format_can_id(0xABC), "0xABC");
    }
}
