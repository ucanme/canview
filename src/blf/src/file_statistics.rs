//! File statistics header definition.

use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Read, Cursor};
use crate::{BlfParseResult, BlfParseError};

const FILE_SIGNATURE: u32 = 0x47474F4C; // "LOGG"

/// Represents the Windows SYSTEMTIME structure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemTime {
    /// The year.
    pub year: u16,
    /// The month (1-12).
    pub month: u16,
    /// The day of the week (0=Sunday, 1=Monday, ...).
    pub day_of_week: u16,
    /// The day of the month (1-31).
    pub day: u16,
    /// The hour (0-23).
    pub hour: u16,
    /// The minute (0-59).
    pub minute: u16,
    /// The second (0-59).
    pub second: u16,
    /// The millisecond (0-999).
    pub milliseconds: u16,
}

impl SystemTime {
    /// Reads a `SystemTime` from a byte cursor.
    pub fn read(cursor: &mut Cursor<&[u8]>) -> BlfParseResult<Self> {
        Ok(Self {
            year: cursor.read_u16::<LittleEndian>()?,
            month: cursor.read_u16::<LittleEndian>()?,
            day_of_week: cursor.read_u16::<LittleEndian>()?,
            day: cursor.read_u16::<LittleEndian>()?,
            hour: cursor.read_u16::<LittleEndian>()?,
            minute: cursor.read_u16::<LittleEndian>()?,
            second: cursor.read_u16::<LittleEndian>()?,
            milliseconds: cursor.read_u16::<LittleEndian>()?,
        })
    }
}

/// Represents the file statistics header at the beginning of a BLF file.
#[derive(Debug, Clone, PartialEq)]
pub struct FileStatistics {
    /// The size of this structure.
    pub statistics_size: u32,
    /// The application ID.
    pub application_id: u8,
    /// The application's major version number.
    pub application_major: u8,
    /// The application's minor version number.
    pub application_minor: u8,
    /// The application's build number.
    pub application_build: u8,
    /// The total size of the file in bytes.
    pub file_size: u64,
    /// The total uncompressed size of all objects.
    pub uncompressed_file_size: u64,
    /// The total count of objects in the file.
    pub object_count: u64,
    /// The timestamp when the measurement started.
    pub measurement_start_time: SystemTime,
    /// The timestamp of the last object in the file.
    pub last_object_time: SystemTime,
}

impl FileStatistics {
    /// Reads a `FileStatistics` header from a byte stream.
    pub fn read(cursor: &mut Cursor<&[u8]>) -> BlfParseResult<Self> {
        let signature = cursor.read_u32::<LittleEndian>()?;
        if signature != 0x474F4C42 {
            return Err(BlfParseError::InvalidFileMagic);
        }
        let header_size = cursor.read_u32::<LittleEndian>()?;
        let _crc = cursor.read_u32::<LittleEndian>()?;
        let application_id = cursor.read_u8()?;
        let _compression_level = cursor.read_u8()?;
        let application_major = cursor.read_u8()?;
        let application_minor = cursor.read_u8()?;
        let file_size = cursor.read_u64::<LittleEndian>()?;
        let uncompressed_file_size = cursor.read_u64::<LittleEndian>()?;
        let object_count = cursor.read_u64::<LittleEndian>()?;
        let application_build = cursor.read_u8()?;
        cursor.set_position(cursor.position() + 3); // Skip padding
        let measurement_start_time = SystemTime::read(cursor)?;
        let last_object_time = SystemTime::read(cursor)?;

        // Skip over reserved fields and API number
        // We can't use set_position with generic Read trait, so we'll read and discard
        let mut skip_buf = [0u8; 4];
        cursor.read_exact(&mut skip_buf)?; // apiNumber
        
        let mut skip_buf = [0u8; 32];
        cursor.read_exact(&mut skip_buf)?; // reserved
        
        let mut skip_buf = [0u8; 96];
        cursor.read_exact(&mut skip_buf)?; // restOfHeader

        // Make sure we've read exactly statistics_size bytes
        // If we've read more, that's an error. If we've read less, skip the remaining bytes.
        let current_pos = cursor.position() as u32;
        if current_pos < header_size {
            let remaining = header_size - current_pos;
            cursor.set_position(cursor.position() + remaining as u64);
        }

        Ok(FileStatistics {
            statistics_size: header_size,
            application_id,
            application_major,
            application_minor,
            application_build,
            file_size,
            uncompressed_file_size,
            object_count,
            measurement_start_time,
            last_object_time,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;

    #[test]
    fn test_read_file_statistics_successfully() {
        let original_stats = FileStatistics {
            statistics_size: 208,
            application_id: 1,
            application_major: 2,
            application_minor: 3,
            application_build: 4,
            file_size: 2048,
            uncompressed_file_size: 4096,
            object_count: 42,
            measurement_start_time: SystemTime { year: 2025, month: 1, day_of_week: 0, day: 2, hour: 3, minute: 4, second: 5, milliseconds: 6 },
            last_object_time: SystemTime { year: 2025, month: 1, day_of_week: 0, day: 2, hour: 3, minute: 4, second: 5, milliseconds: 6 },
        };

        let data = serialize_file_statistics(&original_stats);
        let mut cursor = Cursor::new(&data[..]);
        let parsed_stats = FileStatistics::read(&mut cursor).unwrap();

        assert_eq!(original_stats, parsed_stats);
    }

    #[test]
    fn test_read_file_statistics_invalid_signature() {
        let mut data = vec![0; 104];
        use byteorder::{LittleEndian, WriteBytesExt};
        // Write an invalid signature
        (&mut data[0..4]).write_u32::<LittleEndian>(0xDEADBEEF).unwrap();
        let mut cursor = Cursor::new(&data[..]);
        let result = FileStatistics::read(&mut cursor);

        assert!(matches!(result, Err(BlfParseError::InvalidFileMagic)));
    }
}