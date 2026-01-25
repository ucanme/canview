//! Filter handling utilities
//!
//! This module contains utility functions for filtering operations
//! such as extracting unique channels from message lists.

use blf::LogObject;
use std::collections::HashSet;

/// Extract unique channel numbers from a list of log objects
///
/// This function iterates through all message types and collects
/// unique channel numbers, which can be used for filter dropdowns.
///
/// # Arguments
/// * `messages` - Slice of log objects to process
///
/// # Returns
/// A sorted vector of unique channel numbers
///
/// # Example
/// ```ignore
/// let channels = extract_unique_channels(&messages);
/// // Returns: vec![1, 2, 3]
/// ```
pub fn extract_unique_channels(messages: &[LogObject]) -> Vec<u16> {
    let mut unique_channels = HashSet::new();

    for msg in messages.iter() {
        match msg {
            LogObject::CanMessage(m) => {
                unique_channels.insert(m.channel);
            }
            LogObject::CanMessage2(m) => {
                unique_channels.insert(m.channel);
            }
            LogObject::CanFdMessage(m) => {
                unique_channels.insert(m.channel);
            }
            LogObject::CanFdMessage64(m) => {
                unique_channels.insert(m.channel as u16);
            }
            LogObject::LinMessage(m) => {
                unique_channels.insert(m.channel);
            }
            LogObject::LinMessage2(_) => {
                // LIN2 doesn't have channel info
            }
            _ => {}
        }
    }

    let mut channel_list: Vec<u16> = unique_channels.into_iter().collect();
    channel_list.sort();
    channel_list
}

/// Filter messages by channel number
///
/// # Arguments
/// * `messages` - Slice of log objects to filter
/// * `selected_channels` - Set of channel numbers to include
///
/// # Returns
/// A new vector containing only messages from the selected channels
///
/// # Example
/// ```ignore
/// let channels = HashSet::from([1, 2]);
/// let filtered = filter_by_channel(&messages, &channels);
/// ```
pub fn filter_by_channel(
    messages: &[LogObject],
    selected_channels: &HashSet<u16>,
) -> Vec<LogObject> {
    if selected_channels.is_empty() {
        return messages.to_vec();
    }

    messages
        .iter()
        .filter(|msg| {
            match msg {
                LogObject::CanMessage(m) => selected_channels.contains(&m.channel),
                LogObject::CanMessage2(m) => selected_channels.contains(&m.channel),
                LogObject::CanFdMessage(m) => selected_channels.contains(&m.channel),
                LogObject::CanFdMessage64(m) => selected_channels.contains(&(m.channel as u16)),
                LogObject::LinMessage(m) => selected_channels.contains(&m.channel),
                LogObject::LinMessage2(_) => true, // Include LIN2 if any filter is active
                _ => false,
            }
        })
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_unique_channels_empty() {
        let messages: Vec<LogObject> = vec![];
        let channels = extract_unique_channels(&messages);
        assert_eq!(channels, vec![]);
    }

    #[test]
    fn test_filter_by_channel_empty_selection() {
        let messages: Vec<LogObject> = vec![];
        let selected = HashSet::new();
        let filtered = filter_by_channel(&messages, &selected);
        assert_eq!(filtered.len(), 0);
    }
}
