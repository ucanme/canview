# BLF 解析库 (Binary Logging Format)

## 简介

这是一个用 Rust 编写的高性能 BLF (Binary Logging Format) 解析库，用于解析 Vector Informatik 的 BLF 文件格式。BLF 是一种广泛应用于汽车工业中的二进制日志文件格式，用于存储 CAN、LIN、FlexRay 和 Ethernet 等总线通信数据。

本项目是基于 C++ 实现的直接翻译版本，保持了与原始实现相同的功能和性能特性。

## 特性

- **完整的 BLF 格式支持**：支持解析多种类型的日志对象，包括 CAN、LIN、FlexRay、Ethernet 等总线消息
- **高性能**：使用 Rust 的零成本抽象和内存安全特性，实现高性能解析
- **易于使用**：提供简洁的 API 接口，方便集成到其他项目中
- **内存安全**：利用 Rust 的所有权和借用检查机制，避免内存泄漏和缓冲区溢出
- **可扩展**：模块化设计，易于添加新的消息类型支持

## 支持的消息类型

- CAN 消息 (CanMessage, CanMessage2, CanFdMessage, CanFdMessage64)
- LIN 消息 (LinMessage, LinMessage2 等)
- FlexRay 消息和事件
- Ethernet 帧
- MOST 消息和事件
- 系统变量和环境变量
- 应用触发器和事件注释

## 安装

在你的 `Cargo.toml` 文件中添加依赖：

```toml
[dependencies]
blf = { path = "path/to/blf/crate" }
```

## 使用示例

```rust
use blf::{read_blf_from_file, BlfResult};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 读取 BLF 文件
    let result: BlfResult = read_blf_from_file("example.blf")?;
    
    // 访问文件统计信息
    println!("File statistics: {:?}", result.file_stats);
    
    // 遍历解析出的对象
    for object in result.objects {
        match object {
            LogObject::CanMessage(msg) => {
                println!("CAN Message: ID={:x}, DLC={}, Data={:?}", 
                         msg.id, msg.dlc, msg.data);
            }
            // 处理其他类型的对象...
            _ => {}
        }
    }
    
    Ok(())
}
```

## 项目结构

```
src/
├── blf_core.rs        # 核心结构和错误处理
├── file.rs            # 文件读取和解析
├── file_statistics.rs # 文件统计信息处理
├── parser.rs          # 主解析器实现
├── object_header.rs   # 对象头部处理
├── object_type.rs     # 对象类型定义
├── objects/           # 各种对象类型的实现
│   ├── can/
│   ├── lin/
│   ├── flexray/
│   ├── ethernet/
│   └── ...
└── test_utils.rs      # 测试工具函数
```

## 测试

运行测试套件：

```bash
cargo test
```

## 许可证

本项目采用 MIT 许可证。详见 [LICENSE](LICENSE) 文件。