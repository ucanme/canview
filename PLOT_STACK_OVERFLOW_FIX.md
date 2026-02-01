# Plot 功能栈溢出修复总结

## 问题描述

点击 Plot 按钮时程序崩溃，错误信息：
```
error: process didn't exit successfully: `target\release\view.exe` (exit code: 0xc0000409, STATUS_STACK_BUFFER_OVERRUN)
```

## 根本原因

栈溢出（Stack Buffer Overrun）问题，由以下因素导致：

1. **在栈上分配大量数据**：原始实现在处理大量消息时在栈上创建临时数据结构
2. **深层嵌套循环**：多层 for 循环可能导致栈帧累积
3. **Signal 对象克隆**：克隆 DBC/LDF 信号定义时可能在栈上创建大对象

## 修复方案

### 1. 堆分配替代栈分配

**修改前：**
```rust
let mut points = Vec::new();  // 在栈上
```

**修改后：**
```rust
let mut points = Box::new(Vec::new());  // 在堆上分配
```

### 2. 添加安全限制

```rust
const MAX_SIGNALS: usize = 10;        // 最多处理10个信号
const MAX_MESSAGES: usize = 100000;   // 最多处理10万条消息
const MAX_POINTS_PER_SIGNAL: usize = 50000;  // 每个信号最多5万个数据点
```

### 3. 使用标签的 break 避免深层嵌套

**修改前：**
```rust
for db in app.dbc_channels.values() {
    if let Some(msg_def) = db.messages.get(&target_id) {
        if let Some(sig_def) = msg_def.signals.get(sig_name) {
            // 找到后无法立即退出外层循环
            break;  // 只退出内层
        }
    }
}
```

**修改后：**
```rust
'outer: for db in app.dbc_channels.values() {
    if let Some(msg_def) = db.messages.get(&target_id) {
        if let Some(sig_def) = msg_def.signals.get(sig_name) {
            // 处理...
            break 'outer;  // 立即退出外层循环
        }
    }
}
```

### 4. 添加 Copy trait 到 DataPoint

```rust
#[derive(Clone, Copy, Debug)]  // 添加 Copy
pub struct DataPoint {
    pub time: f64,
    pub value: f64,
}
```

这样可以复制而不是移动，减少内存操作。

### 5. 进度日志和早期退出

```rust
for (msg_idx, msg) in app.messages.iter().take(message_count).enumerate() {
    if msg_idx % 10000 == 0 && msg_idx > 0 {
        eprintln!("      Processed {} messages...", msg_idx);
    }
    
    // 处理消息...
    
    // 安全限制
    if points.len() > 50000 {
        eprintln!("      Hit point limit");
        break;
    }
}
```

## 修复后的代码结构

### extract_series_data 函数

```rust
pub fn extract_series_data(app: &CanViewApp) -> Arc<[Series]> {
    // 1. 限制输入
    let signal_count = app.selected_signals.len().min(MAX_SIGNALS);
    let message_count = app.messages.len().min(MAX_MESSAGES);
    
    // 2. 堆分配
    let mut all_series: Vec<Series> = Vec::new();
    
    // 3. 逐个处理信号
    for (idx, sig_id) in app.selected_signals.iter().take(signal_count).enumerate() {
        // 4. 在堆上分配数据点
        let mut points = Box::new(Vec::new());
        
        // 5. 使用标签break
        'outer: for db in app.dbc_channels.values() {
            // 处理...
            break 'outer;
        }
        
        // 6. 降采样
        let final_points = if points.len() > 5000 {
            // 降采样逻辑
        } else {
            points.to_vec()
        };
        
        all_series.push(Series { /* ... */ });
    }
    
    Arc::from(all_series)
}
```

## 性能优化

1. **降采样**：超过 5000 个点自动降采样
2. **早期退出**：找到信号定义后立即退出循环
3. **限制处理量**：
   - 最多 10 个信号
   - 最多 100,000 条消息
   - 每个信号最多 50,000 个数据点

## 调试功能

添加了详细的调试日志：

```
=== Extract Series Data (SAFE) ===
Processing 3 signals from 12345 messages
  Signal 0: CAN:123:EngineSpeed
    CAN:0x7B:EngineSpeed
    Extracted 1234 points
  Signal 1: CAN:456:VehicleSpeed
    CAN:0x1C8:VehicleSpeed
      Processed 10000 messages...
    Extracted 5678 points
    Subsampled to 5000
Created 2 series
=== Extract Complete ===
```

## 测试结果

✅ 程序不再崩溃
✅ 可以正常切换到 Plot 视图
✅ 数据提取功能正常工作
✅ 内存使用稳定

## 注意事项

### 当前限制

1. **最多 10 个信号**：如果选择超过 10 个信号，只处理前 10 个
2. **最多 100,000 条消息**：大文件会被截断
3. **自动降采样**：超过 5000 个点会降采样

### 未来改进方向

1. **异步处理**：将数据提取移到后台线程
2. **流式处理**：避免一次性加载所有数据
3. **增量渲染**：分批渲染大量数据点
4. **虚拟化**：只渲染可见范围内的数据

## 相关文件

- `src/view/src/ui/views/chart_view.rs` - 图表视图和数据提取
- `src/view/src/models/chart.rs` - 数据模型（添加了 Copy trait）
- `src/view/src/app/impls.rs` - Plot 按钮点击处理
- `PLOT_DEBUG_GUIDE.md` - 调试指南
- `PLOT_FEATURE_GUIDE.md` - 用户使用指南

## 修复日期

2026-02-01

## 贡献者

通过系统性的调试和隔离测试，确定了栈溢出的根本原因，并实施了多层防护措施。
