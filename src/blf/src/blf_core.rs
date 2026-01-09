//! Core BLF structures and error handling.

// byteorder imports removed - not used in blf_core.rs
use std::error::Error;
use std::fmt;
use std::io;

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
    /// Unexpected data was encountered during parsing.
    UnexpectedData,
}

impl fmt::Display for BlfParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BlfParseError::IoError(e) => write!(f, "I/O error: {}", e),
            BlfParseError::InvalidFileMagic => {
                write!(
                    f,
                    "Invalid BLF file magic string (expected 'LOGG' signature)"
                )
            }
            BlfParseError::InvalidContainerMagic => {
                write!(
                    f,
                    "Invalid LOBJ container magic string (expected 0x4A424F4C)"
                )
            }
            BlfParseError::UnexpectedEof => write!(f, "Unexpected end of file while parsing"),
            BlfParseError::UnsupportedCompression(c) => {
                write!(
                    f,
                    "Unsupported compression method: {} (only uncompressed and zlib supported)",
                    c
                )
            }
            BlfParseError::UnknownHeaderVersion(v) => {
                write!(
                    f,
                    "Unknown object header version: {} (only versions 1 and 2 supported)",
                    v
                )
            }
            BlfParseError::UnexpectedData => {
                write!(f, "Unexpected data encountered during parsing")
            }
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
    FlexRayStatusEvent = 45,       // Added
    FlexRayVFrError = 47,          // Added
    FlexRayVFrStatus = 48,         // Added
    FlexRayVFrStartCycle = 49,     // Added
    FlexRayVFrReceiveMsg = 50,     // Added
    FlexRayVFrReceiveMsgEx = 66,   // Added
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

// ObjectHeader is now defined in the objects module
// This file only contains BlfParseError and ObjectType definitions
