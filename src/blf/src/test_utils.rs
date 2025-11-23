//! Contains common helper functions for serialization, used across multiple unit tests.
#![cfg(test)]

use crate::{CanMessage, CanMessage2, CanFdMessage, CanFdMessage64, FileStatistics, LogContainer, ObjectHeader, ObjectType, SystemTime};
use byteorder::{LittleEndian, WriteBytesExt};
use std::io::Write;

/// Helper to write a SystemTime struct to a writer.
pub fn write_system_time(time: &SystemTime, writer: &mut impl Write) {
    writer.write_u16::<LittleEndian>(time.year).unwrap();
    writer.write_u16::<LittleEndian>(time.month).unwrap();
    writer.write_u16::<LittleEndian>(time.day_of_week).unwrap();
    writer.write_u16::<LittleEndian>(time.day).unwrap();
    writer.write_u16::<LittleEndian>(time.hour).unwrap();
    writer.write_u16::<LittleEndian>(time.minute).unwrap();
    writer.write_u16::<LittleEndian>(time.second).unwrap();
    writer.write_u16::<LittleEndian>(time.milliseconds).unwrap();
}

/// Helper to serialize a FileStatistics struct into bytes.
pub fn serialize_file_statistics(stats: &FileStatistics) -> Vec<u8> {
    let mut writer = Vec::new();
    writer.write_u32::<LittleEndian>(0x474F4C42).unwrap(); // Signature "LOGG"
    writer.write_u32::<LittleEndian>(stats.statistics_size).unwrap();
    writer.write_u32::<LittleEndian>(0).unwrap(); // CRC
    writer.write_u8(stats.application_id).unwrap();
    writer.write_u8(2).unwrap(); // compressionLevel
    writer.write_u8(stats.application_major).unwrap();
    writer.write_u8(stats.application_minor).unwrap();
    writer.write_u64::<LittleEndian>(stats.file_size).unwrap();
    writer.write_u64::<LittleEndian>(stats.uncompressed_file_size).unwrap();
    writer.write_u64::<LittleEndian>(stats.object_count).unwrap();
    writer.write_u8(stats.application_build).unwrap();
    writer.write_all(&[0; 3]).unwrap(); // Padding
    write_system_time(&stats.measurement_start_time, &mut writer);
    write_system_time(&stats.last_object_time, &mut writer);

    // Add reserved data
    writer.write_u32::<LittleEndian>(0).unwrap(); // apiNumber
    writer.write_all(&[0; 32]).unwrap(); // reserved
    writer.write_all(&[0; 96]).unwrap(); // restOfHeader

    // Ensure the final size matches statistics_size by padding if necessary.
    if writer.len() < stats.statistics_size as usize {
        let padding_needed = stats.statistics_size as usize - writer.len();
        writer.write_all(&vec![0; padding_needed]).unwrap();
    } else if writer.len() > stats.statistics_size as usize {
        // Truncate to expected size
        writer.truncate(stats.statistics_size as usize);
    }

    writer
}

/// Helper to serialize an ObjectHeader struct into bytes.
pub fn serialize_object_header(header: &ObjectHeader, writer: &mut impl Write) {
    writer.write_u32::<LittleEndian>(header.signature).unwrap();
    writer.write_u16::<LittleEndian>(header.header_size).unwrap();
    writer.write_u16::<LittleEndian>(header.header_version).unwrap();
    writer.write_u32::<LittleEndian>(header.object_size).unwrap();
    writer.write_u32::<LittleEndian>(header.object_type as u32).unwrap();
    writer.write_u32::<LittleEndian>(header.object_flags).unwrap();
    
    if header.header_version == 1 {
        writer.write_u16::<LittleEndian>(0).unwrap(); // client_index
        writer.write_u16::<LittleEndian>(0).unwrap(); // object_version
        writer.write_u64::<LittleEndian>(header.object_time_stamp).unwrap();
    } else if header.header_version == 2 {
        writer.write_u8(header.time_stamp_status.unwrap_or(0)).unwrap();
        writer.write_u8(0).unwrap(); // reserved
        writer.write_u16::<LittleEndian>(0).unwrap(); // object_version
        writer.write_u64::<LittleEndian>(header.object_time_stamp).unwrap();
        writer.write_u64::<LittleEndian>(header.original_time_stamp.unwrap_or(0)).unwrap();
    }
}

/// Helper to serialize a CanMessage object into bytes (including header).
pub fn serialize_can_message(msg: &CanMessage) -> Vec<u8> {
    let mut writer = Vec::new();
    serialize_object_header(&msg.header, &mut writer);
    writer.write_u16::<LittleEndian>(msg.channel).unwrap();
    writer.write_u8(msg.flags).unwrap();
    writer.write_u8(msg.dlc).unwrap();
    writer.write_u32::<LittleEndian>(msg.id).unwrap();
    writer.write_all(&msg.data).unwrap();
    writer
}

/// Helper to serialize a CanMessage2 object into bytes (including header).
pub fn serialize_can_message2(msg: &CanMessage2) -> Vec<u8> {
    let mut writer = Vec::new();
    serialize_object_header(&msg.header, &mut writer);
    writer.write_u16::<LittleEndian>(msg.channel).unwrap();
    writer.write_u8(msg.flags).unwrap();
    writer.write_u8(msg.dlc).unwrap();
    writer.write_u32::<LittleEndian>(msg.id).unwrap();
    writer.write_all(&msg.data).unwrap();
    writer.write_u32::<LittleEndian>(msg.frame_length).unwrap();
    writer.write_u8(msg.bit_count).unwrap();
    writer.write_u8(msg.reserved1).unwrap();
    writer.write_u16::<LittleEndian>(msg.reserved2).unwrap();
    writer
}

/// Helper to serialize a CanFdMessage object into bytes (including header).
pub fn serialize_can_fd_message(msg: &CanFdMessage) -> Vec<u8> {
    let mut writer = Vec::new();
    serialize_object_header(&msg.header, &mut writer);
    writer.write_u16::<LittleEndian>(msg.channel).unwrap();
    writer.write_u8(msg.can_fd_flags).unwrap();
    writer.write_u8(msg.valid_payload_length).unwrap();
    writer.write_u8(msg.arb_bit_count).unwrap();
    writer.write_u8(msg.serial_bit_count).unwrap();
    writer.write_u32::<LittleEndian>(msg.id).unwrap();
    writer.write_all(&msg.data).unwrap();
    writer.write_u32::<LittleEndian>(msg.frame_length).unwrap();
    writer.write_u8(msg.bit_count).unwrap();
    writer.write_u8(msg.dir).unwrap();
    writer.write_u8(msg.edl_brs_esi).unwrap();
    writer.write_u8(msg.reserved1).unwrap();
    writer.write_u32::<LittleEndian>(msg.reserved2).unwrap();
    writer
}

/// Helper to serialize a CanFdMessage64 object into bytes (including header).
pub fn serialize_can_fd_message64(msg: &CanFdMessage64) -> Vec<u8> {
    let mut writer = Vec::new();
    serialize_object_header(&msg.header, &mut writer);
    writer.write_u16::<LittleEndian>(msg.channel).unwrap();
    writer.write_u8(msg.can_fd_flags).unwrap();
    writer.write_u8(msg.valid_payload_length).unwrap();
    writer.write_u8(msg.arb_bit_count).unwrap();
    writer.write_u8(msg.serial_bit_count).unwrap();
    writer.write_u32::<LittleEndian>(msg.id).unwrap();
    writer.write_all(&msg.data).unwrap();
    writer.write_u32::<LittleEndian>(msg.frame_length).unwrap();
    writer.write_u8(msg.bit_count).unwrap();
    writer.write_u8(msg.dir).unwrap();
    writer.write_u8(msg.edl_brs_esi).unwrap();
    writer.write_u8(msg.reserved1).unwrap();
    writer.write_u32::<LittleEndian>(msg.reserved2).unwrap();
    writer
}

/// Helper to serialize a LogContainer into bytes (including header).
pub fn serialize_log_container(container: &LogContainer) -> Vec<u8> {
    let mut writer = Vec::new();
    serialize_object_header(&container.header, &mut writer);
    writer.write_u16::<LittleEndian>(container.compression_method).unwrap();
    writer.write_u16::<LittleEndian>(0).unwrap(); // _reserved1
    writer.write_u32::<LittleEndian>(0).unwrap(); // _reserved2
    writer.write_u32::<LittleEndian>(container.uncompressed_data.len() as u32).unwrap();
    writer.write_u32::<LittleEndian>(0).unwrap(); // _reserved3
    writer.write_all(&container.uncompressed_data).unwrap();
    
    // Ensure the final size matches object_size by padding if necessary.
    let current_size = writer.len();
    let expected_size = container.header.object_size as usize;
    if current_size < expected_size {
        let padding_needed = expected_size - current_size;
        writer.write_all(&vec![0; padding_needed]).unwrap();
    } else if current_size > expected_size {
        // Truncate to expected size
        writer.truncate(expected_size);
    }
    
    writer
}

/// Adds the required padding to a serialized object's byte vector.
pub fn add_padding(data: &mut Vec<u8>) {
    let padding_len = (4 - (data.len() % 4)) % 4;
    if padding_len > 0 {
        data.extend_from_slice(&vec![0; padding_len]);
    }
}