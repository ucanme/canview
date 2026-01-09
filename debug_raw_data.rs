use blf::{read_blf_from_file, LogObject};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <blf_file>", args[0]);
        eprintln!("Example: {} can.blf", args[0]);
        std::process::exit(1);
    }

    let filename = &args[1];
    println!("=== BLF Raw Data Debug Tool ===");
    println!("File: {}\n", filename);

    match read_blf_from_file(filename) {
        Ok(result) => {
            println!("=== File Statistics ===");
            println!(
                "Statistics Size: {} bytes",
                result.file_stats.statistics_size
            );
            println!("API Number: {}", result.file_stats.api_number);
            println!("Application ID: {}", result.file_stats.application_id);
            println!(
                "Application Version: {}.{}.{}",
                result.file_stats.application_major,
                result.file_stats.application_minor,
                result.file_stats.application_build
            );
            println!("File Size: {} bytes", result.file_stats.file_size);
            println!(
                "Uncompressed Size: {} bytes",
                result.file_stats.uncompressed_file_size
            );
            println!("Object Count: {}\n", result.file_stats.object_count);

            println!("=== Log Objects ===");
            println!("Total objects parsed: {}\n", result.objects.len());

            // 显示前30个对象的详细信息
            let limit = 30.min(result.objects.len());

            for (i, obj) in result.objects.iter().take(limit).enumerate() {
                match obj {
                    LogObject::CanMessage(msg) => {
                        println!("[{}] CAN Message:", i);
                        println!("    ID: 0x{:03X} ({} decimal)", msg.id, msg.id);
                        println!("    Channel: {}", msg.channel);
                        println!("    DLC: {}", msg.dlc);
                        println!("    Flags: 0x{:02X}", msg.flags);
                        println!("    Timestamp: {}", msg.header.object_time_stamp);
                        println!("    Header Size: {}", msg.header.header_size);
                        println!("    Data: {:02X?}", msg.data);
                        if msg.data.len() > 0 {
                            let data_str: Vec<String> = msg
                                .data
                                .iter()
                                .take(msg.dlc as usize)
                                .map(|b| format!("{:02X}", b))
                                .collect();
                            println!("    Data (hex): {}", data_str.join(" "));
                        }
                        println!();
                    }
                    LogObject::CanMessage2(msg) => {
                        println!("[{}] CAN Message2:", i);
                        println!("    ID: 0x{:03X}", msg.id);
                        println!("    Channel: {}", msg.channel);
                        println!("    DLC: {} (data len: {})", msg.dlc, msg.data.len());
                        println!("    Data: {:02X?}", msg.data);
                        println!();
                    }
                    LogObject::CanFdMessage(msg) => {
                        println!("[{}] CAN FD Message:", i);
                        println!("    ID: 0x{:03X}", msg.id);
                        println!("    Channel: {}", msg.channel);
                        println!("    DLC: {}", msg.dlc);
                        println!("    Valid Data Bytes: {}", msg.valid_data_bytes);
                        println!("    CAN FD Flags: 0x{:02X}", msg.can_fd_flags);
                        println!("    Data len: {}", msg.data.len());
                        if msg.data.len() > 0 && msg.valid_data_bytes > 0 {
                            println!(
                                "    Data[0..8]: {:02X?}",
                                &msg.data[..msg.data.len().min(8)]
                            );
                        }
                        println!();
                    }
                    LogObject::CanFdMessage64(msg) => {
                        println!("[{}] CAN FD64 Message:", i);
                        println!("    ID: 0x{:08X}", msg.id);
                        println!("    Channel: {}", msg.channel);
                        println!("    DLC: {}", msg.dlc);
                        println!("    Valid Data Bytes: {}", msg.valid_data_bytes);
                        println!("    TX Count: {}", msg.tx_count);
                        println!("    Frame Length: {}", msg.frame_length);
                        println!("    Flags: 0x{:08X}", msg.flags);
                        println!("    BTR CFG ARB: 0x{:08X}", msg.btr_cfg_arb);
                        println!("    BTR CFG DATA: 0x{:08X}", msg.btr_cfg_data);
                        println!("    Bit Count: {}", msg.bit_count);
                        println!("    CRC: 0x{:08X}", msg.crc);
                        println!("    Header Size: {}", msg.header.header_size);
                        println!("    Object Size: {}", msg.header.object_size);
                        println!("    Data len: {}", msg.data.len());

                        if msg.data.len() > 0 {
                            let display_len = msg.data.len().min(16);
                            println!(
                                "    Data[0..{}]: {:02X?}",
                                display_len,
                                &msg.data[..display_len]
                            );

                            // 显示为十六进制字符串
                            let hex_str: Vec<String> = msg
                                .data
                                .iter()
                                .take(display_len)
                                .map(|b| format!("{:02X}", b))
                                .collect();
                            println!("    Data (hex): {}", hex_str.join(" "));
                        } else {
                            println!("    Data: <empty>");
                        }

                        // 检查是否所有数据都是0
                        let all_zero = msg.data.iter().all(|&b| b == 0);
                        if all_zero && msg.data.len() > 0 {
                            println!("    ⚠️  WARNING: All data bytes are 0!");
                        }

                        println!();
                    }
                    LogObject::LinMessage(msg) => {
                        println!("[{}] LIN Message:", i);
                        println!("    ID: {}", msg.id);
                        println!("    Channel: {}", msg.channel);
                        println!("    DLC: {}", msg.dlc);
                        println!("    Data: {:02X?}", msg.data);
                        println!();
                    }
                    _ => {
                        println!("[{}] Other object type\n", i);
                    }
                    _ => {
                        println!(
                            "[{}] Other object type: {:?}\n",
                            i,
                            std::mem::discriminant(obj)
                        );
                    }
                }
            }

            if result.objects.len() > limit {
                println!("... and {} more objects\n", result.objects.len() - limit);
            }

            // 统计分析
            println!("\n=== Statistical Analysis ===");
            let mut non_empty_fd64 = 0;
            let mut empty_fd64 = 0;
            let mut non_zero_id_fd64 = 0;
            let mut total_data_bytes = 0;

            for obj in &result.objects {
                if let LogObject::CanFdMessage64(msg) = obj {
                    if msg.data.len() > 0 {
                        non_empty_fd64 += 1;
                        total_data_bytes += msg.data.len();
                    } else {
                        empty_fd64 += 1;
                    }
                    if msg.id != 0 {
                        non_zero_id_fd64 += 1;
                    }
                }
            }

            println!("CAN FD64 Messages with data: {}", non_empty_fd64);
            println!("CAN FD64 Messages without data: {}", empty_fd64);
            println!("CAN FD64 Messages with non-zero ID: {}", non_zero_id_fd64);
            println!("Total data bytes in CAN FD64: {}", total_data_bytes);

            // 检查前100个对象中是否有任何非零数据
            let mut has_real_data = false;
            for (i, obj) in result.objects.iter().take(100).enumerate() {
                if let LogObject::CanFdMessage64(msg) = obj {
                    if msg.id != 0 || (msg.data.len() > 0 && msg.data.iter().any(|&b| b != 0)) {
                        println!("\n✓ Found first non-trivial message at index {}:", i);
                        println!("  ID: 0x{:08X}", msg.id);
                        println!("  Data: {:02X?}", &msg.data[..msg.data.len().min(16)]);
                        has_real_data = true;
                        break;
                    }
                }
            }

            if !has_real_data {
                println!(
                    "\n⚠️  WARNING: First 100 CAN FD64 messages all have ID=0 and empty/zero data!"
                );
                println!("    This might indicate:");
                println!("    1. File initialization/control messages");
                println!("    2. Parsing error in CanFdMessage64 structure");
                println!("    3. Data is in a different format than expected");
            }
        }
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            std::process::exit(1);
        }
    }
}
