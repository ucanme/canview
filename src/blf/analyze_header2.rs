use std::fs;

fn main() {
    let data = fs::read("../../can.blf").expect("Failed to read file");
    
    println!("=== Detailed can.blf Header Analysis ===");
    println!();
    
    // Show bytes 0x0C-0x3F with offset labels
    println!("Bytes 0x0C-0x2F:");
    for i in (12..48).step_by(4) {
        print!("  0x{:02x}: ", i);
        for j in i..(i+4).min(48) {
            print!("{:02x} ", data[j]);
        }
        // Try to interpret as different types
        if i + 4 <= 48 {
            let as_u32 = u32::from_le_bytes([data[i], data[i+1], data[i+2], data[i+3]]);
            print!(" (u32: {})", as_u32);
        }
        println!();
    }
    println!();
    
    // Parse based on CANalyzer BLF format
    println!("=== Parsed Fields (CANalyzer format) ===");
    println!("Application ID: {}", data[0x0C]);
    println!("Compression Level: {}", data[0x0D]);
    
    // Bytes 0x0E-0x11 might be application version (u32?)
    let app_version = u32::from_le_bytes([data[0x0E], data[0x0F], data[0x10], data[0x11]]);
    println!("Application Version (u32): {} (0x{:08x})", app_version, app_version);
    println!("  - Major: {}", app_version / 1000000);
    println!("  - Minor: {}", (app_version % 1000000) / 1000);
    println!("  - Build: {}", app_version % 1000);
    println!();
    
    // File stats start at 0x18?
    println!("=== File Statistics (starting at 0x18?) ===");
    let file_size = u64::from_le_bytes([
        data[0x18], data[0x19], data[0x1A], data[0x1B],
        data[0x1C], data[0x1D], data[0x1E], data[0x1F]
    ]);
    println!("File Size: {}", file_size);
    
    let uncompressed_size = u64::from_le_bytes([
        data[0x20], data[0x21], data[0x22], data[0x23],
        data[0x24], data[0x25], data[0x26], data[0x27]
    ]);
    println!("Uncompressed Size: {}", uncompressed_size);
    
    let obj_count = u64::from_le_bytes([
        data[0x28], data[0x29], data[0x2A], data[0x2B],
        data[0x2C], data[0x2D], data[0x2E], data[0x2F]
    ]);
    println!("Object Count: {}", obj_count);
    println!();
    
    // Time fields
    let parse_time = |offset: usize, name: &str| {
        let year = u16::from_le_bytes([data[offset], data[offset+1]]);
        let month = u16::from_le_bytes([data[offset+2], data[offset+3]]);
        let day = u16::from_le_bytes([data[offset+6], data[offset+7]]);
        let hour = u16::from_le_bytes([data[offset+8], data[offset+9]]);
        let min = u16::from_le_bytes([data[offset+10], data[offset+11]]);
        let sec = u16::from_le_bytes([data[offset+12], data[offset+13]]);
        println!("{}: {:04}-{:02}-{:02} {:02}:{:02}:{:02}", name, year, month, day, hour, min, sec);
    };
    
    parse_time(0x38, "Start Time");
    parse_time(0x48, "End Time");
}
