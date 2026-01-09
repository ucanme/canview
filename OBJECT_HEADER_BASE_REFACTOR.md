# ObjectHeaderBase 重构文档

## 概述

本次重构将 Rust 代码中的 `ObjectHeader` 结构体重构为更清晰的层次结构，使其与 C++ 代码的继承关系更好地对应。

## C++ 代码结构

在 C++ 代码中，对象头部采用了继承层次结构：

```cpp
// ObjectHeaderBase (基类，16字节)
struct ObjectHeaderBase {
    uint32_t signature;        // +0  (4 bytes)
    uint16_t headerSize;       // +4  (2 bytes)
    uint16_t headerVersion;    // +6  (2 bytes)
    uint32_t objectSize;       // +8  (4 bytes)
    ObjectType objectType;     // +12 (4 bytes)
};

// ObjectHeader (V1版本，继承自ObjectHeaderBase，总共32字节)
struct ObjectHeader : ObjectHeaderBase {
    uint32_t objectFlags;      // +16 (4 bytes)
    uint16_t clientIndex;      // +20 (2 bytes)
    uint16_t objectVersion;    // +22 (2 bytes)
    uint64_t objectTimeStamp;  // +24 (8 bytes)
};

// ObjectHeader2 (V2版本，继承自ObjectHeaderBase，总共48字节)
struct ObjectHeader2 : ObjectHeaderBase {
    uint32_t objectFlags;         // +16 (4 bytes)
    uint8_t  timeStampStatus;     // +20 (1 byte)
    uint8_t  reserved;            // +21 (1 byte)
    uint16_t objectVersion;       // +22 (2 bytes)
    uint64_t objectTimeStamp;     // +24 (8 bytes)
    uint64_t originalTimeStamp;   // +32 (8 bytes)
};
```

## 重构前的 Rust 代码

重构前，Rust 代码只有一个扁平的 `ObjectHeader` 结构体：

```rust
pub struct ObjectHeader {
    pub signature: u32,
    pub header_size: u16,
    pub header_version: u16,
    pub object_size: u32,
    pub object_type: ObjectType,
    pub object_flags: u32,
    pub client_index: u16,
    pub object_version: u16,
    pub object_time_stamp: u64,
    pub original_time_stamp: Option<u64>,
    pub time_stamp_status: Option<u8>,
}
```

**问题**：
- 没有明确表示哪些字段来自基类
- 与 C++ 代码的对应关系不清晰
- 代码可读性和可维护性较差

## 重构后的 Rust 代码

### 新的结构定义

```rust
/// 基础对象头部（对应 C++ ObjectHeaderBase）
#[derive(Debug, Clone)]
pub struct ObjectHeaderBase {
    pub signature: u32,
    pub header_size: u16,
    pub header_version: u16,
    pub object_size: u32,
    pub object_type: ObjectType,
}

/// 完整对象头部（对应 C++ ObjectHeader/ObjectHeader2）
#[derive(Debug, Clone)]
pub struct ObjectHeader {
    /// 基础头部字段（所有版本共有）
    pub base: ObjectHeaderBase,
    
    /// 对象标志（V1 & V2）
    pub object_flags: u32,
    
    /// V1 特定字段
    pub client_index: u16,
    pub object_version: u16,
    
    /// 时间戳（V1 & V2）
    pub object_time_stamp: u64,
    
    /// V2 特定字段
    pub original_time_stamp: Option<u64>,
    pub time_stamp_status: Option<u8>,
    pub reserved: u8,
}
```

### 关键改进

1. **清晰的层次结构**
   - `ObjectHeaderBase` 包含所有版本共有的基础字段
   - `ObjectHeader` 使用组合而非继承，包含 `base` 字段

2. **版本特定字段明确标注**
   - V1 特定字段：`client_index`
   - V2 特定字段：`original_time_stamp`, `time_stamp_status`, `reserved`

3. **便利方法**
   - 实现了 `Deref` 和 `DerefMut` trait，可以方便地访问基础字段
   - 提供了 `version()`, `object_type()`, `signature()` 等访问方法

### 使用示例

```rust
// 创建新的 ObjectHeader
let header = ObjectHeader::new(1, ObjectType::CanMessage);

// 通过 Deref 访问基础字段
let version = header.version();  // 等同于 header.base.header_version
let obj_type = header.object_type();  // 等同于 header.base.object_type

// 直接访问所有字段
let flags = header.object_flags;
let timestamp = header.object_time_stamp;

// V2 特定字段
if header.version() == 2 {
    let original_ts = header.original_time_stamp.unwrap();
}
```

## 文件变更

### 修改的文件

1. **src/blf/src/objects/object_header.rs**
   - 添加了 `ObjectHeaderBase` 结构体
   - 重构了 `ObjectHeader` 结构体，使用组合模式
   - 添加了 `ObjectFlags` 和 `TimeStampStatus` 枚举
   - 实现了 `Deref` 和 `DerefMut` trait
   - 添加了便利方法：`version()`, `object_type()`, `signature()`
   - 保留了原有的 `read()`, `write()` 方法，并更新了实现

2. **src/blf/src/blf_core.rs**
   - 删除了重复的 `ObjectHeader` 定义
   - 现在使用 `objects` 模块导出的版本

### 兼容性

**向后兼容性**：
- 公共 API 保持不变
- 所有现有代码仍能正常工作
- 通过 `Deref` trait，访问 `signature`, `header_size` 等字段时可以使用简写形式

**需要注意的变更**：
- 创建 `ObjectHeader` 时需要使用 `ObjectHeader::new()` 或直接构造
- 访问基础字段时，可以使用 `header.base.field` 或通过 Deref 使用 `header.field`

## 内存布局

### V1 Header (32 bytes)
```
+0x00  signature (u32)          [ObjectHeaderBase]
+0x04  header_size (u16)        [ObjectHeaderBase]
+0x06  header_version (u16)     [ObjectHeaderBase]
+0x08  object_size (u32)        [ObjectHeaderBase]
+0x0C  object_type (u32)        [ObjectHeaderBase]
+0x10  object_flags (u32)
+0x14  client_index (u16)
+0x16  object_version (u16)
+0x18  object_time_stamp (u64)
```

### V2 Header (48 bytes)
```
+0x00  signature (u32)          [ObjectHeaderBase]
+0x04  header_size (u16)        [ObjectHeaderBase]
+0x06  header_version (u16)     [ObjectHeaderBase]
+0x08  object_size (u32)        [ObjectHeaderBase]
+0x0C  object_type (u32)        [ObjectHeaderBase]
+0x10  object_flags (u32)
+0x14  time_stamp_status (u8)
+0x15  reserved (u8)
+0x16  object_version (u16)
+0x18  object_time_stamp (u64)
+0x20  original_time_stamp (u64)
```

## 测试建议

1. **单元测试**
   - 测试 V1 和 V2 header 的读取和写入
   - 验证内存布局与 C++ 版本一致
   - 测试 `ObjectHeaderBase` 的独立功能

2. **集成测试**
   - 测试完整的 BLF 文件解析
   - 验证与现有代码的兼容性

3. **性能测试**
   - 确保重构没有引入性能回归
   - 对比新旧实现的解析速度

## 总结

这次重构提高了代码的可读性和可维护性，使其更清晰地对应 C++ 代码的结构，同时保持了 Rust 的最佳实践（组合优于继承）。所有现有功能都得到保留，并且添加了更清晰的类型层次结构。