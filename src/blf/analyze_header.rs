use std::fs;

fn main() {
    let data = fs::read("../../can.blf").expect("Failed to read file");
    
    println!("=== Analyzing can.blf Header (first 144 bytes) ===");
    println!();
    
    // Parse header fields
    println!("Signature: {:.4?}", String::from_utf8_lossy(&data[0..4]));
    println!("Statistics Size: {}", u32::from_le_bytes([data[4], data[5], data[6], data[7]]));
    println!("CRC: {:#x}", u32::from_le_bytes([data[8], data[9], data[10], data[11]]));
    println!();
    
    println!("Application ID: {}", data[12]);
    println!("Compression Level: {}", data[13]);
    println!("Application Major: {} (0x{:02x})", data[14], data[14]);
    println!("Application Minor: {} (0x{:02x})", data[15], data[15]);
    println!("Application Build: {} (0x{:02x})", data[16], data[16]);
    println!();
    
    println!("File Size: {}", u64::from_le_bytes([
        data[24], data[25], data[26], data[27],
        data[28], data[29], data[30], data[31]
    ]));
    
    println!("Uncompressed Size: {}", u64::from_le_bytes([
        data[32], data[33], data[34], data[35],
        data[36], data[37], data[38], data[39]
    ]));
    
    println!("Object Count: {}", u64::from_le_bytes([
        data[40], data[41], data[42], data[43],
        data[44], data[45], data[46], data[47]
    ]));
    println!();
    
    // Parse SYSTEMTIME structures
    // SYSTEMTIME is 16 bytes: year, month, day_of_week, day, hour, minute, second, milliseconds (all u16)
    let parse_time = |offset: usize| -> (u16, u16, u16, u16, u16, u16, u16, u16) {
        let year = u16::from_le_bytes([data[offset], data[offset+1]]);
        let month = u16::from_le_bytes([data[offset+2], data[offset+3]]);
        let day_of_week = u16::from_le_bytes([data[offset+4], data[offset+5]]);
        let day = u16::from_le_bytes([data[offset+6], data[offset+7]]);
        let hour = u16::from_le_bytes([data[offset+8], data[offset+9]]);
        let minute = u16::from_le_bytes([data[offset+10], data[offset+11]]);
        let second = u16::from_le_bytes([data[offset+12], data[offset+13]]);
        let milliseconds = u16::from_le_bytes([data[offset+14], data[offset+15]]);
        (year, month, day_of_week, day, hour, minute, second, milliseconds)
    };
    
    // Try different offsets for time fields
    for offset in [48, 52, 56, 64] {
        println!("--- Time at offset 0x{:02x} ---", offset);
        let (year, month, dow, day, hour, min, sec, ms) = parse_time(offset);
        println!("  Date: {:04}-{:02}-{:02} (dow: {})", year, month, day, dow);
        println!("  Time: {:02}:{:02}:{:02}.{:03}", hour, min, sec, ms);
        println!();
    }
    
    // Show hex dump of bytes 12-63 (application info through first time)
    println!("=== Hex dump of bytes 0x0C-0x3F ===");
    for i in (12..64).step_by(16) {
        let end = (i + 16).min(64);
        print!("{:02x}: ", i);
        for j in i..end {
            print!("{:02x} ", data[j]);
        }
        println!();
    }
}
