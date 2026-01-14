//! FlexRay status event object definitions.

use crate::BlfParseResult;
use crate::objects::object_header::ObjectHeader;
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Cursor, Read, Write};

/// Represents a FlexRay sync frame (`FLEXRAY_SYNC`, deprecated).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlexRaySync {
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
    pub data_bytes: [u8; 11],
    /// Cycle number.
    pub cycle: u8,
    /// Timestamp of the message.
    pub timestamp: u64,
}

impl FlexRaySync {
    pub(crate) fn read(cursor: &mut Cursor<&[u8]>, header: &ObjectHeader) -> BlfParseResult<Self> {
        let channel = cursor.read_u16::<LittleEndian>()?;
        let mux = cursor.read_u8()?;
        let len = cursor.read_u8()?;
        let message_id = cursor.read_u16::<LittleEndian>()?;
        let crc = cursor.read_u16::<LittleEndian>()?;
        let dir = cursor.read_u8()?;
        let _reserved1 = cursor.read_u8()?;
        let _reserved2 = cursor.read_u16::<LittleEndian>()?;
        let mut data_bytes = [0u8; 11];
        cursor.read_exact(&mut data_bytes)?;
        let cycle = cursor.read_u8()?;

        Ok(Self {
            channel,
            mux,
            len,
            message_id,
            crc,
            dir,
            data_bytes,
            cycle,
            timestamp: header.object_time_stamp,
        })
    }

    pub(crate) fn write<W: Write>(&self, _writer: &mut W) -> BlfParseResult<()> {
        unimplemented!("Writing FlexRaySync is not yet implemented.")
    }
}

/// Represents a FlexRay V6 Start Cycle event (`FLEXRAY_CYCLE`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlexRayV6StartCycleEvent {
    /// Application channel.
    pub channel: u16,
    /// Direction flags.
    pub dir: u8,
    /// Relative cluster time.
    pub cluster_time: u32,
    /// Data bytes (NM vector).
    pub data_bytes: [u8; 2],
    /// Timestamp of the message.
    pub timestamp: u64,
}

impl FlexRayV6StartCycleEvent {
    pub(crate) fn read(cursor: &mut Cursor<&[u8]>, header: &ObjectHeader) -> BlfParseResult<Self> {
        let channel = cursor.read_u16::<LittleEndian>()?;
        let dir = cursor.read_u8()?;
        let _low_time = cursor.read_u8()?;
        let _fpga_tick = cursor.read_u32::<LittleEndian>()?;
        let _fpga_tick_overflow = cursor.read_u32::<LittleEndian>()?;
        let _client_index = cursor.read_u32::<LittleEndian>()?;
        let cluster_time = cursor.read_u32::<LittleEndian>()?;
        let mut data_bytes = [0u8; 2];
        cursor.read_exact(&mut data_bytes)?;
        let _reserved = cursor.read_u16::<LittleEndian>()?;

        Ok(Self {
            channel,
            dir,
            cluster_time,
            data_bytes,
            timestamp: header.object_time_stamp,
        })
    }

    pub(crate) fn write<W: Write>(&self, _writer: &mut W) -> BlfParseResult<()> {
        unimplemented!("Writing FlexRayV6StartCycleEvent is not yet implemented.")
    }
}

/// Represents a FlexRay Status event (`FLEXRAY_STATUS`, deprecated).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlexRayStatusEvent {
    /// Application channel.
    pub channel: u16,
    /// Object version.
    pub version: u16,
    /// Type of status event.
    pub status_type: u16,
    /// Additional info mask 1.
    pub info_mask1: u16,
    /// Additional info mask 2.
    pub info_mask2: u16,
    /// Additional info mask 3.
    pub info_mask3: u16,
    /// Timestamp of the message.
    pub timestamp: u64,
}

impl FlexRayStatusEvent {
    pub(crate) fn read(cursor: &mut Cursor<&[u8]>, header: &ObjectHeader) -> BlfParseResult<Self> {
        let channel = cursor.read_u16::<LittleEndian>()?;
        let version = cursor.read_u16::<LittleEndian>()?;
        let status_type = cursor.read_u16::<LittleEndian>()?;
        let info_mask1 = cursor.read_u16::<LittleEndian>()?;
        let info_mask2 = cursor.read_u16::<LittleEndian>()?;
        let info_mask3 = cursor.read_u16::<LittleEndian>()?;
        let _reserved = [0u16; 18]; // Skip reserved array
        cursor.set_position(cursor.position() + (18 * 2) as u64); // Advance cursor by 18 * sizeof(u16)

        Ok(Self {
            channel,
            version,
            status_type,
            info_mask1,
            info_mask2,
            info_mask3,
            timestamp: header.object_time_stamp,
        })
    }

    pub(crate) fn write<W: Write>(&self, _writer: &mut W) -> BlfParseResult<()> {
        unimplemented!("Writing FlexRayStatusEvent is not yet implemented.")
    }
}

/// Represents a FlexRay Error event (`FR_ERROR`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlexRayVFrError {
    /// Application channel.
    pub channel: u16,
    /// Object version.
    pub version: u16,
    /// Channel mask.
    pub channel_mask: u16,
    /// Cycle number.
    pub cycle: u8,
    /// Client index of send node.
    pub client_index: u32,
    /// Number of cluster.
    pub cluster_no: u32,
    /// Type of communication controller.
    pub tag: u32,
    /// Driver flags for internal usage.
    pub data: [u32; 4],
    /// Timestamp of the message.
    pub timestamp: u64,
}

impl FlexRayVFrError {
    pub(crate) fn read(cursor: &mut Cursor<&[u8]>, header: &ObjectHeader) -> BlfParseResult<Self> {
        let channel = cursor.read_u16::<LittleEndian>()?;
        let version = cursor.read_u16::<LittleEndian>()?;
        let channel_mask = cursor.read_u16::<LittleEndian>()?;
        let cycle = cursor.read_u8()?;
        let _reserved1 = cursor.read_u8()?;
        let client_index = cursor.read_u32::<LittleEndian>()?;
        let cluster_no = cursor.read_u32::<LittleEndian>()?;
        let tag = cursor.read_u32::<LittleEndian>()?;
        let mut data = [0u32; 4];
        for i in 0..4 {
            data[i] = cursor.read_u32::<LittleEndian>()?;
        }
        let _reserved2 = cursor.read_u32::<LittleEndian>()?;

        Ok(Self {
            channel,
            version,
            channel_mask,
            cycle,
            client_index,
            cluster_no,
            tag,
            data,
            timestamp: header.object_time_stamp,
        })
    }

    pub(crate) fn write<W: Write>(&self, _writer: &mut W) -> BlfParseResult<()> {
        unimplemented!("Writing FlexRayVFrError is not yet implemented.")
    }
}

/// Represents a FlexRay Status event (`FR_STATUS`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlexRayVFrStatus {
    /// Application channel.
    pub channel: u16,
    /// Object version.
    pub version: u16,
    /// Channel mask.
    pub channel_mask: u16,
    /// Cycle number.
    pub cycle: u8,
    /// Client index of send node.
    pub client_index: u32,
    /// Number of cluster.
    pub cluster_no: u32,
    /// Wakeup state.
    pub wus: u32,
    /// Sync-State of CC.
    pub cc_sync_state: u32,
    /// Type of communication controller.
    pub tag: u32,
    /// Driver flags for internal usage.
    pub data: [u32; 2],
    /// Timestamp of the message.
    pub timestamp: u64,
}

impl FlexRayVFrStatus {
    pub(crate) fn read(cursor: &mut Cursor<&[u8]>, header: &ObjectHeader) -> BlfParseResult<Self> {
        let channel = cursor.read_u16::<LittleEndian>()?;
        let version = cursor.read_u16::<LittleEndian>()?;
        let channel_mask = cursor.read_u16::<LittleEndian>()?;
        let cycle = cursor.read_u8()?;
        let _reserved1 = cursor.read_u8()?;
        let client_index = cursor.read_u32::<LittleEndian>()?;
        let cluster_no = cursor.read_u32::<LittleEndian>()?;
        let wus = cursor.read_u32::<LittleEndian>()?;
        let cc_sync_state = cursor.read_u32::<LittleEndian>()?;
        let tag = cursor.read_u32::<LittleEndian>()?;
        let mut data = [0u32; 2];
        for i in 0..2 {
            data[i] = cursor.read_u32::<LittleEndian>()?;
        }
        let _reserved2 = [0u16; 18]; // Skip reserved array
        cursor.set_position(cursor.position() + (18 * 2) as u64); // Advance cursor by 18 * sizeof(u16)

        Ok(Self {
            channel,
            version,
            channel_mask,
            cycle,
            client_index,
            cluster_no,
            wus,
            cc_sync_state,
            tag,
            data,
            timestamp: header.object_time_stamp,
        })
    }

    pub(crate) fn write<W: Write>(&self, _writer: &mut W) -> BlfParseResult<()> {
        unimplemented!("Writing FlexRayVFrStatus is not yet implemented.")
    }
}

/// Represents a FlexRay Start Cycle event (`FR_STARTCYCLE`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlexRayVFrStartCycle {
    /// Application channel.
    pub channel: u16,
    /// Version of data struct.
    pub version: u16,
    /// Channel mask.
    pub channel_mask: u16,
    /// Direction flags.
    pub dir: u8,
    /// Cycle number.
    pub cycle: u8,
    /// Client index of send node.
    pub client_index: u32,
    /// Number of cluster.
    pub cluster_no: u32,
    /// Length of NM-Vector in bytes.
    pub nm_size: u16,
    /// Array of databytes (NM vector max. length).
    pub data_bytes: [u8; 12],
    /// Type of communication controller.
    pub tag: u32,
    /// Driver flags for internal usage.
    pub data: [u32; 5],
    /// Timestamp of the message.
    pub timestamp: u64,
}

impl FlexRayVFrStartCycle {
    pub(crate) fn read(cursor: &mut Cursor<&[u8]>, header: &ObjectHeader) -> BlfParseResult<Self> {
        let channel = cursor.read_u16::<LittleEndian>()?;
        let version = cursor.read_u16::<LittleEndian>()?;
        let channel_mask = cursor.read_u16::<LittleEndian>()?;
        let dir = cursor.read_u8()?;
        let cycle = cursor.read_u8()?;
        let client_index = cursor.read_u32::<LittleEndian>()?;
        let cluster_no = cursor.read_u32::<LittleEndian>()?;
        let nm_size = cursor.read_u16::<LittleEndian>()?;
        let mut data_bytes = [0u8; 12];
        cursor.read_exact(&mut data_bytes)?;
        let _reserved1 = cursor.read_u16::<LittleEndian>()?;
        let tag = cursor.read_u32::<LittleEndian>()?;
        let mut data = [0u32; 5];
        for i in 0..5 {
            data[i] = cursor.read_u32::<LittleEndian>()?;
        }
        let _reserved2 = cursor.read_u64::<LittleEndian>()?;

        Ok(Self {
            channel,
            version,
            channel_mask,
            dir,
            cycle,
            client_index,
            cluster_no,
            nm_size,
            data_bytes,
            tag,
            data,
            timestamp: header.object_time_stamp,
        })
    }

    pub(crate) fn write<W: Write>(&self, _writer: &mut W) -> BlfParseResult<()> {
        unimplemented!("Writing FlexRayVFrStartCycle is not yet implemented.")
    }
}
