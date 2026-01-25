# 信号库加载调试指南

## 🔍 问题诊断

### 症状
新打开软件时没有加载之前配置的信号库。

### 可能原因

1. **配置文件位置不对**
2. **加载逻辑未执行**
3. **UI 未刷新**
4. **数据未正确反序列化**

## 📋 诊断步骤

### 步骤 1: 检查配置文件

```powershell
# 检查配置文件是否存在
ls .\target\release\multi_channel_config.json

# 查看内容
cat .\target\release\multi_channel_config.json | ConvertFrom-Json | ConvertTo-Json -Depth 10
```

**预期结果**: 应该看到完整的库、版本、通道信息

### 步骤 2: 运行程序并查看控制台

```powershell
# 从 release 目录运行
cd .\target\release
.\view.exe
```

**预期输出**:
```
📚 加载信号库配置...
  找到 1 个信号库
  ✅ 加载完成:
     - 1 个库
     - 1 个版本
     - 1 个通道
     📦 123: 1 个版本
```

### 步骤 3: 检查 UI

1. 启动程序
2. 切换到 "Library" 视图
3. 检查左侧库列表是否显示

## 🐛 常见问题

### 问题 1: 控制台没有加载信息

**原因**: `load_startup_config` 未执行

**检查**:
```rust
// 在 new() 函数中应该有这行
app.load_startup_config();
```

**位置**: `src/view/src/app/impls.rs:19`

### 问题 2: 配置文件为空或格式错误

**检查**:
```powershell
# 验证 JSON 格式
cat .\target\release\multi_channel_config.json | ConvertFrom-Json
```

**修复**: 删除配置文件，重新创建库

### 问题 3: UI 不显示但控制台有加载信息

**原因**: UI 未刷新或数据绑定问题

**检查**: Library 视图的渲染逻辑

## 🔧 手动测试

### 完整测试流程

```powershell
# 1. 清理旧配置
Remove-Item .\target\release\multi_channel_config.json -ErrorAction SilentlyContinue
Remove-Item .\target\release\config -Recurse -ErrorAction SilentlyContinue

# 2. 启动程序
cd .\target\release
.\view.exe

# 3. 在程序中:
#    - 切换到 Library 视图
#    - 点击 "+ Add Library"
#    - 输入库名: "TestLib"
#    - 按 Enter
#    - 点击 "+ Add Version"
#    - 输入版本名: "v1.0"
#    - 按 Enter
#    - 点击 "+ Add Channel"
#    - 输入通道 ID: "1"
#    - 输入通道名称: "CAN1"
#    - 选择文件: ..\..\sample.dbc
#    - 点击 "Save"

# 4. 验证配置文件
cat .\multi_channel_config.json

# 5. 关闭程序

# 6. 重新启动
.\view.exe

# 7. 查看控制台输出
# 应该看到:
📚 加载信号库配置...
  找到 1 个信号库
  ✅ 加载完成:
     - 1 个库
     - 1 个版本
     - 1 个通道
     📦 TestLib: 1 个版本

# 8. 切换到 Library 视图
# 应该看到 TestLib
```

## 📊 调试输出

### 添加更多调试信息

如果需要更详细的调试信息，可以在代码中添加：

```rust
// 在 load_startup_config 开始处
eprintln!("🔍 DEBUG: load_startup_config called");
eprintln!("🔍 DEBUG: Current dir: {:?}", std::env::current_dir());
eprintln!("🔍 DEBUG: Config path: multi_channel_config.json");

// 在读取文件后
eprintln!("🔍 DEBUG: Config file exists: {}", path.exists());
eprintln!("🔍 DEBUG: Config content length: {} bytes", content.len());

// 在解析后
eprintln!("🔍 DEBUG: Parsed libraries count: {}", config.libraries.len());
```

## ✅ 验证清单

启动后检查：

- [ ] 控制台显示 "📚 加载信号库配置..."
- [ ] 控制台显示库数量
- [ ] 控制台显示版本数量
- [ ] 控制台显示通道数量
- [ ] 控制台列出所有库名
- [ ] Library 视图显示库列表
- [ ] 可以展开库查看版本
- [ ] 可以展开版本查看通道

## 🎯 快速测试脚本

使用提供的测试脚本：

```powershell
.\test-autoload.ps1
```

这个脚本会：
1. 检查配置文件
2. 显示配置内容
3. 启动程序

## 📝 当前配置文件内容

根据您的配置文件：

```json
{
  "libraries": [
    {
      "id": "lib_aeaeb6346d324d62",
      "name": "123",
      "channel_type": "CAN",
      "versions": [
        {
          "name": "123",
          "path": "",
          "date": "2026-01-25",
          "description": "Created version '123' (database file to be added)",
          "channel_databases": [
            {
              "channel_type": "CAN",
              "channel_id": 1,
              "channel_name": "2",
              "database_path": "C:\\Users\\Administrator\\RustroverProjects\\canview\\target\\release\\config\\signal_library\\123\\123\\database.dbc"
            }
          ]
        }
      ]
    }
  ]
}
```

**预期加载结果**:
- 1 个库: "123"
- 1 个版本: "123"
- 1 个通道: "2" (ID: 1)

## 🔄 如果仍然不加载

### 方案 1: 检查代码

确认 `load_startup_config` 在 `new()` 中被调用：

```rust
pub fn new() -> Self {
    let mut app = Self {
        // ... 初始化字段 ...
    };
    
    // 启动时加载配置
    app.load_startup_config();  // ← 这行必须存在
    
    app
}
```

### 方案 2: 添加日志

在 `load_startup_config` 开始处添加：

```rust
fn load_startup_config(&mut self) {
    eprintln!("🚀 load_startup_config 开始执行");
    
    let path = PathBuf::from("multi_channel_config.json");
    eprintln!("📂 配置文件路径: {:?}", path);
    eprintln!("✓ 文件存在: {}", path.exists());
    
    // ... 其余代码 ...
}
```

### 方案 3: 检查 from_libraries

确认 `LibraryManager::from_libraries` 正确实现：

```rust
impl LibraryManager {
    pub fn from_libraries(libraries: Vec<SignalLibrary>) -> Self {
        Self { libraries }
    }
}
```

## 💡 建议

1. **使用测试脚本**: 运行 `.\test-autoload.ps1`
2. **查看控制台**: 启动时仔细查看所有输出
3. **检查 UI**: 确保切换到 Library 视图
4. **验证文件**: 确认配置文件格式正确

---

**创建日期**: 2026-01-25  
**状态**: 调试中  
**配置文件**: `target/release/multi_channel_config.json`
