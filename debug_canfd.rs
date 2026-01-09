use blf::{read_blf_from_file, LogObject};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <blf_file>", args[0]);
        std::process::exit(1);
    }

    let filename = &args[1];
    println!("=== CAN FD Message Debug Tool ===");
    println!("File: {}\n", filename);

    match read_blf_from_file(filename) {
        Ok(result) => {
            println!("Total objects parsed: {}\n", result.objects.len());

            // Find first 5 CAN messages and show detailed info
            let mut can_count = 0;
            for (i, obj) in result.objects.iter().enumerate() {
                if let LogObject::CanMessage(msg) = obj {
                    if can_count < 5 {
                        println!("=== CAN Message #{} ===", can_count);
                        println!("Index in file: {}", i);
                        println!("Channel: {}", msg.channel);
                        println!("DLC: {}", msg.dlc);
                        println!("ID: 0x{:03X}", msg.id);
                        println!("Flags: 0x{:02X}", msg.flags);
                        println!("Data length: {} bytes", msg.data.len());

                        if msg.data.len() > 0 && msg.dlc > 0 {
                            println!("Data:");
                            for chunk in msg.data.chunks(8) {
                                print!("  ");
                                for (i, byte) in chunk.iter().enumerate() {
                                    print!("{:02X} ", byte);
                                    if (i + 1) % 4 == 0 {
                                        print!(" ");
                                    }
                                }
                                println!();
                            }
                        } else {
                            println!("Data: <empty>");
                        }
                        println!();
                    }
                    can_count += 1;
                }
            }

            // Find first 5 CAN FD64 messages and show detailed info
            let mut fd_count = 0;
            for (i, obj) in result.objects.iter().enumerate() {
                if let LogObject::CanFdMessage64(msg) = obj {
                    if fd_count < 5 {
                        println!("=== CAN FD64 Message #{} ===", fd_count);
                        println!("Index in file: {}", i);
                        println!("Channel: {}", msg.channel);
                        println!("DLC: {}", msg.dlc);
                        println!("Valid Data Bytes: {}", msg.valid_data_bytes);
                        println!("TX Count: {}", msg.tx_count);
                        println!("ID: 0x{:08X}", msg.id);
                        println!("Frame Length: {}", msg.frame_length);
                        println!("Flags: 0x{:08X}", msg.flags);
                        println!("  - EDL (FD frame): {}", msg.is_fd_frame());
                        println!("  - BRS (bit rate switch): {}", msg.has_brs());
                        println!("  - ESI (error state): {}", msg.has_esi());
                        println!("  - DIR: {}", msg.dir);
                        println!("BTR CFG ARB: 0x{:08X}", msg.btr_cfg_arb);
                        println!("BTR CFG DATA: 0x{:08X}", msg.btr_cfg_data);
                        println!("Time offset BRS: {} ns", msg.time_offset_brs_ns);
                        println!("Time offset CRC: {} ns", msg.time_offset_crc_del_ns);
                        println!("Bit count: {}", msg.bit_count);
                        println!("CRC: 0x{:08X}", msg.crc);
                        println!("Data length: {} bytes", msg.data.len());

                        if msg.data.len() > 0 {
                            println!("Data (first 32 bytes):");
                            for chunk in msg.data.chunks(16) {
                                print!("  ");
                                for (i, byte) in chunk.iter().enumerate() {
                                    print!("{:02X} ", byte);
                                    if (i + 1) % 8 == 0 {
                                        print!(" ");
                                    }
                                }
                                println!();
                            }
                        } else {
                            println!("Data: <empty>");
                        }

                        if let Some(ref ext) = msg.ext_data {
                            println!("Extended data present:");
                            println!("  BTR EXT ARB: 0x{:08X}", ext.btr_ext_arb);
                            println!("  BTR EXT DATA: 0x{:08X}", ext.btr_ext_data);
                            println!("  Reserved: {} bytes", ext.reserved.len());
                        }

                        println!();
                    }
                    fd_count += 1;
                }
            }

            println!("=== Statistics ===");
            println!("Total CAN messages: {}", can_count);
            println!("Total CAN FD64 messages: {}", fd_count);

            // Count by valid_data_bytes
            let mut byte_counts = std::collections::HashMap::new();
            for obj in &result.objects {
                if let LogObject::CanFdMessage64(msg) = obj {
                    *byte_counts.entry(msg.valid_data_bytes).or_insert(0) += 1;
                }
            }

            println!("\nDistribution by valid_data_bytes:");
            let mut counts: Vec<_> = byte_counts.iter().collect();
            counts.sort_by_key(|&(k, _)| k);
            for (bytes, count) in counts {
                println!("  {} bytes: {} messages", bytes, count);
            }

            // Count by ID
            let mut id_counts = std::collections::HashMap::new();
            for obj in &result.objects {
                if let LogObject::CanFdMessage64(msg) = obj {
                    *id_counts.entry(msg.id).or_insert(0) += 1;
                }
            }

            println!("\nTop 10 IDs by frequency:");
            let mut id_vec: Vec<_> = id_counts.iter().collect();
            id_vec.sort_by(|a, b| b.1.cmp(a.1));
            for (id, count) in id_vec.iter().take(10) {
                println!("  0x{:08X}: {} messages", id, count);
            }
        }
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            std::process::exit(1);
        }
    }
}
