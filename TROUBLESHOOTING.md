# 故障排除指南 - 输入法支持改进

## ❌ 编译问题

### 问题：无法编译

**症状**：
- `cargo build` 命令失败
- 显示编译错误

**解决方案**：

#### 1. 清理并重新编译

```bash
# 清理构建缓存
cargo clean

# 重新编译
cargo build -p view --release
```

#### 2. 检查文件完整性

确保以下文件存在：

```bash
# 检查新增的组件文件
src/view/src/ui/components/text_input_v2.rs

# 检查模块声明
src/view/src/ui/components/mod.rs  # 应该包含: pub mod text_input_v2;

# 检查二进制配置
src/view/Cargo.toml  # 应该包含: [[bin]] 配置
```

#### 3. 验证 mod.rs 配置

打开 `src/view/src/ui/components/mod.rs`，确认包含以下内容：

```rust
pub mod text_input_v2;

pub use text_input_v2::{TextInputBuilderV2, VersionInputBuilder};
```

#### 4. 验证 Cargo.toml 配置

打开 `src/view/Cargo.toml`，在文件末尾添加：

```toml
[[bin]]
name = "view"
path = "src/main.rs"
```

#### 5. 检查语法错误

如果仍有错误，运行：

```bash
# 获取详细错误信息
cargo build -p view 2>&1 | tee build_error.log

# 查看最后50行
tail -50 build_error.log
```

常见错误：

**错误 1：找不到 text_input_v2**
```
error[E0433]: failed to resolve: use of undeclared crate or module `text_input_v2`
```
**解决**：确保 `mod.rs` 中包含 `pub mod text_input_v2;`

**错误 2：找不到 TextInputBuilderV2**
```
error[E0433]: failed to resolve: could not find `TextInputBuilderV2` 
```
**解决**：确保 `mod.rs` 中包含 `pub use text_input_v2::{TextInputBuilderV2, VersionInputBuilder};`

#### 6. 使用工作区编译

```bash
# 编译整个工作区
cargo build --workspace --release

# 或者只编译 view 包
cargo build -p view --release
```

### 问题：编译成功但找不到可执行文件

**症状**：
- 编译成功（"Finished" 消息）
- 找不到 `view.exe` 或 `view` 可执行文件

**解决方案**：

#### 1. 检查可执行文件位置

```bash
# Windows
dir target\release\view.exe

# Linux/macOS
ls -l target/release/view
```

#### 2. 检查 Cargo.toml 配置

确保 `src/view/Cargo.toml` 包含：

```toml
[[bin]]
name = "view"
path = "src/main.rs"
```

#### 3. 使用 cargo run

如果找不到可执行文件，直接使用：

```bash
cargo run -p view --release
```

### 问题：运行时崩溃

**症状**：
- 应用启动后立即崩溃
- 显示 panic 错误

**解决方案**：

#### 1. 使用 debug 模式运行

```bash
# debug 模式有更多错误信息
cargo run -p view
```

#### 2. 检查 panic 消息

查看完整的 panic 堆栈跟踪，定位问题。

#### 3. 检查资源文件

确保所有需要的资源文件存在。

## 🧪 测试问题

### 问题：输入法仍然不工作

**症状**：
- 使用输入法输入中文时只显示拼音
- 无法选择候选词

**临时解决方案**：

#### 方法 1：使用剪贴板粘贴

```
1. 在记事本中输入：测试CAN信号库
2. 复制（Ctrl+C）
3. 在应用中粘贴（Ctrl+V）
4. 按 Enter 确认
```

#### 方法 2：编辑配置文件

打开 `multi_channel_config.json`，直接编辑：

```json
{
  "libraries": [
    {
      "id": "library_1",
      "name": "测试CAN信号库",
      "channel_type": "CAN",
      "versions": []
    }
  ]
}
```

保存后重新加载配置。

#### 方法 3：先用英文后改名

```
1. 创建库：TestLibrary
2. 关闭应用
3. 编辑配置文件，将 name 改为：测试CAN信号库
4. 重新启动应用
```

### 问题：字符显示乱码

**症状**：
- 中文字符显示为方框或问号
- 字符不正确

**解决方案**：

#### 1. 检查字体支持

确保系统字体支持中文。

#### 2. 检查文件编码

确保配置文件是 UTF-8 编码。

```bash
# Linux/macOS 检查编码
file multi_channel_config.json

# 应该显示: UTF-8 Unicode text
```

#### 3. 验证数据存储

应用内部使用 UTF-8，但如果显示问题，可能是字体问题。

### 问题：光标位置不正确

**症状**：
- 光标跳到错误位置
- 删除操作删除半个字符

**解决方案**：

这是字符级 vs 字节级的问题。代码已经修复使用字符级操作：

```rust
// ✅ 正确：字符级
let text_len = text.chars().count();

// ❌ 错误：字节级
let text_len = text.len();
```

如果仍有问题，请提供日志反馈。

## 📊 调试日志

### 查看实时日志

```bash
# Windows PowerShell
cargo run -p view --release 2>&1 | Select-String "TextInput|inserted"

# Linux/macOS
cargo run -p view --release 2>&1 | grep "TextInput\|inserted"
```

### 保存日志到文件

```bash
# 保存完整日志
cargo run -p view --release 2>&1 | tee debug.log

# 只保存输入相关日志
cargo run -p view --release 2>&1 | grep -E "TextInput|inserted" > input.log
```

### 日志分析

**正常工作的输入**：
```
TextInput key_down: keystroke='测试信号库' key='测试信号库'
Library name inserted '测试信号库', text: '测试信号库'
```

**输入法不工作的迹象**：
```
TextInput key_down: keystroke='nihao' key='nihao'
Library name inserted 'nihao', text='nihao'
# 注意：只显示拼音，没有显示中文
```

## 🔧 高级故障排除

### 重置构建环境

```bash
# 1. 完全清理
cargo clean

# 2. 删除 target 目录（可选，但彻底）
rm -rf target/
# Windows: rmdir /s /q target

# 3. 重新获取依赖
cargo fetch

# 4. 重新编译
cargo build -p view --release
```

### 更新依赖

```bash
# 更新 Cargo.lock
cargo update

# 重新编译
cargo build -p view --release
```

### 检查 Rust 版本

```bash
rustc --version
cargo --version
```

确保使用较新版本的 Rust（1.70+）。

### 检查 Git 状态

```bash
git status
git diff
```

确认所有更改已保存。

## 📞 获取帮助

如果以上方法都无法解决问题：

### 提交问题时请提供

1. **系统信息**
   - 操作系统：Windows 10 / Windows 11 / macOS / Linux
   - Rust 版本：`rustc --version`
   - Cargo 版本：`cargo --version`

2. **错误信息**
   - 完整的编译错误（如果有）
   - 完整的运行时错误（如果有）
   - 调试日志（`debug.log`）

3. **重现步骤**
   - 详细的操作步骤
   - 期望的结果
   - 实际的结果

4. **环境信息**
   - 输入法类型（微软拼音、搜狗拼音等）
   - 是否在虚拟机中运行
   - 是否有特殊配置

### 快速诊断脚本

创建 `diagnose.sh` (Linux/macOS) 或 `diagnose.bat` (Windows)：

**diagnose.bat**:
```batch
@echo off
echo === CanView Diagnosis ===
echo.
echo Rust Version:
rustc --version
cargo --version
echo.
echo Files:
dir src\view\src\ui\components\text_input_v2.rs
dir src\view\Cargo.toml
echo.
echo Build Attempt:
cargo build -p view --release 2>&1
echo.
echo Done! Check output above.
pause
```

运行诊断并保存输出。

## 📋 检查清单

在提交问题前，请确认：

- [ ] 已运行 `cargo clean` 并重新编译
- [ ] 已检查所有必需文件存在
- [ ] 已验证 `mod.rs` 配置正确
- [ ] 已验证 `Cargo.toml` 配置正确
- [ ] 已查看完整的编译/运行错误日志
- [ ] 已尝试 debug 模式运行
- [ ] 已确认 Rust 版本 >= 1.70
- [ ] 已提供详细的系统信息
- [ ] 已提供重现步骤

## ✅ 成功编译的标志

当一切正常时，您应该看到：

```
Compiling canview v0.1.0
Compiling view v0.1.0
Finished `release` profile [optimized] target(s) in X.XXs
```

并且可以运行：

```bash
cargo run -p view --release
```

应用启动后，可以在 Config 视图中测试输入法功能。

---

**最后更新**：2024  
**版本**：v0.2.0  
**维护者**：CanView 开发团队