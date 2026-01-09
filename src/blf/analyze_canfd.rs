use std::fs;

fn main() {
    let data = fs::read("../../can.blf").expect("Failed to read");
    
    // 找到第一个 CAN FD 对象 (从 0x90 开始，跳过 144 字节头部)
    // 第一个 LogContainer 在 0x90，header_size=16，所以第一个 CAN FD 在 0x90+16+16=0xB0
    let obj_offset = 0xB0;
    
    println!("=== 分析第一个 CAN FD64 对象 ===");
    println!("偏移: 0x{:02x}", obj_offset);
    println!();
    
    // 对象头部 (16 字节，因为 header_size=16)
    println!("对象头部 (16 字节):");
    for i in 0..16 {
        print!("  0x{:02x}: {:02x}", obj_offset + i, data[obj_offset + i]);
        if (i + 1) % 4 == 0 {
            println!();
        } else {
            print!("  ");
        }
    }
    println!();
    
    // 对象体
    let body_offset = obj_offset + 16;
    println!("对象体 (CAN FD64):");
    println!("  Channel: 0x{:02x} 0x{:02x} = {}", data[body_offset], data[body_offset+1], 
        u16::from_le_bytes([data[body_offset], data[body_offset+1]]));
    println!("  CAN FD Flags: 0x{:02x}", data[body_offset+2]);
    println!("  Valid Payload Length: 0x{:02x} = {}", data[body_offset+3], data[body_offset+3]);
    println!("  Arb Bit Count: 0x{:02x}", data[body_offset+4]);
    println!("  Serial Bit Count: 0x{:02x}", data[body_offset+5]);
    println!("  ID: 0x{:02x} {:02x} {:02x} {:02x} = {:#x}", 
        data[body_offset+6], data[body_offset+7], data[body_offset+8], data[body_offset+9],
        u32::from_le_bytes([data[body_offset+6], data[body_offset+7], data[body_offset+8], data[body_offset+9]]));
    println!();
    
    println!("Data (64 bytes):");
    for i in 0..64 {
        let offset = body_offset + 10 + i;
        print!("{:02x} ", data[offset]);
        if (i + 1) % 16 == 0 {
            println!();
        }
    }
    println!();
    
    // 显示更多的元数据字段
    let meta_offset = body_offset + 10 + 64;
    println!("元数据字段:");
    println!("  Frame Length: 0x{:02x} {:02x} {:02x} {:02x} = {}",
        data[meta_offset], data[meta_offset+1], data[meta_offset+2], data[meta_offset+3],
        u32::from_le_bytes([data[meta_offset], data[meta_offset+1], data[meta_offset+2], data[meta_offset+3]]));
    println!("  Bit Count: 0x{:02x}", data[meta_offset+4]);
    println!("  Dir: 0x{:02x}", data[meta_offset+5]);
    println!("  EDL/BRS/ESI: 0x{:02x}", data[meta_offset+6]);
    println!("  Reserved1: 0x{:02x}", data[meta_offset+7]);
    println!("  Reserved2: 0x{:02x} {:02x} {:02x} {:02x}",
        data[meta_offset+8], data[meta_offset+9], data[meta_offset+10], data[meta_offset+11]);
}
