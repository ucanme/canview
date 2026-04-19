//! Message service
//!
//! Provides message loading, parsing, and management functionality.
//! This service operates on LogObject from the blf crate and provides
//! a clean interface for message operations.

use blf::LogObject;
use std::collections::HashSet;

/// Message service for managing CAN/LIN log messages
///
/// This service provides operations for managing collections of log messages
/// including filtering, searching, and grouping operations.
pub struct MessageService {
    /// All loaded messages
    messages: Vec<LogObject>,
}

impl MessageService {
    /// Create a new message service
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
        }
    }

    /// Create a new message service with initial messages
    pub fn with_messages(messages: Vec<LogObject>) -> Self {
        Self { messages }
    }

    /// Set messages and replace existing ones
    pub fn set_messages(&mut self, messages: Vec<LogObject>) {
        self.messages = messages;
    }

    /// Get all messages
    pub fn messages(&self) -> &[LogObject] {
        &self.messages
    }

    /// Get message count
    pub fn count(&self) -> usize {
        self.messages.len()
    }

    /// Check if service has any messages
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    /// Clear all messages
    pub fn clear(&mut self) {
        self.messages.clear();
    }

    /// Add a single message
    pub fn add_message(&mut self, message: LogObject) {
        self.messages.push(message);
    }

    /// Add multiple messages
    pub fn add_messages(&mut self, messages: Vec<LogObject>) {
        self.messages.extend(messages);
    }

    /// Find messages by ID
    pub fn find_by_id(&self, id: u32) -> Vec<&LogObject> {
        self.messages
            .iter()
            .filter(|msg| match msg {
                LogObject::CanMessage(m) if m.id as u32 == id => true,
                LogObject::CanMessage2(m) if m.id as u32 == id => true,
                LogObject::CanFdMessage(m) if m.id as u32 == id => true,
                LogObject::CanFdMessage64(m) if m.id as u32 == id => true,
                LogObject::LinMessage(m) if m.id as u32 == id => true,
                _ => false,
            })
            .collect()
    }

    /// Find messages by channel
    pub fn find_by_channel(&self, channel: u16) -> Vec<&LogObject> {
        self.messages
            .iter()
            .filter(|msg| msg.channel() == Some(channel))
            .collect()
    }

    /// Find messages by ID and channel
    pub fn find_by_id_and_channel(&self, id: u32, channel: u16) -> Vec<&LogObject> {
        self.messages
            .iter()
            .filter(|msg| {
                let matches_id = match msg {
                    LogObject::CanMessage(m) if m.id as u32 == id => true,
                    LogObject::CanMessage2(m) if m.id as u32 == id => true,
                    LogObject::CanFdMessage(m) if m.id as u32 == id => true,
                    LogObject::CanFdMessage64(m) if m.id as u32 == id => true,
                    LogObject::LinMessage(m) if m.id as u32 == id => true,
                    _ => false,
                };
                matches_id && msg.channel() == Some(channel)
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
                LogObject::CanMessage2(m) => {
                    ids.insert(m.id as u32);
                }
                LogObject::CanFdMessage(m) => {
                    ids.insert(m.id as u32);
                }
                LogObject::CanFdMessage64(m) => {
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
            if let Some(ch) = msg.channel() { channels.insert(ch); };
        }
        let mut sorted: Vec<_> = channels.into_iter().collect();
        sorted.sort_unstable();
        sorted
    }

    /// Get CAN messages only
    pub fn can_messages(&self) -> Vec<&LogObject> {
        self.messages
            .iter()
            .filter(|msg| {
                matches!(
                    msg,
                    LogObject::CanMessage(_)
                        | LogObject::CanMessage2(_)
                        | LogObject::CanFdMessage(_)
                        | LogObject::CanFdMessage64(_)
                )
            })
            .collect()
    }

    /// Get LIN messages only
    pub fn lin_messages(&self) -> Vec<&LogObject> {
        self.messages
            .iter()
            .filter(|msg| matches!(msg, LogObject::LinMessage(_) | LogObject::LinMessage2(_)))
            .collect()
    }

    /// Get error frames only
    pub fn error_frames(&self) -> Vec<&LogObject> {
        self.messages
            .iter()
            .filter(|msg| matches!(msg, LogObject::CanErrorFrame(_)))
            .collect()
    }

    /// Get messages in time range
    pub fn in_time_range(&self, start_seconds: f64, end_seconds: f64) -> Vec<&LogObject> {
        self.messages
            .iter()
            .filter(|msg| {
                if let Some(timestamp) = extract_timestamp(msg) {
                    timestamp >= start_seconds && timestamp <= end_seconds
                } else {
                    false
                }
            })
            .collect()
    }

    /// Group messages by various criteria
    pub fn group_messages(&self) -> MessageGroups {
        let mut by_id: HashSet<u32> = HashSet::new();
        let mut by_channel: HashSet<u16> = HashSet::new();
        let mut can_count = 0;
        let mut lin_count = 0;
        let mut error_count = 0;

        for msg in &self.messages {
            // Track IDs
            match msg {
                LogObject::CanMessage(m) => {
                    by_id.insert(m.id as u32);
                    by_channel.insert(m.channel);
                    can_count += 1;
                }
                LogObject::CanMessage2(m) => {
                    by_id.insert(m.id as u32);
                    by_channel.insert(m.channel);
                    can_count += 1;
                }
                LogObject::CanFdMessage(m) => {
                    by_id.insert(m.id as u32);
                    by_channel.insert(m.channel);
                    can_count += 1;
                }
                LogObject::CanFdMessage64(m) => {
                    by_id.insert(m.id as u32);
                    by_channel.insert(m.channel as u16);
                    can_count += 1;
                }
                LogObject::LinMessage(m) => {
                    by_id.insert(m.id as u32);
                    by_channel.insert(m.channel);
                    lin_count += 1;
                }
                LogObject::LinMessage2(_) => {
                    lin_count += 1;
                }
                LogObject::CanErrorFrame(_) => {
                    error_count += 1;
                }
                _ => {}
            }
        }

        MessageGroups {
            unique_ids: by_id.len(),
            unique_channels: by_channel.len(),
            can_count,
            lin_count,
            error_count,
        }
    }
}

impl Default for MessageService {
    fn default() -> Self {
        Self::new()
    }
}

/// Message grouping statistics
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MessageGroups {
    /// Number of unique IDs
    pub unique_ids: usize,
    /// Number of unique channels
    pub unique_channels: usize,
    /// Number of CAN messages
    pub can_count: usize,
    /// Number of LIN messages
    pub lin_count: usize,
    /// Number of error frames
    pub error_count: usize,
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

#[cfg(test)]
mod tests {
    use super::*;
    use blf::CanMessage;
    use blf::TxDirection;

    #[test]
    fn test_message_service_new() {
        let service = MessageService::new();
        assert_eq!(service.count(), 0);
        assert!(service.is_empty());
    }

    #[test]
    fn test_add_message() {
        let mut service = MessageService::new();
        service.add_message(LogObject::CanMessage(CanMessage {
            channel: 1,
            id: 0x123,
            dlc: 8,
            data: vec![0; 8],
            timestamp: 1000,
            dir: TxDirection::Rx,
            ..Default::default()
        }));

        assert_eq!(service.count(), 1);
        assert!(!service.is_empty());
    }

    #[test]
    fn test_find_by_id() {
        let mut service = MessageService::new();
        service.add_message(LogObject::CanMessage(CanMessage {
            channel: 1,
            id: 0x123,
            dlc: 8,
            data: vec![0; 8],
            timestamp: 1000,
            dir: TxDirection::Rx,
            ..Default::default()
        }));
        service.add_message(LogObject::CanMessage(CanMessage {
            channel: 2,
            id: 0x456,
            dlc: 8,
            data: vec![0; 8],
            timestamp: 2000,
            dir: TxDirection::Rx,
            ..Default::default()
        }));

        let results = service.find_by_id(0x123);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_unique_can_ids() {
        let mut service = MessageService::new();
        service.add_message(LogObject::CanMessage(CanMessage {
            channel: 1,
            id: 0x123,
            dlc: 8,
            data: vec![0; 8],
            timestamp: 1000,
            dir: TxDirection::Rx,
            ..Default::default()
        }));
        service.add_message(LogObject::CanMessage(CanMessage {
            channel: 1,
            id: 0x456,
            dlc: 8,
            data: vec![0; 8],
            timestamp: 2000,
            dir: TxDirection::Rx,
            ..Default::default()
        }));

        let ids = service.unique_can_ids();
        assert_eq!(ids.len(), 2);
        assert_eq!(ids, vec![0x123, 0x456]);
    }

    #[test]
    fn test_clear() {
        let mut service = MessageService::new();
        service.add_message(LogObject::CanMessage(CanMessage {
            channel: 1,
            id: 0x123,
            dlc: 8,
            data: vec![0; 8],
            timestamp: 1000,
            dir: TxDirection::Rx,
            ..Default::default()
        }));

        service.clear();
        assert_eq!(service.count(), 0);
    }

    #[test]
    fn test_group_messages() {
        let mut service = MessageService::new();
        service.add_message(LogObject::CanMessage(CanMessage {
            channel: 1,
            id: 0x123,
            dlc: 8,
            data: vec![0; 8],
            timestamp: 1000,
            dir: TxDirection::Rx,
            ..Default::default()
        }));

        let groups = service.group_messages();
        assert_eq!(groups.can_count, 1);
        assert_eq!(groups.unique_ids, 1);
        assert_eq!(groups.unique_channels, 1);
    }
}
