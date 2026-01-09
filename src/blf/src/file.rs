//! Handles the top-level reading and parsing of BLF files.

use crate::{BlfParseError, BlfParseResult, BlfParser, FileStatistics, LogObject};
use std::fs::{self, File};
use std::io::{BufReader, Cursor, Read, Seek, SeekFrom};
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

    Ok(BlfResult {
        file_stats,
        objects,
    })
}

/// Streaming BLF reader for handling large files efficiently
pub struct StreamingBlfReader {
    reader: BufReader<File>,
    file_stats: FileStatistics,
    parser: BlfParser,
    buffer: Vec<u8>,
    buffer_size: usize,
    total_file_size: u64,
    current_position: u64,
}

impl StreamingBlfReader {
    /// Creates a new streaming BLF reader
    pub fn new<P: AsRef<Path>>(path: P) -> BlfParseResult<Self> {
        let file = File::open(path).map_err(BlfParseError::IoError)?;
        let file_size = file.metadata().map_err(BlfParseError::IoError)?.len();
        let mut reader = BufReader::new(file);

        // Read the file statistics header first
        let mut header_buffer = vec![0u8; 208]; // FileStatistics is typically 208 bytes
        reader
            .read_exact(&mut header_buffer)
            .map_err(BlfParseError::IoError)?;

        let mut cursor = Cursor::new(&header_buffer[..]);
        let file_stats = FileStatistics::read(&mut cursor)?;

        Ok(Self {
            reader,
            file_stats,
            parser: BlfParser::with_debug(),
            buffer: Vec::with_capacity(1024 * 1024), // 1MB buffer
            buffer_size: 1024 * 1024,
            total_file_size: file_size,
            current_position: cursor.position(),
        })
    }

    /// Returns the file statistics
    pub fn file_stats(&self) -> &FileStatistics {
        &self.file_stats
    }

    /// Reads the next batch of log objects
    pub fn read_next_batch(&mut self, batch_size: usize) -> BlfParseResult<Vec<LogObject>> {
        if self.current_position >= self.total_file_size {
            return Ok(Vec::new()); // End of file
        }

        // Calculate how much to read
        let remaining_bytes = self.total_file_size - self.current_position;
        let read_size = std::cmp::min(self.buffer_size as u64, remaining_bytes) as usize;

        // Resize buffer if needed
        self.buffer.resize(read_size, 0);

        // Read data from file
        self.reader
            .read_exact(&mut self.buffer[..read_size])
            .map_err(BlfParseError::IoError)?;

        // Parse the buffer
        let objects = self.parser.parse(&self.buffer)?;
        self.current_position += read_size as u64;

        // Return only the requested batch size
        Ok(objects.into_iter().take(batch_size).collect())
    }

    /// Seeks to a specific position in the file (for random access)
    pub fn seek_to_position(&mut self, position: u64) -> BlfParseResult<()> {
        if position < 208 {
            // Can't seek before file statistics header
            return Err(BlfParseError::InvalidFileMagic);
        }

        self.reader
            .seek(SeekFrom::Start(position))
            .map_err(BlfParseError::IoError)?;
        self.current_position = position;
        Ok(())
    }

    /// Returns the current reading progress (0.0 to 1.0)
    pub fn progress(&self) -> f64 {
        if self.total_file_size == 0 {
            1.0
        } else {
            self.current_position as f64 / self.total_file_size as f64
        }
    }

    /// Checks if we've reached the end of the file
    pub fn is_eof(&self) -> bool {
        self.current_position >= self.total_file_size
    }
}

/// Iterator implementation for streaming BLF reader
pub struct BlfIterator {
    reader: StreamingBlfReader,
    batch_size: usize,
    current_batch: Vec<LogObject>,
    batch_index: usize,
}

impl BlfIterator {
    pub fn new(reader: StreamingBlfReader, batch_size: usize) -> Self {
        Self {
            reader,
            batch_size,
            current_batch: Vec::new(),
            batch_index: 0,
        }
    }
}

impl Iterator for BlfIterator {
    type Item = BlfParseResult<LogObject>;

    fn next(&mut self) -> Option<Self::Item> {
        // If we've consumed all objects in the current batch, load the next batch
        if self.batch_index >= self.current_batch.len() {
            match self.reader.read_next_batch(self.batch_size) {
                Ok(batch) => {
                    if batch.is_empty() {
                        return None; // End of file
                    }
                    self.current_batch = batch;
                    self.batch_index = 0;
                }
                Err(e) => return Some(Err(e)),
            }
        }

        // Return the next object from the current batch
        if self.batch_index < self.current_batch.len() {
            let obj = self.current_batch[self.batch_index].clone();
            self.batch_index += 1;
            Some(Ok(obj))
        } else {
            None
        }
    }
}

/// Convenience function to create a streaming BLF iterator
pub fn stream_blf_from_file<P: AsRef<Path>>(
    path: P,
    batch_size: usize,
) -> BlfParseResult<BlfIterator> {
    let reader = StreamingBlfReader::new(path)?;
    Ok(BlfIterator::new(reader, batch_size))
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
            base: crate::objects::object_header::ObjectHeaderBase {
                signature: 0x4A424F4C, // "LOBJ"
                header_size: 32,
                header_version: 1,
                object_size: 48, // header + can_msg_fields + data
                object_type: ObjectType::CanMessage,
            },
            object_flags: 0,
            client_index: 0,
            object_version: 0,
            object_time_stamp: 1000,
            original_time_stamp: None,
            time_stamp_status: None,
            reserved: 0,
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
            base: crate::objects::object_header::ObjectHeaderBase {
                signature: 0x4A424F4C, // "LOBJ"
                header_size: 32,       // 修正header_size为实际大小
                header_version: 1,
                object_size: 0, // Will be calculated later
                object_type: ObjectType::LogContainer,
            },
            object_flags: 0,
            client_index: 0,
            object_version: 0,
            object_time_stamp: 0,
            original_time_stamp: None,
            time_stamp_status: None,
        };
        let mut log_container = LogContainer {
            header: container_header.clone(),
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
            api_number: 0,
            application_id: 1,
            compression_level: 0,
            application_major: 1,
            application_minor: 0,
            file_size: (208 + container_bytes.len()) as u64,
            uncompressed_file_size: (208 + inner_object_bytes.len()) as u64,
            object_count: 1,
            application_build: 0,
            measurement_start_time: SystemTime {
                year: 2025,
                month: 11,
                day: 22,
                day_of_week: 0,
                hour: 8,
                minute: 30,
                second: 0,
                milliseconds: 0,
            },
            last_object_time: SystemTime {
                year: 2025,
                month: 11,
                day: 22,
                day_of_week: 0,
                hour: 8,
                minute: 30,
                second: 1,
                milliseconds: 0,
            },
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

    #[test]
    fn test_streaming_blf_reader() {
        // Create a simple BLF file for testing
        let can_msg_header = ObjectHeader {
            signature: 0x4A424F4C,
            header_size: 32,
            header_version: 1,
            object_size: 48,
            object_type: ObjectType::CanMessage,
            object_flags: 0,
            client_index: 0,
            object_version: 0,
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

        let mut inner_object_bytes = serialize_can_message(&can_message);
        add_padding(&mut inner_object_bytes);

        let container_header = ObjectHeader {
            signature: 0x4A424F4C,
            header_size: 32,
            header_version: 1,
            object_size: 0,
            object_type: ObjectType::LogContainer,
            object_flags: 0,
            client_index: 0,
            object_version: 0,
            object_time_stamp: 0,
            original_time_stamp: None,
            time_stamp_status: None,
        };

        let mut log_container = LogContainer {
            header: container_header.clone(),
            compression_method: 0,
            uncompressed_data: inner_object_bytes.clone(),
        };

        log_container.header.object_size = log_container.calculate_object_size();

        let mut container_bytes = serialize_log_container(&log_container);
        add_padding(&mut container_bytes);

        let file_stats = FileStatistics {
            statistics_size: 208,
            api_number: 0,
            application_id: 1,
            compression_level: 0,
            application_major: 1,
            application_minor: 0,
            file_size: (208 + container_bytes.len()) as u64,
            uncompressed_file_size: (208 + inner_object_bytes.len()) as u64,
            object_count: 1,
            application_build: 0,
            measurement_start_time: SystemTime {
                year: 2025,
                month: 11,
                day: 22,
                day_of_week: 0,
                hour: 8,
                minute: 30,
                second: 0,
                milliseconds: 0,
            },
            last_object_time: SystemTime {
                year: 2025,
                month: 11,
                day: 22,
                day_of_week: 0,
                hour: 8,
                minute: 30,
                second: 1,
                milliseconds: 0,
            },
        };

        let file_stats_bytes = serialize_file_statistics(&file_stats);

        let mut blf_data = Vec::new();
        blf_data.extend(file_stats_bytes);
        blf_data.extend(container_bytes);

        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        temp_file.write_all(&blf_data).unwrap();
        temp_file.flush().unwrap();

        // Test streaming reader
        let mut reader = StreamingBlfReader::new(temp_file.path()).unwrap();
        assert_eq!(reader.file_stats(), &file_stats);
        assert!(!reader.is_eof());

        let batch = reader.read_next_batch(10).unwrap();
        assert_eq!(batch.len(), 1);

        if let LogObject::CanMessage(parsed_msg) = &batch[0] {
            assert_eq!(parsed_msg.id, can_message.id);
            assert_eq!(parsed_msg.data, can_message.data);
        } else {
            panic!("Expected CanMessage");
        }

        // Test iterator
        let iterator = stream_blf_from_file(temp_file.path(), 5).unwrap();
        let objects: Result<Vec<_>, _> = iterator.collect();
        assert!(objects.is_ok());
        let objects = objects.unwrap();
        assert_eq!(objects.len(), 1);
    }
}
