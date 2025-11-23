# BLF Parser Library (Binary Logging Format)

## 选择语言 (Language)

- [中文版本 (Chinese)](README_zh.md)
- [English Version](README_en.md)

## Overview

This repository contains a high-performance BLF (Binary Logging Format) parser library written in Rust for parsing Vector Informatik's BLF file format. BLF is a binary log file format widely used in the automotive industry to store bus communication data such as CAN, LIN, FlexRay, and Ethernet.

This project is a direct translation from a C++ implementation, maintaining the same functionality and performance characteristics as the original.

## Language Versions

We provide documentation in two languages for your convenience:
1. [README in Chinese (中文版)](README_zh.md)
2. [README in English (英文版)](README_en.md)

Please select the language you prefer to read the full documentation.

## Features

- **Complete BLF Format Support**: Supports parsing various types of log objects, including CAN, LIN, FlexRay, Ethernet bus messages
- **High Performance**: Uses Rust's zero-cost abstractions and memory safety features for high-performance parsing
- **Easy to Use**: Provides a clean API interface for easy integration into other projects
- **Memory Safe**: Leverages Rust's ownership and borrowing mechanisms to avoid memory leaks and buffer overflows
- **Extensible**: Modular design makes it easy to add support for new message types

## Supported Message Types

- CAN messages (CanMessage, CanMessage2, CanFdMessage, CanFdMessage64)
- LIN messages (LinMessage, LinMessage2, etc.)
- FlexRay messages and events
- Ethernet frames
- MOST messages and events
- System variables and environment variables
- Application triggers and event comments

## Installation

Add the dependency to your `Cargo.toml` file:

```toml
[dependencies]
blf = { path = "path/to/blf/crate" }
```

## Usage Example

``rust
use blf::{read_blf_from_file, BlfResult};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read BLF file
    let result: BlfResult = read_blf_from_file("example.blf")?;
    
    // Access file statistics
    println!("File statistics: {:?}", result.file_stats);
    
    // Iterate through parsed objects
    for object in result.objects {
        match object {
            LogObject::CanMessage(msg) => {
                println!("CAN Message: ID={:x}, DLC={}, Data={:?}", 
                         msg.id, msg.dlc, msg.data);
            }
            // Handle other object types...
            _ => {}
        }
    }
    
    Ok(())
}
```

## Project Structure

```
src/
├── blf_core.rs        # Core structures and error handling
├── file.rs            # File reading and parsing
├── file_statistics.rs # File statistics processing
├── parser.rs          # Main parser implementation
├── object_header.rs   # Object header processing
├── object_type.rs     # Object type definitions
├── objects/           # Implementation of various object types
│   ├── can/
│   ├── lin/
│   ├── flexray/
│   ├── ethernet/
│   └── ...
└── test_utils.rs      # Test utility functions
```

## Testing

Run the test suite:

```bash
cargo test
```

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

---

# BLF Parser Library (Binary Logging Format) (English)

## Introduction

This is a high-performance BLF (Binary Logging Format) parser library written in Rust for parsing Vector Informatik's BLF file format. BLF is a binary log file format widely used in the automotive industry to store bus communication data such as CAN, LIN, FlexRay, and Ethernet.

This project is a direct translation from a C++ implementation, maintaining the same functionality and performance characteristics as the original.

## Features

- **Complete BLF Format Support**: Supports parsing various types of log objects, including CAN, LIN, FlexRay, Ethernet bus messages
- **High Performance**: Uses Rust's zero-cost abstractions and memory safety features for high-performance parsing
- **Easy to Use**: Provides a clean API interface for easy integration into other projects
- **Memory Safe**: Leverages Rust's ownership and borrowing mechanisms to avoid memory leaks and buffer overflows
- **Extensible**: Modular design makes it easy to add support for new message types

## Supported Message Types

- CAN messages (CanMessage, CanMessage2, CanFdMessage, CanFdMessage64)
- LIN messages (LinMessage, LinMessage2, etc.)
- FlexRay messages and events
- Ethernet frames
- MOST messages and events
- System variables and environment variables
- Application triggers and event comments

## Installation

Add the dependency to your `Cargo.toml` file:

```toml
[dependencies]
blf = { path = "path/to/blf/crate" }
```

## Usage Example

``rust
use blf::{read_blf_from_file, BlfResult};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read BLF file
    let result: BlfResult = read_blf_from_file("example.blf")?;
    
    // Access file statistics
    println!("File statistics: {:?}", result.file_stats);
    
    // Iterate through parsed objects
    for object in result.objects {
        match object {
            LogObject::CanMessage(msg) => {
                println!("CAN Message: ID={:x}, DLC={}, Data={:?}", 
                         msg.id, msg.dlc, msg.data);
            }
            // Handle other object types...
            _ => {}
        }
    }
    
    Ok(())
}
```

## Project Structure

```
src/
├── blf_core.rs        # Core structures and error handling
├── file.rs            # File reading and parsing
├── file_statistics.rs # File statistics processing
├── parser.rs          # Main parser implementation
├── object_header.rs   # Object header processing
├── object_type.rs     # Object type definitions
├── objects/           # Implementation of various object types
│   ├── can/
│   ├── lin/
│   ├── flexray/
│   ├── ethernet/
│   └── ...
└── test_utils.rs      # Test utility functions
```

## Testing

Run the test suite:

```bash
cargo test
```

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.