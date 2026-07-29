# 2026-07-29 — task.md 三项反馈修复设计

## 背景与范围

`task.md` 收到用户 3 条新反馈（替换了之前的 5 条列表）。本 spec 处理全部 3 项：

| # | 反馈 | 本 spec 处理 |
|---|---|---|
| 1 | 在信号库通道信息 name 输入框按回车后光标仍在闪，需要不闪 | ✅ |
| 2 | 在已激活信号库某通道上替换了文件，plot 中加载的选项数据没有同步更新 | ✅（仅修复同步，不加替换按钮） |
| 3 | 折线图单卡片高度不高、背景虚线占空间、上下空白多、上方还有信号名，可用区域不多，看图费劲 | ✅ |

## 用户决策摘要

- **Issue 1**：`channel_name` 输入框按回车 → blur 该输入框（不跳焦、不提交），光标自然消失。表单保持打开，用户接着点 "Select File..." 选文件。`channel_id` 输入框按回车 → 保持现状（跳焦到 `channel_name`）。
- **Issue 2**：仅修复同步，不加「替换文件」按钮。三个写库入口（`save_channel_config` / `delete_channel` / `apply_version_to_mappings`）末尾触发 plot 数据重提。
- **Issue 3**：`render_single_chart` 用 `gpui::canvas()` 完全自绘，不画虚线网格；保留 X 轴 + Y 轴 + 动态时间刻度；卡片尺寸 360px + p_2 + py_1 + xs 标题；canvas 内部 padding 上 4 / 下 14 / 左 36 / 右 4，最大化折线显示区域。
- **blur API**：用 `window.blur(cx)`（gpui 公开 API），不修改 gpui-component InputState。

## 架构

三个 Issue 分布在三个文件，互不耦合，可独立实现：

1. **`src/view/src/app/state.rs`**：`PendingAddChannelFocus` 枚举改名 `ChannelConfirm` → `ChannelBlur`
2. **`src/view/src/ui/views/library_management.rs`**：`channel_name_input` 的 PressEnter 分支改设 `ChannelBlur`
3. **`src/view/src/app/impls_rendering.rs`**：render 中处理 `ChannelBlur` 分支，调 `window.blur(cx)`
4. **`src/view/src/app/impls.rs`**：三个写库入口末尾调 `extract_and_update_series_data`
5. **`src/view/src/ui/views/chart_view.rs`**：重写 `render_single_chart` 用 `gpui::canvas()` 自绘

## 组件设计

### Issue 1 — channel_name 回车后光标不闪

**当前问题**：

`library_management.rs:1351-1356` 在 `channel_name_input` 收到 `PressEnter` 时，设：
```rust
this.pending_add_channel_focus = Some(crate::app::PendingAddChannelFocus::ChannelConfirm);
```

`impls_rendering.rs:1668-1672` 在 render 中处理 `ChannelConfirm`：
```rust
PendingAddChannelFocus::ChannelConfirm => {
    if let Some(name_input) = &self.channel_name_input {
        name_input.update(cx, |state, cx| state.focus(window, cx));  // ← 焦点又放回 name 输入框
    }
}
```

这就是「光标继续闪」的根源 —— PressEnter 后焦点重新放回 name 输入框。

**修复**：

1. **`src/view/src/app/state.rs:283`** —— `enum PendingAddChannelFocus` 变体改名：

   当前：
   ```rust
   pub enum PendingAddChannelFocus {
       ChannelName,
       ChannelConfirm,
   }
   ```

   改为：
   ```rust
   pub enum PendingAddChannelFocus {
       ChannelName,
       ChannelBlur,  // 原 ChannelConfirm，语义改为 blur name 输入框
   }
   ```

   `state.rs:426` 初始化 `pending_add_channel_focus: None` 不变。

2. **`src/view/src/ui/views/library_management.rs:1351-1356`** —— `channel_name_input` 的 PressEnter 分支：

   改为：
   ```rust
   gpui_component::input::InputEvent::PressEnter { .. } => {
       this.pending_add_channel_focus =
           Some(crate::app::PendingAddChannelFocus::ChannelBlur);
       cx.notify();
   }
   ```

   `channel_id_input` 的 PressEnter 分支（1332-1337）保持现状。

3. **`src/view/src/app/impls_rendering.rs:1655-1674`** —— render 中处理 `ChannelBlur`：

   改 `ChannelConfirm` 分支为 `ChannelBlur`：
   ```rust
   PendingAddChannelFocus::ChannelBlur => {
       // 让窗口失焦，name 输入框光标自然停止闪烁
       window.blur(cx);
   }
   ```

   `window.blur(cx)` 是 gpui 公开 API（`Window::blur(&mut self, cx)`），让窗口当前焦点元素失焦。

**影响检查**：

- `PendingAddChannelFocus::ChannelConfirm` 在代码库中只在 `library_management.rs:1354`（写入）和 `impls_rendering.rs:1668`（读取）两处使用。改名后两处同步即可，无其他引用。
- `app/mod.rs:12` 的 `pub use state::{..., PendingAddChannelFocus}` 不变（enum 名不变，只是变体改）。

### Issue 2 — plot 数据同步

**当前问题**：

- `impls.rs:1786 save_channel_config` 成功添加通道后，在 1994-1998 行调用 `apply_version_to_mappings`（仅当当前版本是激活版本）—— 这函数会 `internal_load_library_version` 重新加载 DBC 到 `app.dbc_channels`，但**不刷新 `plot_data`**。
- `impls.rs:2011 delete_channel` 从 `dbc_channels` / `ldf_channels` 移除通道，但**不刷新 `plot_data`**。
- `impls.rs:1357 apply_version_to_mappings` 更新 mappings + 重新加载 DBC，**不刷新 `plot_data`**。

结果：用户切换到 Plot 视图前，`plot_data` 仍是旧 DBC 提取的旧信号集；侧边栏（`extract_signal_items` 直接读 `dbc_channels`）会变，但当前已绘制的图表区不变。即使切换 Plot 视图，`status_bar.rs:767` 调用 `extract_and_update_series_data` 是按 `selected_signals`（旧 ID）过滤，新 DBC 若信号名不同则匹配不上，看不到新数据。

**修复**：三个函数末尾调用 `crate::ui::views::chart_view::extract_and_update_series_data(self)`。

1. **`impls.rs:1357 apply_version_to_mappings`** —— 在末尾 `cx.notify()` 之前（1417 行附近）加：

   ```rust
   // 刷新 plot 数据：库版本变了，已选信号对应的 series 要重新从新 DBC 提取
   crate::ui::views::chart_view::extract_and_update_series_data(self);

   self.status_msg = format!("✅ Applied version {} to all plot channels", version_name).into();
   cx.notify();
   ```

   不加激活版本判断 —— `apply_version_to_mappings` 本身就是「应用到 mappings」必然影响 plot。

2. **`impls.rs:1786 save_channel_config`** —— 在 `apply_version_to_mappings` 调用之后、`cx.notify()` 之前（1997-2000 行附近）加：

   ```rust
   if is_active_version {
       self.apply_version_to_mappings(&library_id.clone(), &version_name.clone(), cx);
   }
   // 无条件刷新 plot 数据，确保新通道立即在 plot 中可见
   crate::ui::views::chart_view::extract_and_update_series_data(self);

   cx.notify();
   ```

   根据用户决策「三个函数都要调用」。`apply_version_to_mappings` 内部已刷一次 plot_data，这里再刷一次（重复刷新按用户决策接受，性能影响可忽略 —— `extract_and_update_series_data` 是 `O(已选信号 × 消息数)`，已选信号通常 ≤ 20）。

3. **`impls.rs:2011 delete_channel`** —— 在末尾 `cx.notify()` 之前（2091 行附近）加版本激活判断：

   ```rust
   let is_active_version = self.active_library_id.as_deref() == Some(library_id.as_str())
       && self.active_version_name.as_deref() == Some(version_name.as_str());
   if is_active_version {
       crate::ui::views::chart_view::extract_and_update_series_data(self);
   }

   self.status_msg = match cleanup_result { ... };
   cx.notify();
   ```

   删除非激活版本上的通道不应刷新 plot（plot 跟激活版本绑定），所以加判断。

**为什么不在 `internal_load_library_version` 末尾统一刷新**：

用户决策选择「每个写库命令末尾触发」而非「在 internal_load_library_version 统一触发」。理由是显式调用点能清楚看出「这里库变了，刷新图」的因果关系，避免 internal_load_library_version 的副作用扩散到所有调用方（比如 `config_controller.rs:88` 启动时加载库的路径也会触发刷新，那是没必要的）。

### Issue 3 — 折线图自绘

**当前 `render_single_chart`**（`chart_view.rs:720-810`）：

```rust
div().h(px(250.0)).p_4().border_1().rounded_lg()
    .child(div().text_sm().font_weight(SEMIBOLD).text_color(series.color)
        .child(format!("{} {} | {} pts | ...", series.name, unit, ...)))
    .child(div().flex_1().py_2().child(LineChart::new(...).x(...).y(...).stroke(...).linear().tick_margin(1)))
```

问题：
- 卡片 250px + p_4（16px×2=32px padding）+ 内层 py_2（8px×2=16px） = 250 - 32 - 16 = 202px 给折线
- 顶部标题占一行（~20px），实际折线区 ~180px
- LineChart 内部硬编码 `Grid::new().y((0..=3).map(...)).stroke(border).dash_array(&[4, 2])` 画 4 条水平虚线
- LineChart 内部 Y 轴强制从 0 开始（`ScaleLinear::chain(Some(Y::zero()))`），数值小的信号折线压在底部

**新设计 — 完全用 `gpui::canvas()` 自绘**：

#### 卡片尺寸

- `.h(px(360.0))`（原 250px，加高 110px）
- `.p_2()`（原 p_4，外层 padding 8px）
- 内层 chart wrapper：`.py_1()`（原 py_2，4px）
- 标题：`.text_xs()`（原 text_sm，12px）

#### canvas 自绘布局

```
┌──────────────────────────────────────────────────────┐ p_2 (8px outer)
│ EngineSpeed [km/h] | 100 pts | 0.123s-12.345s       │ ← text_xs 标题
│ ┌────────────────────────────────────────────────┐  │ py_1 (4px)
│ │ 240┆                          ╱─────           │  │
│ │    │            ╱─────╲     ╱                  │  │ canvas 内部:
│ │ 120┆      ╱───╱        ╲──╱                    │  │   上 4px / 下 14px
│ │    │ ───╱                                   ●  │  │   左 36px / 右 4px
│ │   0┆──────────────────────────────────────●──── │  │
│ │    └──┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┘  │
│ │     0s    2s    4s    6s    8s   10s   12s        │ ← X 轴时间标签
│ └────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────┘
```

#### canvas paint 实现骨架

```rust
struct ChartLayout {
    bounds: Bounds<Pixels>,
    min_t: f64, max_t: f64, t_range: f64,
    min_v: f64, max_v: f64, v_range: f64,
}

fn render_single_chart(
    series: &Series,
    _start_time: Option<chrono::NaiveDateTime>,
    show_points: bool,
) -> impl IntoElement {
    let points = series.points.clone();
    let color = series.color;
    let time_labels = series.time_labels.clone();
    let title = format!(
        "{} {} | {} pts",
        series.name.split(':').last().unwrap_or(&series.name),
        series.unit.as_ref().map(|u| format!("[{}]", u)).unwrap_or_default(),
        series.points.len(),
    );

    div()
        .flex()
        .flex_col()
        .h(px(360.0))
        .bg(rgb(0x18181b))
        .border_1()
        .border_color(rgb(0x27272a))
        .rounded_lg()
        .p_2()
        .child(div().text_xs().text_color(color).child(title))
        .child(
            div()
                .flex_1()
                .py_1()
                .child(gpui::canvas(
                    move |bounds, _window, _cx| {
                        // prepaint: 预计算坐标变换参数
                        if points.is_empty() { return None; }
                        let (min_t, max_t) = (points.first().unwrap().time, points.last().unwrap().time);
                        let (min_v, max_v) = points.iter().fold(
                            (f64::INFINITY, f64::NEG_INFINITY),
                            |(mn, mx), p| (mn.min(p.value), mx.max(p.value)),
                        );
                        let v_range = (max_v - min_v).max(1e-9);
                        let t_range = (max_t - min_t).max(1e-9);
                        Some(ChartLayout { bounds, min_t, max_t, min_v, max_v, t_range, v_range })
                    },
                    move |_bounds, layout, window, cx| {
                        let layout = match layout { Some(l) => l, None => return };
                        let bounds = layout.bounds;
                        let pad_left = px(36.0);
                        let pad_right = px(4.0);
                        let pad_top = px(4.0);
                        let pad_bottom = px(14.0);
                        let chart_w = bounds.size.width - pad_left - pad_right;
                        let chart_h = bounds.size.height - pad_top - pad_bottom;

                        // 1. Y 轴（左侧垂直线 + 3 个数值刻度）
                        // 2. X 轴（底部水平线 + 动态刻度）
                        // 3. 折线（PathBuilder::stroke）
                        // 4. 可选 data points（paint_quad 小圆）
                    },
                ))
        )
}
```

注：`gpui::canvas` 的 prepaint 返回 `T`，paint 接收 `T`。`points.is_empty()` 时返回 `None` 跳过 paint。

#### Y 轴绘制

```rust
// Y 轴线
let y_axis_x = bounds.origin.x + pad_left;
let y_axis_top = bounds.origin.y + pad_top;
let y_axis_bottom = bounds.origin.y + pad_top + chart_h;
let stroke = cx.theme().border;  // 或固定 rgb(0x3f3f46)
window.paint_path(
    PathBuilder::stroke(px(1.)).move_to(point(y_axis_x, y_axis_top)).line_to(point(y_axis_x, y_axis_bottom)).build().unwrap(),
    stroke,
);

// 3 个 Y 轴刻度（max / mid / min）
for (i, label) in [layout.max_v, (layout.max_v + layout.min_v) / 2.0, layout.min_v].iter().enumerate() {
    let y_px = y_axis_top + (chart_h * i as f32 / 2.0);
    // 画刻度横线 + 在 y_px 右对齐显示 label（用 window.paint_text 或 paint_glyph）
}
```

注：gpui 文本绘制 API 是 `window.paint_text(text, origin, font_size, color, ...)`. 实现时确认签名。

#### X 轴绘制（动态刻度）

```rust
let x_axis_y = bounds.origin.y + pad_top + chart_h;
let x_axis_left = bounds.origin.x + pad_left;
let x_axis_right = bounds.origin.x + pad_left + chart_w;

// X 轴线
window.paint_path(
    PathBuilder::stroke(px(1.)).move_to(point(x_axis_left, x_axis_y)).line_to(point(x_axis_right, x_axis_y)).build().unwrap(),
    stroke,
);

// 动态刻度数量：每 80px 一个刻度，clamp [2, 6]
let n_ticks = ((chart_w.as_f32() / 80.0).floor() as usize).clamp(2, 6);
for i in 0..n_ticks {
    let t = layout.min_t + layout.t_range * (i as f64 / (n_ticks - 1) as f64);
    let x_px = x_axis_left + px((chart_w.as_f32() * i as f32 / (n_ticks - 1) as f32));
    // 画刻度短线 + 在 x_px 居中显示 time_labels 对应的时间
    // 时间标签：优先用 series.time_labels 里离 t 最近的，否则用 format_time_label(t, t_range)
}
```

#### 时间标签格式

复用 `series.time_labels`（已预格式化，见 `chart_view.rs:1133-1196`）。如果 `time_labels` 为空（zoom 后可能），用本地 fallback：

```rust
fn format_time_label(t: f64, span: f64) -> String {
    if span < 60.0 { format!("{:.3}s", t) }
    else if span < 3600.0 { format!("{:.1}s", t) }
    else { format!("{:.1}min", t / 60.0) }
}
```

#### 折线绘制

```rust
let mut builder = PathBuilder::stroke(px(1.5));
let mut started = false;
for p in &points {
    let x = x_axis_left + px(((p.time - layout.min_t) / layout.t_range * chart_w.as_f32() as f64) as f32);
    let y = y_axis_top + px(((layout.max_v - p.value) / layout.v_range * chart_h.as_f32() as f64) as f32);
    if !started {
        builder.move_to(point(x, y));
        started = true;
    } else {
        builder.line_to(point(x, y));
    }
}
if let Ok(path) = builder.build() {
    window.paint_path(path, color);
}
```

#### data points（可选）

`show_points` 为 true 时，每个点画一个小圆（半径 3px）：

```rust
if show_points {
    for p in &points {
        let x = ...; let y = ...;
        window.paint_quad(
            gpui::quad(point(x - px(3.), y - px(3.)), size(px(6.), px(6.)),
            gpui::Corners::all(px(3.)),
            color,
            gpui::Edges::default(),
        );
    }
}
```

#### Y 轴范围不从 0 开始

LineChart 原实现强制 `ScaleLinear::chain(Some(Y::zero()))`，让 Y 轴从 0 开始 —— 对小数值信号折线压在底部。新实现用 `min_v / max_v` 真实数据范围，折线占满整个 chart_h（除非 min_v == max_v 时加 fallback 范围）。

## 数据流

### Issue 1 数据流

```
用户在 channel_name 输入框按回车
  → InputState 触发 PressEnter 事件
  → library_management.rs 的 cx.subscribe 回调执行
  → 设 pending_add_channel_focus = Some(ChannelBlur), cx.notify()
  → render 触发
  → impls_rendering.rs render 函数读到 ChannelBlur
  → 调用 window.blur(cx)
  → name 输入框失焦，光标停止闪烁
```

### Issue 2 数据流

```
用户在激活版本上操作通道（add / delete / 切换版本）
  → save_channel_config / delete_channel / apply_version_to_mappings 执行
  → 写库 → 更新 dbc_channels / ldf_channels → 写 config 文件
  → 末尾调 extract_and_update_series_data(self)
  → 重新从 messages + dbc_channels 提取 series 到 plot_data / plot_full_data
  → cx.notify() 触发重绘
  → Plot 视图立即显示新 DBC 的信号
```

### Issue 3 数据流

```
render_chart_canvas → 遍历 app.selected_signals
  → 对每个 signal_id 在 series_data 中查找 series
  → 找到则调 render_single_chart(series, ...)
  → 内部用 gpui::canvas() 自绘：标题 + Y 轴 + X 轴 + 折线（无网格）
  → 找不到则调 render_no_data_chart(signal_id)（保持不变）
```

## 测试

### 单元测试（已有模式，参考 `chart_view.rs:1298-1316`）

1. **Issue 1**：无需单测 —— 行为是 UI 焦点切换，依赖 window/cx 环境，难单测。靠手动验证。
2. **Issue 2**：可选 —— 难单测，因 `extract_and_update_series_data` 需要 `Context<CanViewApp>` 和完整 app 状态。靠手动验证。
3. **Issue 3**：
   - `calc_x_tick_count(width)` 纯函数单测：`width=400 → 4`, `width=80 → 1 → clamp 2`, `width=600 → 6 → clamp 6`
   - `format_time_label(t, span)` 纯函数单测：`span=30 → "12.345s"`, `span=300 → "5.0s"`, `span=4000 → "1.1min"`

### 手动验证清单

1. **Issue 1**：
   - Library → Add Channel → 输入 channel_id 按 Enter → 焦点跳到 name 输入框（保持原行为）
   - 输入 channel_name 按 Enter → name 输入框光标消失（不闪）✅
   - 表单仍打开，可继续点 "Select File..." 选文件
   - 选完文件后点 ✓ Confirm 仍可正常提交

2. **Issue 2**：
   - 激活某 library version → Plot 视图选几个信号 → 看到折线
   - 切到 Library → 在激活版本上 Add Channel 添加新通道（选不同 DBC 文件）→ 切回 Plot → 侧边栏显示新通道 + 信号
   - 勾选新通道的信号 → 折线立即出现（无需手动刷新）✅
   - 在激活版本上 Delete Channel 删除某通道 → 切回 Plot → 该通道信号从侧边栏消失，对应折线消失 ✅
   - 切换激活版本（Library 页点 Apply to Plot）→ Plot 侧边栏 + 折线立即更新 ✅

3. **Issue 3**：
   - Plot 视图选 1-3 个有数据的信号 → 单卡片高度 360px，折线占大部分区域
   - 上下左右非折线区域尽量小（外层 p_2、内层 py_1、canvas padding 4/14/36/4）
   - 背景无虚线网格 ✅
   - X 轴有时间刻度（数量随卡片宽度变化，2-6 个）
   - Y 轴有数值刻度（max / mid / min）
   - 标题 text_xs，含信号名 + unit + 点数
   - show_points 切换 → 折线上有/无小圆点

## 风险与注意

1. **`window.blur()` 签名**：实现时需确认 `Window::blur` 的实际签名（`&mut self` 还是 `&mut self, cx: &mut Context`）。从 gpui 0.2.2 源码看是 `pub fn blur(&mut self)`（无 cx 参数），但项目用的 gpui 版本可能不同 —— 实现前先 grep 确认。spec 中写作 `window.blur(cx)` 是占位，实现时按实际签名调整。

2. **`window.paint_text` / `paint_glyph` API**：gpui 文本绘制 API 在不同版本签名不同。实现时若 `paint_text` 不可用，用 `window.paint_quad` + 预渲染文本贴图，或退化为不画刻度文本（保留轴 + 折线）。

3. **`window.paint_path` 路径**：`PathBuilder::stroke(px(1.)).move_to(...).line_to(...).build()` 返回 `Result<Path, ...>`，需要 `unwrap_or` 处理。空数据集（`points.is_empty()`）时直接跳过 paint。

4. **`PendingAddChannelFocus::ChannelConfirm` 改名**：检查所有引用点。grep 已确认只有 `library_management.rs:1354`（写入）和 `impls_rendering.rs:1668`（读取）两处。

5. **重复刷新 plot_data**：用户已确认接受 `save_channel_config` 和 `apply_version_to_mappings` 重复调用 `extract_and_update_series_data`。若实际出现性能问题（大日志 + 多次刷新），后续可优化为只刷一次。

6. **`extract_and_update_series_data` 调用时机**：在 `cx.notify()` 之前调用，确保重绘时用的是新 `plot_data`。

## 不做的事

- 不加「替换文件」按钮（Issue 2 用户决策）
- 不改 `render_no_data_chart` / `render_legend`（保持视觉一致，最小改动）
- 不动态计算 Y 轴宽度（用户决策「采用确定的尺寸」）
- 不写运行时切换开关（去掉网格是默认且唯一行为）
- 不优化重复刷新（接受用户决策）
