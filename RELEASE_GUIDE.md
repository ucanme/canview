# 跨平台发布指南

本文档说明如何在 Windows、macOS 和 Linux 上构建和发布 CANVIEW。

## 目录

- [本地构建](#本地构建)
- [GitHub Actions 自动构建](#github-actions-自动构建)
- [发布流程](#发布流程)
- [平台特定说明](#平台特定说明)

## 本地构建

### Windows

#### 前提条件
- Rust 工具链
- PowerShell 5.0+

#### 构建步骤
```powershell
# 方法 1: 使用批处理脚本
.\package.bat

# 方法 2: 使用 PowerShell 脚本
.\package.ps1 -Version "1.0.0"
```

#### 输出
- 文件夹: `release-package\CANVIEW-v1.0.0\`
- 压缩包: `release-package\CANVIEW-v1.0.0.zip`

### macOS

#### 前提条件
- Rust 工具链
- Xcode Command Line Tools

#### 构建步骤
```bash
# 赋予执行权限
chmod +x package.sh

# 运行打包脚本
./package.sh 1.0.0
```

#### 输出
- 文件夹: `release-package/CANVIEW-v1.0.0-macos/`
- 压缩包: `release-package/CANVIEW-v1.0.0-macos.zip`

### Linux

#### 前提条件
- Rust 工具链
- 开发库：
  ```bash
  # Ubuntu/Debian
  sudo apt-get install libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libssl-dev
  
  # Fedora
  sudo dnf install libxcb-devel libxkbcommon-devel openssl-devel
  
  # Arch Linux
  sudo pacman -S libxcb libxkbcommon openssl
  ```

#### 构建步骤
```bash
# 赋予执行权限
chmod +x package.sh

# 运行打包脚本
./package.sh 1.0.0
```

#### 输出
- 文件夹: `release-package/CANVIEW-v1.0.0-linux/`
- 压缩包: `release-package/CANVIEW-v1.0.0-linux.tar.gz`

## GitHub Actions 自动构建

### 触发方式

#### 方法 1: 推送标签（推荐）

```bash
# 1. 更新版本号和 CHANGELOG
vim CHANGELOG.md
vim Cargo.toml

# 2. 提交更改
git add .
git commit -m "Release v1.0.0"

# 3. 创建并推送标签
git tag v1.0.0
git push origin v1.0.0
```

GitHub Actions 将自动：
1. 在 Linux、macOS、Windows 上构建
2. 创建 GitHub Release
3. 上传所有平台的压缩包

#### 方法 2: 手动触发

1. 访问 GitHub 仓库的 Actions 页面
2. 选择 "Build and Release" 工作流
3. 点击 "Run workflow"
4. 输入版本号（例如：1.0.0）
5. 点击 "Run workflow" 按钮

### 工作流程

```
触发 (标签推送或手动)
    ↓
并行构建三个平台
    ├─ Linux (Ubuntu)
    ├─ macOS
    └─ Windows
    ↓
上传构建产物
    ↓
创建 GitHub Release (仅标签触发)
    ↓
上传发布资产
```

### 构建矩阵

| 平台 | 运行环境 | 输出格式 | 文件名示例 |
|------|----------|----------|------------|
| Linux | ubuntu-latest | tar.gz | CANVIEW-v1.0.0-linux.tar.gz |
| macOS | macos-latest | zip | CANVIEW-v1.0.0-macos.zip |
| Windows | windows-latest | zip | CANVIEW-v1.0.0.zip |

## 发布流程

### 完整发布检查清单

- [ ] **代码准备**
  - [ ] 所有功能已完成并测试
  - [ ] 所有测试通过
  - [ ] 代码已审查
  - [ ] 无已知的严重 bug

- [ ] **文档更新**
  - [ ] 更新 CHANGELOG.md
  - [ ] 更新 README.md（如有必要）
  - [ ] 更新版本号相关文档

- [ ] **版本号更新**
  - [ ] 更新 `Cargo.toml` 中的版本号
  - [ ] 更新 `src/view/Cargo.toml` 中的版本号
  - [ ] 确保版本号符合语义化版本规范

- [ ] **本地测试**
  - [ ] 在本地构建并测试
  - [ ] 验证打包脚本正常工作
  - [ ] 测试解压后的程序可以运行

- [ ] **提交和标签**
  - [ ] 提交所有更改
  - [ ] 创建版本标签
  - [ ] 推送到 GitHub

- [ ] **GitHub Actions**
  - [ ] 验证所有平台构建成功
  - [ ] 检查构建产物
  - [ ] 验证 Release 创建成功

- [ ] **发布后**
  - [ ] 下载并测试所有平台的发布包
  - [ ] 更新发布说明（如有必要）
  - [ ] 通知用户新版本发布

### 版本号规范

遵循 [语义化版本 2.0.0](https://semver.org/lang/zh-CN/)：

- **主版本号 (MAJOR)**：不兼容的 API 修改
- **次版本号 (MINOR)**：向下兼容的功能性新增
- **修订号 (PATCH)**：向下兼容的问题修正

示例：
- `1.0.0` → `1.0.1`：Bug 修复
- `1.0.1` → `1.1.0`：新功能
- `1.1.0` → `2.0.0`：破坏性变更

### 发布命令示例

```bash
# 1. 确保在主分支
git checkout main
git pull origin main

# 2. 更新版本号
# 编辑 Cargo.toml 和 CHANGELOG.md

# 3. 提交更改
git add Cargo.toml CHANGELOG.md
git commit -m "chore: bump version to 1.0.0"

# 4. 创建标签
git tag -a v1.0.0 -m "Release version 1.0.0"

# 5. 推送
git push origin main
git push origin v1.0.0

# 6. 等待 GitHub Actions 完成构建
```

## 平台特定说明

### Windows

**依赖项**：
- 无需额外依赖，GPUI 已包含所需的 Windows API

**注意事项**：
- 可执行文件名为 `canview.exe`
- 使用 `start.bat` 启动脚本
- 压缩格式为 ZIP

**测试**：
```powershell
# 解压并测试
Expand-Archive CANVIEW-v1.0.0.zip -DestinationPath test
cd test\CANVIEW-v1.0.0
.\start.bat
```

### macOS

**依赖项**：
- 系统自带，无需额外安装

**注意事项**：
- 可执行文件名为 `canview`（无扩展名）
- 使用 `start.sh` 启动脚本
- 压缩格式为 ZIP
- 首次运行可能需要在"系统偏好设置 > 安全性与隐私"中允许

**测试**：
```bash
# 解压并测试
unzip CANVIEW-v1.0.0-macos.zip
cd CANVIEW-v1.0.0-macos
./start.sh
```

**代码签名**（可选）：
```bash
# 如果需要分发给其他用户，建议进行代码签名
codesign -s "Developer ID Application: Your Name" bin/canview
```

### Linux

**依赖项**：
运行时需要以下库：
- libxcb
- libxkbcommon
- libssl

**注意事项**：
- 可执行文件名为 `canview`（无扩展名）
- 使用 `start.sh` 启动脚本
- 压缩格式为 tar.gz
- 可能需要安装运行时依赖

**测试**：
```bash
# 解压并测试
tar -xzf CANVIEW-v1.0.0-linux.tar.gz
cd CANVIEW-v1.0.0-linux
./start.sh
```

**创建 .desktop 文件**（可选）：
```bash
cat > ~/.local/share/applications/canview.desktop << EOF
[Desktop Entry]
Type=Application
Name=CANVIEW
Exec=/path/to/CANVIEW/bin/canview
Icon=/path/to/CANVIEW/assets/icon.png
Terminal=false
Categories=Development;
EOF
```

## 故障排除

### 构建失败

**问题**：Rust 编译错误
```
解决方案：
1. 确保 Rust 工具链是最新的：rustup update
2. 清理构建缓存：cargo clean
3. 重新构建：cargo build --release
```

**问题**：依赖项缺失（Linux）
```
解决方案：
安装所需的开发库（见 Linux 前提条件部分）
```

### GitHub Actions 失败

**问题**：工作流权限不足
```
解决方案：
在仓库设置中启用 Actions 权限：
Settings > Actions > General > Workflow permissions
选择 "Read and write permissions"
```

**问题**：Release 创建失败
```
解决方案：
1. 确保标签格式正确（v1.0.0）
2. 检查 GITHUB_TOKEN 权限
3. 确保没有同名的 Release 已存在
```

### 打包脚本错误

**问题**：找不到文件
```
解决方案：
确保在项目根目录运行脚本
```

**问题**：权限错误（Linux/macOS）
```
解决方案：
chmod +x package.sh
```

## 最佳实践

1. **版本控制**
   - 始终在主分支创建发布
   - 使用有意义的提交消息
   - 保持 CHANGELOG 更新

2. **测试**
   - 在所有目标平台上测试
   - 使用干净的环境测试
   - 验证所有功能正常工作

3. **文档**
   - 保持文档与代码同步
   - 提供清晰的安装说明
   - 记录已知问题

4. **自动化**
   - 使用 GitHub Actions 自动构建
   - 自动化测试流程
   - 自动化发布流程

5. **安全**
   - 不在代码中硬编码密钥
   - 使用 GitHub Secrets 存储敏感信息
   - 定期更新依赖项

## 相关资源

- [GitHub Actions 文档](https://docs.github.com/en/actions)
- [Rust 交叉编译指南](https://rust-lang.github.io/rustup/cross-compilation.html)
- [语义化版本规范](https://semver.org/lang/zh-CN/)
- [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/)

---

**最后更新**: 2026-01-25
