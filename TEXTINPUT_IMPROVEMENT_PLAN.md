# TextInput 改进计划

## 基于 gpui-component 的最佳实践

### 当前状态分析

**现有 text_input.rs (canview):**
- ✅ 基础 IME 支持（多字符输入）
- ✅ 字符验证系统
- ✅ Builder 模式 API
- ❌ 缺少光标渲染
- ❌ 缺少选择区域
- ❌ 事件处理较简单
- ❌ 无滚动支持

**gpui-component input:**
- ✅ 完整的光标系统（位置、闪烁、渲染）
- ✅ 选择区域和高亮
- ✅ 丰富的键盘事件（actions 系统）
- ✅ 滚动和自动调整
- ✅ 使用 Rope 处理文本
- ✅ 多行支持
- ✅ LSP 集成
- ✅ 撤销/重做

### 改进优先级

#### 第一阶段：核心功能改进（必需）

1. **光标渲染和位置管理**
   - 添加可见光标
   - 光标闪烁动画
   - 正确的光标位置计算
   - 参考文件：`cursor.rs`, `blink_cursor.rs`, `element.rs:63-110`

2. **改进文本渲染**
   - 使用 `ShapedLine` 进行文本布局
   - 支持文本选择高亮
   - IME 组合窗口支持
   - 参考文件：`element.rs`, `text_wrapper.rs`

3. **增强事件处理**
   - 支持更多键盘操作（Ctrl/Cmd 键组合）
   - 鼠标选择支持
   - 焦点管理改进
   - 参考文件：`state.rs:102-250`, `movement.rs`

#### 第二阶段：用户体验提升（推荐）

4. **选择区域支持**
   - 文本选择（Shift + 方向键）
   - 选择高亮显示
   - 复制/剪切/粘贴
   - 参考文件：`selection.rs`, `state.rs:280-295`

5. **滚动和布局**
   - 自动滚动到光标
   - 内容溢出时显示滚动条
   - 动态高度调整
   - 参考文件：`element.rs:176-200`, `input.rs:166-232`

6. **撤销/重做**
   - 历史记录系统
   - Undo/Redo 操作
   - 参考文件：`state.rs`, `history.rs`

#### 第三阶段：高级功能（可选）

7. **多行支持**
   - 支持换行
   - 行号显示
   - 参考文件：`mode.rs`, `element.rs`

8. **搜索和替换**
   - 增量搜索
   - 高亮匹配项
   - 参考文件：`search.rs`

### 实现策略

#### 方案 A：渐进式改进（推荐）
保持现有 API，逐步添加新功能

**优点：**
- 不破坏现有代码
- 可以按需添加功能
- 学习曲线平缓

**步骤：**
1. 先改进现有 `TextInputBuilder`
2. 添加内部状态管理（可选使用 Entity）
3. 逐步添加光标渲染、选择等功能
4. 保持向后兼容

#### 方案 B：完全重构
基于 gpui-component 创建新的 TextInput 组件

**优点：**
- 获得所有高级功能
- 代码结构更清晰
- 未来扩展性更好

**缺点：**
- 需要修改所有现有用法
- 开发时间长
- 可能引入新 bug

### 建议的实现代码结构

```rust
// 新的模块结构
src/ui/components/text_input/
├── mod.rs              // 导出和 Builder
├── state.rs            // TextInputState 实体
├── cursor.rs           // 光标管理
├── selection.rs        // 选择区域
├── render.rs           // 渲染逻辑
└── events.rs           // 事件处理
```

### 核心改进点

#### 1. 光标渲染（来自 element.rs）

```rust
// 绘制光标的关键代码模式
fn render_cursor(&self, window: &mut Window, cx: &mut App) {
    let state = self.state.read(cx);
    let cursor_bounds = state.cursor_bounds();

    // 绘制闪烁光标
    div()
        .absolute()
        .left(cursor_bounds.origin.x)
        .top(cursor_bounds.origin.y)
        .w(px(CURSOR_WIDTH))
        .h(cursor_bounds.size.height)
        .bg(cx.theme().cursor)
}
```

#### 2. 文本选择（来自 element.rs）

```rust
// 文本选择高亮
fn render_selection(&self, window: &mut Window, cx: &mut App) {
    let state = self.state.read(cx);
    let selection = state.selected_range;

    for range in selection.visible_ranges() {
        div()
            .absolute()
            .left(range.start_x)
            .w(range.width)
            .bg(cx.theme().selection)
    }
}
```

#### 3. 事件处理（来自 state.rs）

```rust
// 改进的键盘事件处理
fn on_key_down(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut App) {
    match event.keystroke.as_str() {
        "backspace" => self.backspace(cx),
        "enter" => self.enter(cx),
        "left" => self.move_left(cx),
        "shift-left" => self.select_left(cx),
        "ctrl-c" => self.copy(cx),
        "ctrl-v" => self.paste(cx),
        _ => self.insert_text(&event.keystroke.to_string(), cx),
    }
}
```

### 使用 Rope 数据结构（可选，用于大文本）

```rust
use ropey::Rope;

pub struct TextInputState {
    pub text: Rope,  // 替代 String
    pub cursor: usize,
    pub selection: Range<usize>,
}

impl TextInputState {
    pub fn insert_text(&mut self, text: &str) {
        self.text.insert(self.cursor, text);
        self.cursor += text.len();
    }
}
```

### 建议的 API 改进

保持现有 Builder 模式，但添加更多选项：

```rust
TextInputBuilder::new()
    .text(state.text.clone())
    .placeholder("Enter text...")
    .validation(TextInputValidation::LibraryName)
    .multiline(false)              // 新增：多行支持
    .show_cursor(true)              // 新增：显示光标
    .enable_selection(true)         // 新增：选择支持
    .scrollable(true)               // 新增：滚动支持
    .on_change(|new_text, cx| { ... })
    .build(...)
```

### 测试计划

1. **基础功能测试**
   - 字符输入（包括中文）
   - Backspace/Delete
   - Enter 提交
   - 焦点管理

2. **光标测试**
   - 光标显示和闪烁
   - 光标移动（方向键）
   - IME 光标位置

3. **选择测试**
   - 文本选择
   - 复制/粘贴

4. **集成测试**
   - 在 library_view 中使用
   - 验证所有现有功能

### 时间估算

- 第一阶段（核心功能）：2-3 天
- 第二阶段（用户体验）：2-3 天
- 第三阶段（高级功能）：3-5 天

### 参考文件位置

**gpui-component 关键文件：**
- `../gpui-component/crates/ui/src/input/state.rs` - 状态管理
- `../gpui-component/crates/ui/src/input/element.rs` - 渲染逻辑
- `../gpui-component/crates/ui/src/input/cursor.rs` - 光标处理
- `../gpui-component/crates/ui/src/input/selection.rs` - 选择区域
- `../gpui-component/crates/ui/src/input/movement.rs` - 移动操作
- `../gpui-component/crates/ui/src/input/input.rs` - 主组件

**canview 当前文件：**
- `src/view/src/ui/components/text_input.rs` - 当前实现
- `src/view/src/ui/views/library_view.rs` - 使用示例
