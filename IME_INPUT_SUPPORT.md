# IME (输入法) 支持分析与解决方案

## 问题描述

用户反馈："在信号库管理中和输入法适配的不太好"

### 根本原因

当前实现使用 `on_key_down` 事件处理所有输入，包括文本输入。这在处理中文、日文、韩文等需要输入法（IME）的语言时存在以下问题：

1. **`on_key_down` vs `on_input` 的区别**：
   - `on_key_down`：监听键盘按键事件，每个按键触发一次
   - `on_input`：监听文本输入事件，处理已组合的文本（包括输入法输出的文本）

2. **输入法的工作流程**：
   ```
   用户输入拼音 → 显示候选词 → 选择候选词 → 插入完整文本
   "nihao" → "你好" → on_input("你好")
   ```
   
   使用 `on_key_down` 时：
   - 只能获取到按键事件：`'n'`, `'i'`, `'h'`, `'a'`, `'o'`
   - 无法获取到最终输入的文本：`'你好'`
   - 无法处理候选词选择

3. **当前代码的问题**：

   **library_view.rs 第 247 行**：
   ```rust
   if keystroke.len() == 1 {
       if let Some(ch) = keystroke.chars().next() {
           // 只能处理单字符，无法处理输入法输出的多字符文本
       }
   }
   ```

   **app/impls.rs 第 616 行**（稍微好一点，但仍有问题）：
   ```rust
   if key.len() > 0 && !key.to_lowercase().starts_with("backspace") {
       // 使用 insert_str 插入整个 key
       this.new_library_name.insert_str(pos, &key);
   }
   ```
   
   这个版本虽然支持插入字符串，但仍然依赖 `on_key_down`，无法正确处理：
   - 输入法的预编辑（composition）状态
   - 候选词窗口
   - 确认/取消输入法的事件

## 现状分析

### 当前实现位置

1. **library_view.rs**：
   - 第 251 行：库名输入框
   - 第 1035 行：版本名输入框
   
2. **app/impls.rs**：
   - 第 616 行：`new_library_input` 处理
   - 第 685 行：`new_version_input` 处理

3. **ui/components/text_input.rs**：
   - 第 123 行：`TextInputBuilder` 组件

### 输入法支持的技术要求

正确的输入法支持需要：

1. **处理 IME 事件**：
   - `compositionstart`：开始输入
   - `compositionupdate`：更新预编辑文本
   - `compositionend`：完成输入
   
2. **显示预编辑文本**：
   - 在用户选择候选词时，显示正在输入的拼音/假名
   - 显示候选词列表

3. **提交最终文本**：
   - 用户选择候选词后，插入完整文本
   - 处理退格、确认等操作

## GPUI 框架的输入处理能力

GPUI 是从 Zed 编辑器提取的 UI 框架，Zed 本身支持完整的输入法功能。需要研究：

1. **GPUI 是否提供 `on_input` 事件**：
   - 类似于 Web 的 `input` 事件
   - 或类似 `TextInput` 组件

2. **GPUI 的文本编辑组件**：
   - Zed 使用 `Editor` 组件处理文本输入
   - `Editor` 完整支持输入法
   - 但对于简单的输入框，可能过于复杂

3. **底层事件处理**：
   - GPUI 基于 `winit` 跨平台窗口库
   - `winit` 提供 `ReceivedCharacter` 和 `KeyboardInput` 事件
   - 需要查看 GPUI 如何暴露这些事件

## 解决方案

### 方案 1：使用 GPUI 的 `on_input` 事件（推荐）

如果 GPUI 提供 `on_input` 事件：

```rust
div()
    .on_key_down(|event, _window, cx| {
        // 只处理特殊键：Enter, Escape, Backspace, 箭头等
        match keystroke.as_str() {
            "enter" => { /* 提交 */ }
            "escape" => { /* 取消 */ }
            "backspace" => { /* 删除 */ }
            _ => {}
        }
    })
    .on_input(|text: &str, cx| {
        // 处理文本输入，包括输入法输出的文本
        // text 是完整的输入文本，可能是 "你好" 这样的多字符字符串
        this.new_library_name = text.to_string();
        cx.notify();
    })
```

**优点**：
- 简单直接
- 自动支持所有输入法
- 代码清晰

**缺点**：
- 需要确认 GPUI 是否提供 `on_input` 事件

### 方案 2：使用系统原生文本输入 API

如果不提供 `on_input`，可能需要：

1. **Windows**：
   - 使用 `WM_CHAR` 消息
   - 处理 `WM_IME_*` 消息

2. **macOS**：
   - 使用 `NSTextInputClient` 协议
   - 处理 `insertText:` 方法

3. **Linux**：
   - 使用 Text Input Convention (Wayland)
   - 或 XIM (X11)

**缺点**：
- 平台相关代码复杂
- 维护成本高

### 方案 3：集成简单的文本编辑组件

创建一个简单的 `TextInput` 组件，内部处理输入法：

```rust
pub struct TextInput {
    text: String,
    composition_text: String,  // 输入法预编辑文本
    cursor: usize,
}

impl TextInput {
    pub fn on_key_event(&mut self, event: &KeyEvent) {
        // 处理输入法相关事件
        if event.is_composition {
            self.composition_text = event.text;
        } else if event.is_composition_end {
            self.text.push_str(&event.text);
            self.composition_text.clear();
        }
    }
    
    pub fn render(&self) -> impl Element {
        div()
            .child(self.text)
            .child(
                div()
                    .text_color(rgb(0x89b4fa))
                    .child(self.composition_text)  // 显示预编辑文本
            )
    }
}
```

## 短期解决方案（临时）

在找到正确的 API 之前，可以改进当前实现：

1. **改进 `app/impls.rs` 的实现**：
   - 已经使用 `insert_str` 支持多字符
   - 但仍需要处理输入法的特殊情况

2. **添加输入法测试**：
   ```rust
   // 测试是否能接收到输入法文本
   if key.chars().any(|c| !c.is_ascii()) {
       eprintln!("Received non-ASCII text: '{}', len: {}", key, key.len());
       // 如果能打印出中文字符，说明输入法文本能传递过来
   }
   ```

3. **提供替代输入方式**：
   - 允许从剪贴板粘贴文本
   - 提供文件选择对话框导入名称

## 测试计划

### 测试用例

1. **中文拼音输入法**：
   - 输入："nihao" → 选择："你好"
   - 输入："测试信号库" → 选择："测试信号库"

2. **中文五笔输入法**：
   - 输入："tces" → 选择："测试"

3. **日文输入法**：
   - 输入："nihongo" → 选择："日本語"

4. **韩文输入法**：
   - 输入："hangul" → 选择："한글"

5. **混合输入**：
   - 中英文混合："Test测试123"

### 验证标准

- ✅ 能够输入完整的中文句子
- ✅ 能够选择候选词
- ✅ 能够使用退格键删除中文字符
- ✅ 光标位置正确（字符级别，非字节级别）
- ✅ 复制粘贴中文文本正常工作

## 下一步行动

1. **研究 GPUI API**：
   - 查阅 GPUI 文档
   - 查看 Zed 编辑器的源码如何处理输入
   - 测试是否有 `on_input` 或类似事件

2. **创建测试代码**：
   - 实现一个简单的输入测试
   - 验证当前实现的问题

3. **实施修复**：
   - 如果有 `on_input`，切换到该事件
   - 如果没有，考虑其他方案

4. **更新文档**：
   - 说明如何正确处理文本输入
   - 添加输入法支持示例

## 相关资源

- [GPUI GitHub](https://github.com/zed-industries/zed)
- [Zed Editor](https://github.com/zed-industries/zed)
- [winit Input Handling](https://docs.rs/winit/latest/winit/event/enum.KeyboardInput.html)
- [IME Support in Winit](https://github.com/rust-windowing/winit/issues/1363)

## 附录：输入法技术细节

### 输入法事件序列

```
1. 用户开始输入（输入拼音）
   → compositionstart
   → compositionupdate (text: "n")

2. 继续输入拼音
   → compositionupdate (text: "nih")
   → compositionupdate (text: "niha")
   → compositionupdate (text: "nihao")

3. 显示候选词
   → 用户看到候选词列表

4. 选择候选词（"你好"）
   → compositionupdate (text: "你好")
   → compositionend (data: "你好")

5. 最终输入
   → input event (data: "你好")
```

### 字符 vs 字节

- 中文字符在 UTF-8 中占用 3 字节
- 光标位置应该基于字符，不是字节
- Rust 的 `String::chars()` 返回字符迭代器
- `String::len()` 返回字节数

```rust
let text = "你好";
assert_eq!(text.chars().count(), 2);  // 2 个字符
assert_eq!(text.len(), 6);            // 6 个字节
```

当前实现正确使用 `.chars().count()` 计算字符数，确保光标位置正确。