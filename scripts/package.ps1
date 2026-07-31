# can-viewer 打包脚本
# 创建一个包含可执行文件、配置文件和文档的完整发行包

param(
    [string]$Version = "1.0.0",
    [string]$OutputDir = ".\release-package"
)

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "can-viewer 打包脚本 v$Version" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# 1. 编译 Release 版本
Write-Host "📦 步骤 1: 编译 Release 版本..." -ForegroundColor Green
cargo build --release -p viewer
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ 编译失败！" -ForegroundColor Red
    exit 1
}
Write-Host "✅ 编译成功！" -ForegroundColor Green
Write-Host ""

# 2. 创建发布目录结构
Write-Host "📁 步骤 2: 创建发布目录..." -ForegroundColor Green
$PackageName = "can-viewer-v$Version"
$PackageDir = Join-Path $OutputDir $PackageName

# 清理旧的发布目录
if (Test-Path $PackageDir) {
    Remove-Item -Path $PackageDir -Recurse -Force
}

# 创建目录结构
New-Item -ItemType Directory -Path $PackageDir -Force | Out-Null
New-Item -ItemType Directory -Path "$PackageDir\bin" -Force | Out-Null
New-Item -ItemType Directory -Path "$PackageDir\config" -Force | Out-Null
New-Item -ItemType Directory -Path "$PackageDir\config\signal_library" -Force | Out-Null
New-Item -ItemType Directory -Path "$PackageDir\docs" -Force | Out-Null
New-Item -ItemType Directory -Path "$PackageDir\samples" -Force | Out-Null
New-Item -ItemType Directory -Path "$PackageDir\assets" -Force | Out-Null

Write-Host "✅ 目录结构创建完成！" -ForegroundColor Green
Write-Host ""

# 3. 复制可执行文件
Write-Host "📋 步骤 3: 复制可执行文件..." -ForegroundColor Green
Copy-Item -Path ".\target\release\viewer.exe" -Destination "$PackageDir\bin\can-viewer.exe"
Write-Host "✅ 可执行文件已复制到 bin\can-viewer.exe" -ForegroundColor Green
Write-Host ""

# 4. 复制配置文件
Write-Host "📋 步骤 4: 复制配置文件..." -ForegroundColor Green

# 创建默认配置文件
$defaultConfig = @"
{
  "libraries": [],
  "mappings": [],
  "active_library_id": null,
  "active_version_name": null
}
"@
$defaultConfig | Out-File -FilePath "$PackageDir\config\default_config.json" -Encoding UTF8

# 如果存在用户配置，也复制一份作为示例
if (Test-Path ".\multi_channel_config.json") {
    Copy-Item -Path ".\multi_channel_config.json" -Destination "$PackageDir\config\example_config.json"
}

# 创建信号库存储说明文件
$signalLibraryReadme = @"
# 信号库本地存储目录

此目录用于存储信号库的数据库文件。

## 目录结构

```
signal_library/
└── {库名}/
    └── {版本}/
        └── database.{dbc|ldf}
```

## 示例

```
signal_library/
├── BMW_PTCAN/
│   ├── v1.0/
│   │   └── database.dbc
│   └── v2.0/
│       └── database.dbc
└── Ford_LIN/
    └── v1.5/
        └── database.ldf
```

## 说明

- 当您在软件中添加信号库和版本时，数据库文件会自动复制到此目录
- 配置文件中保存的是此目录下的路径，确保软件可移植性
- 您也可以手动将数据库文件放入相应的目录结构中

## 注意事项

- 请勿手动删除此目录下的文件，除非您确定不再需要
- 备份软件时，请同时备份此目录

---
更新时间: $(Get-Date -Format "yyyy-MM-dd HH:mm:ss")
"@
$signalLibraryReadme | Out-File -FilePath "$PackageDir\config\signal_library\README.txt" -Encoding UTF8

Write-Host "✅ 配置文件已创建" -ForegroundColor Green
Write-Host ""

# 5. 复制示例文件
Write-Host "📋 步骤 5: 复制示例文件..." -ForegroundColor Green
if (Test-Path ".\sample.dbc") {
    Copy-Item -Path ".\sample.dbc" -Destination "$PackageDir\samples\sample.dbc"
}
if (Test-Path ".\sample.blf") {
    Copy-Item -Path ".\sample.blf" -Destination "$PackageDir\samples\sample.blf"
}
Write-Host "✅ 示例文件已复制" -ForegroundColor Green
Write-Host ""

# 6. 复制资源文件
Write-Host "📋 步骤 6: 复制资源文件..." -ForegroundColor Green
if (Test-Path ".\assets") {
    Copy-Item -Path ".\assets\*" -Destination "$PackageDir\assets\" -Recurse -Force
}
Write-Host "✅ 资源文件已复制" -ForegroundColor Green
Write-Host ""

# 7. 复制文档
Write-Host "📋 步骤 7: 复制文档..." -ForegroundColor Green
Copy-Item -Path ".\README.md" -Destination "$PackageDir\docs\README.md" -ErrorAction SilentlyContinue
Copy-Item -Path ".\BUILD.md" -Destination "$PackageDir\docs\BUILD.md" -ErrorAction SilentlyContinue
Copy-Item -Path ".\ADD_CHANNEL_CRASH_FIX.md" -Destination "$PackageDir\docs\ADD_CHANNEL_CRASH_FIX.md" -ErrorAction SilentlyContinue

Write-Host "✅ 文档已复制" -ForegroundColor Green
Write-Host ""

# 8. 创建启动脚本
Write-Host "📋 步骤 8: 创建启动脚本..." -ForegroundColor Green

$launchScript = @"
@echo off
REM can-viewer 启动脚本
echo Starting can-viewer...
cd /d "%~dp0"
bin\can-viewer.exe
pause
"@
$launchScript | Out-File -FilePath "$PackageDir\start.bat" -Encoding ASCII

Write-Host "✅ 启动脚本已创建" -ForegroundColor Green
Write-Host ""

# 9. 创建 README
Write-Host "📋 步骤 9: 创建发布说明..." -ForegroundColor Green

$releaseReadme = @"
# can-viewer v$Version

## 目录结构

```
can-viewer-v$Version/
├── bin/              # 可执行文件
│   └── can-viewer.exe   # 主程序
├── config/           # 配置文件目录
│   ├── signal_library/        # 信号库本地存储
│   │   └── README.txt         # 存储说明
│   ├── default_config.json    # 默认配置
│   └── example_config.json    # 配置示例
├── samples/          # 示例文件
│   ├── sample.dbc    # DBC 示例
│   └── sample.blf    # BLF 示例
├── assets/           # 资源文件（图标等）
├── docs/             # 文档
│   ├── README.md     # 使用说明
│   └── BUILD.md      # 编译说明
├── start.bat         # 启动脚本
└── README.txt        # 本文件
```

## 快速开始

1. 双击 `start.bat` 启动程序
2. 或者直接运行 `bin\can-viewer.exe`

## 配置文件

程序会在以下位置查找配置文件：
1. 当前目录下的 `multi_channel_config.json`
2. `config\default_config.json`

您可以复制 `config\example_config.json` 到程序根目录并重命名为 `multi_channel_config.json` 来自定义配置。

## 信号库存储

程序支持信号库本地存储功能：

- **存储位置**: `config\signal_library\`
- **自动管理**: 添加信号库时，数据库文件会自动复制到此目录
- **目录结构**: `{库名}\{版本}\database.{dbc|ldf}`
- **可移植性**: 整个软件目录可以直接复制到其他位置使用

### 使用方法

1. 在软件中切换到"Library"视图
2. 点击"+ Add Library"创建新库
3. 点击"+ Add Version"添加版本
4. 点击"+ Add Channel"配置通道并选择数据库文件
5. 文件会自动复制到 `config\signal_library\{库名}\{版本}\` 目录

## 功能特性

- BLF 文件解析和查看
- DBC/LDF 数据库支持
- 多通道配置管理
- 信号解码和显示
- 图表分析（开发中）

## 系统要求

- Windows 10 或更高版本
- 64 位操作系统

## 技术支持

如有问题，请查看 `docs` 目录下的文档或联系开发团队。

---
构建时间: $(Get-Date -Format "yyyy-MM-dd HH:mm:ss")
版本: $Version
"@
$releaseReadme | Out-File -FilePath "$PackageDir\README.txt" -Encoding UTF8

Write-Host "✅ 发布说明已创建" -ForegroundColor Green
Write-Host ""

# 10. 创建压缩包
Write-Host "📦 步骤 10: 创建压缩包..." -ForegroundColor Green
$ZipPath = Join-Path $OutputDir "$PackageName.zip"
if (Test-Path $ZipPath) {
    Remove-Item -Path $ZipPath -Force
}

Compress-Archive -Path $PackageDir -DestinationPath $ZipPath -CompressionLevel Optimal
Write-Host "✅ 压缩包已创建: $ZipPath" -ForegroundColor Green
Write-Host ""

# 完成
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "✅ 打包完成！" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "发布包位置:" -ForegroundColor Yellow
Write-Host "  文件夹: $PackageDir" -ForegroundColor White
Write-Host "  压缩包: $ZipPath" -ForegroundColor White
Write-Host ""
Write-Host "您可以将压缩包分发给用户，解压后即可使用。" -ForegroundColor Cyan
