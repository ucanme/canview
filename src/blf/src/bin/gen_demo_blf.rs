//! Generate a demo BLF file with realistic CAN traffic matching demo.dbc.
//! Produces ~5 seconds of data across IDs 0x100, 0x101, 0x200, 0x201, 0x302, 0x303
//! wrapped in zlib-compressed LogContainers. Output: demo.blf
//!
//! Run with: cargo run --release -p blf --bin gen_demo_blf

use blf::{BlfParseResult, FileStatistics, OBJECT_SIGNATURE, ObjectType, SystemTime};
use byteorder::{LittleEndian, WriteBytesExt};
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::fs::File;
use std::io::Write;

const OBJECT_HEADER_BASE_SIZE: u32 = 16;
const OBJECT_HEADER_V1_SIZE: u32 = 32;
const LOG_CONTAINER_META_SIZE: u32 = 16; // compression_method + reserved + uncompressed_size + reserved

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

fn serialize_file_statistics(stats: &FileStatistics) -> Vec<u8> {
    // Layout matches FileStatistics::read exactly:
    //   signature(4) + statistics_size(4) + api_number(4)
    //   + app_id(1) + comp_lvl(1) + major(1) + minor(1)
    //   + file_size(8) + uncompressed(8) + object_count(4) + application_build(4)
    //   + start_time(16) + last_object_time(16) = 72 bytes of consumed header,
    //   then reserved padding to reach statistics_size bytes total.
    let mut writer = Vec::new();
    writer.write_u32::<LittleEndian>(0x47474F4C).unwrap(); // "LOGG"
    writer.write_u32::<LittleEndian>(stats.statistics_size).unwrap();
    writer.write_u32::<LittleEndian>(stats.api_number).unwrap();
    writer.write_u8(stats.application_id).unwrap();
    writer.write_u8(stats.compression_level).unwrap();
    writer.write_u8(stats.application_major).unwrap();
    writer.write_u8(stats.application_minor).unwrap();
    writer.write_u64::<LittleEndian>(stats.file_size).unwrap();
    writer.write_u64::<LittleEndian>(stats.uncompressed_file_size).unwrap();
    writer.write_u32::<LittleEndian>(stats.object_count).unwrap();
    writer.write_u32::<LittleEndian>(stats.application_build).unwrap();
    write_system_time(&stats.measurement_start_time, &mut writer);
    write_system_time(&stats.last_object_time, &mut writer);
    // Pad with zeros up to statistics_size. Reader skips these remaining
    // bytes after consuming the 72-byte header above.
    let consumed = writer.len() as u32;
    if stats.statistics_size > consumed {
        writer
            .write_all(&vec![0u8; (stats.statistics_size - consumed) as usize])
            .unwrap();
    }
    writer
}

fn serialize_can_message(
    channel: u16,
    flags: u32,
    timestamp_ns: u64,
    can_id: u32,
    data: &[u8; 8],
) -> Vec<u8> {
    let body_size = 2 + 1 + 1 + 4 + 8; // channel + flags + dlc + id + data
    let object_size = OBJECT_HEADER_V1_SIZE + body_size as u32;
    let mut writer = Vec::new();
    // ObjectHeaderBase
    writer.write_u32::<LittleEndian>(OBJECT_SIGNATURE).unwrap();
    writer.write_u16::<LittleEndian>(OBJECT_HEADER_BASE_SIZE as u16).unwrap();
    writer.write_u16::<LittleEndian>(1).unwrap(); // header_version
    writer.write_u32::<LittleEndian>(object_size).unwrap();
    writer.write_u32::<LittleEndian>(ObjectType::CanMessage as u32).unwrap();
    // ObjectHeader V1
    writer.write_u32::<LittleEndian>(flags).unwrap();
    writer.write_u16::<LittleEndian>(0).unwrap(); // client_index
    writer.write_u16::<LittleEndian>(0).unwrap(); // object_version
    writer.write_u64::<LittleEndian>(timestamp_ns).unwrap();
    // Body
    writer.write_u16::<LittleEndian>(channel).unwrap();
    writer.write_u8(0).unwrap(); // msg flags
    writer.write_u8(8).unwrap(); // dlc
    writer.write_u32::<LittleEndian>(can_id).unwrap();
    writer.write_all(data).unwrap();
    writer
}

fn serialize_log_container_zlib(inner_objects_bytes: Vec<u8>) -> Vec<u8> {
    let uncompressed_size = inner_objects_bytes.len() as u32;
    let mut compressed = Vec::new();
    {
        let mut encoder = ZlibEncoder::new(&mut compressed, Compression::default());
        encoder.write_all(&inner_objects_bytes).unwrap();
        encoder.finish().unwrap();
    }
    // The reader's LogContainer::read computes data_size as
    //   object_size - header_size - 16
    // and reads compressed_data from the cursor at the position AFTER
    // ObjectHeaderBase::read (which always consumes 16 bytes). So we use
    // header_size=16 and write ONLY the 16-byte base header + 16 bytes of
    // LogContainer metadata + zlib stream. No V1 extension fields.
    let raw_size = OBJECT_HEADER_BASE_SIZE + LOG_CONTAINER_META_SIZE + compressed.len() as u32;
    let pad = (4 - (raw_size % 4)) % 4;
    let object_size = raw_size + pad;
    let mut writer = Vec::with_capacity(object_size as usize);
    // ObjectHeaderBase (16 bytes, no V1 extension)
    writer.write_u32::<LittleEndian>(OBJECT_SIGNATURE).unwrap();
    writer.write_u16::<LittleEndian>(OBJECT_HEADER_BASE_SIZE as u16).unwrap();
    writer.write_u16::<LittleEndian>(1).unwrap(); // header_version (unused but kept)
    writer.write_u32::<LittleEndian>(object_size).unwrap();
    writer.write_u32::<LittleEndian>(ObjectType::LogContainer as u32).unwrap();
    // LogContainer body (16 bytes metadata)
    writer.write_u16::<LittleEndian>(2).unwrap(); // compression_method = zlib
    writer.write_u16::<LittleEndian>(0).unwrap(); // reserved1
    writer.write_u32::<LittleEndian>(0).unwrap(); // reserved2
    writer.write_u32::<LittleEndian>(uncompressed_size).unwrap();
    writer.write_u32::<LittleEndian>(0).unwrap(); // reserved3
    writer.write_all(&compressed).unwrap();
    writer.write_all(&vec![0u8; pad as usize]).unwrap();
    writer
}

fn main() -> BlfParseResult<()> {
    let out_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "demo.blf".to_string());
    println!("Generating {}...", out_path);

    let start_time = SystemTime {
        year: 2026,
        month: 7,
        day_of_week: 5,
        day: 31,
        hour: 10,
        minute: 0,
        second: 0,
        milliseconds: 0,
    };

    // Statistics header (144 bytes consumed by reader)
    let stats = FileStatistics {
        statistics_size: 144,
        api_number: 0,
        application_id: 1,
        compression_level: 0,
        application_major: 1,
        application_minor: 0,
        file_size: 0,
        uncompressed_file_size: 0,
        object_count: 0,
        application_build: 0,
        measurement_start_time: start_time.clone(),
        last_object_time: SystemTime {
            year: 2026,
            month: 7,
            day_of_week: 5,
            day: 31,
            hour: 10,
            minute: 2,
            second: 0,
            milliseconds: 0,
        },
    };
    let stats_bytes = serialize_file_statistics(&stats);

    // Generate ~5 seconds of traffic at 100ms cycle
    // Pack into containers of ~50 messages each (similar to real Vector BLF)
    let mut all_object_bytes: Vec<u8> = Vec::new();
    let mut total_objects = 0u32;
    let mut rng_state = 0x1234_5678u32;
    let mut next_rand = || {
        // simple xorshift for deterministic demo data
        let mut x = rng_state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        rng_state = x;
        x
    };

    let cycle_ms = 100u64;
    let cycles = 1200u64; // 2 minutes of traffic
    for cycle in 0..cycles {
        let t_ms = cycle * cycle_ms;
        let t_ns = t_ms * 1_000_000;

        // EngineStatus (ID 0x100) — every 100ms
        let mut data = [0u8; 8];
        data[0] = ((next_rand() % 16000) as u16 & 0xFF) as u8; // rpm low
        data[1] = ((next_rand() % 16000) as u16 >> 8) as u8; // rpm high
        data[2] = (40 + (next_rand() % 80) as u8) as u8; // temp
        data[3] = (next_rand() % 256) as u8; // oil pressure
        data[4] = (next_rand() % 256) as u8; // fuel
        data[5] = (next_rand() % 256) as u8; // battery
        data[6] = (next_rand() % 8) as u8; // status flags
        all_object_bytes.extend_from_slice(&serialize_can_message(
            1, 2, t_ns, 0x100, &data,
        ));
        total_objects += 1;

        // VehicleSpeed (ID 0x101) — every 100ms
        let mut data = [0u8; 8];
        let speed = (next_rand() % 3500) as u16;
        data[0] = (speed & 0xFF) as u8;
        data[1] = (speed >> 8) as u8;
        let odo = (cycle as u32 * 7) as u32; // odometer accumulates
        data[2] = (odo & 0xFF) as u8;
        data[3] = ((odo >> 8) & 0xFF) as u8;
        data[4] = ((odo >> 16) & 0xFF) as u8;
        data[5] = ((odo >> 24) & 0xFF) as u8;
        let accel = (next_rand() as i16) / 4;
        data[6] = (accel & 0xFF) as u8;
        data[7] = ((accel >> 8) & 0xFF) as u8;
        all_object_bytes.extend_from_slice(&serialize_can_message(
            1, 2, t_ns + 5_000_000, 0x101, &data,
        ));
        total_objects += 1;

        // ControlCommand (ID 0x200) — every 200ms (even cycles)
        if cycle % 2 == 0 {
            let mut data = [0u8; 8];
            let mode = (cycle / 10) % 4;
            data[0] = (next_rand() % 4) as u8; // AC + lights + wiper
            data[0] |= (mode << 5) as u8 & 0xE0;
            let steer = (next_rand() as i16) / 4;
            data[1] = (steer & 0xFF) as u8;
            data[2] = ((steer >> 8) & 0xFF) as u8;
            data[3] = (next_rand() % 256) as u8; // brake pressure
            all_object_bytes.extend_from_slice(&serialize_can_message(
                2, 2, t_ns + 10_000_000, 0x200, &data,
            ));
            total_objects += 1;
        }

        // TirePressure (ID 0x201) — every 200ms
        if cycle % 2 == 0 {
            let mut data = [0u8; 8];
            for byte in data.iter_mut() {
                *byte = (next_rand() % 256) as u8;
            }
            all_object_bytes.extend_from_slice(&serialize_can_message(
                3, 2, t_ns + 15_000_000, 0x201, &data,
            ));
            total_objects += 1;
        }

        // BrakeStatus (ID 0x302) — every 50ms (2x per cycle)
        for sub in 0..2 {
            let mut data = [0u8; 8];
            data[0] = (next_rand() % 8) as u8; // status flags + 9-bit pressure low
            data[1] = (next_rand() % 512) as u8 & 0x7F; // pressure high bits
            all_object_bytes.extend_from_slice(&serialize_can_message(
                3, 2, t_ns + sub * 50_000_000, 0x302, &data,
            ));
            total_objects += 1;
        }
    }

    // Pack into LogContainers of 50 messages each.
    let mut file_body = Vec::new();
    let per_container = 50usize;
    let messages: Vec<Vec<u8>> = all_object_bytes
        .chunks(serialize_can_message(0, 0, 0, 0, &[0; 8]).len())
        .map(|c| c.to_vec())
        .collect();
    let mut container_count = 0u32;
    for chunk in messages.chunks(per_container) {
        let mut inner_bytes = Vec::new();
        for msg in chunk {
            inner_bytes.extend_from_slice(msg);
        }
        let container_bytes = serialize_log_container_zlib(inner_bytes);
        file_body.extend_from_slice(&container_bytes);
        container_count += 1;
    }

    // Update stats
    let mut final_stats = stats.clone();
    final_stats.file_size = (stats_bytes.len() + file_body.len()) as u64;
    final_stats.uncompressed_file_size = final_stats.file_size;
    final_stats.object_count = container_count; // count of top-level objects

    let mut file = File::create(&out_path).expect("Failed to create file");
    file.write_all(&serialize_file_statistics(&final_stats))
        .expect("write stats");
    file.write_all(&file_body).expect("write body");

    println!(
        "Successfully generated {}: {} objects across {} containers, {} bytes",
        out_path,
        total_objects,
        container_count,
        stats_bytes.len() + file_body.len()
    );
    Ok(())
}
