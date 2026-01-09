//! 综合测试工具 - BLF 解析和界面功能测试
//!
//! 这个工具用于：
//! 1. 测试 BLF 文件解析功能
//! 2. 生成测试用的 BLF 文件
//! 3. 验证各种 CAN 消息类型的显示
//! 4. 检查 DLC 和 Data 显示是否正确

use blf::{
    CanFdMessage, CanFdMessage64, CanMessage, CanMessage2, FileStatistics, LogContainer, LogObject,
    ObjectHeader, ObjectType, SystemTime,
};
use std::fs::File;
use std::io::Write;

fn main() {
    println!("=== BLF 解析和界面功能综合测试工具 ===\n");

    // 测试菜单
    println!("请选择测试功能：");
    println!("1. 生成测试 BLF 文件（包含所有消息类型）");
    println!("2. 解析现有 BLF 文件并显示信息");
    println!("3. 验证 CAN FD Message64 结构");
    println!("4. 运行所有单元测试");
    println!("5. 生成界面测试数据");
    println!("0. 退出");

    print!("\n请输入选项 (0-5): ");
    std::io::stdout().flush().unwrap();

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    match input.trim() {
        "1" => generate_test_blf(),
        "2" => parse_blf_file(),
        "3" => verify_can_fd_64(),
        "4" => run_unit_tests(),
        "5" => generate_ui_test_data(),
        "0" => println!("退出程序"),
        _ => println!("无效选项"),
    }
}

/// 生成包含所有消息类型的测试 BLF 文件
fn generate_test_blf() {
    println!("\n=== 生成测试 BLF 文件 ===\n");

    let filename = "test_all_messages.blf";
    let mut file = File::create(filename).expect("无法创建文件");

    // 1. 写入 FileStatistics
    let stats = create_test_file_statistics();
    write_file_statistics(&mut file, &stats);
    println!("✓ 写入 FileStatistics");

    // 2. 生成各种测试消息
    let messages = generate_test_messages();

    // 3. 创建 LogContainer
    let container_data = serialize_messages_to_container(&messages);

    // 4. 写入 LogContainer
    file.write_all(&container_data).expect("写入失败");

    println!("\n✓ 成功生成测试文件: {}", filename);
    println!("  包含 {} 个消息对象", messages.len());
    println!("  文件大小: {} 字节", container_data.len());

    // 显示消息摘要
    println!("\n消息类型统计:");
    for msg in &messages {
        match msg {
            LogObject::CanMessage(_) => println!("  - CAN Message (标准 8 字节)"),
            LogObject::CanMessage2(_) => println!("  - CAN Message2 (扩展)"),
            LogObject::CanFdMessage(_) => println!("  - CAN FD Message"),
            LogObject::CanFdMessage64(_) => println!("  - CAN FD Message64"),
            _ => println!("  - 其他类型"),
        }
    }
}

/// 创建测试用的 FileStatistics
fn create_test_file_statistics() -> FileStatistics {
    FileStatistics {
        statistics_size: 144,
        api_number: 0,
        application_id: 1,
        compression_level: 0,
        application_major: 1,
        application_minor: 0,
        application_build: 100,
        file_size: 0, // 会在写入时更新
        uncompressed_file_size: 0,
        object_count: 0,
        measurement_start_time: SystemTime {
            year: 2025,
            month: 1,
            day: 15,
            hour: 10,
            minute: 30,
            second: 0,
            milliseconds: 0,
        },
        last_object_time: SystemTime {
            year: 2025,
            month: 1,
            day: 15,
            hour: 10,
            minute: 30,
            second: 10,
            milliseconds: 0,
        },
    }
}

/// 写入 FileStatistics 到文件
fn write_file_statistics(file: &mut File, stats: &FileStatistics) {
    use byteorder::{LittleEndian, WriteBytesExt};

    file.write_u32::<LittleEndian>(0x47474F4C).unwrap(); // "LOGG"
    file.write_u32::<LittleEndian>(stats.statistics_size)
        .unwrap();
    file.write_u32::<LittleEndian>(0).unwrap(); // CRC
    file.write_u8(stats.application_id).unwrap();
    file.write_u8(stats.compression_level).unwrap();
    file.write_u8(stats.application_major).unwrap();
    file.write_u8(stats.application_minor).unwrap();

    // 填充到偏移量 0x10
    for _ in 0..8 {
        file.write_u8(0).unwrap();
    }

    file.write_u64::<LittleEndian>(stats.file_size).unwrap();
    file.write_u64::<LittleEndian>(stats.uncompressed_file_size)
        .unwrap();
    file.write_u32::<LittleEndian>(stats.object_count).unwrap();
    file.write_u32::<LittleEndian>(stats.application_build)
        .unwrap();

    // 写入时间戳
    write_system_time(file, &stats.measurement_start_time);
    write_system_time(file, &stats.last_object_time);

    // 填充剩余字节到 144
    let current_size = 0x50 + 16; // 固定部分 + 2个时间戳
    let padding = 144 - current_size;
    for _ in 0..padding {
        file.write_u8(0).unwrap();
    }
}

fn write_system_time(file: &mut File, time: &SystemTime) {
    file.write_u16::<LittleEndian>(time.year).unwrap();
    file.write_u8(time.month).unwrap();
    file.write_u8(time.day).unwrap();
    file.write_u8(time.hour).unwrap();
    file.write_u8(time.minute).unwrap();
    file.write_u8(time.second).unwrap();
    file.write_u16::<LittleEndian>(time.milliseconds).unwrap();
    file.write_u16::<LittleEndian>(0).unwrap(); // reserved
}

/// 生成各种测试消息
fn generate_test_messages() -> Vec<LogObject> {
    let mut messages = Vec::new();
    let mut timestamp = 1_000_000_000u64; // 1秒

    // 1. 标准 CAN 消息 (8 字节)
    for i in 0..3 {
        messages.push(LogObject::CanMessage(CanMessage {
            header: create_test_header(ObjectType::CanMessage, 48, timestamp),
            channel: 1,
            flags: 0,
            dlc: 8,
            id: 0x100 + i,
            data: [
                i as u8,
                i as u8 + 1,
                i as u8 + 2,
                i as u8 + 3,
                i as u8 + 4,
                i as u8 + 5,
                i as u8 + 6,
                i as u8 + 7,
            ],
        }));
        timestamp += 10_000_000;
    }

    // 2. CAN Message2 (可变长度)
    messages.push(LogObject::CanMessage2(CanMessage2 {
        header: create_test_header(ObjectType::CanMessage2, 56, timestamp),
        channel: 1,
        flags: 0,
        dlc: 4,
        id: 0x200,
        data: vec![0x11, 0x22, 0x33, 0x44],
        frame_length: 1000,
        bit_count: 64,
        reserved1: 0,
        reserved2: 0,
    }));
    timestamp += 10_000_000;

    // 3. CAN FD Message (不同 DLC)
    for dlc in [8, 12, 16, 20, 32, 64].iter() {
        messages.push(LogObject::CanFdMessage(CanFdMessage {
            header: create_test_header(ObjectType::CanFdMessage, 96, timestamp),
            channel: 2,
            flags: 0,
            dlc: dlc_to_dlc_value(*dlc),
            id: 0x300 + *dlc as u32,
            frame_length: 2000,
            arb_bit_count: 32,
            can_fd_flags: 0x07, // EDL + BRS + ESI
            valid_data_bytes: *dlc as u8,
            reserved1: 0,
            reserved2: 0,
            data: {
                let mut data = [0u8; 64];
                for (i, b) in data.iter_mut().enumerate() {
                    *b = (i % 256) as u8;
                }
                data
            },
            reserved3: 0,
        }));
        timestamp += 10_000_000;
    }

    // 4. CAN FD Message64 (最重要的测试)
    for (dlc, valid_bytes) in [(8, 8), (15, 64), (9, 12), (13, 32)].iter() {
        messages.push(LogObject::CanFdMessage64(CanFdMessage64 {
            header: create_test_header(ObjectType::CanFdMessage64, 120 + valid_bytes, timestamp),
            channel: 3,
            dlc: *dlc,
            valid_data_bytes: *valid_bytes,
            tx_count: 0,
            id: 0x400 + *dlc as u32,
            frame_length: 3000,
            flags: 0x7000,           // EDL + BRS + ESI
            btr_cfg_arb: 0x001C0091, // 示例位时序配置
            btr_cfg_data: 0x001C0011,
            time_offset_brs_ns: 1000,
            time_offset_crc_del_ns: 2000,
            bit_count: 500,
            dir: 1, // TX
            ext_data_offset: 0,
            crc: 0x12345678,
            data: {
                let mut data = vec![0u8; *valid_bytes];
                for (i, b) in data.iter_mut().enumerate() {
                    *b = ((i * 2) % 256) as u8;
                }
                data
            },
            ext_data: None,
        }));
        timestamp += 10_000_000;
    }

    messages
}

fn create_test_header(obj_type: ObjectType, size: u32, timestamp: u64) -> ObjectHeader {
    ObjectHeader {
        signature: 0x4A424F4C, // "LOBJ"
        header_size: 32,
        header_version: 1,
        object_size: size,
        object_type: obj_type,
        object_flags: 0x02, // TimeOneNans
        object_time_stamp: timestamp,
        original_time_stamp: None,
        time_stamp_status: None,
    }
}

fn dlc_to_dlc_value(len: u8) -> u8 {
    match len {
        0..=8 => len,
        12 => 9,
        16 => 10,
        20 => 11,
        24 => 12,
        32 => 13,
        48 => 14,
        64 => 15,
        _ => len,
    }
}

/// 将消息序列化为 LogContainer
fn serialize_messages_to_container(messages: &[LogObject]) -> Vec<u8> {
    use byteorder::{LittleEndian, WriteBytesExt};

    let mut buffer = Vec::new();

    // LogContainer Header
    let container_header = ObjectHeader {
        signature: 0x4A424F4C,
        header_size: 32,
        header_version: 1,
        object_size: 0, // 稍后计算
        object_type: ObjectType::LogContainer,
        object_flags: 0x02,
        object_time_stamp: 0,
        original_time_stamp: None,
        time_stamp_status: None,
    };

    // 写入 container header
    buffer
        .write_u32::<LittleEndian>(container_header.signature)
        .unwrap();
    buffer
        .write_u16::<LittleEndian>(container_header.header_size)
        .unwrap();
    buffer
        .write_u16::<LittleEndian>(container_header.header_version)
        .unwrap();
    buffer
        .write_u32::<LittleEndian>(container_header.object_size)
        .unwrap();
    buffer
        .write_u32::<LittleEndian>(container_header.object_type as u32)
        .unwrap();
    buffer
        .write_u32::<LittleEndian>(container_header.object_flags)
        .unwrap();
    buffer.write_u16::<LittleEndian>(0).unwrap(); // client_index
    buffer.write_u16::<LittleEndian>(0).unwrap(); // object_version
    buffer
        .write_u64::<LittleEndian>(container_header.object_time_stamp)
        .unwrap();

    // LogContainer 特定字段
    buffer.write_u16::<LittleEndian>(0).unwrap(); // compression_method (无压缩)
    buffer.write_u16::<LittleEndian>(0).unwrap(); // reserved1
    buffer.write_u32::<LittleEndian>(0).unwrap(); // reserved2

    // 计算数据大小
    let data_start_pos = buffer.len() + 8; // +4 for uncompressed_size, +4 for reserved3
    let mut data_buffer = Vec::new();

    // 序列化每个消息
    for msg in messages {
        serialize_message(&mut data_buffer, msg);
    }

    // 填充到 4 字节对齐
    while data_buffer.len() % 4 != 0 {
        data_buffer.push(0);
    }

    buffer
        .write_u32::<LittleEndian>(data_buffer.len() as u32)
        .unwrap();
    buffer.write_u32::<LittleEndian>(0).unwrap(); // reserved3
    buffer.extend_from_slice(&data_buffer);

    // 更新 object_size
    let object_size = buffer.len() as u32;
    buffer[12..16].copy_from_slice(&object_size.to_le_bytes());

    buffer
}

fn serialize_message(buffer: &mut Vec<u8>, msg: &LogObject) {
    use byteorder::{LittleEndian, WriteBytesExt};

    match msg {
        LogObject::CanMessage(m) => {
            // Header
            write_object_header(buffer, &m.header);
            // Body
            buffer.write_u16::<LittleEndian>(m.channel).unwrap();
            buffer.write_u8(m.flags).unwrap();
            buffer.write_u8(m.dlc).unwrap();
            buffer.write_u32::<LittleEndian>(m.id).unwrap();
            buffer.extend_from_slice(&m.data);
        }
        LogObject::CanMessage2(m) => {
            write_object_header(buffer, &m.header);
            buffer.write_u16::<LittleEndian>(m.channel).unwrap();
            buffer.write_u8(m.flags).unwrap();
            buffer.write_u8(m.dlc).unwrap();
            buffer.write_u32::<LittleEndian>(m.id).unwrap();
            buffer.extend_from_slice(&m.data);
            buffer.write_u32::<LittleEndian>(m.frame_length).unwrap();
            buffer.write_u8(m.bit_count).unwrap();
            buffer.write_u8(m.reserved1).unwrap();
            buffer.write_u16::<LittleEndian>(m.reserved2).unwrap();
        }
        LogObject::CanFdMessage(m) => {
            write_object_header(buffer, &m.header);
            buffer.write_u16::<LittleEndian>(m.channel).unwrap();
            buffer.write_u8(m.flags).unwrap();
            buffer.write_u8(m.dlc).unwrap();
            buffer.write_u32::<LittleEndian>(m.id).unwrap();
            buffer.write_u32::<LittleEndian>(m.frame_length).unwrap();
            buffer.write_u8(m.arb_bit_count).unwrap();
            buffer.write_u8(m.can_fd_flags).unwrap();
            buffer.write_u8(m.valid_data_bytes).unwrap();
            buffer.write_u8(m.reserved1).unwrap();
            buffer.write_u32::<LittleEndian>(m.reserved2).unwrap();
            buffer.extend_from_slice(&m.data);
            buffer.write_u32::<LittleEndian>(m.reserved3).unwrap();
        }
        LogObject::CanFdMessage64(m) => {
            write_object_header(buffer, &m.header);
            buffer.write_u8(m.channel).unwrap();
            buffer.write_u8(m.dlc).unwrap();
            buffer.write_u8(m.valid_data_bytes).unwrap();
            buffer.write_u8(m.tx_count).unwrap();
            buffer.write_u32::<LittleEndian>(m.id).unwrap();
            buffer.write_u32::<LittleEndian>(m.frame_length).unwrap();
            buffer.write_u32::<LittleEndian>(m.flags).unwrap();
            buffer.write_u32::<LittleEndian>(m.btr_cfg_arb).unwrap();
            buffer.write_u32::<LittleEndian>(m.btr_cfg_data).unwrap();
            buffer
                .write_u32::<LittleEndian>(m.time_offset_brs_ns)
                .unwrap();
            buffer
                .write_u32::<LittleEndian>(m.time_offset_crc_del_ns)
                .unwrap();
            buffer.write_u16::<LittleEndian>(m.bit_count).unwrap();
            buffer.write_u8(m.dir).unwrap();
            buffer.write_u8(m.ext_data_offset).unwrap();
            buffer.write_u32::<LittleEndian>(m.crc).unwrap();
            buffer.extend_from_slice(&m.data);
        }
        _ => {}
    }
}

fn write_object_header(buffer: &mut Vec<u8>, header: &ObjectHeader) {
    use byteorder::{LittleEndian, WriteBytesExt};

    buffer.write_u32::<LittleEndian>(header.signature).unwrap();
    buffer
        .write_u16::<LittleEndian>(header.header_size)
        .unwrap();
    buffer
        .write_u16::<LittleEndian>(header.header_version)
        .unwrap();
    buffer
        .write_u32::<LittleEndian>(header.object_size)
        .unwrap();
    buffer
        .write_u32::<LittleEndian>(header.object_type as u32)
        .unwrap();
    buffer
        .write_u32::<LittleEndian>(header.object_flags)
        .unwrap();
    buffer.write_u16::<LittleEndian>(0).unwrap(); // client_index
    buffer.write_u16::<LittleEndian>(0).unwrap(); // object_version
    buffer
        .write_u64::<LittleEndian>(header.object_time_stamp)
        .unwrap();
}

/// 解析 BLF 文件并显示信息
fn parse_blf_file() {
    println!("\n=== 解析 BLF 文件 ===\n");

    print!("请输入 BLF 文件路径 (留空使用 test_all_messages.blf): ");
    std::io::stdout().flush().unwrap();

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    let path = input.trim();
    let path = if path.is_empty() {
        "test_all_messages.blf"
    } else {
        path
    };

    match blf::read_blf_from_file(path) {
        Ok(result) => {
            println!("\n✓ 成功解析文件: {}", path);
            println!("  文件大小: {} 字节", result.file_stats.file_size);
            println!("  对象数量: {}", result.objects.len());

            // 统计消息类型
            let mut counts = std::collections::HashMap::new();
            for obj in &result.objects {
                let type_name = match obj {
                    LogObject::CanMessage(_) => "CAN Message",
                    LogObject::CanMessage2(_) => "CAN Message2",
                    LogObject::CanFdMessage(_) => "CAN FD Message",
                    LogObject::CanFdMessage64(_) => "CAN FD Message64",
                    LogObject::LinMessage(_) => "LIN Message",
                    _ => "Other",
                };
                *counts.entry(type_name).or_insert(0) += 1;
            }

            println!("\n消息类型统计:");
            for (type_name, count) in &counts {
                println!("  {}: {}", type_name, count);
            }

            // 显示前 5 条消息详情
            println!("\n前 5 条消息详情:");
            for (i, obj) in result.objects.iter().take(5).enumerate() {
                print_message_info(i + 1, obj);
            }
        }
        Err(e) => {
            eprintln!("✗ 解析失败: {}", e);
        }
    }
}

fn print_message_info(index: usize, msg: &LogObject) {
    match msg {
        LogObject::CanMessage(m) => {
            println!(
                "  [{}] CAN: ID=0x{:03X}, Ch={}, DLC={}, Data={:02X?}",
                index,
                m.id,
                m.channel,
                m.dlc,
                &m.data[..m.dlc as usize]
            );
        }
        LogObject::CanMessage2(m) => {
            println!(
                "  [{}] CAN2: ID=0x{:03X}, Ch={}, DLC={}, Data={:02X?}",
                index, m.id, m.channel, m.dlc, &m.data
            );
        }
        LogObject::CanFdMessage(m) => {
            println!(
                "  [{}] CAN FD: ID=0x{:03X}, Ch={}, DLC={}, ValidBytes={}, Flags=0x{:02X}",
                index, m.id, m.channel, m.dlc, m.valid_data_bytes, m.can_fd_flags
            );
            println!("        Data[0..8]={:02X?}", &m.data[..m.data.len().min(8)]);
        }
        LogObject::CanFdMessage64(m) => {
            println!(
                "  [{}] CAN FD64: ID=0x{:03X}, Ch={}, DLC={}, ValidBytes={}, Flags=0x{:04X}",
                index, m.id, m.channel, m.dlc, m.valid_data_bytes, m.flags
            );
            println!(
                "        IsFD={}, BRS={}, ESI={}, Dir={}",
                m.is_fd_frame(),
                m.has_brs(),
                m.has_esi(),
                m.dir
            );
            println!("        Data[0..8]={:02X?}", &m.data[..m.data.len().min(8)]);
        }
        _ => {}
    }
}

/// 验证 CAN FD Message64 结构
fn verify_can_fd_64() {
    println!("\n=== 验证 CAN FD Message64 结构 ===\n");

    // 测试用例
    let test_cases = vec![
        (8, 8, "标准 CAN FD，8 字节数据"),
        (15, 64, "最大 CAN FD，64 字节数据"),
        (9, 12, "DLC=9，实际 12 字节"),
        (13, 32, "DLC=13，实际 32 字节"),
    ];

    for (dlc, valid_bytes, description) in test_cases {
        println!("测试: {}", description);
        println!("  DLC={}, ValidBytes={}", dlc, valid_bytes);

        let msg = CanFdMessage64 {
            header: create_test_header(ObjectType::CanFdMessage64, 120, 1000),
            channel: 1,
            dlc,
            valid_data_bytes,
            tx_count: 0,
            id: 0x123,
            frame_length: 5000,
            flags: 0x7000, // EDL + BRS + ESI
            btr_cfg_arb: 0,
            btr_cfg_data: 0,
            time_offset_brs_ns: 0,
            time_offset_crc_del_ns: 0,
            bit_count: 500,
            dir: 1,
            ext_data_offset: 0,
            crc: 0,
            data: vec![0xAA; valid_bytes as usize],
            ext_data: None,
        };

        // 验证辅助方法
        println!("  is_fd_frame(): {}", msg.is_fd_frame());
        println!("  has_brs(): {}", msg.has_brs());
        println!("  has_esi(): {}", msg.has_esi());
        println!("  is_tx(): {}", msg.is_tx());

        // 验证数据长度
        assert_eq!(
            msg.data.len(),
            valid_bytes as usize,
            "数据长度应该等于 valid_data_bytes"
        );

        // 验证标志位
        assert!(msg.is_fd_frame(), "应该设置 EDL 位");
        assert!(msg.has_brs(), "应该设置 BRS 位");
        assert!(msg.has_esi(), "应该设置 ESI 位");

        println!("  ✓ 验证通过\n");
    }

    println!("✓ 所有 CAN FD Message64 测试通过!");
}

/// 运行单元测试
fn run_unit_tests() {
    println!("\n=== 运行单元测试 ===\n");

    println!("正在编译和运行测试...\n");

    let output = std::process::Command::new("cargo")
        .args(["test", "--package", "blf", "--", "--nocapture"])
        .current_dir("..")
        .output();

    match output {
        Ok(output) => {
            println!("{}", String::from_utf8_lossy(&output.stdout));
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));

            if output.status.success() {
                println!("\n✓ 所有单元测试通过!");
            } else {
                println!("\n✗ 部分测试失败，请检查输出");
            }
        }
        Err(e) => {
            eprintln!("✗ 无法运行测试: {}", e);
        }
    }
}

/// 生成界面测试数据
fn generate_ui_test_data() {
    println!("\n=== 生成界面测试数据 ===\n");

    let filename = "ui_test_messages.json";
    let file = File::create(filename).expect("无法创建文件");

    let messages = generate_test_messages();

    let mut test_data = serde_json::json!({
        "version": "1.0",
        "description": "UI 测试数据 - 包含所有 CAN 消息类型",
        "messages": []
    });

    for msg in &messages {
        let msg_json = match msg {
            LogObject::CanMessage(m) => serde_json::json!({
                "type": "CAN",
                "channel": m.channel,
                "id": format!("0x{:X}", m.id),
                "dlc": m.dlc,
                "data": m.data.iter().take(m.dlc as usize)
                    .map(|b| format!("{:02X}", b))
                    .collect::<Vec<_>>()
                    .join(" "),
                "timestamp": m.header.object_time_stamp
            }),
            LogObject::CanFdMessage64(m) => serde_json::json!({
                "type": "CAN FD64",
                "channel": m.channel,
                "id": format!("0x{:X}", m.id),
                "dlc": m.dlc,
                "valid_data_bytes": m.valid_data_bytes,
                "data": m.data.iter()
                    .map(|b| format!("{:02X}", b))
                    .collect::<Vec<_>>()
                    .join(" "),
                "flags": format!("0x{:04X}", m.flags),
                "is_fd": m.is_fd_frame(),
                "has_brs": m.has_brs(),
                "has_esi": m.has_esi(),
                "timestamp": m.header.object_time_stamp
            }),
            _ => continue,
        };

        test_data["messages"].as_array_mut().unwrap().push(msg_json);
    }

    serde_json::to_writer_pretty(file, &test_data).expect("JSON 写入失败");

    println!("✓ 成功生成界面测试数据: {}", filename);
    println!(
        "  包含 {} 条消息",
        test_data["messages"].as_array().unwrap().len()
    );

    // 显示统计信息
    let mut type_counts = std::collections::HashMap::new();
    for msg in test_data["messages"].as_array().unwrap() {
        let msg_type = msg["type"].as_str().unwrap();
        *type_counts.entry(msg_type).or_insert(0) += 1;
    }

    println!("\n消息类型统计:");
    for (type_name, count) in &type_counts {
        println!("  {}: {}", type_name, count);
    }
}
