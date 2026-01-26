//! File statistics header definition.

use crate::{BlfParseError, BlfParseResult};
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Cursor, Read};

const FILE_SIGNATURE: u32 = 0x47474f4c; // "LOGG" (注意字节序)

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

    /// 转换为 Unix 时间戳（纳秒）
    ///
    /// 返回自 1970-01-01 00:00:00 UTC 以来的纳秒数
    pub fn to_timestamp_nanos(&self) -> i64 {
        use chrono::{TimeZone, Utc};

        if let Some(dt) = Utc
            .with_ymd_and_hms(
                self.year as i32,
                self.month as u32,
                self.day as u32,
                self.hour as u32,
                self.minute as u32,
                self.second as u32,
            )
            .single()
        {
            dt.timestamp_nanos_opt().unwrap_or(0) + (self.milliseconds as i64) * 1_000_000
        } else {
            0
        }
    }

    /// 添加纳秒偏移，返回新的时间戳（纳秒）
    ///
    /// # 参数
    /// - `offset_ns`: 偏移量（纳秒）
    ///
    /// # 返回
    /// 绝对时间戳（纳秒）
    pub fn add_nanoseconds(&self, offset_ns: u64) -> i64 {
        self.to_timestamp_nanos() + offset_ns as i64
    }

    /// 格式化为字符串
    ///
    /// # 格式
    /// `YYYY-MM-DD HH:MM:SS.mmm`
    pub fn format(&self) -> String {
        format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:03}",
            self.year, self.month, self.day, self.hour, self.minute, self.second, self.milliseconds
        )
    }

    /// 格式化绝对时间（基准时间 + 偏移）
    ///
    /// # 参数
    /// - `offset_ns`: 偏移量（纳秒）
    ///
    /// # 返回
    /// 格式化的时间字符串
    pub fn format_with_offset(&self, offset_ns: u64) -> String {
        use chrono::DateTime;

        let absolute_ns = self.add_nanoseconds(offset_ns);

        // 转换为 DateTime
        if let Some(dt) = DateTime::from_timestamp(
            absolute_ns / 1_000_000_000,
            (absolute_ns % 1_000_000_000) as u32,
        ) {
            dt.format("%Y-%m-%d %H:%M:%S%.6f").to_string()
        } else {
            format!("Invalid time (offset: {} ns)", offset_ns)
        }
    }
}

/// Represents the file statistics header at the beginning of a BLF file.
#[derive(Debug, Clone, PartialEq)]
pub struct FileStatistics {
    /// The size of this structure.
    pub statistics_size: u32,
    /// The API number.
    pub api_number: u32,
    /// The application ID.
    pub application_id: u8,
    ///CompressionLevel
    pub compression_level: u8,
    /// The application's major version number.
    pub application_major: u8,
    /// The application's minor version number.
    pub application_minor: u8,
    /// The total size of the file in bytes.
    pub file_size: u64,
    /// The total uncompressed size of all objects.
    pub uncompressed_file_size: u64,
    /// The total count of objects in the file.
    pub object_count: u32,
    /// The application's build number.
    pub application_build: u32,
    /// The timestamp when the measurement started.
    pub measurement_start_time: SystemTime,
    /// The timestamp of the last object in the file.
    pub last_object_time: SystemTime,
}

impl FileStatistics {
    /// Reads a `FileStatistics` header from a byte stream.
    ///
    /// This follows the Vector BLF C++ implementation's format:
    /// 1. signature (4 bytes) - "LOGG"
    /// 2. statisticsSize (4 bytes)
    /// 3. apiNumber (4 bytes)
    /// 4. applicationId (1 byte)
    /// 5. compressionLevel (1 byte)
    /// 6. applicationMajor (1 byte)
    /// 7. applicationMinor (1 byte)
    /// 8. fileSize (8 bytes)
    /// 9. uncompressedFileSize (8 bytes)
    /// 10. objectCount (4 bytes)
    /// 11. applicationBuild (4 bytes)
    /// 12. measurementStartTime (16 bytes - SYSTEMTIME)
    /// 13. lastObjectTime (16 bytes - SYSTEMTIME)
    /// 14. reserved/restorePointsOffset (variable)
    pub fn read(cursor: &mut Cursor<&[u8]>) -> BlfParseResult<Self> {
        // 读取文件签名 (字节序: 0x47474f4c = "LOGG")
        let signature = cursor.read_u32::<LittleEndian>()?;
        if signature != FILE_SIGNATURE {
            return Err(BlfParseError::InvalidFileMagic);
        }

        // 读取统计信息大小
        let statistics_size = cursor.read_u32::<LittleEndian>()?;

        // 读取 API number (在144字节格式中可能为0)
        let api_number = cursor.read_u32::<LittleEndian>()?;

        // 读取应用程序信息
        let application_id = cursor.read_u8()?;
        let compression_level = cursor.read_u8()?;
        let application_major = cursor.read_u8()?;
        let application_minor = cursor.read_u8()?;

        // 读取文件统计信息
        let file_size = cursor.read_u64::<LittleEndian>()?;
        let uncompressed_file_size = cursor.read_u64::<LittleEndian>()?;
        let object_count = cursor.read_u32::<LittleEndian>()?;

        // 读取 application build
        let application_build = cursor.read_u32::<LittleEndian>()?;

        // 读取时间戳信息
        let measurement_start_time = SystemTime::read(cursor)?;
        let last_object_time = SystemTime::read(cursor)?;

        // 读取剩余的保留字段
        let current_pos = cursor.position();
        let remaining_bytes = statistics_size as u64 - current_pos;
        if remaining_bytes > 0 {
            let mut _rest = vec![0u8; remaining_bytes as usize];
            cursor.read_exact(&mut _rest)?;
        }

        Ok(FileStatistics {
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
            statistics_size: 144,
            api_number: 0,
            application_id: 1,
            compression_level: 0,
            application_major: 196,
            application_minor: 103,
            file_size: 21978632,
            uncompressed_file_size: 199968,
            object_count: 166751,
            application_build: 53,
            measurement_start_time: SystemTime {
                year: 2025,
                month: 1,
                day_of_week: 0,
                day: 2,
                hour: 3,
                minute: 4,
                second: 5,
                milliseconds: 6,
            },
            last_object_time: SystemTime {
                year: 2025,
                month: 1,
                day_of_week: 0,
                day: 2,
                hour: 3,
                minute: 4,
                second: 5,
                milliseconds: 6,
            },
        };

        let data = serialize_file_statistics(&original_stats);
        println!("Serialized data length: {}", data.len());
        println!(
            "Expected statistics_size: {}",
            original_stats.statistics_size
        );
        let mut cursor = Cursor::new(&data[..]);
        let parsed_stats = FileStatistics::read(&mut cursor).unwrap();

        // Compare all fields
        assert_eq!(original_stats.statistics_size, parsed_stats.statistics_size);
        assert_eq!(original_stats.api_number, parsed_stats.api_number);
        assert_eq!(original_stats.application_id, parsed_stats.application_id);
        assert_eq!(
            original_stats.compression_level,
            parsed_stats.compression_level
        );
        assert_eq!(
            original_stats.application_major,
            parsed_stats.application_major
        );
        assert_eq!(
            original_stats.application_minor,
            parsed_stats.application_minor
        );
        assert_eq!(
            original_stats.application_build,
            parsed_stats.application_build
        );
        assert_eq!(original_stats.file_size, parsed_stats.file_size);
        assert_eq!(
            original_stats.uncompressed_file_size,
            parsed_stats.uncompressed_file_size
        );
        assert_eq!(original_stats.object_count, parsed_stats.object_count);
        assert_eq!(
            original_stats.measurement_start_time,
            parsed_stats.measurement_start_time
        );
        assert_eq!(
            original_stats.last_object_time,
            parsed_stats.last_object_time
        );
    }

    #[test]
    fn test_read_can_blf_file() {
        // This test validates the actual can.blf file header
        // Based on the hex dump analysis:
        // 0x00-0x03: "LOGG" signature
        // 0x04-0x07: 0x00000090 (144 bytes)
        // 0x08-0x0B: API number (or reserved)
        // 0x0C: 0x01 (application_id)
        // 0x0D: 0x00 (compression_level)
        // 0x0E: 0xc4 (196 - application_major)
        // 0x0F: 0x67 (103 - application_minor)
        // Note: application_build comes later in standard format

        let mut data = vec![0u8; 144];
        let mut cursor = Cursor::new(&mut data[..]);

        // Write the header
        use byteorder::{LittleEndian, WriteBytesExt};
        cursor.write_u32::<LittleEndian>(0x47474f4c).unwrap(); // "LOGG"
        cursor.write_u32::<LittleEndian>(144).unwrap(); // statistics_size
        cursor.write_u32::<LittleEndian>(0).unwrap(); // api_number
        cursor.write_u8(1).unwrap(); // application_id
        cursor.write_u8(0).unwrap(); // compression_level
        cursor.write_u8(196).unwrap(); // application_major
        cursor.write_u8(103).unwrap(); // application_minor

        // Write file statistics
        cursor.write_u64::<LittleEndian>(21978632).unwrap(); // file_size
        cursor.write_u64::<LittleEndian>(199968).unwrap(); // uncompressed_file_size
        cursor.write_u32::<LittleEndian>(166751).unwrap(); // object_count
        cursor.write_u32::<LittleEndian>(53).unwrap(); // application_build

        // Write measurement start time
        cursor.write_u16::<LittleEndian>(2025).unwrap(); // year
        cursor.write_u16::<LittleEndian>(1).unwrap(); // month
        cursor.write_u16::<LittleEndian>(0).unwrap(); // day_of_week
        cursor.write_u16::<LittleEndian>(2).unwrap(); // day
        cursor.write_u16::<LittleEndian>(3).unwrap(); // hour
        cursor.write_u16::<LittleEndian>(4).unwrap(); // minute
        cursor.write_u16::<LittleEndian>(5).unwrap(); // second
        cursor.write_u16::<LittleEndian>(6).unwrap(); // milliseconds

        // Write last object time (same as start time for this test)
        cursor.write_u16::<LittleEndian>(2025).unwrap(); // year
        cursor.write_u16::<LittleEndian>(1).unwrap(); // month
        cursor.write_u16::<LittleEndian>(0).unwrap(); // day_of_week
        cursor.write_u16::<LittleEndian>(2).unwrap(); // day
        cursor.write_u16::<LittleEndian>(3).unwrap(); // hour
        cursor.write_u16::<LittleEndian>(4).unwrap(); // minute
        cursor.write_u16::<LittleEndian>(5).unwrap(); // second
        cursor.write_u16::<LittleEndian>(6).unwrap(); // milliseconds

        // Fill the rest with zeros (reserved + padding)
        let remaining = 144 - cursor.position() as usize;
        cursor.write_all(&vec![0u8; remaining]).unwrap();

        // Now read it back
        let mut read_cursor = Cursor::new(&data[..]);
        let stats = FileStatistics::read(&mut read_cursor).unwrap();

        // Verify all fields match can.blf format
        assert_eq!(stats.statistics_size, 144);
        assert_eq!(stats.api_number, 0); // From the data we wrote
        assert_eq!(stats.application_id, 1);
        assert_eq!(stats.compression_level, 0);
        assert_eq!(stats.application_major, 196);
        assert_eq!(stats.application_minor, 103);
        assert_eq!(stats.application_build, 53);
        assert_eq!(stats.file_size, 21978632);
        assert_eq!(stats.uncompressed_file_size, 199968);
        assert_eq!(stats.object_count, 166751);
        assert_eq!(stats.measurement_start_time.year, 2025);
        assert_eq!(stats.measurement_start_time.month, 1);
        assert_eq!(stats.last_object_time.year, 2025);
        assert_eq!(stats.last_object_time.month, 1);
    }

    #[test]
    fn test_read_file_statistics_invalid_signature() {
        let mut data = vec![0; 104];
        use byteorder::{LittleEndian, WriteBytesExt};
        // Write an invalid signature
        (&mut data[0..4])
            .write_u32::<LittleEndian>(0xDEADBEEF)
            .unwrap();
        let mut cursor = Cursor::new(&data[..]);
        let result = FileStatistics::read(&mut cursor);

        assert!(matches!(result, Err(BlfParseError::InvalidFileMagic)));
    }
}
