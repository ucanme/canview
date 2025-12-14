use blf::{
    CanMessage, FileStatistics, LogContainer, ObjectHeader, ObjectType, SystemTime,
    CanMessage2, CanFdMessage, CanFdMessage64
};
use byteorder::{LittleEndian, WriteBytesExt};
use std::fs::File;
use std::io::Write;
use std::path::Path;

// --- Helper functions copied from test_utils.rs ---

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
    
    // Manual padding calculation to match object_size if needed, 
    // but typically CAN Message is fixed size + data? 
    // V1 Header (32) + Channel(2) + Flags(1) + DLC(1) + ID(4) + Data(8) = 48 bytes.
    // If msg.header.object_size is larger, we pad.
    let current_len = writer.len();
    if current_len < msg.header.object_size as usize {
        let padding = msg.header.object_size as usize - current_len;
        writer.write_all(&vec![0; padding]).unwrap();
    }
    
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

fn main() {
    let filename = "sample.blf";
    println!("Generating {}...", filename);

    let start_time = SystemTime { year: 2025, month: 12, day_of_week: 5, day: 13, hour: 10, minute: 0, second: 0, milliseconds: 0 };
    
    // 1. Create FileStatistics
    let stats_size = 224; // Standard size? test_utils used 208 but let's see. 
    // Header size in read() is read from first u32 (after signature). 
    // test_utils matches logic. 
    // sizeof(BL_OBJ_HEADER_BASE) + sizeof(BL_OBJ_HEADER_V1) + rest?
    // Actually FileStatistics struct mimics the file header part.
    // In test_utils, padding logic suggests we just fill to match statistics_size.
    
    let stats = FileStatistics {
        statistics_size: 208, // Match the reader's expected size (full header with reserved fields)
        // Let's use 144 which is common. Or just enough to cover fields.
        // 4 (Sig) + 4 (Size) + 4 (CRC) + 1 (AppID) + 1 (Comp) + 1 (Maj) + 1 (Min) + 8 (Size) + 8 (Uncomp) + 8 (ObjCount) + 1 (Build) + 3 (Pad) + 16 (Time1) + 16 (Time2) + 4 (Api) + 32 (Res) + 96 (Rest) = ?
        // 4+4+4+1+1+1+1+8+8+8+1+3+16+16 = 76 bytes.
        // + 4 (Api) + 32 (Res) + 96 (Rest) = 208 bytes.
        // So 208 seems correct for full header.
        application_id: 1, // CANalyzer?
        application_major: 1,
        application_minor: 0,
        application_build: 0,
        file_size: 0, // Will update later if possible, or just leave as 0 for now (some tools ignore it)
        uncompressed_file_size: 0,
        object_count: 0,
        measurement_start_time: start_time.clone(),
        last_object_time: start_time.clone(),
    };

    let stats_bytes = serialize_file_statistics(&stats);
    
    // 2. Create some CAN messages
    let mut messages_bytes = Vec::new();
    let mut object_count = 0;

    for i in 0..10 {
        // Create a CAN Message
        let header = ObjectHeader {
            signature: 0x4A424F4C, // LOBJ
            header_size: 32, // V1 header size
            header_version: 1, 
            object_size: 48, // 32 + 16 (body)
            object_type: ObjectType::CanMessage,
            object_flags: 1, // Time One?
            object_time_stamp: (i as u64) * 1000000, // Dummy timestamp
            original_time_stamp: None,
            time_stamp_status: None,
        };

        let msg = CanMessage {
            header,
            channel: 1,
            flags: 0,
            dlc: 8,
            id: 0x100 + i,
            data: [i as u8; 8],
        };
        
        let msg_bytes = serialize_can_message(&msg);
        messages_bytes.extend_from_slice(&msg_bytes);
        object_count += 1;
    }

    // 3. Wrap in LogContainer (Uncompressed)
    let container_header_size = 32; // V1
    let container_extra_size = 16;
    let container_data_size = messages_bytes.len();
    let container_total_size = container_header_size + container_extra_size + container_data_size; 
    // Pad to 4 bytes alignment
    let padding = (4 - (container_total_size % 4)) % 4;
    let final_container_size = container_total_size + padding;
    
    let container_header = ObjectHeader {
        signature: 0x4A424F4C,
        header_size: 32,
        header_version: 1,
        object_size: final_container_size as u32,
        object_type: ObjectType::LogContainer,
        object_flags: 0,
        object_time_stamp: 0,
        original_time_stamp: None,
        time_stamp_status: None,
    };

    let container = LogContainer {
        header: container_header,
        compression_method: 0, // Uncompressed
        uncompressed_data: messages_bytes,
    };
    
    let container_bytes = serialize_log_container(&container);

    // 4. Write to file
    let mut file = File::create(filename).expect("Failed to create file");
    
    // Update stats with correct size/count
    let mut final_stats = stats.clone();
    final_stats.file_size = (stats_bytes.len() + container_bytes.len()) as u64;
    final_stats.uncompressed_file_size = final_stats.file_size; // Since no compression
    final_stats.object_count = object_count as u64; // Count of internal objects? Or count of container?
    // Usually object_count is total number of objects in file (headers).
    // FileStatistics is not counted. LogContainer is 1 object.
    // But LogContainer contains other objects.
    // If strict, maybe 1 (container) + 10 (messages).
    // But let's just write what we have.
    
    // Re-serialize stats
    let final_stats_bytes = serialize_file_statistics(&final_stats);
    
    file.write_all(&final_stats_bytes).expect("Failed to write stats");
    file.write_all(&container_bytes).expect("Failed to write container");

    println!("Successfully generated {}", filename);
}
