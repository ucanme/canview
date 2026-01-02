use blf::{read_blf_from_file, LogObject};
use std::path::Path;

#[test]
fn test_read_sample_blf() {
    // The test runs from the crate directory or workspace root.
    // We try to locate the file relative to where the test binary runs.
    let possible_paths = [
        "sample.blf",
        "../../sample.blf",
        "../../../sample.blf",
    ];

    let mut path = Path::new("sample.blf");
    let mut found = false;

    for p in &possible_paths {
        if Path::new(p).exists() {
            path = Path::new(p);
            found = true;
            break;
        }
    }

    if !found {
        // If not found, print current directory for debugging
        let current_dir = std::env::current_dir().unwrap();
        panic!("Could not find sample.blf. Current directory: {:?}", current_dir);
    }

    println!("Reading BLF file from: {:?}", path);

    let result = read_blf_from_file(path).expect("Failed to parse BLF file");

    // Verify statistics (roughly)
    // We generated 144 bytes stats + 32 header + 16 container overhead + 10 * 48 bytes data = 
    // 144 + 32 + 16 + 480 = 672 bytes.
    // The sizes might be slightly different due to padding but let's check basic counts.
    
    let object_count = result.objects.len();
    assert_eq!(object_count, 12, "Expected 12 objects, found {}", object_count);

    // Verify first object
    if let LogObject::CanMessage(msg) = &result.objects[0] {
        assert_eq!(msg.id, 0x100);
        assert_eq!(msg.dlc, 8);
        assert_eq!(msg.data, [0, 0, 0, 0, 0, 0, 0, 0]);
    } else {
        panic!("First object is not a CanMessage");
    }

    // Verify last object
    if let LogObject::CanMessage(msg) = &result.objects[9] {
        assert_eq!(msg.id, 0x109);
        assert_eq!(msg.dlc, 8);
        assert_eq!(msg.data, [9, 9, 9, 9, 9, 9, 9, 9]);
    } else {
        panic!("Last object is not a CanMessage");
    }
}
