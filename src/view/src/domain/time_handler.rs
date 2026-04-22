//! Time handling utilities for CAN/LIN bus data
//!
//! This module provides time-related functionality without UI dependencies.

use chrono::{NaiveDateTime, TimeZone, Utc};
use std::fmt;

/// Represents a timestamp in the BLF file
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlfTimestamp {
    /// Nanoseconds since some epoch (typically file start or system boot)
    pub nanoseconds: u64,
}

impl BlfTimestamp {
    /// Create a new BLF timestamp from nanoseconds
    pub fn from_nanos(nanos: u64) -> Self {
        Self { nanoseconds: nanos }
    }

    /// Create a new BLF timestamp from microseconds
    pub fn from_micros(micros: u64) -> Self {
        Self {
            nanoseconds: micros * 1_000,
        }
    }

    /// Convert to seconds as f64
    pub fn to_seconds(self) -> f64 {
        self.nanoseconds as f64 / 1_000_000_000.0
    }

    /// Get the nanoseconds part (0-999999999)
    pub fn nanos_part(self) -> u32 {
        (self.nanoseconds % 1_000_000_000) as u32
    }

    /// Get the whole seconds part
    pub fn seconds_part(self) -> u64 {
        self.nanoseconds / 1_000_000_000
    }
}

/// Time range for filtering or displaying data
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TimeRange {
    pub start: f64,
    pub end: f64,
}

impl TimeRange {
    /// Create a new time range
    pub fn new(start: f64, end: f64) -> Self {
        Self { start, end }
    }

    /// Check if a timestamp is within this range
    pub fn contains(&self, timestamp: f64) -> bool {
        timestamp >= self.start && timestamp <= self.end
    }

    /// Get the duration of this range
    pub fn duration(&self) -> f64 {
        self.end - self.start
    }
}

/// Handles time formatting and conversion
pub struct TimeHandler {
    /// Reference timestamp (typically start of measurement)
    start_time: Option<NaiveDateTime>,
}

impl TimeHandler {
    /// Create a new time handler
    pub fn new() -> Self {
        Self { start_time: None }
    }

    /// Create a new time handler with a reference time
    pub fn with_start_time(start_time: NaiveDateTime) -> Self {
        Self {
            start_time: Some(start_time),
        }
    }

    /// Set the reference start time
    pub fn set_start_time(&mut self, start_time: NaiveDateTime) {
        self.start_time = Some(start_time);
    }

    /// Get the reference start time
    pub fn start_time(&self) -> Option<NaiveDateTime> {
        self.start_time
    }

    /// Convert nanoseconds to seconds
    pub fn nanos_to_seconds(nanos: u64) -> f64 {
        nanos as f64 / 1_000_000_000.0
    }

    /// Convert timestamp to seconds relative to start time
    pub fn timestamp_to_seconds(&self, nanos: u64) -> f64 {
        Self::nanos_to_seconds(nanos)
    }

    /// Calculate absolute time from timestamp
    pub fn calculate_absolute_time(&self, nanos: u64) -> Option<NaiveDateTime> {
        let start = self.start_time?;
        let duration = chrono::Duration::nanoseconds(nanos as i64);
        start.checked_add_signed(duration)
    }

    /// Format timestamp for display
    pub fn format_timestamp(&self, nanos: u64) -> FormattedTimestamp {
        let seconds = self.timestamp_to_seconds(nanos);
        let absolute_time = self.calculate_absolute_time(nanos);

        FormattedTimestamp {
            relative_seconds: seconds,
            absolute_time,
        }
    }
}

impl Default for TimeHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Formatted timestamp ready for display
#[derive(Clone, Debug)]
pub struct FormattedTimestamp {
    /// Relative time in seconds since start
    pub relative_seconds: f64,
    /// Absolute wall-clock time (if available)
    pub absolute_time: Option<NaiveDateTime>,
}

impl fmt::Display for FormattedTimestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.6}s", self.relative_seconds)?;

        if let Some(abs_time) = self.absolute_time {
            write!(f, " ({})", abs_time.format("%Y-%m-%d %H:%M:%S%.3f"))?;
        }

        Ok(())
    }
}

/// Timestamp formatter with various display formats
pub struct TimestampFormatter {
    /// Format to use for display
    format: TimestampFormat,
}

/// Available timestamp formats
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TimestampFormat {
    /// Seconds only: "123.456789s"
    Seconds,
    /// Seconds with 6 decimals: "123.456789s"
    SecondsMicros,
    /// HH:MM:SS.mmm format
    HMS,
    /// Absolute time: "2024-01-15 14:30:45.123"
    Absolute,
}

impl Default for TimestampFormatter {
    fn default() -> Self {
        Self {
            format: TimestampFormat::SecondsMicros,
        }
    }
}

impl TimestampFormatter {
    /// Create a new formatter with specified format
    pub fn new(format: TimestampFormat) -> Self {
        Self { format }
    }

    /// Format a timestamp value
    pub fn format(&self, seconds: f64, absolute_time: Option<NaiveDateTime>) -> String {
        match self.format {
            TimestampFormat::Seconds => format!("{:.2}s", seconds),
            TimestampFormat::SecondsMicros => format!("{:.6}s", seconds),
            TimestampFormat::HMS => {
                let total_secs = seconds as i64;
                let hours = total_secs / 3600;
                let minutes = (total_secs % 3600) / 60;
                let secs = total_secs % 60;
                let millis = ((seconds % 1.0) * 1000.0) as u32;
                format!("{:02}:{:02}:{:02}.{:03}", hours, minutes, secs, millis)
            }
            TimestampFormat::Absolute => {
                if let Some(abs) = absolute_time {
                    abs.format("%Y-%m-%d %H:%M:%S%.3f").to_string()
                } else {
                    format!("{:.6}s", seconds)
                }
            }
        }
    }

    /// Format timestamp from nanoseconds
    pub fn format_nanos(&self, nanos: u64, start_time: Option<NaiveDateTime>) -> String {
        let seconds = TimeHandler::nanos_to_seconds(nanos);
        let absolute_time = start_time.and_then(|start| {
            let duration = chrono::Duration::nanoseconds(nanos as i64);
            start.checked_add_signed(duration)
        });

        self.format(seconds, absolute_time)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nanos_to_seconds() {
        assert_eq!(TimeHandler::nanos_to_seconds(1_000_000_000), 1.0);
        assert_eq!(TimeHandler::nanos_to_seconds(1_500_000_000), 1.5);
        assert_eq!(TimeHandler::nanos_to_seconds(500_000_000), 0.5);
    }

    #[test]
    fn test_time_range() {
        let range = TimeRange::new(1.0, 5.0);
        assert!(range.contains(2.0));
        assert!(range.contains(1.0));
        assert!(range.contains(5.0));
        assert!(!range.contains(0.5));
        assert!(!range.contains(5.5));
        assert_eq!(range.duration(), 4.0);
    }

    #[test]
    fn test_blf_timestamp() {
        let ts = BlfTimestamp::from_nanos(1_500_000_000);
        assert_eq!(ts.to_seconds(), 1.5);
        assert_eq!(ts.seconds_part(), 1);
        assert_eq!(ts.nanos_part(), 500_000_000);
    }

    #[test]
    fn test_timestamp_formatter() {
        let formatter = TimestampFormatter::new(TimestampFormat::Seconds);
        assert_eq!(formatter.format(1.5, None), "1.50s");

        let formatter = TimestampFormatter::new(TimestampFormat::SecondsMicros);
        assert_eq!(formatter.format(1.5, None), "1.500000s");

        let formatter = TimestampFormatter::new(TimestampFormat::HMS);
        assert_eq!(formatter.format(3661.5, None), "01:01:01.500");
    }
}
