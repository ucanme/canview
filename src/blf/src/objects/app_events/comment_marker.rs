//! Event comment and global marker object definitions.

use crate::BlfParseResult;
use crate::objects::object_header::ObjectHeader;
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Cursor, Read, Write};

/// Represents a comment for an event (`EVENT_COMMENT`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventComment {
    /// Type of the commented event.
    pub commented_event_type: u32,
    /// The comment text.
    pub text: String,
    /// Timestamp of the message.
    pub timestamp: u64,
}

impl EventComment {
    pub(crate) fn read(cursor: &mut Cursor<&[u8]>, header: &ObjectHeader) -> BlfParseResult<Self> {
        let commented_event_type = cursor.read_u32::<LittleEndian>()?;
        let text_length = cursor.read_u32::<LittleEndian>()? as usize;
        let _reserved = cursor.read_u64::<LittleEndian>()?;
        let mut text_bytes = vec![0; text_length];
        cursor.read_exact(&mut text_bytes)?;
        let text = String::from_utf8_lossy(&text_bytes).to_string();
        Ok(Self {
            commented_event_type,
            text,
            timestamp: header.object_time_stamp,
        })
    }

    pub(crate) fn write<W: Write>(&self, _writer: &mut W) -> BlfParseResult<()> {
        unimplemented!("Writing EventComment is not yet implemented.")
    }
}

/// Represents a global marker (`GLOBAL_MARKER`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalMarker {
    /// Type of the commented event.
    pub commented_event_type: u32,
    /// Foreground color of the marker group.
    pub foreground_color: u32,
    /// Background color of the marker group.
    pub background_color: u32,
    /// Defines whether a marker can be relocated.
    pub is_relocatable: u8,
    /// Group name.
    pub group_name: String,
    /// Marker name.
    pub marker_name: String,
    /// Description text.
    pub description: String,
    /// Timestamp of the message.
    pub timestamp: u64,
}

impl GlobalMarker {
    pub(crate) fn read(cursor: &mut Cursor<&[u8]>, header: &ObjectHeader) -> BlfParseResult<Self> {
        let commented_event_type = cursor.read_u32::<LittleEndian>()?;
        let foreground_color = cursor.read_u32::<LittleEndian>()?;
        let background_color = cursor.read_u32::<LittleEndian>()?;
        let is_relocatable = cursor.read_u8()?;
        let _reserved1 = cursor.read_u8()?;
        let _reserved2 = cursor.read_u16::<LittleEndian>()?;
        let group_name_length = cursor.read_u32::<LittleEndian>()? as usize;
        let marker_name_length = cursor.read_u32::<LittleEndian>()? as usize;
        let description_length = cursor.read_u32::<LittleEndian>()? as usize;
        let _reserved3 = cursor.read_u32::<LittleEndian>()?;
        let _reserved4 = cursor.read_u64::<LittleEndian>()?;

        let mut group_name_bytes = vec![0; group_name_length];
        cursor.read_exact(&mut group_name_bytes)?;
        let group_name = String::from_utf8_lossy(&group_name_bytes).to_string();

        let mut marker_name_bytes = vec![0; marker_name_length];
        cursor.read_exact(&mut marker_name_bytes)?;
        let marker_name = String::from_utf8_lossy(&marker_name_bytes).to_string();

        let mut description_bytes = vec![0; description_length];
        cursor.read_exact(&mut description_bytes)?;
        let description = String::from_utf8_lossy(&description_bytes).to_string();

        Ok(Self {
            commented_event_type,
            foreground_color,
            background_color,
            is_relocatable,
            group_name,
            marker_name,
            description,
            timestamp: header.object_time_stamp,
        })
    }

    pub(crate) fn write<W: Write>(&self, _writer: &mut W) -> BlfParseResult<()> {
        unimplemented!("Writing GlobalMarker is not yet implemented.")
    }
}
