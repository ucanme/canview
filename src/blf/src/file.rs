

//! Handles the top-level reading and parsing of BLF files.

use crate::{
    BlfParser, BlfParseError, BlfParseResult, FileStatistics, LogObject, ObjectHeader,
};
use std::fs;
use std::io::Cursor;
use std::path::Path;

/// Represents the complete result of parsing a BLF file.
#[derive(Debug)]
pub struct BlfResult {
    /// The file statistics header.
    pub file_stats: FileStatistics,
    /// A vector of all parsed log objects.
    pub objects: Vec<LogObject>,
}

/// Reads a BLF file from the given path and parses its content.
///
/// This function orchestrates the entire parsing process:
/// 1. Reads the raw byte data from the specified file path.
/// 2. Parses the initial `FileStatistics` header to get file metadata.
/// 3. Slices the remaining byte data and passes it to the `BlfParser` to parse all log objects.
///
/// # Arguments
///
/// * `path` - A type that can be referenced as a `Path` to the BLF file.
///
/// # Returns
///
/// A `BlfParseResult` containing a `BlfResult` struct on success, which holds both the
/// file statistics and the list of parsed log objects.
pub fn read_blf_from_file<P: AsRef<Path>>(path: P) -> BlfParseResult<BlfResult> {
    let data = fs::read(path).map_err(BlfParseError::IoError)?;
    let mut cursor = Cursor::new(&data[..]);

    // 1. Parse the file statistics header. This will advance the cursor.
    let file_stats = FileStatistics::read(&mut cursor)?;

    // 2. Parse the log objects from the rest of the data slice.
    let parser = BlfParser::new();
    let remaining_data = &data[cursor.position() as usize..];
    let objects = parser.parse(remaining_data)?;

    Ok(BlfResult { file_stats, objects })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;
    use crate::{CanMessage, LogContainer, ObjectHeader, ObjectType, SystemTime};
    use std::io::Write;

    #[test]
    fn test_read_blf_from_file_successfully() {
        // 1. --- Define the objects we want to serialize ---
        let can_msg_header = ObjectHeader {
            signature: 0x4A424F4C, // "LOBJ"
            header_size: 24,
            header_version: 1,
            object_size: 40, // header + can_msg_fields + data
            object_type: ObjectType::CanMessage,
            object_flags: 0,
            object_time_stamp: 1000,
            original_time_stamp: None,
            time_stamp_status: None,
        };
        let can_message = CanMessage {
            header: can_msg_header,
            channel: 1,
            flags: 0,
            dlc: 8,
            id: 0x123,
            data: [1, 2, 3, 4, 5, 6, 7, 8],
        };

        // 2. --- Serialize the inner object ---
        let mut inner_object_bytes = serialize_can_message(&can_message);
        add_padding(&mut inner_object_bytes);

        // 3. --- Create and serialize the LogContainer ---
        let container_header = ObjectHeader {
            signature: 0x4A424F4C, // "LOBJ"
            header_size: 32, // 修正header_size为实际大小
            header_version: 1,
            object_size: 0, // Will be calculated later
            object_type: ObjectType::LogContainer,
            object_flags: 0,
            object_time_stamp: 0,
            original_time_stamp: None,
            time_stamp_status: None,
        };
        let mut log_container = LogContainer {
            header: ObjectHeader {
                signature: 0x4A424F4C, // "LOBJ"
                ..container_header.clone()
            },
            compression_method: 0, // No compression
            uncompressed_data: inner_object_bytes.clone(),
        };
        // Correctly calculate the object size
        let calculated_size = log_container.calculate_object_size();
        
        // 更新LogContainer的header大小
        log_container.header.object_size = calculated_size;

        let mut container_bytes = serialize_log_container(&log_container);
        add_padding(&mut container_bytes);

        // 4. --- Create and serialize the FileStatistics header ---
        let file_stats = FileStatistics {
            statistics_size: 208,
            application_id: 1,
            application_major: 1,
            application_minor: 0,
            application_build: 0,
            file_size: (208 + container_bytes.len()) as u64,
            uncompressed_file_size: (208 + inner_object_bytes.len()) as u64,
            object_count: 1,
            measurement_start_time: SystemTime { year: 2025, month: 11, day: 22, day_of_week: 0, hour: 8, minute: 30, second: 0, milliseconds: 0 },
            last_object_time: SystemTime { year: 2025, month: 11, day: 22, day_of_week: 0, hour: 8, minute: 30, second: 1, milliseconds: 0 },
        };
        let file_stats_bytes = serialize_file_statistics(&file_stats);

        // 5. --- Combine all parts into a single byte array ---
        let mut blf_data = Vec::new();
        blf_data.extend(file_stats_bytes.clone());
        blf_data.extend(container_bytes.clone());

        // 6. --- Write to a temporary file and parse it ---
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        temp_file.write_all(&blf_data).unwrap();

        let result = read_blf_from_file(temp_file.path()).unwrap();

        // 7. --- Assert the results ---
        assert_eq!(result.file_stats, file_stats);
        assert_eq!(result.objects.len(), 1);

        if let Some(LogObject::CanMessage(parsed_can_message)) = result.objects.first() {
            assert_eq!(parsed_can_message, &can_message);
        } else {
            panic!("Expected a CanMessage but found something else.");
        }
    }
}