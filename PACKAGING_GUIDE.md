# CANVIEW 打包和发布指南

## 概述

本文档说明如何将 CANVIEW 应用程序打包成一个完整的发行包，包含可执行文件、配置文件、文档和示例文件。

## 打包方法

### 方法 1: 使用批处理脚本（推荐）

1. 双击运行 `package.bat`
2. 等待打包完成
3. 在 `release-package` 目录下找到打包好的文件夹和压缩包

### 方法 2: 使用 PowerShell 脚本

```powershell
# 使用默认版本号 1.0.0
.\package.ps1

# 指定版本号
.\package.ps1 -Version "1.2.3"

# 指定输出目录
.\package.ps1 -Version "1.2.3" -OutputDir ".\dist"
```

## 发布包结构

打包后的目录结构如下：

```
CANVIEW-v1.0.0/
├── bin/                      # 可执行文件目录
│   └── canview.exe          # 主程序（从 view.exe 重命名）
│
├── config/                   # 配置文件目录
│   ├── default_config.json  # 默认配置模板
│   └── example_config.json  # 配置示例（如果存在）
│
├── samples/                  # 示例文件目录
│   ├── sample.dbc           # DBC 示例文件
│   └── sample.blf           # BLF 示例文件
│
├── assets/                   # 资源文件目录
│   └── (图标、图片等资源)
│
├── docs/                     # 文档目录
│   ├── README.md            # 使用说明
│   ├── BUILD.md             # 编译说明
│   └── ADD_CHANNEL_CRASH_FIX.md  # 修复说明
│
├── start.bat                # 启动脚本
└── README.txt               # 发布说明
```

## 配置文件说明

### 配置文件位置优先级

程序启动时会按以下顺序查找配置文件：

1. **程序根目录**: `multi_channel_config.json`（最高优先级）
2. **config 目录**: `config/default_config.json`（默认配置）

### 配置文件格式

```json
{
  "libraries": [
    {
      "id": "lib_xxx",
      "name": "库名称",
      "channel_type": "CAN",
      "versions": [
        {
          "name": "版本名",
          "path": "数据库文件路径",
          "date": "2026-01-25",
          "description": "版本描述",
          "channel_databases": [
            {
              "channel_type": "CAN",
              "channel_id": 1,
              "channel_name": "通道名称",
              "database_path": "C:\\path\\to\\database.dbc"
            }
          ]
        }
      ]
    }
  ],
  "mappings": [],
  "active_library_id": null,
  "active_version_name": null
}
```

## 用户使用指南

### 安装步骤

1. 解压 `CANVIEW-v1.0.0.zip` 到任意目录
2. （可选）复制 `config\example_config.json` 到根目录并重命名为 `multi_channel_config.json`
3. 根据需要编辑配置文件

### 启动程序

**方法 1**: 双击 `start.bat`

**方法 2**: 直接运行 `bin\canview.exe`

**方法 3**: 在命令行中运行：
```cmd
cd CANVIEW-v1.0.0
bin\canview.exe
```

### 配置通道

1. 启动程序后，切换到 "Library" 视图
2. 点击 "Add Library" 创建新库
3. 选择库后，点击 "Add Version" 添加版本
4. 在版本中点击 "Add Channel" 添加通道配置
5. 选择对应的 DBC 或 LDF 数据库文件

## 打包脚本工作流程

打包脚本执行以下步骤：

1. **编译**: 使用 `cargo build --release -p view` 编译 Release 版本
2. **创建目录**: 创建发布包的目录结构
3. **复制文件**: 
   - 可执行文件 → `bin/canview.exe`
   - 配置文件 → `config/`
   - 示例文件 → `samples/`
   - 资源文件 → `assets/`
   - 文档文件 → `docs/`
4. **生成脚本**: 创建 `start.bat` 启动脚本
5. **生成文档**: 创建 `README.txt` 发布说明
6. **压缩打包**: 创建 `.zip` 压缩包

## 自定义打包

如果需要自定义打包内容，可以编辑 `package.ps1` 脚本：

- **添加文件**: 在相应步骤中添加 `Copy-Item` 命令
- **修改目录结构**: 调整 `New-Item` 命令
- **更改版本号**: 使用 `-Version` 参数或修改默认值

## 发布检查清单

在发布前，请确认：

- [ ] 程序编译成功，无错误
- [ ] 所有必要的配置文件已包含
- [ ] 示例文件可以正常加载
- [ ] 文档已更新到最新版本
- [ ] 启动脚本可以正常运行
- [ ] 压缩包可以正常解压
- [ ] 在干净的环境中测试过

## 版本管理

建议的版本号格式：`主版本.次版本.修订号`

- **主版本**: 重大功能变更或不兼容的 API 修改
- **次版本**: 新功能添加，向后兼容
- **修订号**: Bug 修复和小改进

示例：
- `1.0.0` - 首次正式发布
- `1.1.0` - 添加新功能
- `1.1.1` - Bug 修复

## 故障排除

### 打包失败

**问题**: 编译失败
- **解决**: 检查 Rust 工具链是否正确安装，运行 `cargo clean` 后重试

**问题**: 文件复制失败
- **解决**: 确保源文件存在，检查文件路径是否正确

**问题**: 压缩失败
- **解决**: 确保有足够的磁盘空间，检查是否有权限问题

### 程序运行问题

**问题**: 双击 start.bat 后窗口闪退
- **解决**: 在命令行中运行 `bin\canview.exe` 查看错误信息

**问题**: 找不到配置文件
- **解决**: 确保配置文件在正确的位置，检查文件名是否正确

## 技术细节

### 可执行文件重命名

打包脚本会将 `view.exe` 重命名为 `canview.exe`，这样更符合项目名称。

### 配置文件编码

所有配置文件使用 UTF-8 编码，确保支持中文等多语言字符。

### 启动脚本

`start.bat` 脚本会：
1. 切换到程序所在目录
2. 运行 `bin\canview.exe`
3. 程序退出后暂停，方便查看错误信息

## 更新日志

### v1.0.0 (2026-01-25)
- 初始版本
- 支持 BLF 文件解析
- 支持 DBC/LDF 数据库
- 多通道配置管理
- 修复添加通道时的崩溃问题

---

**注意**: 本文档会随着项目更新而更新，请以最新版本为准。
