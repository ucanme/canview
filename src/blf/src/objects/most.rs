//! MOST (Media Oriented Systems Transport) object definitions.
//! Most object definitions.

use crate::BlfParseResult;
use crate::objects::object_header::ObjectHeader;
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Cursor, Read};

/// Represents a message from the MOST Control Channel in spy mode (`MOST_SPY`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MostSpy {
    /// Channel number.
    pub channel: u16,
    /// Direction of the message.
    pub dir: u8,
    /// Source address.
    pub source_adr: u32,
    /// Destination address.
    pub dest_adr: u32,
    /// The 17-byte message payload.
    pub msg: [u8; 17],
    /// Control message sub-type.
    pub r_typ: u16,
    /// Addressing mode.
    pub r_typ_adr: u8,
    /// Transmission state.
    pub state: u8,
    /// Acknowledge/Negative-acknowledge status.
    pub ack_nack: u8,
    /// Cyclic Redundancy Check.
    pub crc: u32,
    /// Timestamp of the message.
    pub timestamp: u64,
}

impl MostSpy {
    pub(crate) fn read(cursor: &mut Cursor<&[u8]>, header: &ObjectHeader) -> BlfParseResult<Self> {
        let channel = cursor.read_u16::<LittleEndian>()?;
        let dir = cursor.read_u8()?;
        let _reserved1 = cursor.read_u8()?;
        let source_adr = cursor.read_u32::<LittleEndian>()?;
        let dest_adr = cursor.read_u32::<LittleEndian>()?;
        let mut msg = [0u8; 17];
        cursor.read_exact(&mut msg)?;
        let _reserved2 = cursor.read_u8()?;
        let r_typ = cursor.read_u16::<LittleEndian>()?;
        let r_typ_adr = cursor.read_u8()?;
        let state = cursor.read_u8()?;
        let _reserved3 = cursor.read_u8()?;
        let ack_nack = cursor.read_u8()?;
        let crc = cursor.read_u32::<LittleEndian>()?;
        Ok(Self {
            channel,
            dir,
            source_adr,
            dest_adr,
            msg,
            r_typ,
            r_typ_adr,
            state,
            ack_nack,
            crc,
            timestamp: header.object_time_stamp,
        })
    }
}

/// Represents a message from the MOST Control Channel in node mode (`MOST_CTRL`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MostCtrl {
    /// Channel number.
    pub channel: u16,
    /// Direction of the message.
    pub dir: u8,
    /// Source address.
    pub source_adr: u32,
    /// Destination address.
    pub dest_adr: u32,
    /// The 17-byte message payload.
    pub msg: [u8; 17],
    /// Control message sub-type.
    pub r_typ: u16,
    /// Addressing mode.
    pub r_typ_adr: u8,
    /// Transmission state.
    pub state: u8,
    /// Acknowledge/Negative-acknowledge status.
    pub ack_nack: u8,
    /// Timestamp of the message.
    pub timestamp: u64,
}

impl MostCtrl {
    pub(crate) fn read(cursor: &mut Cursor<&[u8]>, header: &ObjectHeader) -> BlfParseResult<Self> {
        let channel = cursor.read_u16::<LittleEndian>()?;
        let dir = cursor.read_u8()?;
        let _reserved1 = cursor.read_u8()?;
        let source_adr = cursor.read_u32::<LittleEndian>()?;
        let dest_adr = cursor.read_u32::<LittleEndian>()?;
        let mut msg = [0u8; 17];
        cursor.read_exact(&mut msg)?;
        let _reserved2 = cursor.read_u8()?;
        let r_typ = cursor.read_u16::<LittleEndian>()?;
        let r_typ_adr = cursor.read_u8()?;
        let state = cursor.read_u8()?;
        let _reserved3 = cursor.read_u8()?;
        let ack_nack = cursor.read_u8()?;
        Ok(Self {
            channel,
            dir,
            source_adr,
            dest_adr,
            msg,
            r_typ,
            r_typ_adr,
            state,
            ack_nack,
            timestamp: header.object_time_stamp,
        })
    }
}

/// Represents a message on the MOST Packet Data Channel (`MOST_PKT2`).
#[derive(Debug, Clone, PartialEq)]
pub struct MostPkt2 {
    /// Channel number.
    pub channel: u16,
    /// Direction of the message.
    pub dir: u8,
    /// Source address.
    pub source_adr: u32,
    /// Destination address.
    pub dest_adr: u32,
    /// Arbitration byte.
    pub arbitration: u8,
    /// Number of quadlets to follow.
    pub quads_to_follow: u8,
    /// Cyclic Redundancy Check.
    pub crc: u16,
    /// Priority.
    pub priority: u8,
    /// Transfer type (Node or Spy).
    pub transfer_type: u8,
    /// Transmission state.
    pub state: u8,
    /// The variable-length packet data.
    pub pkt_data: Vec<u8>,
    /// Timestamp of the message.
    pub timestamp: u64,
    /// Original timestamp, if available.
    pub original_timestamp: Option<u64>,
}

impl MostPkt2 {
    pub(crate) fn read(cursor: &mut Cursor<&[u8]>, header: &ObjectHeader) -> BlfParseResult<Self> {
        let channel = cursor.read_u16::<LittleEndian>()?;
        let dir = cursor.read_u8()?;
        let _reserved1 = cursor.read_u8()?;
        let source_adr = cursor.read_u32::<LittleEndian>()?;
        let dest_adr = cursor.read_u32::<LittleEndian>()?;
        let arbitration = cursor.read_u8()?;
        let _time_res = cursor.read_u8()?;
        let quads_to_follow = cursor.read_u8()?;
        let _reserved2 = cursor.read_u8()?;
        let crc = cursor.read_u16::<LittleEndian>()?;
        let priority = cursor.read_u8()?;
        let transfer_type = cursor.read_u8()?;
        let state = cursor.read_u8()?;
        let _reserved3 = cursor.read_u8()?;
        let _reserved4 = cursor.read_u16::<LittleEndian>()?;
        let pkt_data_length = cursor.read_u32::<LittleEndian>()? as usize;
        let _reserved5 = cursor.read_u32::<LittleEndian>()?;
        let mut pkt_data = vec![0; pkt_data_length];
        cursor.read_exact(&mut pkt_data)?;

        Ok(Self {
            channel,
            dir,
            source_adr,
            dest_adr,
            arbitration,
            quads_to_follow,
            crc,
            priority,
            transfer_type,
            state,
            pkt_data,
            timestamp: header.object_time_stamp,
            original_timestamp: header.original_time_stamp,
        })
    }
}

/// Represents a MOST light lock event (`MOST_LIGHTLOCK`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MostLightLock {
    /// Application channel.
    pub channel: u16,
    /// Signal state.
    pub state: i16,
    /// Timestamp of the message.
    pub timestamp: u64,
}

impl MostLightLock {
    pub(crate) fn read(cursor: &mut Cursor<&[u8]>, header: &ObjectHeader) -> BlfParseResult<Self> {
        let channel = cursor.read_u16::<LittleEndian>()?;
        let state = cursor.read_i16::<LittleEndian>()?;
        let _reserved = cursor.read_u32::<LittleEndian>()?;
        Ok(Self {
            channel,
            state,
            timestamp: header.object_time_stamp,
        })
    }
}

/// Represents MOST network statistics (`MOST_STATISTIC`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MostStatistic {
    /// Application channel.
    pub channel: u16,
    /// Number of messages on Asynchronous channel.
    pub pkt_cnt: u16,
    /// Number of messages on Control channel.
    pub frm_cnt: i32,
    /// Number of signal stat transitions.
    pub light_cnt: i32,
    /// Receive buffer level.
    pub buffer_level: i32,
    /// Timestamp of the message.
    pub timestamp: u64,
}

impl MostStatistic {
    pub(crate) fn read(cursor: &mut Cursor<&[u8]>, header: &ObjectHeader) -> BlfParseResult<Self> {
        let channel = cursor.read_u16::<LittleEndian>()?;
        let pkt_cnt = cursor.read_u16::<LittleEndian>()?;
        let frm_cnt = cursor.read_i32::<LittleEndian>()?;
        let light_cnt = cursor.read_i32::<LittleEndian>()?;
        let buffer_level = cursor.read_i32::<LittleEndian>()?;
        Ok(Self {
            channel,
            pkt_cnt,
            frm_cnt,
            light_cnt,
            buffer_level,
            timestamp: header.object_time_stamp,
        })
    }
}

/// Represents a MOST hardware mode event (`MOST_HWMODE`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MostHwMode {
    /// Application channel.
    pub channel: u16,
    /// Hardware mode flags.
    pub hw_mode: u16,
    /// Bitmask of changed bits.
    pub hw_mode_mask: u16,
    /// Timestamp of the message.
    pub timestamp: u64,
    /// Original timestamp, if available.
    pub original_timestamp: Option<u64>,
}

impl MostHwMode {
    pub(crate) fn read(cursor: &mut Cursor<&[u8]>, header: &ObjectHeader) -> BlfParseResult<Self> {
        let channel = cursor.read_u16::<LittleEndian>()?;
        let _reserved = cursor.read_u16::<LittleEndian>()?;
        let hw_mode = cursor.read_u16::<LittleEndian>()?;
        let hw_mode_mask = cursor.read_u16::<LittleEndian>()?;
        Ok(Self {
            channel,
            hw_mode,
            hw_mode_mask,
            timestamp: header.object_time_stamp,
            original_timestamp: header.original_time_stamp,
        })
    }
}

/// Represents MOST register data (`MOST_REG`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MostReg {
    /// Application channel.
    pub channel: u16,
    /// Operation type of a register event.
    pub sub_type: u8,
    /// Operation handle.
    pub handle: u32,
    /// Register address offset.
    pub offset: u32,
    /// ID of chip.
    pub chip: u16,
    /// Number of valid bytes in reg_data.
    pub reg_data_len: u16,
    /// Register data.
    pub reg_data: [u8; 16],
    /// Timestamp of the message.
    pub timestamp: u64,
    /// Original timestamp, if available.
    pub original_timestamp: Option<u64>,
}

impl MostReg {
    pub(crate) fn read(cursor: &mut Cursor<&[u8]>, header: &ObjectHeader) -> BlfParseResult<Self> {
        let channel = cursor.read_u16::<LittleEndian>()?;
        let sub_type = cursor.read_u8()?;
        let _reserved1 = cursor.read_u8()?;
        let handle = cursor.read_u32::<LittleEndian>()?;
        let offset = cursor.read_u32::<LittleEndian>()?;
        let chip = cursor.read_u16::<LittleEndian>()?;
        let reg_data_len = cursor.read_u16::<LittleEndian>()?;
        let mut reg_data = [0u8; 16];
        cursor.read_exact(&mut reg_data)?;
        Ok(Self {
            channel,
            sub_type,
            handle,
            offset,
            chip,
            reg_data_len,
            reg_data,
            timestamp: header.object_time_stamp,
            original_timestamp: header.original_time_stamp,
        })
    }
}

/// Represents MOST general register data (`MOST_GENREG`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MostGenReg {
    /// Application channel.
    pub channel: u16,
    /// Operation type of a register event.
    pub sub_type: u8,
    /// Operation handle.
    pub handle: u32,
    /// Register ID.
    pub reg_id: u16,
    /// Register value.
    pub reg_value: u64,
    /// Timestamp of the message.
    pub timestamp: u64,
    /// Original timestamp, if available.
    pub original_timestamp: Option<u64>,
}

impl MostGenReg {
    pub(crate) fn read(cursor: &mut Cursor<&[u8]>, header: &ObjectHeader) -> BlfParseResult<Self> {
        let channel = cursor.read_u16::<LittleEndian>()?;
        let sub_type = cursor.read_u8()?;
        let _reserved1 = cursor.read_u8()?;
        let handle = cursor.read_u32::<LittleEndian>()?;
        let reg_id = cursor.read_u16::<LittleEndian>()?;
        let _reserved2 = cursor.read_u16::<LittleEndian>()?;
        let _reserved3 = cursor.read_u32::<LittleEndian>()?;
        let reg_value = cursor.read_u64::<LittleEndian>()?;
        Ok(Self {
            channel,
            sub_type,
            handle,
            reg_id,
            reg_value,
            timestamp: header.object_time_stamp,
            original_timestamp: header.original_time_stamp,
        })
    }
}

/// Represents a MOST network state event (`MOST_NETSTATE`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MostNetState {
    /// Application channel.
    pub channel: u16,
    /// Current network state.
    pub state_new: u16,
    /// Previous network state.
    pub state_old: u16,
    /// Timestamp of the message.
    pub timestamp: u64,
    /// Original timestamp, if available.
    pub original_timestamp: Option<u64>,
}

impl MostNetState {
    pub(crate) fn read(cursor: &mut Cursor<&[u8]>, header: &ObjectHeader) -> BlfParseResult<Self> {
        let channel = cursor.read_u16::<LittleEndian>()?;
        let state_new = cursor.read_u16::<LittleEndian>()?;
        let state_old = cursor.read_u16::<LittleEndian>()?;
        let _reserved = cursor.read_u16::<LittleEndian>()?;
        Ok(Self {
            channel,
            state_new,
            state_old,
            timestamp: header.object_time_stamp,
            original_timestamp: header.original_time_stamp,
        })
    }
}

/// Represents a MOST data lost event (`MOST_DATALOST`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MostDataLost {
    /// Application channel.
    pub channel: u16,
    /// Data loss information flags.
    pub info: u32,
    /// Number of lost messages on Control channel.
    pub lost_msgs_ctrl: u32,
    /// Number of lost messages on Packet Data Channel.
    pub lost_msgs_async: u32,
    /// Absolute time of last good message in nanoseconds.
    pub last_good_time_stamp_ns: u64,
    /// Absolute time of next good message in nanoseconds.
    pub next_good_time_stamp_ns: u64,
    /// Timestamp of the message.
    pub timestamp: u64,
    /// Original timestamp, if available.
    pub original_timestamp: Option<u64>,
}

impl MostDataLost {
    pub(crate) fn read(cursor: &mut Cursor<&[u8]>, header: &ObjectHeader) -> BlfParseResult<Self> {
        let channel = cursor.read_u16::<LittleEndian>()?;
        let _reserved1 = cursor.read_u16::<LittleEndian>()?;
        let info = cursor.read_u32::<LittleEndian>()?;
        let lost_msgs_ctrl = cursor.read_u32::<LittleEndian>()?;
        let lost_msgs_async = cursor.read_u32::<LittleEndian>()?;
        let last_good_time_stamp_ns = cursor.read_u64::<LittleEndian>()?;
        let next_good_time_stamp_ns = cursor.read_u64::<LittleEndian>()?;
        Ok(Self {
            channel,
            info,
            lost_msgs_ctrl,
            lost_msgs_async,
            last_good_time_stamp_ns,
            next_good_time_stamp_ns,
            timestamp: header.object_time_stamp,
            original_timestamp: header.original_time_stamp,
        })
    }
}

/// Represents a MOST trigger event (`MOST_TRIGGER`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MostTrigger {
    /// Application channel.
    pub channel: u16,
    /// Trigger mode.
    pub mode: u16,
    /// Hardware that generated the trigger event.
    pub hw: u16,
    /// Value of IO register before trigger.
    pub previous_trigger_value: u32,
    /// Value of IO register after trigger.
    pub current_trigger_value: u32,
    /// Timestamp of the message.
    pub timestamp: u64,
    /// Original timestamp, if available.
    pub original_timestamp: Option<u64>,
}

impl MostTrigger {
    pub(crate) fn read(cursor: &mut Cursor<&[u8]>, header: &ObjectHeader) -> BlfParseResult<Self> {
        let channel = cursor.read_u16::<LittleEndian>()?;
        let _reserved1 = cursor.read_u16::<LittleEndian>()?;
        let mode = cursor.read_u16::<LittleEndian>()?;
        let hw = cursor.read_u16::<LittleEndian>()?;
        let previous_trigger_value = cursor.read_u32::<LittleEndian>()?;
        let current_trigger_value = cursor.read_u32::<LittleEndian>()?;
        Ok(Self {
            channel,
            mode,
            hw,
            previous_trigger_value,
            current_trigger_value,
            timestamp: header.object_time_stamp,
            original_timestamp: header.original_time_stamp,
        })
    }
}
