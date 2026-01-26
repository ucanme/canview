//! CAN FD Message 64 object definitions.

use crate::BlfParseResult;
use crate::objects::object_header::ObjectHeader;
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Cursor, Read};

/// Optional extended data for CanFdMessage64
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CanFdExtFrameData {
    /// Bit rate in arbitration phase
    /// - Bit 0-7: TSEG1-1
    /// - Bit 8-15: TSEG2-1
    /// - Bit 16-27: Prescaler
    /// - Bit 28-31: Quartz Frequency (0=16MHz, 1=32MHz, 2=80MHz)
    pub btr_ext_arb: u32,

    /// Bit rate in data phase
    /// - Bit 0-7: TSEG1-1
    /// - Bit 8-15: TSEG2-1
    /// - Bit 16-27: Prescaler
    /// - Bit 28-31: Quartz Frequency (0=16MHz, 1=32MHz, 2=80MHz)
    pub btr_ext_data: u32,

    /// Reserved data
    pub reserved: Vec<u8>,
}

impl CanFdExtFrameData {
    /// Reads CanFdExtFrameData from a byte cursor
    pub fn read(cursor: &mut Cursor<&[u8]>, size: usize) -> BlfParseResult<Self> {
        let btr_ext_arb = cursor.read_u32::<LittleEndian>()?;
        let btr_ext_data = cursor.read_u32::<LittleEndian>()?;

        // Calculate remaining size for reserved data
        let header_size = 8; // 2 * u32
        let reserved_size = size.saturating_sub(header_size);

        let mut reserved = vec![0u8; reserved_size];
        if reserved_size > 0 {
            cursor.read_exact(&mut reserved)?;
        }

        Ok(Self {
            btr_ext_arb,
            btr_ext_data,
            reserved,
        })
    }

    /// Calculates the size of this structure
    pub fn calculate_size(&self) -> usize {
        8 + self.reserved.len()
    }
}

/// Represents a 64-byte CAN FD message (`CAN_FD_MESSAGE_64`).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CanFdMessage64 {
    /// The object header.
    pub header: ObjectHeader,

    /// Application channel
    pub channel: u8,

    /// Data Length Code (DLC)
    /// - DLC=0-8: CAN=0-8, CAN FD=0-8
    /// - DLC=9: CAN=8, CAN FD=12
    /// - DLC=10: CAN=8, CAN FD=16
    /// - DLC=11: CAN=8, CAN FD=20
    /// - DLC=12: CAN=8, CAN FD=24
    /// - DLC=13: CAN=8, CAN FD=32
    /// - DLC=14: CAN=8, CAN FD=48
    /// - DLC=15: CAN=8, CAN FD=64
    pub dlc: u8,

    /// Valid payload length of data
    pub valid_data_bytes: u8,

    /// txRequiredCount (4 bits), txReqCount (4 bits)
    /// (Bits 0-3) Number of required transmission attempts
    /// (Bits 4-7) Max Number of transmission attempts
    pub tx_count: u8,

    /// Frame identifier
    pub id: u32,

    /// Message duration in ns (without 3 interframe-space bit times and by Rx-messages also without 1 end-of-frame bit time)
    pub frame_length: u32,

    /// Message flags
    /// - Bit 0: Must be 0
    /// - Bit 1: Reserved, for internal use
    /// - Bit 2: 1=NERR (single wire on low speed CAN)
    /// - Bit 3: 1=High voltage wake up
    /// - Bit 4: 1=Remote frame (only CAN)
    /// - Bit 5: Reserved, must be 0
    /// - Bit 6: 1=Tx Acknowledge
    /// - Bit 7: 1=Tx Request
    /// - Bit 8: Reserved, must be 0
    /// - Bit 9: SRR (CAN FD)
    /// - Bit 10: R0
    /// - Bit 11: R1
    /// - Bit 12: EDL, 0=CAN frame, 1=CAN FD frame
    /// - Bit 13: BRS (CAN FD)
    /// - Bit 14: ESI
    /// - Bit 15: Reserved, must be 0
    /// - Bit 16: Reserved, must be 0
    /// - Bit 17: 1=Frame is part of a burst
    /// - Bit 18-31: Reserved, must be 0
    pub flags: u32,

    /// Bit rate used in arbitration phase
    /// - Bit 0-7: Quartz Frequency
    /// - Bit 8-15: Prescaler
    /// - Bit 16-23: BTL Cycles
    /// - Bit 24-31: Sampling Point
    pub btr_cfg_arb: u32,

    /// Bit rate used in data phase
    pub btr_cfg_data: u32,

    /// Time offset of BRS field in nanoseconds
    pub time_offset_brs_ns: u32,

    /// Time offset of CRC delimiter field in nanoseconds
    pub time_offset_crc_del_ns: u32,

    /// Bit count of the message
    pub bit_count: u16,

    /// Direction of the message (0=Rx, 1=Tx, 2=TxRq)
    pub dir: u8,

    /// Offset if extDataOffset is used
    pub ext_data_offset: u8,

    /// CRC for CAN
    pub crc: u32,

    /// CAN FD data bytes (actual length may be shorter than 64 bytes)
    pub data: Vec<u8>,

    /// Optional extended frame data
    pub ext_data: Option<CanFdExtFrameData>,
}

impl CanFdMessage64 {
    /// Reads a `CanFdMessage64` from a byte cursor.
    pub fn read(cursor: &mut Cursor<&[u8]>, header: &ObjectHeader) -> BlfParseResult<Self> {
        // Check if we need to skip bytes (similar to CanMessage)
        // Some BLF variants have extra metadata before the actual CAN FD data
        let remaining = &cursor.get_ref()[cursor.position() as usize..];
        let skip_bytes = if remaining.len() >= 32 {
            // Check both offset 0 and offset 16
            let channel_at_0 = remaining[0];
            let dlc_at_1 = remaining[1];
            let id_at_4 =
                u32::from_le_bytes([remaining[4], remaining[5], remaining[6], remaining[7]]);

            let channel_at_16 = remaining[16];
            let dlc_at_17 = remaining[17];
            let id_at_20 =
                u32::from_le_bytes([remaining[20], remaining[21], remaining[22], remaining[23]]);

            // Offset 0 looks invalid (all zeros or suspicious) AND offset 16 looks valid
            let offset_0_invalid = (channel_at_0 == 0 && dlc_at_1 == 0 && id_at_4 == 0)
                || (dlc_at_1 == 0 && id_at_4 == 0 && channel_at_0 <= 1);

            let offset_16_valid =
                (channel_at_16 > 0 || dlc_at_17 > 0 || id_at_20 > 0) && dlc_at_17 <= 15;

            if offset_0_invalid && offset_16_valid {
                16
            } else {
                0
            }
        } else {
            0
        };

        // Skip extra bytes if detected
        if skip_bytes > 0 {
            let mut temp = [0u8; 16];
            cursor.read_exact(&mut temp)?;
        }

        let channel = { cursor.read_u8()? };

        let dlc = cursor.read_u8()?;
        let valid_data_bytes = cursor.read_u8()?;
        let tx_count = cursor.read_u8()?;
        let id = cursor.read_u32::<LittleEndian>()?;
        let frame_length = cursor.read_u32::<LittleEndian>()?;
        let flags = cursor.read_u32::<LittleEndian>()?;
        let btr_cfg_arb = cursor.read_u32::<LittleEndian>()?;
        let btr_cfg_data = cursor.read_u32::<LittleEndian>()?;
        let time_offset_brs_ns = cursor.read_u32::<LittleEndian>()?;
        let time_offset_crc_del_ns = cursor.read_u32::<LittleEndian>()?;
        let bit_count = cursor.read_u16::<LittleEndian>()?;
        let dir = cursor.read_u8()?;
        let ext_data_offset = cursor.read_u8()?;
        let crc = cursor.read_u32::<LittleEndian>()?;

        // Read data - actual length is valid_data_bytes
        let mut data = vec![0u8; valid_data_bytes as usize];
        if valid_data_bytes > 0 {
            cursor.read_exact(&mut data)?;
        }

        // Check if we have extended data
        let ext_data = if ext_data_offset != 0
            && (header.object_size as usize) >= (ext_data_offset as usize + 8)
        {
            // Calculate the size of ext data based on remaining object size
            let total_object_size = header.object_size as usize;
            let ext_data_size = total_object_size.saturating_sub(ext_data_offset as usize);

            if ext_data_size > 0 {
                Some(CanFdExtFrameData::read(cursor, ext_data_size)?)
            } else {
                None
            }
        } else {
            None
        };

        Ok(Self {
            header: header.clone(),
            channel,
            dlc,
            valid_data_bytes,
            tx_count,
            id,
            frame_length,
            flags,
            btr_cfg_arb,
            btr_cfg_data,
            time_offset_brs_ns,
            time_offset_crc_del_ns,
            bit_count,
            dir,
            ext_data_offset,
            crc,
            data,
            ext_data,
        })
    }

    /// Check if this is a CAN FD frame (EDL bit set)
    pub fn is_fd_frame(&self) -> bool {
        (self.flags & 0x1000) != 0
    }

    /// Check if bit rate switch is enabled (BRS bit set)
    pub fn has_brs(&self) -> bool {
        (self.flags & 0x2000) != 0
    }

    /// Check if error state indicator is set (ESI bit set)
    pub fn has_esi(&self) -> bool {
        (self.flags & 0x4000) != 0
    }

    /// Check if this is a TX message
    pub fn is_tx(&self) -> bool {
        self.dir == 1
    }

    /// Check if this is a TX request
    pub fn is_tx_request(&self) -> bool {
        self.dir == 2
    }
}

/// Flags for CanFdMessage64
impl CanFdMessage64 {
    /// NERR flag (single wire on low speed CAN)
    pub const FLAG_NERR: u32 = 0x0004;
    /// High voltage wake up
    pub const FLAG_HIGH_VOLTAGE_WAKEUP: u32 = 0x0008;
    /// Remote frame (only CAN)
    pub const FLAG_REMOTE_FRAME: u32 = 0x0010;
    /// Tx Acknowledge
    pub const FLAG_TX_ACK: u32 = 0x0040;
    /// Tx Request
    pub const FLAG_TX_REQUEST: u32 = 0x0080;
    /// SRR (CAN FD)
    pub const FLAG_SRR: u32 = 0x0200;
    /// EDL - 0=CAN frame, 1=CAN FD frame
    pub const FLAG_EDL: u32 = 0x1000;
    /// BRS (CAN FD)
    pub const FLAG_BRS: u32 = 0x2000;
    /// ESI (Error State Indicator)
    pub const FLAG_ESI: u32 = 0x4000;
    /// Frame is part of a burst
    pub const FLAG_BURST: u32 = 0x20000;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ObjectHeader;
    use crate::ObjectType;

    #[test]
    fn test_can_fd_message64_read_basic() {
        let header = ObjectHeader {
            base: crate::objects::object_header::ObjectHeaderBase {
                signature: 0x4A424F4C,
                header_size: 32,
                header_version: 1,
                object_size: 120, // header (32) + body (88 with no data)
                object_type: ObjectType::CanFdMessage64,
            },
            object_flags: 0,
            client_index: 0,
            object_version: 0,
            object_time_stamp: 1000,
            original_time_stamp: None,
            time_stamp_status: None,
            reserved: 0,
        };

        // Create test data: channel=1, dlc=8, valid_bytes=8, tx_count=0, id=0x123
        // rest zeros
        let mut test_data = vec![0u8; 88];
        test_data[0] = 1; // channel
        test_data[1] = 8; // dlc
        test_data[2] = 8; // valid_data_bytes
        test_data[3] = 0; // tx_count
        test_data[4..8].copy_from_slice(&0x123u32.to_le_bytes()); // id

        // Add 8 bytes of data at position 40 (after all header fields)
        for i in 0..8 {
            test_data[40 + i] = i as u8;
        }

        let mut cursor = Cursor::new(&test_data[..]);
        let msg = CanFdMessage64::read(&mut cursor, &header).unwrap();

        assert_eq!(msg.channel, 1);
        assert_eq!(msg.dlc, 8);
        assert_eq!(msg.valid_data_bytes, 8);
        assert_eq!(msg.id, 0x123);
        assert_eq!(msg.data.len(), 8);
        assert_eq!(msg.data[0], 0);
        assert_eq!(msg.data[7], 7);
    }

    #[test]
    fn test_can_fd_message64_flags() {
        let header = ObjectHeader {
            base: crate::objects::object_header::ObjectHeaderBase {
                signature: 0x4A424F4C,
                header_size: 32,
                header_version: 1,
                object_size: 120,
                object_type: ObjectType::CanFdMessage64,
            },
            object_flags: 0,
            client_index: 0,
            object_version: 0,
            object_time_stamp: 1000,
            original_time_stamp: None,
            time_stamp_status: None,
            reserved: 0,
        };

        let mut test_data = vec![0u8; 88];
        test_data[0] = 1; // channel
        // flags at offset 12 (after channel(1) + dlc(1) + valid_bytes(1) + tx_count(1) + id(4) + frame_length(4))
        test_data[12..16].copy_from_slice(&0x1000u32.to_le_bytes()); // EDL bit set

        let mut cursor = Cursor::new(&test_data[..]);
        let msg = CanFdMessage64::read(&mut cursor, &header).unwrap();

        assert!(msg.is_fd_frame());
        assert!(!msg.has_brs());
        assert!(!msg.has_esi());
    }

    #[test]
    fn test_can_fd_message64_with_brs_esi() {
        let header = ObjectHeader {
            base: crate::objects::object_header::ObjectHeaderBase {
                signature: 0x4A424F4C,
                header_size: 32,
                header_version: 1,
                object_size: 120,
                object_type: ObjectType::CanFdMessage64,
            },
            object_flags: 0,
            client_index: 0,
            object_version: 0,
            object_time_stamp: 1000,
            original_time_stamp: None,
            time_stamp_status: None,
            reserved: 0,
        };

        let mut test_data = vec![0u8; 88];
        test_data[0] = 1; // channel
        // EDL + BRS + ESI bits set (offset 12)
        test_data[12..16].copy_from_slice(&0x7000u32.to_le_bytes());

        let mut cursor = Cursor::new(&test_data[..]);
        let msg = CanFdMessage64::read(&mut cursor, &header).unwrap();

        assert!(msg.is_fd_frame());
        assert!(msg.has_brs());
        assert!(msg.has_esi());
    }
}
