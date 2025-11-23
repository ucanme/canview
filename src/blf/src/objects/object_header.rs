//! Defines the ObjectHeader struct for BLF log objects.

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Cursor, Write};
use crate::error::{BlfParseError, BlfParseResult};
use super::object_type::ObjectType;

/// Represents the common header for all BLF log objects (V1 and V2).
#[derive(Debug, Clone)]
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
            let _client_index = cursor.read_u16::<LittleEndian>()?;
            let _object_version = cursor.read_u16::<LittleEndian>()?;
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
        writer.write_u32::<LittleEndian>(self.object_type as u32)?; // Cast enum to u32

        if self.header_version == 1 {
            writer.write_u32::<LittleEndian>(self.object_flags)?;
            writer.write_u16::<LittleEndian>(0)?; // _client_index
            writer.write_u16::<LittleEndian>(0)?; // _object_version
            writer.write_u64::<LittleEndian>(self.object_time_stamp)?;
        } else if self.header_version == 2 {
            writer.write_u32::<LittleEndian>(self.object_flags)?;
            writer.write_u8(self.time_stamp_status.unwrap_or(0))?; // time_stamp_status
            writer.write_u8(0)?; // _reserved
            writer.write_u16::<LittleEndian>(0)?; // _object_version
            writer.write_u64::<LittleEndian>(self.object_time_stamp)?;
            writer.write_u64::<LittleEndian>(self.original_time_stamp.unwrap_or(0))?; // original_time_stamp
        } else {
            return Err(BlfParseError::UnknownHeaderVersion(self.header_version));
        }
        Ok(())
    }
}
