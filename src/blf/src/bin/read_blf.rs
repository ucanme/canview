use blf::{LogObject, read_blf_from_file};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <blf_file>", args[0]);
        eprintln!("Example: {} can.blf", args[0]);
        std::process::exit(1);
    }

    let filename = &args[1];
    println!("Reading BLF file: {}", filename);

    match read_blf_from_file(filename) {
        Ok(result) => {
            println!("\n=== File Statistics ===");
            println!(
                "Statistics Size: {} bytes",
                result.file_stats.statistics_size
            );
            println!("API Number: {}", result.file_stats.api_number);
            println!();
            println!("Application Information:");
            println!("  Application ID: {}", result.file_stats.application_id);
            println!(
                "  Compression Level: {}",
                result.file_stats.compression_level
            );
            println!(
                "  Version: {}.{}.{}",
                result.file_stats.application_major,
                result.file_stats.application_minor,
                result.file_stats.application_build
            );
            println!();
            println!("File Information:");
            println!("  File Size: {} bytes", result.file_stats.file_size);
            println!(
                "  Uncompressed Size: {} bytes",
                result.file_stats.uncompressed_file_size
            );
            println!("  Object Count: {}", result.file_stats.object_count);
            println!();
            println!("Measurement Time:");
            println!(
                "  Start Time: {:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:03}",
                result.file_stats.measurement_start_time.year,
                result.file_stats.measurement_start_time.month,
                result.file_stats.measurement_start_time.day,
                result.file_stats.measurement_start_time.hour,
                result.file_stats.measurement_start_time.minute,
                result.file_stats.measurement_start_time.second,
                result.file_stats.measurement_start_time.milliseconds
            );
            println!(
                "  End Time: {:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:03}",
                result.file_stats.last_object_time.year,
                result.file_stats.last_object_time.month,
                result.file_stats.last_object_time.day,
                result.file_stats.last_object_time.hour,
                result.file_stats.last_object_time.minute,
                result.file_stats.last_object_time.second,
                result.file_stats.last_object_time.milliseconds
            );

            println!("\n=== Log Objects ===");
            println!("Total objects parsed: {}", result.objects.len());

            // Show first 20 objects
            let limit = 20.min(result.objects.len());
            for (i, obj) in result.objects.iter().take(limit).enumerate() {
                match obj {
                    LogObject::CanMessage(msg) => {
                        println!(
                            "[{}] CAN Message: ID={:#06x}, Channel={}, DLC={}, Data={:02x?}",
                            i,
                            msg.id,
                            msg.channel,
                            msg.dlc,
                            &msg.data[..msg.dlc as usize]
                        );
                    }
                    LogObject::CanMessage2(msg) => {
                        println!(
                            "[{}] CAN Message2: ID={:#06x}, Channel={}, DLC={}, Data={:02x?}",
                            i, msg.id, msg.channel, msg.dlc, msg.data
                        );
                    }
                    LogObject::CanFdMessage(msg) => {
                        println!(
                            "[{}] CAN FD Message: ID={:#06x}, Channel={}, Length={}, Data={:02x?}",
                            i,
                            msg.id,
                            msg.channel,
                            msg.valid_data_bytes,
                            &msg.data[..(msg.valid_data_bytes as usize).min(msg.data.len())]
                        );
                    }
                    LogObject::CanFdMessage64(msg) => {
                        println!(
                            "[{}] CAN FD64 Message: ID={:#06x}, Channel={}, Length={}, Data={:02x?}",
                            i,
                            msg.id,
                            msg.channel,
                            msg.valid_data_bytes,
                            &msg.data[..(msg.valid_data_bytes as usize).min(msg.data.len())]
                        );
                    }
                    LogObject::LinMessage(msg) => {
                        println!(
                            "[{}] LIN Message: ID={}, Channel={}, DLC={}, Data={:02x?}",
                            i,
                            msg.id,
                            msg.channel,
                            msg.dlc,
                            &msg.data[..msg.dlc as usize]
                        );
                    }
                    LogObject::AppTrigger(_) => {
                        println!("[{}] AppTrigger", i);
                    }
                    LogObject::EventComment(_) => {
                        println!("[{}] EventComment", i);
                    }
                    LogObject::GlobalMarker(_) => {
                        println!("[{}] GlobalMarker", i);
                    }
                    LogObject::Unhandled {
                        object_type,
                        timestamp: _,
                        data: _,
                    } => {
                        println!("[!] Unhandled object type: {:?}", object_type);
                    }
                    _ => {
                        println!("[{}] Other object type", i);
                    }
                }
            }

            if result.objects.len() > limit {
                println!("... and {} more objects", result.objects.len() - limit);
            }

            println!("\n=== Summary ===");
            let mut can_count = 0;
            let mut canfd_count = 0;
            let mut lin_count = 0;
            let mut other_count = 0;

            for obj in &result.objects {
                match obj {
                    LogObject::CanMessage(_) | LogObject::CanMessage2(_) => can_count += 1,
                    LogObject::CanFdMessage(_) | LogObject::CanFdMessage64(_) => canfd_count += 1,
                    LogObject::LinMessage(_) => lin_count += 1,
                    _ => other_count += 1,
                }
            }

            println!("CAN Messages: {}", can_count);
            println!("CAN FD Messages: {}", canfd_count);
            println!("LIN Messages: {}", lin_count);
            println!("Other Objects: {}", other_count);
        }
        Err(e) => {
            eprintln!("Error parsing BLF file: {}", e);
            std::process::exit(1);
        }
    }
}
