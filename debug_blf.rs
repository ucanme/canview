//! Debug tool to analyze can.blf file and understand why length is 0
//!
//! Usage: cargo run --bin debug_blf

use std::fs::File;
use std::io::{Read, Seek};

fn main() {
    println!("=== BLF File Debug Tool ===\n");

    let filename = "can.blf";
    println!("Opening file: {}\n", filename);

    let mut file = match File::open(filename) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error opening file: {}", e);
            return;
        }
    };

    // Get file size
    let file_size = match file.metadata() {
        Ok(meta) => meta.len(),
        Err(e) => {
            eprintln!("Error getting metadata: {}", e);
            return;
        }
    };

    println!(
        "File size: {} bytes ({} MB)\n",
        file_size,
        file_size / 1_048_576
    );

    // Read first 200 bytes to see the file header
    let mut buffer = vec![0u8; 200.min(file_size as usize)];
    match file.read_exact(&mut buffer) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            return;
        }
    }

    println!("First 200 bytes (hex dump):");
    for (i, chunk) in buffer.chunks(16).enumerate() {
        print!("{:04X}: ", i * 16);
        for (j, byte) in chunk.iter().enumerate() {
            print!("{:02X} ", byte);
            if j == 7 {
                print!(" ");
            }
        }

        // Print ASCII representation
        print!(" | ");
        for byte in chunk {
            if byte.is_ascii_graphic() || *byte == b' ' {
                print!("{}", *byte as char);
            } else {
                print!(".");
            }
        }
        println!();
    }

    // Parse file statistics header
    println!("\n=== File Statistics Header ===\n");

    if &buffer[0..4] == b"LOGG" {
        println!("✓ Signature: LOGG (correct)");
    } else {
        println!("✗ Signature: {:?} (expected LOGG)", &buffer[0..4]);
    }

    let stats_size = u32::from_le_bytes([buffer[4], buffer[5], buffer[6], buffer[7]]);
    println!("Statistics Size: {} bytes", stats_size);

    let crc = u32::from_le_bytes([buffer[8], buffer[9], buffer[10], buffer[11]]);
    println!("CRC: 0x{:08X}", crc);

    let app_id = buffer[12];
    let compression = buffer[13];
    println!("Application ID: {}", app_id);
    println!("Compression Level: {}", compression);

    let major = buffer[14];
    let minor = buffer[15];
    println!("Application Version: {}.{}", major, minor);

    // Read file size from stats
    let stat_file_size = u64::from_le_bytes([
        buffer[24], buffer[25], buffer[26], buffer[27], buffer[28], buffer[29], buffer[30],
        buffer[31],
    ]);
    println!("File Size in Stats: {} bytes", stat_file_size);

    let uncompressed_size = u64::from_le_bytes([
        buffer[32], buffer[33], buffer[34], buffer[35], buffer[36], buffer[37], buffer[38],
        buffer[39],
    ]);
    println!("Uncompressed Size: {} bytes", uncompressed_size);

    let object_count = u32::from_le_bytes([buffer[40], buffer[41], buffer[42], buffer[43]]);
    println!("Object Count: {}", object_count);

    let app_build = u32::from_le_bytes([buffer[44], buffer[45], buffer[46], buffer[47]]);
    println!("Application Build: {}", app_build);

    // Check if this is the problem - object count might be 0!
    if object_count == 0 {
        println!("\n⚠️ WARNING: Object Count is 0!");
        println!("This might indicate:");
        println!("  1. File was not properly finalized");
        println!("  2. File statistics were not updated");
        println!("  3. Different file format than expected");
    }

    // Read timestamp
    println!("\nStart Time:");
    let year = u16::from_le_bytes([buffer[48], buffer[49]]);
    let month = buffer[50];
    let day = buffer[51];
    let hour = buffer[52];
    let minute = buffer[53];
    let second = buffer[54];
    let millis = u16::from_le_bytes([buffer[56], buffer[57]]);
    println!(
        "  {}-{:02}-{:02} {:02}:{:02}:{:02}.{:03}",
        year, month, day, hour, minute, second, millis
    );

    // Now let's try to find the first object
    println!("\n=== Searching for First Object ===\n");

    // The first object should start right after file statistics (at offset stats_size)
    let first_object_offset = stats_size as usize;

    println!(
        "Expected first object offset: {} (0x{:X})",
        first_object_offset, first_object_offset
    );

    // Read more of the file if needed
    let read_size = (first_object_offset + 100).min(file_size as usize);
    if read_size > buffer.len() {
        buffer.resize(read_size, 0);
        file.rewind().unwrap();
        file.read_exact(&mut buffer).unwrap();
    }

    // Check for LOBJ signature at first object offset
    if first_object_offset + 4 <= buffer.len() {
        let sig = &buffer[first_object_offset..first_object_offset + 4];
        if sig == b"LOBJ" {
            println!("✓ Found LOBJ signature at offset {}", first_object_offset);

            // Parse object header
            let sig = u32::from_le_bytes([
                buffer[first_object_offset],
                buffer[first_object_offset + 1],
                buffer[first_object_offset + 2],
                buffer[first_object_offset + 3],
            ]);
            let header_size = u16::from_le_bytes([
                buffer[first_object_offset + 4],
                buffer[first_object_offset + 5],
            ]);
            let header_version = u16::from_le_bytes([
                buffer[first_object_offset + 6],
                buffer[first_object_offset + 7],
            ]);
            let object_size = u32::from_le_bytes([
                buffer[first_object_offset + 8],
                buffer[first_object_offset + 9],
                buffer[first_object_offset + 10],
                buffer[first_object_offset + 11],
            ]);
            let object_type = u32::from_le_bytes([
                buffer[first_object_offset + 12],
                buffer[first_object_offset + 13],
                buffer[first_object_offset + 14],
                buffer[first_object_offset + 15],
            ]);

            println!("\nFirst Object Header:");
            println!("  Signature: 0x{:08X} ({:?})", sig, sig);
            println!("  Header Size: {}", header_size);
            println!("  Header Version: {}", header_version);
            println!("  Object Size: {} bytes", object_size);
            println!("  Object Type: {} (0x{:02X})", object_type, object_type);

            // Object type meanings
            let type_name = match object_type {
                1 => "CAN_MESSAGE",
                2 => "CAN_ERROR",
                10 => "LOG_CONTAINER",
                100 => "CAN_FD_MESSAGE",
                101 => "CAN_FD_MESSAGE_64",
                _ => "UNKNOWN",
            };
            println!("  Type Name: {}", type_name);

            // Check if object_size is 0
            if object_size == 0 {
                println!("\n⚠️ WARNING: Object Size is 0!");
                println!("This indicates the object header is malformed!");
            }
        } else {
            println!(
                "✗ No LOBJ signature found at offset {}",
                first_object_offset
            );
            println!(
                "Found: {:02X} {:02X} {:02X} {:02X}",
                sig[0], sig[1], sig[2], sig[3]
            );

            // Try to find LOBJ in the file
            println!("\nSearching for LOBJ signature...");
            let mut found_at = None;
            for i in 0..buffer.len().saturating_sub(4) {
                if &buffer[i..i + 4] == b"LOBJ" {
                    found_at = Some(i);
                    break;
                }
            }

            if let Some(offset) = found_at {
                println!("Found LOBJ at offset: {} (0x{:X})", offset, offset);
                println!("This suggests file statistics size is incorrect!");
                println!("Expected: {}, Actual gap: {}", stats_size, offset);
            } else {
                println!("No LOBJ signature found in first {} bytes", buffer.len());
            }
        }
    }

    println!("\n=== Analysis Complete ===\n");
    println!("Summary:");
    println!("- File is readable and has correct LOGG signature");
    println!("- File statistics size: {}", stats_size);
    println!("- Object count in stats: {}", object_count);

    if object_count == 0 {
        println!("\n⚠️  ISSUE FOUND: Object count is 0!");
        println!("   This is likely why you see 'length = 0'");
        println!("\nPossible causes:");
        println!("   1. BLF file was not properly closed when written");
        println!("   2. File statistics need to be recalculated");
        println!("   3. The file uses a different format than expected");
    }
}
