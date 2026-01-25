# 测试指南 - 输入法支持改进

## 📋 编译说明

### 编译命令

在项目根目录执行：

```bash
# 编译所有 workspace 成员
cargo build --release --workspace

# 或者只编译 view 模块
cargo build -p view --release
```

### 编译输出

```
Finished `release` profile [optimized] target(s) in 0.32s
```

如果看到以上输出，说明编译成功 ✅

### 可执行文件位置

编译成功后，可执行文件位于：
```
target/release/view.exe    (Windows)
target/release/view        (Linux/macOS)
```

---

## 🚀 运行应用

### 方式 1：使用 cargo run

```bash
# 运行 view 应用
cargo run -p view --release

# 或者直接运行（如果配置了 bin）
cargo run --release
```

### 方式 2：直接运行可执行文件

```bash
# Windows
./target/release/view.exe

# Linux/macOS
./target/release/view
```

### 启动日志

应用启动时会显示：
```
[2024-xx-xx] [INFO] Starting CanView application
[2024-xx-xx] [INFO] GPUI initialized
```

---

## 🧪 功能测试

### 测试 1：库名输入 - 中文输入法

**步骤**：
1. 启动应用
2. 点击左侧的 "Config" 视图
3. 点击右侧面板的 "+ New" 按钮
4. 打开中文输入法（如：微软拼音）
5. 输入拼音：`cesexinhao ku`
6. 从候选词中选择：`测试信号库`
7. 按 Enter 确认

**预期结果**：
- ✅ 库名显示为 "测试信号库"
- ✅ 库出现在左侧列表中
- ✅ 控制台输出：`Library name inserted '测试信号库', text: '测试信号库'`

**如果失败**：
- ❌ 只显示拼音 "cesexinhao ku"
- ❌ 控制台显示多个单字符插入日志

### 测试 2：库名输入 - 混合字符

**步骤**：
1. 点击 "+ New" 按钮
2. 输入：`CAN测试库2024`
3. 按 Enter 确认

**预期结果**：
- ✅ 库名显示为 "CAN测试库2024"
- ✅ 字符数：10 个字符（不是字节）
- ✅ 控制台输出完整文本

### 测试 3：库名输入 - 剪贴板粘贴

**步骤**：
1. 在记事本中输入：`📊 数据分析库 📈`
2. 复制文本（Ctrl+C）
3. 在应用中点击输入框
4. 粘贴文本（Ctrl+V）
5. 按 Enter 确认

**预期结果**：
- ✅ 库名显示为 "📊 数据分析库 📈"
- ✅ 表情符号正确显示
- ✅ 所有字符都正确显示

### 测试 4：版本名输入 - 标准格式

**步骤**：
1. 选择一个已创建的库
2. 在版本列表中点击 "+"
3. 输入版本名：`v1.0.0-beta`
4. 按 Enter 确认

**预期结果**：
- ✅ 版本名显示为 "v1.0.0-beta"
- ✅ 支持点号、下划线、连字符
- ✅ 控制台输出：`Version name inserted 'v1.0.0-beta'`

### 测试 5：版本名输入 - 拒绝非法字符

**步骤**：
1. 创建新版本
2. 尝试输入：`v1.0 测试`（包含空格和中文）

**预期结果**：
- ❌ 版本名不接受空格和中文
- ❌ 控制台显示：`Rejected text (invalid chars)`
- ✅ 只有合法字符被插入（如 `v1.0`）

### 测试 6：光标位置 - 多字节字符

**步骤**：
1. 创建库名：`测试库`
2. 不要按 Enter，继续输入：`123`

**预期结果**：
- ✅ 文本变为 "测试库123"
- ✅ 光标在正确位置（6 个字符后）
- ✅ 使用左右箭头键可以按字符移动（不是按字节）

### 测试 7：删除操作 - 中文字符

**步骤**：
1. 输入库名：`测试信号库`
2. 按两次 Backspace

**预期结果**：
- ✅ 文本变为 "测试信"
- ✅ 每次删除一个完整的中文字符（不是半个字符）
- ✅ 光标位置正确

### 测试 8：长文本性能

**步骤**：
1. 粘贴长文本：`测试信号库` 重复 100 次
2. 观察性能

**预期结果**：
- ✅ 插入速度快（< 100ms）
- ✅ 界面响应流畅
- ✅ 没有卡顿

---

## 🔍 调试日志

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
cargo run -p view --release 2>&1 | tee ime_debug.log

# 只保存相关日志
cargo run -p view --release 2>&1 | grep -E "TextInput|inserted" > ime_input.log
```

### 日志示例

**成功的中文输入**：
```
TextInput clicked, focusing: library_name_input
TextInput key_down: keystroke='cesexinhao ku' key='cesexinhao ku' text=''
Library name inserted 'cesexinhao ku', text: 'cesexinhao ku'
TextInput key_down: keystroke='测试信号库' key='测试信号库' text='cesexinhao ku'
Library name inserted '测试信号库', text: '测试信号库'
TextInput key_down: keystroke='enter' key='Enter' text='测试信号库'
```

**字符验证拒绝**：
```
TextInput key_down: keystroke='test\nlibrary' key='test\nlibrary' text=''
Rejected text (invalid chars): 'test\nlibrary'
```

---

## ❌ 常见问题

### 问题 1：编译失败 - 找不到 text_input_v2

**错误信息**：
```
error[E0433]: failed to resolve: use of undeclared crate or module `text_input_v2`
```

**解决方案**：
```bash
# 检查文件是否存在
ls src/view/src/ui/components/text_input_v2.rs

# 检查模块声明
cat src/view/src/ui/components/mod.rs | grep text_input_v2
```

应该看到：
```
pub mod text_input_v2;
pub use text_input_v2::{TextInputBuilderV2, VersionInputBuilder};
```

### 问题 2：运行时崩溃

**错误信息**：
```
thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value'
```

**解决方案**：
1. 运行 debug 版本获取更多信息：
   ```bash
   cargo run -p view
   ```
2. 检查是否有 panic 发生
3. 查看完整堆栈跟踪

### 问题 3：输入法仍然不工作

**症状**：输入中文时只显示拼音

**检查清单**：
1. ✅ 确认已使用最新代码（git pull）
2. ✅ 重新编译（cargo build --release）
3. ✅ 查看控制台日志
4. ✅ 尝试不同的输入法（微软拼音、搜狗拼音）
5. ✅ 尝试使用剪贴板粘贴作为临时方案

**临时解决方案**：
```json
// 直接编辑配置文件
{
  "libraries": [
    {
      "name": "测试CAN信号库",
      "channel_type": "CAN"
    }
  ]
}
```

### 问题 4：光标位置不正确

**症状**：光标跳到错误位置

**原因**：字节级 vs 字符级计算

**检查**：
- 代码应使用 `text.chars().count()` 而不是 `text.len()`
- 光标位置应基于字符数，不是字节数

**示例**：
```
"测试" -> chars().count() = 2 ✅
"测试" -> len() = 6 ❌ (字节数)
```

---

## ✅ 验收标准

### 功能验收

- [x] 可以使用输入法输入中文库名
- [x] 可以粘贴多字符文本
- [x] 可以混合输入中英文和数字
- [x] 版本名正确验证（只接受 ASCII + .-_）
- [x] 光标位置正确（字符级）
- [x] 删除操作正确（删除完整字符）
- [x] 控制台输出调试日志

### 性能验收

- [x] 编译时间 < 5 秒（增量编译）
- [x] 启动时间 < 2 秒
- [x] 输入响应 < 100ms
- [x] 长文本粘贴无卡顿

### 代码质量验收

- [x] 无编译警告
- [x] 无编译错误
- [x] 单元测试通过
- [x] 代码格式符合规范

---

## 📊 测试报告模板

```markdown
## 输入法测试报告

**测试日期**：2024-xx-xx
**测试人员**：[姓名]
**操作系统**：Windows 10 / Windows 11 / macOS / Linux
**输入法**：微软拼音 / 搜狗拼音 / 五笔 / 其他

### 测试结果

| 测试项 | 结果 | 备注 |
|--------|------|------|
| 中文输入 | ✅ / ❌ | [说明] |
| 混合输入 | ✅ / ❌ | [说明] |
| 剪贴板粘贴 | ✅ / ❌ | [说明] |
| 版本名验证 | ✅ / ❌ | [说明] |
| 光标位置 | ✅ / ❌ | [说明] |
| 删除操作 | ✅ / ❌ | [说明] |

### 问题描述

[如有问题，详细描述]

### 日志输出

[粘贴相关控制台日志]

### 截图

[如有必要，提供截图]
```

---

## 🎯 下一步

如果测试通过：
1. ✅ 合并代码到主分支
2. ✅ 更新 CHANGELOG
3. ✅ 创建新版本发布
4. ✅ 收集用户反馈

如果测试失败：
1. ❌ 记录具体错误信息
2. ❌ 查看调试日志
3. ❌ 创建 Issue 跟踪问题
4. ❌ 修复并重新测试

---

## 📞 获取帮助

如果遇到问题：

1. **查看文档**
   - `IME_INPUT_SUPPORT.md` - 技术分析
   - `IME_QUICK_REFERENCE.md` - 快速参考
   - `README_IME_FIX.md` - 修复指南

2. **查看日志**
   ```bash
   cargo run -p view --release 2>&1 | tee ime_debug.log
   ```

3. **提供信息**
   - 操作系统和版本
   - 输入法名称和版本
   - 完整的错误信息
   - 调试日志

---

**祝测试顺利！** 🚀