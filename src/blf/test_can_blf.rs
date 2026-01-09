use blf::read_blf_from_file;

fn main() {
    let result = read_blf_from_file("../../can.blf").expect("Failed to parse");
    
    println!("Application: {}.{}.{}.{}",
        result.file_stats.application_major,
        result.file_stats.application_minor,
        result.file_stats.application_build,
        result.file_stats.application_id
    );
    println!("File Size: {}", result.file_stats.file_size);
    println!("Object Count: {}", result.file_stats.object_count);
}
