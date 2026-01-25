# GitHub Actions 自动化打包指南

## 📋 概述

本项目使用 GitHub Actions 实现跨平台自动化打包和发布。

## 🔄 工作流说明

### 1. CI 工作流 (ci.yml)

**触发条件**:
- 推送到 main/master/develop 分支
- Pull Request 到 main/master/develop 分支

**功能**:
- ✅ 代码格式检查 (rustfmt)
- ✅ 代码质量检查 (clippy)
- ✅ 运行测试
- ✅ 跨平台构建测试

**运行平台**:
- Ubuntu (Linux)
- Windows
- macOS

### 2. Release 工作流 (release.yml)

**触发条件**:
- 推送版本标签 (如 `v1.0.0`)
- 手动触发

**功能**:
- ✅ Windows 打包 (.exe, .zip)
- ✅ macOS 打包 (.dmg, .app, .tar.gz)
- ✅ Linux 打包 (.deb, .rpm, .tar.gz, .AppImage)
- ✅ 自动创建 GitHub Release
- ✅ 上传所有安装包

## 🚀 使用方法

### 方式 1: 推送标签发布（推荐）

```bash
# 1. 更新版本号
# 编辑相关文件中的版本号

# 2. 提交更改
git add .
git commit -m "Release v1.0.0"

# 3. 创建并推送标签
git tag v1.0.0
git push origin v1.0.0

# 4. GitHub Actions 自动开始构建
# 访问 https://github.com/你的用户名/canview/actions 查看进度
```

### 方式 2: 手动触发

1. 访问 GitHub 仓库
2. 点击 "Actions" 标签
3. 选择 "Release Build" 工作流
4. 点击 "Run workflow"
5. 选择分支并运行

## 📦 构建产物

### Windows

| 文件 | 说明 |
|------|------|
| `CANVIEW-Setup-v1.0.0.exe` | 安装程序（推荐） |
| `CANVIEW-v1.0.0.zip` | 便携版 |

### macOS

| 文件 | 说明 |
|------|------|
| `CANVIEW-v1.0.0.dmg` | DMG 镜像（推荐） |
| `CANVIEW-v1.0.0-macos.tar.gz` | 压缩包 |

### Linux

| 文件 | 说明 |
|------|------|
| `canview_1.0.0_amd64.deb` | Debian/Ubuntu 包 |
| `canview-1.0.0-1.*.rpm` | Fedora/RHEL 包 |
| `canview-v1.0.0-x86_64.AppImage` | AppImage（推荐） |
| `canview-v1.0.0-linux-amd64.tar.gz` | 通用包 |

## 🔍 工作流详解

### Windows 构建流程

```yaml
1. 检出代码
2. 安装 Rust 工具链
3. 缓存依赖
4. 编译 Release 版本
5. 创建 ZIP 包
6. 安装 Inno Setup
7. 创建安装程序
8. 上传构建产物
```

### macOS 构建流程

```yaml
1. 检出代码
2. 安装 Rust 工具链
3. 缓存依赖
4. 安装 create-dmg
5. 编译 Release 版本
6. 创建 .app 包
7. 创建 DMG 镜像
8. 创建 tar.gz 包
9. 上传构建产物
```

### Linux 构建流程

```yaml
1. 检出代码
2. 安装 Rust 工具链
3. 缓存依赖
4. 安装系统依赖
5. 安装 AppImage 工具
6. 编译 Release 版本
7. 创建 .deb 包
8. 创建 .rpm 包
9. 创建 tar.gz 包
10. 创建 AppImage
11. 上传构建产物
```

## ⚙️ 配置说明

### 环境变量

```yaml
env:
  CARGO_TERM_COLOR: always  # Cargo 输出彩色
```

### 缓存配置

为了加速构建，工作流缓存了：
- Cargo registry
- Cargo index
- 构建目标文件

### 版本号提取

```yaml
# 从 Git 标签提取版本号
if [[ $GITHUB_REF == refs/tags/* ]]; then
  VERSION=${GITHUB_REF#refs/tags/v}
else
  VERSION="dev"
fi
```

## 🛠️ 自定义配置

### 修改触发条件

```yaml
# 只在特定分支触发
on:
  push:
    branches: [ main ]
    tags: [ 'v*' ]

# 添加定时触发
on:
  schedule:
    - cron: '0 0 * * 0'  # 每周日午夜
```

### 添加构建步骤

```yaml
- name: 自定义步骤
  run: |
    echo "执行自定义命令"
    # 你的命令
```

### 修改 Release 说明

编辑 `release.yml` 中的 `body` 部分：

```yaml
body: |
  ## 新版本发布
  
  ### 新功能
  - 功能 1
  - 功能 2
  
  ### 修复
  - 修复 1
  - 修复 2
```

## 📊 构建状态徽章

在 README.md 中添加徽章：

```markdown
![CI](https://github.com/你的用户名/canview/workflows/CI/badge.svg)
![Release](https://github.com/你的用户名/canview/workflows/Release%20Build/badge.svg)
```

## 🔐 Secrets 配置

### GITHUB_TOKEN

GitHub 自动提供，无需配置。用于：
- 创建 Release
- 上传资产

### 可选 Secrets

如果需要代码签名：

```yaml
# Windows 代码签名
WINDOWS_CERTIFICATE: ${{ secrets.WINDOWS_CERTIFICATE }}
WINDOWS_CERTIFICATE_PASSWORD: ${{ secrets.WINDOWS_CERTIFICATE_PASSWORD }}

# macOS 代码签名
MACOS_CERTIFICATE: ${{ secrets.MACOS_CERTIFICATE }}
MACOS_CERTIFICATE_PASSWORD: ${{ secrets.MACOS_CERTIFICATE_PASSWORD }}
```

配置方法：
1. 访问仓库 Settings → Secrets and variables → Actions
2. 点击 "New repository secret"
3. 添加 Secret

## 🐛 故障排除

### 问题 1: 构建失败

**检查**:
1. 查看 Actions 日志
2. 确认依赖是否正确
3. 本地测试构建脚本

### 问题 2: 上传失败

**解决**:
```yaml
# 添加 continue-on-error
- name: Upload artifact
  uses: actions/upload-artifact@v3
  with:
    name: my-artifact
    path: path/to/file
  continue-on-error: true
```

### 问题 3: 缓存问题

**清理缓存**:
1. 访问 Actions 页面
2. 点击 "Caches"
3. 删除相关缓存

### 问题 4: 权限错误

**解决**:
```yaml
# 添加权限
permissions:
  contents: write  # 允许创建 Release
```

## 📈 优化建议

### 1. 并行构建

```yaml
strategy:
  matrix:
    os: [ubuntu-latest, windows-latest, macos-latest]
  max-parallel: 3  # 最多并行 3 个
```

### 2. 条件执行

```yaml
# 只在主分支构建
if: github.ref == 'refs/heads/main'

# 只在标签时发布
if: startsWith(github.ref, 'refs/tags/')
```

### 3. 超时设置

```yaml
jobs:
  build:
    timeout-minutes: 60  # 60 分钟超时
```

## 📝 最佳实践

### 1. 版本号管理

```bash
# 使用语义化版本
v1.0.0  # 主版本.次版本.修订号
v1.1.0  # 新功能
v1.1.1  # Bug 修复
```

### 2. 发布流程

```bash
# 1. 开发完成
git checkout develop
git commit -m "Feature: xxx"

# 2. 合并到主分支
git checkout main
git merge develop

# 3. 打标签
git tag v1.0.0

# 4. 推送
git push origin main --tags
```

### 3. 测试策略

```yaml
# 先测试，再构建
jobs:
  test:
    runs-on: ubuntu-latest
    steps: [...]
  
  build:
    needs: test  # 依赖测试通过
    runs-on: ubuntu-latest
    steps: [...]
```

## 🎯 完整示例

### 发布新版本

```bash
# 1. 确保代码最新
git pull origin main

# 2. 更新版本号
# 编辑 Cargo.toml, installer.iss 等

# 3. 提交更改
git add .
git commit -m "chore: bump version to 1.0.0"

# 4. 创建标签
git tag -a v1.0.0 -m "Release version 1.0.0"

# 5. 推送
git push origin main
git push origin v1.0.0

# 6. 等待 GitHub Actions 完成
# 访问 https://github.com/你的用户名/canview/releases
```

## 📚 相关资源

- **GitHub Actions 文档**: https://docs.github.com/en/actions
- **actions-rs**: https://github.com/actions-rs
- **工作流语法**: https://docs.github.com/en/actions/reference/workflow-syntax-for-github-actions

## ✅ 检查清单

发布前检查：

- [ ] 代码已提交
- [ ] 版本号已更新
- [ ] CHANGELOG 已更新
- [ ] 本地测试通过
- [ ] CI 测试通过
- [ ] 标签已创建
- [ ] 标签已推送

发布后验证：

- [ ] Actions 构建成功
- [ ] Release 已创建
- [ ] 所有平台的安装包已上传
- [ ] 下载并测试安装包
- [ ] 更新文档

---

**创建日期**: 2026-01-25  
**状态**: ✅ 完成  
**自动化程度**: 100%
