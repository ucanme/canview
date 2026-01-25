# BLF 时间解析修复指南

## 问题描述

当前解析 BLF 文件时，所有帧的时间戳都显示为相同的值，没有正确计算每一帧的实际时间。

## BLF 时间戳机制

### 时间组成

BLF 文件中的时间由两部分组成：

1. **基准时间** (`measurement_start_time`): 
   - 位于文件头的 `FileStatistics` 对象中
   - 表示测量开始的绝对时间
   - 格式：`SystemTime` 结构（年、月、日、时、分、秒、毫秒）

2. **偏移时间** (`object_time_stamp`):
   - 位于每个对象的 `ObjectHeader` 中
   - 表示相对于基准时间的偏移量（单位：纳秒）
   - 格式：`u64` (纳秒)

### 实际时间计算

```
实际时间 = 基准时间 + (偏移时间 / 1,000,000,000)
```

## 当前实现分析

### 文件结构

```
src/blf/src/
├── file_statistics.rs      # 包含 measurement_start_time
├── object_header.rs         # 包含 object_time_stamp  
├── parser.rs                # 解析逻辑
└── objects/                 # 各种对象类型
```

### 关键代码位置

#### 1. 基准时间（FileStatistics）
**文件**: `src/blf/src/file_statistics.rs`

```rust
pub struct FileStatistics {
    // ...
    pub measurement_start_time: SystemTime,  // 基准时间
    // ...
}
```

#### 2. 偏移时间（ObjectHeader）
**文件**: `src/blf/src/object_header.rs`

```rust
pub struct ObjectHeader {
    // ...
    pub object_time_stamp: u64,  // 偏移时间（纳秒）
    // ...
}
```

#### 3. 对象时间戳
**文件**: `src/blf/src/parser.rs`

```rust
pub struct LogContainer {
    pub timestamp: u64,  // 直接使用 object_time_stamp
    // ...
}
```

## 问题根源

当前代码直接使用 `object_time_stamp` 作为时间戳，没有加上基准时间，导致：
- 所有时间戳都是相对时间（偏移量）
- 没有转换为实际的绝对时间
- 显示时可能都显示为相同的值（如果偏移量很小）

## 修复方案

### 方案 1: 在解析时计算绝对时间

修改 `parser.rs` 中的 `parse_log_container` 方法，传入基准时间并计算绝对时间。

```rust
// 在 BlfFile 中添加方法
impl BlfFile {
    pub fn get_absolute_timestamp(&self, relative_ns: u64) -> chrono::DateTime<chrono::Utc> {
        // 将 SystemTime 转换为 DateTime
        let base_time = self.file_stats.measurement_start_time.to_datetime();
        
        // 添加偏移量（纳秒）
        base_time + chrono::Duration::nanoseconds(relative_ns as i64)
    }
}
```

### 方案 2: 在显示时计算

在 UI 层显示时，使用基准时间 + 偏移时间计算实际时间。

```rust
// 在显示逻辑中
fn format_timestamp(base_time: &SystemTime, offset_ns: u64) -> String {
    let base_dt = base_time.to_datetime();
    let actual_time = base_dt + Duration::nanoseconds(offset_ns as i64);
    actual_time.format("%Y-%m-%d %H:%M:%S%.3f").to_string()
}
```

### 方案 3: 扩展 LogContainer 结构

添加绝对时间字段，同时保留相对时间。

```rust
pub struct LogContainer {
    pub relative_timestamp_ns: u64,  // 相对时间（纳秒）
    pub absolute_timestamp: Option<DateTime<Utc>>,  // 绝对时间
    // ...
}
```

## 推荐实现

### 步骤 1: 添加时间转换工具

**文件**: `src/blf/src/objects/system.rs`

```rust
impl SystemTime {
    /// 转换为 chrono::DateTime
    pub fn to_datetime(&self) -> chrono::DateTime<chrono::Utc> {
        chrono::Utc.ymd(
            self.year as i32,
            self.month as u32,
            self.day as u32
        ).and_hms_milli(
            self.hour as u32,
            self.minute as u32,
            self.second as u32,
            self.milliseconds as u32
        )
    }
    
    /// 添加纳秒偏移
    pub fn add_nanoseconds(&self, offset_ns: u64) -> chrono::DateTime<chrono::Utc> {
        let base = self.to_datetime();
        base + chrono::Duration::nanoseconds(offset_ns as i64)
    }
}
```

### 步骤 2: 在 BlfFile 中添加时间计算方法

**文件**: `src/blf/src/file.rs`

```rust
impl BlfFile {
    /// 将相对时间戳转换为绝对时间
    pub fn to_absolute_time(&self, relative_ns: u64) -> chrono::DateTime<chrono::Utc> {
        self.file_stats.measurement_start_time.add_nanoseconds(relative_ns)
    }
    
    /// 格式化时间戳为字符串
    pub fn format_timestamp(&self, relative_ns: u64) -> String {
        let dt = self.to_absolute_time(relative_ns);
        dt.format("%Y-%m-%d %H:%M:%S%.6f").to_string()
    }
}
```

### 步骤 3: 在 UI 中使用

**文件**: `src/view/src/...`（UI 代码）

```rust
// 显示时间时
let timestamp_str = blf_file.format_timestamp(log_container.timestamp);
println!("Time: {}", timestamp_str);

// 或者获取 DateTime 对象进行进一步处理
let absolute_time = blf_file.to_absolute_time(log_container.timestamp);
```

## 测试验证

### 测试用例

```rust
#[test]
fn test_timestamp_calculation() {
    let base_time = SystemTime {
        year: 2026,
        month: 1,
        day_of_week: 6,  // Saturday
        day: 25,
        hour: 15,
        minute: 30,
        second: 0,
        milliseconds: 0,
    };
    
    // 测试：1秒后的时间戳
    let offset_ns = 1_000_000_000u64;  // 1秒 = 10^9 纳秒
    let result = base_time.add_nanoseconds(offset_ns);
    
    assert_eq!(result.hour(), 15);
    assert_eq!(result.minute(), 30);
    assert_eq!(result.second(), 1);
}
```

### 验证步骤

1. **解析 BLF 文件**
   ```rust
   let blf = BlfFile::from_file("sample.blf")?;
   println!("Base time: {:?}", blf.file_stats.measurement_start_time);
   ```

2. **检查第一帧时间**
   ```rust
   if let Some(first_log) = blf.log_containers.first() {
       println!("Relative: {} ns", first_log.timestamp);
       println!("Absolute: {}", blf.format_timestamp(first_log.timestamp));
   }
   ```

3. **验证时间递增**
   ```rust
   for (i, log) in blf.log_containers.iter().take(10).enumerate() {
       println!("Frame {}: {}", i, blf.format_timestamp(log.timestamp));
   }
   ```

## 注意事项

1. **时区处理**: SystemTime 可能是本地时间或 UTC，需要确认
2. **纳秒精度**: 确保不会溢出，使用 `i64` 或 `u64`
3. **性能**: 如果频繁转换，考虑缓存结果
4. **显示格式**: 根据需要调整时间格式字符串

## 相关文件

- `src/blf/src/file_statistics.rs` - 基准时间定义
- `src/blf/src/object_header.rs` - 偏移时间定义
- `src/blf/src/objects/system.rs` - SystemTime 结构
- `src/blf/src/parser.rs` - 解析逻辑
- `src/blf/src/file.rs` - BlfFile 主结构

## 下一步

1. [ ] 实现 `SystemTime::to_datetime()` 方法
2. [ ] 实现 `SystemTime::add_nanoseconds()` 方法
3. [ ] 在 `BlfFile` 中添加时间转换方法
4. [ ] 更新 UI 显示逻辑
5. [ ] 添加单元测试
6. [ ] 验证实际 BLF 文件

---

**创建日期**: 2026-01-25  
**状态**: 📝 待实现
