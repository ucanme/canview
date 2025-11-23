//! WLAN object definitions.

use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Cursor, Read};
use crate::{BlfParseResult, ObjectHeader};

/// Represents a WLAN frame object.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WlanFrame {
    /// The common object header.
    pub header: ObjectHeader,
    /// Channel number.
    pub channel: u16,
    /// Flags.
    pub flags: u8,
    /// Data rate in 0.1 Mb/s.
    pub data_rate: u8,
    /// Received signal strength in dBm.
    pub signal_strength: i32,
    /// Frame data.
    pub frame_data: Vec<u8>,
}

impl WlanFrame {
    /// Reads a `WlanFrame` from a byte stream.
    pub fn read(cursor: &mut Cursor<&[u8]>, header: &ObjectHeader) -> BlfParseResult<Self> {
        let channel = cursor.read_u16::<LittleEndian>()?;
        let flags = cursor.read_u8()?;
        let data_rate = cursor.read_u8()?;
        let signal_strength = cursor.read_i32::<LittleEndian>()?;
        
        let data_size = header.object_size as usize - header.header_size as usize - 8; // 8 bytes for the fields above
        let mut frame_data = vec![0u8; data_size];
        cursor.read_exact(&mut frame_data)?;

        Ok(Self {
            header: header.clone(),
            channel,
            flags,
            data_rate,
            signal_strength,
            frame_data,
        })
    }
}

/// Represents a WLAN statistic object.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WlanStatistic {
    /// The common object header.
    pub header: ObjectHeader,
    /// Channel number.
    pub channel: u16,
    /// Flags.
    pub flags: u8,
    /// RSSI value.
    pub rssi: u8,
    /// Transmission rate in 0.1 Mb/s.
    pub transmission_rate: u16,
    /// Transmission delay in us.
    pub transmission_delay: u32,
}

impl WlanStatistic {
    /// Reads a `WlanStatistic` from a byte stream.
    pub fn read(cursor: &mut Cursor<&[u8]>, header: &ObjectHeader) -> BlfParseResult<Self> {
        let channel = cursor.read_u16::<LittleEndian>()?;
        let flags = cursor.read_u8()?;
        let rssi = cursor.read_u8()?;
        let transmission_rate = cursor.read_u16::<LittleEndian>()?;
        let transmission_delay = cursor.read_u32::<LittleEndian>()?;

        Ok(Self {
            header: header.clone(),
            channel,
            flags,
            rssi,
            transmission_rate,
            transmission_delay,
        })
    }
}
