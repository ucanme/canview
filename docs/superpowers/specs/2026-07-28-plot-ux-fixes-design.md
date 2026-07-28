# 2026-07-28 — Plot Sidebar UX Fixes (task.md 第 1/2/4/5 项)

## 背景与范围

`task.md` 收到用户 5 条反馈。本 spec 处理其中 4 项 plot / library 管理 UX 改进；第 3 项「信号集」是独立的新功能，单独成 spec。

| # | 反馈 | 本 spec 处理 |
|---|---|---|
| 1 | plot 侧边栏信号库太长，按通道/msg/signal 三级折叠 | ✅ |
| 2 | plot 回放时支持「全部取消」已选信号 | ✅ |
| 3 | 用户配置信号集 | ❌（独立 spec） |
| 4 | 选中信号若无数据，在图表上用图标说明 | ✅ |
| 5 | 信号库通道名输入框按回车后光标仍在闪 | ✅ |

## 用户决策摘要

- **折叠默认状态**：默认全部折叠；展开某通道时自动收起之前展开的通道（accordion 模式）；session 内记忆状态。
- **「全部取消」按钮**：放在侧边栏底部「Plot N signals」按钮左侧；点击同时清空 `selected_signals` 和当前图表 (`plot_data` / `plot_full_data`)。
- **无数据提示**：右侧图表区，为每个**已选但无数据**的信号渲染一张与正常图表外观一致的占位卡片，居中显示图标 + "No data for {signal} / 检查通道 ID 匹配或时间范围"。
- **回车键行为**：在添加通道输入行内，回车按字段顺序跳焦：`channel_id → channel_name → ✓ Confirm`；`✓ Confirm` 上回车提交表单（调用现有 `save_channel_config`）。不自动弹出文件选择框——用户自行点击 "Select File..." 选择文件。
- **拆模块**：把侧边栏渲染从 `chart_view.rs` 抽到新模块 `plot_sidebar.rs`。
- **拆纯函数**：把 `extract_signal_items(app) -> Vec<SidebarItem>` 拆为独立纯函数，让折叠/搜索过滤状态可单测。

## 架构

四处改动，分布在两个文件 + 一个新模块：

1. **新模块 `src/view/src/ui/views/plot_sidebar.rs`**（约 500 行，从 `chart_view.rs` 第 132-576 行迁移并扩展）
   - `SidebarItem` 枚举（迁移，`ChannelHeader` 增加 `is_expanded` 字段、`MessageHeader` 复用 `is_expanded` 字段实际生效）
   - `extract_signal_items(app: &CanViewApp) -> Vec<SidebarItem>` — 新纯函数
   - `render_signal_sidebar(window, app, view, cx)` — 侧边栏外壳 + 调用 `extract_signal_items` + 喂给 `uniform_list`
   - `render_sidebar_item(item, view)` — 单项渲染（迁移，ChannelHeader/MessageHeader 加 click 切换展开状态）
   - 模块在 `src/view/src/ui/views/mod.rs` 中导出

2. **`src/view/src/app/state.rs`**
   - `CanViewApp` 新增字段：
     - `expanded_channels: std::collections::HashSet<u16>`
     - `expanded_messages: std::collections::HashSet<(u16, u32)>`  // (ch_id, msg_id)
   - `new_with_maximized_state_and_bounds` 初始化为空 `HashSet`
   - `RuntimeState` 新增同样两个字段；`save_runtime_state` / `restore_runtime_state` 同步读写（窗口最大化/还原保留状态）
   - **不写入 `multi_channel_config.json`** —— 这是 UI 会话状态，不是用户配置

3. **`src/view/src/ui/views/chart_view.rs`**
   - 移除已迁移到 `plot_sidebar.rs` 的代码
   - `render_signal_sidebar` 的调用点改为 `plot_sidebar::render_signal_sidebar`
   - `render_chart_canvas` 修改：遍历 `app.selected_signals` 而不是 `series_data`；对每个 signal 在 `series_data` 中按 name 查找；找不到则调用新函数 `render_no_data_chart(signal_id) -> impl IntoElement`
   - 新增 `render_no_data_chart(signal_id: &str)` — 渲染高度 250px、深色背景、圆角边框、与 `render_single_chart` 一致的卡片，居中显示图标 + 文字
   - `render_single_chart` 的现有 `if series.points.is_empty()` 分支**保留**（防御性，与 `render_no_data_chart` 视觉一致即可，避免重复逻辑）

4. **`src/view/src/ui/views/library_management.rs`**
   - `render_add_channel_input_row_with_path`：
     - 移除 `on_key_down` 中 `enter` 分支（不再立即调用 `save_channel_config`）
     - 保留 `escape` 分支（取消输入，关闭表单）
     - 在创建 `channel_id_input` / `channel_name_input` 时（第 1326 行附近、第 1335 行附近），各加一个 `cx.subscribe` 监听 `InputEvent::PressEnter`：
       - `channel_id_input` 收到 PressEnter → `window.focus(&self.channel_name_input)`（聚焦到 name 输入框）
       - `channel_name_input` 收到 PressEnter → `window.focus(&self.add_ch_confirm_button_focus_handle)`（聚焦到 ✓ 按钮）
   - `✓ Confirm` 按钮（第 1561 行附近 `add-ch-confirm`）：增加 `FocusHandle`（在 `CanViewApp` 状态中新增 `add_ch_confirm_focus_handle: Option<gpui::FocusHandle>`，在打开添加通道输入时创建），增加 `on_key_down` 监听 `enter` → 调用 `save_channel_config`

5. **`src/view/src/app/impls.rs`**
   - 新增 `clear_selected_signals(&mut self, cx: &mut Context<Self>)` 方法：
     ```rust
     pub fn clear_selected_signals(&mut self, cx: &mut Context<Self>) {
         self.selected_signals.clear();
         self.plot_data = std::sync::Arc::from([]);
         self.plot_full_data = std::sync::Arc::from([]);
         self.plot_zoom_start = None;
         self.plot_zoom_end = None;
         cx.notify();
     }
     ```
   - 新增 `toggle_channel_expanded(ch_id: u16, cx)` / `toggle_message_expanded(ch_id: u16, msg_id: u32, cx)` 方法，供侧边栏 click 监听调用；前者实现 accordion 逻辑（展开 B 时清空 `expanded_channels` 中除 B 以外的项）

## 组件设计

### `SidebarItem` 字段调整

```rust
enum SidebarItem {
    ChannelHeader {
        name: String,
        ch_id: u16,
        is_can: bool,
        is_loaded: bool,
        mapping: Option<ChannelMapping>,
        is_expanded: bool,           // 新增
        selected_count: usize,       // 新增：该通道下已选信号数，显示在 header 右侧
    },
    MessageHeader {
        name: String,
        id: u32,
        is_can: bool,
        is_expanded: bool,           // 现有字段，开始真正使用
        ch_id: u16,                   // 新增：用于 toggle_message_expanded
    },
    SignalItem {
        name: String,
        id: String,
        size: u32,
        is_selected: bool,
        is_can: bool,
        ch_id: u16,                   // 新增：便于查找所属通道
        msg_id: u32,                  // 新增：便于查找所属 message
    },
}
```

### `extract_signal_items` 逻辑

```
fn extract_signal_items(app: &CanViewApp) -> Vec<SidebarItem>:
    items = []
    filter = app.signal_filter_text.to_lowercase()

    # 计算每个通道下已选信号数（用于 ChannelHeader 角标）
    selected_by_channel: HashMap<u16, usize> = count of selected_signals where signal_id starts with "CAN:ch_id:" or "LIN:ch_id:"

    # 渲染时的展开覆盖：搜索过滤非空时强制展开含匹配项的通道/message
    force_expand_due_to_search = !filter.is_empty()

    for ch_id in sorted(app.dbc_channels.keys()):
        dbc = ...
        # 计算 channel 的最终 is_expanded
        manual_expanded = app.expanded_channels.contains(ch_id)
        is_expanded = manual_expanded || force_expand_due_to_search

        # 收集该通道下的 messages
        channel_items = []
        channel_has_matches = filter.is_empty()  # 空过滤时通道始终显示
        for msg in sorted(dbc.messages.values() by id):
            matches_msg = msg.name contains filter || hex(msg.id) contains filter
            matching_signals = msg.signals where name contains filter
            if matches_msg || !matching_signals.is_empty():
                channel_has_matches = true
                msg_expanded = app.expanded_messages.contains((ch_id, msg.id)) || force_expand_due_to_search
                channel_items.push(MessageHeader { name, id: msg.id, is_can: true, is_expanded: msg_expanded, ch_id })
                if msg_expanded:
                    for sig in matching_signals (sorted by start_bit):
                        signal_id = "CAN:ch_id:msg.id:sig.name"
                        channel_items.push(SignalItem { name, id: signal_id, size, is_selected, is_can: true, ch_id, msg_id: msg.id })

        if channel_has_matches:
            items.push(ChannelHeader {
                name: "Channel {ch_id} (CAN)",
                ch_id, is_can: true, is_loaded: true, mapping: None,
                is_expanded,
                selected_count: selected_by_channel.get(ch_id).unwrap_or(0),
            })
            if is_expanded:
                items.extend(channel_items)

    # 同样处理 ldf_channels（LIN，结构对称）
    # 同样处理 app_config.mappings 中未加载的通道（仅 ChannelHeader，不展开）
    return items
```

关键点：
- `extract_signal_items` 是纯函数，输入只有 `&CanViewApp`，返回一个 `Vec`，**没有 cx / window / 任何副作用**。可单测。
- 渲染时 `render_signal_sidebar` 调用它一次，把结果喂给 `uniform_list`。
- `MessageHeader.is_expanded` 与 `ChannelHeader.is_expanded` 各自独立；通道折叠时 message 子项根本不出现在 items 里（性能优化：折叠的通道/message 不渲染子项）。

### 折叠 / 展开 click 处理

在 `render_sidebar_item` 中：
- `ChannelHeader`：整行可点击 → `view.update(cx, |this, cx| this.toggle_channel_expanded(ch_id, cx))`
- `MessageHeader`：整行可点击 → `view.update(cx, |this, cx| this.toggle_message_expanded(ch_id, msg_id, cx))`
- `SignalItem`：保留现有 checkbox 行为

### `toggle_channel_expanded` 实现

```rust
pub fn toggle_channel_expanded(&mut self, ch_id: u16, cx: &mut Context<Self>) {
    if self.expanded_channels.contains(&ch_id) {
        self.expanded_channels.remove(&ch_id);
    } else {
        // accordion: 先清空，再插入新的
        self.expanded_channels.clear();
        self.expanded_channels.insert(ch_id);
        // 也清空 expanded_messages 中属于其他通道的项
        self.expanded_messages.retain(|(c, _)| *c == ch_id);
    }
    cx.notify();
}

pub fn toggle_message_expanded(&mut self, ch_id: u16, msg_id: u32, cx: &mut Context<Self>) {
    let key = (ch_id, msg_id);
    if self.expanded_messages.contains(&key) {
        self.expanded_messages.remove(&key);
    } else {
        self.expanded_messages.insert(key);
    }
    cx.notify();
}
```

注意：accordion 只作用于通道层；message 之间可同时展开多个（用户决策未明确反对；折叠 message 让侧边栏变短也不那么关键）。

### 「Clear」按钮

在 `render_signal_sidebar` 底部操作栏（现有「绘制 N 个信号」按钮的左侧），仅在 `!app.selected_signals.is_empty()` 时显示：

```rust
.child(
    div()
        .px_3().py_1p5()
        .bg(rgb(0x3f3f46))
        .rounded(px(4.0))
        .cursor_pointer()
        .hover(|s| s.bg(rgb(0x52525b)))
        .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
            this.clear_selected_signals(cx);
        }))
        .child(div().text_xs().font_weight(FontWeight::BOLD)
            .text_color(rgb(0xffffff))
            .child(format!("清除全部 ({})", app.selected_signals.len())))
)
```

### `render_no_data_chart`

```rust
fn render_no_data_chart(signal_id: &str) -> impl IntoElement {
    div()
        .flex().flex_col()
        .h(px(250.0))
        .bg(rgb(0x18181b))
        .border_1()
        .border_color(rgb(0x27272a))
        .rounded_lg()
        .p_4()
        .items_center()
        .justify_center()
        .child(
            div().flex().flex_col().items_center().gap_2()
                .child(div().text_xl().text_color(rgb(0x71717a)).child("⊘"))  // 无数据图标
                .child(div().text_sm().text_color(rgb(0xa1a1aa))
                    .child(format!("No data for '{}'", signal_id)))
                .child(div().text_xs().text_color(rgb(0x52525b))
                    .child("检查通道 ID 匹配 (DBC vs 日志) 或时间范围"))
        )
}
```

`render_chart_canvas` 修改：

```rust
// 之前：
.children(series_data.iter().map(|series| render_single_chart(series, start_time, show_points)))

// 之后：
.children(app.selected_signals.iter().map(|signal_id| {
    let series = series_data.iter().find(|s| &s.name == signal_id);
    match series {
        Some(s) => render_single_chart(s, start_time, show_points).into_any_element(),
        None => render_no_data_chart(signal_id).into_any_element(),
    }
}))
```

注：`Series.name` 字段需确认等于 `signal_id`。检查 `extract_series_data` 第 1480 行附近，`Series` 的 `name` 设置逻辑——若现有 `Series.name` 不是完整 `signal_id` 而是 `signal_name`，则需要在 `extract_series_data` 中改为 `signal_id`（包含通道/msg 上下文），或在 `render_no_data_chart` 匹配时按其他键匹配。**实现时第一步先验证此假设**，根据实际 `Series.name` 格式调整匹配逻辑。

### Enter 键跳焦实现

`render_add_channel_button` 中创建输入框时增加 PressEnter 订阅（伪代码，实际行号在 1326 和 1335 附近）：

```rust
let id_input = cx.new(|cx| InputState::new(window, cx).placeholder("Channel ID"));
cx.subscribe(&id_input, |this, input, event, cx| {
    if let gpui_component::input::InputEvent::Change = event {
        this.new_channel_id = input.read(cx).text().to_string();
    } else if let gpui_component::input::InputEvent::PressEnter { .. } = event {
        // 跳焦到 name 输入框
        if let Some(name_input) = &this.channel_name_input {
            name_input.focus_handle(cx).focus(window);
        }
    }
}).detach();
this.channel_id_input = Some(id_input);

let name_input = cx.new(|cx| InputState::new(window, cx).placeholder("Channel name"));
cx.subscribe(&name_input, |this, input, event, cx| {
    if let gpui_component::input::InputEvent::Change = event {
        this.new_channel_name = input.read(cx).text().to_string();
    } else if let gpui_component::input::InputEvent::PressEnter { .. } = event {
        // 跳焦到 ✓ Confirm 按钮
        if let Some(handle) = &this.add_ch_confirm_focus_handle {
            handle.focus(window);
        }
    }
}).detach();
this.channel_name_input = Some(name_input);
```

注：`window` 在 `cx.subscribe` 闭包中不可直接访问——需要 `cx.listener` 模式或把 `window` 通过 `cx.listener(move |_this, _event, window, cx| ...)` 传入。具体 API 形态在实现时按 GPUI 当前签名调整。`add_ch_confirm_focus_handle` 是 `CanViewApp` 上的新字段：`Option<gpui::FocusHandle>`，在打开 add-channel 表单时 `cx.focus_handle()` 创建并赋值；✓ Confirm 按钮上 `.track_focus(&self.add_ch_confirm_focus_handle)` 绑定；✓ 按钮 `on_key_down` 监听 Enter → `save_channel_config`。

注：现有的 `render_add_channel_input_row_with_path` 整个 `div` 上有 `on_key_down` 监听 `enter` → `save_channel_config`。**这个监听要删除**，否则会在 input 自己处理 PressEnter 之前先触发提交。删除后，input 的 PressEnter 订阅独占处理；行级 `on_key_down` 只保留 `escape`。

## 数据流

| 修改 | 修改字段 | 触发者 |
|---|---|---|
| 折叠/展开通道 | `expanded_channels: HashSet<u16>` | 点击 ChannelHeader |
| 折叠/展开 message | `expanded_messages: HashSet<(u16,u32)>` | 点击 MessageHeader |
| 全部取消 | `selected_signals`, `plot_data`, `plot_full_data`, `plot_zoom_*` | 点击 Clear 按钮 |
| 选中信号 | `selected_signals` | 点击 SignalItem checkbox（现有） |

搜索过滤**不修改状态**——`extract_signal_items` 在 render 时计算 `is_expanded` 覆盖值（搜索非空 → 强制展开含匹配项的通道/message）。搜索清空 → 恢复手动展开状态。

## 持久化

- `expanded_channels` 和 `expanded_messages` 写入 `RuntimeState`（窗口最大化/还原保留），**不写入 `multi_channel_config.json`**
- 理由：这是 UI 会话状态（与 `selected_signals` 一致——后者也不持久化到磁盘）

## 错误处理

无需新增错误路径。现有校验保留：
- 任何字段为空时按 Enter 不跳焦（input 自身拒绝空提交，只切焦点不切字段空值）
- ✓ Confirm 上按 Enter 但未选路径 → 现有 `save_channel_config` 校验触发 `"Please select a database file"` 状态消息
- `clear_selected_signals` 无失败路径

## 测试计划

| 测试 | 类型 | 目的 |
|---|---|---|
| `extract_signal_items_empty_state` | 单元 | 启动时所有通道折叠，空过滤显示所有通道标题，无 message/signal 子项 |
| `extract_signal_items_search_expand` | 单元 | 非空过滤（如 "engine"）下，含匹配 signal 的通道 `is_expanded=true`，对应 message `is_expanded=true`，匹配 signal 在 items 中 |
| `extract_signal_items_search_clear_restores` | 单元 | 搜索 "engine" 后清空过滤，恢复 `expanded_channels`/`expanded_messages` 的手动状态 |
| `toggle_channel_expanded_accordion` | 单元 | 展开通道 A，再展开通道 B → A 折叠，B 展开；`expanded_channels` 只含 B |
| `toggle_channel_expanded_collapse_message` | 单元 | 通道 A 展开 + message M 展开；展开通道 B → `expanded_messages` 中属于 A 的项被移除 |
| `clear_selected_signals_resets_plot` | 单元 | 调用后 `selected_signals.is_empty()`、`plot_data.is_empty()`、`plot_zoom_start.is_none()` |
| `render_no_data_chart_for_selected_missing` | 集成（render smoke） | 选中 2 信号（1 无数据） → `render_chart_canvas` 输出含 1 chart + 1 no-data-card（用 render_to_state 检查 children 数量，若 GPUI 不可行则人工） |
| Enter 键跳焦链 | 手动 | id 输入框 Enter → 焦点到 name；name Enter → 焦点到 ✓；✓ Enter → 提交；任意阶段选路径后 ✓ Enter 仍能提交 |

GPUI render 不可单元测试（无 Display），所以 `extract_signal_items` 被特意拆为纯函数——折叠/搜索逻辑在不渲染的情况下可测。`render_no_data_chart` 与 `render_chart_canvas` 的集成验证靠人工跑 app。

## 兼容性 / 迁移

- 现有 `multi_channel_config.json` 不需要 schema migration —— 不新增持久化字段
- `RuntimeState` 是内存结构，schema 变化不影响磁盘
- 用户已有的 `selected_signals` 不变（仍然不持久化）

## 不在范围内

- 第 3 项「信号集」—— 独立 spec
- Library 管理（左中右三栏）的整体重构
- Plot 图表本身的性能优化
- 拖拽 BLF 文件、状态栏多文件段、Help 菜单等已有功能
