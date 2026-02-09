//! Helper methods for CanViewApp impl blocks
//!
//! This module contains private helper methods that break down large functions
//! into smaller, more manageable pieces.

use super::state::CanViewApp;
use blf::LogObject;

impl CanViewApp {
    /// Filter messages based on ID and channel filters
    ///
    /// This is a helper method extracted from render_log_view to reduce complexity.
    /// It applies both ID and channel filters to the message list.
    fn filter_messages(&self) -> Vec<LogObject> {
        match (self.id_filter, self.channel_filter) {
            (None, None) => self.messages.clone(),
            (Some(filter_id), None) => {
                // Only ID filter
                self.messages
                    .iter()
                    .filter(|msg| match msg {
                        LogObject::CanMessage(can_msg) => can_msg.id == filter_id,
                        LogObject::CanMessage2(can_msg) => can_msg.id == filter_id,
                        LogObject::CanFdMessage(fd_msg) => fd_msg.id == filter_id,
                        LogObject::CanFdMessage64(fd_msg) => fd_msg.id == filter_id,
                        LogObject::LinMessage(lin_msg) => lin_msg.id as u32 == filter_id,
                        LogObject::LinMessage2(_) => false,
                        _ => false,
                    })
                    .cloned()
                    .collect()
            }
            (None, Some(filter_ch)) => {
                // Only Channel filter
                self.messages
                    .iter()
                    .filter(|msg| match msg {
                        LogObject::CanMessage(can_msg) => can_msg.channel == filter_ch,
                        LogObject::CanMessage2(can_msg) => can_msg.channel == filter_ch,
                        LogObject::CanFdMessage(fd_msg) => fd_msg.channel == filter_ch,
                        LogObject::CanFdMessage64(fd_msg) => fd_msg.channel as u16 == filter_ch,
                        LogObject::LinMessage(lin_msg) => lin_msg.channel == filter_ch,
                        LogObject::LinMessage2(_) => false,
                        _ => false,
                    })
                    .cloned()
                    .collect()
            }
            (Some(filter_id), Some(filter_ch)) => {
                // Both filters
                self.messages
                    .iter()
                    .filter(|msg| match msg {
                        LogObject::CanMessage(can_msg) => {
                            can_msg.id == filter_id && can_msg.channel == filter_ch
                        }
                        LogObject::CanMessage2(can_msg) => {
                            can_msg.id == filter_id && can_msg.channel == filter_ch
                        }
                        LogObject::CanFdMessage(fd_msg) => {
                            fd_msg.id == filter_id && fd_msg.channel == filter_ch
                        }
                        LogObject::CanFdMessage64(fd_msg) => {
                            fd_msg.id == filter_id && fd_msg.channel as u16 == filter_ch
                        }
                        LogObject::LinMessage(lin_msg) => {
                            lin_msg.id as u32 == filter_id && lin_msg.channel == filter_ch
                        }
                        LogObject::LinMessage2(_) => false,
                        _ => false,
                    })
                    .cloned()
                    .collect()
            }
        }
    }
}
