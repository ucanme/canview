# ObjectHeader 字段对齐方案

## 概述

本文档记录了将 Rust 实现的 `ObjectHeader` 与 Vector BLF C++ 实现对齐的方案。

## C++ ObjectHeader 结构

### ObjectHeaderBase (基础部分 - 16 字节)

```cpp
struct ObjectHeaderBase {
    uint32_t signature;        // +0  (4 bytes) - 0x4A424F4C ("LOBJ")
    uint16_t headerSize;       // +4  (2 bytes) - 头部大小
    uint16_t headerVersion;    // +6  (2 bytes) - 版本号 (1 或 2)
    uint32_t objectSize;       // +8  (4 bytes) - 对象总大小
    ObjectType objectType;     // +12 (4 bytes) - 对象类型 (enum uint32_t)
};
```

### ObjectHeader Version 1 (扩展部分 +16 字节)

```cpp
struct ObjectHeader : ObjectHeaderBase {
    uint32_t objectFlags;      // +16 (4 bytes) - 对象标志
    uint16_t clientIndex;      // +20 (2 bytes) - 客户端索引
    uint16_t objectVersion;    // +22 (2 bytes) - 对象版本
    uint64_t objectTimeStamp;  // +24 (8 bytes) - 时间戳
};
// 总大小: 32 字节
```

### ObjectHeader Version 2 (扩展部分 +32 字节)

```cpp
struct ObjectHeader2 : ObjectHeaderBase {
    uint32_t objectFlags;         // +16 (4 bytes) - 对象标志
    uint8_t  timeStampStatus;     // +20 (1 byte)  - 时间戳状态
    uint8_t  reserved;            // +21 (1 byte)  - 保留
    uint16_t objectVersion;       // +22 (2 bytes) - 对象版本
    uint64_t objectTimeStamp;     // +24 (8 bytes) - 时间戳
    uint64_t originalTimeStamp;   // +32 (8 bytes) - 原始时间戳
};
// 总大小: 48 字节
```

## 字段对齐表

| 偏移 | 大小 | 字段名 | V1 | V2 | 说明 |
|------|------|--------|-----|-----|------|
| +0   | 4    | signature | ✓ | ✓ | 固定值 0x4A424F4C |
| +4   | 2    | headerSize | ✓ | ✓ | V1=32, V2=48 |
| +6   | 2    | headerVersion | ✓ | ✓ | 1 或 2 |
| +8   | 4    | objectSize | ✓ | ✓ | 对象总大小 |
| +12  | 4    | objectType | ✓ | ✓ | 对象类型枚举 |
| +16  | 4    | objectFlags | ✓ | ✓ | 标志位 |
| +20  | 2    | clientIndex | ✓ | - | 客户端索引 |
| +20  | 1    | timeStampStatus | - | ✓ | 时间戳状态 |
| +21  | 1    | reserved | - | ✓ | 保留字段 |
| +22  | 2    | objectVersion | ✓ | ✓ | 对象版本 |
| +24  | 8    | objectTimeStamp | ✓ | ✓ | 时间戳 |
| +32  | 8    | originalTimeStamp | - | ✓ | 原始时间戳 |

## Rust 实现修正

### 当前问题

1. **缺少字段**: `client_index` 和 `object_version` 未在结构体中定义
2. **读取时忽略**: V1 读取时将这两个字段丢弃
3. **写入时硬编码**: 写入时使用固定值 0

### 修正后的结构体

```rust
#[derive(Debug, Clone)]
pub struct ObjectHeader {
    // ObjectHeaderBase
    pub signature: u32,              // +0  (4 bytes)
    pub header_size: u16,            // +4  (2 bytes)
    pub header_version: u16,         // +6  (2 bytes)
    pub object_size: u32,            // +8  (4 bytes)
    pub object_type: ObjectType,     // +12 (4 bytes)
    
    // Version-dependent fields
    pub object_flags: u32,           // +16 (4 bytes)
    
    // V1: client_index (2 bytes) + object_version (2 bytes)
    // V2: time_stamp_status (1 byte) + reserved (1 byte) + object_version (2 bytes)
    pub client_index: u16,           // +20 (2 bytes) - V1 only
    pub object_version: u16,         // +22 (2 bytes) - V1 & V2
    
    pub object_time_stamp: u64,      // +24 (8 bytes)
    
    // V2 only
    pub original_time_stamp: Option<u64>,    // +32 (8 bytes) - V2 only
    pub time_stamp_status: Option<u8>,       // +20 (1 byte) - V2 only (与client_index重叠)
}
```

### 内存布局说明

**重要**: 由于 Rust 的 enum 和 Option 类型，内存布局需要注意：

1. **V1 header (32 bytes)**:
   - 使用 `client_index` 字段
   - `time_stamp_status` 和 `original_time_stamp` 为 `None`
   - 实际内存布局: 16 (base) + 4 (flags) + 2 (client_idx) + 2 (version) + 8 (timestamp) = 32 bytes

2. **V2 header (48 bytes)**:
   - 使用 `time_stamp_status` 和 `original_time_stamp`
   - `client_index` 设为 0（不使用）
   - 实际内存布局: 16 (base) + 4 (flags) + 1 (status) + 1 (reserved) + 2 (version) + 8 (timestamp) + 8 (original) = 40 bytes

## 实现步骤

### 步骤 1: 更新 `src/blf/src/objects/object_header.rs`

✅ 已完成 - 添加了 `client_index` 和 `object_version` 字段

### 步骤 2: 更新 `src/blf/src/blf_core.rs`

需要修改:
1. 在 `ObjectHeader` 结构体中添加字段
2. 在 `read()` 方法中正确读取这些字段
3. 移除对不同 header_size 的特殊处理，使用标准的 32/48 字节

### 步骤 3: 更新 `src/blf/src/test_utils.rs`

需要修改 `serialize_object_header()` 函数:
1. 添加 `client_index` 参数
2. 添加 `object_version` 参数
3. 正确写入这些字段

### 步骤 4: 更新其他文件

检查并更新以下文件中所有创建 `ObjectHeader` 的地方:
- `src/blf/src/bin/gen_test_blf.rs`
- `src/blf/src/bin/generate_blf.rs`
- `src/blf/src/file.rs` (测试)
- 其他测试文件

## ObjectFlags 常量

根据 C++ 代码，应该添加以下常量:

```rust
impl ObjectHeader {
    /// 10微秒时间戳精度
    pub const FLAG_TIME_TEN_MICS: u32 = 0x00000001;
    
    /// 1纳秒时间戳精度
    pub const FLAG_TIME_ONE_NANS: u32 = 0x00000002;
}
```

## 验证方法

1. **编译检查**: `cargo build --package blf`
2. **单元测试**: `cargo test --package blf`
3. **实际文件**: `cargo run --package blf --bin read_blf -- sample.blf`
4. **字段验证**: 检查读取的 header 是否包含正确的 `client_index` 和 `object_version`

## 注意事项

1. **向后兼容**: 修改后需要确保仍能正确读取现有的 BLF 文件
2. **默认值**: 新字段应该有合理的默认值 (通常是 0)
3. **版本检查**: 读写时必须根据 `header_version` 正确处理字段
4. **测试覆盖**: 为新字段添加测试用例

## 参考资料

- C++ 源码: `c++/src/Vector/BLF/ObjectHeader.h`
- C++ 源码: `c++/src/Vector/BLF/ObjectHeader.cpp`
- C++ 源码: `c++/src/Vector/BLF/ObjectHeaderBase.h`
- Vector BLF 文档 (如有)