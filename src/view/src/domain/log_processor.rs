//! Log processing logic for BLF files
//!
//! This module handles BLF log data processing, filtering, and statistics
//! without UI dependencies.

use blf::LogObject;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Statistics about log data
#[derive(Clone, Debug, Default)]
pub struct LogStatistics {
    /// Total number of messages
    pub total_messages: usize,
    /// Number of CAN messages
    pub can_messages: usize,
    /// Number of CAN FD messages
    pub canfd_messages: usize,
    /// Number of LIN messages
    pub lin_messages: usize,
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

/// Message filter criteria
#[derive(Clone, Debug, Default)]
pub struct MessageFilter {
    /// Filter by CAN/LIN ID (None means show all)
    pub id_filter: Option<u32>,
    /// Filter by channel number (None means show all)
    pub channel_filter: Option<u16>,
    /// Only show CAN messages
    pub can_only: bool,
    /// Only show LIN messages
    pub lin_only: bool,
    /// Only show error frames
    pub errors_only: bool,
}

impl MessageFilter {
    /// Create a new filter with no criteria
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by specific ID
    pub fn with_id(mut self, id: u32) -> Self {
        self.id_filter = Some(id);
        self
    }

    /// Filter by specific channel
    pub fn with_channel(mut self, channel: u16) -> Self {
        self.channel_filter = Some(channel);
        self
    }

    /// Only CAN messages
    pub fn can_only(mut self) -> Self {
        self.can_only = true;
        self.lin_only = false;
        self
    }

    /// Only LIN messages
    pub fn lin_only(mut self) -> Self {
        self.lin_only = true;
        self.can_only = false;
        self
    }

    /// Check if a message matches the filter
    pub fn matches(&self, message: &LogObject) -> bool {
        // Check ID filter
        if let Some(filter_id) = self.id_filter {
            match message {
                LogObject::CanMessage(msg) if msg.id as u32 == filter_id => {}
                LogObject::CanFdMessage(msg) if msg.id as u32 == filter_id => {}
                LogObject::LinMessage(msg) if msg.id as u32 == filter_id => {}
                _ => return false,
            }
        }

        // Check channel filter
        if let Some(filter_ch) = self.channel_filter {
            let msg_channel = match message {
                LogObject::CanMessage(msg) => Some(msg.channel),
                LogObject::CanFdMessage(msg) => Some(msg.channel),
                LogObject::LinMessage(msg) => Some(msg.channel),
                _ => None,
            };

            if msg_channel != Some(filter_ch) {
                return false;
            }
        }

        // Check CAN/LIN only
        if self.can_only {
            if !matches!(
                message,
                LogObject::CanMessage(_) | LogObject::CanFdMessage(_)
            ) {
                return false;
            }
        }

        if self.lin_only {
            if !matches!(message, LogObject::LinMessage(_)) {
                return false;
            }
        }

        // Check errors only
        if self.errors_only {
            if !matches!(message, LogObject::CanErrorFrame(_)) {
                return false;
            }
        }

        true
    }
}

/// Log data processor
pub struct LogProcessor {
    /// All loaded messages
    messages: Vec<LogObject>,
    /// Filtered messages (cache)
    filtered_messages: Option<Vec<LogObject>>,
    /// Current filter
    current_filter: Option<MessageFilter>,
    /// Cached statistics
    statistics: Option<LogStatistics>,
}

impl LogProcessor {
    /// Create a new log processor
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            filtered_messages: None,
            current_filter: None,
            statistics: None,
        }
    }

    /// Create a new log processor with messages
    pub fn with_messages(messages: Vec<LogObject>) -> Self {
        let mut processor = Self::new();
        processor.set_messages(messages);
        processor
    }

    /// Set messages and invalidate caches
    pub fn set_messages(&mut self, messages: Vec<LogObject>) {
        self.messages = messages;
        self.filtered_messages = None;
        self.statistics = None;
    }

    /// Get all messages
    pub fn messages(&self) -> &[LogObject] {
        &self.messages
    }

    /// Get message count
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    /// Clear all messages
    pub fn clear(&mut self) {
        self.messages.clear();
        self.filtered_messages = None;
        self.statistics = None;
    }

    /// Add a single message
    pub fn add_message(&mut self, message: LogObject) {
        self.messages.push(message);
        self.filtered_messages = None;
        self.statistics = None;
    }

    /// Add multiple messages
    pub fn add_messages(&mut self, messages: Vec<LogObject>) {
        self.messages.extend(messages);
        self.filtered_messages = None;
        self.statistics = None;
    }

    /// Apply a filter and return filtered messages
    pub fn apply_filter(&mut self, filter: MessageFilter) -> &[LogObject] {
        // Check if filter is the same as current
        if let Some(current) = &self.current_filter {
            if Self::filters_equal(current, &filter) {
                // Same filter, return cached result
                return self.filtered_messages.as_ref().unwrap_or(&self.messages);
            }
        }

        // Apply new filter
        self.current_filter = Some(filter.clone());

        if Self::is_filter_empty(&filter) {
            // No filtering needed
            self.filtered_messages = None;
            &self.messages
        } else {
            // Apply filter
            let filtered: Vec<LogObject> = self
                .messages
                .iter()
                .filter(|msg| filter.matches(msg))
                .cloned()
                .collect();

            self.filtered_messages = Some(filtered);
            self.filtered_messages.as_deref().unwrap_or(&self.messages)
        }
    }

    /// Get currently filtered messages (if any filter is applied)
    pub fn filtered_messages(&self) -> Option<&[LogObject]> {
        self.filtered_messages.as_deref()
    }

    /// Clear filter and return all messages
    pub fn clear_filter(&mut self) -> &[LogObject] {
        self.current_filter = None;
        self.filtered_messages = None;
        &self.messages
    }

    /// Get current filter
    pub fn current_filter(&self) -> Option<&MessageFilter> {
        self.current_filter.as_ref()
    }

    /// Calculate and cache statistics
    pub fn calculate_statistics(&mut self) -> &LogStatistics {
        if self.statistics.is_some() {
            return self.statistics.as_ref().unwrap();
        }

        let mut stats = LogStatistics::default();
        let mut can_ids = HashSet::new();
        let mut lin_ids = HashSet::new();
        let mut channels = HashSet::new();
        let mut min_time = None;
        let mut max_time = None;

        for msg in &self.messages {
            stats.total_messages += 1;

            match msg {
                LogObject::CanMessage(m) => {
                    stats.can_messages += 1;
                    can_ids.insert(m.id);
                    channels.insert(m.channel);
                }
                LogObject::CanFdMessage(m) => {
                    stats.canfd_messages += 1;
                    can_ids.insert(m.id);
                    channels.insert(m.channel);
                }
                LogObject::LinMessage(m) => {
                    stats.lin_messages += 1;
                    lin_ids.insert(m.id);
                    channels.insert(m.channel);
                }
                _ => {}
            }

            // Track time range
            if let Some(timestamp) = Self::extract_timestamp(msg) {
                let current_min: f64 = min_time.unwrap_or(timestamp);
                let current_max: f64 = max_time.unwrap_or(timestamp);
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

        self.statistics = Some(stats);
        self.statistics.as_ref().unwrap()
    }

    /// Get statistics without recalculating if cached
    pub fn statistics(&self) -> Option<&LogStatistics> {
        self.statistics.as_ref()
    }

    /// Find messages by ID
    pub fn find_by_id(&self, id: u32) -> Vec<&LogObject> {
        self.messages
            .iter()
            .filter(|msg| match msg {
                LogObject::CanMessage(m) if m.id as u32 == id => true,
                LogObject::CanFdMessage(m) if m.id as u32 == id => true,
                LogObject::LinMessage(m) if m.id as u32 == id => true,
                _ => false,
            })
            .collect()
    }

    /// Find messages by channel
    pub fn find_by_channel(&self, channel: u16) -> Vec<&LogObject> {
        self.messages
            .iter()
            .filter(|msg| match msg {
                LogObject::CanMessage(m) if m.channel == channel => true,
                LogObject::CanFdMessage(m) if m.channel == channel => true,
                LogObject::LinMessage(m) if m.channel == channel => true,
                _ => false,
            })
            .collect()
    }

    /// Get all unique CAN IDs
    pub fn unique_can_ids(&self) -> Vec<u32> {
        let mut ids = HashSet::new();
        for msg in &self.messages {
            match msg {
                LogObject::CanMessage(m) => {
                    ids.insert(m.id as u32);
                }
                LogObject::CanFdMessage(m) => {
                    ids.insert(m.id as u32);
                }
                _ => {}
            }
        }
        let mut sorted: Vec<_> = ids.into_iter().collect();
        sorted.sort_unstable();
        sorted
    }

    /// Get all unique LIN IDs
    pub fn unique_lin_ids(&self) -> Vec<u32> {
        let mut ids = HashSet::new();
        for msg in &self.messages {
            if let LogObject::LinMessage(m) = msg {
                ids.insert(m.id as u32);
            }
        }
        let mut sorted: Vec<_> = ids.into_iter().collect();
        sorted.sort_unstable();
        sorted
    }

    /// Get all active channels
    pub fn active_channels(&self) -> Vec<u16> {
        let mut channels = HashSet::new();
        for msg in &self.messages {
            match msg {
                LogObject::CanMessage(m) => {
                    channels.insert(m.channel);
                }
                LogObject::CanFdMessage(m) => {
                    channels.insert(m.channel);
                }
                LogObject::LinMessage(m) => {
                    channels.insert(m.channel);
                }
                _ => {}
            }
        }
        let mut sorted: Vec<_> = channels.into_iter().collect();
        sorted.sort_unstable();
        sorted
    }

    // Helper methods

    fn is_filter_empty(filter: &MessageFilter) -> bool {
        filter.id_filter.is_none()
            && filter.channel_filter.is_none()
            && !filter.can_only
            && !filter.lin_only
            && !filter.errors_only
    }

    fn filters_equal(a: &MessageFilter, b: &MessageFilter) -> bool {
        a.id_filter == b.id_filter
            && a.channel_filter == b.channel_filter
            && a.can_only == b.can_only
            && a.lin_only == b.lin_only
            && a.errors_only == b.errors_only
    }

    fn extract_timestamp(msg: &LogObject) -> Option<f64> {
        // Use the timestamp() method from LogObject
        // Check if message has a timestamp (CAN/LIN messages do)
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

impl Default for LogProcessor {
    fn default() -> Self {
        Self::new()
    }
}

/// Message grouping result
#[derive(Clone, Debug)]
pub struct GroupedMessages {
    /// Groups by ID
    pub by_id: HashMap<u32, Vec<LogObject>>,
    /// Groups by channel
    pub by_channel: HashMap<u16, Vec<LogObject>>,
    /// CAN messages only
    pub can_messages: Vec<LogObject>,
    /// LIN messages only
    pub lin_messages: Vec<LogObject>,
}

/// Group messages by various criteria
pub fn group_messages(messages: &[LogObject]) -> GroupedMessages {
    let mut by_id: HashMap<u32, Vec<LogObject>> = HashMap::new();
    let mut by_channel: HashMap<u16, Vec<LogObject>> = HashMap::new();
    let mut can_messages = Vec::new();
    let mut lin_messages = Vec::new();

    for msg in messages {
        // Group by ID
        let id = match msg {
            LogObject::CanMessage(m) => m.id as u32,
            LogObject::CanFdMessage(m) => m.id as u32,
            LogObject::LinMessage(m) => m.id as u32,
            _ => continue,
        };

        by_id.entry(id).or_default().push(msg.clone());

        // Group by channel
        let channel = match msg {
            LogObject::CanMessage(m) => m.channel,
            LogObject::CanFdMessage(m) => m.channel,
            LogObject::LinMessage(m) => m.channel,
            _ => continue,
        };

        by_channel.entry(channel).or_default().push(msg.clone());

        // Separate CAN/LIN
        match msg {
            LogObject::CanMessage(_) | LogObject::CanFdMessage(_) => {
                can_messages.push(msg.clone());
            }
            LogObject::LinMessage(_) => {
                lin_messages.push(msg.clone());
            }
            _ => {}
        }
    }

    GroupedMessages {
        by_id,
        by_channel,
        can_messages,
        lin_messages,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_filter() {
        let filter = MessageFilter::new().with_id(0x123).with_channel(1);

        assert_eq!(filter.id_filter, Some(0x123));
        assert_eq!(filter.channel_filter, Some(1));
    }

    #[test]
    fn test_log_processor() {
        let mut processor = LogProcessor::new();
        assert_eq!(processor.message_count(), 0);

        processor.add_message(LogObject::CanMessage(blf::CanMessage {
            channel: 1,
            id: 0x123,
            dlc: 8,
            data: vec![0; 8],
            timestamp: 1000,
            dir: blf::TxDirection::Rx,
            ..Default::default()
        }));

        assert_eq!(processor.message_count(), 1);
        assert_eq!(processor.unique_can_ids(), vec![0x123]);
        assert_eq!(processor.active_channels(), vec![1]);
    }

    #[test]
    fn test_statistics() {
        let mut processor = LogProcessor::new();

        processor.add_messages(vec![
            LogObject::CanMessage(blf::CanMessage {
                channel: 1,
                id: 0x100,
                dlc: 8,
                data: vec![0; 8],
                timestamp: 0,
                dir: blf::TxDirection::Rx,
                ..Default::default()
            }),
            LogObject::CanMessage(blf::CanMessage {
                channel: 1,
                id: 0x200,
                dlc: 8,
                data: vec![0; 8],
                timestamp: 1_000_000_000,
                dir: blf::TxDirection::Rx,
                ..Default::default()
            }),
        ]);

        let stats = processor.calculate_statistics();
        assert_eq!(stats.total_messages, 2);
        assert_eq!(stats.can_messages, 2);
        assert_eq!(stats.unique_can_ids, 2);
        assert_eq!(stats.active_channels, 1);
    }

    #[test]
    fn test_filtering() {
        let mut processor = LogProcessor::new();

        processor.add_messages(vec![
            LogObject::CanMessage(blf::CanMessage {
                channel: 1,
                id: 0x100,
                dlc: 8,
                data: vec![0; 8],
                timestamp: 0,
                dir: blf::TxDirection::Rx,
                ..Default::default()
            }),
            LogObject::CanMessage(blf::CanMessage {
                channel: 2,
                id: 0x200,
                dlc: 8,
                data: vec![0; 8],
                timestamp: 0,
                dir: blf::TxDirection::Rx,
                ..Default::default()
            }),
        ]);

        // Filter by channel 1
        let filtered = processor.apply_filter(MessageFilter::new().with_channel(1));
        assert_eq!(filtered.len(), 1);

        // Filter by ID
        let filtered = processor.apply_filter(MessageFilter::new().with_id(0x200));
        assert_eq!(filtered.len(), 1);

        // Clear filter
        let all = processor.clear_filter();
        assert_eq!(all.len(), 2);
    }
}
