//! CAN message object definitions (non-FD).

use crate::BlfParseResult;
use crate::objects::object_header::ObjectHeader;
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Cursor, Read};

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
        // Check if we need to skip 16 bytes (eckzeit and other metadata)
        // This happens in some BLF file variants where the format includes:
        // - 8 bytes: eckzeit (cycle time in ns)
        // - 8 bytes: reserved/unknown fields
        // before the actual CAN message data
        let remaining = &cursor.get_ref()[cursor.position() as usize..];
        let skip_bytes = if remaining.len() >= 24 {
            // Check both offset 0 and offset 16
            let channel_at_0 = u16::from_le_bytes([remaining[0], remaining[1]]);
            let dlc_at_3 = remaining[3];
            let id_at_4 =
                u32::from_le_bytes([remaining[4], remaining[5], remaining[6], remaining[7]]);

            let channel_at_16 = u16::from_le_bytes([remaining[16], remaining[17]]);
            let dlc_at_19 = remaining[19];
            let id_at_20 =
                u32::from_le_bytes([remaining[20], remaining[21], remaining[22], remaining[23]]);

            // Offset 0 looks invalid (all zeros or suspicious) AND offset 16 looks valid
            let offset_0_invalid = (channel_at_0 == 0 && dlc_at_3 == 0 && id_at_4 == 0)
                || (dlc_at_3 == 0 && id_at_4 == 0 && channel_at_0 <= 1);

            let offset_16_valid =
                (channel_at_16 > 0 || dlc_at_19 > 0 || id_at_20 > 0) && dlc_at_19 <= 8;

            if offset_0_invalid && offset_16_valid {
                16
            } else {
                0
            }
        } else {
            0
        };

        // Skip the extra bytes if detected
        if skip_bytes > 0 {
            let mut temp = [0u8; 16];
            cursor.read_exact(&mut temp)?;
        }

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

/// Flags for CanMessage and CanMessage2
impl CanMessage2 {
    /// Transmit direction (Bit 0)
    pub const FLAG_TX: u8 = 1 << 0;
    /// Single wire operation (Bit 5)
    pub const FLAG_NERR: u8 = 1 << 5;
    /// Wake up message (Bit 6)
    pub const FLAG_WU: u8 = 1 << 6;
    /// Remote transmission request (Bit 7)
    pub const FLAG_RTR: u8 = 1 << 7;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ObjectHeader;
    use crate::test_utils::*;

    #[test]
    fn test_can_message_read() {
        use crate::ObjectType;

        let original_msg = CanMessage {
            header: ObjectHeader {
                base: crate::objects::object_header::ObjectHeaderBase {
                    signature: 0x4A424F4C, // "LOBJ"
                    header_size: 32,
                    header_version: 1,
                    object_size: 48, // 32 (header) + 16 (body)
                    object_type: ObjectType::CanMessage,
                },
                object_flags: 0,
                client_index: 0,
                object_version: 0,
                object_time_stamp: 1000,
                original_time_stamp: None,
                time_stamp_status: None,
                reserved: 0,
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
                base: crate::objects::object_header::ObjectHeaderBase {
                    signature: 0x4A424F4C, // "LOBJ"
                    header_size: 32,
                    header_version: 1,
                    object_size: 56, // 32 (header) + 24 (body)
                    object_type: ObjectType::CanMessage2,
                },
                object_flags: 0,
                client_index: 0,
                object_version: 0,
                object_time_stamp: 1000,
                original_time_stamp: None,
                time_stamp_status: None,
                reserved: 0,
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
        let parsed_msg = CanMessage2::read(&mut cursor, &header, 4).unwrap();

        assert_eq!(original_msg, parsed_msg);
    }
}
