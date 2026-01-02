//! CAN and CAN-FD message object definitions.

use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Cursor, Read};
use crate::{BlfParseResult, ObjectHeader};

/// Represents a standard CAN message (`CAN_MESSAGE`).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CanMessage {
    /// The object header.
    pub header: ObjectHeader,
    /// The channel number.
    pub channel: u16,
    /// Message flags.
    pub flags: u8,
    /// Data Length Code (DLC).
    pub dlc: u8,
    /// The CAN message ID.
    pub id: u32,
    /// The message data payload (up to 8 bytes).
    pub data: [u8; 8],
}

impl CanMessage {
    /// Reads a `CanMessage` from a byte cursor.
    pub fn read(cursor: &mut Cursor<&[u8]>, header: &ObjectHeader) -> BlfParseResult<Self> {
        let channel = cursor.read_u16::<LittleEndian>()?;
        let flags = cursor.read_u8()?;
        let dlc = cursor.read_u8()?;
        let id = cursor.read_u32::<LittleEndian>()?;
        let mut data = [0u8; 8];
        cursor.read_exact(&mut data)?;

        Ok(Self {
            header: header.clone(),
            channel,
            flags,
            dlc,
            id,
            data,
        })
    }
}

/// Represents an extended CAN message (`CAN_MESSAGE2`).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CanMessage2 {
    /// The object header.
    pub header: ObjectHeader,
    /// The channel number.
    pub channel: u16,
    /// Message flags.
    pub flags: u8,
    /// Data Length Code (DLC).
    pub dlc: u8,
    /// The CAN message ID.
    pub id: u32,
    /// The message data payload.
    pub data: Vec<u8>,
    /// Message duration in ns.
    pub frame_length: u32,
    /// Total number of bits of the message including EOF and Interframe space.
    pub bit_count: u8,
    /// Reserved field.
    pub reserved1: u8,
    /// Reserved field.
    pub reserved2: u16,
}

impl CanMessage2 {
    /// Reads a `CanMessage2` from a byte cursor.
    pub fn read(
        cursor: &mut Cursor<&[u8]>,
        header: &ObjectHeader,
        data_size: usize,
    ) -> BlfParseResult<Self> {
        let channel = cursor.read_u16::<LittleEndian>()?;
        let flags = cursor.read_u8()?;
        let dlc = cursor.read_u8()?;
        let id = cursor.read_u32::<LittleEndian>()?;
        // Read data based on the provided data_size
        let mut data = vec![0u8; data_size];
        cursor.read_exact(&mut data)?;
        let frame_length = cursor.read_u32::<LittleEndian>()?;
        let bit_count = cursor.read_u8()?;
        let reserved1 = cursor.read_u8()?;
        let reserved2 = cursor.read_u16::<LittleEndian>()?;

        Ok(Self {
            header: header.clone(),
            channel,
            flags,
            dlc,
            id,
            data,
            frame_length,
            bit_count,
            reserved1,
            reserved2,
        })
    }
}

/// Represents a CAN FD message (`CAN_FD_MESSAGE`).
#[derive(Debug, Clone, PartialEq)]
pub struct CanFdMessage {
    /// The object header.
    pub header: ObjectHeader,
    /// Channel number.
    pub channel: u16,
    /// CAN FD flags.
    pub can_fd_flags: u8,
    /// Valid payload length of the data.
    pub valid_payload_length: u8,
    /// Arbitration bit count.
    pub arb_bit_count: u8,
    /// Serial bit count.
    pub serial_bit_count: u8,
    /// The CAN FD message ID.
    pub id: u32,
    /// Message data.
    pub data: [u8; 64],
    /// Message duration in nanoseconds.
    pub frame_length: u32,
    /// Total number of bits in the message including EOF and interframe space.
    pub bit_count: u8,
    /// Direction of bus event (0=Rx, 1=Tx, 2=TxRq).
    pub dir: u8,
    /// EDL, BRS, ESI bits.
    pub edl_brs_esi: u8,
    /// Reserved field.
    pub reserved1: u8,
    /// Reserved field.
    pub reserved2: u32,
}

impl CanFdMessage {
    /// Reads a `CanFdMessage` from a byte cursor.
    pub fn read(cursor: &mut Cursor<&[u8]>, header: &ObjectHeader) -> BlfParseResult<Self> {
        let channel = cursor.read_u16::<LittleEndian>()?;
        let can_fd_flags = cursor.read_u8()?;
        let valid_payload_length = cursor.read_u8()?;
        let arb_bit_count = cursor.read_u8()?;
        let serial_bit_count = cursor.read_u8()?;
        let id = cursor.read_u32::<LittleEndian>()?;
        let mut data = [0u8; 64];
        cursor.read_exact(&mut data)?;
        let frame_length = cursor.read_u32::<LittleEndian>()?;
        let bit_count = cursor.read_u8()?;
        let dir = cursor.read_u8()?;
        let edl_brs_esi = cursor.read_u8()?;
        let reserved1 = cursor.read_u8()?;
        let reserved2 = cursor.read_u32::<LittleEndian>()?;

        Ok(Self {
            header: header.clone(),
            channel,
            can_fd_flags,
            valid_payload_length,
            arb_bit_count,
            serial_bit_count,
            id,
            data,
            frame_length,
            bit_count,
            dir,
            edl_brs_esi,
            reserved1,
            reserved2,
        })
    }
}

/// Represents a 64-byte CAN FD message (`CAN_FD_MESSAGE_64`).
#[derive(Debug, Clone, PartialEq)]
pub struct CanFdMessage64 {
    /// The object header.
    pub header: ObjectHeader,
    /// Channel number.
    pub channel: u16,
    /// CAN FD flags.
    pub can_fd_flags: u8,
    /// Valid payload length of the data.
    pub valid_payload_length: u8,
    /// Arbitration bit count.
    pub arb_bit_count: u8,
    /// Serial bit count.
    pub serial_bit_count: u8,
    /// The CAN FD message ID.
    pub id: u32,
    /// Message data.
    pub data: [u8; 64],
    /// Message duration in nanoseconds.
    pub frame_length: u32,
    /// Total number of bits in the message including EOF and interframe space.
    pub bit_count: u8,
    /// Direction of bus event (0=Rx, 1=Tx, 2=TxRq).
    pub dir: u8,
    /// EDL, BRS, ESI bits.
    pub edl_brs_esi: u8,
    /// Reserved field.
    pub reserved1: u8,
    /// Reserved field.
    pub reserved2: u32,
}

impl CanFdMessage64 {
    /// Reads a `CanFdMessage64` from a byte cursor.
    pub fn read(cursor: &mut Cursor<&[u8]>, header: &ObjectHeader) -> BlfParseResult<Self> {
        let channel = cursor.read_u16::<LittleEndian>()?;
        let can_fd_flags = cursor.read_u8()?;
        let valid_payload_length = cursor.read_u8()?;
        let arb_bit_count = cursor.read_u8()?;
        let serial_bit_count = cursor.read_u8()?;
        let id = cursor.read_u32::<LittleEndian>()?;
        let mut data = [0u8; 64];
        cursor.read_exact(&mut data)?;
        let frame_length = cursor.read_u32::<LittleEndian>()?;
        let bit_count = cursor.read_u8()?;
        let dir = cursor.read_u8()?;
        let edl_brs_esi = cursor.read_u8()?;
        let reserved1 = cursor.read_u8()?;
        let reserved2 = cursor.read_u32::<LittleEndian>()?;

        Ok(Self {
            header: header.clone(),
            channel,
            can_fd_flags,
            valid_payload_length,
            arb_bit_count,
            serial_bit_count,
            id,
            data,
            frame_length,
            bit_count,
            dir,
            edl_brs_esi,
            reserved1,
            reserved2,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;
    use crate::ObjectHeader;

    #[test]
    fn test_can_message_read() {
        use crate::ObjectType;
        
        let original_msg = CanMessage {
            header: ObjectHeader {
                signature: 0x4A424F4C, // "LOBJ"
                header_size: 32,
                header_version: 1,
                object_size: 48, // 32 (header) + 16 (body)
                object_type: ObjectType::CanMessage,
                object_flags: 0,
                object_time_stamp: 1000,
                original_time_stamp: None,
                time_stamp_status: None,
            },
            channel: 1,
            flags: 2,
            dlc: 8,
            id: 0x123,
            data: [1, 2, 3, 4, 5, 6, 7, 8],
        };

        let data = serialize_can_message(&original_msg);
        let mut cursor = Cursor::new(&data[..]);
        let header = ObjectHeader::read(&mut cursor).unwrap();
        let parsed_msg = CanMessage::read(&mut cursor, &header).unwrap();

        assert_eq!(original_msg, parsed_msg);
    }

    #[test]
    fn test_can_message2_read() {
        use crate::ObjectType;
        
        let data_payload = vec![10, 20, 30, 40];
        let original_msg = CanMessage2 {
            header: ObjectHeader {
                signature: 0x4A424F4C, // "LOBJ"
                header_size: 32,
                header_version: 1,
                object_size: 56, // 32 (header) + 24 (body)
                object_type: ObjectType::CanMessage2,
                object_flags: 0,
                object_time_stamp: 1000,
                original_time_stamp: None,
                time_stamp_status: None,
            },
            channel: 2,
            flags: 1,
            dlc: 4,
            id: 0x456,
            data: data_payload,
            frame_length: 12345,
            bit_count: 64,
            reserved1: 0,
            reserved2: 0,
        };

        let data = serialize_can_message2(&original_msg);
        let mut cursor = Cursor::new(&data[..]);
        let header = ObjectHeader::read(&mut cursor).unwrap();
        let parsed_msg = CanMessage2::read(&mut cursor, &header, 4).unwrap(); // Pass correct data size

        assert_eq!(original_msg, parsed_msg);
    }

    #[test]
    fn test_can_fd_message_read() {
        use crate::ObjectType;
        
        let original_msg = CanFdMessage {
            header: ObjectHeader {
                signature: 0x4A424F4C, // "LOBJ"
                header_size: 32,
                header_version: 1,
                object_size: 132, // header + body
                object_type: ObjectType::CanFdMessage,
                object_flags: 0,
                object_time_stamp: 1000,
                original_time_stamp: None,
                time_stamp_status: None,
            },
            channel: 3,
            can_fd_flags: 1, // BRS
            valid_payload_length: 64,
            arb_bit_count: 32,
            serial_bit_count: 16,
            id: 0x789,
            data: [5; 64],
            frame_length: 54321,
            bit_count: 64,
            dir: 1,
            edl_brs_esi: 1,
            reserved1: 0,
            reserved2: 0,
        };

        let data = serialize_can_fd_message(&original_msg);
        let mut cursor = Cursor::new(&data[..]);
        let header = ObjectHeader::read(&mut cursor).unwrap();
        let parsed_msg = CanFdMessage::read(&mut cursor, &header).unwrap();

        assert_eq!(original_msg, parsed_msg);
    }

    #[test]
    fn test_can_fd_message64_read() {
        use crate::ObjectType;
        
        let original_msg = CanFdMessage64 {
            header: ObjectHeader {
                signature: 0x4A424F4C, // "LOBJ"
                header_size: 32,
                header_version: 1,
                object_size: 132, // header + body
                object_type: ObjectType::CanFdMessage64,
                object_flags: 0,
                object_time_stamp: 1000,
                original_time_stamp: None,
                time_stamp_status: None,
            },
            channel: 4,
            can_fd_flags: 1, // EDL
            valid_payload_length: 20,
            arb_bit_count: 32,
            serial_bit_count: 16,
            id: 0xABC,
            data: [0; 64],
            frame_length: 9876,
            bit_count: 250,
            dir: 1,
            edl_brs_esi: 1,
            reserved1: 0,
            reserved2: 0,
        };

        let data = serialize_can_fd_message64(&original_msg);
        let mut cursor = Cursor::new(&data[..]);
        let header = ObjectHeader::read(&mut cursor).unwrap();
        let parsed_msg = CanFdMessage64::read(&mut cursor, &header).unwrap();

        assert_eq!(original_msg, parsed_msg);
    }

    #[test]
    fn test_can_fd_message64_read_no_data() {
        use crate::ObjectType;
        
        let original_msg = CanFdMessage64 {
            header: ObjectHeader {
                signature: 0x4A424F4C, // "LOBJ"
                header_size: 32,
                header_version: 1,
                object_size: 132, // header + body
                object_type: ObjectType::CanFdMessage64,
                object_flags: 0,
                object_time_stamp: 1000,
                original_time_stamp: None,
                time_stamp_status: None,
            },
            channel: 0,
            can_fd_flags: 0,
            valid_payload_length: 0,
            arb_bit_count: 0,
            serial_bit_count: 0,
            id: 0,
            data: [0; 64],
            frame_length: 0,
            bit_count: 0,
            dir: 0,
            edl_brs_esi: 0,
            reserved1: 0,
            reserved2: 0,
        };

        let data = serialize_can_fd_message64(&original_msg);
        let mut cursor = Cursor::new(&data[..]);
        let header = ObjectHeader::read(&mut cursor).unwrap();
        let parsed_msg = CanFdMessage64::read(&mut cursor, &header).unwrap();

        assert_eq!(original_msg, parsed_msg);
    }
}
