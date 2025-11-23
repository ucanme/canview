//! Contains definitions for system-level event objects.

use crate::{BlfParseResult, ObjectHeader};
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Cursor;

/// Represents a `DATA_LOST_BEGIN` object.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DataLostBegin {
    /// The object header.
    pub header: ObjectHeader,
    /// Identifier for the leaking queue.
    pub queue_identifier: u32,
}

impl DataLostBegin {
    /// Reads a `DataLostBegin` from a byte cursor.
    pub fn read(cursor: &mut Cursor<&[u8]>, header: &ObjectHeader) -> BlfParseResult<Self> {
        let queue_identifier = cursor.read_u32::<LittleEndian>()?;
        Ok(Self {
            header: header.clone(),
            queue_identifier,
        })
    }
}

/// Represents a `DATA_LOST_END` object.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DataLostEnd {
    /// The object header.
    pub header: ObjectHeader,
    /// Identifier for the leaking queue.
    pub queue_identifier: u32,
    /// Timestamp of the first object lost.
    pub first_object_lost_time_stamp: u64,
    /// Number of lost events.
    pub number_of_lost_events: u32,
}

impl DataLostEnd {
    /// Reads a `DataLostEnd` from a byte cursor.
    pub fn read(cursor: &mut Cursor<&[u8]>, header: &ObjectHeader) -> BlfParseResult<Self> {
        let queue_identifier = cursor.read_u32::<LittleEndian>()?;
        let first_object_lost_time_stamp = cursor.read_u64::<LittleEndian>()?;
        let number_of_lost_events = cursor.read_u32::<LittleEndian>()?;
        cursor.set_position(cursor.position() + 4); // Skip reserved bytes
        Ok(Self {
            header: header.clone(),
            queue_identifier,
            first_object_lost_time_stamp,
            number_of_lost_events,
        })
    }
}