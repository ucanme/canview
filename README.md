# CanView: BLF Parser & Desktop Viewer

[中文文档 (Chinese Version)](README_zh.md)

## Introduction

CanView is a high-performance toolset for processing BLF (Binary Logging Format) files, commonly used in the automotive industry for CAN, LIN, FlexRay, and Ethernet bus logging.

The project consists of two parts:
1.  **`blf`**: A high-performance Rust library for parsing BLF files.
2.  **`view`**: A modern desktop application built with Dioxus for visualizing log data.

## Features

### BLF Parser Library
- **Comprehensive Support**: Parses various log objects including CAN, CAN FD, LIN, FlexRay, and Ethernet.
- **High Performance**: Built with Rust for zero-cost abstractions and memory safety.
- **Easy Integration**: Clean API for use in other Rust projects.

### Desktop Viewer
- **Modern UI**: Built with Dioxus/Tao/Wry for a sleek, responsive interface.
- **Frameless Design**: Custom title bar and window controls for a premium look.
- **Data Visualization**: Clear list view of messages with timestamps, IDs, and data payloads.
- **Interactive**: Drag-and-drop parsable regions (planned) and easy file loading.

## Quick Start

### Running the Viewer

Ensure you have Rust installed.

```bash
# Run the desktop application
cargo run -p view
```

### Using the Library

Add the dependency to your `Cargo.toml`:

```toml
[dependencies]
blf = { path = "src/blf" }
```

### Library Usage Example

```rust
use blf::{BlfParser, LogObject};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read file content
    let bytes = std::fs::read("example.blf")?;
    
    // Parse BLF data
    let parser = BlfParser::new();
    let objects = parser.parse(&bytes)?;
    
    // Iterate through objects
    for object in objects {
        match object {
            LogObject::CanMessage(msg) => {
                println!("CAN Message: ID={:x}, DLC={}, Data={:?}", 
                         msg.id, msg.dlc, msg.data);
            }
            LogObject::CanFdMessage(msg) => {
                println!("CAN FD Message: ID={:x}, Len={}, Data={:?}", 
                         msg.id, msg.valid_payload_length, msg.data);
            }
            _ => {}
        }
    }
    
    Ok(())
}
```

## Project Structure

```
canview/
├── src/
│   ├── blf/           # BLF Parser Library
│   │   ├── src/
│   │   │   ├── objects/  # Object implementations
│   │   │   ├── parser.rs # Core parser logic
│   │   │   └── ...
│   │   └── Cargo.toml
│   │
│   └── view/          # Desktop Application
│       ├── src/
│       │   └── main.rs   # UI logic
│       └── Cargo.toml
├── Cargo.toml         # Workspace configuration
└── README.md          # English Documentation
```

## Supported Message Types

-   CAN (CanMessage, CanMessage2, CanFdMessage, CanFdMessage64)
-   CAN Error & Statistics
-   LIN (LinMessage, LinMessage2, etc.)
-   FlexRay (Messages, Status, Cycles)
-   Ethernet Frames
-   System Events (AppTrigger, Comments)

## License

This project is licensed under the MIT License.