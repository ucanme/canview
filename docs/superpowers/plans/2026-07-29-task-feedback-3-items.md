# task.md 三项反馈修复 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 修复 task.md 三项用户反馈：channel_name 回车不闪、库变更后 plot 数据同步、折线图自绘去网格加高。

**Architecture:** 三个独立 Issue 分别改 5 个文件。Issue 1 改 enum + 两个调用点（blur name 输入框）。Issue 2 在三个写库命令末尾调 `extract_and_update_series_data`。Issue 3 重写 `render_single_chart` 用 `gpui::canvas()` 自绘，不画网格。

**Tech Stack:** Rust + GPUI (zed git) + gpui-component 0.5.0；项目用 `cargo run --bin view`。

## Global Constraints

- 依赖固定：`gpui = { git = "https://github.com/zed-industries/zed", features = ["runtime_shaders"] }`，`gpui-component = "0.5.0"`（见 `Cargo.toml:9-10`、`src/view/Cargo.toml:9-10`）—— 不要改依赖版本
- 不加「替换文件」按钮（用户明确决策）
- 不改 `render_no_data_chart` / `render_legend`（保持视觉一致）
- 不写无关注释；注释只在 WHY 不明显时
- `window.blur()` 签名是 `pub fn blur(&mut self)`（无 cx 参数）—— 已在 `zed-a70e2ad075855582/ee0e370/crates/gpui/src/window.rs:1579` 确认
- 文本绘制用 `gpui_component::plot::label::Text` + `PlotLabel::paint(bounds, window, cx)`（不是 `window.paint_text` —— 这个 API 不存在）
- 路径绘制用 `gpui::PathBuilder::stroke(px(N))` + `window.paint_path(path, color)`
- 矩形/圆点绘制用 `gpui::quad(point, size)` + `window.paint_quad(quad)`
- 测试运行：`cargo test --workspace -p view`（或 `cargo test -p view --lib` 跑单元测试）
- 编译检查：`cargo build --bin view` 或 `cargo check -p view`
- 提交信息格式：`feat(scope): ...` / `fix(scope): ...`，含 `Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>`

---

## Task 1: Issue 1 — channel_name 回车后失焦

**Files:**
- Modify: `src/view/src/app/state.rs:283`（`PendingAddChannelFocus` 枚举）
- Modify: `src/view/src/ui/views/library_management.rs:1351-1356`（PressEnter 订阅）
- Modify: `src/view/src/app/impls_rendering.rs:1655-1674`（render 处理 `ChannelBlur`）

**Interfaces:**
- Consumes: `gpui::Window::blur(&mut self)`（已存在，无 cx 参数）
- Produces: `PendingAddChannelFocus::ChannelBlur` 变体（替代 `ChannelConfirm`）

- [ ] **Step 1: 改 `PendingAddChannelFocus` 枚举变体**

打开 `src/view/src/app/state.rs`，定位 `pub enum PendingAddChannelFocus`（约 283 行）。当前：

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
    ChannelBlur,
}
```

注意：只改 `ChannelConfirm` → `ChannelBlur`，不动 `ChannelName`。

- [ ] **Step 2: 改 `channel_name_input` 的 PressEnter 分支**

打开 `src/view/src/ui/views/library_management.rs`，定位 1351 行附近 `channel_name_input` 的 subscribe 回调。当前：

```rust
let name_input =
    cx.new(|cx| InputState::new(window, cx).placeholder("Channel name"));
cx.subscribe(&name_input, |this, input, event, cx| {
    match event {
        gpui_component::input::InputEvent::Change => {
            this.new_channel_name = input.read(cx).text().to_string();
        }
        gpui_component::input::InputEvent::PressEnter { .. } => {
            // Defer focus to next render where window is available
            this.pending_add_channel_focus =
                Some(crate::app::PendingAddChannelFocus::ChannelConfirm);
            cx.notify();
        }
        _ => {}
    }
})
.detach();
```

把 `ChannelConfirm` 改为 `ChannelBlur`：

```rust
gpui_component::input::InputEvent::PressEnter { .. } => {
    // 让 render 在下次帧调 window.blur()，name 输入框光标自然消失
    this.pending_add_channel_focus =
        Some(crate::app::PendingAddChannelFocus::ChannelBlur);
    cx.notify();
}
```

`channel_id_input` 的 PressEnter 分支（1332-1337）保持 `ChannelName` 不变。

- [ ] **Step 3: 改 render 中的 `ChannelConfirm` 分支**

打开 `src/view/src/app/impls_rendering.rs`，定位 1655 行附近 `if let Some(target) = self.pending_add_channel_focus.take()`。当前：

```rust
if let Some(target) = self.pending_add_channel_focus.take() {
    use crate::app::PendingAddChannelFocus;
    match target {
        PendingAddChannelFocus::ChannelName => {
            if let Some(name_input) = &self.channel_name_input {
                name_input.update(cx, |state, cx| state.focus(window, cx));
            }
        }
        PendingAddChannelFocus::ChannelConfirm => {
            if let Some(name_input) = &self.channel_name_input {
                name_input.update(cx, |state, cx| state.focus(window, cx));
            }
        }
    }
}
```

把 `ChannelConfirm` 分支改为 `ChannelBlur`，调 `window.blur()`：

```rust
PendingAddChannelFocus::ChannelBlur => {
    // 让窗口失焦，name 输入框光标自然停止闪烁
    window.blur();
}
```

注意：`window.blur()` 无 cx 参数。删掉 `if let Some(name_input) = &self.channel_name_input { ... }` 包裹。

`ChannelName` 分支保持不变。

- [ ] **Step 4: 编译检查**

运行：`cargo check -p view 2>&1 | tail -20`
预期：编译通过，无 `ChannelConfirm` 未解析符号错误。

如果有 `cannot find variant ChannelConfirm` 错误，说明还有遗漏的引用 —— 用 `grep -rn "ChannelConfirm" src/view/src` 找到，全部改为 `ChannelBlur`。

- [ ] **Step 5: 手动验证**

运行：`cargo run --release --bin view`

操作步骤：
1. Library → 选中某 library → 选中某 version → 点 "+ Add Channel" 按钮
2. 在 "Channel ID" 输入框输入数字（如 `1`），按 Enter → 焦点应跳到 "Channel name" 输入框（保持原行为）
3. 在 "Channel name" 输入框输入字符（如 `engine`），按 Enter → **name 输入框光标立即消失（不闪）**
4. 表单仍然打开，"Channel name" 输入框不可输入（已失焦），可继续点 "Select File..." 选文件
5. 选完文件后点 ✓ Confirm 仍能正常提交

- [ ] **Step 6: 提交**

```bash
git add src/view/src/app/state.rs src/view/src/ui/views/library_management.rs src/view/src/app/impls_rendering.rs
git commit -m "$(cat <<'EOF'
fix(library): Enter in channel_name blurs input instead of refocusing

Was: PressEnter set ChannelConfirm → render refocused name input (cursor kept blinking)
Now: PressEnter sets ChannelBlur → render calls window.blur() (cursor stops)

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

## Task 2: Issue 2 — plot 数据同步（三处写库命令末尾刷新）

**Files:**
- Modify: `src/view/src/app/impls.rs:1417`（`apply_version_to_mappings` 末尾）
- Modify: `src/view/src/app/impls.rs:1997-2000`（`save_channel_config` 成功分支末尾）
- Modify: `src/view/src/app/impls.rs:2090-2091`（`delete_channel` 末尾）

**Interfaces:**
- Consumes: `crate::ui::views::chart_view::extract_and_update_series_data(&mut self)`（已存在 pub fn，见 `chart_view.rs:1267`）
- Consumes: `self.active_library_id: Option<String>`、`self.active_version_name: Option<String>`（见 `state.rs`）
- Produces: 三个写库命令完成后 `plot_data` / `plot_full_data` 自动重提

- [ ] **Step 1: `apply_version_to_mappings` 末尾加刷新**

打开 `src/view/src/app/impls.rs`，定位 `apply_version_to_mappings` 函数（约 1357 行）。函数末尾当前：

```rust
        // Load into memory
        self.internal_load_library_version(1, library_id, version_name);

        // Save config
        self.save_config(cx);

        self.status_msg =
            format!("✅ Applied version {} to all plot channels", version_name).into();
        cx.notify();
    }
```

在 `cx.notify();` 之前加一行：

```rust
        // Load into memory
        self.internal_load_library_version(1, library_id, version_name);

        // Save config
        self.save_config(cx);

        // 刷新 plot 数据：库版本变了，已选信号对应的 series 要重新从新 DBC 提取
        crate::ui::views::chart_view::extract_and_update_series_data(self);

        self.status_msg =
            format!("✅ Applied version {} to all plot channels", version_name).into();
        cx.notify();
    }
```

- [ ] **Step 2: `save_channel_config` 末尾加刷新**

同文件，定位 `save_channel_config` 函数（约 1786 行）的成功分支末尾（约 1994-2000 行）。当前：

```rust
                    // Auto-reload if this version is currently active
                    let is_active_version = self.active_library_id.as_deref() == Some(library_id.as_str())
                        && self.active_version_name.as_deref() == Some(version_name.as_str());
                    if is_active_version {
                        self.apply_version_to_mappings(&library_id.clone(), &version_name.clone(), cx);
                    }

                    cx.notify();
                }
```

在 `cx.notify();` 之前加一行（无条件刷新）：

```rust
                    // Auto-reload if this version is currently active
                    let is_active_version = self.active_library_id.as_deref() == Some(library_id.as_str())
                        && self.active_version_name.as_deref() == Some(version_name.as_str());
                    if is_active_version {
                        self.apply_version_to_mappings(&library_id.clone(), &version_name.clone(), cx);
                    }

                    // 刷新 plot 数据（即使是非激活版本路径，也是 no-op 无害）
                    crate::ui::views::chart_view::extract_and_update_series_data(self);

                    cx.notify();
                }
```

- [ ] **Step 3: `delete_channel` 末尾加刷新（带激活版本判断）**

同文件，定位 `delete_channel` 函数（约 2011 行）。函数末尾当前（约 2083-2093 行）：

```rust
            self.status_msg = match cleanup_result {
                Ok(()) => format!("Channel {} deleted", channel_id).into(),
                Err(e) => format!(
                    "Channel {} deleted, but local files cleanup failed: {}",
                    channel_id, e
                )
                .into(),
            };
            cx.notify();
        }
    }
```

在 `self.status_msg = match cleanup_result {` 之前加激活版本判断 + 刷新：

```rust
            // 若删除的是激活版本的通道，刷新 plot 数据
            let is_active_version = self.active_library_id.as_deref() == Some(library_id.as_str())
                && self.active_version_name.as_deref() == Some(version_name.as_str());
            if is_active_version {
                crate::ui::views::chart_view::extract_and_update_series_data(self);
            }

            self.status_msg = match cleanup_result {
                Ok(()) => format!("Channel {} deleted", channel_id).into(),
                Err(e) => format!(
                    "Channel {} deleted, but local files cleanup failed: {}",
                    channel_id, e
                )
                .into(),
            };
            cx.notify();
        }
    }
```

注意：`library_id` 和 `version_name` 在 `delete_channel` 函数顶部已绑定（约 2012-2020 行 `let library_id = ...; let version_name = ...;`），直接用即可。

- [ ] **Step 4: 编译检查**

运行：`cargo check -p view 2>&1 | tail -20`
预期：编译通过，无错误。

- [ ] **Step 5: 手动验证**

运行：`cargo run --release --bin view`

**Scenario A — 添加新通道同步：**
1. 加载 sample.blf → Library 页 → 创建一个 library + version → Add Channel 选 sample.dbc → Apply to Plot 激活
2. 切到 Plot → 侧边栏展开通道 → 勾选某信号（如 `EngineSpeed`）→ 折线显示
3. 切回 Library → 在同一激活版本上 "+ Add Channel" 加第二个通道（选另一个 DBC 文件，或同一文件 channel_id=2）→ 点 ✓ Confirm 提交
4. 立即切到 Plot → 侧边栏出现新通道（channel_id=2）+ 新信号
5. 勾选新通道的信号 → 折线立即出现（无需切走再切回）

**Scenario B — 删除通道同步：**
1. 同上，激活版本含 2 个通道
2. Plot 视图勾选 channel 1 + channel 2 的信号 → 都有折线
3. 切到 Library → 删除 channel 1（点 🗑）
4. 切到 Plot → channel 1 的信号从侧边栏消失，对应折线消失，channel 2 折线保留

**Scenario C — 切换激活版本同步：**
1. 创建 2 个 version，version A 用 sample.dbc，version B 用 test_can.dbc
2. 激活 version A → Plot 选信号 → 折线
3. 切到 Library → 选中 version B → 点 "Apply to Plot"
4. 切到 Plot → 侧边栏信号集变成 version B 的信号（旧 A 信号已勾选的若 B 中同名则保留，否则消失）

- [ ] **Step 6: 提交**

```bash
git add src/view/src/app/impls.rs
git commit -m "$(cat <<'EOF'
fix(library): refresh plot_data after library mutations

save_channel_config, delete_channel, apply_version_to_mappings
now call extract_and_update_series_data so the plot reflects DBC
changes without manual view-switch refresh.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

## Task 3: Issue 3 — 折线图自绘纯函数（坐标 + 刻度算法）

**Files:**
- Modify: `src/view/src/ui/views/chart_view.rs`（新增两个纯函数 + 单测）
- Test: 同文件 `#[cfg(test)] mod tests` 块

**Interfaces:**
- Consumes: 无
- Produces:
  - `pub fn calc_x_tick_count(chart_w_px: f32) -> usize` — 按 80px/刻度计算 X 轴刻度数，clamp [2, 6]
  - `pub fn format_time_label(t: f64, span: f64) -> String` — 时间格式化 fallback

这两个纯函数为 Task 4 的 canvas paint 提供坐标算法，先单测锁定行为。

- [ ] **Step 1: 加纯函数实现 + 单测**

打开 `src/view/src/ui/views/chart_view.rs`，定位文件末尾的 `#[cfg(test)] mod tests`（约 1298 行）。在 `mod tests` 之前插入两个纯函数：

```rust
/// 按 80px/刻度估算 X 轴刻度数量，clamp [2, 6]。
/// chart_w_px 是 canvas 内可用于画折线的水平像素（已扣除左右 padding）。
pub fn calc_x_tick_count(chart_w_px: f32) -> usize {
    let approx = (chart_w_px / 80.0).floor() as usize;
    approx.clamp(2, 6)
}

/// 时间标签 fallback（series.time_labels 为空时用）。
/// span 是 max_t - min_t（秒）。
/// < 60s → 三位小数秒；< 1h → 一位小数秒；否则 → 一位小数分钟。
pub fn format_time_label(t: f64, span: f64) -> String {
    if span < 60.0 {
        format!("{:.3}s", t)
    } else if span < 3600.0 {
        format!("{:.1}s", t)
    } else {
        format!("{:.1}min", t / 60.0)
    }
}
```

然后在 `mod tests` 块末尾（`}` 之前）加单测：

```rust
    #[test]
    fn calc_x_tick_count_basic() {
        assert_eq!(calc_x_tick_count(400.0), 5);
        assert_eq!(calc_x_tick_count(80.0), 2);   // 80px → 1 → clamp 2
        assert_eq!(calc_x_tick_count(40.0), 2);   // 40px → 0 → clamp 2
        assert_eq!(calc_x_tick_count(600.0), 6);  // 600/80=7.5 → 7 → clamp 6
        assert_eq!(calc_x_tick_count(160.0), 2);
        assert_eq!(calc_x_tick_count(320.0), 4);
    }

    #[test]
    fn format_time_label_ranges() {
        assert_eq!(format_time_label(12.345, 30.0), "12.345s");
        assert_eq!(format_time_label(5.0, 300.0), "5.0s");
        assert_eq!(format_time_label(120.0, 4000.0), "2.0min");
    }
```

- [ ] **Step 2: 跑测试验证通过**

运行：`cargo test -p view --lib calc_x_tick_count_basic format_time_label_ranges -- --nocapture 2>&1 | tail -20`
预期：两个测试 PASS。

- [ ] **Step 3: 提交**

```bash
git add src/view/src/ui/views/chart_view.rs
git commit -m "$(cat <<'EOF'
feat(plot): add calc_x_tick_count + format_time_label pure fns with tests

Pure helpers for canvas-based chart rendering in next commit.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

## Task 4: Issue 3 — `render_single_chart` 用 gpui::canvas 自绘

**Files:**
- Modify: `src/view/src/ui/views/chart_view.rs:720-810`（替换 `render_single_chart`）

**Interfaces:**
- Consumes:
  - `crate::models::{DataPoint, Series}`（`models/chart.rs`，字段 `name / unit / points: Vec<DataPoint> / color: Hsla / time_labels: Vec<String>`）
  - `DataPoint { time: f64, value: f64, index: usize }`
  - `gpui::canvas(prepaint, paint)` —— `prepaint: FnOnce(Bounds, &mut Window, &mut App) -> T`，`paint: FnOnce(Bounds, T, &mut Window, &mut App)`
  - `gpui::PathBuilder::stroke(px(N))` —— 返回 `Self`；`move_to(&mut self, point)` 和 `line_to(&mut self, point)` 返回 `()`；`build(self) -> Result<Path, Error>`。**注意：`move_to` / `line_to` 不能链式调用**，必须分语句：`let mut b = PathBuilder::stroke(px(1.)); b.move_to(p1); b.line_to(p2); let path = b.build();`
  - `gpui::Window::paint_path(path, color)`、`Window::paint_quad(PaintQuad)`
  - `gpui::point(x, y) -> Point<Pixels>`、`gpui::px(N) -> Pixels`、`gpui::size(w, h) -> Size<Pixels>`、`gpui::Bounds::new(origin, size)`、`gpui::fill(bounds, background) -> PaintQuad`
  - `gpui_component::plot::label::Text` —— `Text::new(text, origin, color).font_size(px).align(TextAlign)`
  - `gpui_component::plot::PlotLabel` —— `PlotLabel::new(Vec<Text>).paint(bounds, window, cx)`
- Consumes (本 plan Task 3): `calc_x_tick_count(chart_w_px: f32) -> usize`
- Produces: 重写后的 `render_single_chart(series, _start_time, show_points) -> impl IntoElement`

**布局参数（确认值，不要改）：**
- 卡片高度 `px(360.0)`（原 250）
- 卡片外层 `p_2()`（原 p_4，8px）
- 内层 chart wrapper `py_1()`（原 py_2，4px），不加 px
- 标题 `text_xs()`（原 text_sm）
- canvas 内部 padding：上 4 / 下 14 / 左 36 / 右 4（像素）

- [ ] **Step 1: 替换 `render_single_chart` 整个函数**

打开 `src/view/src/ui/views/chart_view.rs`，定位 `fn render_single_chart(`（约 720 行）。整个函数（720-810 行）替换为下面的实现。

注意：保持函数签名不变，调用方（`render_chart_canvas` 约 201 行）不感知内部实现。

```rust
/// Render a single chart for one signal — gpui canvas self-drawn, no grid.
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
                        // prepaint: 预计算坐标变换参数；空数据返回 None 跳过 paint
                        if points.is_empty() {
                            return None;
                        }
                        let min_t = points.first().unwrap().time;
                        let max_t = points.last().unwrap().time;
                        let (min_v, max_v) = points.iter().fold(
                            (f64::INFINITY, f64::NEG_INFINITY),
                            |(mn, mx), p| (mn.min(p.value), mx.max(p.value)),
                        );
                        let v_range = (max_v - min_v).max(1e-9);
                        let t_range = (max_t - min_t).max(1e-9);
                        Some(ChartLayout {
                            bounds,
                            min_t,
                            max_t,
                            min_v,
                            max_v,
                            t_range,
                            v_range,
                        })
                    },
                    move |_bounds, layout, window, cx| {
                        let layout = match layout {
                            Some(l) => l,
                            None => return,
                        };
                        let bounds = layout.bounds;
                        let pad_left = px(36.0);
                        let pad_right = px(4.0);
                        let pad_top = px(4.0);
                        let pad_bottom = px(14.0);
                        let chart_w = bounds.size.width - pad_left - pad_right;
                        let chart_h = bounds.size.height - pad_top - pad_bottom;

                        // 坐标系原点（左下角）和右上角
                        let origin_x = bounds.origin.x + pad_left;
                        let origin_y = bounds.origin.y + pad_top; // 顶部 Y
                        let x_axis_y = bounds.origin.y + pad_top + chart_h; // 底部 X 轴线 Y

                        let stroke_color = cx.theme().border;

                        // === 1. Y 轴线 ===
                        {
                            let mut b = PathBuilder::stroke(px(1.));
                            b.move_to(point(origin_x, origin_y));
                            b.line_to(point(origin_x, x_axis_y));
                            if let Ok(path) = b.build() {
                                window.paint_path(path, stroke_color);
                            }
                        }

                        // === 2. X 轴线 ===
                        {
                            let mut b = PathBuilder::stroke(px(1.));
                            b.move_to(point(origin_x, x_axis_y));
                            b.line_to(point(bounds.origin.x + pad_left + chart_w, x_axis_y));
                            if let Ok(path) = b.build() {
                                window.paint_path(path, stroke_color);
                            }
                        }

                        // === 3. Y 轴刻度 + 标签（max / mid / min，3 个）===
                        let y_values = [layout.max_v, (layout.max_v + layout.min_v) / 2.0, layout.min_v];
                        let mut y_labels: Vec<gpui_component::plot::label::Text> = Vec::with_capacity(3);
                        for (i, v) in y_values.iter().enumerate() {
                            let y_px = origin_y + chart_h * (i as f32 / 2.0);
                            // 短刻度线（向左 4px）
                            {
                                let mut b = PathBuilder::stroke(px(1.));
                                b.move_to(point(origin_x - px(4.), y_px));
                                b.line_to(point(origin_x, y_px));
                                if let Ok(path) = b.build() {
                                    window.paint_path(path, stroke_color);
                                }
                            }
                            // 标签文本（在 origin_x - 6px 处右对齐，垂直居中于 y_px）
                            let label = format!("{:.1}", v);
                            y_labels.push(
                                gpui_component::plot::label::Text::new(
                                    label,
                                    point(origin_x - px(6.), y_px - px(5.)),
                                    cx.theme().muted_foreground,
                                )
                                .font_size(px(10.))
                                .align(gpui::TextAlign::Right),
                            );
                        }
                        let y_plot_label = gpui_component::plot::PlotLabel::new(y_labels);
                        y_plot_label.paint(&bounds, window, cx);

                        // === 4. X 轴刻度 + 标签（动态数量）===
                        let n_ticks = calc_x_tick_count(chart_w.as_f32());
                        let mut x_labels: Vec<gpui_component::plot::label::Text> = Vec::with_capacity(n_ticks);
                        for i in 0..n_ticks {
                            let ratio = if n_ticks == 1 { 0.0 } else { i as f32 / (n_ticks - 1) as f32 };
                            let x_px = origin_x + chart_w * ratio;
                            // 短刻度线（向下 4px）
                            {
                                let mut b = PathBuilder::stroke(px(1.));
                                b.move_to(point(x_px, x_axis_y));
                                b.line_to(point(x_px, x_axis_y + px(4.)));
                                if let Ok(path) = b.build() {
                                    window.paint_path(path, stroke_color);
                                }
                            }
                            // 时间标签：优先用 time_labels 索引采样，否则用 format_time_label fallback
                            let t = layout.min_t + layout.t_range * ratio as f64;
                            let label_text = if !time_labels.is_empty() {
                                // 按 ratio 在 time_labels 里采样
                                let idx = ((ratio * time_labels.len() as f32) as usize)
                                    .min(time_labels.len() - 1);
                                time_labels[idx].clone()
                            } else {
                                format_time_label(t, layout.t_range)
                            };
                            x_labels.push(
                                gpui_component::plot::label::Text::new(
                                    label_text,
                                    point(x_px, x_axis_y + px(4.)),
                                    cx.theme().muted_foreground,
                                )
                                .font_size(px(10.))
                                .align(gpui::TextAlign::Center),
                            );
                        }
                        let x_plot_label = gpui_component::plot::PlotLabel::new(x_labels);
                        x_plot_label.paint(&bounds, window, cx);

                        // === 5. 折线 ===
                        let mut builder = PathBuilder::stroke(px(1.5));
                        let mut started = false;
                        for p in &points {
                            let x = origin_x
                                + px(
                                    ((p.time - layout.min_t) / layout.t_range
                                        * chart_w.as_f32() as f64) as f32,
                                );
                            let y = origin_y
                                + px(
                                    ((layout.max_v - p.value) / layout.v_range
                                        * chart_h.as_f32() as f64) as f32,
                                );
                            if !started {
                                builder.move_to(point(x, y));
                                started = true;
                            } else {
                                builder.line_to(point(x, y));
                            }
                        }
                        if started {
                            if let Ok(path) = builder.build() {
                                window.paint_path(path, color);
                            }
                        }

                        // === 6. data points（可选，show_points 为 true）===
                        if show_points {
                            for p in &points {
                                let x = origin_x
                                    + px(
                                        ((p.time - layout.min_t) / layout.t_range
                                            * chart_w.as_f32() as f64) as f32,
                                    );
                                let y = origin_y
                                    + px(
                                        ((layout.max_v - p.value) / layout.v_range
                                            * chart_h.as_f32() as f64) as f32,
                                    );
                                // 6px 实心方块（用 gpui::fill 简化）
                                let dot_bounds = gpui::Bounds::new(
                                    point(x - px(3.), y - px(3.)),
                                    size(px(6.), px(6.)),
                                );
                                window.paint_quad(gpui::fill(dot_bounds, color));
                            }
                        }
                    },
                )),
        )
}

/// Canvas 自绘坐标变换参数
struct ChartLayout {
    bounds: gpui::Bounds<gpui::Pixels>,
    min_t: f64,
    max_t: f64,
    t_range: f64,
    min_v: f64,
    max_v: f64,
    v_range: f64,
}
```

注意：
- `ChartLayout` 在文件作用域定义（紧贴 `render_single_chart` 之后）。
- 不再使用 `gpui_component::chart::LineChart`。
- 删除原 `render_single_chart` 里的 `time_labels` / `min_time` / `max_time` / `time_span` / `total_points` / `label_step` 等局部变量 —— 旧逻辑由新 paint 闭包内的坐标计算替代。
- 不再使用 `use gpui_component::chart::LineChart;`（文件顶部 import 若不再被引用会变 unused —— 让 cargo check 报出来再删，避免漏删仍被其他函数引用的）。

- [ ] **Step 2: 处理编译错误**

运行：`cargo check -p view 2>&1 | tail -40`

预期可能的错误及修复：

1. `unused import: gpui_component::chart::LineChart` —— 检查 chart_view.rs 顶部 `use` 语句，若 LineChart 在文件其他地方（render_no_data_chart / render_legend）不再被引用，删除该 use 行。若仍被引用，保留。

2. `cannot find type ChartLayout` —— 确认 `struct ChartLayout {...}` 已加在 `render_single_chart` 函数之后、其他函数之前。

3. `Text::new` 参数类型不匹配 —— `gpui_component::plot::label::Text::new(text, origin, color)` 中 origin 是 `Point<T>` where T 可被 `Into<Pixels>`。我们传 `point(origin_x - px(6.), y_px - px(5.))` 是 `Point<Pixels>`，应能编译。如果不行，看错误信息调整。

4. `gpui::fill(bounds, color)` —— 已确认是 `pub fn fill(bounds: impl Into<Bounds<Pixels>>, background: impl Into<Background>) -> PaintQuad`。`gpui::Bounds::new(origin, size)` 已确认存在。如果 `fill` 未在 `use gpui::*` 导出，改用 `gpui::fill` 全路径。

- [ ] **Step 3: 编译通过后跑全量测试**

运行：`cargo test -p view --lib 2>&1 | tail -20`
预期：所有测试 PASS（包括 Task 3 的两个新测试）。

- [ ] **Step 4: 手动验证 — 折线图视觉效果**

运行：`cargo run --release --bin view`

操作：
1. 加载 sample.blf → Library 激活含 sample.dbc 的 version
2. 切到 Plot → 侧边栏勾选若干信号（如 EngineSpeed、RPM）→ 看到折线
3. **检查项**：
   - 单卡片高度明显比之前高（360px vs 250px）
   - 折线区域占卡片大部分空间（外层 p_2、内层 py_1、canvas padding 4/14/36/4 都很小）
   - **背景无虚线网格**（无 4 条水平虚线）
   - 标题字号小（text_xs），含信号名 + unit + 点数
   - Y 轴左侧有 3 个数值刻度（max / mid / min），刻度线 + 文本
   - X 轴底部有时间刻度（数量 2-6 个，取决于卡片宽度），刻度线 + 文本
   - 切换 Points: ON → 折线上有 6px 圆点
4. 多选几个信号（4+）→ 滚动卡片 → 每个卡片视觉一致

- [ ] **Step 5: 手动验证 — 无数据卡片不变**

侧边栏勾选一个不存在的信号 ID（比如手动删 channels 后旧 selected_signals 项）→ 应该看到 `render_no_data_chart` 占位卡片（与本任务改动无关，但确认没破坏）。

- [ ] **Step 6: 提交**

```bash
git add src/view/src/ui/views/chart_view.rs
git commit -m "$(cat <<'EOF'
feat(plot): self-draw chart canvas — no grid, taller card, dynamic ticks

Replaces gpui-component LineChart with gpui::canvas() paint:
- Card height 250px → 360px, p_4 → p_2, py_2 → py_1, text_sm → text_xs
- Removes hardcoded Grid (dashed horizontal lines)
- Y axis: 3 ticks (max/mid/min) with labels
- X axis: 2-6 ticks based on chart width (calc_x_tick_count)
- Y range uses real min/max, not forced-from-zero

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

## Task 5: 全量回归 + 编译清理

**Files:**
- 无修改，仅检查

- [ ] **Step 1: 全 workspace 编译**

运行：`cargo build --bin view 2>&1 | tail -10`
预期：编译通过，0 错误 0 警告。

如果有 warning（unused imports / dead code），逐一处理：
- `unused import: gpui_component::chart::LineChart` → 删 use 行
- `unused import: ...` 其他 → 删
- `dead_code: ...` → 不动，可能是原本就有的

- [ ] **Step 2: 全 workspace 测试**

运行：`cargo test --workspace 2>&1 | tail -30`
预期：所有测试 PASS。

- [ ] **Step 3: clippy 静态检查**

运行：`cargo clippy --workspace 2>&1 | tail -20`
预期：无新增 clippy 警告（已有的不要修）。

- [ ] **Step 4: 三项手动端到端验证**

运行：`cargo run --release --bin view`

按 Task 1 / Task 2 / Task 4 的 Step 5/6 手动验证清单各跑一遍，确认三个 Issue 都修好。

- [ ] **Step 5: 终态 commit（如有清理）**

如果 Step 1 有清理改动：

```bash
git add -A
git commit -m "$(cat <<'EOF'
chore(plot): clean up unused imports after canvas rewrite

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

无清理改动则跳过。

---

## Self-Review

**Spec 覆盖**：
- Issue 1（回车不闪）：Task 1 ✅
- Issue 2（plot 同步）：Task 2（三处写库命令） ✅
- Issue 3（折线图自绘）：Task 3（纯函数）+ Task 4（canvas 实现） ✅
- 全局回归：Task 5 ✅

**Placeholder 扫描**：无 TBD / TODO / vague 步骤；每步都有具体代码或具体命令。

**类型一致性**：
- `ChartLayout` 在 Task 4 Step 1 定义，同文件使用 — 一致
- `calc_x_tick_count(chart_w_px: f32) -> usize` 在 Task 3 定义，Task 4 调用 — 一致
- `format_time_label(t: f64, span: f64) -> String` 同上 — 一致
- `PendingAddChannelFocus::ChannelBlur` 在 Task 1 Step 1 定义，Step 2 写入，Step 3 读取 — 一致

**风险点（已在 Task 中标注）**：
- `gpui::quad` 签名可能在不同 gpui 版本不同 → Task 4 Step 2 提供了 grep 命令确认
- `gpui_component::plot::label::Text::new` 参数类型 → 同上
- `time_labels` 为空时的 fallback → Task 4 Step 1 已加 `format_time_label` 兜底
