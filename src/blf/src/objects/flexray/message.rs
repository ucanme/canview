//! FlexRay message object definitions.

use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Cursor, Read, Write};
use crate::{BlfParseResult, ObjectHeader};

/// Represents a FlexRay data frame (`FLEXRAY_DATA`, deprecated).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlexRayData {
    /// Channel number.
    pub channel: u16,
    /// Multiplexer.
    pub mux: u8,
    /// Length.
    pub len: u8,
    /// Message ID.
    pub message_id: u16,
    /// CRC.
    pub crc: u16,
    /// Direction.
    pub dir: u8,
    /// Data bytes.
    pub data_bytes: [u8; 12],
    /// Timestamp of the message.
    pub timestamp: u64,
}

impl FlexRayData {
    pub(crate) fn read(cursor: &mut Cursor<&[u8]>, header: &ObjectHeader) -> BlfParseResult<Self> {
        let channel = cursor.read_u16::<LittleEndian>()?;
        let mux = cursor.read_u8()?;
        let len = cursor.read_u8()?;
        let message_id = cursor.read_u16::<LittleEndian>()?;
        let crc = cursor.read_u16::<LittleEndian>()?;
        let dir = cursor.read_u8()?;
        let _reserved1 = cursor.read_u8()?;
        let _reserved2 = cursor.read_u16::<LittleEndian>()?;
        let mut data_bytes = [0u8; 12];
        cursor.read_exact(&mut data_bytes)?;

        Ok(Self {
            channel,
            mux,
            len,
            message_id,
            crc,
            dir,
            data_bytes,
            timestamp: header.object_time_stamp,
        })
    }

    pub(crate) fn write<W: Write>(&self, _writer: &mut W) -> BlfParseResult<()> {
        unimplemented!("Writing FlexRayData is not yet implemented.")
    }
}

/// Represents a FlexRay V6 message (`FLEXRAY_MESSAGE`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlexRayV6Message {
    /// Channel number.
    pub channel: u16,
    /// Direction flag.
    pub dir: u8,
    /// Slot identifier.
    pub frame_id: u16,
    /// Payload length.
    pub length: u8,
    /// Current cycle number.
    pub cycle: u8,
    /// Payload data.
    pub data_bytes: [u8; 64],
    /// Timestamp of the message.
    pub timestamp: u64,
}

impl FlexRayV6Message {
    pub(crate) fn read(cursor: &mut Cursor<&[u8]>, header: &ObjectHeader) -> BlfParseResult<Self> {
        let channel = cursor.read_u16::<LittleEndian>()?;
        let dir = cursor.read_u8()?;
        let _low_time = cursor.read_u8()?;
        let _fpga_tick = cursor.read_u32::<LittleEndian>()?;
        let _fpga_tick_overflow = cursor.read_u32::<LittleEndian>()?;
        let _client_index = cursor.read_u32::<LittleEndian>()?;
        let _cluster_time = cursor.read_u32::<LittleEndian>()?;
        let frame_id = cursor.read_u16::<LittleEndian>()?;
        let _header_crc = cursor.read_u16::<LittleEndian>()?;
        let _frame_state = cursor.read_u16::<LittleEndian>()?;
        let length = cursor.read_u8()?;
        let cycle = cursor.read_u8()?;
        let _header_bit_mask = cursor.read_u8()?;
        let _reserved1 = cursor.read_u8()?;
        let _reserved2 = cursor.read_u16::<LittleEndian>()?;
        let mut data_bytes = [0u8; 64];
        cursor.read_exact(&mut data_bytes)?;

        Ok(Self {
            channel,
            dir,
            frame_id,
            length,
            cycle,
            data_bytes,
            timestamp: header.object_time_stamp,
        })
    }

    pub(crate) fn write<W: Write>(&self, _writer: &mut W) -> BlfParseResult<()> {
        unimplemented!("Writing FlexRayV6Message is not yet implemented.")
    }
}

/// Represents a FlexRay message received or transmitted (`FR_RCVMESSAGE`).
#[derive(Debug, Clone, PartialEq)]
pub struct FlexRayVFrReceiveMsg {
    /// Application channel.
    pub channel: u16,
    /// Version of data struct.
    pub version: u16,
    /// Channel mask.
    pub channel_mask: u8,
    /// Direction flags.
    pub dir: u8,
    /// Client index of send node.
    pub client_index: u32,
    /// Number of cluster.
    pub cluster_no: u32,
    /// Slot identifier.
    pub frame_id: u16,
    /// Header CRC FlexRay channel 1 (A).
    pub header_crc1: u16,
    /// Header CRC FlexRay channel 2 (B).
    pub header_crc2: u16,
    /// Payload length in bytes.
    pub byte_count: u16,
    /// Number of bytes of the payload stored in dataBytes.
    pub data_count: u16,
    /// Cycle number.
    pub cycle: u8,
    /// Type of communication controller.
    pub tag: u32,
    /// Driver flags for internal usage.
    pub data: u32,
    /// Frame flags.
    pub frame_flags: u32,
    /// Not used, reserved.
    pub app_parameter: u32,
    /// Payload.
    pub data_bytes: [u8; 254],
    /// Timestamp of the message.
    pub timestamp: u64,
}

impl FlexRayVFrReceiveMsg {
    pub(crate) fn read(cursor: &mut Cursor<&[u8]>, header: &ObjectHeader) -> BlfParseResult<Self> {
        let channel = cursor.read_u16::<LittleEndian>()?;
        let version = cursor.read_u16::<LittleEndian>()?;
        let channel_mask = cursor.read_u8()?;
        let dir = cursor.read_u8()?;
        let _reserved1 = cursor.read_u16::<LittleEndian>()?; // reservedFlexRayVFrReceiveMsg1
        let client_index = cursor.read_u32::<LittleEndian>()?;
        let cluster_no = cursor.read_u32::<LittleEndian>()?;
        let frame_id = cursor.read_u16::<LittleEndian>()?;
        let header_crc1 = cursor.read_u16::<LittleEndian>()?;
        let header_crc2 = cursor.read_u16::<LittleEndian>()?;
        let byte_count = cursor.read_u16::<LittleEndian>()?;
        let data_count = cursor.read_u16::<LittleEndian>()?;
        let cycle = cursor.read_u8()?;
        let _reserved2 = cursor.read_u8()?;
        let tag = cursor.read_u32::<LittleEndian>()?;
        let data = cursor.read_u32::<LittleEndian>()?;
        let frame_flags = cursor.read_u32::<LittleEndian>()?;
        let app_parameter = cursor.read_u32::<LittleEndian>()?;
        let mut data_bytes = [0u8; 254];
        cursor.read_exact(&mut data_bytes)?;
        let _reserved3 = cursor.read_u16::<LittleEndian>()?;
        let _reserved4 = cursor.read_u32::<LittleEndian>()?;

        Ok(Self {
            channel,
            version,
            channel_mask,
            dir,
            client_index,
            cluster_no,
            frame_id,
            header_crc1,
            header_crc2,
            byte_count,
            data_count,
            cycle,
            tag,
            data,
            frame_flags,
            app_parameter,
            data_bytes,
            timestamp: header.object_time_stamp,
        })
    }

    pub(crate) fn write<W: Write>(&self, _writer: &mut W) -> BlfParseResult<()> {
        unimplemented!("Writing FlexRayVFrReceiveMsg is not yet implemented.")
    }
}

/// Represents an extended FlexRay message or PDU received or transmitted (`FR_RCVMESSAGE_EX`).
#[derive(Debug, Clone, PartialEq)]
pub struct FlexRayVFrReceiveMsgEx {
    /// Application channel.
    pub channel: u16,
    /// Version of data struct.
    pub version: u16,
    /// Channel mask.
    pub channel_mask: u16,
    /// Direction flags.
    pub dir: u16,
    /// Client index of send node.
    pub client_index: u32,
    /// Number of cluster.
    pub cluster_no: u32,
    /// Slot identifier.
    pub frame_id: u16,
    /// Header CRC FlexRay channel 1 (A).
    pub header_crc1: u16,
    /// Header CRC FlexRay channel 2 (B).
    pub header_crc2: u16,
    /// Payload length in bytes.
    pub byte_count: u16,
    /// Number of bytes of the payload stored in dataBytes.
    pub data_count: u16,
    /// Cycle number.
    pub cycle: u16,
    /// Type of communication controller.
    pub tag: u32,
    /// Controller specific frame state information.
    pub data: u32,
    /// Frame flags.
    pub frame_flags: u32,
    /// Not used, reserved.
    pub app_parameter: u32,
    /// Frame CRC.
    pub frame_crc: u32,
    /// Length of frame in ns.
    pub frame_length_ns: u32,
    /// For PDUs only: This is the slot ID of the frame which contains this PDU.
    pub frame_id1: u16,
    /// For PDUs only: offset in bytes of PDU in an owner (raw) frame.
    pub pdu_offset: u16,
    /// Only valid for frames. Bitmask indicating which PDUs must be extracted.
    pub blf_log_mask: u16,
    /// Payload.
    pub data_bytes: Vec<u8>,
    /// Timestamp of the message.
    pub timestamp: u64,
}

impl FlexRayVFrReceiveMsgEx {
    pub(crate) fn read(cursor: &mut Cursor<&[u8]>, header: &ObjectHeader) -> BlfParseResult<Self> {
        let channel = cursor.read_u16::<LittleEndian>()?;
        let version = cursor.read_u16::<LittleEndian>()?;
        let channel_mask = cursor.read_u16::<LittleEndian>()?;
        let dir = cursor.read_u16::<LittleEndian>()?;
        let client_index = cursor.read_u32::<LittleEndian>()?;
        let cluster_no = cursor.read_u32::<LittleEndian>()?;
        let frame_id = cursor.read_u16::<LittleEndian>()?;
        let header_crc1 = cursor.read_u16::<LittleEndian>()?;
        let header_crc2 = cursor.read_u16::<LittleEndian>()?;
        let byte_count = cursor.read_u16::<LittleEndian>()?;
        let data_count = cursor.read_u16::<LittleEndian>()?;
        let cycle = cursor.read_u16::<LittleEndian>()?;
        let tag = cursor.read_u32::<LittleEndian>()?;
        let data = cursor.read_u32::<LittleEndian>()?;
        let frame_flags = cursor.read_u32::<LittleEndian>()?;
        let app_parameter = cursor.read_u32::<LittleEndian>()?;
        let frame_crc = cursor.read_u32::<LittleEndian>()?;
        let frame_length_ns = cursor.read_u32::<LittleEndian>()?;
        let frame_id1 = cursor.read_u16::<LittleEndian>()?;
        let pdu_offset = cursor.read_u16::<LittleEndian>()?;
        let blf_log_mask = cursor.read_u16::<LittleEndian>()?;
        cursor.set_position(cursor.position() + (13 * 2)); // Skip reservedFlexRayVFrReceiveMsgEx1

        let data_bytes_len = data_count as usize;
        let mut data_bytes = vec![0; data_bytes_len];
        cursor.read_exact(&mut data_bytes)?;

        // The C++ code reads a dynamically sized reserved block at the end.
        // We must calculate its size and skip it to ensure the cursor is correctly positioned.
        let fixed_part_size = 2 + 2 + 2 + 2 + 4 + 4 + 2 + 2 + 2 + 2 + 2 + 2 + 4 + 4 + 4 + 4 + 4 + 4 + 2 + 2 + 2 + (13 * 2);
        let remaining_size = (header.object_size as usize - header.calculate_header_size() as usize)
            .saturating_sub(fixed_part_size + data_bytes_len);
        cursor.set_position(cursor.position() + remaining_size as u64);

        Ok(Self {
            channel,
            version,
            channel_mask,
            dir,
            client_index,
            cluster_no,
            frame_id,
            header_crc1,
            header_crc2,
            byte_count,
            data_count,
            cycle,
            tag,
            data,
            frame_flags,
            app_parameter,
            frame_crc,
            frame_length_ns,
            frame_id1,
            pdu_offset,
            blf_log_mask,
            data_bytes,
            timestamp: header.object_time_stamp,
        })
    }

    pub(crate) fn write<W: Write>(&self, _writer: &mut W) -> BlfParseResult<()> {
        unimplemented!("Writing FlexRayVFrReceiveMsgEx is not yet implemented.")
    }
}
