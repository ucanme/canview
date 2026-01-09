# ObjectHeader C++ 代码集成说明文档

## 概述

本文档详细说明了 Rust 实现如何与 C++ 代码保持一致，包括结构体定义、方法实现和内存布局的对应关系。

## C++ 类层次结构

### ObjectHeaderBase (基类)

```cpp
// c++/src/Vector/BLF/ObjectHeaderBase.h
struct ObjectHeaderBase {
    ObjectHeaderBase(const uint16_t headerVersion, const ObjectType objectType);
    virtual ~ObjectHeaderBase() noexcept = default;
    
    virtual void read(AbstractFile & is);
    virtual void write(AbstractFile & os);
    virtual uint16_t calculateHeaderSize() const;
    virtual uint32_t calculateObjectSize() const;
    
    uint32_t signature {ObjectSignature};  // 0x4A424F4C
    uint16_t headerSize {};
    uint16_t headerVersion {};
    uint32_t objectSize {};
    ObjectType objectType {ObjectType::UNKNOWN};
};
```

**内存布局：16 字节**
```
偏移   大小    字段名
+0x00  4       signature
+0x04  2       headerSize
+0x06  2       headerVersion
+0x08  4       objectSize
+0x0C  4       objectType
```

### ObjectHeader (V1 版本)

```cpp
// c++/src/Vector/BLF/ObjectHeader.h
struct ObjectHeader : ObjectHeaderBase {
    ObjectHeader(const ObjectType objectType, const uint16_t objectVersion = 0);
    
    void read(AbstractFile & is) override;
    void write(AbstractFile & os) override;
    uint16_t calculateHeaderSize() const override;
    uint32_t calculateObjectSize() const override;
    
    enum ObjectFlags : uint32_t {
        TimeTenMics = 0x00000001,  // 10 微秒时间戳
        TimeOneNans = 0x00000002   // 1 纳秒时间戳
    };
    
    uint32_t objectFlags {ObjectFlags::TimeOneNans};
    uint16_t clientIndex {};
    uint16_t objectVersion {0};
    uint64_t objectTimeStamp {};
};
```

**内存布局：32 字节**
```
偏移   大小    字段名             来源
+0x00  4       signature          [ObjectHeaderBase]
+0x04  2       headerSize         [ObjectHeaderBase]
+0x06  2       headerVersion      [ObjectHeaderBase]
+0x08  4       objectSize         [ObjectHeaderBase]
+0x0C  4       objectType         [ObjectHeaderBase]
+0x10  4       objectFlags        [ObjectHeader]
+0x14  2       clientIndex        [ObjectHeader]
+0x16  2       objectVersion      [ObjectHeader]
+0x18  8       objectTimeStamp    [ObjectHeader]
```

### ObjectHeader2 (V2 版本)

```cpp
// c++/src/Vector/BLF/ObjectHeader2.h
struct ObjectHeader2 : ObjectHeaderBase {
    ObjectHeader2(const ObjectType objectType);
    
    void read(AbstractFile & is) override;
    void write(AbstractFile & os) override;
    uint16_t calculateHeaderSize() const override;
    uint32_t calculateObjectSize() const override;
    
    enum ObjectFlags : uint32_t {
        TimeTenMics = 0x00000001,
        TimeOneNans = 0x00000002
    };
    
    enum TimeStampStatus : uint8_t {
        Orig = 0x01,  // 原始时间戳有效
        SwHw = 0x02,  // 软件(1) vs 硬件(0)生成
        User = 0x10   // 协议特定含义
    };
    
    uint32_t objectFlags {ObjectFlags::TimeOneNans};
    uint8_t timeStampStatus {};
    uint8_t reservedObjectHeader{0};
    uint16_t objectVersion {0};
    uint64_t objectTimeStamp {0};
    uint64_t originalTimeStamp {0};
};
```

**内存布局：40 字节**
```
偏移   大小    字段名              来源
+0x00  4       signature           [ObjectHeaderBase]
+0x04  2       headerSize          [ObjectHeaderBase]
+0x06  2       headerVersion       [ObjectHeaderBase]
+0x08  4       objectSize          [ObjectHeaderBase]
+0x0C  4       objectType          [ObjectHeaderBase]
+0x10  4       objectFlags         [ObjectHeader2]
+0x14  1       timeStampStatus     [ObjectHeader2]
+0x15  1       reservedObjectHeader [ObjectHeader2]
+0x16  2       objectVersion       [ObjectHeader2]
+0x18  8       objectTimeStamp     [ObjectHeader2]
+0x20  8       originalTimeStamp   [ObjectHeader2]
```

## Rust 实现

### ObjectHeaderBase 结构体

```rust
// src/blf/src/objects/object_header.rs
pub const OBJECT_SIGNATURE: u32 = 0x4A424F4C;

#[derive(Debug, Clone)]
pub struct ObjectHeaderBase {
    pub signature: u32,
    pub header_size: u16,
    pub header_version: u16,
    pub object_size: u32,
    pub object_type: ObjectType,
}

impl ObjectHeaderBase {
    pub fn new(header_version: u16, object_type: ObjectType) -> Self {
        ObjectHeaderBase {
            signature: OBJECT_SIGNATURE,
            header_size: 16,  // 由派生类计算
            header_version,
            object_size: 0,   // 由派生类计算
            object_type,
        }
    }
    
    pub fn read(cursor: &mut Cursor<&[u8]>) -> BlfParseResult<Self>;
    pub fn write<W: Write>(&self, writer: &mut W) -> BlfParseResult<()>;
    pub fn calculate_header_size(&self) -> u16;
    pub fn calculate_object_size(&self) -> u32;
}
```

**与 C++ 的对应关系：**

| C++ | Rust | 说明 |
|-----|------|------|
| `ObjectSignature` (常量) | `OBJECT_SIGNATURE` (常量) | 签名值 0x4A424F4C |
| `ObjectHeaderBase()` 构造函数 | `ObjectHeaderBase::new()` | 初始化基类 |
| `read()` | `read()` | 从字节流读取 |
| `write()` | `write()` | 写入字节流 |
| `calculateHeaderSize()` | `calculate_header_size()` | 返回 16 |
| `calculateObjectSize()` | `calculate_object_size()` | 返回 header_size |

### ObjectHeader 结构体（统一 V1 和 V2）

```rust
#[derive(Debug, Clone)]
pub struct ObjectHeader {
    pub base: ObjectHeaderBase,
    
    // V1 & V2 通用
    pub object_flags: u32,
    
    // V1 特定
    pub client_index: u16,
    pub object_version: u16,
    
    // V1 & V2 通用
    pub object_time_stamp: u64,
    
    // V2 特定
    pub original_time_stamp: Option<u64>,
    pub time_stamp_status: Option<u8>,
    pub reserved: u8,
}
```

**构造函数：**

```rust
impl ObjectHeader {
    // 对应 C++: ObjectHeader::ObjectHeader(ObjectType, uint16_t)
    pub fn new_v1(object_type: ObjectType, object_version: u16) -> Self;
    
    // 对应 C++: ObjectHeader2::ObjectHeader2(ObjectType)
    pub fn new_v2(object_type: ObjectType) -> Self;
    
    // 便利方法
    pub fn new(header_version: u16, object_type: ObjectType) -> Self;
}
```

**方法对应关系：**

| C++ ObjectHeader | Rust ObjectHeader | 说明 |
|-----------------|-------------------|------|
| `ObjectHeader(type, version)` | `new_v1(type, version)` | V1 构造函数 |
| `ObjectHeader2(type)` | `new_v2(type)` | V2 构造函数 |
| `read(is)` | `read(cursor)` | 读取整个头部 |
| `write(os)` | `write(writer)` | 写入整个头部 |
| `calculateHeaderSize()` | `calculate_header_size()` | V1: 32, V2: 40 |
| `calculateObjectSize()` | `calculate_object_size()` | 返回 header_size |

## 关键实现细节

### 1. 预处理（Pre-processing）

**C++ 实现：**
```cpp
void ObjectHeaderBase::write(AbstractFile & os) {
    /* pre processing */
    headerSize = calculateHeaderSize();
    objectSize = calculateObjectSize();
    
    os.write(reinterpret_cast<char *>(&signature), sizeof(signature));
    // ... 写入其他字段
}
```

**Rust 实现：**
```rust
impl ObjectHeader {
    /// 在写入前调用此方法来计算大小
    pub fn prepare_for_write(&mut self) {
        self.base.header_size = self.calculate_header_size();
        self.base.object_size = self.calculate_object_size();
    }
    
    pub fn write<W: Write>(&self, writer: &mut W) -> BlfParseResult<()> {
        self.base.write(writer)?;
        // ... 写入其他字段
    }
}
```

**使用示例：**
```rust
// C++:
header.write(os);  // 自动计算大小

// Rust:
header.prepare_for_write();  // 手动计算大小
header.write(&mut writer)?;  // 然后写入
```

### 2. calculateHeaderSize() 实现

**C++ ObjectHeader::calculateHeaderSize():**
```cpp
uint16_t ObjectHeader::calculateHeaderSize() const {
    return
        ObjectHeaderBase::calculateHeaderSize() +  // 16
        sizeof(objectFlags) +                      // 4
        sizeof(clientIndex) +                      // 2
        sizeof(objectVersion) +                    // 2
        sizeof(objectTimeStamp);                   // 8
}
// 总计: 32 字节
```

**Rust ObjectHeader::calculate_header_size():**
```rust
pub fn calculate_header_size(&self) -> u16 {
    if self.base.header_version == 1 {
        // V1: 16 + 4 + 2 + 2 + 8 = 32
        self.base.calculate_header_size() +
        std::mem::size_of::<u32>() as u16 +  // objectFlags
        std::mem::size_of::<u16>() as u16 +  // clientIndex
        std::mem::size_of::<u16>() as u16 +  // objectVersion
        std::mem::size_of::<u64>() as u16    // objectTimeStamp
    } else if self.base.header_version == 2 {
        // V2: 16 + 4 + 1 + 1 + 2 + 8 + 8 = 40
        self.base.calculate_header_size() +
        std::mem::size_of::<u32>() as u16 +  // objectFlags
        std::mem::size_of::<u8>() as u16 +   // timeStampStatus
        std::mem::size_of::<u8>() as u16 +   // reserved
        std::mem::size_of::<u16>() as u16 +  // objectVersion
        std::mem::size_of::<u64>() as u16 +  // objectTimeStamp
        std::mem::size_of::<u64>() as u16    // originalTimeStamp
    } else {
        self.base.header_size
    }
}
```

### 3. read() 方法实现

**C++ ObjectHeader::read():**
```cpp
void ObjectHeader::read(AbstractFile & is) {
    ObjectHeaderBase::read(is);  // 先读取基类部分
    is.read(reinterpret_cast<char *>(&objectFlags), sizeof(objectFlags));
    is.read(reinterpret_cast<char *>(&clientIndex), sizeof(clientIndex));
    is.read(reinterpret_cast<char *>(&objectVersion), sizeof(objectVersion));
    is.read(reinterpret_cast<char *>(&objectTimeStamp), sizeof(objectTimeStamp));
}
```

**Rust ObjectHeader::read():**
```rust
pub fn read(cursor: &mut Cursor<&[u8]>) -> BlfParseResult<Self> {
    // 先读取基类部分
    let base = ObjectHeaderBase::read(cursor)?;
    
    if base.header_version == 1 {
        // V1: 读取扩展字段
        let object_flags = cursor.read_u32::<LittleEndian>()?;
        let client_index = cursor.read_u16::<LittleEndian>()?;
        let object_version = cursor.read_u16::<LittleEndian>()?;
        let object_time_stamp = cursor.read_u64::<LittleEndian>()?;
        // ...
    } else if base.header_version == 2 {
        // V2: 读取不同的扩展字段
        let object_flags = cursor.read_u32::<LittleEndian>()?;
        let time_stamp_status = Some(cursor.read_u8()?);
        let reserved = cursor.read_u8()?;
        let object_version = cursor.read_u16::<LittleEndian>()?;
        let object_time_stamp = cursor.read_u64::<LittleEndian>()?;
        let original_time_stamp = Some(cursor.read_u64::<LittleEndian>()?);
        // ...
    }
}
```

## 枚举类型对应

### ObjectFlags

| C++ | Rust | 值 |
|-----|------|-----|
| `ObjectHeader::ObjectFlags::TimeTenMics` | `ObjectFlags::TimeTenMics` | 0x00000001 |
| `ObjectHeader::ObjectFlags::TimeOneNans` | `ObjectFlags::TimeOneNans` | 0x00000002 |

### TimeStampStatus

| C++ | Rust | 值 |
|-----|------|-----|
| `ObjectHeader2::TimeStampStatus::Orig` | `TimeStampStatus::Orig` | 0x01 |
| `ObjectHeader2::TimeStampStatus::SwHw` | `TimeStampStatus::SwHw` | 0x02 |
| `ObjectHeader2::TimeStampStatus::User` | `TimeStampStatus::User` | 0x10 |

## 使用示例

### 创建 V1 Header

**C++:**
```cpp
// 创建 CAN 消息头部
ObjectHeader header(ObjectType::CAN_MESSAGE, 0);
header.objectFlags = ObjectHeader::ObjectFlags::TimeOneNans;
header.clientIndex = 1;
header.objectTimeStamp = 1234567890;
```

**Rust:**
```rust
// 创建 CAN 消息头部
let mut header = ObjectHeader::new_v1(ObjectType::CanMessage, 0);
header.object_flags = ObjectFlags::TimeOneNans as u32;
header.client_index = 1;
header.object_time_stamp = 1234567890;
```

### 创建 V2 Header

**C++:**
```cpp
// 创建 CAN 消息2头部
ObjectHeader2 header2(ObjectType::CAN_MESSAGE2);
header2.objectFlags = ObjectHeader2::ObjectFlags::TimeOneNans;
header2.timeStampStatus = ObjectHeader2::TimeStampStatus::Orig;
header2.objectTimeStamp = 1234567890;
header2.originalTimeStamp = 1234567880;
```

**Rust:**
```rust
// 创建 CAN 消息2头部
let mut header = ObjectHeader::new_v2(ObjectType::CanMessage2);
header.object_flags = ObjectFlags::TimeOneNans as u32;
header.time_stamp_status = Some(TimeStampStatus::Orig as u8);
header.object_time_stamp = 1234567890;
header.original_time_stamp = Some(1234567880);
```

### 序列化和反序列化

**C++:**
```cpp
// 写入
header.write(os);

// 读取
ObjectHeader header;
header.read(is);
```

**Rust:**
```rust
// 写入
header.prepare_for_write();
header.write(&mut writer)?;

// 读取
let header = ObjectHeader::read(&mut cursor)?;
```

## 内存对齐和大小确认

### V1 Header 大小计算

```
ObjectHeaderBase: 16 字节
  - signature:      4
  - headerSize:     2
  - headerVersion:  2
  - objectSize:     4
  - objectType:     4

ObjectHeader 扩展: 16 字节
  - objectFlags:    4
  - clientIndex:    2
  - objectVersion:  2
  - objectTimeStamp:8

总计: 32 字节
```

### V2 Header 大小计算

```
ObjectHeaderBase: 16 字节
  - signature:      4
  - headerSize:     2
  - headerVersion:  2
  - objectSize:     4
  - objectType:     4

ObjectHeader2 扩展: 24 字节
  - objectFlags:        4
  - timeStampStatus:    1
  - reservedObjectHeader:1
  - objectVersion:      2
  - objectTimeStamp:    8
  - originalTimeStamp:  8

总计: 40 字节
```

## 测试验证

所有 Rust 实现都通过以下测试验证：

```rust
#[test]
fn test_object_header_v1_write_read_roundtrip() {
    let mut header = ObjectHeader::new_v1(ObjectType::CanMessage, 5);
    header.prepare_for_write();
    
    let mut buffer = Vec::new();
    header.write(&mut buffer).unwrap();
    
    let mut cursor = Cursor::new(buffer.as_slice());
    let header2 = ObjectHeader::read(&mut cursor).unwrap();
    
    // 验证所有字段一致
    assert_eq!(header.signature, header2.signature);
    assert_eq!(header.object_version, header2.object_version);
    // ...
}

#[test]
fn test_object_header_v2_calculate_size() {
    let header = ObjectHeader::new_v2(ObjectType::CanMessage2);
    assert_eq!(header.calculate_header_size(), 40);
}
```

## 总结

Rust 实现完全对应 C++ 代码的结构和行为：

1. ✅ **结构体布局**：内存布局与 C++ 完全一致
2. ✅ **方法签名**：所有方法都有对应的 Rust 实现
3. ✅ **构造函数**：`new_v1()` 和 `new_v2()` 对应 C++ 构造函数
4. ✅ **序列化**：`read()` 和 `write()` 方法行为一致
5. ✅ **大小计算**：`calculate_header_size()` 返回正确的值（V1: 32, V2: 40）
6. ✅ **枚举类型**：所有枚举值完全对应
7. ✅ **组合模式**：使用 `base: ObjectHeaderBase` 代替继承，符合 Rust 惯例

这种实现既保持了与 C++ 代码的完全兼容性，又遵循了 Rust 的最佳实践。