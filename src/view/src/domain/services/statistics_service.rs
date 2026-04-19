//! Statistics service
//!
//! Provides statistical analysis of log messages.
//! This service calculates various statistics about message collections.

use blf::LogObject;

/// Log statistics
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LogStatistics {
    /// Total number of messages
    pub total_messages: usize,
    /// Number of CAN messages
    pub can_messages: usize,
    /// Number of CAN FD messages
    pub canfd_messages: usize,
    /// Number of LIN messages
    pub lin_messages: usize,
    /// Number of error frames
    pub error_frames: usize,
    /// Number of unique CAN IDs
    pub unique_can_ids: usize,
    /// Number of unique LIN IDs
    pub unique_lin_ids: usize,
    /// Number of channels with data
    pub active_channels: usize,
    /// Time range of the log
    pub time_range: Option<TimeRange>,
}

/// Time range for log data
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TimeRange {
    pub start_seconds: f64,
    pub end_seconds: f64,
}

impl TimeRange {
    pub fn new(start: f64, end: f64) -> Self {
        Self {
            start_seconds: start,
            end_seconds: end,
        }
    }

    pub fn duration(&self) -> f64 {
        self.end_seconds - self.start_seconds
    }
}

/// Statistics service for analyzing log messages
///
/// This service provides methods to calculate various statistics
/// about message collections.
pub struct StatisticsService;

impl StatisticsService {
    /// Create a new statistics service
    pub fn new() -> Self {
        Self
    }

    /// Calculate statistics for a collection of messages
    pub fn calculate_statistics(&self, messages: &[LogObject]) -> LogStatistics {
        let mut stats = LogStatistics::default();
        let mut can_ids = std::collections::HashSet::new();
        let mut lin_ids = std::collections::HashSet::new();
        let mut channels = std::collections::HashSet::new();
        let mut min_time = None;
        let mut max_time = None;

        for msg in messages {
            stats.total_messages += 1;

            match msg {
                LogObject::CanMessage(m) => {
                    stats.can_messages += 1;
                    can_ids.insert(m.id as u32);
                    channels.insert(m.channel);
                }
                LogObject::CanMessage2(m) => {
                    stats.can_messages += 1;
                    can_ids.insert(m.id as u32);
                    channels.insert(m.channel);
                }
                LogObject::CanFdMessage(m) => {
                    stats.canfd_messages += 1;
                    can_ids.insert(m.id as u32);
                    channels.insert(m.channel);
                }
                LogObject::CanFdMessage64(m) => {
                    stats.canfd_messages += 1;
                    can_ids.insert(m.id as u32);
                    channels.insert(m.channel as u16);
                }
                LogObject::LinMessage(m) => {
                    stats.lin_messages += 1;
                    lin_ids.insert(m.id as u32);
                    channels.insert(m.channel);
                }
                LogObject::LinMessage2(_) => {
                    stats.lin_messages += 1;
                }
                LogObject::CanErrorFrame(_) => {
                    stats.error_frames += 1;
                }
                _ => {}
            }

            // Track time range
            if let Some(timestamp) = Self::extract_timestamp(msg) {
                let current_min = min_time.unwrap_or(timestamp);
                let current_max = max_time.unwrap_or(timestamp);
                min_time = Some(current_min.min(timestamp));
                max_time = Some(current_max.max(timestamp));
            }
        }

        stats.unique_can_ids = can_ids.len();
        stats.unique_lin_ids = lin_ids.len();
        stats.active_channels = channels.len();

        if let (Some(min), Some(max)) = (min_time, max_time) {
            stats.time_range = Some(TimeRange::new(min, max));
        }

        stats
    }

    /// Calculate message frequency by ID
    ///
    /// Returns a map of message IDs to their occurrence count
    pub fn calculate_id_frequency(&self, messages: &[LogObject]) -> std::collections::HashMap<u32, usize> {
        let mut frequency = std::collections::HashMap::new();

        for msg in messages {
            let id = match msg {
                LogObject::CanMessage(m) => Some(m.id as u32),
                LogObject::CanMessage2(m) => Some(m.id as u32),
                LogObject::CanFdMessage(m) => Some(m.id as u32),
                LogObject::CanFdMessage64(m) => Some(m.id as u32),
                LogObject::LinMessage(m) => Some(m.id as u32),
                _ => None,
            };

            if let Some(id) = id {
                *frequency.entry(id).or_insert(0) += 1;
            }
        }

        frequency
    }

    /// Calculate message frequency by channel
    ///
    /// Returns a map of channel numbers to their message count
    pub fn calculate_channel_frequency(
        &self,
        messages: &[LogObject],
    ) -> std::collections::HashMap<u16, usize> {
        let mut frequency = std::collections::HashMap::new();

        for msg in messages {
            let channel = msg.channel();
            if let Some(ch) = channel { *frequency.entry(ch).or_insert(0) += 1; }
        }

        frequency
    }

    /// Calculate message rate (messages per second)
    ///
    /// Returns the average message rate over the entire time range
    pub fn calculate_message_rate(&self, messages: &[LogObject]) -> Option<f64> {
        if messages.is_empty() {
            return None;
        }

        let mut min_time = None;
        let mut max_time = None;

        for msg in messages {
            if let Some(timestamp) = Self::extract_timestamp(msg) {
                let current_min = min_time.unwrap_or(timestamp);
                let current_max = max_time.unwrap_or(timestamp);
                min_time = Some(current_min.min(timestamp));
                max_time = Some(current_max.max(timestamp));
            }
        }

        if let (Some(min), Some(max)) = (min_time, max_time) {
            let duration = max - min;
            if duration > 0.0 {
                Some(messages.len() as f64 / duration)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Get message type distribution
    ///
    /// Returns a breakdown of messages by type
    pub fn get_message_type_distribution(
        &self,
        messages: &[LogObject],
    ) -> MessageTypeDistribution {
        let mut distribution = MessageTypeDistribution::default();

        for msg in messages {
            match msg {
                LogObject::CanMessage(_) | LogObject::CanMessage2(_) => {
                    distribution.can_messages += 1;
                }
                LogObject::CanFdMessage(_) | LogObject::CanFdMessage64(_) => {
                    distribution.canfd_messages += 1;
                }
                LogObject::LinMessage(_) | LogObject::LinMessage2(_) => {
                    distribution.lin_messages += 1;
                }
                LogObject::CanErrorFrame(_) => {
                    distribution.error_frames += 1;
                }
                _ => {
                    distribution.other_messages += 1;
                }
            }
        }

        distribution
    }

    /// Extract timestamp from a log object
    fn extract_timestamp(msg: &LogObject) -> Option<f64> {
        match msg {
            LogObject::CanMessage(_)
            | LogObject::CanMessage2(_)
            | LogObject::CanFdMessage(_)
            | LogObject::CanFdMessage64(_)
            | LogObject::LinMessage(_)
            | LogObject::LinMessage2(_) => Some(msg.timestamp() as f64 / 10_000_000.0),
            _ => None,
        }
    }
}

impl Default for StatisticsService {
    fn default() -> Self {
        Self::new()
    }
}

/// Message type distribution statistics
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MessageTypeDistribution {
    pub can_messages: usize,
    pub canfd_messages: usize,
    pub lin_messages: usize,
    pub error_frames: usize,
    pub other_messages: usize,
}

impl MessageTypeDistribution {
    /// Get total message count
    pub fn total(&self) -> usize {
        self.can_messages
            + self.canfd_messages
            + self.lin_messages
            + self.error_frames
            + self.other_messages
    }

    /// Get CAN percentage
    pub fn can_percentage(&self) -> f64 {
        if self.total() == 0 {
            return 0.0;
        }
        (self.can_messages + self.canfd_messages) as f64 / self.total() as f64 * 100.0
    }

    /// Get LIN percentage
    pub fn lin_percentage(&self) -> f64 {
        if self.total() == 0 {
            return 0.0;
        }
        self.lin_messages as f64 / self.total() as f64 * 100.0
    }

    /// Get error percentage
    pub fn error_percentage(&self) -> f64 {
        if self.total() == 0 {
            return 0.0;
        }
        self.error_frames as f64 / self.total() as f64 * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use blf::CanMessage;
    use blf::TxDirection;

    fn create_test_can_message(channel: u16, id: u32, timestamp: u64) -> LogObject {
        LogObject::CanMessage(CanMessage {
            channel,
            id: id as u32,
            dlc: 8,
            data: vec![0; 8],
            timestamp,
            dir: TxDirection::Rx,
            ..Default::default()
        })
    }

    #[test]
    fn test_calculate_statistics() {
        let service = StatisticsService::new();
        let messages = vec![
            create_test_can_message(1, 0x123, 0),
            create_test_can_message(1, 0x456, 1_000_000_000),
            create_test_can_message(2, 0x123, 2_000_000_000),
        ];

        let stats = service.calculate_statistics(&messages);

        assert_eq!(stats.total_messages, 3);
        assert_eq!(stats.can_messages, 3);
        assert_eq!(stats.unique_can_ids, 2);
        assert_eq!(stats.active_channels, 2);
        assert!(stats.time_range.is_some());
    }

    #[test]
    fn test_calculate_id_frequency() {
        let service = StatisticsService::new();
        let messages = vec![
            create_test_can_message(1, 0x123, 0),
            create_test_can_message(1, 0x123, 1000),
            create_test_can_message(1, 0x456, 2000),
        ];

        let frequency = service.calculate_id_frequency(&messages);

        assert_eq!(frequency.get(&0x123), Some(&2));
        assert_eq!(frequency.get(&0x456), Some(&1));
    }

    #[test]
    fn test_calculate_channel_frequency() {
        let service = StatisticsService::new();
        let messages = vec![
            create_test_can_message(1, 0x123, 0),
            create_test_can_message(1, 0x456, 1000),
            create_test_can_message(2, 0x789, 2000),
        ];

        let frequency = service.calculate_channel_frequency(&messages);

        assert_eq!(frequency.get(&1), Some(&2));
        assert_eq!(frequency.get(&2), Some(&1));
    }

    #[test]
    fn test_calculate_message_rate() {
        let service = StatisticsService::new();
        let messages = vec![
            create_test_can_message(1, 0x123, 0),
            create_test_can_message(1, 0x456, 500_000_000), // 0.5 seconds
            create_test_can_message(1, 0x789, 1_000_000_000), // 1.0 seconds
        ];

        let rate = service.calculate_message_rate(&messages);

        assert!(rate.is_some());
        // 3 messages in 1 second = 3 msg/sec
        assert!((rate.unwrap() - 3.0).abs() < 0.1);
    }

    #[test]
    fn test_message_type_distribution() {
        let service = StatisticsService::new();
        let messages = vec![
            create_test_can_message(1, 0x123, 0),
            create_test_can_message(1, 0x456, 1000),
            LogObject::CanErrorFrame(blf::CanErrorFrame {
                channel: 1,
                timestamp: 2000,
                ..Default::default()
            }),
        ];

        let distribution = service.get_message_type_distribution(&messages);

        assert_eq!(distribution.can_messages, 2);
        assert_eq!(distribution.error_frames, 1);
        assert_eq!(distribution.total(), 3);
        assert!((distribution.can_percentage() - 66.67).abs() < 0.1);
    }

    #[test]
    fn test_time_range() {
        let service = StatisticsService::new();
        let messages = vec![
            create_test_can_message(1, 0x123, 0),
            create_test_can_message(1, 0x456, 1_000_000_000),
        ];

        let stats = service.calculate_statistics(&messages);

        assert!(stats.time_range.is_some());
        let range = stats.time_range.unwrap();
        assert_eq!(range.start_seconds, 0.0);
        assert_eq!(range.end_seconds, 100.0); // 1_000_000_000 / 10_000_000
        assert_eq!(range.duration(), 100.0);
    }

    #[test]
    fn test_empty_messages() {
        let service = StatisticsService::new();
        let messages: Vec<LogObject> = vec![];

        let stats = service.calculate_statistics(&messages);
        assert_eq!(stats.total_messages, 0);

        let rate = service.calculate_message_rate(&messages);
        assert!(rate.is_none());
    }
}
