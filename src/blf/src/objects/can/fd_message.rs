//! CAN FD message object definitions.

use crate::{BlfParseResult};
use crate::objects::object_header::ObjectHeader;
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Cursor, Read};

/// Represents a CAN FD message (`CAN_FD_MESSAGE`).
#[derive(Debug, Clone, PartialEq)]
pub struct CanFdMessage {
    /// The object header.
    pub header: ObjectHeader,
    /// Application channel
    pub channel: u16,
    /// CAN Message Flags (dir, rtr, wu & nerr)
    pub flags: u8,
    /// Data Length Code (DLC)
    pub dlc: u8,
    /// Frame identifier
    pub id: u32,
    /// Message duration in ns (without 3 inter frame space bits and by Rx-message also without 1 End-Of-Frame bit)
    pub frame_length: u32,
    /// Bit count of arbitration phase
    pub arb_bit_count: u8,
    /// CAN FD flags (EDL, BRS, ESI)
    pub can_fd_flags: u8,
    /// Valid payload length of data
    pub valid_data_bytes: u8,
    /// Reserved field
    pub reserved1: u8,
    /// Reserved field
    pub reserved2: u32,
    /// CAN FD data bytes
    pub data: [u8; 64],
    /// Reserved field
    pub reserved3: u32,
}

impl Default for CanFdMessage {
    fn default() -> Self {
        Self {
            header: ObjectHeader::default(),
            channel: 0,
            flags: 0,
            dlc: 0,
            id: 0,
            frame_length: 0,
            arb_bit_count: 0,
            can_fd_flags: 0,
            valid_data_bytes: 0,
            reserved1: 0,
            reserved2: 0,
            data: [0u8; 64],
            reserved3: 0,
        }
    }
}

impl CanFdMessage {
    /// Reads a `CanFdMessage` from a byte cursor.
    pub fn read(cursor: &mut Cursor<&[u8]>, header: &ObjectHeader) -> BlfParseResult<Self> {
        let channel = cursor.read_u16::<LittleEndian>()?;
        println!("-=--channel---{}", channel);
        let flags = cursor.read_u8()?;
        let dlc = cursor.read_u8()?;
        let id = cursor.read_u32::<LittleEndian>()?;
        let frame_length = cursor.read_u32::<LittleEndian>()?;
        let arb_bit_count = cursor.read_u8()?;
        let can_fd_flags = cursor.read_u8()?;
        let valid_data_bytes = cursor.read_u8()?;
        let reserved1 = cursor.read_u8()?;
        let reserved2 = cursor.read_u32::<LittleEndian>()?;
        let mut data = [0u8; 64];
        cursor.read_exact(&mut data)?;
        let reserved3 = cursor.read_u32::<LittleEndian>()?;

        Ok(Self {
            header: header.clone(),
            channel,
            flags,
            dlc,
            id,
            frame_length,
            arb_bit_count,
            can_fd_flags,
            valid_data_bytes,
            reserved1,
            reserved2,
            data,
            reserved3,
        })
    }
}

/// Flags for CanFdMessage
impl CanFdMessage {
    /// Transmit direction
    pub const FLAG_TX: u8 = 1 << 0;
    /// Single wire operation
    pub const FLAG_NERR: u8 = 1 << 5;
    /// Wake up message (high voltage)
    pub const FLAG_WU: u8 = 1 << 6;
    /// Remote transmission request
    pub const FLAG_RTR: u8 = 1 << 7;
}

/// CAN FD flags for CanFdMessage
impl CanFdMessage {
    /// Extended data length
    pub const FD_FLAG_EDL: u8 = 1 << 0;
    /// Bit rate switch
    pub const FD_FLAG_BRS: u8 = 1 << 1;
    /// Error state indicator
    pub const FD_FLAG_ESI: u8 = 1 << 2;
}
