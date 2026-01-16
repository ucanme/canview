//! Message filtering functionality

use blf::LogObject;

/// Filter messages by ID
pub fn filter_by_id(messages: &[LogObject], filter_id: u32) -> Vec<LogObject> {
    messages
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

/// Filter messages by channel
pub fn filter_by_channel(messages: &[LogObject], filter_ch: u16) -> Vec<LogObject> {
    messages
        .iter()
        .filter(|msg| msg.channel() == filter_ch)
        .cloned()
        .collect()
}

/// Filter messages by both ID and channel
pub fn filter_by_id_and_channel(
    messages: &[LogObject],
    filter_id: u32,
    filter_ch: u16,
) -> Vec<LogObject> {
    messages
        .iter()
        .filter(|msg| {
            let matches_id = match msg {
                LogObject::CanMessage(can_msg) => can_msg.id == filter_id,
                LogObject::CanMessage2(can_msg) => can_msg.id == filter_id,
                LogObject::CanFdMessage(fd_msg) => fd_msg.id == filter_id,
                LogObject::CanFdMessage64(fd_msg) => fd_msg.id == filter_id,
                LogObject::LinMessage(lin_msg) => lin_msg.id as u32 == filter_id,
                LogObject::LinMessage2(_) => false,
                _ => false,
            };
            matches_id && msg.channel() == filter_ch
        })
        .cloned()
        .collect()
}

/// Get unique message IDs from messages
pub fn get_unique_ids(messages: &[LogObject]) -> Vec<u32> {
    use std::collections::HashSet;

    let mut ids = HashSet::new();
    for msg in messages {
        match msg {
            LogObject::CanMessage(can_msg) => { ids.insert(can_msg.id); }
            LogObject::CanMessage2(can_msg) => { ids.insert(can_msg.id); }
            LogObject::CanFdMessage(fd_msg) => { ids.insert(fd_msg.id); }
            LogObject::CanFdMessage64(fd_msg) => { ids.insert(fd_msg.id); }
            LogObject::LinMessage(lin_msg) => { ids.insert(lin_msg.id as u32); }
            _ => {}
        }
    }

    let mut sorted_ids: Vec<_> = ids.into_iter().collect();
    sorted_ids.sort_unstable();
    sorted_ids
}

/// Get unique channels from messages
pub fn get_unique_channels(messages: &[LogObject]) -> Vec<u16> {
    use std::collections::HashSet;

    let mut channels = HashSet::new();
    for msg in messages {
        channels.insert(msg.channel());
    }

    let mut sorted_channels: Vec<_> = channels.into_iter().collect();
    sorted_channels.sort_unstable();
    sorted_channels
}

/// Format ID as decimal or hexadecimal
pub fn format_id(id: u32, decimal: bool) -> String {
    if decimal {
        id.to_string()
    } else {
        format!("{:X}", id)
    }
}
