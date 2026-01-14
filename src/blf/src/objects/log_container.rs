//! Log container object definition.

use crate::objects::object_header::ObjectHeaderBase;
use crate::{BlfParseError, BlfParseResult};
use byteorder::{LittleEndian, ReadBytesExt};
use flate2::read::ZlibDecoder;
use std::io::{Cursor, Read};

/// Represents a container for other log objects, which may be compressed (`LOG_CONTAINER`).
#[derive(Debug, Clone)]
pub struct LogContainer {
    /// The header of this log container.
    pub header: ObjectHeaderBase,
    /// The compression method used (0 = None, 2 = zlib).
    pub compression_method: u16,
    /// The uncompressed data.
    pub uncompressed_data: Vec<u8>,
}

impl LogContainer {
    /// Reads and uncompresses a `LogContainer` from a byte stream.
    pub fn read(cursor: &mut Cursor<&[u8]>, header: ObjectHeaderBase) -> BlfParseResult<Self> {
        let compression_method = cursor.read_u16::<LittleEndian>()?;
        let _reserved1 = cursor.read_u16::<LittleEndian>()?;
        let _reserved2 = cursor.read_u32::<LittleEndian>()?;
        let uncompressed_size = cursor.read_u32::<LittleEndian>()? as usize;
        let _reserved3 = cursor.read_u32::<LittleEndian>()?;

        let log_container_specific_fields_size = 16;
        let data_size = (header.object_size as usize)
            .saturating_sub(header.header_size as usize)
            .saturating_sub(log_container_specific_fields_size);

        let mut compressed_data = vec![0; data_size];
        cursor.read_exact(&mut compressed_data)?;

        let uncompressed_data = match compression_method {
            0 => compressed_data,
            2 => {
                let mut decoder = ZlibDecoder::new(&compressed_data[..]);
                let mut uncompressed = Vec::with_capacity(uncompressed_size);
                decoder.read_to_end(&mut uncompressed)?;

                // Debug: Print first 128 bytes of uncompressed data
                if uncompressed.len() > 0 {
                    println!(
                        "=== LogContainer uncompressed data (first 128 bytes) ===sumlen:{}",
                        uncompressed.len()
                    );
                    let dump_len = uncompressed.len().min(128);
                    for i in (0..dump_len).step_by(16) {
                        print!("{:04x}: ", i);
                        for j in i..(i + 16).min(dump_len) {
                            print!("{:02x} ", uncompressed[j]);
                        }
                        println!();
                    }
                    println!("=== End hex dump ===\n");
                }

                uncompressed
            }
            _ => return Err(BlfParseError::UnsupportedCompression(compression_method)),
        };

        Ok(LogContainer {
            header,
            compression_method,
            uncompressed_data,
        })
    }

    /// Calculate the total object size in bytes for this LogContainer
    pub fn calculate_object_size(&self) -> u32 {
        // Object size should be header_size + compressed data size
        // Header size + log container specific fields (compression_method + reserved1 + reserved2 + uncompressed_size + reserved3)
        // + actual data size
        self.header.header_size as u32 + 16 + self.uncompressed_data.len() as u32
    }
}
