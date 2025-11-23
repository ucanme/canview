//! Application trigger object definition.

use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Cursor, Write};
use crate::{BlfParseResult, ObjectHeader};

/// Represents an application-defined trigger (`APP_TRIGGER`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppTrigger {
    /// Pre-trigger time.
    pub pre_trigger_time: u64,
    /// Post-trigger time.
    pub post_trigger_time: u64,
    /// Channel of the event which triggered.
    pub channel: u16,
    /// Trigger type flags.
    pub flags: u16,
    /// Timestamp of the message.
    pub timestamp: u64,
}

impl AppTrigger {
    pub(crate) fn read(cursor: &mut Cursor<&[u8]>, header: &ObjectHeader) -> BlfParseResult<Self> {
        let pre_trigger_time = cursor.read_u64::<LittleEndian>()?;
        let post_trigger_time = cursor.read_u64::<LittleEndian>()?;
        let channel = cursor.read_u16::<LittleEndian>()?;
        let flags = cursor.read_u16::<LittleEndian>()?;
        Ok(Self {
            pre_trigger_time,
            post_trigger_time,
            channel,
            flags,
            timestamp: header.object_time_stamp,
        })
    }

    pub(crate) fn write<W: Write>(&self, _writer: &mut W) -> BlfParseResult<()> {
        unimplemented!("Writing AppTrigger is not yet implemented.")
    }
}
