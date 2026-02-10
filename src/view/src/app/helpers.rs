//! Application helper functions
//!
//! This module contains utility functions and helpers used throughout the application.
//! These are pure functions that don't depend on application state.

use chrono::{TimeZone, Utc};

/// Convert BLF timestamp to seconds
///
/// BLF timestamps are in 100ns units since epoch
pub fn convert_timestamp_to_seconds(timestamp: u64, flags: u32) -> f64 {
    // BLF timestamp is in 100ns units
    // flags might contain timezone info, but we ignore for now
    timestamp as f64 / 10_000_000.0
}

/// Format timestamp as string
///
/// Converts a timestamp (in seconds) to a human-readable string
pub fn format_timestamp_static(timestamp: f64) -> String {
    let secs = timestamp.trunc() as i64;
    let nanos = ((timestamp.fract() * 1_000_000_000.0) as u32).min(999_999_999);

    if let Some(dt) = Utc.timestamp_opt(secs, nanos).single() {
        dt.format("%Y-%m-%d %H:%M:%S%.3f").to_string()
    } else {
        format!("{:.3}", timestamp)
    }
}

/// Format timestamp as string with date
///
/// Similar to format_timestamp_static but with explicit date formatting
pub fn format_timestamp_with_date(timestamp: f64) -> String {
    let secs = timestamp.trunc() as i64;
    let nanos = ((timestamp.fract() * 1_000_000_000.0) as u32).min(999_999_999);

    if let Some(dt) = Utc.timestamp_opt(secs, nanos).single() {
        dt.format("%H:%M:%S%.3f").to_string()
    } else {
        format!("{:.3}", timestamp)
    }
}

/// Calculate time difference string
///
/// Returns a human-readable time difference between two timestamps
pub fn format_time_difference(start: f64, end: f64) -> String {
    let diff = end - start;
    if diff < 1.0 {
        format!("{:.0}ms", diff * 1000.0)
    } else if diff < 60.0 {
        format!("{:.2}s", diff)
    } else if diff < 3600.0 {
        let mins = (diff / 60.0).trunc() as u32;
        let secs = diff % 60.0;
        format!("{}m {:.1}s", mins, secs)
    } else {
        let hours = (diff / 3600.0).trunc() as u32;
        let mins = ((diff % 3600.0) / 60.0).trunc() as u32;
        format!("{}h {}m", hours, mins)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_timestamp_to_seconds() {
        // 1 second in 100ns units = 10,000,000
        let timestamp = 10_000_000u64;
        assert!((convert_timestamp_to_seconds(timestamp, 0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_format_timestamp_static() {
        let timestamp = 1.0;
        let formatted = format_timestamp_static(timestamp);
        assert!(formatted.contains(":"));
    }

    #[test]
    fn test_format_time_difference() {
        assert_eq!(format_time_difference(0.0, 0.5), "500ms");
        assert_eq!(format_time_difference(0.0, 1.5), "1.50s");
        assert_eq!(format_time_difference(0.0, 90.0), "1m 30.0s");
    }
}
