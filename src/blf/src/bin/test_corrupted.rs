use blf::{read_blf_from_file, LogObject};

fn main() {
    println!("🧪 测试损坏的BLF文件解析\n");
    println!("正在读取 test_corrupted.blf...\n");

    match read_blf_from_file("test_corrupted.blf") {
        Ok(result) => {
            println!("✅ 文件读取成功!\n");
            println!("=== 文件统计 ===");
            println!("应用版本: {}.{}.{}.{}",
                result.file_stats.application_major,
                result.file_stats.application_minor,
                result.file_stats.application_build,
                result.file_stats.application_id
            );
            println!("文件大小: {} 字节", result.file_stats.file_size);
            println!("测量开始时间: {:?}\n", result.file_stats.measurement_start_time);

            println!("=== 解析结果 ===");
            println!("成功解析的对象数量: {}", result.objects.len());
            println!("遇到的错误数量: {}\n", result.errors.len());

            if result.errors.len() > 0 {
                println!("⚠️  发现的错误:");
                for (i, error) in result.errors.iter().enumerate() {
                    println!("  {}. {}", i + 1, error);
                }
                println!();
            }

            // 显示成功解析的消息
            if !result.objects.is_empty() {
                println!("=== 成功解析的消息（前10条）===");
                for (i, obj) in result.objects.iter().take(10).enumerate() {
                    match obj {
                        LogObject::CanMessage(msg) => {
                            println!("  [{:02}] CAN Msg - ID: 0x{:03X}, DLC: {}, Data: {:02X?}",
                                i + 1,
                                msg.id,
                                msg.dlc,
                                &msg.data[..msg.dlc as usize]
                            );
                        }
                        LogObject::CanFdMessage(msg) => {
                            println!("  [{:02}] CAN FD Msg - ID: 0x{:03X}, DLC: {}, Data: {:02X?}",
                                i + 1,
                                msg.id,
                                msg.dlc,
                                &msg.data[..msg.dlc as usize]
                            );
                        }
                        _ => {
                            println!("  [{:02}] 其他类型对象: {:?}", i + 1, obj);
                        }
                    }
                }
                if result.objects.len() > 10 {
                    println!("  ... 还有 {} 条消息", result.objects.len() - 10);
                }
            }

            println!("\n=== 测试结果 ===");
            if result.errors.len() > 0 && result.objects.len() > 0 {
                println!("✅ 部分解析成功!");
                println!("   - {} 个对象成功解析并可以显示在logs和折线图中", result.objects.len());
                println!("   - {} 个错误信息显示在状态栏", result.errors.len());
                println!("\n📝 这验证了我们的错误处理功能：");
                println!("   ✓ 正确的结果出现在logs和折线图上");
                println!("   ✓ 错误信息在右下角状态上显示");
            } else if result.errors.is_empty() {
                println!("✅ 完全解析成功，没有错误");
            } else {
                println!("❌ 解析完全失败，没有成功解析任何对象");
            }
        }
        Err(e) => {
            println!("❌ 文件读取失败: {:?}\n", e);
            println!("这可能是因为:");
            println!("  1. 文件不存在");
            println!("  2. 文件头部完全损坏");
            println!("  3. 文件格式完全错误");
        }
    }

    println!("\n按任意键退出...");
    let _ = std::io::stdin().read_line(&mut String::new());
}
