# CanView: BLF 解析库与可视化工具

## 简介

CanView 是一个高性能的 BLF (Binary Logging Format) 工具集，包含：
1. **BLF 解析库** (`blf`): 用于解析 Vector Informatik 的 BLF 文件格式。
2. **可视化工具** (`view`): 基于 Dioxus 开发的现代化桌面应用，用于查看 CAN/CAN-FD 等总线数据。

BLF 是一种广泛应用于汽车工业中的二进制日志文件格式，用于存储 CAN、LIN、FlexRay 和 Ethernet 等总线通信数据。

## 功能特性

### BLF 解析库
- **完整的 BLF 格式支持**：支持解析多种类型的日志对象，包括 CAN、LIN、FlexRay、Ethernet 等总线消息
- **高性能**：使用 Rust 的零成本抽象和内存安全特性，实现高性能解析
- **内存安全**：利用 Rust 的所有权和借用检查机制，避免内存泄漏和缓冲区溢出

### 桌面可视化工具 (Desktop Viewer)
- **现代化 UI**：基于 Dioxus 框架开发，提供流畅的用户体验
- **无边框设计**：自定义标题栏和窗口控制，界面简洁美观
- **快速浏览**：支持加载大型 BLF 文件并展示消息列表
- **详细信息**：直观展示时间戳、消息类型、ID、DLC 和数据负载

## 快速开始

### 运行可视化工具

确保你已经安装了 Rust 环境。

```bash
# 运行桌面查看器
cargo run -p view
```

### 在项目中使用解析库

在你的 `Cargo.toml` 文件中添加依赖：

```toml
[dependencies]
blf = { path = "src/blf" }
```

### 使用示例 (解析库)

```rust
use blf::{BlfParser, LogObject};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 读取文件内容
    let bytes = std::fs::read("example.blf")?;
    
    // 解析 BLF 数据
    let parser = BlfParser::new();
    let objects = parser.parse(&bytes)?;
    
    // 遍历解析出的对象
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

## 项目结构

```
canview/
├── src/
│   ├── blf/           # BLF 解析库核心代码
│   │   ├── src/
│   │   │   ├── objects/  # 各种对象类型的实现 (CAN, LIN, Ethernet...)
│   │   │   ├── parser.rs # 主解析器实现
│   │   │   └── ...
│   │   └── Cargo.toml
│   │
│   └── view/          # 桌面可视化应用 (Dioxus)
│       ├── src/
│       │   └── main.rs   # UI 逻辑与渲染
│       └── Cargo.toml
├── Cargo.toml         # 工作空间配置
└── README_zh.md       # 中文文档
```

## 支持的消息类型

- CAN 消息 (CanMessage, CanMessage2, CanFdMessage, CanFdMessage64)
- CAN 错误与统计 (CanErrorFrame, CanDriverError, CanDriverStatistic)
- LIN 消息 (LinMessage, LinMessage2 等)
- FlexRay 消息和事件
- Ethernet 帧
- 应用触发器和事件注释

## 许可证

本项目采用 MIT 许可证。