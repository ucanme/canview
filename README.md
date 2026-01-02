# CanView: BLF Parser & Desktop Viewer

[中文文档 (Chinese Version)](README_zh.md)

## Introduction

CanView is a high-performance toolset for processing and visualizing BLF (Binary Logging Format) files, widely used in the automotive industry for CAN, LIN, FlexRay, and Ethernet bus logging.

The project is organized into three main components:
1.  **`blf`**: A high-performance Rust library for parsing BLF files.
2.  **`parser`**: A library for parsing network description files like DBC (CAN) and LDF (LIN).
3.  **`view`**: A modern desktop application built with Dioxus/Tao/Wry for visualizing logs and decoding signals.

## Features

### 📚 BLF Parser Library (`blf`)
- **Comprehensive Support**: Parses various log objects including CAN, CAN FD, LIN, FlexRay, and Ethernet.
- **High Performance**: Built with Rust for zero-cost abstractions and memory safety.
- **Easy Integration**: Clean API for use in other Rust projects.

### 🗂️ Database Parser (`parser`)
- **DBC Support**: Parses Vector DBC files for CAN signal definitions, including comments.
- **LDF Support**: Parses LIN Description Files (LDF) for LIN signal definitions.
- **Comment Parsing**: Extracts comments and descriptions from database files for better context.

### 🖥️ Desktop Viewer (`view`)
- **Modern UI**: Built with Dioxus for a sleek, responsive interface.
- **Log Visualization**: Clear list view of messages with timestamps, channels, IDs, and payloads.
- **Multi-Channel Decoding**: 
    - Map different channels (CAN/LIN) to specific DBC or LDF files.
    - Support for multiple databases active simultaneously on different channels.
- **Signal Decoding**: Real-time decoding of CAN and LIN signals based on loaded databases.
- **Configuration Management**: 
    - **Signal Libraries**: Organize your DBC/LDF files into libraries with version control.
    - **Active Version**: Switch between different decoding configurations instantly.
    - **JSON Config**: Save and load your channel mappings and library configurations.
- **Files**: Open `.blf` or `.bin` log files.

## Quick Start

### Running the Viewer

Ensure you have Rust installed.

```bash
# Run the desktop application
cargo run -p view
```

### Using the Libraries

Add the dependencies to your `Cargo.toml`:

```toml
[dependencies]
blf = { path = "src/blf" }
parser = { path = "src/parser" }
```

### Library Usage Example

```rust
use blf::{read_blf_from_file, LogObject};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read BLF file
    let result = read_blf_from_file("example.blf")?;
    
    // Iterate through objects
    for object in result.objects {
        match object {
            LogObject::CanMessage(msg) => {
                println!("CAN Message: ID={:x}, DLC={}, Data={:?}", 
                         msg.id, msg.dlc, msg.data);
            }
            LogObject::LinMessage(msg) => {
                 println!("LIN Message: ID={:x}, DLC={}, Data={:?}", 
                         msg.id, msg.dlc, msg.data);
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
│   │   ├── src/objects/      # BLF Object implementations
│   │   └── ...
│   │
│   ├── parser/        # Database Parser Library
│   │   ├── src/dbc/          # DBC parsing logic
│   │   ├── src/ldf/          # LDF parsing logic
│   │   └── ...
│   │
│   └── view/          # Desktop Application
│       ├── src/main.rs       # UI logic and State Management
│       └── ...
│
├── Cargo.toml         # Workspace configuration
└── README.md          # Project Documentation
```

## Supported Message Types

-   **CAN**: CanMessage, CanMessage2, CanFdMessage, CanFdMessage64
-   **LIN**: LinMessage, LinMessage2, etc.
-   **FlexRay**: Messages, Status, Cycles
-   **Ethernet**: Ethernet Frames
-   **System**: AppTrigger, Comments
-   **Statistics**: CAN Error, CAN Driver Error

## License

This project is licensed under the MIT License.