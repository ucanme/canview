//! LIN message object definitions.

use crate::{BlfParseResult};
use crate::objects::object_header::ObjectHeader;
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Cursor, Read};

/// Represents a LIN message (`LIN_MESSAGE`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinMessage {
    /// The object header.
    pub header: ObjectHeader,
    /// Channel number.
    pub channel: u16,
    /// Frame identifier.
    pub id: u8,
    /// Frame length.
    pub dlc: u8,
    /// Payload data.
    pub data: [u8; 8],
    /// FSM identifier.
    pub fsm_id: u16,
    /// FSM state.
    pub fsm_state: u16,
    /// Header time.
    pub header_time: u32,
    /// Full time.
    pub full_time: u32,
    /// Checksum.
    pub crc: u8,
    /// Direction.
    pub dir: u8,
}

impl LinMessage {
    /// Reads a `LinMessage` from a byte cursor.
    pub fn read(cursor: &mut Cursor<&[u8]>, header: &ObjectHeader) -> BlfParseResult<Self> {
        // Based on C++ LinMessage.cpp
        let channel = cursor.read_u16::<LittleEndian>()?;
        let id = cursor.read_u8()?;
        let dlc = cursor.read_u8()?;
        let mut data = [0u8; 8];
        cursor.read_exact(&mut data)?;
        let fsm_id = cursor.read_u16::<LittleEndian>()?;
        let fsm_state = cursor.read_u16::<LittleEndian>()?;
        let header_time = cursor.read_u32::<LittleEndian>()?;
        let full_time = cursor.read_u32::<LittleEndian>()?;
        let crc = cursor.read_u8()?;
        let dir = cursor.read_u8()?;
        cursor.set_position(cursor.position() + 2 + 4); // reservedLinMessage1 + reservedLinMessage2

        Ok(Self {
            header: header.clone(),
            channel,
            id,
            dlc,
            data,
            fsm_id,
            fsm_state,
            header_time,
            full_time,
            crc,
            dir,
        })
    }
}

/// Represents an extended LIN message (`LIN_MESSAGE2`).
#[derive(Debug, Clone, PartialEq)]
pub struct LinMessage2 {
    /// The object header.
    pub header: ObjectHeader,
    /// Data bytes.
    pub data: [u8; 8],
    /// Checksum byte value.
    pub crc: u16,
    /// Direction of bus event.
    pub dir: u8,
    /// Flag indicating if this is a simulated frame.
    pub simulated: u8,
    /// Flag indicating if this is an Event-Triggered Frame.
    pub is_etf: u8,
    // V2+ fields
    /// Response baudrate in bit/sec.
    pub resp_baudrate: Option<u32>,
    // V3+ fields
    /// Exact header baudrate in bit/sec.
    pub exact_header_baudrate: Option<f64>,
}
impl LinMessage2 {
    /// Reads a `LinMessage2` from a byte cursor.
    pub fn read(cursor: &mut Cursor<&[u8]>, header: &ObjectHeader, data_size: usize) -> BlfParseResult<Self> {
        // Based on C++ LinMessage2.cpp
        let mut data = [0u8; 8];
        cursor.read_exact(&mut data)?;
        let crc = cursor.read_u16::<LittleEndian>()?;
        let dir = cursor.read_u8()?;
        let simulated = cursor.read_u8()?;
        let is_etf = cursor.read_u8()?;
        cursor.set_position(cursor.position() + 1 + 1 + 1 + 1 + 2 + 4); // Skip etfAssocIndex, etfAssocEtfId, fsmId, fsmState, reserved1, reserved2

        let remaining_size = data_size.saturating_sub(8 + 2 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 2 + 4);

        let mut resp_baudrate = None;
        if remaining_size >= 4 {
            resp_baudrate = Some(cursor.read_u32::<LittleEndian>()?);
        }

        let mut exact_header_baudrate = None;
        if remaining_size >= 12 { // 4 for resp_baudrate + 8 for this f64
            exact_header_baudrate = Some(cursor.read_f64::<LittleEndian>()?);
        }

        Ok(Self {
            header: header.clone(),
            data,
            crc,
            dir,
            simulated,
            is_etf,
            resp_baudrate,
            exact_header_baudrate,
        })
    }
}
