//! Ethernet frame object definitions.

use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Cursor, Read, Write};
use crate::{BlfParseResult};
use crate::objects::object_header::ObjectHeader;

/// Represents an Ethernet frame (`ETHERNET_FRAME`).
#[derive(Debug, Clone, PartialEq)]
pub struct EthernetFrame {
    /// Ethernet (MAC) address of source computer.
    pub source_address: [u8; 6],
    /// The channel of the frame.
    pub channel: u16,
    /// Ethernet (MAC) address of target computer.
    pub destination_address: [u8; 6],
    /// Direction flag (0=Rx, 1=Tx, 2=TxRq).
    pub dir: u16,
    /// EtherType which indicates the protocol for the payload.
    pub frame_type: u16,
    /// TPID when VLAN tag is valid, zero otherwise.
    pub tpid: u16,
    /// TCI when VLAN tag is valid, zero otherwise.
    pub tci: u16,
    /// Length of the Ethernet payload data in bytes.
    pub payload_length: u16,
    /// Ethernet payload data.
    pub payload: Vec<u8>,
    /// Timestamp of the message.
    pub timestamp: u64,
}

impl EthernetFrame {
    pub(crate) fn read(cursor: &mut Cursor<&[u8]>, header: &ObjectHeader) -> BlfParseResult<Self> {
        let mut source_address = [0u8; 6];
        cursor.read_exact(&mut source_address)?;
        let channel = cursor.read_u16::<LittleEndian>()?;
        let mut destination_address = [0u8; 6];
        cursor.read_exact(&mut destination_address)?;
        let dir = cursor.read_u16::<LittleEndian>()?;
        let frame_type = cursor.read_u16::<LittleEndian>()?;
        let tpid = cursor.read_u16::<LittleEndian>()?;
        let tci = cursor.read_u16::<LittleEndian>()?;
        let payload_length = cursor.read_u16::<LittleEndian>()?;
        let _reserved = cursor.read_u64::<LittleEndian>()?;

        let mut payload = vec![0; payload_length as usize];
        cursor.read_exact(&mut payload)?;

        Ok(Self {
            source_address,
            channel,
            destination_address,
            dir,
            frame_type,
            tpid,
            tci,
            payload_length,
            payload,
            timestamp: header.object_time_stamp,
        })
    }

    pub(crate) fn write<W: Write>(&self, _writer: &mut W) -> BlfParseResult<()> {
        unimplemented!("Writing EthernetFrame is not yet implemented.")
    }
}
