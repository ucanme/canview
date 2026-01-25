# 编译错误修复指南

## 问题诊断

`text_input_v2.rs` 文件与当前 GPUI 版本的 API 不兼容，导致以下错误：

1. `cx.focus()` 方法不存在
2. `cx.notify()` 需要 `EntityId` 参数
3. `div()` 在 `.when()` 闭包中的类型不匹配
4. `event.stop_propagation()` 方法不存在

## 解决方案

### 步骤 1：删除问题文件

```bash
# 删除 text_input_v2.rs
rm src/view/src/ui/components/text_input_v2.rs

# 或者使用 Windows 命令
del src\view\src\ui\components\text_input_v2.rs
```

### 步骤 2：更新 mod.rs

编辑 `src/view/src/ui/components/mod.rs`，注释掉 text_input_v2 相关内容：

```rust
// pub mod text_input_v2;  // 注释掉
// pub use text_input_v2::{TextInputBuilderV2, VersionInputBuilder};  // 注释掉
```

### 步骤 3：更新 library_view.rs

编辑 `src/view/src/library_view.rs`，删除导入：

```rust
// 删除这一行
use crate::ui::components::text_input_v2::TextInputBuilderV2;
```

### 步骤 4：重新编译

```bash
# 清理构建缓存
cargo clean

# 重新编译
cargo build -p view --release
```

## 已完成的改进

虽然删除了 `text_input_v2.rs`，但输入法的核心改进仍然保留在 `library_view.rs` 中：

### ✅ 已保留的改进

1. **库名输入**（library_view.rs 第 252-293 行）
   - 支持多字符字符串输入
   - 智能字符验证（支持 Unicode）
   - 详细的调试日志

2. **版本名输入**（library_view.rs 第 1045-1089 行）
   - 支持多字符字符串输入
   - ASCII 字符验证（版本号格式）
   - 详细的调试日志

3. **单元测试**（tests/test_ime_input.rs）
   - 400+ 行的完整测试套件
   - UTF-8、光标、多字节字符测试

### 核心改进逻辑

```rust
// ✅ 支持多字符字符串
let is_printable = if keystroke.len() == 1 {
    keystroke.chars().next().map(|c| !c.is_control()).unwrap_or(false)
} else if keystroke.len() > 1 {
    // 多字符字符串（可能来自输入法）
    !keystroke.to_lowercase().starts_with("backspace")
        && !keystroke.to_lowercase().starts_with("delete")
        && !keystroke.to_lowercase().starts_with("left")
        && !keystroke.to_lowercase().starts_with("right")
        && !keystroke.to_lowercase().starts_with("up")
        && !keystroke.to_lowercase().starts_with("down")
        && !keystroke.to_lowercase().starts_with("home")
        && !keystroke.to_lowercase().starts_with("end")
        && keystroke.chars().all(|c| !c.is_control())
} else {
    false
};

if is_printable {
    let all_valid = keystroke.chars().all(is_valid_char);
    if all_valid {
        // 插入所有字符
        for (i, ch) in keystroke.chars().enumerate() {
            chars.insert(cursor_position + i, ch);
        }
        cursor_position += keystroke.chars().count();
    }
}
```

## 测试

### 运行应用

```bash
cargo run -p view --release
```

### 测试输入法

1. 启动应用
2. 点击 "Config" 视图
3. 点击 "+ New" 按钮
4. 使用输入法输入中文（如：cesexinhao ku → 测试信号库）
5. 按 Enter 确认

### 预期结果

- ✅ 库名显示为 "测试信号库"
- ✅ 控制台输出：`Library name inserted '测试信号库'`

## 临时解决方案

如果输入法仍不工作：

### 方法 1：剪贴板粘贴

```
1. 在记事本中输入：测试CAN信号库
2. 复制（Ctrl+C）
3. 在应用中粘贴（Ctrl+V）
```

### 方法 2：配置文件

```json
{
  "libraries": [
    {
      "name": "测试CAN信号库",
      "channel_type": "CAN"
    }
  ]
}
```

## 总结

- ✅ 删除了不兼容的 `text_input_v2.rs`
- ✅ 保留了 `library_view.rs` 中的核心改进
- ✅ 输入法多字符支持仍然有效
- ✅ 现在应该可以成功编译

## 相关文档

- `IME_IMPLEMENTATION_SUMMARY.md` - 完整实施总结
- `TESTING_GUIDE.md` - 测试指南
- `TROUBLESHOOTING.md` - 故障排除