//! CAN error and status object definitions.

use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Cursor;
use crate::{BlfParseResult};
use crate::objects::object_header::ObjectHeader;

/// Represents a CAN error frame (`CAN_ERROR`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CanErrorFrame {
    /// The object header.
    pub header: ObjectHeader,
    /// Channel the frame was sent or received on.
    pub channel: u16,
    /// Length of the error frame.
    pub length: u16,
}

impl CanErrorFrame {
    pub(crate) fn read(cursor: &mut Cursor<&[u8]>, header: &ObjectHeader) -> BlfParseResult<Self> {
        let channel = cursor.read_u16::<LittleEndian>()?;
        let length = cursor.read_u16::<LittleEndian>()?;

        // Based on C++ CanErrorFrame.cpp, skip reserved bytes if length > 0
        if length > 0 {
            cursor.set_position(cursor.position() + 4); // Skip reserved u32
        }

        Ok(Self {
            header: header.clone(),
            channel,
            length,
        })
    }
}

/// Represents a CAN overload frame (`CAN_OVERLOAD`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CanOverloadFrame {
    /// The object header.
    pub header: ObjectHeader,
    /// Channel the frame was sent or received on.
    pub channel: u16,
}

impl CanOverloadFrame {
    pub(crate) fn read(cursor: &mut Cursor<&[u8]>, header: &ObjectHeader) -> BlfParseResult<Self> {
        let channel = cursor.read_u16::<LittleEndian>()?;

        // Based on C++ CanOverloadFrame.cpp, skip reserved bytes
        cursor.set_position(cursor.position() + 2 + 4); // Skip reserved u16 and u32

        Ok(Self {
            header: header.clone(),
            channel,
        })
    }
}

/// Represents CAN driver statistics (`CAN_STATISTIC`).
#[derive(Debug, Clone, PartialEq)]
pub struct CanDriverStatistic {
    /// The object header.
    pub header: ObjectHeader,
    /// CAN channel the statistic data belongs to.
    pub channel: u16,
    /// Busload in 1/100 percent.
    pub bus_load: u16,
    /// Number of standard data frames.
    pub standard_data_frames: u32,
    /// Number of extended data frames.
    pub extended_data_frames: u32,
    /// Number of standard remote frames.
    pub standard_remote_frames: u32,
    /// Number of extended remote frames.
    pub extended_remote_frames: u32,
    /// Number of error frames.
    pub error_frames: u32,
    /// Number of overload frames.
    pub overload_frames: u32,
}

impl CanDriverStatistic {
    pub(crate) fn read(cursor: &mut Cursor<&[u8]>, header: &ObjectHeader) -> BlfParseResult<Self> {
        let channel = cursor.read_u16::<LittleEndian>()?;
        let bus_load = cursor.read_u16::<LittleEndian>()?;
        let standard_data_frames = cursor.read_u32::<LittleEndian>()?;
        let extended_data_frames = cursor.read_u32::<LittleEndian>()?;
        let standard_remote_frames = cursor.read_u32::<LittleEndian>()?;
        let extended_remote_frames = cursor.read_u32::<LittleEndian>()?;
        let error_frames = cursor.read_u32::<LittleEndian>()?;
        let overload_frames = cursor.read_u32::<LittleEndian>()?;

        // Based on C++ CanDriverStatistic.cpp, skip reserved bytes
        cursor.set_position(cursor.position() + 4); // Skip reserved u32

        Ok(Self {
            header: header.clone(),
            channel,
            bus_load,
            standard_data_frames,
            extended_data_frames,
            standard_remote_frames,
            extended_remote_frames,
            error_frames,
            overload_frames,
        })
    }
}

/// Represents a CAN driver error (`CAN_DRIVER_ERROR`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CanDriverError {
    /// The object header.
    pub header: ObjectHeader,
    /// CAN channel the error belongs to.
    pub channel: u16,
    /// Number of transmit errors.
    pub tx_errors: u8,
    /// Number of receive errors.
    pub rx_errors: u8,
    /// Error code.
    pub error_code: u32,
}

impl CanDriverError {
    pub(crate) fn read(cursor: &mut Cursor<&[u8]>, header: &ObjectHeader) -> BlfParseResult<Self> {
        let channel = cursor.read_u16::<LittleEndian>()?;
        let tx_errors = cursor.read_u8()?;
        let rx_errors = cursor.read_u8()?;
        let error_code = cursor.read_u32::<LittleEndian>()?;
        Ok(Self {
            header: header.clone(),
            channel,
            tx_errors,
            rx_errors,
            error_code,
        })
    }
}
