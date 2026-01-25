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
pub fn format_timestamp(timestamp: u64, start_time: Option<chrono::NaiveDateTime>) -> String {
    if let Some(start) = start_time {
        let msg_time = start + chrono::Duration::nanoseconds(timestamp as i64);
        // Format: YYYY-MM-DD HH:MM:SS.mmmmmm (microseconds)
        msg_time.format("%Y-%m-%d %H:%M:%S%.6f").to_string()
    } else {
        // If no start time, show nanoseconds as seconds with microsecond precision
        format!("{:.6}", timestamp as f64 / 1_000_000_000.0)
    }
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
