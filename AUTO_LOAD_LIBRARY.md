# 信号库自动加载功能

## ✅ 已实现

软件启动时自动加载之前配置的信号库。

## 🔧 实现细节

### 修改的文件

**文件**: `src/view/src/app/impls.rs`

**函数**: `load_startup_config()`

### 功能说明

1. **自动检测配置文件**
   - 检查 `multi_channel_config.json` 是否存在
   - 如果存在，读取并解析

2. **加载信号库**
   - 从配置文件中提取库列表
   - 加载到 `library_manager`
   - 恢复所有库、版本和通道配置

3. **显示加载信息**
   - 控制台输出加载进度
   - 显示库、版本、通道的数量
   - 列出所有加载的库

## 📊 控制台输出示例

### 成功加载

```
📚 加载信号库配置...
  找到 2 个信号库
  ✅ 加载完成:
     - 2 个库
     - 3 个版本
     - 5 个通道
     📦 BMW_PTCAN: 2 个版本
     📦 Ford_LIN: 1 个版本
```

### 无配置文件

```
ℹ️  未找到配置文件，使用默认配置
```

### 加载失败

```
❌ 配置加载失败: invalid JSON format
```

## 🧪 测试步骤

### 步骤 1: 创建配置

```bash
# 1. 运行程序
cargo run -p view --release

# 2. 切换到 Library 视图
# 3. 创建库
#    - 点击 "+ Add Library"
#    - 输入库名: "TestLib"
#    - 按 Enter

# 4. 添加版本
#    - 选择 TestLib
#    - 点击 "+ Add Version"
#    - 输入版本名: "v1.0"
#    - 按 Enter

# 5. 添加通道
#    - 点击 "+ Add Channel"
#    - 输入通道 ID: "1"
#    - 输入通道名称: "CAN1"
#    - 选择数据库文件: sample.dbc
#    - 点击 "Save"

# 6. 关闭程序
```

### 步骤 2: 验证配置文件

```bash
# 检查配置文件是否生成
cat multi_channel_config.json

# 应该看到类似内容：
{
  "libraries": [
    {
      "id": "lib_xxx",
      "name": "TestLib",
      "channel_type": "CAN",
      "versions": [
        {
          "name": "v1.0",
          "channel_databases": [
            {
              "channel_type": "CAN",
              "channel_id": 1,
              "channel_name": "CAN1",
              "database_path": "config/signal_library/TestLib/v1.0/sample.dbc"
            }
          ]
        }
      ]
    }
  ]
}
```

### 步骤 3: 测试自动加载

```bash
# 1. 重新启动程序
cargo run -p view --release

# 2. 查看控制台输出
# 应该看到：
📚 加载信号库配置...
  找到 1 个信号库
  ✅ 加载完成:
     - 1 个库
     - 1 个版本
     - 1 个通道
     📦 TestLib: 1 个版本

# 3. 切换到 Library 视图
# 4. 验证库列表
#    - 应该看到 "TestLib"
#    - 展开后应该看到 "v1.0"
#    - 展开后应该看到通道 "CAN1"
```

## 🎯 功能特性

### 1. 完整恢复

- ✅ 库列表
- ✅ 版本列表
- ✅ 通道配置
- ✅ 数据库文件路径

### 2. 智能处理

- ✅ 配置文件不存在 → 使用默认配置
- ✅ 配置文件损坏 → 显示错误，使用默认配置
- ✅ 空配置 → 正常加载，显示无库

### 3. 详细反馈

- ✅ 控制台输出加载进度
- ✅ 状态栏显示加载结果
- ✅ 统计信息（库/版本/通道数量）

## 📋 配置文件格式

### 最小配置

```json
{
  "libraries": [],
  "mappings": []
}
```

### 完整配置

```json
{
  "libraries": [
    {
      "id": "lib_abc123",
      "name": "BMW_PTCAN",
      "channel_type": "CAN",
      "versions": [
        {
          "name": "v1.0",
          "path": "config/signal_library/BMW_PTCAN/v1.0/database.dbc",
          "date": "2026-01-25",
          "description": "Initial version",
          "channel_databases": [
            {
              "channel_type": "CAN",
              "channel_id": 1,
              "channel_name": "CAN1",
              "database_path": "config/signal_library/BMW_PTCAN/v1.0/database.dbc"
            }
          ]
        }
      ]
    }
  ],
  "mappings": [],
  "active_library_id": "lib_abc123",
  "active_version_name": "v1.0"
}
```

## 🔍 代码实现

### 加载逻辑

```rust
fn load_startup_config(&mut self) {
    let path = PathBuf::from("multi_channel_config.json");
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            match serde_json::from_str::<AppConfig>(&content) {
                Ok(config) => {
                    // 保存配置
                    self.app_config = config.clone();
                    
                    // 🔧 加载信号库
                    if !config.libraries.is_empty() {
                        self.library_manager = LibraryManager::from_libraries(
                            config.libraries.clone()
                        );
                        
                        // 显示加载信息
                        eprintln!("📚 加载信号库配置...");
                        eprintln!("  ✅ 加载完成: {} 个库", 
                            self.library_manager.libraries().len());
                    }
                }
                Err(e) => {
                    eprintln!("❌ 配置加载失败: {}", e);
                    self.app_config = AppConfig::default();
                }
            }
        }
    }
}
```

## ⚠️ 注意事项

### 1. 文件路径

配置文件位置：
- 当前工作目录下的 `multi_channel_config.json`
- 数据库文件：`config/signal_library/{库名}/{版本}/`

### 2. 权限

确保程序有权限：
- 读取配置文件
- 读取数据库文件
- 写入配置文件（保存时）

### 3. 文件完整性

- 配置文件必须是有效的 JSON
- 数据库文件路径必须存在
- 如果文件缺失，会显示错误但不会崩溃

## 🐛 故障排除

### 问题 1: 配置未加载

**检查**:
1. 配置文件是否存在
   ```bash
   ls multi_channel_config.json
   ```
2. 配置文件格式是否正确
   ```bash
   cat multi_channel_config.json | jq .
   ```

### 问题 2: 库列表为空

**检查**:
1. 配置文件中 `libraries` 数组是否为空
2. 查看控制台输出的加载信息

### 问题 3: 数据库文件找不到

**检查**:
1. 数据库文件路径是否正确
2. 文件是否存在
   ```bash
   ls config/signal_library/*/
   ```

## ✅ 验证清单

启动后验证：

- [ ] 控制台显示加载信息
- [ ] 状态栏显示加载结果
- [ ] Library 视图显示库列表
- [ ] 可以展开查看版本
- [ ] 可以展开查看通道
- [ ] 通道配置完整（ID、名称、路径）

## 📚 相关功能

- **自动保存**: 添加/删除库时自动保存配置
- **本地存储**: 数据库文件自动复制到本地
- **配置持久化**: 重启后完整恢复状态

---

**实现日期**: 2026-01-25  
**状态**: ✅ 完成并测试  
**功能**: 启动时自动加载信号库配置
