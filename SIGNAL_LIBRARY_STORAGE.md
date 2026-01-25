# 信号库本地存储功能

## 概述

信号库本地存储功能提供了一个结构化的方式来管理和存储数据库文件。文件按照"库名 → 版本"的层级结构组织，便于管理和查找。

## 目录结构

```
config/
└── signal_library/
    ├── BMW_PTCAN/                    # 库名
    │   ├── v1.0/                     # 版本名
    │   │   └── database.dbc          # 数据库文件
    │   └── v2.0/
    │       └── database.dbc
    ├── Ford_LIN/
    │   └── v1.5/
    │       └── database.ldf
    └── Tesla_CAN/
        └── 2024-01-25/
            └── database.dbc
```

## 功能特性

- ✅ 自动创建目录结构
- ✅ 文件名清理（移除非法字符）
- ✅ 支持 DBC 和 LDF 文件
- ✅ 文件复制和管理
- ✅ 列出所有库和版本
- ✅ 删除库或版本

## 使用方法

### 1. 创建存储管理器

```rust
use view::library::SignalLibraryStorage;

// 创建存储管理器（自动创建基础目录）
let storage = SignalLibraryStorage::new()?;

// 查看基础路径
println!("Storage path: {:?}", storage.base_path());
```

### 2. 复制数据库文件

```rust
use std::path::Path;

// 复制数据库文件到本地存储
let source_path = Path::new("C:/databases/bmw_ptcan.dbc");
let dest_path = storage.copy_database(
    "BMW_PTCAN",           // 库名
    "v1.0",                // 版本名
    source_path            // 源文件路径
)?;

println!("Database copied to: {:?}", dest_path);
// 输出: Database copied to: "config/signal_library/BMW_PTCAN/v1.0/database.dbc"
```

### 3. 获取数据库路径

```rust
// 获取已存储的数据库文件路径
if let Some(db_path) = storage.get_database_path("BMW_PTCAN", "v1.0") {
    println!("Database found at: {:?}", db_path);
    // 可以使用这个路径加载数据库
} else {
    println!("Database not found");
}
```

### 4. 列出所有库

```rust
// 列出所有已存储的库
let libraries = storage.list_libraries()?;
for lib in libraries {
    println!("Library: {}", lib);
}
```

### 5. 列出库的所有版本

```rust
// 列出特定库的所有版本
let versions = storage.list_versions("BMW_PTCAN")?;
for ver in versions {
    println!("Version: {}", ver);
}
```

### 6. 删除库或版本

```rust
// 删除特定版本
storage.delete_version("BMW_PTCAN", "v1.0")?;

// 删除整个库
storage.delete_library("BMW_PTCAN")?;
```

## 集成到应用程序

### 在应用状态中添加存储管理器

**文件**: `src/view/src/app/state.rs`

```rust
use crate::library::SignalLibraryStorage;

pub struct CanViewApp {
    // ... 其他字段
    
    /// 信号库本地存储管理器
    pub signal_storage: SignalLibraryStorage,
}

impl CanViewApp {
    pub fn new(cx: &mut Context<Self>) -> Self {
        // 初始化存储管理器
        let signal_storage = SignalLibraryStorage::new()
            .expect("Failed to initialize signal library storage");
        
        Self {
            // ... 其他字段初始化
            signal_storage,
        }
    }
}
```

### 在添加版本时自动复制文件

**文件**: `src/view/src/app/impls.rs`

```rust
impl CanViewApp {
    /// 添加版本并复制数据库文件到本地存储
    pub fn add_version_with_storage(
        &mut self,
        library_id: &str,
        version_name: String,
        source_path: &Path,
        description: String,
    ) -> Result<(), String> {
        // 1. 获取库信息
        let library = self.library_manager
            .find_library(library_id)
            .ok_or("Library not found")?;
        
        // 2. 复制文件到本地存储
        let local_path = self.signal_storage
            .copy_database(&library.name, &version_name, source_path)
            .map_err(|e| format!("Failed to copy database: {}", e))?;
        
        // 3. 使用本地路径添加版本
        self.library_manager.add_version(
            library_id,
            version_name,
            local_path.to_string_lossy().to_string(),
            description,
        )?;
        
        Ok(())
    }
}
```

### 在删除版本时清理本地文件

```rust
impl CanViewApp {
    /// 删除版本并清理本地存储
    pub fn remove_version_with_cleanup(
        &mut self,
        library_id: &str,
        version_name: &str,
    ) -> Result<(), String> {
        // 1. 获取库信息
        let library = self.library_manager
            .find_library(library_id)
            .ok_or("Library not found")?;
        
        // 2. 从库管理器中删除版本
        self.library_manager.remove_version(
            library_id,
            version_name,
            &self.app_config.mappings,
        )?;
        
        // 3. 清理本地存储
        self.signal_storage
            .delete_version(&library.name, version_name)
            .map_err(|e| format!("Failed to delete local files: {}", e))?;
        
        Ok(())
    }
}
```

## 配置选项

### 自定义存储路径

如果需要自定义存储路径，可以修改 `SignalLibraryStorage::get_base_path()` 方法：

```rust
fn get_base_path() -> Result<PathBuf> {
    // 方法 1: 使用环境变量
    if let Ok(custom_path) = std::env::var("CANVIEW_STORAGE_PATH") {
        return Ok(PathBuf::from(custom_path));
    }
    
    // 方法 2: 使用用户主目录
    if let Some(home_dir) = dirs::home_dir() {
        return Ok(home_dir.join(".canview").join("signal_library"));
    }
    
    // 默认：程序所在目录
    // ...
}
```

## 最佳实践

### 1. 错误处理

```rust
match storage.copy_database("BMW_PTCAN", "v1.0", source_path) {
    Ok(path) => {
        println!("✅ Database copied successfully");
        // 继续处理
    }
    Err(e) => {
        eprintln!("❌ Failed to copy database: {}", e);
        // 显示错误给用户
    }
}
```

### 2. 检查文件是否已存在

```rust
// 在复制前检查是否已存在
if let Some(existing_path) = storage.get_database_path("BMW_PTCAN", "v1.0") {
    println!("⚠️ Database already exists at: {:?}", existing_path);
    // 询问用户是否覆盖
} else {
    // 复制新文件
    storage.copy_database("BMW_PTCAN", "v1.0", source_path)?;
}
```

### 3. 批量操作

```rust
// 批量复制多个版本
let versions = vec![
    ("v1.0", "path/to/v1.0.dbc"),
    ("v1.1", "path/to/v1.1.dbc"),
    ("v2.0", "path/to/v2.0.dbc"),
];

for (version, path) in versions {
    match storage.copy_database("BMW_PTCAN", version, Path::new(path)) {
        Ok(_) => println!("✅ Copied {}", version),
        Err(e) => eprintln!("❌ Failed to copy {}: {}", version, e),
    }
}
```

## 注意事项

1. **文件名清理**: 库名和版本名中的特殊字符（如 `/`, `\`, `:` 等）会被替换为 `_`
2. **文件扩展名**: 系统会自动检测并保留原文件的扩展名（.dbc 或 .ldf）
3. **目录权限**: 确保程序有权限在目标目录创建文件和文件夹
4. **磁盘空间**: 复制文件会占用额外的磁盘空间

## 故障排除

### 问题：无法创建目录

```
Error: Failed to create signal library base directory
```

**解决方案**:
- 检查程序是否有写入权限
- 确认磁盘空间充足
- 尝试以管理员权限运行

### 问题：文件复制失败

```
Error: Failed to copy database from ... to ...
```

**解决方案**:
- 确认源文件存在且可读
- 检查目标目录是否可写
- 确认文件未被其他程序占用

### 问题：找不到数据库文件

```
get_database_path() returns None
```

**解决方案**:
- 确认库名和版本名正确
- 检查文件是否已被删除
- 使用 `list_libraries()` 和 `list_versions()` 查看实际存在的内容

## 相关文件

- `src/view/src/library/storage.rs` - 存储管理器实现
- `src/view/src/library/mod.rs` - 库管理模块
- `src/view/src/app/state.rs` - 应用状态
- `src/view/src/app/impls.rs` - 应用实现

## 测试

运行单元测试：

```bash
cargo test -p view --lib library::storage
```

---

**创建日期**: 2026-01-25  
**版本**: 1.0  
**状态**: ✅ 已实现
