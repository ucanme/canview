//! System-level and file-related object definitions.

use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Cursor;
use crate::{BlfParseResult, ObjectHeader};

/// Represents a data lost begin event (`DATA_LOST_BEGIN`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLostBegin {
    /// Identifier for the leaking queue.
    pub queue_identifier: u32,
    /// Timestamp of the message.
    pub timestamp: u64,
}

impl DataLostBegin {
    pub(crate) fn read(cursor: &mut Cursor<&[u8]>, header: &ObjectHeader) -> BlfParseResult<Self> {
        let queue_identifier = cursor.read_u32::<LittleEndian>()?;
        Ok(Self {
            queue_identifier,
            timestamp: header.object_time_stamp,
        })
    }
}

/// Represents a data lost end event (`DATA_LOST_END`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLostEnd {
    /// Identifier for the leaking queue.
    pub queue_identifier: u32,
    /// Timestamp of the first object lost.
    pub first_object_lost_time_stamp: u64,
    /// Number of lost events.
    pub number_of_lost_events: u32,
    /// Timestamp of the message.
    pub timestamp: u64,
}

impl DataLostEnd {
    pub(crate) fn read(cursor: &mut Cursor<&[u8]>, header: &ObjectHeader) -> BlfParseResult<Self> {
        let queue_identifier = cursor.read_u32::<LittleEndian>()?;
        let first_object_lost_time_stamp = cursor.read_u64::<LittleEndian>()?;
        let number_of_lost_events = cursor.read_u32::<LittleEndian>()?;
        Ok(Self {
            queue_identifier,
            first_object_lost_time_stamp,
            number_of_lost_events,
            timestamp: header.object_time_stamp,
        })
    }
}

/// Represents a SYSTEMTIME structure used in BLF headers for timestamps.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SystemTime {
    /// Year.
    pub year: u16,
    /// Month.
    pub month: u16,
    /// Day of week (0=Sunday, 6=Saturday).
    pub day_of_week: u16,
    /// Day.
    pub day: u16,
    /// Hour.
    pub hour: u16,
    /// Minute.
    pub minute: u16,
    /// Second.
    pub second: u16,
    /// Milliseconds.
    pub milliseconds: u16,
}

impl SystemTime {
    pub(crate) fn read(cursor: &mut Cursor<&[u8]>) -> BlfParseResult<Self> {
        let year = cursor.read_u16::<LittleEndian>()?;
        let month = cursor.read_u16::<LittleEndian>()?;
        let day_of_week = cursor.read_u16::<LittleEndian>()?;
        let day = cursor.read_u16::<LittleEndian>()?;
        let hour = cursor.read_u16::<LittleEndian>()?;
        let minute = cursor.read_u16::<LittleEndian>()?;
        let second = cursor.read_u16::<LittleEndian>()?;
        let milliseconds = cursor.read_u16::<LittleEndian>()?;
        Ok(Self {
            year,
            month,
            day_of_week,
            day,
            hour,
            minute,
            second,
            milliseconds,
        })
    }
}

/// Represents file statistics (`FILE_STATISTICS`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileStatistics {
    /// File signature, should be "LOGG" (0x47474F4C).
    pub signature: u32,
    /// Size of the statistics block.
    pub statistics_size: u32,
    /// BL API number (major * 1000000 + minor * 1000 + build * 100 + patch).
    pub api_number: u32,
    /// Application ID.
    pub application_id: u8,
    /// Compression level.
    pub compression_level: u8,
    /// Application major number.
    pub application_major: u8,
    /// Application minor number.
    pub application_minor: u8,
    /// (Compressed) file size in bytes.
    pub file_size: u64,
    /// Uncompressed file size in bytes.
    pub uncompressed_file_size: u64,
    /// Number of objects.
    pub object_count: u32,
    /// Application build number.
    pub application_build: u32,
    /// Measurement start time.
    pub measurement_start_time: SystemTime,
    /// Last object time.
    pub last_object_time: SystemTime,
    /// File position of the (first) LogContainer that contains RestorePointContainer objects.
    pub restore_points_offset: u64,
}

impl FileStatistics {
    pub(crate) fn read(cursor: &mut Cursor<&[u8]>) -> BlfParseResult<Self> {
        let signature = cursor.read_u32::<LittleEndian>()?;
        let statistics_size = cursor.read_u32::<LittleEndian>()?;
        let api_number = cursor.read_u32::<LittleEndian>()?;
        let application_id = cursor.read_u8()?;
        let compression_level = cursor.read_u8()?;
        let application_major = cursor.read_u8()?;
        let application_minor = cursor.read_u8()?;
        let file_size = cursor.read_u64::<LittleEndian>()?;
        let uncompressed_file_size = cursor.read_u64::<LittleEndian>()?;
        let object_count = cursor.read_u32::<LittleEndian>()?;
        let application_build = cursor.read_u32::<LittleEndian>()?;
        let measurement_start_time = SystemTime::read(cursor)?;
        let last_object_time = SystemTime::read(cursor)?;
        let restore_points_offset = cursor.read_u64::<LittleEndian>()?;
        let _reserved_file_statistics: [u32; 16] = [
            cursor.read_u32::<LittleEndian>()?, cursor.read_u32::<LittleEndian>()?,
            cursor.read_u32::<LittleEndian>()?, cursor.read_u32::<LittleEndian>()?,
            cursor.read_u32::<LittleEndian>()?, cursor.read_u32::<LittleEndian>()?,
            cursor.read_u32::<LittleEndian>()?, cursor.read_u32::<LittleEndian>()?,
            cursor.read_u32::<LittleEndian>()?, cursor.read_u32::<LittleEndian>()?,
            cursor.read_u32::<LittleEndian>()?, cursor.read_u32::<LittleEndian>()?,
            cursor.read_u32::<LittleEndian>()?, cursor.read_u32::<LittleEndian>()?,
            cursor.read_u32::<LittleEndian>()?, cursor.read_u32::<LittleEndian>()?,
        ];

        Ok(Self {
            signature,
            statistics_size,
            api_number,
            application_id,
            compression_level,
            application_major,
            application_minor,
            file_size,
            uncompressed_file_size,
            object_count,
            application_build,
            measurement_start_time,
            last_object_time,
            restore_points_offset,
        })
    }
}
