//! LIN error and status object definitions.

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Cursor, Write};
use crate::{BlfParseResult, ObjectHeader};

/// Represents a LIN CRC error (`LIN_CRC_ERROR`, deprecated).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinCrcError {
    /// Channel number.
    pub channel: u16,
    /// Frame identifier.
    pub id: u8,
    /// Frame length.
    pub dlc: u8,
    /// Data bytes.
    pub data: [u8; 8],
    /// Checksum byte value.
    pub crc: u16,
    /// Direction of bus event.
    pub dir: u8,
    /// Timestamp of the message.
    pub timestamp: u64,
}

impl LinCrcError {
    pub(crate) fn read(cursor: &mut Cursor<&[u8]>, header: &ObjectHeader) -> BlfParseResult<Self> {
        let channel = cursor.read_u16::<LittleEndian>()?;
        let id = cursor.read_u8()?;
        let dlc = cursor.read_u8()?;
        let mut data = [0u8; 8];
        cursor.read_exact(&mut data)?;
        let _fsm_id = cursor.read_u8()?;
        let _fsm_state = cursor.read_u8()?;
        let _header_time = cursor.read_u8()?;
        let _full_time = cursor.read_u8()?;
        let crc = cursor.read_u16::<LittleEndian>()?;
        let dir = cursor.read_u8()?;

        Ok(Self {
            channel,
            id,
            dlc,
            data,
            crc,
            dir,
            timestamp: header.object_time_stamp,
        })
    }

    pub(crate) fn write<W: Write>(&self, _writer: &mut W) -> BlfParseResult<()> {
        unimplemented!("Writing LinCrcError is not yet implemented.")
    }
}

/// Represents a LIN receive error (`LIN_RCV_ERROR`, deprecated).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinReceiveError {
    /// Channel number.
    pub channel: u16,
    /// Frame identifier.
    pub id: u8,
    /// Frame length.
    pub dlc: u8,
    /// State and reason for the error.
    pub state_reason: u8,
    /// Byte value that resulted in the protocol violation.
    pub offending_byte: u8,
    /// Detail level of the error (0: short, 1: full).
    pub short_error: u8,
    /// Flag indicating if timeout occurred during DLC detection.
    pub timeout_during_dlc_detection: u8,
    /// Timestamp of the message.
    pub timestamp: u64,
}

impl LinReceiveError {
    pub(crate) fn read(cursor: &mut Cursor<&[u8]>, header: &ObjectHeader) -> BlfParseResult<Self> {
        let channel = cursor.read_u16::<LittleEndian>()?;
        let id = cursor.read_u8()?;
        let dlc = cursor.read_u8()?;
        let _fsm_id = cursor.read_u8()?;
        let _fsm_state = cursor.read_u8()?;
        let _header_time = cursor.read_u8()?;
        let _full_time = cursor.read_u8()?;
        let state_reason = cursor.read_u8()?;
        let offending_byte = cursor.read_u8()?;
        let short_error = cursor.read_u8()?;
        let timeout_during_dlc_detection = cursor.read_u8()?;
        let _reserved = cursor.read_u32::<LittleEndian>()?;
        Ok(Self {
            channel,
            id,
            dlc,
            state_reason,
            offending_byte,
            short_error,
            timeout_during_dlc_detection,
            timestamp: header.object_time_stamp,
        })
    }

    pub(crate) fn write<W: Write>(&self, _writer: &mut W) -> BlfParseResult<()> {
        unimplemented!("Writing LinReceiveError is not yet implemented.")
    }
}

/// Represents a LIN send error (`LIN_SND_ERROR`, deprecated).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinSendError {
    /// Channel number.
    pub channel: u16,
    /// Frame identifier.
    pub id: u8,
    /// Frame length.
    pub dlc: u8,
    /// Timestamp of the message.
    pub timestamp: u64,
}

impl LinSendError {
    pub(crate) fn read(cursor: &mut Cursor<&[u8]>, header: &ObjectHeader) -> BlfParseResult<Self> {
        let channel = cursor.read_u16::<LittleEndian>()?;
        let id = cursor.read_u8()?;
        let dlc = cursor.read_u8()?;
        let _fsm_id = cursor.read_u8()?;
        let _fsm_state = cursor.read_u8()?;
        let _header_time = cursor.read_u8()?;
        let _full_time = cursor.read_u8()?;
        Ok(Self {
            channel,
            id,
            dlc,
            timestamp: header.object_time_stamp,
        })
    }

    pub(crate) fn write<W: Write>(&self, _writer: &mut W) -> BlfParseResult<()> {
        unimplemented!("Writing LinSendError is not yet implemented.")
    }
}

/// Represents a LIN slave timeout (`LIN_SLV_TIMEOUT`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinSlaveTimeout {
    /// Channel number.
    pub channel: u16,
    /// Slave identifier.
    pub slave_id: u8,
    /// Source state identifier.
    pub state_id: u8,
    /// Target state identifier.
    pub follow_state_id: u32,
    /// Timestamp of the message.
    pub timestamp: u64,
}

impl LinSlaveTimeout {
    pub(crate) fn read(cursor: &mut Cursor<&[u8]>, header: &ObjectHeader) -> BlfParseResult<Self> {
        let channel = cursor.read_u16::<LittleEndian>()?;
        let slave_id = cursor.read_u8()?;
        let state_id = cursor.read_u8()?;
        let follow_state_id = cursor.read_u32::<LittleEndian>()?;
        Ok(Self {
            channel,
            slave_id,
            state_id,
            follow_state_id,
            timestamp: header.object_time_stamp,
        })
    }

    pub(crate) fn write<W: Write>(&self, _writer: &mut W) -> BlfParseResult<()> {
        unimplemented!("Writing LinSlaveTimeout is not yet implemented.")
    }
}

/// Represents a LIN synchronization error (`LIN_SYN_ERROR`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinSyncError {
    /// Channel number.
    pub channel: u16,
    /// Time intervals detected between falling signal edges of the Sync field.
    pub time_diff: [u16; 4],
    /// Timestamp of the message.
    pub timestamp: u64,
}

impl LinSyncError {
    pub(crate) fn read(cursor: &mut Cursor<&[u8]>, header: &ObjectHeader) -> BlfParseResult<Self> {
        let channel = cursor.read_u16::<LittleEndian>()?;
        let _reserved1 = cursor.read_u16::<LittleEndian>()?;
        let mut time_diff = [0u16; 4];
        for i in 0..4 {
            time_diff[i] = cursor.read_u16::<LittleEndian>()?;
        }
        let _reserved2 = cursor.read_u32::<LittleEndian>()?;
        Ok(Self {
            channel,
            time_diff,
            timestamp: header.object_time_stamp,
        })
    }

    pub(crate) fn write<W: Write>(&self, _writer: &mut W) -> BlfParseResult<()> {
        unimplemented!("Writing LinSyncError is not yet implemented.")
    }
}
