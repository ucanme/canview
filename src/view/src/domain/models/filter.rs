//! Filter domain model
//!
//! Pure business model for message filtering criteria.
//! No UI framework dependencies.

use std::collections::HashSet;
use std::fmt;

use super::message::{Message, MessageType, Direction};

/// Filter type enumeration
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FilterType {
    /// Filter by message ID
    Id,
    /// Filter by channel number
    Channel,
    /// Filter by message type
    MessageType,
    /// Filter by direction (TX/RX)
    Direction,
    /// Filter by signal name
    SignalName,
    /// Filter by data content
    DataPattern,
    /// Custom filter with multiple criteria
    Custom,
}

impl FilterType {
    /// Get display name for the filter type
    pub fn as_str(&self) -> &'static str {
        match self {
            FilterType::Id => "ID",
            FilterType::Channel => "Channel",
            FilterType::MessageType => "Type",
            FilterType::Direction => "Direction",
            FilterType::SignalName => "Signal",
            FilterType::DataPattern => "Data",
            FilterType::Custom => "Custom",
        }
    }
}

impl fmt::Display for FilterType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// ID filter specification
#[derive(Clone, Debug, PartialEq)]
pub struct IdFilter {
    /// Message ID to filter by
    pub id: u32,
    /// Whether to match extended frames (29-bit IDs)
    pub extended_only: bool,
}

impl IdFilter {
    /// Create a new ID filter
    pub fn new(id: u32) -> Self {
        Self {
            id,
            extended_only: false,
        }
    }

    /// Create a filter for standard IDs (11-bit)
    pub fn standard(id: u32) -> Self {
        Self {
            id,
            extended_only: false,
        }
    }

    /// Create a filter for extended IDs (29-bit)
    pub fn extended(id: u32) -> Self {
        Self {
            id,
            extended_only: true,
        }
    }

    /// Check if a message matches this filter
    pub fn matches(&self, message: &Message) -> bool {
        if message.id != self.id {
            return false;
        }

        if self.extended_only && !message.is_extended() {
            return false;
        }

        true
    }
}

/// Channel filter specification
#[derive(Clone, Debug, PartialEq)]
pub struct ChannelFilter {
    /// Channel number to filter by
    pub channel: u16,
}

impl ChannelFilter {
    /// Create a new channel filter
    pub fn new(channel: u16) -> Self {
        Self { channel }
    }

    /// Check if a message matches this filter
    pub fn matches(&self, message: &Message) -> bool {
        message.channel == self.channel
    }
}

/// Message type filter specification
#[derive(Clone, Debug, PartialEq)]
pub struct MessageTypeFilter {
    /// Allowed message types
    pub allowed_types: HashSet<MessageType>,
}

impl MessageTypeFilter {
    /// Create a new message type filter
    pub fn new(types: HashSet<MessageType>) -> Self {
        Self {
            allowed_types: types,
        }
    }

    /// Create a filter for a single type
    pub fn single(message_type: MessageType) -> Self {
        let mut types = HashSet::new();
        types.insert(message_type);
        Self { allowed_types: types }
    }

    /// Create a filter for CAN messages only
    pub fn can_only() -> Self {
        Self::single(MessageType::Can)
    }

    /// Check if a message matches this filter
    pub fn matches(&self, message: &Message) -> bool {
        self.allowed_types.contains(&message.message_type)
    }

    /// Add a message type to the filter
    pub fn add_type(&mut self, message_type: MessageType) {
        self.allowed_types.insert(message_type);
    }

    /// Remove a message type from the filter
    pub fn remove_type(&mut self, message_type: &MessageType) {
        self.allowed_types.remove(message_type);
    }
}

/// Direction filter specification
#[derive(Clone, Debug, PartialEq)]
pub struct DirectionFilter {
    /// Allowed directions
    pub allowed_directions: HashSet<Direction>,
}

impl DirectionFilter {
    /// Create a new direction filter
    pub fn new(directions: HashSet<Direction>) -> Self {
        Self {
            allowed_directions: directions,
        }
    }

    /// Create a filter for RX messages only
    pub fn rx_only() -> Self {
        let mut directions = HashSet::new();
        directions.insert(Direction::Rx);
        Self {
            allowed_directions: directions,
        }
    }

    /// Create a filter for TX messages only
    pub fn tx_only() -> Self {
        let mut directions = HashSet::new();
        directions.insert(Direction::Tx);
        Self {
            allowed_directions: directions,
        }
    }

    /// Check if a message matches this filter
    pub fn matches(&self, message: &Message) -> bool {
        self.allowed_directions.contains(&message.direction)
    }

    /// Add a direction to the filter
    pub fn add_direction(&mut self, direction: Direction) {
        self.allowed_directions.insert(direction);
    }

    /// Remove a direction from the filter
    pub fn remove_direction(&mut self, direction: &Direction) {
        self.allowed_directions.remove(direction);
    }
}

/// Signal name filter specification
#[derive(Clone, Debug, PartialEq)]
pub struct SignalNameFilter {
    /// Signal name pattern (supports partial matching)
    pub pattern: String,
    /// Case-sensitive matching
    pub case_sensitive: bool,
}

impl SignalNameFilter {
    /// Create a new signal name filter
    pub fn new(pattern: String) -> Self {
        Self {
            pattern,
            case_sensitive: false,
        }
    }

    /// Create a case-sensitive filter
    pub fn case_sensitive(pattern: String) -> Self {
        Self {
            pattern,
            case_sensitive: true,
        }
    }

    /// Check if a message matches this filter
    pub fn matches(&self, message: &Message) -> bool {
        if let Some(name) = &message.name {
            if self.case_sensitive {
                name.contains(&self.pattern)
            } else {
                name.to_lowercase().contains(&self.pattern.to_lowercase())
            }
        } else {
            false
        }
    }
}

/// Data pattern filter specification
#[derive(Clone, Debug, PartialEq)]
pub struct DataPatternFilter {
    /// Byte pattern to match
    pub pattern: Vec<u8>,
    /// Mask for pattern matching (optional)
    pub mask: Option<Vec<u8>>,
    /// Match position (None means anywhere)
    pub position: Option<usize>,
}

impl DataPatternFilter {
    /// Create a new data pattern filter
    pub fn new(pattern: Vec<u8>) -> Self {
        Self {
            pattern,
            mask: None,
            position: None,
        }
    }

    /// Create a filter with mask
    pub fn with_mask(pattern: Vec<u8>, mask: Vec<u8>) -> Self {
        Self {
            pattern,
            mask: Some(mask),
            position: None,
        }
    }

    /// Create a filter for specific position
    pub fn at_position(pattern: Vec<u8>, position: usize) -> Self {
        Self {
            pattern,
            mask: None,
            position: Some(position),
        }
    }

    /// Check if a message matches this filter
    pub fn matches(&self, message: &Message) -> bool {
        if self.pattern.is_empty() {
            return true;
        }

        let data = &message.data;

        // Check position-specific filter
        if let Some(pos) = self.position {
            if pos + self.pattern.len() > data.len() {
                return false;
            }

            for (i, &pattern_byte) in self.pattern.iter().enumerate() {
                let data_byte = data[pos + i];
                let masked_data = if let Some(mask) = &self.mask {
                    data_byte & mask[i]
                } else {
                    data_byte
                };

                if masked_data != pattern_byte {
                    return false;
                }
            }
            return true;
        }

        // Check if pattern appears anywhere in data
        for start in 0..=data.len().saturating_sub(self.pattern.len()) {
            let mut matches = true;

            for (i, &pattern_byte) in self.pattern.iter().enumerate() {
                let data_byte = data[start + i];
                let masked_data = if let Some(mask) = &self.mask {
                    data_byte & mask[i]
                } else {
                    data_byte
                };

                if masked_data != pattern_byte {
                    matches = false;
                    break;
                }
            }

            if matches {
                return true;
            }
        }

        false
    }
}

/// Composite filter criteria
///
/// Combines multiple filters with logical AND/OR relationships
#[derive(Clone, Debug, PartialEq)]
pub struct FilterCriteria {
    /// ID filter (optional)
    pub id_filter: Option<IdFilter>,
    /// Channel filter (optional)
    pub channel_filter: Option<ChannelFilter>,
    /// Message type filter (optional)
    pub message_type_filter: Option<MessageTypeFilter>,
    /// Direction filter (optional)
    pub direction_filter: Option<DirectionFilter>,
    /// Signal name filter (optional)
    pub signal_filter: Option<SignalNameFilter>,
    /// Data pattern filter (optional)
    pub data_filter: Option<DataPatternFilter>,
    /// Whether to use AND logic (all filters must match)
    /// or OR logic (any filter can match)
    pub match_all: bool,
}

impl Default for FilterCriteria {
    fn default() -> Self {
        Self {
            id_filter: None,
            channel_filter: None,
            message_type_filter: None,
            direction_filter: None,
            signal_filter: None,
            data_filter: None,
            match_all: true,
        }
    }
}

impl FilterCriteria {
    /// Create a new filter criteria with AND logic
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a filter criteria with OR logic
    pub fn any_match() -> Self {
        Self {
            match_all: false,
            ..Default::default()
        }
    }

    /// Add an ID filter
    pub fn with_id(mut self, id: u32) -> Self {
        self.id_filter = Some(IdFilter::new(id));
        self
    }

    /// Add a channel filter
    pub fn with_channel(mut self, channel: u16) -> Self {
        self.channel_filter = Some(ChannelFilter::new(channel));
        self
    }

    /// Add a message type filter
    pub fn with_message_type(mut self, message_type: MessageType) -> Self {
        self.message_type_filter = Some(MessageTypeFilter::single(message_type));
        self
    }

    /// Add a direction filter
    pub fn with_direction(mut self, direction: Direction) -> Self {
        self.direction_filter = Some(DirectionFilter::new(
            [direction].into_iter().collect()
        ));
        self
    }

    /// Add a signal name filter
    pub fn with_signal(mut self, pattern: String) -> Self {
        self.signal_filter = Some(SignalNameFilter::new(pattern));
        self
    }

    /// Set AND logic
    pub fn match_all(mut self) -> Self {
        self.match_all = true;
        self
    }

    /// Set OR logic
    pub fn match_any(mut self) -> Self {
        self.match_all = false;
        self
    }

    /// Check if a message matches all the criteria
    pub fn matches(&self, message: &Message) -> bool {
        let id_match = self.id_filter.as_ref().map_or(true, |f| f.matches(message));
        let channel_match = self.channel_filter.as_ref().map_or(true, |f| f.matches(message));
        let type_match = self.message_type_filter.as_ref().map_or(true, |f| f.matches(message));
        let direction_match = self.direction_filter.as_ref().map_or(true, |f| f.matches(message));
        let signal_match = self.signal_filter.as_ref().map_or(true, |f| f.matches(message));
        let data_match = self.data_filter.as_ref().map_or(true, |f| f.matches(message));

        if self.match_all {
            // AND logic: all filters must match
            id_match && channel_match && type_match && direction_match && signal_match && data_match
        } else {
            // OR logic: any filter can match
            id_match || channel_match || type_match || direction_match || signal_match || data_match
        }
    }

    /// Filter a list of messages
    pub fn filter_messages(&self, messages: &[Message]) -> Vec<Message> {
        messages
            .iter()
            .filter(|msg| self.matches(msg))
            .cloned()
            .collect()
    }

    /// Check if any filter is active
    pub fn is_active(&self) -> bool {
        self.id_filter.is_some()
            || self.channel_filter.is_some()
            || self.message_type_filter.is_some()
            || self.direction_filter.is_some()
            || self.signal_filter.is_some()
            || self.data_filter.is_some()
    }

    /// Clear all filters
    pub fn clear(&mut self) {
        self.id_filter = None;
        self.channel_filter = None;
        self.message_type_filter = None;
        self.direction_filter = None;
        self.signal_filter = None;
        self.data_filter = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;

    fn create_test_message(id: u32, channel: u16) -> Message {
        Message::new(
            0,
            NaiveDateTime::default(),
            channel,
            id,
            MessageType::Can,
            Direction::Rx,
            Some("TestSignal".to_string()),
            8,
            vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08],
            0.0,
        )
    }

    #[test]
    fn test_id_filter() {
        let filter = IdFilter::new(0x123);
        let msg = create_test_message(0x123, 1);
        assert!(filter.matches(&msg));

        let msg2 = create_test_message(0x456, 1);
        assert!(!filter.matches(&msg2));
    }

    #[test]
    fn test_extended_id_filter() {
        let filter = IdFilter::extended(0x12345678);
        let ext_msg = create_test_message(0x12345678, 1);
        assert!(ext_msg.is_extended());
        assert!(filter.matches(&ext_msg));

        let std_msg = create_test_message(0x123, 1);
        assert!(!std_msg.is_extended());
        assert!(!filter.matches(&std_msg));
    }

    #[test]
    fn test_channel_filter() {
        let filter = ChannelFilter::new(1);
        let msg1 = create_test_message(0x123, 1);
        assert!(filter.matches(&msg1));

        let msg2 = create_test_message(0x123, 2);
        assert!(!filter.matches(&msg2));
    }

    #[test]
    fn test_message_type_filter() {
        let filter = MessageTypeFilter::can_only();
        let can_msg = create_test_message(0x123, 1);
        assert!(filter.matches(&can_msg));

        let error_msg = Message::new(
            0,
            NaiveDateTime::default(),
            1,
            0x123,
            MessageType::Error,
            Direction::Rx,
            None,
            0,
            vec![],
            0.0,
        );
        assert!(!filter.matches(&error_msg));
    }

    #[test]
    fn test_direction_filter() {
        let filter = DirectionFilter::rx_only();
        let rx_msg = create_test_message(0x123, 1);
        assert!(filter.matches(&rx_msg));

        let tx_msg = Message::new(
            0,
            NaiveDateTime::default(),
            1,
            0x123,
            MessageType::Can,
            Direction::Tx,
            None,
            8,
            vec![0; 8],
            0.0,
        );
        assert!(!filter.matches(&tx_msg));
    }

    #[test]
    fn test_signal_filter() {
        let filter = SignalNameFilter::new("test".to_string());
        let msg_with_name = create_test_message(0x123, 1);
        assert!(filter.matches(&msg_with_name));

        let msg_without_name = Message::new(
            0,
            NaiveDateTime::default(),
            1,
            0x456,
            MessageType::Can,
            Direction::Rx,
            None,
            8,
            vec![0; 8],
            0.0,
        );
        assert!(!filter.matches(&msg_without_name));
    }

    #[test]
    fn test_data_pattern_filter() {
        let filter = DataPatternFilter::new(vec![0x01, 0x02, 0x03]);
        let msg = create_test_message(0x123, 1);
        assert!(filter.matches(&msg));

        let msg2 = Message::new(
            0,
            NaiveDateTime::default(),
            1,
            0x456,
            MessageType::Can,
            Direction::Rx,
            None,
            8,
            vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF],
            0.0,
        );
        assert!(!filter.matches(&msg2));
    }

    #[test]
    fn test_filter_criteria_and() {
        let criteria = FilterCriteria::new()
            .with_id(0x123)
            .with_channel(1)
            .match_all();

        let msg = create_test_message(0x123, 1);
        assert!(criteria.matches(&msg));

        let msg_wrong_channel = create_test_message(0x123, 2);
        assert!(!criteria.matches(&msg_wrong_channel));

        let msg_wrong_id = create_test_message(0x456, 1);
        assert!(!criteria.matches(&msg_wrong_id));
    }

    #[test]
    fn test_filter_criteria_or() {
        let criteria = FilterCriteria::new()
            .with_id(0x123)
            .with_channel(2)
            .match_any();

        let msg1 = create_test_message(0x123, 1); // Matches ID
        assert!(criteria.matches(&msg1));

        let msg2 = create_test_message(0x456, 2); // Matches channel
        assert!(criteria.matches(&msg2));

        let msg3 = create_test_message(0x789, 3); // Matches neither
        assert!(!criteria.matches(&msg3));
    }

    #[test]
    fn test_filter_criteria_is_active() {
        let mut criteria = FilterCriteria::new();
        assert!(!criteria.is_active());

        criteria.id_filter = Some(IdFilter::new(0x123));
        assert!(criteria.is_active());

        criteria.clear();
        assert!(!criteria.is_active());
    }

    #[test]
    fn test_filter_messages() {
        let msg1 = create_test_message(0x123, 1);
        let msg2 = create_test_message(0x456, 2);
        let msg3 = create_test_message(0x123, 2);

        let messages = vec![msg1.clone(), msg2, msg3];

        let criteria = FilterCriteria::new().with_id(0x123);
        let filtered = criteria.filter_messages(&messages);

        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|m| m.id == 0x123));
    }
}
