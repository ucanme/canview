# UI Redesign — Top Bar, Tabs, Filter Bar, Status Bar

- **日期**:2026-07-21
- **方案**:A — 抽组件 + 样式整理(保持 Zed-dark 风格,聚焦标题栏与选项控件)
- **范围**:整体视觉统一(样式 + 布局 + 组件抽取)

---

## 1. 目标与非目标

### 目标

1. 把当前散落在 `impls_rendering.rs`(2496 行)与 `library_view.rs`(1192 行)里的顶部栏、tab、过滤器、状态栏代码抽取为 4 个可复用组件。
2. 所有颜色走 `theme::colors` token,新组件内零 `rgb(0x...)` 硬编码。
3. 保持现有 4 个 view(Log / Plot / Library / Config)的行为不变,但状态栏新增"当前 view 名"小标签显示。
4. FilterBar 仅用于 Log 与 Library 视图;Plot 视图保留其独立的 plot toolbar(信号选择/zoom),本次不动。
5. 清理 backup/deprecated/examples 文件,消除维护负担。

### 非目标

- 不引入新 view、新交互(命令面板、面包屑等)
- 不重写 `impls_rendering.rs` 全部渲染逻辑,只迁移顶部栏/过滤器/状态栏相关代码
- 不动 BLF/DBC/LDF 解析与 plot 绘制

### 必要的 state 调整

为支持 StatusBar 显示文件名,在 `CanViewApp` 新增 1 个字段:
```rust
pub current_file_name: Option<String>,   // 加载 BLF 时设置,关闭/新加载时重置
```
在 `LoadBlfFile` 命令处理中(`app/commands/load.rs::apply_blf_result`)写入,在 `CanViewApp::new_with_maximized_state_and_bounds` 初始化为 `None`。这是唯一必要的 state 变更,除此之外 `CanViewApp` 字段不增不减。

---

## 2. 架构

### 2.1 组件结构

新增 4 个组件,放在 `src/view/src/ui/components/`:

| 文件 | 组件 | 职责 |
|---|---|---|
| `top_bar.rs` | `TopBar` | 单行 36px,内嵌 `TabBar`,含 File 菜单 + 当前库徽章 + (Win/Linux) 窗口控制按钮 |
| `tab_bar.rs` | `TabBar` / `Tab` | 4 个 view tab,active 态用底部 2px indicator |
| `filter_bar.rs` | `FilterBar` / `FilterChip` | Log 与 Library 视图共用的过滤器/选项控件容器 |
| `status_bar.rs` | `StatusBar` | 单行 24px,文件名 / 计数 / 服务器状态 / 当前库 / view 名 |

### 2.2 数据流

```
CanViewApp::render (impls_rendering.rs)
    ├── TopBar::new(&self, view, cx)
    │       └── TabBar::new(current_view, on_view_change)
    ├── match current_view
    │       ├── LogView     → render_log_view (内含 FilterBar)
    │       ├── PlotView    → render_plot_view (保留现有 plot toolbar,本次不动)
    │       ├── LibraryView → render_library_view (内含 FilterBar)
    │       └── ConfigView  → render_config_view
    └── StatusBar::new(&self, cx)
```

所有组件**无状态**——纯函数式从 `CanViewApp` 读数据,通过 `view.update(cx, ...)` 写回。不新建 Entity。

### 2.3 渲染入口签名

```rust
impl CanViewApp {
    fn render(&mut self, view: Entity<Self>, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex_col()
            .child(TopBar::new(self, view.clone(), cx))
            .child(/* content area: match current_view */)
            .child(StatusBar::new(self))
            .child(/* overlay: share/import/file dialogs */)
    }
}
```

---

## 3. TopBar 组件

### 3.1 布局

```
macOS:  [traffic-light-space 80px] [File][Log][Signal Plot][Library] [...] [📚 lib / ver]        [ — ][ □ ][ ✕ ]
Win:    [File][Log][Signal Plot][Library] [...] [📚 lib / ver]                          [ — ][ □ ][ ✕ ]
Linux:  [File][Log][Signal Plot][Library] [...] [📚 lib / ver]                          [ — ][ □ ][ ✕ ]
```

- 高 36px,bg `theme::colors::BG_MUTED`,底部 1px `BORDER_SUBTLE`
- `window_control_area(WindowControlArea::Drag)` 让空白处可拖动窗口

### 3.2 内部区段

1. **File 按钮**:`Button::Ghost`,Small size,active 时 `show_file_menu = true` 时按钮高亮
2. **TabBar**:见 §4
3. **当前库徽章**:仅 `active_library_id.is_some()` 时显示
   - bg `colors::ACCENT_GREEN_BG` (`#1a2e1a`),border `ACCENT_GREEN_BORDER` (`#2d5a2d`)
   - 文字 `ACCENT_GREEN_LIGHT` (`#a6e3a1`),13px,左侧 ?? 图标
   - 拼在 tab 右侧,click → 切到 LibraryView
4. **窗口控制按钮**(仅 Win/Linux):
   - minimize:`—`(10×1px 灰线),hover bg `SURFACE1`
   - maximize:`□`(10×10px 灰边框),hover bg `SURFACE1`
   - close:`✕`(13px 灰字),hover bg `colors::CLOSE_HOVER` (`#c53030`),hover 时文字白色

### 3.3 props

回调统一使用 `impl Fn(...)` + `'static`,内部捕获 `view: Entity<CanViewApp>`。所有 4 个组件 props 风格保持一致:

```rust
pub fn render_top_bar(
    app: &CanViewApp,
    view: Entity<CanViewApp>,
    cx: &mut Context<CanViewApp>,
) -> impl IntoElement {
    // 内部构建 Button.on_mouse_down 闭包,捕获 view.clone()
    // 闭包签名统一为 `move |_, _, cx| { view.update(cx, |app, cx| { ... }); }`
}
```

`TopBar` 需要的字段(全部从 `app` 读取):
- `current_view: AppView`
- `active_lib_badge: Option<(String, String)>` — 由 `active_library_id` + `active_version_name` 推导
- `show_file_menu: bool`
- `is_macos: bool`(`cfg!(target_os = "macos")`)

回调(全部为 `view.update(cx, ...)` 闭包):
- `on_file_menu_toggle`
- `on_view_change(AppView)`
- `on_active_lib_click`
- `on_minimize(Window)`, `on_maximize(Window)`, `on_close(Window)` — 接 `Window` 引用

---

## 4. TabBar 组件

### 4.1 布局

- 高度与 TopBar 同(36px),内部 `div().flex().items_center().gap(px(2.))`
- 4 个 Tab:`File` 之后是 `Log` / `Signal Plot` / `Library`
- 每个 Tab 内部 `div().px(px(8.)).h_full().flex().items_center().cursor_pointer()`

### 4.2 状态样式

| 状态 | 文字色 | 背景 | 底部 indicator |
|---|---|---|---|
| active | `TEXT_PRIMARY` | 透明 | 2px `colors::PRIMARY` |
| inactive | `TEXT_MUTED` | 透明 | 无 |
| hover (inactive) | `TEXT_SECONDARY` | `SURFACE0` | 无 |

字号 13px (`typography::SM`)。点击触发 `on_view_change(view)`。

### 4.3 实现

```rust
pub fn render_tab_bar(
    current_view: AppView,
    on_view_change: impl Fn(AppView) + 'static,
    view: Entity<CanViewApp>,
) -> impl IntoElement {
    let tabs = [
        ("Log", AppView::LogView),
        ("Signal Plot", AppView::PlotView),
        ("Library", AppView::LibraryView),
    ];
    div().flex().items_center().gap(px(2.))
        .children(tabs.map(|(label, view_val)| {
            let active = current_view == view_val;
            let view_clone = view.clone();
            div()
                .px(px(8.))
                .h_full()
                .flex()
                .items_center()
                .cursor_pointer()
                .text_sm()
                .text_color(if active { colors::TEXT_PRIMARY } else { colors::TEXT_MUTED })
                .hover(|s| if active { s } else {
                    s.text_color(colors::TEXT_SECONDARY).bg(colors::SURFACE0)
                })
                .when(active, |el| el.border_b_2().border_color(colors::PRIMARY))
                .child(label.to_string())
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    cx.stop_propagation();
                    view_clone.update(cx, |app, cx| {
                        app.current_view = view_val;
                        if view_val == AppView::PlotView {
                            chart_view::extract_and_update_series_data(app);
                        }
                        cx.notify();
                    });
                })
        }))
}
```

### 4.4 props

```rust
pub fn render_tab_bar(
    app: &CanViewApp,
    view: Entity<CanViewApp>,
) -> impl IntoElement
// 内部读取 app.current_view,通过 view.update 触发 on_view_change
```

---

## 5. FilterBar 组件

### 5.1 容器

- 高 ~36px(由内容撑),bg `colors::BG_ELEVATED`,底部 1px `BORDER_SUBTLE`
- 内部 `div().flex().items_center().gap(px(8.)).px(px(12.)).py(px(4.))`

### 5.2 Log 视图 FilterBar

```
[ID ▾ 0x123]  [Channel ▾ All]  [🔍 Signal search...]        [Hex/Dec toggle]  [Display points toggle]
```

| 控件 | 实现 |
|---|---|
| ID Chip | `FilterChip::new("ID", current_value, on_click)` |
| Channel Chip | `FilterChip::new("Channel", current_value, on_click)` |
| Signal 搜索框 | 复用 `gpui_component::Input` + `InputState`,统一高度 28px、radius 4、bg `BG_MUTED`,左侧 ?? 图标 |
| Hex/Dec toggle | 2 个 Ghost 按钮拼成 toggle group,active 蓝色 |
| Display points toggle | 标签 + checkbox,放在 FilterBar 右侧 |

### 5.3 Library 视图 FilterBar

```
[Type ▾ ALL / CAN / LIN]  [🔍 Search libraries...]        [+ New Library]  [?? Share]  [📥 Import]
```

| 控件 | 实现 |
|---|---|
| Type Chip | `FilterChip::new("Type", current_type_label, on_click)` |
| 搜索框 | 同 Log variant |
| + New Library | `Button::Ghost` Small |
| ?? Share / 停止共享 | `Button::Ghost` Small,server running 时变红"Stop Share" |
| 📥 Import | `Button::Ghost` Small |

### 5.4 FilterChip 组件

```rust
pub fn render_filter_chip(
    label: &str,
    value: &str,
    active: bool,
    on_click: impl Fn() + 'static,
) -> impl IntoElement {
    div()
        .h(px(28.))
        .px(px(8.))
        .flex()
        .items_center()
        .gap(px(4.))
        .bg(colors::SURFACE0)
        .border_1()
        .border_color(if active { colors::BORDER_FOCUSED } else { colors::BORDER_DEFAULT })
        .rounded(radius::MD)
        .cursor_pointer()
        .hover(|s| s.bg(colors::SURFACE1))
        .text_sm()
        .child(span().text_color(colors::TEXT_MUTED).child(label.to_string()))
        .child(span().text_color(colors::TEXT_SECONDARY).child(value.to_string()))
        .child(span().text_color(colors::TEXT_MUTED).child("▾"))
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            cx.stop_propagation();
            on_click();
        })
}
```

### 5.5 props

```rust
pub fn render_filter_bar(
    app: &CanViewApp,
    view: Entity<CanViewApp>,
    variant: FilterBarVariant,
) -> impl IntoElement
// 内部根据 variant 选择要渲染的 chip / 输入框 / 按钮
// 所有按钮闭包通过 view.update(cx, ...) 回写到 app
```

`FilterBarVariant` 枚举:
```rust
pub enum FilterBarVariant { Log, Library }
```

读取的 `app` 字段:
- Log variant:`id_filter`, `id_filter_text`, `channel_filter`, `channel_filter_text`, `signal_filter_text`, `id_display_decimal`, `show_plot_points`
- Library variant:`library_filter_type`, `library_search_query`, `is_sharing`(`server_handle.is_some()`)

回调(全部为 `view.update(cx, ...)` 闭包):
- Log:`on_id_filter_click`, `on_channel_filter_click`, `on_signal_filter_change`, `on_id_display_toggle`, `on_plot_points_toggle`
- Library:`on_type_filter_click`, `on_library_search_change`, `on_new_library`, `on_share_toggle`, `on_import`

---

## 6. StatusBar 组件

### 6.1 布局

```
[📂 filename.blf] | [12,345 msgs] | [DBC: 3] | [LDF: 2]      [● Server ON http://192.168.1.5:8080] | [📚 lib / ver] | [log view]
```

- 高 24px,bg `colors::BG_MUTED`,顶部 1px `BORDER_SUBTLE`
- 内部 `div().flex().items_center().justify_between().px(px(12.))`
- 字号 11px(`typography::XS`),颜色 `TEXT_MUTED`,数字部分 `TEXT_SECONDARY`

### 6.2 左侧四段(竖线分隔)

| 段 | 内容 |
|---|---|
| 1 | ?? 图标 + 文件名(未加载时:`No file loaded — File > Open BLF...`,`TEXT_PLACEHOLDER`) |
| 2 | `12,345 msgs` 千分位 |
| 3 | `DBC: 3` |
| 4 | `LDF: 2` |

段间用 `div().w(px(1.)).h(px(12.)).bg(colors::BORDER_SUBTLE)` 分隔,左右 `gap(px(8.))`。

### 6.3 右侧三段(竖线分隔)

| 段 | 内容 |
|---|---|
| 1 | 服务器状态:运行时绿点 + URL,停止时灰点 + `Share disabled`。点击复制 URL 到剪贴板 + 2s toast |
| 2 | 当前库徽章(同顶部栏但更小,字号 11px) |
| 3 | 当前 view 名小写:`log view` / `plot view` / `library view` / `config view` |

### 6.4 props

```rust
pub fn render_status_bar(
    app: &CanViewApp,
    view: Entity<CanViewApp>,
) -> impl IntoElement
```

读取的 `app` 字段:
- `file_name` = `app.current_file_name`(新增字段,见 §1 必要的 state 调整)
- `message_count` = `app.messages.len()`
- `dbc_count` = `app.dbc_channels.len()`
- `ldf_count` = `app.ldf_channels.len()`
- `server_running` = `app.server_handle.is_some()`
- `server_url` = `app.share_url()`(已存在)
- `active_lib_badge` — 由 `active_library_id` + `active_version_name` 推导
- `current_view` = `app.current_view`

回调:`on_copy_url(String)` — 通过 `cx.write_to_clipboard` + 设置 `share_url_copied = true` + 2s 后重置(复用现有 share dialog 复制逻辑)。

---

## 7. 样式 Token 化

### 7.1 新增 token

在 `src/view/src/ui/theme/mod.rs` 的 `colors` 模块加。Rgba 字段为 `f32`(0.0-1.0),与现有 `palette` 写法一致:

```rust
// Accent — current library badge
pub const ACCENT_GREEN_LIGHT: Rgba = palette::GREEN;     // 0xa6e3a1
pub const ACCENT_GREEN_BG: Rgba = Rgba {
    r: 0x1a as f32 / 255.0, g: 0x2e as f32 / 255.0, b: 0x1a as f32 / 255.0, a: 1.0,
};
pub const ACCENT_GREEN_BORDER: Rgba = Rgba {
    r: 0x2d as f32 / 255.0, g: 0x5a as f32 / 255.0, b: 0x2d as f32 / 255.0, a: 1.0,
};

// Window control — close hover
pub const CLOSE_HOVER: Rgba = Rgba {
    r: 0xc5 as f32 / 255.0, g: 0x30 as f32 / 255.0, b: 0x30 as f32 / 255.0, a: 1.0,
};
```

### 7.2 Token 化规则

- 新组件 100% 使用 `theme::colors::*`,**禁止** `rgb(0x...)` 硬编码
- 间距统一使用 `theme::spacing::{XS, SM, MD, LG, XL}`,**禁止** 裸 `px(4.)` / `px(8.)` 等
- 圆角:chip/card/button 用 `radius::MD`(4px),对话框用 `radius::LG`(6px)
- 字号:tab/过滤器文字用 `typography::SM`(13px),徽章/小标签/状态栏用 `typography::XS`(11px)

### 7.3 现有硬编码的迁移

`impls_rendering.rs` 中顶部栏(行 1834-2063)、状态栏(行 2082+)、过滤器相关代码迁移到新组件后**整体删除**,自然消除硬编码。view 主体渲染(消息列表、plot、library 列表)暂不动,后续单独整改。

---

## 8. 文件清理

| 文件 | 行数 | 处理 |
|---|---|---|
| `src/view/src/ui/components/button_backup.rs` | - | 删除 |
| `src/view/src/ui/components/mod_old.rs` | - | 删除 |
| `src/view/src/app/impls.rs.after_deletion` | - | 删除 |
| `src/view/src/temp_impl1.txt` | - | 删除 |
| `src/view/src/main_backup.rs` | - | 删除 |
| `src/view/src/library_view_debug.rs` | 115 | 删除 |
| `src/view/src/library_view_focused.rs` | 375 | 删除 |
| `src/view/src/ui/components/dropdown_examples.rs` | - | 移到 `examples/` |
| `src/view/src/ui/components/button_examples.rs` | - | 移到 `examples/` |
| `src/view/src/ui/components/modal_examples.rs` | - | 移到 `examples/` |
| `src/view/src/ui/components/tabs_examples.rs` | - | 移到 `examples/` |
| `src/view/src/views/common_examples.rs` | 635 | 移到 `examples/` |

`mod.rs` 中的 `pub mod xxx_examples;` 与 `mod_old` 等声明同步删除。

---

## 9. 实施顺序

每个 commit 后必须 `cargo build` 通过、`cargo clippy --workspace -- -D warnings` 无新警告。

| # | commit 标题 | 内容 |
|---|---|---|
| 1 | `feat(ui): add TopBar/TabBar/FilterBar/StatusBar skeletons` | 新建 4 个文件,导出空函数,注册到 `mod.rs` |
| 2 | `feat(ui/theme): add accent and close-hover color tokens` | `theme/mod.rs` 加 4 个新 token |
| 3 | `refactor(ui): extract TopBar+TabBar from impls_rendering` | 实现 TopBar/TabBar,删除 `impls_rendering.rs` 顶部栏代码(行 1834-2063) |
| 4 | `refactor(ui): extract FilterBar from log/library views` | 实现 FilterBar,替换 Log/Library 视图中的过滤器 |
| 5 | `refactor(ui): extract StatusBar from impls_rendering` | 实现 StatusBar,删除 `impls_rendering.rs` 底部状态栏代码 |
| 6 | `chore(ui): remove backup/examples/debug files` | 删除 §8 中 12 个文件,同步 `mod.rs` 声明 |

---

## 10. 测试

### 10.1 自动化

- `cargo build --release --bin view` — 编译通过
- `cargo clippy --workspace -- -D warnings` — 0 新警告(已有 baseline 警告 381 个,本次只承诺不增加)
- `cargo fmt --all`
- `cargo test --workspace` — 现有测试不回归
- 新增:在 `src/view/src/ui/components/top_bar.rs` 等文件底部加 `#[cfg(test)] mod tests`,仅测试纯函数(token 颜色常量值、`format_msg_count` 千分位函数等),不测渲染

### 10.2 手工走查

`cargo run --release --bin view`,按顺序验证 7 项:

1. 启动应用,顶部栏 4 个 tab(Log/Signal Plot/Library + File)可点击切换,active 时底部 2px 蓝色 indicator 显示
2. macOS 下交通灯位置正常(appers_transparent + 80px 占位);Win/Linux 下窗口控制按钮 minimize/maximize/close 工作正常,close hover 红色
3. 切到 Log view:FilterBar 显示 ID/Channel chip,点击展开下拉,搜索框可输入,Hex/Dec 和 Display points toggle 可切换
4. 切到 Library view:FilterBar 显示 Type chip + 搜索框 + + Share Import 按钮,全部可用
5. 加载 sample.dbc + test_can.blf:状态栏左侧文件名 + 计数显示正确,千分位正确
6. 启动 LAN share:状态栏右侧显示绿色 ● + URL,点击 URL 复制到剪贴板并出现 2s toast
7. 全程无新增 panic/warning 日志,`cargo run` stderr 不出现新的 eprintln

### 10.3 回退策略

每个 commit 独立可 `git revert`,保留前序成果。若 commit 4(FilterBar)出现行为差异,可单独 revert,保留 TopBar/TabBar(commit 3)改动。

---

## 11. 风险与缓解

| 风险 | 缓解 |
|---|---|
| 抽取 TopBar 时遗漏现有 on_key_down 处理(过滤框输入回车等) | 保留 `on_key_down` 在 `CanViewApp::render` 根级,TopBar 只负责布局,不吞键盘事件 |
| FilterBar 替换导致 ID/Channel 过滤下拉时序差异 | 复用现有 dropdown 组件,只换容器和 chip 样式,内部 dropdown 实现不动 |
| StatusBar 点击 URL 复制与现有 share dialog 复制重复 | 复用同一 `cx.write_to_clipboard` 调用,toast 提示用同一 `share_url_copied` 状态 |
| 381 个 baseline clippy 警告 | 本次只承诺不增加,不修复历史警告;若新组件触发的 clippy 警告,允许在 commit 1-5 中加 `#[allow]` 但需说明原因 |

---

## 12. 验收标准

- [ ] 4 个新组件文件存在,`mod.rs` 中正确导出
- [ ] `impls_rendering.rs` 行数下降 ≥ 400(从 2496 → ~2000 或更少)
- [ ] 新组件内 `grep -r "rgb(0x" src/view/src/ui/components/{top_bar,tab_bar,filter_bar,status_bar}.rs` 无匹配
- [ ] §8 列出的 12 个文件全部删除或移走
- [ ] `cargo build --release --bin view` 通过
- [ ] `cargo clippy --workspace` 警告数 ≤ 381(baseline)
- [ ] §10.2 手工走查 7 项全部通过
