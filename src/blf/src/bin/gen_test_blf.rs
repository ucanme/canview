use byteorder::{LittleEndian, WriteBytesExt};
use std::fs::File;
use std::io::{Cursor, Write};

// --- Minimal structures copied from blf crate ---

#[derive(Debug, Clone, PartialEq)]
pub struct SystemTime {
    pub year: u16,
    pub month: u16,
    pub day_of_week: u16,
    pub day: u16,
    pub hour: u16,
    pub minute: u16,
    pub second: u16,
    pub milliseconds: u16,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FileStatistics {
    pub statistics_size: u32,
    pub api_number: u32,
    pub application_id: u8,
    pub compression_level: u8,
    pub application_major: u8,
    pub application_minor: u8,
    pub file_size: u64,
    pub uncompressed_file_size: u64,
    pub object_count: u64,
    pub application_build: u8,
    pub measurement_start_time: SystemTime,
    pub last_object_time: SystemTime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ObjectType {
    CanMessage = 1,
    LogContainer = 10,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectHeaderBase {
    pub signature: u32,
    pub header_size: u16,
    pub header_version: u16,
    pub object_size: u32,
    pub object_type: ObjectType,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectHeader {
    pub base: ObjectHeaderBase,
    pub object_flags: u32,
    pub client_index: u16,
    pub object_version: u16,
    pub object_time_stamp: u64,
    pub original_time_stamp: Option<u64>,
    pub time_stamp_status: Option<u8>,
    pub reserved: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanMessage {
    pub header: ObjectHeader,
    pub channel: u16,
    pub flags: u8,
    pub dlc: u8,
    pub id: u32,
    pub data: [u8; 8],
}

// --- Serialization functions ---

fn write_system_time(time: &SystemTime, writer: &mut impl Write) {
    writer.write_u16::<LittleEndian>(time.year).unwrap();
    writer.write_u16::<LittleEndian>(time.month).unwrap();
    writer.write_u16::<LittleEndian>(time.day_of_week).unwrap();
    writer.write_u16::<LittleEndian>(time.day).unwrap();
    writer.write_u16::<LittleEndian>(time.hour).unwrap();
    writer.write_u16::<LittleEndian>(time.minute).unwrap();
    writer.write_u16::<LittleEndian>(time.second).unwrap();
    writer.write_u16::<LittleEndian>(time.milliseconds).unwrap();
}

pub fn serialize_file_statistics(stats: &FileStatistics) -> Vec<u8> {
    let mut writer = Vec::new();
    writer.write_u32::<LittleEndian>(0x47474F4C).unwrap(); // "LOGG"
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
        .write_u64::<LittleEndian>(stats.object_count)
        .unwrap();
    writer.write_u8(stats.application_build).unwrap();
    write_system_time(&stats.measurement_start_time, &mut writer);
    write_system_time(&stats.last_object_time, &mut writer);
    writer.write_u32::<LittleEndian>(0).unwrap(); // apiNumber
    writer.write_all(&[0; 32]).unwrap(); // reserved
    writer.write_all(&[0; 96]).unwrap(); // restOfHeader
    writer
}

pub fn serialize_object_header(header: &ObjectHeader, writer: &mut impl Write) {
    writer
        .write_u32::<LittleEndian>(header.base.signature)
        .unwrap();
    writer
        .write_u16::<LittleEndian>(header.base.header_size)
        .unwrap();
    writer
        .write_u16::<LittleEndian>(header.base.header_version)
        .unwrap();
    writer
        .write_u32::<LittleEndian>(header.base.object_size)
        .unwrap();
    writer
        .write_u32::<LittleEndian>(header.base.object_type as u32)
        .unwrap();
    writer
        .write_u32::<LittleEndian>(header.object_flags)
        .unwrap();
    writer.write_u16::<LittleEndian>(0).unwrap(); // client_index
    writer.write_u16::<LittleEndian>(0).unwrap(); // object_version
    writer
        .write_u64::<LittleEndian>(header.object_time_stamp)
        .unwrap();
}

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

pub fn add_padding(data: &mut Vec<u8>) {
    let padding_len = (4 - (data.len() % 4)) % 4;
    if padding_len > 0 {
        data.extend_from_slice(&vec![0; padding_len]);
    }
}

fn main() {
    let start_time = SystemTime {
        year: 2025,
        month: 12,
        day_of_week: 6,
        day: 27,
        hour: 22,
        minute: 0,
        second: 0,
        milliseconds: 0,
    };

    // 1. Create messages for Channel 1 and Channel 2
    let mut inner_data: Vec<u8> = Vec::new();

    // Channel 1: EngineData (0x101 = 257)
    for i in 0..5 {
        let mut data = [0u8; 8];
        let rpm = 1000 + i * 500;
        let temp = 70 + i * 5;
        data[0] = (rpm & 0xFF) as u8;
        data[1] = (rpm >> 8) as u8;
        data[2] = temp as u8;

        let msg = CanMessage {
            header: ObjectHeader {
                base: ObjectHeaderBase {
                    signature: 0x4A424F4C,
                    header_size: 32,
                    header_version: 1,
                    object_size: 48,
                    object_type: ObjectType::CanMessage,
                },
                object_flags: 0,
                client_index: 0,
                object_version: 0,
                object_time_stamp: (i as u64 + 1) * 1000000,
                original_time_stamp: None,
                time_stamp_status: None,
                reserved: 0,
            },
            channel: 1,
            flags: 0,
            dlc: 8,
            id: 0x101,
            data,
        };
        let mut m_bytes = serialize_can_message(&msg);
        add_padding(&mut m_bytes);
        inner_data.extend(m_bytes);
    }

    // Channel 2: BodyStatus (0x201 = 513)
    for i in 0..5 {
        let mut data = [0u8; 8];
        let door_open = if i % 2 == 0 { 1 } else { 0 };
        let window_pos = i * 20;
        data[0] = door_open | (window_pos << 1);

        let msg = CanMessage {
            header: ObjectHeader {
                base: ObjectHeaderBase {
                    signature: 0x4A424F4C,
                    header_size: 32,
                    header_version: 1,
                    object_size: 48,
                    object_type: ObjectType::CanMessage,
                },
                object_flags: 0,
                client_index: 0,
                object_version: 0,
                object_time_stamp: (i as u64 + 6) * 1000000,
                original_time_stamp: None,
                time_stamp_status: None,
                reserved: 0,
            },
            channel: 2,
            flags: 0,
            dlc: 4,
            id: 0x201,
            data,
        };
        let mut m_bytes = serialize_can_message(&msg);
        add_padding(&mut m_bytes);
        inner_data.extend(m_bytes);
    }

    // Also include the original 10 messages (0x100-0x109) on Ch 1 for integration tests
    for i in 0..10 {
        let data = [i as u8; 8];
        let msg = CanMessage {
            header: ObjectHeader {
                base: ObjectHeaderBase {
                    signature: 0x4A424F4C,
                    header_size: 32,
                    header_version: 1,
                    object_size: 48,
                    object_type: ObjectType::CanMessage,
                },
                object_flags: 0,
                client_index: 0,
                object_version: 0,
                object_time_stamp: (i as u64 + 11) * 1000000,
                original_time_stamp: None,
                time_stamp_status: None,
                reserved: 0,
            },
            channel: 1,
            flags: 0,
            dlc: 8,
            id: 0x100 + i as u32,
            data,
        };
        let mut m_bytes = serialize_can_message(&msg);
        add_padding(&mut m_bytes);
        inner_data.extend(m_bytes);
    }

    // 2. Create LogContainer
    let mut container_writer = Vec::new();
    let container_size = (32 + 16 + inner_data.len()) as u32;
    let container_header = ObjectHeader {
        base: ObjectHeaderBase {
            signature: 0x4A424F4C,
            header_size: 32,
            header_version: 1,
            object_size: container_size,
            object_type: ObjectType::LogContainer,
        },
        object_flags: 0,
        client_index: 0,
        object_version: 0,
        object_time_stamp: 0,
        original_time_stamp: None,
        time_stamp_status: None,
        reserved: 0,
    };
    serialize_object_header(&container_header, &mut container_writer);
    container_writer.write_u16::<LittleEndian>(0).unwrap(); // compression_method: 0
    container_writer.write_u16::<LittleEndian>(0).unwrap(); // res1
    container_writer.write_u32::<LittleEndian>(0).unwrap(); // res2
    container_writer
        .write_u32::<LittleEndian>(inner_data.len() as u32)
        .unwrap();
    container_writer.write_u32::<LittleEndian>(0).unwrap(); // res3
    container_writer.extend(inner_data);
    add_padding(&mut container_writer);

    // 3. Create FileStatistics
    let stats = FileStatistics {
        statistics_size: 208,
        api_number: 0,
        application_id: 1,
        compression_level: 0,
        application_major: 1,
        application_minor: 0,
        application_build: 0,
        file_size: (208 + container_writer.len()) as u64,
        uncompressed_file_size: (208 + container_writer.len()) as u64,
        object_count: 21,
        measurement_start_time: start_time.clone(),
        last_object_time: start_time,
    };
    let stats_bytes = serialize_file_statistics(&stats);

    // 4. Combine and write to file
    let mut final_data = Vec::new();
    final_data.extend(stats_bytes);
    final_data.extend(container_writer);

    let mut file = File::create("sample.blf").unwrap();
    file.write_all(&final_data).unwrap();
    println!("Generated sample.blf");
}
