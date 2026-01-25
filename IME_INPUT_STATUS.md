# IME 中文输入支持 - 当前状态

## ✅ 已完成的工作

1. **实现了 EntityInputHandler trait** (`src/view/src/app/entity_input_handler.rs`)
   - ✅ 实现了所有必需的方法
   - ✅ `replace_text_in_range` - 接收 IME 提交的中文文本
   - ✅ `replace_and_mark_text_in_range` - 处理 IME 组合文本

2. **添加了输入状态管理**
   - ✅ `ImeTextInputState` - 存储 IME 输入状态
   - ✅ `is_editing_library_name` - 标记正在编辑

3. **更新了输入框渲染**
   - ✅ 点击输入框时设置 IME 状态
   - ✅ 同步 text 和 cursor_position

## ❌ 当前问题

**中文仍然无法输入的根本原因：**

虽然我们实现了 `EntityInputHandler`，但它**没有被正确注册到 GPUI 的输入系统中**。

### 问题分析

1. **`on_key_down` 无法捕获 IME 文本**
   ```rust
   div().on_key_down(|event, ...| {
       // 这个方法只能接收键盘按键事件
       // 永远无法接收到 IME 提交的中文文本！
   })
   ```

2. **缺少焦点系统集成**
   - GPUI 需要使用 `FocusHandle` 来管理焦点
   - 当前实现使用自定义的 `focused_library_input: Option<String>`
   - IME 输入处理器需要通过 `window.handle_input(focus_handle, handler)` 注册

3. **paint 阶段注册缺失**
   - `EntityInputHandler` 必须在 paint 阶段通过 `window.handle_input()` 注册
   - 当前 `render_library_view` 返回 `impl IntoElement`，无法在 paint 闭包中注册

## 🔧 需要的修复

### 选项 1: 使用 FocusHandle（推荐）

**优点**: 正确集成 GPUI 系统
**缺点**: 需要较大重构

```rust
pub struct CanViewApp {
    // 添加焦点句柄
    library_focus_handle: FocusHandle,
}

impl CanViewApp {
    fn new() -> Self {
        Self {
            library_focus_handle: FocusHandle::new(),
            ...
        }
    }

    fn render_library_view(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // 当输入框有焦点时注册输入处理器
        if self.is_editing_library_name {
            let handler = ElementInputHandler::new(bounds, cx.entity().clone());
            window.handle_input(&self.library_focus_handle, handler, cx);
        }
    }
}
```

### 选项 2: 使用 Window.set_input_handler()

直接在窗口级别设置输入处理器（可能影响整个应用）

### 选项 3: 等待 GPUI 更新

查看是否有更简单的方法来注册 EntityInputHandler

## 📝 当前实现的工作原理

```
用户输入中文的期望流程:
1. 点击输入框 → is_editing_library_name = true
2. 输入拼音 "ceshi"
3. GPUI 应该调用 → replace_and_mark_text_in_range("ceshi", ...)
4. 选择 "测试"
5. GPUI 应该调用 → replace_text_in_range("测试")  ← 这一步没有发生！
6. new_library_name 更新为 "测试"
```

**问题**: 第5步的 `replace_text_in_range` 从未被调用，因为输入处理器没有被注册。

## 🧪 如何验证当前状态

1. **运行应用**: `./target/release/view.exe`
2. **点击 Library 标签**
3. **点击 "+ New Library"**
4. **点击输入框**
5. **输入中文（如 "ceshi"）**
6. **观察**:
   - ❌ 输入法候选窗口显示正常
   - ❌ 选择候选后，输入框为空
   - ❌ 终端没有看到 "IME INPUT RECEIVED" 消息

**如果看到以下消息，说明 IME 工作正常**:
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
IME INPUT RECEIVED!
  Text: '测试'
  Range: None
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

## 📚 参考资源

- GPUI Window API: `~/.cargo/git/checkouts/zed-*/crates/gpui/src/window.rs`
  - `handle_input()` 方法（行 3595-3620）
- GPUI Input API: `~/.cargo/git/checkouts/zed-*/crates/gpui/src/input.rs`
  - `ElementInputHandler` 结构（行 82+）

## 🎯 下一步行动

1. **决定采用哪个方案**
   - 推荐选项 1（FocusHandle）
   - 需要添加 `gpui::FocusHandle` 到状态

2. **实现 FocusHandle 集成**
   - 修改状态结构添加 FocusHandle
   - 在输入框渲染时使用 focus_handle
   - 在 paint 阶段注册输入处理器

3. **测试验证**
   - 重新编译
   - 测试中文输入
   - 验证调试输出

## 🔗 修改的文件列表

- ✅ `src/view/src/app/state.rs` - 添加字段
- ✅ `src/view/src/app/entity_input_handler.rs` - 实现trait
- ✅ `src/view/src/app/impls.rs` - 添加 window 参数
- ✅ `src/view/src/ui/views/library_management.rs` - 更新点击处理
- ✅ `src/view/src/ui/components/ime_text_input.rs` - 简化为数据结构
