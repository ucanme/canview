//! Core BLF structures and error handling.

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
use std::io::{self, Cursor, Read, Write};
use std::fmt;
use std::error::Error;

/// Represents a parsing error that can occur while processing a BLF file.
#[derive(Debug)]
pub enum BlfParseError {
    /// An I/O error occurred while reading the data.
    IoError(io::Error),
    /// The file does not start with the expected "LOGG" magic string.
    InvalidFileMagic,
    /// A log container does not start with the expected "LOBJ" magic string.
    InvalidContainerMagic,
    /// The data ended unexpectedly while parsing an object.
    UnexpectedEof,
    /// An unknown or unsupported compression method was specified in a LogContainer.
    UnsupportedCompression(u16),
    /// An unknown object header version was encountered.
    UnknownHeaderVersion(u16),
}

impl fmt::Display for BlfParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BlfParseError::IoError(e) => write!(f, "I/O error: {}", e),
            BlfParseError::InvalidFileMagic => write!(f, "Invalid BLF file magic string"),
            BlfParseError::InvalidContainerMagic => write!(f, "Invalid LOBJ container magic string"),
            BlfParseError::UnexpectedEof => write!(f, "Unexpected end of file"),
            BlfParseError::UnsupportedCompression(c) => write!(f, "Unsupported compression method: {}", c),
            BlfParseError::UnknownHeaderVersion(v) => write!(f, "Unknown object header version: {}", v),
        }
    }
}

impl Error for BlfParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            BlfParseError::IoError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for BlfParseError {
    fn from(err: io::Error) -> Self {
        if err.kind() == io::ErrorKind::UnexpectedEof {
            BlfParseError::UnexpectedEof
        } else {
            BlfParseError::IoError(err)
        }
    }
}

/// A specialized `Result` type for BLF parsing operations.
pub type BlfParseResult<T> = Result<T, BlfParseError>;

/// Represents the type of a BLF log object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u32)]
pub enum ObjectType {
    /// Unknown object
    #[default]
    Unknown = 0,
    CanMessage = 1,
    CanError = 2,
    CanOverload = 3,
    CanStatistic = 4,
    AppTrigger = 5,
    EnvInteger = 6,
    EnvDouble = 7,
    EnvString = 8,
    LogContainer = 10,
    LinMessage = 11,
    LinCrcError = 12,
    LinDlcInfo = 13,
    LinReceiveError = 14,
    LinSendError = 15,
    LinSlaveTimeout = 16,
    LinSchedulerModeChange = 17,
    LinSyncError = 18,
    LinBaudrate = 19,
    LinSleep = 20,
    LinWakeup = 21,
    MostSpy = 22,
    MostCtrl = 23,
    MostLightLock = 24,
    MostStatistic = 25,
    FlexRayData = 29,
    FlexRaySync = 30,
    CanDriverError = 31,
    MostPkt = 32,
    MostPkt2 = 33,
    MostHwMode = 34,
    MostReg = 35,
    MostGenReg = 36,
    MostNetState = 37,
    MostDataLost = 38,
    MostTrigger = 39,
    FlexRayMessage = 41,
    LinMessage2 = 57,
    EthernetFrame = 71,
    SystemVariable = 72,
    CanMessage2 = 86,
    EventComment = 92,
    GlobalMarker = 96,
    CanFdMessage = 100,
    CanFdMessage64 = 101,
    FlexRayV6StartCycleEvent = 40, // Added
    FlexRayStatusEvent = 45, // Added
    FlexRayVFrError = 47, // Added
    FlexRayVFrStatus = 48, // Added
    FlexRayVFrStartCycle = 49, // Added
    FlexRayVFrReceiveMsg = 50, // Added
    FlexRayVFrReceiveMsgEx = 66, // Added
}

impl From<u32> for ObjectType {
    fn from(val: u32) -> Self {
        match val {
            1 => ObjectType::CanMessage,
            2 => ObjectType::CanError,
            3 => ObjectType::CanOverload,
            4 => ObjectType::CanStatistic,
            5 => ObjectType::AppTrigger,
            6 => ObjectType::EnvInteger,
            7 => ObjectType::EnvDouble,
            8 => ObjectType::EnvString,
            10 => ObjectType::LogContainer,
            11 => ObjectType::LinMessage,
            12 => ObjectType::LinCrcError,
            13 => ObjectType::LinDlcInfo,
            14 => ObjectType::LinReceiveError,
            15 => ObjectType::LinSendError,
            16 => ObjectType::LinSlaveTimeout,
            17 => ObjectType::LinSchedulerModeChange,
            18 => ObjectType::LinSyncError,
            19 => ObjectType::LinBaudrate,
            20 => ObjectType::LinSleep,
            21 => ObjectType::LinWakeup,
            22 => ObjectType::MostSpy,
            23 => ObjectType::MostCtrl,
            24 => ObjectType::MostLightLock,
            25 => ObjectType::MostStatistic,
            29 => ObjectType::FlexRayData,
            30 => ObjectType::FlexRaySync,
            31 => ObjectType::CanDriverError,
            32 => ObjectType::MostPkt,
            33 => ObjectType::MostPkt2,
            34 => ObjectType::MostHwMode,
            35 => ObjectType::MostReg,
            36 => ObjectType::MostGenReg,
            37 => ObjectType::MostNetState,
            38 => ObjectType::MostDataLost,
            39 => ObjectType::MostTrigger,
            40 => ObjectType::FlexRayV6StartCycleEvent,
            41 => ObjectType::FlexRayMessage,
            45 => ObjectType::FlexRayStatusEvent,
            47 => ObjectType::FlexRayVFrError,
            48 => ObjectType::FlexRayVFrStatus,
            49 => ObjectType::FlexRayVFrStartCycle,
            50 => ObjectType::FlexRayVFrReceiveMsg,
            57 => ObjectType::LinMessage2,
            66 => ObjectType::FlexRayVFrReceiveMsgEx,
            71 => ObjectType::EthernetFrame,
            72 => ObjectType::SystemVariable,
            86 => ObjectType::CanMessage2,
            92 => ObjectType::EventComment,
            96 => ObjectType::GlobalMarker,
            100 => ObjectType::CanFdMessage,
            101 => ObjectType::CanFdMessage64,
            _ => ObjectType::Unknown,
        }
    }
}

/// Represents the common header for all BLF log objects (V1 and V2).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ObjectHeader {
    /// Object signature, should be "LOBJ" (0x4A424F4C).
    pub signature: u32,
    /// Size of this header in bytes.
    pub header_size: u16,
    /// Version of the object header (1 or 2).
    pub header_version: u16,
    /// Total size of the object in bytes (header + data).
    pub object_size: u32,
    /// Type of the object.
    pub object_type: ObjectType,
    /// Object-specific flags.
    pub object_flags: u32,
    /// Timestamp of the object.
    pub object_time_stamp: u64,
    /// Original timestamp (only in V2 headers).
    pub original_time_stamp: Option<u64>,
    /// Timestamp status (only in V2 headers).
    pub time_stamp_status: Option<u8>,
}

impl ObjectHeader {
    /// Reads an `ObjectHeader` (V1 or V2) from a byte stream.
    pub fn read(cursor: &mut Cursor<&[u8]>) -> BlfParseResult<Self> {
        let signature = cursor.read_u32::<LittleEndian>()?;
        println!("signature: {:x}", signature);
        if signature != 0x4A424F4C {
            return Err(BlfParseError::InvalidContainerMagic);
        }
        let header_size = cursor.read_u16::<LittleEndian>()?;
        let header_version = cursor.read_u16::<LittleEndian>()?;
        let object_size = cursor.read_u32::<LittleEndian>()?;
        let object_type = ObjectType::from(cursor.read_u32::<LittleEndian>()?);

        let object_flags;
        let object_time_stamp;
        let mut original_time_stamp = None;
        let mut time_stamp_status = None;

        if header_version == 1 {
            object_flags = cursor.read_u32::<LittleEndian>()?;
            let _object_version = cursor.read_u16::<LittleEndian>()?;
            let _client_index = cursor.read_u16::<LittleEndian>()?;
            object_time_stamp = cursor.read_u64::<LittleEndian>()?;
        } else if header_version == 2 {
            object_flags = cursor.read_u32::<LittleEndian>()?;
            time_stamp_status = Some(cursor.read_u8()?);
            let _reserved = cursor.read_u8()?;
            let _object_version = cursor.read_u16::<LittleEndian>()?;
            object_time_stamp = cursor.read_u64::<LittleEndian>()?;
            original_time_stamp = Some(cursor.read_u64::<LittleEndian>()?);
        } else {
            return Err(BlfParseError::UnknownHeaderVersion(header_version));
        }

        Ok(ObjectHeader {
            signature,
            header_size,
            header_version,
            object_size,
            object_type,
            object_flags,
            object_time_stamp,
            original_time_stamp,
            time_stamp_status,
        })
    }

    /// Writes an `ObjectHeader` to a byte stream.
    pub fn write<W: Write>(&self, writer: &mut W) -> BlfParseResult<()> {
        writer.write_u32::<LittleEndian>(self.signature)?;
        writer.write_u16::<LittleEndian>(self.header_size)?;
        writer.write_u16::<LittleEndian>(self.header_version)?;
        writer.write_u32::<LittleEndian>(self.object_size)?;
        writer.write_u32::<LittleEndian>(self.object_type as u32)?;
        writer.write_u32::<LittleEndian>(self.object_flags)?;

        if self.header_version == 1 {
            writer.write_u16::<LittleEndian>(0)?; // _object_version
            writer.write_u16::<LittleEndian>(0)?; // _client_index
            writer.write_u64::<LittleEndian>(self.object_time_stamp)?;
        } else if self.header_version == 2 {
            writer.write_u8(self.time_stamp_status.unwrap_or(0))?;
            writer.write_u8(0)?; // _reserved
            writer.write_u16::<LittleEndian>(0)?; // _object_version
            writer.write_u64::<LittleEndian>(self.object_time_stamp)?;
            writer.write_u64::<LittleEndian>(self.original_time_stamp.unwrap_or(0))?;
        } else {
            return Err(BlfParseError::UnknownHeaderVersion(self.header_version));
        }
        Ok(())
    }

    /// Calculates the size of the header based on its version.
    pub fn calculate_header_size(&self) -> u16 {
        if self.header_version == 1 {
            24 // signature (4) + header_size (2) + header_version (2) + object_size (4) + object_type (4) + object_flags (4) + client_index (2) + object_version (2) + object_time_stamp (8)
        } else if self.header_version == 2 {
            32 // signature (4) + header_size (2) + header_version (2) + object_size (4) + object_type (4) + object_flags (4) + time_stamp_status (1) + reserved (1) + object_version (2) + object_time_stamp (8) + original_time_stamp (8)
        } else {
            0 // Should not happen with proper validation
        }
    }
}

