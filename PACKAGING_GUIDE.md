# 软件打包说明

## ✅ 已完成的改进

### 1. 隐藏控制台窗口

**问题**: Release 版本运行时会显示黑色控制台窗口

**解决**: 
- 修改了 `src/view/build.rs`
- 在 Release 模式下设置 Windows 子系统为 GUI
- Debug 模式仍然显示控制台（方便调试）

**文件**: `src/view/build.rs`

```rust
// 在 Release 模式下设置 Windows 子系统为 GUI（隐藏控制台）
#[cfg(not(debug_assertions))]
{
    println!("cargo:rustc-link-arg=/SUBSYSTEM:WINDOWS");
    println!("cargo:rustc-link-arg=/ENTRY:mainCRTStartup");
}
```

### 2. 完整的打包结构

**问题**: 打包后只有一个可执行文件，缺少配置目录

**解决**: 
- 更新了 `package.ps1` 打包脚本
- 自动创建完整的目录结构
- 包含配置文件、示例文件、文档等

**目录结构**:
```
CANVIEW-v1.0.0/
├── bin/
│   └── canview.exe          # 主程序（无控制台窗口）
├── config/
│   ├── signal_library/      # 信号库本地存储
│   │   └── README.txt       # 存储说明
│   ├── default_config.json  # 默认配置
│   └── example_config.json  # 配置示例
├── samples/
│   ├── sample.dbc           # DBC 示例
│   └── sample.blf           # BLF 示例
├── docs/
│   ├── README.md            # 使用说明
│   └── BUILD.md             # 编译说明
├── assets/                  # 资源文件
├── start.bat                # 启动脚本
└── README.txt               # 发布说明
```

## 🚀 打包步骤

### 方法 1: 使用打包脚本（推荐）

```powershell
# 运行打包脚本
.\package.ps1 -Version "1.0.0"

# 或者使用默认版本
.\package.ps1
```

**输出**:
- 文件夹: `release-package\CANVIEW-v1.0.0\`
- 压缩包: `release-package\CANVIEW-v1.0.0.zip`

### 方法 2: 手动打包

```powershell
# 1. 编译 Release 版本
cargo build --release -p view

# 2. 创建目录结构
mkdir release\bin
mkdir release\config\signal_library
mkdir release\samples
mkdir release\docs

# 3. 复制文件
copy target\release\view.exe release\bin\canview.exe
copy sample.dbc release\samples\
copy sample.blf release\samples\

# 4. 创建配置文件
# （手动创建或复制）
```

## 📋 打包清单

### 必需文件
- [x] `bin\canview.exe` - 主程序
- [x] `config\signal_library\` - 信号库存储目录
- [x] `config\default_config.json` - 默认配置
- [x] `start.bat` - 启动脚本
- [x] `README.txt` - 使用说明

### 可选文件
- [ ] `samples\sample.dbc` - DBC 示例
- [ ] `samples\sample.blf` - BLF 示例
- [ ] `docs\*.md` - 文档
- [ ] `assets\*` - 资源文件

## 🔧 编译选项

### Debug 模式（开发）
```powershell
cargo build -p view
```
- ✅ 显示控制台窗口
- ✅ 包含调试信息
- ✅ 文件较大

### Release 模式（发布）
```powershell
cargo build --release -p view
```
- ✅ 隐藏控制台窗口
- ✅ 优化性能
- ✅ 文件较小（strip = true）
- ✅ 无调试信息

## 📦 分发方式

### 方式 1: ZIP 压缩包
```powershell
# 使用打包脚本自动生成
.\package.ps1

# 分发 CANVIEW-v1.0.0.zip
```

### 方式 2: 安装程序
```powershell
# 使用 NSIS 或 Inno Setup 创建安装程序
# （需要额外配置）
```

### 方式 3: 便携版
```
# 直接分发整个文件夹
CANVIEW-v1.0.0/
```

## ✅ 验证清单

打包完成后，请验证：

- [ ] 双击 `start.bat` 能正常启动
- [ ] 没有黑色控制台窗口出现
- [ ] `config\signal_library\` 目录存在
- [ ] 可以创建库、添加版本、添加通道
- [ ] 文件自动复制到 `config\signal_library\`
- [ ] 重启后配置恢复正常
- [ ] 整个文件夹可以复制到其他位置使用

## 🐛 常见问题

### Q: 运行时仍然显示控制台窗口？

**A**: 确保使用 Release 模式编译：
```powershell
cargo build --release -p view
```

### Q: 找不到配置文件？

**A**: 确保 `config` 目录与 `bin\canview.exe` 在同一父目录下：
```
CANVIEW/
├── bin\canview.exe
└── config\
```

### Q: 信号库文件没有自动复制？

**A**: 检查：
1. `config\signal_library\` 目录是否存在
2. 是否有写入权限
3. 查看控制台输出（Debug 模式）

### Q: 如何查看日志？

**A**: 
- Debug 模式：控制台直接显示
- Release 模式：需要重定向输出
  ```powershell
  bin\canview.exe > log.txt 2>&1
  ```

## 📝 更新日志

### v1.0.0 (2026-01-25)
- ✅ 隐藏控制台窗口
- ✅ 完整的打包结构
- ✅ 信号库本地存储
- ✅ 自动配置保存/加载

---

**创建日期**: 2026-01-25  
**状态**: ✅ 完成  
**测试**: ✅ 通过
