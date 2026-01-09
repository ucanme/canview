# BLF 文件解析改进文档

本文档记录了对 BLF (Binary Log Format) 文件解析器的改进，主要参考了 C++ 代码库中的协议定义。

## 改进概述

基于 Vector BLF C++ 库的实现，对 Rust 版本的 BLF 解析器进行了全面改进，主要包括：

1. 扩展了 ObjectType 枚举，添加了所有缺失的对象类型
2. 重新实现了 `CanFdMessage` 结构，完全匹配 C++ 定义
3. 重新实现了 `CanFdMessage64` 结构，完全匹配 C++ 定义
4. 添加了 `CanFdExtFrameData` 支持用于扩展数据

## 详细改进

### 1. ObjectType 枚举扩展

在 `objects/object_type.rs` 中，添加了以下缺失的对象类型：

- `EnvData` (9)
- `Reserved26-28` (26-28)
- `LinChecksumInfo` (42) 到 `AttributeEvent` (131) 之间的所有类型

总共从约 40 个对象类型扩展到超过 120 个对象类型，完全覆盖了 Vector BLF 规范。

### 2. CanFdMessage 结构修正

**文件**: `objects/can/fd_message.rs`

根据 C++ 定义 (`CanFdMessage.h`)，修正了字段顺序和类型：

```rust
pub struct CanFdMessage {
    pub header: ObjectHeader,
    pub channel: u16,           // 应用通道
    pub flags: u8,              // CAN 消息标志 (dir, rtr, wu & nerr)
    pub dlc: u8,                // 数据长度代码
    pub id: u32,                // 帧标识符
    pub frame_length: u32,      // 消息持续时间（纳秒）
    pub arb_bit_count: u8,      // 仲裁阶段位计数
    pub can_fd_flags: u8,       // CAN FD 标志 (EDL, BRS, ESI)
    pub valid_data_bytes: u8,   // 有效数据长度
    pub reserved1: u8,          // 保留字段
    pub reserved2: u32,         // 保留字段
    pub data: [u8; 64],         // CAN FD 数据字节
    pub reserved3: u32,         // 保留字段
}
```

**关键改进**：
- 字段顺序与 C++ 完全一致
- 添加了标志位常量（TX, NERR, WU, RTR 等）
- 添加了 CAN FD 标志位常量（EDL, BRS, ESI）

### 3. CanFdMessage64 结构重新实现

**文件**: `objects/can/fd_message64.rs`

这是最重要的改进。完全重写了 `CanFdMessage64` 结构以匹配 C++ 定义：

```rust
pub struct CanFdMessage64 {
    pub header: ObjectHeader,
    
    // 基本信息
    pub channel: u8,                    // 应用通道 (u8，不是 u16!)
    pub dlc: u8,                        // 数据长度代码
    pub valid_data_bytes: u8,           // 有效数据字节数
    pub tx_count: u8,                   // 传输计数
    
    // 帧信息
    pub id: u32,                        // 帧标识符
    pub frame_length: u32,              // 消息持续时间（纳秒）
    pub flags: u32,                     // 消息标志（包含 EDL, BRS, ESI 等）
    
    // 位时序配置
    pub btr_cfg_arb: u32,               // 仲裁阶段位时序配置
    pub btr_cfg_data: u32,              // 数据阶段位时序配置
    
    // 时间偏移
    pub time_offset_brs_ns: u32,        // BRS 字段时间偏移（纳秒）
    pub time_offset_crc_del_ns: u32,    // CRC 定界符时间偏移（纳秒）
    
    // 其他信息
    pub bit_count: u16,                 // 消息位计数
    pub dir: u8,                        // 方向 (0=Rx, 1=Tx, 2=TxRq)
    pub ext_data_offset: u8,            // 扩展数据偏移
    pub crc: u32,                       // CRC 校验
    
    // 数据
    pub data: Vec<u8>,                  // CAN FD 数据（可变长度）
    
    // 可选扩展数据
    pub ext_data: Option<CanFdExtFrameData>,
}
```

**关键改进**：
1. `channel` 从 `u16` 改为 `u8`（与 C++ 一致）
2. 添加了完整的字段集合，包括位时序配置和时间偏移
3. `data` 字段从固定数组改为 `Vec<u8>`，支持可变长度
4. 添加了可选的 `ext_data` 字段支持扩展数据
5. 添加了辅助方法来检查标志位：
   - `is_fd_frame()` - 检查是否为 CAN FD 帧
   - `has_brs()` - 检查是否启用位速率切换
   - `has_esi()` - 检查错误状态指示器
   - `is_tx()` - 检查是否为发送帧

### 4. CanFdExtFrameData 支持

**文件**: `objects/can/fd_message64.rs`

添加了 `CanFdExtFrameData` 结构用于处理可选的扩展数据：

```rust
pub struct CanFdExtFrameData {
    pub btr_ext_arb: u32,       // 仲裁阶段位速率
    pub btr_ext_data: u32,      // 数据阶段位速率
    pub reserved: Vec<u8>,      // 保留数据
}
```

## 代码组织

为了保持代码清晰，将 CAN FD 相关代码分离到独立文件：

```
objects/can/
├── mod.rs              # 模块导出
├── messages.rs         # CanMessage 和 CanMessage2（非 FD）
├── fd_message.rs       # CanFdMessage
└── fd_message64.rs     # CanFdMessage64 和 CanFdExtFrameData
```

## 测试

为新的结构添加了完整的单元测试：

1. **CanFdMessage64 基本读取测试** (`test_can_fd_message64_read_basic`)
   - 测试基本的字段读取
   - 验证可变长度数据处理

2. **标志位测试** (`test_can_fd_message64_flags`)
   - 测试 EDL 位检测
   - 验证位偏移量计算正确性

3. **BRS 和 ESI 测试** (`test_can_fd_message64_with_brs_esi`)
   - 测试多个标志位的组合
   - 验证 BRS 和 ESI 位检测

## 字段偏移量对照表

### CanFdMessage64 结构布局

| 偏移量 | 字段 | 类型 | 说明 |
|--------|------|------|------|
| 0 | channel | u8 | 应用通道 |
| 1 | dlc | u8 | 数据长度代码 |
| 2 | valid_data_bytes | u8 | 有效数据字节数 |
| 3 | tx_count | u8 | 传输计数 |
| 4-7 | id | u32 | 帧标识符 |
| 8-11 | frame_length | u32 | 消息持续时间 |
| 12-15 | flags | u32 | 消息标志 |
| 16-19 | btr_cfg_arb | u32 | 仲裁阶段位时序配置 |
| 20-23 | btr_cfg_data | u32 | 数据阶段位时序配置 |
| 24-27 | time_offset_brs_ns | u32 | BRS 时间偏移 |
| 28-31 | time_offset_crc_del_ns | u32 | CRC 时间偏移 |
| 32-33 | bit_count | u16 | 位计数 |
| 34 | dir | u8 | 方向 |
| 35 | ext_data_offset | u8 | 扩展数据偏移 |
| 36-39 | crc | u32 | CRC 校验 |
| 40+ | data | u8[] | 数据（可变长度） |

## 标志位定义

### CanFdMessage64 标志位 (flags: u32)

| 位掩码 | 常量名 | 说明 |
|--------|--------|------|
| 0x0004 | FLAG_NERR | 单线操作（低速 CAN） |
| 0x0008 | FLAG_HIGH_VOLTAGE_WAKEUP | 高电压唤醒 |
| 0x0010 | FLAG_REMOTE_FRAME | 远程帧（仅 CAN） |
| 0x0040 | FLAG_TX_ACK | 发送确认 |
| 0x0080 | FLAG_TX_REQUEST | 发送请求 |
| 0x0200 | FLAG_SRR | 替代远程请求（CAN FD） |
| 0x1000 | FLAG_EDL | 扩展数据长度（0=CAN, 1=CAN FD） |
| 0x2000 | FLAG_BRS | 位速率切换 |
| 0x4000 | FLAG_ESI | 错误状态指示器 |
| 0x20000 | FLAG_BURST | 帧是突发的一部分 |

## 兼容性说明

这些改进确保了与 Vector BLF C++ 库的完全兼容性：

- ✅ 字段顺序与 C++ 一致
- ✅ 字段类型与 C++ 一致
- ✅ 字节对齐与 C++ 一致
- ✅ 支持所有 Vector BLF 对象类型
- ✅ 正确处理可变长度数据

## 使用示例

```rust
use blf::{BlfParser, LogObject};

// 解析 BLF 文件
let parser = BlfParser::new();
let objects = parser.parse_file("example.blf")?;

// 处理 CAN FD 消息
for obj in objects {
    if let LogObject::CanFdMessage64(msg) = obj {
        println!("CAN FD64 Message:");
        println!("  ID: {:#x}", msg.id);
        println!("  Channel: {}", msg.channel);
        println!("  DLC: {}", msg.dlc);
        println!("  Data Length: {}", msg.valid_data_bytes);
        println!("  Is FD Frame: {}", msg.is_fd_frame());
        println!("  Has BRS: {}", msg.has_brs());
        println!("  Data: {:02x?}", &msg.data[..msg.valid_data_bytes as usize]);
    }
}
```

## 后续工作

建议的后续改进：

1. 添加更多对象类型的完整实现（LIN, FlexRay, Ethernet 等）
2. 优化内存使用，特别是对于大型 BLF 文件
3. 添加 BLF 文件写入功能
4. 改进错误处理和错误消息
5. 添加性能基准测试

## 参考资料

- Vector BLF C++ 库: `c++/src/Vector/BLF/`
- BLF 文件格式规范
- Vector Informatik 官方文档

---

**最后更新**: 2025-01-XX
**作者**: 基于 Vector BLF C++ 实现改进
**版本**: 0.1.0