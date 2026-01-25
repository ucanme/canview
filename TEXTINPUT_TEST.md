# TextInput 组件使用说明

## 已实现的功能

### 1. **TextInputBuilder 组件**
位置: `src/view/src/ui/components/text_input.rs`

### 2. **使用方法**
```rust
TextInputBuilder::new()
    .text(initial_text)
    .placeholder("Enter text...")
    .focused(true)
    .build(
        "unique_id",  // 元素ID,用于焦点管理
        on_change,    // 文本变化回调
        on_submit,    // 回车提交回调
        on_cancel     // ESC取消回调
    )
```

### 3. **支持的键盘事件**
- **字符输入**: 所有可打印 ASCII 字符
- **空格**: 支持空格输入
- **Backspace**: 删除最后一个字符
- **Enter**: 触发 on_submit 回调
- **Escape**: 触发 on_cancel 回调

### 4. **焦点管理**
- `.focusable()`: 使元素可以获得焦点
- `.id()`: 设置唯一标识符
- `cx.focus()`: 请求焦点
- 点击输入框自动获得焦点

### 5. **视觉反馈**
- **未聚焦**: 灰色边框 (0x2a2a2a)
- **已聚焦**: 蓝色边框 (0x89b4fa)
- **空文本**: 显示 placeholder
- **有文本**: 显示实际文本内容

## 测试步骤

### 1. 启动应用
```bash
./target/release/view.exe
```

### 2. 导航到库管理界面
- 切换到 ConfigView

### 3. 测试TextInput
1. **点击 "+ New" 按钮**
   - 应该显示输入框
   - 显示 "Library name..." placeholder

2. **点击输入框**
   - 边框变为蓝色 (聚焦状态)
   - 可以看到光标 (如果系统支持)

3. **输入文本**
   - 使用键盘输入任意字符
   - 输入应该实时显示在输入框中
   - 支持空格输入

4. **测试删除**
   - 按 Backspace 删除字符
   - 最后一个字符被删除

5. **测试提交**
   - 按 Enter 创建库
   - 或点击 "Create" 按钮

6. **测试取消**
   - 按 Escape 取消
   - 或点击 "Cancel" 按钮

## 调试信息

程序会输出以下调试日志:
```
TextInput clicked, focusing: library_name_input
TextInput key_down: a, text: ''
TextInput key_down: b, text: 'a'
```

如果遇到问题,请检查日志输出。

## 已知问题

1. **光标显示**
   - 取决于窗口系统和GPUI的实现
   - 某些平台可能不显示文本光标

2. **焦点管理**
   - 确保输入框有唯一的ID
   - 使用 `.focusable()` 标记可聚焦元素

## 下一步优化

1. 添加光标闪烁动画
2. 支持文本选择
3. 支持复制粘贴
4. 添加输入验证
5. 支持多行输入

## 集成示例

```rust
// 在 library_view.rs 中
TextInputBuilder::new()
    .text(new_library_name.clone())
    .placeholder("Library name...")
    .focused(true)
    .build(
        "library_name_input",
        {
            let view = view.clone();
            move |new_text, cx| {
                view.update(cx, |this, cx| {
                    this.new_library_name = new_text.to_string();
                    cx.notify();
                });
            }
        },
        {
            let view = view.clone();
            move |_text, cx| {
                view.update(cx, |this, cx| {
                    this.create_new_library(cx);
                });
            }
        },
        {
            let view = view.clone();
            move |cx| {
                view.update(cx, |this, cx| {
                    this.new_library_name = String::new();
                    cx.notify();
                });
            }
        }
    )
```

## 编译和运行

```bash
# 编译
cargo build --release

# 运行
./target/release/view.exe
```

## 检查清单

- [x] TextInput 组件封装
- [x] 支持键盘输入
- [x] 支持焦点管理
- [x] 支持点击聚焦
- [x] 支持回车提交
- [x] 支持ESC取消
- [x] 支持backspace删除
- [x] 正确的事件传播控制
- [ ] 光标显示 (取决于平台)
- [ ] 文本选择功能
- [ ] 复制粘贴功能
