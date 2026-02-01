# Plot 功能调试指南

## 问题：点击 Plot 时程序崩溃

已添加详细的调试日志来帮助诊断问题。

## 如何查看调试日志

### Windows

1. **从命令行运行程序**
   ```powershell
   cd c:\Users\Administrator\RustroverProjects\canview
   cargo run --release -p view
   ```

2. **查看控制台输出**
   - 调试信息会输出到 stderr (标准错误输出)
   - 在命令行窗口中可以直接看到

### 调试日志内容

当点击 Plot 按钮时，会输出以下信息：

```
=== Extract Series Data ===
Selected signals: 3
Messages: 12345
DBC channels: 2
LDF channels: 1
  Parsing signal 0: CAN:123:EngineSpeed
    BusType: CAN, MsgID: 123, SignalName: EngineSpeed
  Parsing signal 1: CAN:456:VehicleSpeed
    BusType: CAN, MsgID: 456, SignalName: VehicleSpeed
  Parsing signal 2: LIN:16:BatteryVoltage
    BusType: LIN, MsgID: 16, SignalName: BatteryVoltage
Series after filtering: 3
  Series 0: EngineSpeed (CAN:0x7B) - 1234 points
  Series 1: VehicleSpeed (CAN:0x1C8) - 1234 points
  Series 2: BatteryVoltage (LIN:0x10) - 567 points
=== Extract Complete ===
```

## 常见崩溃原因和解决方案

### 1. 没有选择信号

**日志输出：**
```
=== Extract Series Data ===
Selected signals: 0
No signals selected
```

**解决方案：**
- 在 Library 视图中选择至少一个信号（点击信号前的复选框）

### 2. 没有加载 BLF 文件

**日志输出：**
```
=== Extract Series Data ===
Selected signals: 3
Messages: 0
No messages loaded
```

**解决方案：**
- 点击 "Open BLF" 按钮加载 BLF 日志文件

### 3. 信号 ID 格式错误

**日志输出：**
```
  Parsing signal 0: InvalidFormat
    Warning: Invalid signal ID format (expected BusType:MsgID:SignalName)
```

**解决方案：**
- 检查信号库配置
- 确保信号 ID 格式为：`BusType:MessageID:SignalName`
- 例如：`CAN:123:EngineSpeed` 或 `LIN:16:BatteryVoltage`

### 4. DBC/LDF 数据库未加载

**日志输出：**
```
=== Extract Series Data ===
Selected signals: 3
Messages: 1000
DBC channels: 0
LDF channels: 0
```

**解决方案：**
- 确保在 Library 视图中选择了版本
- 检查版本是否包含 DBC 或 LDF 文件
- 重新导入数据库文件

### 5. 数据点为空或无效

**日志输出：**
```
Series after filtering: 0
```

或者

```
Warning: Invalid data values in chart, skipping rendering
```

**可能原因：**
- BLF 文件中没有匹配的 Message ID
- 信号定义错误（例如 start_bit 或 size 不正确）
- DBC/LDF 文件与 BLF 文件不匹配

**解决方案：**
- 检查 BLF 文件是否包含所选信号的数据
- 验证 DBC/LDF 文件的 Message ID 是否与 BLF 中的 ID 匹配
- 在 Logs 视图中查看原始消息，确认 ID

### 6. 渲染崩溃

如果在渲染阶段崩溃，现在有以下保护机制：

1. **空系列检查**
   ```rust
   if series.is_empty() { return; }
   ```

2. **无效数值检查**
   ```rust
   if !min_t.is_finite() || !max_t.is_finite() { return; }
   ```

3. **零范围保护**
   ```rust
   let v_range = if max_v == min_v { 1.0 } else { max_v - min_v };
   ```

4. **边界检查**
   ```rust
   if content_bounds.size.width <= px(0.0) { return; }
   ```

## 调试步骤

1. **清空选择，重新开始**
   - 切换到 Library 视图
   - 取消所有信号选择
   - 重新选择 1-2 个信号测试

2. **验证数据完整性**
   - 在 Logs 视图中检查是否有消息
   - 确认消息的 ID 与选择的信号匹配

3. **逐步测试**
   - 先选择一个信号
   - 查看是否能成功绘图
   - 逐步增加信号数量

4. **检查文件匹配**
   - 确认 DBC/LDF 文件版本正确
   - 确认 BLF 文件来源与数据库匹配

## 性能说明

- **数据点限制**：每个信号最多显示 5000 个数据点（自动降采样）
- **降采样日志**：
  ```
  Subsampled EngineSpeed from 12000 to 4800 points
  ```

## 报告问题

如果仍然崩溃，请提供以下信息：

1. 完整的控制台输出（从 "=== Extract Series Data ===" 开始）
2. 选择的信号列表
3. BLF 文件大小和消息数量
4. 使用的 DBC/LDF 文件版本

## 临时解决方案

如果绘图功能不稳定，可以：

1. 使用 Logs 视图查看原始数据
2. 导出数据到 CSV 文件（如果实现）
3. 选择更少的信号（1-3个）
4. 使用较小的 BLF 文件测试

---

## 代码改进

本次添加的安全保护：

1. ✅ 空数据检查
2. ✅ 无效值检查（NaN, Infinity）
3. ✅ 零除保护
4. ✅ 边界检查
5. ✅ 详细的调试日志
6. ✅ 步长保护（防止 step_by(0)）

这些改进应该能防止大多数崩溃情况。
