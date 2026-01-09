//! Simple test binary to parse can.blf with debug output
//!
//! Usage: cargo run --bin test_parse_can

use blf::{BlfParser, read_blf_from_file};

fn main() {
    println!("=== Testing can.blf File Parsing ===\n");

    // Enable debug mode by creating parser with_debug    let parser = BlfParser::with_debug();

    // Read the file
    println!("Reading can.blf...\n");
    match read_blf_from_file("can.blf") {
        Ok(result) => {
            println!("✓ Successfully read file\n");

            // Display file statistics
            println!("=== File Statistics ===");
            println!("  File Size: {} bytes", result.file_stats.file_size);
            println!(
                "  Uncompressed Size: {} bytes",
                result.file_stats.uncompressed_file_size
            );
            println!(
                "  Object Count in Stats: {}",
                result.file_stats.object_count
            );
            println!(
                "  Application Build: {}",
                result.file_stats.application_build
            );
            println!();

            // Display parsed objects
            println!("=== Parsed Objects ===");
            println!("  Total Objects Found: {}\n", result.objects.len());

            if result.objects.is_empty() {
                println!("⚠️  WARNING: No objects were parsed!");
                println!(
                    "\nFile statistics show {} objects",
                    result.file_stats.object_count
                );
                println!("But parser returned 0 objects");
                println!("\nThis suggests LogContainer parsing is failing!");
                println!("\nLet's try parsing manually...\n");

                // Try to read the file again and parse manually
                use blf::{ObjectHeader, ObjectType};
                use std::fs;
                use std::io::Cursor;

                match fs::read("can.blf") {
                    Ok(data) => {
                        let mut cursor = Cursor::new(&data[..]);

                        // Skip file statistics (144 bytes)
                        cursor.set_position(144);

                        // Try to read first object
                        println!("Attempting to read first object at offset 144...");
                        match ObjectHeader::read(&mut cursor) {
                            Ok(header) => {
                                println!("✓ Read object header:");
                                println!("  Type: {:?}", header.object_type);
                                println!("  Size: {}", header.object_size);
                                println!("  Timestamp: {}\n", header.object_time_stamp);

                                if header.object_type == ObjectType::LogContainer {
                                    println!("First object is a LogContainer");
                                    println!("Size: {} bytes", header.object_size);
                                    println!("This contains the actual CAN messages!\n");

                                    // Try to read the container
                                    use blf::LogContainer;
                                    match LogContainer::read(&mut cursor, header.clone()) {
                                        Ok(container) => {
                                            println!("✓ Successfully read LogContainer");
                                            println!(
                                                "  Uncompressed data size: {} bytes",
                                                container.uncompressed_data.len()
                                            );
                                            println!(
                                                "  Compression method: {}",
                                                container.compression_method
                                            );
                                            println!();

                                            // Now parse the inner objects with our parser
                                            println!("Parsing inner objects...\n");
                                            match parser.parse(&container.uncompressed_data[..]) {
                                                Ok(inner_objects) => {
                                                    println!(
                                                        "✓ Successfully parsed inner objects!"
                                                    );
                                                    println!(
                                                        "  Total objects: {}\n",
                                                        inner_objects.len()
                                                    );

                                                    if !inner_objects.is_empty() {
                                                        println!("First 10 objects:");
                                                        for (i, obj) in inner_objects
                                                            .iter()
                                                            .take(10)
                                                            .enumerate()
                                                        {
                                                            match obj {
                                                                blf::LogObject::CanMessage(m) => {
                                                                    println!(
                                                                        "  [{}] CAN: ID=0x{:03X}, Ch={}, DLC={}, Data={:02X?}",
                                                                        i,
                                                                        m.id,
                                                                        m.channel,
                                                                        m.dlc,
                                                                        &m.data[..(m.dlc as usize)
                                                                            .min(8)]
                                                                    );
                                                                }
                                                                blf::LogObject::CanMessage2(m) => {
                                                                    println!(
                                                                        "  [{}] CAN2: ID=0x{:03X}, Ch={}, DLC={}, Len={}",
                                                                        i,
                                                                        m.id,
                                                                        m.channel,
                                                                        m.dlc,
                                                                        m.data.len()
                                                                    );
                                                                }
                                                                blf::LogObject::CanFdMessage(m) => {
                                                                    println!(
                                                                        "  [{}] CAN FD: ID=0x{:03X}, Ch={}, DLC={}, Valid={}",
                                                                        i,
                                                                        m.id,
                                                                        m.channel,
                                                                        m.dlc,
                                                                        m.valid_data_bytes
                                                                    );
                                                                }
                                                                blf::LogObject::CanFdMessage64(
                                                                    m,
                                                                ) => {
                                                                    println!(
                                                                        "  [{}] CAN FD64: ID=0x{:03X}, Ch={}, DLC={}, Valid={}",
                                                                        i,
                                                                        m.id,
                                                                        m.channel,
                                                                        m.dlc,
                                                                        m.valid_data_bytes
                                                                    );
                                                                }
                                                                _ => {
                                                                    println!(
                                                                        "  [{}] Other type: {:?}",
                                                                        i, obj
                                                                    );
                                                                }
                                                            }
                                                        }
                                                    } else {
                                                        println!("⚠️  No inner objects found!");
                                                        println!(
                                                            "First 100 bytes of uncompressed data:"
                                                        );
                                                        for (i, chunk) in container
                                                            .uncompressed_data[..100
                                                            .min(container.uncompressed_data.len())]
                                                            .chunks(16)
                                                            .enumerate()
                                                        {
                                                            print!("{:04X}: ", i * 16);
                                                            for byte in chunk {
                                                                print!("{:02X} ", byte);
                                                            }
                                                            println!();
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    println!(
                                                        "✗ Error parsing inner objects: {:?}",
                                                        e
                                                    );
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            println!("✗ Error reading LogContainer: {:?}", e);
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                println!("✗ Error reading object header: {:?}", e);
                            }
                        }
                    }
                    Err(e) => {
                        println!("✗ Error reading file: {:?}", e);
                    }
                }
            } else {
                println!("✓ Successfully parsed {} objects!\n", result.objects.len());

                println!("First 20 objects:");
                for (i, obj) in result.objects.iter().take(20).enumerate() {
                    match obj {
                        blf::LogObject::CanMessage(m) => {
                            println!(
                                "  [{}] CAN: ID=0x{:03X}, Ch={}, DLC={}, Data={:02X?}",
                                i,
                                m.id,
                                m.channel,
                                m.dlc,
                                &m.data[..(m.dlc as usize).min(8)]
                            );
                        }
                        blf::LogObject::CanMessage2(m) => {
                            println!(
                                "  [{}] CAN2: ID=0x{:03X}, Ch={}, DLC={}, Len={}",
                                i,
                                m.id,
                                m.channel,
                                m.dlc,
                                m.data.len()
                            );
                        }
                        blf::LogObject::CanFdMessage(m) => {
                            println!(
                                "  [{}] CAN FD: ID=0x{:03X}, Ch={}, DLC={}, Valid={}",
                                i, m.id, m.channel, m.dlc, m.valid_data_bytes
                            );
                        }
                        blf::LogObject::CanFdMessage64(m) => {
                            println!(
                                "  [{}] CAN FD64: ID=0x{:03X}, Ch={}, DLC={}, Valid={}",
                                i, m.id, m.channel, m.dlc, m.valid_data_bytes
                            );
                        }
                        _ => {
                            println!("  [{}] {:?}", i, std::mem::discriminant(obj));
                        }
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("✗ Error reading file: {:?}", e);
        }
    }
}
