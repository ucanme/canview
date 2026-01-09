//! Simple test to verify can.blf parsing and understand why length might be 0

use blf::read_blf_from_file;

fn main() {
    println!("=== Testing can.blf File Parsing ===\n");

    // Read the BLF file
    println!("Reading can.blf...");
    match read_blf_from_file("can.blf") {
        Ok(result) => {
            println!("✓ Successfully read file\n");

            // Display file statistics
            println!("File Statistics:");
            println!("  File Size: {} bytes", result.file_stats.file_size);
            println!(
                "  Uncompressed Size: {} bytes",
                result.file_stats.uncompressed_file_size
            );
            println!(
                "  Object Count in Stats: {}",
                result.file_stats.object_count
            );

            // Display actual parsed objects
            println!("\nParsed Objects:");
            println!("  Total Objects Found: {}", result.objects.len());

            if result.objects.is_empty() {
                println!("\n⚠️  WARNING: No objects were parsed!");
                println!("\nPossible reasons:");
                println!("  1. LogContainer decompression failed");
                println!("  2. Object parsing logic has issues");
                println!("  3. File format is unexpected");

                println!("\nDebugging info:");
                println!(
                    "  File stats show {} objects",
                    result.file_stats.object_count
                );
                println!("  But parser returned 0 objects");
                println!("  This suggests LogContainer parsing is failing");
            } else {
                println!("\nFirst 10 objects:");
                for (i, obj) in result.objects.iter().take(10).enumerate() {
                    match obj {
                        blf::LogObject::CanMessage(m) => {
                            println!(
                                "  [{}] CAN: ID=0x{:03X}, Ch={}, DLC={}, Data={:02X?}",
                                i, m.id, m.channel, m.dlc, &m.data[..m.dlc as usize.min(8)]
                            );
                        }
                        blf::LogObject::CanMessage2(m) => {
                            println!(
                                "  [{}] CAN2: ID=0x{:03X}, Ch={}, DLC={}, Data={:02X?}",
                                i, m.id, m.channel, m.dlc, &m.data[..m.dlc as usize.min(8)]
                            );
                        }
                        blf::LogObject::CanFdMessage(m) => {
                            println!(
                                "  [{}] CAN FD: ID=0x{:03X}, Ch={}, DLC={}, ValidBytes={}",
                                i, m.id, m.channel, m.dlc, m.valid_data_bytes
                            );
                        }
                        blf::LogObject::CanFdMessage64(m) => {
                            println!(
                                "  [{}] CAN FD64: ID=0x{:03X}, Ch={}, DLC={}, ValidBytes={}",
                                i, m.id, m.channel, m.dlc, m.valid_data_bytes
                            );
                        }
                        blf::LogObject::LinMessage(m) => {
                            println!(
                                "  [{}] LIN: ID=0x{:03X}, Ch={}, DLC={}",
                                i, m.id, m.channel, m.dlc
                            );
                        }
                        _ => {
                            println!("  [{}] Other type", i);
                        }
                    }
                }

                println!("\n✓ Parsing successful!");
                println!("  File has {} total objects", result.objects.len());
            }
        }
        Err(e) => {
            eprintln!("✗ Error reading file: {}", e);
            eprintln!("\nError details:");
            eprintln!("  {:?}", e);
        }
    }
}
