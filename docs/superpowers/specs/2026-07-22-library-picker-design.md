# Library Picker Overlay — Design Spec

- **日期**:2026-07-22
- **目标**:重新设计打开 BLF 后无激活库时的库选择体验,替代当前粗糙的警告卡片
- **范围**:重写 `library_picker.rs` 组件,新增 2 个 state 字段,修正触发与重置逻辑

---

## 1. 背景与现状

### 当前问题

现有 `library_picker.rs` 实现存在 4 个粗糙点:

1. **视觉层级乱**:⚠ 警告图标 + 标题 + 提示语 + 库列表 + 底部按钮堆叠,看起来像警告框而非"选择"入口
2. **库/版本列表交互粗糙**:树状结构 `└─ v1.0 [Activate]` 不易扫读,多版本时重复点击目标多;version_name → lib_id 反向查找逻辑有歧义
3. **底部 tip + 按钮语义乱**:`Tip: active library is also shown in the top bar` 是废话,`+ Create new library` 与 `Open Library →` 语义重叠
4. **位置/尺寸不准**:固定 `top(80px)` + `width 440px`,大屏时位置奇怪,且遮挡数据视图

### 目标

打开 BLF 后,如果没激活库,在数据视图(Log/Plot)中心显示**模态选择面板**:
- 只覆盖数据区,顶栏/底栏仍可点击
- 视觉像"选择"而非"警告"
- 一行一库 + 版本下拉 + Activate,清晰高效
- 可主动 dismiss,不被反复打扰

---

## 2. 触发条件

`render_library_picker_overlay` 返回 `Some` 当且仅当以下条件全部满足:

1. `app.current_file_name.is_some()`(已打开 BLF)
2. `app.current_view` ∈ {LogView, PlotView}(在数据视图)
3. `app.active_library_id.is_none() || app.active_version_name.is_none()`(无激活库)
4. `app.library_picker_dismissed == false`(用户未主动关闭)

任一条件不满足,返回 `None`,渲染层用 `.when_some()` 跳过。

---

## 3. 渲染结构

```
CanViewApp::render
  └── (after content area, before status bar)
      └── render_library_picker_overlay(app, view)   ← Option<impl IntoElement>
          ├── backdrop (全屏, rgba(0,0,0,0.3), 阻止点击穿透)
          └── card (480×自适应, max_h 400, 居中数据区)
              ├── header
              │   ├── title: "Select signal library"
              │   └── close ✕ (右上角)
              ├── description: "Pick a library version to decode signals from this BLF file."
              ├── library list
              │   ├── if empty: "No libraries yet. Click \"+ Create new library\" to add one."
              │   └── else: one LibraryRow per library
              │       ├── 📚 library name
              │       ├── version dropdown (default = latest_version)
              │       └── [ Activate ] button
              └── footer (border-top 1px)
                  ├── [ + Create new library ] (left, ghost button)
                  └── [ Open Library → ] (right, primary button)
```

遮罩只覆盖数据视图区域,顶栏(36px)和底栏(24px)在遮罩之上仍可点击 — 通过将 picker overlay 作为内容区的 `absolute` 子元素(不是兄弟节点),遮罩自动被内容区边界裁剪,不覆盖顶/底栏。

---

## 4. 视觉规范

### 卡片
- 宽 480px
- 高度自适应,最小 280px,最大 400px(库多时内部滚动)
- 居中(`top: 50%`, `left: 50%`, transform translate -50%)
- 背景 `colors::BG_ELEVATED`
- border 1px `colors::BORDER_DEFAULT`
- `radius::LG` (6px)
- `shadow_lg`
- padding `spacing::LG`,gap `spacing::MD`

### 标题
- "Select signal library"
- 13px `colors::TEXT_PRIMARY`
- `FontWeight::SEMIBOLD`
- **无 ⚠ 图标**

### 副标题
- "Pick a library version to decode signals from this BLF file."
- 11px `colors::TEXT_MUTED`

### 关闭 ✕
- 右上角,12px `TEXT_MUTED`
- hover → `TEXT_PRIMARY`
- 点击 → `library_picker_dismissed = true`

### 库行(LibraryRow)
- 行高 36px,行间 1px `colors::BORDER_SUBTLE` 分隔线
- 库名:`📚 {name}` 13px `colors::TEXT_PRIMARY`
- 版本下拉:width 120px,height 24px
  - 默认显示 `lib.latest_version().name`(或 `library_picker_selected_version[lib.id]` 若存在)
  - 点击展开列出版本列表,选中后更新 HashMap
- Activate 按钮:width 80px,height 24px,primary 样式(蓝底白字)
  - hover 加深(`PRIMARY_HOVER`)
  - 点击 → `activate_library_version(lib.id, selected_version_name)`

### 版本下拉
- 复用现有 `dropdown.rs::Dropdown` 组件,或用本地实现
- 显示格式:`v1.2`(版本名)
- 下拉面板:bg `BG_ELEVATED`,border `BORDER_DEFAULT`,`shadow_md`,`radius::MD`
- 选中项 hover `SURFACE0`

### 底部
- 顶 1px `colors::BORDER_SUBTLE` 分隔线
- padding-top `spacing::SM`
- 左:`+ Create new library` — Ghost 按钮(`SURFACE0` bg + `TEXT_SECONDARY`)
- 右:`Open Library →` — Primary 按钮(`PRIMARY` bg + `BG_DEFAULT` text)
- 中间用 `flex_1` 推开

### 无库场景
- 库列表区域显示:`No libraries yet. Click "+ Create new library" to add one.`
- 11px `colors::TEXT_MUTED`,居中
- 底部 `+ Create new library` 高亮(`PRIMARY` 样式)
- `Open Library →` 仍可用,但灰一些(用 `disabled` 样式)

### 遮罩
- 全屏 `rgba(0, 0, 0, 0.3)`
- 点击关闭
- 只覆盖数据区(顶/底栏不受影响)

---

## 5. 交互逻辑

### 显示
每帧渲染前评估触发条件(§2),全部满足且 dismissed=false → 渲染 picker overlay。

### 关闭路径
| 路径 | 动作 |
|---|---|
| ✕ 按钮 | `library_picker_dismissed = true` |
| ESC 键 | `library_picker_dismissed = true` |
| 点击遮罩 | `library_picker_dismissed = true` |
| 点 Activate 成功 | picker 因触发条件失效自动消失 |

### dismissed 重置
以下场景重置 `library_picker_dismissed = false`:

1. 加载新 BLF 文件(`apply_blf_result` Ok 路径)
2. 切换 view(从 Log→Plot、Plot→Log、Log/Plot→Library、Library→Log/Plot)— 任何 view 切换都重置,因为用户可能重新评估需求

实现位置:
- `apply_blf_result`(impls.rs Ok 路径)
- 现有 view 切换回调:Log/Plot toggle、Library 按钮、badge 点击

### Activate 流程
1. 用户在 LibraryRow 的版本下拉选版本(可选,默认最新)
2. 点 [Activate]
3. 读取 `library_picker_selected_version[lib.id]` 或 `lib.latest_version().name`
4. 调 `app.activate_library_version(lib_id, version_name, cx)`
5. `active_library_id` 被设 → 触发条件失效 → picker 自动消失
6. 状态栏显示 `✅ Applied version X to all plot channels`

### 错误处理
- `activate_library_version` 失败时不关闭 picker
- 状态栏显示错误消息(由 `apply_version_to_mappings` 实现)
- 用户可尝试其他库/版本

### 键盘行为
- ESC 关闭(picker 焦点)
- Tab/Shift+Tab 在 ✕、版本下拉、Activate、Create、Open 间循环(GPUI 默认)
- Enter 在版本下拉展开时选中当前项

---

## 6. State 变更

### 新增字段(在 `CanViewApp`)

```rust
pub library_picker_dismissed: bool,
pub library_picker_selected_version: std::collections::HashMap<String, String>,
```

初始化(`new_with_maximized_state_and_bounds`):
```rust
library_picker_dismissed: false,
library_picker_selected_version: std::collections::HashMap::new(),
```

### RuntimeState 不需要新增字段
`library_picker_dismissed` 和 `library_picker_selected_version` 是 UI 临时状态,不需要跨窗口保存/恢复。

### 写入点

- `library_picker_dismissed`:
  - ✕/ESC/遮罩点击 → `true`
  - `apply_blf_result` Ok → `false`
  - view 切换回调(Log/Plot toggle, Library button, badge) → `false`
- `library_picker_selected_version`:
  - 版本下拉选中项 → `insert(lib_id, version_name)`

---

## 7. 文件结构

| 文件 | 改动 |
|---|---|
| `src/view/src/app/state.rs` | 加 2 个字段 + 初始化 |
| `src/view/src/app/impls.rs` | `apply_blf_result` Ok 路径重置 dismissed |
| `src/view/src/app/impls_rendering.rs` | view 切换回调重置 dismissed;**将 picker overlay 从 render 根移到 content area 内部**,作为 content div 的 absolute 子元素 |
| `src/view/src/ui/components/status_bar.rs` | Log/Plot toggle 回调中重置 dismissed |
| `src/view/src/ui/components/top_bar.rs` | Library 按钮/badge 回调中重置 dismissed |
| `src/view/src/ui/components/library_picker.rs` | **重写**:`render_library_picker_overlay` + `render_library_row` + 内部版本下拉 |

`library_picker.rs` 重写后:
- 公开 `render_library_picker_overlay(app, view) -> Option<impl IntoElement>`
- 私有 `render_library_row`、`render_version_dropdown`、`render_backdrop`、`render_card`
- `#[cfg(test)] mod tests` 含触发条件单元测试(用 `CanViewApp::new_state`)

### 关键改动:接入点

现有接入(`impls_rendering.rs:1860-1866`)把 picker 放在 render 根级别 — 遮罩会覆盖整个窗口,顶栏/底栏都被遮。**改为放到 content area div 内部**(content area 已是 `overflow_hidden`,自带边界裁剪),遮罩自然只覆盖内容区。

```rust
// impls_rendering.rs 修改后
.child(
    // Content area - Zed style
    div()
        .flex_1()
        .bg(...)
        .overflow_hidden()
        .relative()  // for absolute children
        .child(match self.current_view { ... })
        // 移到这里:picker 作为 content 的 absolute 子元素
        .when_some(
            crate::ui::components::render_library_picker_overlay(self, view.clone()),
            |el, picker| el.child(picker),
        ),
)
.child(crate::ui::components::render_status_bar(self, view.clone()))
```

注意 content area 需要加 `.relative()` 才能让 picker 的 `.absolute()` 子元素定位正确。

---

## 8. 测试与验收

### 编译验证
- `cargo +nightly build -p view` 通过
- `cargo +nightly clippy -p view` warnings ≤ 328 (baseline)

### 手工验收(9 项)
1. 触发:打开新 BLF(无激活库)→ picker 出现,标题 "Select signal library",无 ⚠
2. 关闭 ✕:picker 消失;切到 Plot,picker 不再出现
3. 关闭 ESC:picker 消失
4. 关闭遮罩:点遮罩空白处,picker 消失
5. 无库场景:picker 显示 "No libraries yet...",底部 Create 高亮,Open 灰
6. 版本下拉:库有 2+ 版本时下拉可展开,可改选;Activate 按选中版本激活
7. Activate 流程:点 [Activate] → 状态栏 "✅ Applied version X",picker 消失,信号列填充
8. 重置:切到 Library view 再回数据视图 → dismissed 重置,无激活库时 picker 重新出现
9. 背景:picker 显示时,顶栏 File/Library 可点击(遮罩不覆盖顶/底栏)

### 验收标准
- `library_picker.rs` 内零 `rgb(0x` 硬编码
- 卡片宽 480px,居中,只覆盖数据区
- 无 ⚠ 图标
- 无 "Tip: active library is also shown in the top bar." 文案
- 库行清晰,版本下拉工作,Activate 立即生效
- dismissed 重置逻辑正确,不反复打扰用户
- 编译/clippy 通过

---

## 9. 不在本次范围

- 不修改 `activate_library_version` 内部逻辑
- 不修改 `dropdown.rs` 组件(只复用)
- 不动 `AppView` 枚举
- 不修改 Library view 内部渲染
- 不修改 BLF/DBC 解析

---

## 10. 风险与缓解

| 风险 | 缓解 |
|---|---|
| 切换 view 频繁重置 dismissed,picker 反复出现 | 仅在 view 切换的实际场景重置(用户在数据视图切换 Log/Plot 才重置,进 Library 视图后再回数据视图是合理重置) |
| 版本下拉状态管理复杂 | 用 HashMap 简单存储,每帧重新计算默认值(latest_version) |
| 遮罩点击误关闭 | 关闭后用户重新进入数据视图时 picker 会再出现(切 view 触发重置) |
| 库很多时卡片溢出 | max_h 400px + 内部 overflow_hidden(后续可加滚动) |
