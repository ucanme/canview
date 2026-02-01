# Plot 功能临时禁用说明

## 当前状态

Plot 功能因栈溢出问题已被**临时禁用**。

**禁用方式：**
- Plot 按钮已从界面中移除（在 `src/view/src/app/impls.rs` 中被注释）
- Plot 视图代码保留，但无法访问

## 问题原因

经过详细调试和隔离测试，确定问题在于：

1. ⚠️ **`extract_series_data` 函数导致栈溢出**
   - 即使使用堆分配（`Box<Vec>`）
   - 即使限制数据量
   - 即使简化数据处理逻辑

2. ⚠️ **使用 gpui-component 的 LineChart 也会崩溃**
   - 不是自定义绘制代码的问题
   - 库组件也无法避免崩溃

3. ✅ **UI 渲染本身没有问题**
   - 纯文本显示正常
   - 空数据时不崩溃

## 测试过的解决方案

| 方案 | 结果 | 说明 |
|------|------|------|
| 堆分配数据 | ❌ 失败 | 使用 `Box<Vec>` 仍然崩溃 |
| 限制数据量 | ❌ 失败 | 限制到 10 个信号、1000 个点仍崩溃 |
| 使用标签 break | ❌ 失败 | 优化循环结构无效 |
| 禁用 canvas 绘制 | ❌ 失败 | 纯文本也崩溃 |
| 使用 gpui-component | ❌ 失败 | LineChart 组件也崩溃 |
| 完全禁用数据提取 | ✅ 成功 | 不调用 extract_series_data 即可正常运行 |

## 临时解决方案

**方案 A：移除 Plot 功能**（当前）
```rust
// 在 src/view/src/app/impls.rs 中
// Plot 按钮已被注释，无法访问
```

**方案 B：延后实现**
- 将 Plot 功能移到后续版本
- 优先保证其他功能稳定

## 可能的长期解决方案

### 1. 异步处理（推荐）

将数据提取移到后台线程：

```rust
pub fn start_extract_series_data(app: &mut CanViewApp, cx: &mut Context<CanViewApp>) {
    // 在后台线程处理
    cx.spawn(|view, mut cx| async move {
        // 数据提取逻辑
        let series = extract_on_background_thread().await;
        
        view.update(&mut cx, |this, cx| {
            this.plot_data = series;
            cx.notify();
        }).ok();
    }).detach();
}
```

### 2. 分批加载

不一次性加载所有数据：

```rust
// 每次只加载一个信号
// 使用状态机逐步加载
pub struct PlotLoadingState {
    current_signal: usize,
    loaded_series: Vec<Series>,
}
```

### 3. 使用外部工具

- 导出数据到 CSV
- 使用专业绘图工具（如 Python + matplotlib）

### 4. 重写为 C/C++ 扩展

如果 Rust 栈限制是问题，考虑：
- 用 C/C++ 实现数据处理
- 通过 FFI 调用

## 重新启用 Plot 功能的步骤

如果要尝试重新启用（不推荐，直到找到解决方案）：

1. **取消注释 Plot 按钮**
   ```rust
   // 在 src/view/src/app/impls.rs 约 3122 行
   // 移除 /* */ 注释
   ```

2. **启用数据提取**
   ```rust
   // 在按钮点击处理中
   this.plot_data = crate::ui::views::chart_view::extract_series_data(this);
   ```

3. **测试小数据集**
   - 只选择 1-2 个信号
   - 使用小型 BLF 文件（< 1000 条消息）

⚠️ **警告：** 即使这样也可能崩溃！

## 相关文件

- `src/view/src/app/impls.rs` - Plot 按钮定义（已注释）
- `src/view/src/ui/views/chart_view.rs` - Plot 视图实现
- `src/view/src/app/state.rs` - PlotView 枚举和 plot_data 字段
- `src/view/src/models/chart.rs` - Series 和 DataPoint 数据模型

## 建议

**当前建议：保持 Plot 功能禁用**

理由：
1. 其他功能（Logs、Library）工作正常
2. Plot 功能存在严重的稳定性问题
3. 需要更多时间研究根本原因

**未来计划：**
1. 调查 Windows 栈大小限制
2. 研究异步处理方案
3. 考虑使用外部绘图工具

## 日期

2026-02-01

---

**注意：** 如果需要查看信号数据，可以：
1. 在 Logs 视图中查看原始消息
2. 使用过滤功能筛选特定信号
3. 等待未来版本的稳定 Plot 功能
