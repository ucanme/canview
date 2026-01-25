# BLF 时间解析使用指南

## 问题解决

✅ **已修复**: BLF 文件中所有 message 时间显示相同的问题

## 原理说明

BLF 文件的时间由两部分组成：

1. **基准时间** (`measurement_start_time`): 文件头中的绝对时间
2. **偏移时间** (`object_time_stamp`): 每个对象相对于基准时间的偏移（纳秒）

**实际时间 = 基准时间 + 偏移时间**

## 使用方法

### 1. 基础用法

```rust
use blf::read_blf_from_file;

// 读取 BLF 文件
let blf_result = read_blf_from_file("path/to/file.blf")?;

// 获取基准时间
println!("测量开始时间: {}", blf_result.measurement_start_time_str());

// 遍历所有对象并显示实际时间
for (i, obj) in blf_result.objects.iter().enumerate() {
    let relative_ns = obj.timestamp();  // 相对时间（纳秒）
    let absolute_time = blf_result.format_timestamp(relative_ns);
    
    println!("Message {}: {}", i, absolute_time);
}
```

### 2. 获取绝对时间戳

```rust
// 获取 Unix 时间戳（纳秒）
let relative_ns = obj.timestamp();
let absolute_ns = blf_result.to_absolute_timestamp_ns(relative_ns);

// 转换为秒
let absolute_seconds = absolute_ns as f64 / 1_000_000_000.0;
println!("时间戳: {:.6} 秒", absolute_seconds);
```

### 3. 格式化时间字符串

```rust
// 方式 1: 使用 BlfResult 的方法
let time_str = blf_result.format_timestamp(obj.timestamp());
// 输出: "2026-01-25 16:30:45.123456"

// 方式 2: 直接使用 SystemTime
let time_str = blf_result.file_stats.measurement_start_time
    .format_with_offset(obj.timestamp());
```

### 4. 计算时间差

```rust
// 计算两个消息之间的时间差
let msg1_time = blf_result.objects[0].timestamp();
let msg2_time = blf_result.objects[1].timestamp();

let time_diff_ns = msg2_time - msg1_time;
let time_diff_ms = time_diff_ns as f64 / 1_000_000.0;

println!("时间差: {:.3} ms", time_diff_ms);
```

### 5. 过滤特定时间范围的消息

```rust
use chrono::{DateTime, Utc, TimeZone};

// 定义时间范围
let start_time = Utc.with_ymd_and_hms(2026, 1, 25, 10, 0, 0).unwrap();
let end_time = Utc.with_ymd_and_hms(2026, 1, 25, 12, 0, 0).unwrap();

let start_ns = start_time.timestamp_nanos_opt().unwrap();
let end_ns = end_time.timestamp_nanos_opt().unwrap();

// 过滤消息
let filtered_messages: Vec<_> = blf_result.objects.iter()
    .filter(|obj| {
        let abs_ns = blf_result.to_absolute_timestamp_ns(obj.timestamp());
        abs_ns >= start_ns && abs_ns <= end_ns
    })
    .collect();

println!("找到 {} 条消息在指定时间范围内", filtered_messages.len());
```

## 完整示例

```rust
use blf::{read_blf_from_file, LogObject};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 读取 BLF 文件
    let blf_result = read_blf_from_file("test.blf")?;
    
    println!("=== BLF 文件信息 ===");
    println!("测量开始时间: {}", blf_result.measurement_start_time_str());
    println!("对象总数: {}", blf_result.objects.len());
    println!();
    
    // 2. 显示前 10 条消息的时间
    println!("=== 前 10 条消息 ===");
    for (i, obj) in blf_result.objects.iter().take(10).enumerate() {
        let time_str = blf_result.format_timestamp(obj.timestamp());
        
        match obj {
            LogObject::CanMessage(msg) => {
                println!("[{}] CAN Message - ID: 0x{:X}, Time: {}", 
                    i, msg.id, time_str);
            }
            LogObject::CanMessage2(msg) => {
                println!("[{}] CAN Message2 - ID: 0x{:X}, Time: {}", 
                    i, msg.id, time_str);
            }
            _ => {
                println!("[{}] Other - Time: {}", i, time_str);
            }
        }
    }
    
    // 3. 统计时间分布
    println!("\n=== 时间统计 ===");
    if let (Some(first), Some(last)) = (blf_result.objects.first(), blf_result.objects.last()) {
        let duration_ns = last.timestamp() - first.timestamp();
        let duration_s = duration_ns as f64 / 1_000_000_000.0;
        
        println!("记录时长: {:.3} 秒", duration_s);
        println!("平均消息间隔: {:.6} ms", 
            duration_s * 1000.0 / blf_result.objects.len() as f64);
    }
    
    Ok(())
}
```

## API 参考

### BlfResult 方法

| 方法 | 说明 | 返回值 |
|------|------|--------|
| `to_absolute_timestamp_ns(relative_ns)` | 转换为绝对时间戳 | `i64` (纳秒) |
| `format_timestamp(relative_ns)` | 格式化时间字符串 | `String` |
| `measurement_start_time_str()` | 获取基准时间字符串 | `String` |

### SystemTime 方法

| 方法 | 说明 | 返回值 |
|------|------|--------|
| `to_timestamp_nanos()` | 转换为 Unix 时间戳 | `i64` (纳秒) |
| `add_nanoseconds(offset_ns)` | 添加偏移量 | `i64` (纳秒) |
| `format()` | 格式化基准时间 | `String` |
| `format_with_offset(offset_ns)` | 格式化绝对时间 | `String` |

### LogObject 方法

| 方法 | 说明 | 返回值 |
|------|------|--------|
| `timestamp()` | 获取相对时间戳 | `u64` (纳秒) |
| `channel()` | 获取通道 ID | `Option<u16>` |

## 时间格式

### 基准时间格式
```
YYYY-MM-DD HH:MM:SS.mmm
示例: 2026-01-25 16:30:45.123
```

### 绝对时间格式
```
YYYY-MM-DD HH:MM:SS.ffffff
示例: 2026-01-25 16:30:45.123456
```

## 注意事项

1. **时间单位**: 所有偏移时间都是纳秒（ns）
2. **时区**: 所有时间都是 UTC
3. **精度**: 支持微秒级精度（6 位小数）
4. **溢出**: 使用 `i64` 存储时间戳，支持到 2262 年

## 性能优化

### 批量处理

```rust
// 预先计算所有时间戳
let timestamps: Vec<String> = blf_result.objects.iter()
    .map(|obj| blf_result.format_timestamp(obj.timestamp()))
    .collect();

// 后续直接使用
for (i, time_str) in timestamps.iter().enumerate() {
    println!("{}: {}", i, time_str);
}
```

### 使用数值而非字符串

```rust
// 如果只需要比较或计算，使用数值更快
let timestamps_ns: Vec<i64> = blf_result.objects.iter()
    .map(|obj| blf_result.to_absolute_timestamp_ns(obj.timestamp()))
    .collect();

// 计算平均值
let avg_ns = timestamps_ns.iter().sum::<i64>() / timestamps_ns.len() as i64;
```

## 测试验证

```rust
#[test]
fn test_timestamp_conversion() {
    use blf::file_statistics::SystemTime;
    
    let base_time = SystemTime {
        year: 2026,
        month: 1,
        day: 25,
        day_of_week: 6,
        hour: 16,
        minute: 30,
        second: 0,
        milliseconds: 0,
    };
    
    // 测试：1 秒后
    let offset_ns = 1_000_000_000u64;  // 1 秒 = 10^9 纳秒
    let result = base_time.format_with_offset(offset_ns);
    
    assert!(result.contains("16:30:01"));
    println!("✅ 时间转换正确: {}", result);
}
```

## 故障排除

### 问题：时间显示为 "Invalid time"

**原因**: 日期无效（如 2月30日）

**解决**: 检查 BLF 文件的 `measurement_start_time` 是否正确

### 问题：时间都是 1970-01-01

**原因**: 基准时间为 0 或无效

**解决**: 
```rust
// 检查基准时间
println!("基准时间: {:?}", blf_result.file_stats.measurement_start_time);
```

### 问题：时间差异很大

**原因**: 可能是时区问题或文件损坏

**解决**: 验证文件完整性，检查时区设置

## 相关文件

- `src/blf/src/file_statistics.rs` - SystemTime 实现
- `src/blf/src/file.rs` - BlfResult 实现
- `src/blf/src/parser.rs` - LogObject 定义
- `BLF_TIMESTAMP_FIX.md` - 修复说明文档

---

**更新日期**: 2026-01-25  
**版本**: 1.0  
**状态**: ✅ 已实现并测试
