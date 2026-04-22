# 信号库管理功能重新规划方案

## 一、当前实现分析

### 1.1 现有功能模块

**数据模型层** (`src/models/library.rs`)
- ✅ `SignalLibrary` - 信号库
- ✅ `LibraryVersion` - 版本管理
- ✅ `ChannelDatabase` - 通道数据库配置
- ✅ `DatabaseType` - 数据库类型枚举
- ✅ 完整的序列化支持

**业务逻辑层** (`src/library/mod.rs`)
- ✅ `LibraryManager` - 库管理器
- ✅ CRUD操作（创建、删除、查找）
- ✅ 版本管理（添加、删除版本）
- ✅ 数据库验证和统计
- ✅ DBC/LDF文件加载

**UI层** (`src/ui/views/library_view.rs`)
- ✅ 库列表组件
- ✅ 版本列表组件
- ✅ 基础交互逻辑

### 1.2 当前问题

**架构问题**
- ❌ UI组件未完全集成到主应用
- ❌ 功能分散在多个模块，缺少统一的工作流
- ❌ 缺少用户引导和帮助信息

**用户体验问题**
- ❌ 没有清晰的"导入 → 管理 → 使用"流程
- ❌ 缺少快速操作入口
- ❌ 版本切换不够直观
- ❌ 多通道配置复杂度较高

**功能缺失**
- ❌ 没有数据库文件浏览器
- ❌ 缺少批量导入功能
- ❌ 没有库的搜索和过滤
- ❌ 缺少使用历史记录
- ❌ 没有库的导出/分享功能

---

## 二、重新设计目标

### 2.1 核心原则

1. **简单优先** - 从单文件快速导入开始，逐步引导到高级功能
2. **可视化优先** - 用图形界面展示库、版本、通道的关系
3. **流程驱动** - 按实际使用流程组织功能（导入 → 管理 → 应用）
4. **渐进式复杂** - 基础功能简单，高级功能可选

### 2.2 用户场景

**场景A：快速查看单个DBC文件**
```
用户需求：打开一个BLF文件，需要查看DBC信号
操作流程：
1. 打开BLF文件
2. 点击"导入数据库"
3. 选择DBC文件
4. 立即看到信号解析
```

**场景B：管理车辆项目的多个DBC版本**
```
用户需求：管理BMW项目的v1.0、v2.0、v3.0版本
操作流程：
1. 创建"BMW PT-CAN"库
2. 导入v1.0文件
3. 导入v2.0文件
4. 在不同版本间快速切换对比
```

**场景C：配置多通道系统**
```
用户需求：配置包含CAN通道1-16和LIN通道1-4的系统
操作流程：
1. 创建"车辆诊断系统"库
2. 为每个通道添加对应的数据库
3. 保存配置
4. 后续一键加载完整配置
```

---

## 三、新UI架构设计

### 3.1 整体布局

```
┌─────────────────────────────────────────────────────────┐
│  CanView                            🔧 Settings 👤 Help │
├─────────────────────────────────────────────────────────┤
│  📋 Logs  🔧 Config 📊 Analytics  📚 Library          │  <- 导航栏
├─────────────────────────────────────────────────────────┤
│                                                           │
│  ┌─────────────┬──────────────────────────────────┐    │
│  │             │                                  │    │
│  │  Library    │   Library Detail View           │    │
│  │  List       │                                  │    │
│  │             │   ┌──────────────────────────┐  │    │
│  │  📁 BMW     │   │ Version: v1.0             │  │    │
│  │  📁 Ford    │   │ Date: 2024-01-15          │  │    │
│  │  📁 Toyota  │   │ Channels: 1, 2, 3         │  │    │
│  │  📁 + New   │   │                          │  │    │
│  │             │   │ [Load] [Edit] [Delete]   │  │    │
│  │             │   └──────────────────────────┘  │    │
│  │             │                                  │    │
│  │             │   Versions:                    │    │
│  │             │   • v1.0 (current)            │    │
│  │             │   • v0.9                      │    │
│  │             │   • v0.8                      │    │
│  │             │                                  │    │
│  │             │   [+ Add Version]              │    │
│  │             │                                  │    │
│  └─────────────┴──────────────────────────────────┘    │
│                                                           │
│  Quick Actions:                                           │
│  [📥 Import DBC/LDF]  [🔍 Search]  [📤 Export]      │
└─────────────────────────────────────────────────────────┘
```

### 3.2 三栏布局（详情视图）

```
┌──────────────────────────────────────────────────────────┐
│  Libraries │ Versions │ Channel Config                  │
│            │          │                                  │
│  BMW  ✓    │ v1.0 ✓   │  CH1: /path/to/ch1.dbc      [x] │
│  Ford      │ v0.9     │  CH2: /path/to/ch2.dbc      [x] │
│  Toyota    │ v0.8     │  CH3: /path/to/ch3.dbc      [x] │
│  + New     │ + Add    │  [+ Add Channel]                 │
│            │          │                                  │
│  Search:   │          │  Apply to:                       │
│  [_____]   │          │  □ All channels at once         │
│            │          │  □ Selected channels only       │
└──────────────────────────────────────────────────────────┘
```

---

## 四、功能模块重新设计

### 4.1 快速导入（Quick Import）

**目的：** 让用户最快速度开始使用

**UI设计：**
```
┌─────────────────────────────────┐
│  Import Database File           │
├─────────────────────────────────┤
│                                 │
│  Select a DBC or LDF file:      │
│  [Browse...]                    │
│                                 │
│  File: bmw_ptcan_2024.dbc       │
│  Type: DBC                      │
│  Messages: 156                  │
│  Signals: 1,234                 │
│                                 │
│  Options:                       │
│  □ Add to existing library       │
│  □ Create new library           │
│  Library name: [BMW PT-CAN]     │
│                                 │
│  [Cancel] [Import & Load]       │
│                                 │
└─────────────────────────────────┘
```

**工作流程：**
1. 用户点击"Import Database"按钮
2. 选择文件
3. 自动验证并显示统计信息
4. 用户选择添加到现有库或创建新库
5. 立即加载并应用

**代码结构：**
```rust
// 新增：快速导入模块
mod quick_import {
    pub struct QuickImportWizard {
        step: ImportStep,
        selected_file: Option<PathBuf>,
        validation_result: Option<DatabaseValidation>,
        target_library: Option<TargetLibrary>,
    }

    enum ImportStep {
        SelectFile,
        ReviewInfo,
        ChooseTarget,
    }
}
```

### 4.2 库浏览器（Library Browser）

**目的：** 可视化展示所有库和版本

**特性：**
- 📁 树形视图：库 → 版本 → 通道
- 🔍 搜索和过滤
- 🏷️ 标签系统（项目、车辆类型等）
- 📊 统计信息卡片
- 🟢 状态指示（使用中、未使用、过期）

**UI设计：**
```
┌─────────────────────────────────────────────────┐
│  Library Browser                    🔍 [Search] │
├─────────────────────────────────────────────────┤
│                                                  │
│  Filter: [All] [In Use] [CAN] [LIN]            │
│                                                  │
│  ┌──────────────────────────────────────────┐   │
│  │ 📁 BMW PT-CAN (CAN)                       │   │
│  │    3 versions • Used by CH1-3            │   │
│  │    🏷️ #production #diagnostics            │   │
│  │    v1.0 • v0.9 • v0.8                      │   │
│  │    [Load] [Edit] [Export]                  │   │
│  ├──────────────────────────────────────────┤   │
│  │ 📁 Ford LIN (LIN)                         │   │
│  │    1 version • Not in use                 │   │
│  │    v1.0                                    │   │
│  └──────────────────────────────────────────┘   │
│                                                  │
│  [+ New Library]                                │
└─────────────────────────────────────────────────┘
```

### 4.3 版本对比（Version Diff）

**目的：** 对比不同版本间的差异

**UI设计：**
```
┌─────────────────────────────────────────────────┐
│  Version Comparison: v1.0 vs v0.9               │
├─────────────────────────────────────────────────┤
│                                                  │
│  Added Messages: 12                             │
│  Removed Messages: 3                             │
│  Modified Signals: 45                            │
│                                                  │
│  ┌─ Message Changes ─────────────────────────┐   │
│  │  + MSG_0x100 (New)                         │   │
│  │  + MSG_0x101 (New)                         │   │
│  │  - MSG_0x200 (Removed)                     │   │
│  │  ~ MSG_0x300 (Modified)                    │   │
│  └──────────────────────────────────────────┘   │
│                                                  │
│  [Apply v1.0] [Keep v0.9]                       │
└─────────────────────────────────────────────────┘
```

### 4.4 多通道配置器（Multi-Channel Configurator）

**目的：** 简化多通道系统配置

**特性：**
- 📋 通道矩阵视图
- 🔄 批量操作
- 📋 模板保存和加载
- ✅ 配置验证

**UI设计：**
```
┌─────────────────────────────────────────────────┐
│  Channel Configuration                          │
├─────────────────────────────────────────────────┤
│                                                  │
│  Library: BMW PT-CAN v1.0                       │
│                                                  │
│  Channel Matrix:                                 │
│  ┌───┬──────────────────────────────────────┐   │
│  │   │  CH1  CH2  CH3  CH4  CH5  CH6  ...   │   │
│  │   │  ✓    ✓    ✓    ✓    ✓    ✓         │   │
│  ├───┼──────────────────────────────────────┤   │
│  │DB │  📋   📋   📋   📋   📋   📋        │   │
│  └───┴──────────────────────────────────────┘   │
│                                                  │
│  Selected: 6 channels                           │
│  [Apply to Selected] [Apply to All]             │
│                                                  │
│  Quick Templates:                                │
│  [All CAN] [All LIN] [First 8] [Custom...]      │
│                                                  │
└─────────────────────────────────────────────────┘
```

---

## 五、实现计划

### 阶段1：核心功能重构（优先级：高）

**目标：** 建立清晰的基础架构

**任务列表：**
1. ✅ 重构 `LibraryManager` API，简化核心操作
2. ✅ 实现 `QuickImportWizard` 组件
3. ✅ 创建统一的 `LibraryService` 接口
4. ✅ 添加文件变更监控（自动重载）
5. ✅ 实现库的搜索和过滤

**代码结构：**
```rust
// 新的服务层
pub mod services {
    pub trait LibraryService {
        async fn import_file(&mut self, path: PathBuf) -> Result<ImportResult>;
        async fn load_version(&mut self, lib_id: &str, ver: &str) -> Result<()>;
        async fn switch_version(&mut self, lib_id: &str, from: &str, to: &str) -> Result<VersionDiff>;
        fn search_libraries(&self, query: &str) -> Vec<LibraryItem>;
    }
}
```

### 阶段2：UI组件开发（优先级：高）

**任务列表：**
1. ✅ 实现 `LibraryBrowser` 组件（树形视图）
2. ✅ 实现 `VersionComparisonView` 组件
3. ✅ 实现 `ChannelMatrix` 组件
4. ✅ 添加拖拽支持（库排序、版本重排）
5. ✅ 实现右键菜单（上下文操作）

### 阶段3：高级功能（优先级：中）

**任务列表：**
1. ✅ 批量导入功能
2. ✅ 库模板系统
3. ✅ 导出/分享功能（打包库和配置）
4. ✅ 使用统计和报告
5. ✅ 自动备份和版本控制

### 阶段4：体验优化（优先级：中）

**任务列表：**
1. ✅ 键盘快捷键
2. ✅ 撤销/重做支持
3. ✅ 操作提示和帮助
4. ✅ 性能优化（大型库加载）
5. ✅ 离线模式支持

---

## 六、数据模型优化

### 6.1 增强的元数据

```rust
pub struct SignalLibrary {
    pub id: String,
    pub name: String,
    pub channel_type: ChannelType,
    pub versions: Vec<LibraryVersion>,

    // 新增字段
    pub tags: Vec<String>,              // 标签
    pub project: Option<String>,         // 关联项目
    pub description: String,            // 描述
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub usage_count: usize,              // 使用次数
    pub last_used: Option<chrono::DateTime<chrono::Utc>>,
}

pub struct LibraryVersion {
    pub name: String,
    pub path: String,
    pub date: String,
    pub description: String,
    pub channel_databases: Vec<ChannelDatabase>,

    // 新增字段
    pub checksum: String,               // 文件校验和
    pub file_size: u64,                 // 文件大小
    pub is_valid: bool,                 // 验证状态
    pub validation_error: Option<String>,
    pub stats: DatabaseStats,           // 详细统计
    pub author: Option<String>,         // 作者
    pub changelog: String,              // 变更日志
}
```

### 6.2 配置持久化

```rust
pub struct LibraryConfig {
    // 库配置
    pub libraries: Vec<SignalLibrary>,

    // 用户偏好
    pub auto_reload: bool,
    pub default_library: Option<String>,
    pub view_preferences: ViewPreferences,

    // 使用统计
    pub recent_libraries: Vec<RecentLibrary>,
    pub usage_stats: HashMap<String, LibraryStats>,
}

pub struct RecentLibrary {
    pub library_id: String,
    pub version_name: String,
    pub last_used: chrono::DateTime<chrono::Utc>,
    pub use_count: usize,
}
```

---

## 七、API设计

### 7.1 核心API

```rust
impl LibraryManager {
    // === 基础CRUD ===
    pub fn create_library(&mut self, name: String, ty: ChannelType)
        -> Result<&SignalLibrary>;
    pub fn delete_library(&mut self, id: &str) -> Result<()>;
    pub fn update_library(&mut self, id: String, updates: LibraryUpdates)
        -> Result<()>;

    // === 版本管理 ===
    pub fn add_version(&mut self, lib_id: &str, version: VersionSpec)
        -> Result<()>;
    pub fn remove_version(&mut self, lib_id: &str, ver: &str)
        -> Result<()>;
    pub fn switch_version(&mut self, lib_id: &str, to: &str)
        -> Result<VersionDiff>;

    // === 查询和搜索 ===
    pub fn list_libraries(&self) -> Vec<LibraryItem>;
    pub fn search(&self, query: &SearchQuery) -> Vec<LibraryItem>;
    pub fn get_library_detail(&self, id: &str) -> LibraryDetail;

    // === 批量操作 ===
    pub fn batch_import(&mut self, files: Vec<PathBuf>)
        -> Vec<ImportResult>;
    pub fn batch_delete(&mut self, ids: Vec<&str>)
        -> Result<Vec<String>>;

    // === 高级功能 ===
    pub fn compare_versions(&self, lib_id: &str, v1: &str, v2: &str)
        -> VersionDiff;
    pub fn export_library(&self, id: &str, format: ExportFormat)
        -> Result<Vec<u8>>;
    pub fn import_library(&mut self, data: &[u8])
        -> Result<ImportResult>;
}
```

### 7.2 异步API

```rust
#[async_trait]
pub trait AsyncLibraryService {
    async fn validate_file(&self, path: PathBuf)
        -> Result<DatabaseValidation>;
    async fn load_database(&self, path: PathBuf, ty: ChannelType)
        -> Result<Database>;
    async fn auto_reload(&self, lib_id: &str)
        -> Result<ReloadResult>;
}
```

---

## 八、用户工作流设计

### 8.1 首次使用流程

```
启动应用
   │
   ├─> 欢迎向导
   │   ├─> "要查看BLF文件？"
   │   │   └─> [打开BLF] → 自动检测DBC → 推荐导入
   │   │
   │   └─> "有DBC/LDF文件？"
   │       └─> [导入文件] → 创建库 → 立即使用
   │
   └─> 完成引导 → 进入主界面
```

### 8.2 日常使用流程

```
打开应用
   │
   ├─> 最近使用的库（自动显示在顶部）
   │   └─> 点击即可加载
   │
   ├─> 打开BLF文件
   │   └─> 自动加载关联的库
   │
   └─> 切换库版本
       └─> Library Browser → 选择版本 → Load
```

---

## 九、关键技术决策

### 9.1 性能优化

**问题：** 大型DBC文件（>10MB）加载缓慢

**解决方案：**
1. **延迟加载** - 只加载元数据，消息内容按需加载
2. **缓存机制** - 缓存解析后的数据库对象
3. **增量加载** - 支持分块加载大型文件
4. **后台验证** - 在后台线程中验证文件

```rust
pub struct CachedDatabase {
    metadata: DatabaseMetadata,  // 轻量级元数据
    db: OnceLock<Database>,        // 延迟加载的完整数据库
    cache_key: String,             // 缓存键
}
```

### 9.2 数据一致性

**问题：** 配置文件和内存状态可能不同步

**解决方案：**
1. **原子更新** - 使用临时文件 + 重命名
2. **变更日志** - 记录所有变更操作
3. **自动保存** - 定时自动保存
4. **冲突检测** - 检测文件被外部修改

### 9.3 错误处理

**原则：**
- 用户友好的错误消息
- 自动恢复（尽可能）
- 详细的日志记录
- 提供解决建议

```rust
pub enum LibraryError {
    FileNotFound { path: String, suggestion: String },
    InvalidDatabase { error: String, line: Option<usize> },
    VersionConflict { current: String, attempted: String },
    LibraryInUse { library: String, channels: Vec<u16> },

    // 自动恢复
    #[serde(skip)]
    recover: Option<RecoveryAction],
}

pub enum RecoveryAction {
    Retry,
    Skip,
    UseBackup(PathBuf),
    ResetToDefault,
}
```

---

## 十、测试策略

### 10.1 单元测试

```rust
#[cfg(test)]
mod tests {
    // LibraryManager核心功能
    test_create_library();
    test_add_version();
    test_delete_library_in_use();
    test_version_comparison();

    // 数据验证
    test_validate_dbc();
    test_validate_ldf();
    test_detect_file_type();

    // 并发测试
    test_concurrent_access();
    test_race_conditions();
}
```

### 10.2 集成测试

```rust
#[tokio::test]
async test_import_workflow() {
    // 模拟完整的导入流程
    let manager = LibraryManager::new();

    // 1. 创建库
    manager.create_library("Test".into(), ChannelType::CAN)?;

    // 2. 添加版本
    manager.add_version("lib_xxx", "v1.0".into(), path.into(), desc.into())?;

    // 3. 加载并验证
    let db = manager.load_database(&path, ChannelType::CAN)?;
    assert!(db.message_count() > 0);
}
```

### 10.3 UI测试

```rust
#[test]
fn test_library_browser_render() {
    // 测试库浏览器渲染
    let libraries = create_test_libraries();
    let element = render_library_list(&libraries, &None, &[], cx);

    // 验证渲染结果
    assert!(element.text().contains("BMW"));
}
```

---

## 十一、文档和培训

### 11.1 用户文档

1. **快速开始指南** - 5分钟上手
2. **完整教程** - 详细功能说明
3. **视频教程** - 演示常见操作
4. **FAQ** - 常见问题解答

### 11.2 开发者文档

1. **架构设计文档**
2. **API参考手册**
3. **贡献指南**
4. **故障排除指南**

---

## 十二、成功指标

### 12.1 用户体验指标

- ⏱️ 导入时间 < 5秒（普通DBC文件）
- 🎯 新用户5分钟内完成首次导入
- 📊 版本切换时间 < 1秒
- 💾 配置保存/加载 < 100ms

### 12.2 功能覆盖率

- ✅ 支持DBC和LDF文件
- ✅ 单通道和多通道配置
- ✅ 版本管理和对比
- ✅ 搜索和过滤
- ✅ 批量操作
- ✅ 导入/导出

### 12.3 稳定性指标

- 🐛 零崩溃（导入、切换、加载）
- 🔄 100%数据一致性
- 📉 < 1%文件验证失败
- 🚀 响应时间 < 100ms（UI操作）

---

## 十三、实施时间表

### Phase 1（第1-2周）：基础重构
- 重构LibraryManager API
- 实现QuickImportWizard
- 添加搜索和过滤

### Phase 2（第3-4周）：UI开发
- LibraryBrowser组件
- 版本对比视图
- 多通道配置器

### Phase 3（第5-6周）：高级功能
- 批量导入
- 模板系统
- 导出/分享

### Phase 4（第7-8周）：优化和测试
- 性能优化
- 用户体验优化
- 测试和文档

---

## 十四、总结

本重新规划方案聚焦于：

✅ **用户优先** - 简化核心流程，快速上手
✅ **功能完整** - 覆盖从导入到应用的全流程
✅ **可扩展** - 模块化设计，易于添加新功能
✅ **高性能** - 优化大型库的加载和切换
✅ **可维护** - 清晰的架构，完善的测试

**下一步行动：**
1. 与团队讨论并确认方案
2. 创建详细的UI原型
3. 开始Phase 1实施
