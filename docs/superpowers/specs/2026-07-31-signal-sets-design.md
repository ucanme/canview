# 2026-07-31 — 信号集 (Signal Sets) 设计

## 背景与范围

`task.md` 提出业务最需要的功能：

> 信号集，在单个信号库上可以添加一些通道+msg_id+signal 作为信号集合，后续在回放的时候，如果选中了信号库，用户可以选择该信号库下的信号集来批量回放某些信号集合。

本 spec 设计「信号集」的完整数据模型、创建流程、回放流程、UI 集成与测试策略。

**范围：**
- ✅ 数据模型 + 持久化（独立 `signal_sets.json` 文件，按 library_id 索引）
- ✅ 创建流程：从 plot 侧栏选择 → 保存为信号集
- ✅ 回放流程：plot 侧栏下拉框选择集 → 批量加载到 `selected_signals` → 绘图
- ✅ 库重命名 / 删除时同步迁移信号集归属
- ✅ 单元测试覆盖纯函数与往返序列化

**不在范围内（明确）：**
- Library tab 中的信号集管理 UI（重命名 / 删除 / 重排条目）
- 信号集导入 / 导出
- LAN 共享信号集（现有 LAN 共享只覆盖库本身）

## 用户决策摘要

| 决策点 | 选择 |
|---|---|
| 信号集归属 | **Per-library** — 集合挂在 `SignalLibrary` 下，跨版本复用；引用的 signal 在某个版本的 DBC 中不存在时静默跳过 |
| 创建方式 | **从 plot 侧栏当前选择保存** — 用户用现有侧栏勾选信号，点 `保存为信号集…` 命名保存 |
| 回放选择 UI | **Plot 侧栏顶部下拉框** — 在 `信号选择` header 与搜索框之间加一行 dropdown |
| 命名流程 | **侧栏内联输入** — 与现有 inline-input 模式一致，按 Enter 保存 / Esc 取消 |
| 持久化位置 | **独立文件 `signal_sets.json`** — 与 `can-viewer_config.json` 同目录，独立于 `AppConfig` 加载 / 保存路径 |

## 架构

新增一个独立模块 `src/view/src/library/signal_sets.rs` 负责数据 + 持久化；新增一个控制器 `src/view/src/controllers/signal_set_controller.rs` 负责业务操作；UI 改动全部集中在 `src/view/src/ui/views/plot_sidebar.rs`。复用现有 `gpui_component::input::Input` 与 `crate::ui::components::dropdown` 组件保持视觉一致。

```
┌─────────────────────────────────────────────────┐
│  plot_sidebar.rs                                │
│  ┌─ Header (existing) ──────────────────────┐  │
│  │ 信号选择 (Signals)        [active set] [N]│  │
│  ├─ Signal Sets dropdown (NEW) ─────────────┤  │
│  │ [▼ Select a set…                      ]   │  │
│  ├─ Search box (existing) ─────────────────┤  │
│  │ [🔍 search signals…                   ]   │  │
│  ├─ Virtualized list (existing) ────────────┤  │
│  │ Channel 1 (CAN)                          │  │
│  │  ▸ 0x100 EngineData                      │  │
│  │    ☐ EngineSpeed         16b             │  │
│  ├─ Bottom bar (existing + NEW) ────────────┤  │
│  │ [清除全部 (N)] [绘制 N 个信号] [保存为…] │  │
│  │ — or when naming —                       │  │
│  │ [输入集名…                     ] [取消]   │  │
│  └──────────────────────────────────────────┘  │
└─────────────────────────────────────────────────┘
```

**数据流：**

```
用户勾选信号 → selected_signals: Vec<String>
   ↓ 点 "保存为信号集…"
用户命名 → Enter
   ↓
signal_set_controller::save_current_selection_as_signal_set
   ├─ 解析 selected_signals → Vec<SignalSetEntry>
   ├─ 取 app.app_config.active_library_id
   ├─ push 到 signal_set_store.sets_by_library[lib_id]
   └─ save_signal_set_store(&app.signal_set_store)  // 写 signal_sets.json

用户切换库版本 → active_signal_set 被清空
   ↓
用户从 dropdown 选 "Engine signals"
   ↓
signal_set_controller::apply_signal_set
   ├─ 取 set.entries
   ├─ 根据 library.channel_type 决定 bus = "CAN" | "LIN"
   ├─ 清空 selected_signals
   ├─ 推入 "BUS:CH:0xMSG:SIG" 形式字符串
   ├─ active_signal_set = Some((lib_id, set_name))
   └─ extract_and_update_series_data(this)  // 触发绘图
```

## 组件设计

### 1. 数据模型 — `src/view/src/library/signal_sets.rs` (新文件)

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 信号集的单个条目。匹配 `selected_signals` 的
/// `BUS:CHANNEL:MSG_ID:SIGNAL_NAME` 格式（去除 bus 前缀，
/// bus 由父库的 channel_type 决定，避免冗余存储）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SignalSetEntry {
    pub channel_id: u16,
    pub msg_id: u32,
    pub signal_name: String,
}

/// 一个命名信号集
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SignalSet {
    pub name: String,
    pub entries: Vec<SignalSetEntry>,
}

/// 库 ID → 信号集列表的映射
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SignalSetStore {
    #[serde(default)]
    pub sets_by_library: HashMap<String, Vec<SignalSet>>,
}
```

**持久化函数：**

```rust
/// 与 can-viewer_config.json 同目录的 signal_sets.json 路径
/// 复用 libraries_base_path 的策略：优先用 app.config_file_path 的父目录，
/// 否则 fallback 到 executable 目录，最后 fallback 到 cwd
pub fn signal_set_store_path(config_file_path: Option<&Path>) -> PathBuf {
    if let Some(dir) = config_file_path.and_then(|p| p.parent()) {
        return dir.join("signal_sets.json");
    }
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            return exe_dir.join("signal_sets.json");
        }
    }
    PathBuf::from("signal_sets.json")
}

/// 启动时加载；文件缺失返回空 store
pub fn load_signal_set_store(config_file_path: Option<&Path>) -> SignalSetStore { ... }

/// 每次变更后保存
pub fn save_signal_set_store(
    store: &SignalSetStore,
    config_file_path: Option<&Path>,
) -> Result<(), String> { ... }
```

**说明：** 所有调用点（控制器中的 `save_signal_set_store` 与 `new_state()` 中的 `load_signal_set_store`）都传 `app.config_file_path.as_deref()`，与现有 `config_controller::save_config` 一致。下文示例代码中为简洁将 `save_signal_set_store(&app.signal_set_store)` 略写为不带第二参数，实际实现需传 `app.config_file_path.as_deref()`。

**纯函数：**

```rust
/// 解析 "CAN:1:0x100:EngineSpeed" → SignalSetEntry
/// 无效格式返回 None（少于 4 部分 / channel 解析失败 / msg_id 解析失败 / signal_name 空）
pub fn parse_signal_id(sig_id: &str) -> Option<SignalSetEntry> { ... }

/// 从集合 + channel_type 重建 selected_signals 形式的字符串列表
pub fn build_selected_signals_from_set(
    set: &SignalSet,
    channel_type: ChannelType,
) -> Vec<String> { ... }
```

### 2. App state — `src/view/src/app/state.rs`

新增字段：

```rust
/// 信号集存储，按 library_id 索引。启动时从 signal_sets.json 加载
pub signal_set_store: crate::library::signal_sets::SignalSetStore,

/// 当前激活的信号集 (library_id, set_name)；仅通过下拉框选择设置
pub active_signal_set: Option<(String, String)>,

/// 保存为信号集流程的内联输入
pub pending_signal_set_name: Option<String>,

/// 保存为信号集输入行可见性
pub show_save_set_input: bool,
```

`CanViewerApp::new_state()` 中初始化 `signal_set_store = load_signal_set_store()`，其他三个为 `None` / `false`。

### 3. 控制器 — `src/view/src/controllers/signal_set_controller.rs` (新文件)

```rust
/// 从 selected_signals 创建一个新的信号集，归属到当前激活的库
pub fn save_current_selection_as_signal_set(
    app: &mut CanViewerApp,
    name: &str,
    cx: &mut Context<CanViewerApp>,
)

/// 应用一个信号集到 selected_signals 并触发绘图
pub fn apply_signal_set(
    app: &mut CanViewerApp,
    library_id: &str,
    set_name: &str,
    cx: &mut Context<CanViewerApp>,
)

/// 清除当前激活的信号集；selected_signals 被清空
pub fn clear_active_signal_set(
    app: &mut CanViewerApp,
    cx: &mut Context<CanViewerApp>,
)

/// 删除一个信号集（用于将来从管理器入口删除；本 spec 仅实现对称 API，
/// Library tab 不暴露 UI）
pub fn delete_signal_set(
    app: &mut CanViewerApp,
    library_id: &str,
    set_name: &str,
    cx: &mut Context<CanViewerApp>,
)
```

每个变更函数末尾调用 `save_signal_set_store(&app.signal_set_store)` 持久化，然后 `cx.notify()`。

**`save_current_selection_as_signal_set` 详细逻辑：**

```rust
pub fn save_current_selection_as_signal_set(app, name, cx) {
    let name = name.trim().to_string();
    if name.is_empty() {
        app.status_msg = "Set name cannot be empty".into();
        cx.notify();
        return;
    }

    let library_id = match &app.app_config.active_library_id {
        Some(id) => id.clone(),
        None => {
            app.status_msg = "Activate a library first".into();
            cx.notify();
            return;
        }
    };

    // 解析 selected_signals → entries
    let mut entries = Vec::new();
    for sig_id in &app.selected_signals {
        if let Some(entry) = parse_signal_id(sig_id) {
            entries.push(entry);
        } else {
            eprintln!("⚠️ Skipping unparseable signal_id: {}", sig_id);
        }
    }
    if entries.is_empty() {
        app.status_msg = "No valid signals selected to save".into();
        cx.notify();
        return;
    }

    // 检查重名
    let sets = app.signal_set_store.sets_by_library
        .entry(library_id.clone()).or_default();
    if sets.iter().any(|s| s.name == name) {
        app.status_msg = format!("Set '{}' already exists", name).into();
        cx.notify();
        return;
    }

    let count = entries.len();
    sets.push(SignalSet { name: name.clone(), entries });
    let _ = save_signal_set_store(&app.signal_set_store);
    app.show_save_set_input = false;
    app.pending_signal_set_name = None;
    app.status_msg = format!("Saved set '{}' ({} signals)", name, count).into();
    cx.notify();
}
```

**`apply_signal_set` 详细逻辑：**

```rust
pub fn apply_signal_set(app, library_id, set_name, cx) {
    let library = match app.library_manager.find_library(library_id) {
        Some(lib) => lib,
        None => {
            app.status_msg = "Library not found".into();
            cx.notify();
            return;
        }
    };
    let sets = app.signal_set_store.sets_by_library.get(library_id);
    let set = match sets.and_then(|s| s.iter().find(|s| s.name == set_name)) {
        Some(s) => s.clone(),
        None => {
            app.status_msg = "Set not found".into();
            cx.notify();
            return;
        }
    };

    // 清空当前选择，从 set 重建
    app.selected_signals.clear();
    let rebuilt = build_selected_signals_from_set(&set, library.channel_type);
    app.selected_signals.extend(rebuilt);

    app.active_signal_set = Some((library_id.to_string(), set_name.to_string()));

    // 触发绘图
    crate::ui::views::chart_view::extract_and_update_series_data(app);

    app.status_msg = format!("Loaded set '{}' ({} signals)", set_name, set.entries.len()).into();
    cx.notify();
}
```

### 4. 与现有库变更钩子集成 — `src/view/src/controllers/library_controller.rs`

- **`apply_version_to_mappings`** — 设置新 mappings 后追加：
  ```rust
  app.active_signal_set = None;
  ```
  因为激活的库 / 版本变了，旧 set 不再适用。

- **`rename_library`** — 在 `LibraryManager::rename_library` 成功后，迁移 signal_set_store 的 key：
  ```rust
  if let Some(sets) = app.signal_set_store.sets_by_library.remove(&old_id) {
      app.signal_set_store.sets_by_library.insert(new_id.clone(), sets);
      let _ = save_signal_set_store(&app.signal_set_store);
  }
  // 如果 active_signal_set 指向 old_id，也要更新
  if let Some((lid, _)) = &app.active_signal_set {
      if lid == &old_id {
          app.active_signal_set = Some((new_id.clone(), app.active_signal_set.as_ref().unwrap().1.clone()));
      }
  }
  ```

- **`delete_library`** — 删除库前移除 store 中的条目：
  ```rust
  if app.signal_set_store.sets_by_library.remove(library_id).is_some() {
      let _ = save_signal_set_store(&app.signal_set_store);
  }
  if let Some((lid, _)) = &app.active_signal_set {
      if lid == library_id {
          app.active_signal_set = None;
      }
  }
  ```

### 5. UI — `src/view/src/ui/views/plot_sidebar.rs`

#### 5.1 新增 dropdown 行

在 `render_signal_sidebar` 的 header 与 search box 之间插入：

```rust
.child(render_signal_set_dropdown(app, view.clone(), cx))
```

```rust
fn render_signal_set_dropdown(
    app: &CanViewerApp,
    view: Entity<CanViewerApp>,
    _cx: &mut Context<CanViewerApp>,
) -> impl IntoElement {
    let items = build_set_dropdown_items(app);  // 纯函数，可单测
    // 用 crate::ui::components::dropdown 渲染
    // - 无 active library → 禁用 placeholder "Activate a library first"
    // - active lib 无 sets → 禁用 placeholder "No sets for this library"
    // - 有 sets → 列出，每项 "set_name (N)"，末尾若 active_signal_set.is_some()
    //   追加分隔符 + "✕ Clear set selection"
    // 点击 set 项 → signal_set_controller::apply_signal_set
    // 点击 ✕ → signal_set_controller::clear_active_signal_set
}
```

#### 5.2 修改底部 action bar

当 `selected_count > 0` 且 `!app.show_save_set_input`，在现有两按钮右侧加第三个：

```rust
.child(
    div()
        .px_3().py_1p5()
        .bg(rgb(0x3f3f46)).rounded(px(4.0))
        .cursor_pointer().hover(|s| s.bg(rgb(0x52525b)))
        .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
            this.show_save_set_input = true;
            this.pending_signal_set_name = Some(String::new());
            cx.notify();
        }))
        .child(div().text_xs().text_color(rgb(0xffffff))
            .child("保存为信号集…"))
)
```

当 `app.show_save_set_input` 为 true，整个底部 bar 替换为内联输入行：

```rust
div().p_2().bg(rgb(0x131314)).border_t_1().border_color(rgb(0x27272a))
    .flex().gap_2()
    .child(
        // 输入框，Enter → save_current_selection_as_signal_set
        // Esc / 取消按钮 → 重置 show_save_set_input = false
        // 复用现有 gpui_component::input::Input 模式
    )
    .child(
        div().px_3().py_1p5()
            .bg(rgb(0x3f3f46)).rounded(px(4.0))
            .cursor_pointer()
            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                this.show_save_set_input = false;
                this.pending_signal_set_name = None;
                cx.notify();
            }))
            .child(div().text_xs().text_color(rgb(0xffffff)).child("取消"))
    )
```

#### 5.3 Header 显示当前激活的 set

`render_signal_sidebar` 的 header 部分，在 item count 旁追加一个小徽章（仅 `active_signal_set.is_some()` 时显示）：

```rust
.when_some(&app.active_signal_set, |this, (lib_id, set_name)| {
    // 校验 lib_id 仍是当前 active_library_id；否则不显示（陈旧状态）
    if app.app_config.active_library_id.as_ref() == Some(lib_id) {
        this.child(
            div().px_1p5().py(px(1.0))
                .bg(rgb(0x3b82f6)).rounded(px(3.0))
                .text_xs().text_color(rgb(0xffffff))
                .child(set_name.clone())
        )
    } else {
        this
    }
})
```

#### 5.4 手动选择时清空 active_signal_set

`src/view/src/ui/views/plot_sidebar.rs` 中现有的 sidebar checkbox 点击处理（`SignalItem` 渲染处，当前在 plot_sidebar.rs:226-228 推 / 移 `selected_signals`）：

```rust
// 现有：toggle selected_signals 中的 sig_id
// 追加（在 toggle 之后）：
if this.active_signal_set.is_some() {
    this.active_signal_set = None;
}
```

这让 dropdown 在用户开始手动编辑后自动回到 `"Select a set…"`，避免「集绑定」与「手动选择」语义混乱。

**注意：两种「清空」语义不同：**

- **`clear_active_signal_set`**（用户从 dropdown 点 ✕）：清空 `active_signal_set` **且** 清空 `selected_signals`，回到「空选择」状态。
- **手动 checkbox 点击**（toggle 单个 signal）：清空 `active_signal_set` 但 **保留** `selected_signals`，用户接着手动编辑。

两者的区别是 dropdown ✕ 是「重置到空」，手动编辑是「在当前集选择上增删 → 转为手动模式」。

#### 5.5 纯函数 `build_set_dropdown_items`

```rust
#[derive(Clone, Debug, PartialEq)]
pub enum SetDropdownItem {
    Placeholder(String),     // 禁用占位
    Set { name: String, count: usize },
    ClearActive,             // ✕
}

pub fn build_set_dropdown_items(app: &CanViewerApp) -> Vec<SetDropdownItem> {
    let Some(lib_id) = &app.app_config.active_library_id else {
        return vec![SetDropdownItem::Placeholder("Activate a library first".into())];
    };
    let Some(sets) = app.signal_set_store.sets_by_library.get(lib_id) else {
        return vec![SetDropdownItem::Placeholder("No sets for this library".into())];
    };
    if sets.is_empty() {
        return vec![SetDropdownItem::Placeholder("No sets for this library".into())];
    }
    let mut items: Vec<SetDropdownItem> = sets.iter()
        .map(|s| SetDropdownItem::Set { name: s.name.clone(), count: s.entries.len() })
        .collect();
    if app.active_signal_set.is_some() {
        items.push(SetDropdownItem::ClearActive);
    }
    items
}
```

## 测试

### 单元测试（纯函数）

1. `signal_sets.rs::tests::test_parse_signal_id`
   - 有效 CAN：`"CAN:1:0x100:EngineSpeed"` → `Some(SignalSetEntry { 1, 256, "EngineSpeed" })`
   - 有效 LIN：`"LIN:2:0x20:Speed"` → `Some(SignalSetEntry { 2, 32, "Speed" })`
   - 0x 前缀 vs 十进制：`"CAN:1:256:Speed"` 与 `"CAN:1:0x100:Speed"` 都解析为 `msg_id = 256`
   - 无效 bus：`"J1939:1:0x100:Speed"` → `None`
   - 空 signal_name：`"CAN:1:0x100:"` → `None`
   - 部分缺失：`"CAN:1:0x100"` → `None`（少于 4 部分）
   - channel 解析失败：`"CAN:abc:0x100:Speed"` → `None`
   - msg_id 解析失败：`"CAN:1:0xGG:Speed"` → `None`

2. `signal_sets.rs::tests::test_build_selected_signals_from_set`
   - 空集 → 空 vec
   - 1 个条目 + CAN → `vec!["CAN:1:0x100:EngineSpeed".to_string()]`
   - 1 个条目 + LIN → `vec!["LIN:2:0x20:Speed".to_string()]`
   - 多条目保持顺序

3. `signal_sets.rs::tests::test_store_roundtrip`
   - 构造 store with 2 libraries / 3 sets / mixed entries
   - serialize → deserialize → assert_eq

4. `plot_sidebar.rs::tests::test_build_set_dropdown_items`
   - 无 active library → `[Placeholder("Activate a library first")]`
   - active lib 无 sets → `[Placeholder("No sets for this library")]`
   - active lib 有 2 sets → `[Set{name:"A",count:2}, Set{name:"B",count:3}]`
   - active lib 有 sets + active_signal_set Some → 末尾追加 `ClearActive`
   - active lib 有 sets + active_signal_set None → 不追加 ClearActive

### 手动验证清单（实施后）

- [ ] 加载一个 DBC，勾选 3 个信号，点 "保存为信号集…" 命名 "test"，重启 app，下拉框仍能看到 "test (3)"
- [ ] 选 "test"，selected_signals 被替换为 set 中的 3 个信号，chart 立即更新
- [ ] 选 "test" 后手动勾选第 4 个信号，dropdown 回到 "Select a set…"，但 selected_signals 保留 4 个
- [ ] 切换到另一个库版本，dropdown 重置为 "Select a set…"，selected_signals 被清空
- [ ] 重命名库，下拉框中的 set 仍可见（store key 已迁移）
- [ ] 删除库，下拉框显示 "No sets for this library"（如果新激活的库无 set）或 "Activate a library first"
- [ ] 空 selected_signals 时点 "保存为信号集…" 不允许（按钮不显示，因为 selected_count = 0）

## 错误处理

| 场景 | 行为 |
|---|---|
| 保存时无 active library | status `"Activate a library first"`，输入行不渲染 |
| 空 set 名 | status `"Set name cannot be empty"`，输入行保持可见让用户重试 |
| 重名 | status `"Set 'X' already exists"`，输入行保持可见 |
| selected_signals 全部解析失败 | status `"No valid signals selected to save"`，不创建空集 |
| 应用不存在的 set | 不可能（下拉框只列当前 lib 的 set），但控制器防御性 noop + status |
| 写盘失败 | `eprintln!` + status `"Warning: could not save signal sets to disk"`，不阻塞内存变更 |

## 边缘情况

- **首次启动**：`signal_sets.json` 不存在 → `load_signal_set_store()` 返回空 `SignalSetStore::default()`，第一次保存时创建文件。
- **配置迁移**：现有 `can-viewer_config.json` 没有 `signal_sets` 字段，因为信号集存在独立文件 → 不影响 `AppConfig`，无迁移成本。
- **库重命名**：`signal_set_store.sets_by_library` 的 key 从 `old_id` 迁到 `new_id`；若 `active_signal_set` 指向 `old_id`，同步更新。
- **库删除**：`sets_by_library[deleted_id]` 移除；若 `active_signal_set` 指向被删库，清空 `active_signal_set`。
- **应用 set 时清空 selected_signals**：`apply_signal_set` 先 `selected_signals.clear()`，再从 set 重建，避免与现有选择叠加。
- **应用 set 后用户手动编辑**：sidebar checkbox 点击 handler 中检查 `active_signal_set.is_some()` → 置 `None`。selection 保留，仅解除 set 绑定。
- **Plot 数据刷新**：`apply_signal_set` 末尾调 `extract_and_update_series_data(this)`，与现有 Plot 按钮路径一致。
- **集合中引用的 signal 不在当前版本 DBC 中**：仍推入 `selected_signals`，`extract_series_data` 在 `dbc.messages` 中找不到时静默跳过（与今天手动勾选未加载信号行为一致）。

## 实施顺序（粗略）

1. 数据模型 + 持久化（`signal_sets.rs` + 单元测试）
2. App state 字段 + 启动加载
3. 控制器四个函数 + 单元测试
4. 与 `library_controller` 的 rename / delete / apply 钩子
5. `plot_sidebar.rs` UI：dropdown 行 + 底部按钮 + 命名输入行 + header 徽章
6. `build_set_dropdown_items` 纯函数 + 单测
7. impls.rs 中 checkbox 点击 hook
8. 手动验证清单逐项跑通

## 不在范围内（明确）

- Library tab 中的信号集管理 UI（重命名 / 删除 / 重排条目）— 用户通过侧栏选择→保存创建，通过 dropdown 选择覆盖。完整的 entry-level 管理 UI 留待将来。
- 信号集导入 / 导出（JSON 文件可手动复制，无 GUI）。
- LAN 共享信号集（现有 LAN 共享只覆盖库本身；set 跟随库的归属，不单独共享）。
- 跨库的「全局信号集」（集合绑定到库，跨库时失效）。
