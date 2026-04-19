//! Filter service
//!
//! Provides message filtering functionality using domain-level filter criteria.
//! This service uses the FilterCriteria from domain/models/filter.

use blf::LogObject;
use crate::domain::models::filter::FilterCriteria;

/// Filter service for filtering log messages
///
/// This service provides filtering capabilities using domain-level
/// filter criteria. It maintains a cache of filtered results for
/// performance optimization.
pub struct FilterService {
    /// Current filter criteria
    current_filter: Option<FilterCriteria>,
    /// Cached filtered message indices
    filtered_indices: Option<Vec<usize>>,
}

impl FilterService {
    /// Create a new filter service
    pub fn new() -> Self {
        Self {
            current_filter: None,
            filtered_indices: None,
        }
    }

    /// Apply a filter to messages
    ///
    /// Returns indices of messages that match the filter criteria.
    /// This method caches results for performance.
    pub fn apply_filter(
        &mut self,
        messages: &[LogObject],
        filter: &FilterCriteria,
    ) -> Vec<usize> {
        // Check if filter is the same as current
        if let Some(current) = &self.current_filter {
            if current == filter && self.filtered_indices.is_some() {
                // Same filter, return cached result
                return self.filtered_indices.clone().unwrap();
            }
        }

        // Apply new filter
        self.current_filter = Some(filter.clone());

        if !filter.is_active() {
            // No filtering needed, return all indices
            let all_indices: Vec<usize> = (0..messages.len()).collect();
            self.filtered_indices = Some(all_indices.clone());
            return all_indices;
        }

        // Apply filter using LogObject methods
        let filtered: Vec<usize> = messages
            .iter()
            .enumerate()
            .filter(|(idx, msg)| self.message_matches_filter(msg, filter))
            .map(|(idx, _)| idx)
            .collect();

        self.filtered_indices = Some(filtered.clone());
        filtered
    }

    /// Get filtered messages by index
    ///
    /// Returns the actual messages corresponding to the filtered indices
    pub fn get_filtered_messages<'a>(
        &self,
        messages: &'a [LogObject],
    ) -> Vec<&'a LogObject> {
        if let Some(indices) = &self.filtered_indices {
            indices
                .iter()
                .filter_map(|&idx| messages.get(idx))
                .collect()
        } else {
            // No filter applied, return all messages
            messages.iter().collect()
        }
    }

    /// Get count of filtered messages
    pub fn filtered_count(&self) -> usize {
        self.filtered_indices
            .as_ref()
            .map(|indices| indices.len())
            .unwrap_or(0)
    }

    /// Clear the current filter
    pub fn clear_filter(&mut self) {
        self.current_filter = None;
        self.filtered_indices = None;
    }

    /// Get current filter
    pub fn current_filter(&self) -> Option<&FilterCriteria> {
        self.current_filter.as_ref()
    }

    /// Check if a filter is currently active
    pub fn is_filter_active(&self) -> bool {
        self.current_filter.is_some()
            && self.current_filter
                .as_ref()
                .map(|f| f.is_active())
                .unwrap_or(false)
    }

    /// Invalidate the filter cache
    ///
    /// Call this when the underlying messages change
    pub fn invalidate_cache(&mut self) {
        self.filtered_indices = None;
    }

    // Helper methods

    /// Check if a message matches the filter criteria
    fn message_matches_filter(&self, msg: &LogObject, filter: &FilterCriteria) -> bool {
        // Check ID filter
        if let Some(id_filter) = &filter.id_filter {
            let msg_id = match msg {
                LogObject::CanMessage(m) => Some(m.id as u32),
                LogObject::CanMessage2(m) => Some(m.id as u32),
                LogObject::CanFdMessage(m) => Some(m.id as u32),
                LogObject::CanFdMessage64(m) => Some(m.id as u32),
                LogObject::LinMessage(m) => Some(m.id as u32),
                _ => None,
            };

            if !id_filter.matches_id(msg_id) {
                return false;
            }
        }

        // Check channel filter
        if let Some(channel_filter) = &filter.channel_filter {
            let channel = msg.channel();
            if channel != Some(channel_filter.channel) {
                return false;
            }
        }

        // Additional filters can be added here as needed

        true
    }
}

// Helper extension for IdFilter
trait IdFilterHelper {
    fn matches_id(&self, msg_id: Option<u32>) -> bool;
}

impl IdFilterHelper for crate::domain::models::filter::IdFilter {
    fn matches_id(&self, msg_id: Option<u32>) -> bool {
        match msg_id {
            Some(id) => {
                if id != self.id {
                    return false;
                }

                if self.extended_only && id <= 0x7FF {
                    return false;
                }

                true
            }
            None => false,
        }
    }
}

impl Default for FilterService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_can_message(channel: u16, id: u32) -> LogObject {
        // Note: Using a minimal message structure
        // The actual structure depends on the blf crate version
        LogObject::CanMessage(blf::CanMessage {
            channel,
            id,
            dlc: 8,
            data: vec![0; 8],
        })
    }

    #[test]
    fn test_filter_service_new() {
        let service = FilterService::new();
        assert!(!service.is_filter_active());
        assert_eq!(service.filtered_count(), 0);
    }

    #[test]
    fn test_clear_filter() {
        let mut service = FilterService::new();
        let messages = vec![
            create_test_can_message(1, 0x123),
            create_test_can_message(2, 0x456),
        ];

        let filter = FilterCriteria::new().with_id(0x123);
        service.apply_filter(&messages, &filter);
        assert!(service.is_filter_active());

        service.clear_filter();
        assert!(!service.is_filter_active());
        assert_eq!(service.filtered_count(), 0);
    }

    #[test]
    fn test_no_filter() {
        let mut service = FilterService::new();
        let messages = vec![
            create_test_can_message(1, 0x123),
            create_test_can_message(2, 0x456),
        ];

        let filter = FilterCriteria::new();
        let indices = service.apply_filter(&messages, &filter);

        // No active filter should return all messages
        assert_eq!(indices.len(), 2);
    }
}
