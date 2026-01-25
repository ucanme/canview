# CANVIEW 安装包制作指南

## 📦 概述

本指南介绍如何为 CANVIEW 创建专业的 Windows 安装程序。

## 🛠️ 准备工作

### 1. 安装 Inno Setup

**下载地址**: https://jrsoftware.org/isdl.php

**推荐版本**: Inno Setup 6.x

**安装步骤**:
1. 下载 `innosetup-6.x.x.exe`
2. 运行安装程序
3. 默认安装路径: `C:\Program Files (x86)\Inno Setup 6\`
4. 确保勾选"添加到 PATH"选项

### 2. 验证安装

```powershell
# 检查 Inno Setup 是否安装成功
Test-Path "C:\Program Files (x86)\Inno Setup 6\ISCC.exe"
# 应该返回 True
```

## 🚀 快速开始

### 方法 1: 使用自动化脚本（推荐）

```powershell
# 构建安装程序
.\build-installer.ps1

# 或指定版本号
.\build-installer.ps1 -Version "1.0.0"

# 或指定 Inno Setup 路径
.\build-installer.ps1 -InnoSetupPath "D:\Tools\Inno Setup 6\ISCC.exe"
```

**输出**: `installer-output\CANVIEW-Setup-v1.0.0.exe`

### 方法 2: 手动构建

```powershell
# 1. 编译程序
cargo build --release -p view

# 2. 准备文件
# 确保以下文件/目录存在：
# - target\release\view.exe
# - config\
# - sample.dbc (可选)
# - sample.blf (可选)
# - README.md (可选)

# 3. 编译安装脚本
& "C:\Program Files (x86)\Inno Setup 6\ISCC.exe" installer.iss
```

## 📋 安装包功能

### 包含的功能

1. ✅ **图形化安装向导**
   - 欢迎页面
   - 许可协议
   - 安装路径选择
   - 组件选择
   - 安装进度
   - 完成页面

2. ✅ **自动化配置**
   - 创建开始菜单快捷方式
   - 可选桌面图标
   - 可选快速启动图标
   - 自动创建配置目录
   - 设置目录权限

3. ✅ **智能升级**
   - 自动检测旧版本
   - 静默卸载旧版本
   - 保留用户配置

4. ✅ **完整卸载**
   - 卸载程序
   - 清理注册表
   - 可选保留配置文件

5. ✅ **多语言支持**
   - 简体中文
   - English

## 📂 安装包结构

### 安装后的目录结构

```
C:\Program Files\CANVIEW\
├── bin\
│   └── canview.exe          # 主程序
├── config\
│   ├── signal_library\      # 信号库存储（用户可写）
│   └── default_config.json  # 默认配置
├── samples\
│   ├── sample.dbc           # 示例文件
│   └── sample.blf
├── docs\
│   ├── README.md
│   └── *.md                 # 其他文档
└── assets\                  # 资源文件
```

### 用户数据目录

```
C:\Users\{用户名}\AppData\Roaming\CANVIEW\
├── multi_channel_config.json  # 用户配置
└── logs\                       # 日志文件
```

## ⚙️ 自定义配置

### 修改安装脚本 (installer.iss)

#### 1. 修改应用信息

```ini
#define MyAppName "CANVIEW"
#define MyAppVersion "1.0.0"
#define MyAppPublisher "Your Company"
#define MyAppURL "https://github.com/yourusername/canview"
```

#### 2. 修改安装路径

```ini
DefaultDirName={autopf}\{#MyAppName}  ; Program Files
; 或
DefaultDirName={userdocs}\{#MyAppName}  ; 文档目录
```

#### 3. 添加文件

```ini
[Files]
Source: "your-file.txt"; DestDir: "{app}"; Flags: ignoreversion
```

#### 4. 添加注册表项

```ini
[Registry]
Root: HKCU; Subkey: "Software\{#MyAppName}"; ValueType: string; ValueName: "InstallPath"; ValueData: "{app}"
```

#### 5. 添加环境变量

```ini
[Registry]
Root: HKCU; Subkey: "Environment"; ValueType: expandsz; ValueName: "Path"; ValueData: "{olddata};{app}\bin"
```

## 🎨 自定义外观

### 1. 修改图标

```ini
SetupIconFile=assets\ico\canview.ico
UninstallDisplayIcon={app}\bin\canview.exe
```

### 2. 添加安装向导图片

```ini
WizardImageFile=assets\installer\wizard.bmp
WizardSmallImageFile=assets\installer\wizard-small.bmp
```

### 3. 修改主题

```ini
WizardStyle=modern  ; 现代风格
; 或
WizardStyle=classic  ; 经典风格
```

## 🔧 高级功能

### 1. 静默安装

```powershell
# 完全静默安装
CANVIEW-Setup-v1.0.0.exe /VERYSILENT /NORESTART

# 静默安装但显示进度
CANVIEW-Setup-v1.0.0.exe /SILENT /NORESTART

# 指定安装目录
CANVIEW-Setup-v1.0.0.exe /DIR="D:\CANVIEW" /SILENT
```

### 2. 命令行参数

```powershell
# 创建桌面图标
CANVIEW-Setup-v1.0.0.exe /TASKS="desktopicon"

# 不创建任何图标
CANVIEW-Setup-v1.0.0.exe /TASKS=""

# 指定语言
CANVIEW-Setup-v1.0.0.exe /LANG=chinesesimplified
```

### 3. 日志记录

```powershell
# 生成安装日志
CANVIEW-Setup-v1.0.0.exe /LOG="install.log"
```

## 📊 构建流程

### 完整构建流程

```
1. 编译程序
   ↓
2. 准备文件
   ↓
3. 更新版本号
   ↓
4. 编译安装脚本
   ↓
5. 生成安装程序
   ↓
6. 测试安装
   ↓
7. 分发
```

### 自动化脚本流程

```powershell
# build-installer.ps1 执行流程：
1. 检查 Inno Setup
2. 编译 Release 版本
3. 准备配置文件
4. 更新版本号
5. 构建安装程序
6. 显示结果
```

## ✅ 测试清单

### 安装测试

- [ ] 全新安装成功
- [ ] 升级安装成功
- [ ] 自定义路径安装成功
- [ ] 桌面图标创建成功
- [ ] 开始菜单快捷方式创建成功
- [ ] 程序能正常启动
- [ ] 配置目录权限正确
- [ ] 无控制台窗口

### 卸载测试

- [ ] 卸载程序运行正常
- [ ] 文件完全删除
- [ ] 注册表清理干净
- [ ] 快捷方式删除
- [ ] 可选保留配置文件

### 兼容性测试

- [ ] Windows 10 (x64)
- [ ] Windows 11 (x64)
- [ ] 标准用户权限安装
- [ ] 管理员权限安装
- [ ] 中文系统
- [ ] 英文系统

## 🐛 常见问题

### Q: 提示"未找到 Inno Setup"？

**A**: 
1. 确认已安装 Inno Setup
2. 检查安装路径是否正确
3. 使用 `-InnoSetupPath` 参数指定路径

### Q: 安装程序无法运行？

**A**: 
1. 检查是否被杀毒软件拦截
2. 右键 → 属性 → 解除锁定
3. 以管理员身份运行

### Q: 升级时配置丢失？

**A**: 
安装脚本会自动保留用户配置，确保：
1. 配置文件在 `%APPDATA%\CANVIEW\`
2. 不要手动删除旧版本

### Q: 如何创建便携版？

**A**: 
使用 `package.ps1` 而不是 `build-installer.ps1`：
```powershell
.\package.ps1 -Version "1.0.0"
```

## 📝 版本发布流程

### 1. 准备发布

```powershell
# 更新版本号
# 编辑 installer.iss 中的 MyAppVersion

# 更新文档
# 编辑 CHANGELOG.md
```

### 2. 构建安装包

```powershell
.\build-installer.ps1 -Version "1.0.0"
```

### 3. 测试

```powershell
# 在干净的虚拟机中测试安装
```

### 4. 发布

```powershell
# 上传到 GitHub Releases
# 或其他分发渠道
```

## 📚 相关资源

- **Inno Setup 官网**: https://jrsoftware.org/isinfo.php
- **Inno Setup 文档**: https://jrsoftware.org/ishelp/
- **示例脚本**: https://jrsoftware.org/ishelp/index.php?topic=samples

## 🎯 最佳实践

1. **版本号管理**: 使用语义化版本 (Semantic Versioning)
2. **代码签名**: 为安装程序添加数字签名
3. **自动更新**: 集成自动更新检查功能
4. **错误处理**: 添加详细的错误日志
5. **用户反馈**: 收集安装过程中的问题

---

**创建日期**: 2026-01-25  
**状态**: ✅ 完成  
**工具**: Inno Setup 6.x
