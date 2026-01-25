# 信号库管理界面完善计划

## 需求清单

### 1. ✅ 三栏布局（已实现）
- 左栏：库列表
- 中栏：版本列表  
- 右栏：通道配置
- 用分割线隔开

### 2. 🔧 CAN/LIN 类型选择（需完善）
- [x] 创建库时选择类型
- [ ] UI 显示类型标识
- [ ] 类型筛选功能

### 3. 🔧 通道配置验证（需完善）
- [ ] 通道 ID 必须是整数（添加验证）
- [x] 通道名称输入
- [x] 数据库文件路径选择

### 4. 📁 本地存储（需实现）
- [ ] 使用 SignalLibraryStorage 保存文件
- [ ] 配置文件自动保存
- [ ] 启动时自动加载

### 5. 🎨 UI 优化
- [ ] 更清晰的类型标识
- [ ] 更好的错误提示
- [ ] 输入验证反馈

## 实现步骤

### 步骤 1: 添加通道 ID 验证

**文件**: `src/view/src/ui/views/library_management.rs`

在添加通道输入时验证 ID 必须是整数：

```rust
// 在提交通道时验证
fn validate_channel_id(id_str: &str) -> Result<u16, String> {
    id_str.parse::<u16>()
        .map_err(|_| "通道 ID 必须是 0-65535 之间的整数".to_string())
}
```

### 步骤 2: 集成本地存储

**文件**: `src/view/src/app/impls.rs`

添加版本时自动复制文件到本地：

```rust
use crate::library::SignalLibraryStorage;

impl CanViewApp {
    pub fn add_version_with_storage(
        &mut self,
        library_id: &str,
        version_name: String,
        source_path: &Path,
        description: String,
        channel_dbs: Vec<ChannelDatabase>,
    ) -> Result<(), String> {
        // 1. 获取库信息
        let library = self.library_manager
            .find_library(library_id)
            .ok_or("Library not found")?;
        
        // 2. 初始化存储管理器
        let storage = SignalLibraryStorage::new()
            .map_err(|e| format!("Failed to init storage: {}", e))?;
        
        // 3. 复制所有通道的数据库文件到本地
        let mut local_channel_dbs = Vec::new();
        for channel_db in channel_dbs {
            let source = Path::new(&channel_db.database_path);
            let local_path = storage.copy_database(
                &library.name,
                &version_name,
                source
            ).map_err(|e| format!("Failed to copy database: {}", e))?;
            
            // 使用本地路径创建新的 ChannelDatabase
            let mut local_db = channel_db.clone();
            local_db.database_path = local_path.to_string_lossy().to_string();
            local_channel_dbs.push(local_db);
        }
        
        // 4. 添加版本（使用本地路径）
        self.library_manager.add_version_with_channels(
            library_id,
            version_name,
            description,
            local_channel_dbs,
        )?;
        
        // 5. 保存配置
        self.save_config()?;
        
        Ok(())
    }
}
```

### 步骤 3: 自动加载配置

**文件**: `src/view/src/app/impls.rs`

在应用启动时加载配置：

```rust
impl CanViewApp {
    pub fn load_config_on_startup(&mut self) -> Result<(), String> {
        // 1. 获取配置文件路径
        let config_path = self.get_config_file_path()?;
        
        if !config_path.exists() {
            return Ok(()); // 首次运行，没有配置文件
        }
        
        // 2. 读取配置
        let config_str = std::fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config: {}", e))?;
        
        let config: AppConfig = serde_json::from_str(&config_str)
            .map_err(|e| format!("Failed to parse config: {}", e))?;
        
        // 3. 加载库管理器
        self.library_manager = LibraryManager::from_libraries(config.libraries);
        self.app_config = config;
        
        Ok(())
    }
}
```

### 步骤 4: UI 显示类型标识

**文件**: `src/view/src/ui/views/library_management.rs`

在库列表项中显示类型：

```rust
fn render_library_item(...) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .gap_2()
        .child(
            // 类型标识
            div()
                .px_1()
                .py_0p5()
                .rounded_sm()
                .text_xs()
                .bg(match library.channel_type {
                    ChannelType::CAN => rgb(0x3b82f6), // 蓝色
                    ChannelType::LIN => rgb(0x10b981), // 绿色
                })
                .text_color(rgb(0xffffff))
                .child(match library.channel_type {
                    ChannelType::CAN => "CAN",
                    ChannelType::LIN => "LIN",
                })
        )
        .child(library.name.clone())
        // ...
}
```

### 步骤 5: 添加输入验证反馈

**文件**: `src/view/src/ui/views/library_management.rs`

在通道 ID 输入框旁显示验证状态：

```rust
fn render_channel_id_input(...) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .gap_2()
        .child(
            // 输入框
            Input::new(channel_id_input)
                .placeholder("通道 ID")
                .width(px(80.0))
        )
        .child(
            // 验证状态图标
            if let Ok(_) = new_channel_id.parse::<u16>() {
                div()
                    .text_color(rgb(0x10b981))
                    .child("✓")
            } else if !new_channel_id.is_empty() {
                div()
                    .text_color(rgb(0xef4444))
                    .child("✗ 必须是整数")
            } else {
                div()
            }
        )
}
```

## 配置文件格式

### config/app_config.json

```json
{
  "libraries": [
    {
      "id": "lib_abc123",
      "name": "BMW PTCAN",
      "channel_type": "CAN",
      "versions": [
        {
          "name": "v1.0",
          "path": "config/signal_library/BMW_PTCAN/v1.0/database.dbc",
          "date": "2026-01-25",
          "description": "Initial version",
          "channel_databases": [
            {
              "channel_id": 1,
              "channel_name": "CAN1",
              "database_path": "config/signal_library/BMW_PTCAN/v1.0/database.dbc"
            }
          ]
        }
      ]
    }
  ],
  "mappings": []
}
```

## 目录结构

```
canview/
├── config/
│   ├── app_config.json          # 应用配置
│   └── signal_library/          # 信号库本地存储
│       ├── BMW_PTCAN/
│       │   ├── v1.0/
│       │   │   └── database.dbc
│       │   └── v2.0/
│       │       └── database.dbc
│       └── Ford_LIN/
│           └── v1.5/
│               └── database.ldf
└── src/
    └── view/
        └── src/
            ├── library/
            │   ├── mod.rs
            │   └── storage.rs       # 本地存储管理
            └── ui/
                └── views/
                    └── library_management.rs
```

## 用户流程

### 添加新版本

1. 用户选择库
2. 点击"添加版本"
3. 输入版本名称
4. 配置通道：
   - 输入通道 ID（整数验证）
   - 输入通道名称
   - 选择数据库文件
5. 点击"确认"
6. 系统自动：
   - 复制文件到 `config/signal_library/{库名}/{版本}/`
   - 更新配置
   - 保存到 `config/app_config.json`

### 应用启动

1. 检查 `config/app_config.json` 是否存在
2. 如果存在，加载配置
3. 恢复库列表、版本列表
4. 所有文件路径指向本地存储

## 优先级

1. **高优先级**（立即实现）
   - [x] 通道 ID 整数验证
   - [ ] 本地存储集成
   - [ ] 配置自动保存/加载

2. **中优先级**（后续优化）
   - [ ] UI 类型标识
   - [ ] 输入验证反馈
   - [ ] 错误提示优化

3. **低优先级**（可选）
   - [ ] 类型筛选
   - [ ] 批量导入
   - [ ] 版本比较

## 测试计划

### 单元测试

```rust
#[test]
fn test_channel_id_validation() {
    assert!(validate_channel_id("1").is_ok());
    assert!(validate_channel_id("65535").is_ok());
    assert!(validate_channel_id("abc").is_err());
    assert!(validate_channel_id("65536").is_err());
}

#[test]
fn test_local_storage() {
    let storage = SignalLibraryStorage::new().unwrap();
    let source = Path::new("test.dbc");
    let dest = storage.copy_database("TestLib", "v1.0", source).unwrap();
    assert!(dest.exists());
}
```

### 集成测试

1. 创建库 → 验证配置文件
2. 添加版本 → 验证文件复制
3. 重启应用 → 验证自动加载
4. 删除版本 → 验证文件清理

---

**创建日期**: 2026-01-25  
**状态**: 📋 规划中  
**优先级**: 高
