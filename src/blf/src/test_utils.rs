//! Contains common helper functions for serialization, used across multiple unit tests.
#![cfg(test)]

use crate::{
    CanFdMessage, CanFdMessage64, CanMessage, CanMessage2, FileStatistics, LogContainer,
    ObjectHeader, ObjectType, SystemTime,
};
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
    writer.write_u32::<LittleEndian>(0x47474F4C).unwrap(); // Signature "LOGG"
    writer
        .write_u32::<LittleEndian>(stats.statistics_size)
        .unwrap();
    writer.write_u32::<LittleEndian>(stats.api_number).unwrap(); // API number
    writer.write_u8(stats.application_id).unwrap();
    writer.write_u8(stats.compression_level).unwrap();
    writer.write_u8(stats.application_major).unwrap();
    writer.write_u8(stats.application_minor).unwrap();
    writer.write_u64::<LittleEndian>(stats.file_size).unwrap();
    writer
        .write_u64::<LittleEndian>(stats.uncompressed_file_size)
        .unwrap();
    writer
        .write_u32::<LittleEndian>(stats.object_count)
        .unwrap();
    writer
        .write_u32::<LittleEndian>(stats.application_build)
        .unwrap();
    write_system_time(&stats.measurement_start_time, &mut writer);
    write_system_time(&stats.last_object_time, &mut writer);

    // Add reserved data (32 bytes for 144-byte headers, more for larger headers)
    let reserved_size = if stats.statistics_size <= 144 {
        32
    } else {
        32 + 96
    };
    writer.write_all(&vec![0u8; reserved_size]).unwrap(); // reserved + restOfHeader

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

/// Helper to serialize an ObjectHeaderBase struct into bytes.
pub fn serialize_object_header_base(header: &crate::objects::object_header::ObjectHeaderBase, writer: &mut impl Write) {
    use crate::objects::object_header::OBJECT_SIGNATURE;
    use byteorder::WriteBytesExt;

    writer.write_u32::<byteorder::LittleEndian>(header.signature).unwrap();
    writer.write_u16::<byteorder::LittleEndian>(header.header_size).unwrap();
    writer.write_u16::<byteorder::LittleEndian>(header.header_version).unwrap();
    writer.write_u32::<byteorder::LittleEndian>(header.object_size).unwrap();
    writer.write_u32::<byteorder::LittleEndian>(header.object_type as u32).unwrap();
}

/// Helper to serialize an ObjectHeader struct into bytes.
pub fn serialize_object_header(header: &crate::objects::object_header::ObjectHeader, writer: &mut impl Write) {
    use crate::objects::object_header::OBJECT_SIGNATURE;
    use byteorder::WriteBytesExt;

    // Write base header first
    serialize_object_header_base(&header.base, writer);

    // Write version-specific fields
    writer.write_u32::<byteorder::LittleEndian>(header.object_flags).unwrap();

    if header.base.header_version == 1 {
        writer.write_u16::<byteorder::LittleEndian>(header.client_index).unwrap();
        writer.write_u16::<byteorder::LittleEndian>(header.object_version).unwrap();
        writer.write_u64::<byteorder::LittleEndian>(header.object_time_stamp).unwrap();
    } else if header.base.header_version == 2 {
        writer.write_u8(header.time_stamp_status.unwrap_or(0)).unwrap();
        writer.write_u8(0).unwrap(); // reserved
        writer.write_u16::<byteorder::LittleEndian>(header.object_version).unwrap();
        writer.write_u64::<byteorder::LittleEndian>(header.object_time_stamp).unwrap();
        writer.write_u64::<byteorder::LittleEndian>(header.original_time_stamp.unwrap_or(0)).unwrap();
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
    writer.write_u8(msg.flags).unwrap();
    writer.write_u8(msg.dlc).unwrap();
    writer.write_u32::<LittleEndian>(msg.id).unwrap();
    writer.write_u32::<LittleEndian>(msg.frame_length).unwrap();
    writer.write_u8(msg.arb_bit_count).unwrap();
    writer.write_u8(msg.can_fd_flags).unwrap();
    writer.write_u8(msg.valid_data_bytes).unwrap();
    writer.write_u8(msg.reserved1).unwrap();
    writer.write_u32::<LittleEndian>(msg.reserved2).unwrap();
    writer.write_all(&msg.data).unwrap();
    writer.write_u32::<LittleEndian>(msg.reserved3).unwrap();
    writer
}

/// Helper to serialize a CanFdMessage64 object into bytes (including header).
pub fn serialize_can_fd_message64(msg: &CanFdMessage64) -> Vec<u8> {
    let mut writer = Vec::new();
    serialize_object_header(&msg.header, &mut writer);
    writer.write_u8(msg.channel).unwrap();
    writer.write_u8(msg.dlc).unwrap();
    writer.write_u8(msg.valid_data_bytes).unwrap();
    writer.write_u8(msg.tx_count).unwrap();
    writer.write_u32::<LittleEndian>(msg.id).unwrap();
    writer.write_u32::<LittleEndian>(msg.frame_length).unwrap();
    writer.write_u32::<LittleEndian>(msg.flags).unwrap();
    writer.write_u32::<LittleEndian>(msg.btr_cfg_arb).unwrap();
    writer.write_u32::<LittleEndian>(msg.btr_cfg_data).unwrap();
    writer
        .write_u32::<LittleEndian>(msg.time_offset_brs_ns)
        .unwrap();
    writer
        .write_u32::<LittleEndian>(msg.time_offset_crc_del_ns)
        .unwrap();
    writer.write_u16::<LittleEndian>(msg.bit_count).unwrap();
    writer.write_u8(msg.dir).unwrap();
    writer.write_u8(msg.ext_data_offset).unwrap();
    writer.write_u32::<LittleEndian>(msg.crc).unwrap();
    writer.write_all(&msg.data).unwrap();
    // Note: ext_data is not serialized in tests for simplicity
    writer
}

/// Helper to serialize a LogContainer into bytes (including header).
pub fn serialize_log_container(container: &LogContainer) -> Vec<u8> {
    let mut writer = Vec::new();
    serialize_object_header_base(&container.header, &mut writer);
    writer
        .write_u16::<LittleEndian>(container.compression_method)
        .unwrap();
    writer.write_u16::<LittleEndian>(0).unwrap(); // _reserved1
    writer.write_u32::<LittleEndian>(0).unwrap(); // _reserved2
    writer
        .write_u32::<LittleEndian>(container.uncompressed_data.len() as u32)
        .unwrap();
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
