# CANVIEW 跨平台打包指南

## 🌍 支持的平台

| 平台 | 打包格式 | 脚本 |
|------|----------|------|
| Windows | `.exe` 安装程序, `.zip` | `build-installer.ps1`, `package.ps1` |
| macOS | `.dmg`, `.app`, `.tar.gz` | `package-macos.sh` |
| Linux | `.deb`, `.rpm`, `.tar.gz`, `.AppImage` | `package-linux.sh` |

## 📦 Windows 打包

### 方式 1: 安装程序 (.exe)

**要求**:
- Inno Setup 6.x
- PowerShell

**命令**:
```powershell
.\build-installer.ps1 -Version "1.0.0"
```

**输出**:
- `installer-output\CANVIEW-Setup-v1.0.0.exe`

### 方式 2: ZIP 压缩包

**命令**:
```powershell
.\package.ps1 -Version "1.0.0"
```

**输出**:
- `release-package\CANVIEW-v1.0.0.zip`

## 🍎 macOS 打包

### 准备工作

```bash
# 安装 create-dmg（可选，用于创建 DMG）
brew install create-dmg
```

### 打包命令

```bash
# 给脚本添加执行权限
chmod +x package-macos.sh

# 执行打包
./package-macos.sh 1.0.0
```

### 输出文件

1. **CANVIEW.app** - macOS 应用包
   - 位置: `release-package/CANVIEW.app`
   - 使用: 拖到 Applications 文件夹

2. **CANVIEW-v1.0.0.dmg** - 安装镜像（如果安装了 create-dmg）
   - 位置: `release-package/CANVIEW-v1.0.0.dmg`
   - 使用: 双击打开，拖动安装

3. **CANVIEW-v1.0.0-macos.tar.gz** - 压缩包
   - 位置: `release-package/CANVIEW-v1.0.0-macos.tar.gz`
   - 使用: 解压后拖到 Applications

### 安装方法

```bash
# 方法 1: 使用 DMG
# 1. 双击 .dmg 文件
# 2. 将 CANVIEW.app 拖到 Applications 文件夹

# 方法 2: 使用 tar.gz
tar -xzf CANVIEW-v1.0.0-macos.tar.gz
mv CANVIEW.app /Applications/

# 方法 3: 直接使用 .app
cp -r CANVIEW.app /Applications/
```

### 代码签名（可选）

```bash
# 签名应用
codesign --force --deep --sign "Developer ID Application: Your Name" CANVIEW.app

# 公证应用
xcrun notarytool submit CANVIEW-v1.0.0.dmg --keychain-profile "AC_PASSWORD"
```

## 🐧 Linux 打包

### 准备工作

```bash
# Debian/Ubuntu
sudo apt install dpkg-dev rpm

# Fedora/RHEL
sudo dnf install rpm-build dpkg

# 安装 AppImage 工具（可选）
wget https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage
chmod +x appimagetool-x86_64.AppImage
sudo mv appimagetool-x86_64.AppImage /usr/local/bin/appimagetool
```

### 打包命令

```bash
# 给脚本添加执行权限
chmod +x package-linux.sh

# 执行打包
./package-linux.sh 1.0.0
```

### 输出文件

1. **canview_1.0.0_amd64.deb** - Debian/Ubuntu 包
   - 适用: Debian, Ubuntu, Linux Mint 等

2. **canview-1.0.0-1.*.rpm** - RPM 包
   - 适用: Fedora, RHEL, CentOS, openSUSE 等

3. **canview-v1.0.0-linux-amd64.tar.gz** - 通用包
   - 适用: 所有 Linux 发行版

4. **canview-v1.0.0-x86_64.AppImage** - AppImage（如果安装了工具）
   - 适用: 所有 Linux 发行版，无需安装

### 安装方法

#### Debian/Ubuntu

```bash
# 方法 1: 使用 dpkg
sudo dpkg -i canview_1.0.0_amd64.deb
sudo apt-get install -f  # 修复依赖

# 方法 2: 使用 apt
sudo apt install ./canview_1.0.0_amd64.deb
```

#### Fedora/RHEL/CentOS

```bash
# 方法 1: 使用 rpm
sudo rpm -i canview-1.0.0-1.*.rpm

# 方法 2: 使用 dnf
sudo dnf install canview-1.0.0-1.*.rpm

# 方法 3: 使用 yum
sudo yum install canview-1.0.0-1.*.rpm
```

#### 通用方法 (tar.gz)

```bash
# 解压
tar -xzf canview-v1.0.0-linux-amd64.tar.gz
cd canview-1.0.0

# 安装到系统
sudo cp -r usr/* /usr/

# 或安装到用户目录
mkdir -p ~/.local
cp -r usr/* ~/.local/
```

#### AppImage

```bash
# 添加执行权限
chmod +x canview-v1.0.0-x86_64.AppImage

# 直接运行
./canview-v1.0.0-x86_64.AppImage

# 或集成到系统
./canview-v1.0.0-x86_64.AppImage --appimage-extract
sudo mv squashfs-root /opt/canview
sudo ln -s /opt/canview/AppRun /usr/local/bin/canview
```

## 🔧 跨平台编译

### 使用 GitHub Actions

创建 `.github/workflows/release.yml`:

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build-windows:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Build
        run: cargo build --release -p view
      - name: Package
        run: .\package.ps1 -Version ${{ github.ref_name }}
      - uses: actions/upload-artifact@v3
        with:
          name: windows-package
          path: release-package/*.zip

  build-macos:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Build and Package
        run: |
          chmod +x package-macos.sh
          ./package-macos.sh ${{ github.ref_name }}
      - uses: actions/upload-artifact@v3
        with:
          name: macos-package
          path: release-package/*.tar.gz

  build-linux:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Install dependencies
        run: sudo apt-get install -y dpkg-dev rpm
      - name: Build and Package
        run: |
          chmod +x package-linux.sh
          ./package-linux.sh ${{ github.ref_name }}
      - uses: actions/upload-artifact@v3
        with:
          name: linux-packages
          path: |
            release-package/*.deb
            release-package/*.rpm
            release-package/*.tar.gz
```

## 📋 平台特定配置

### Windows

**文件**: `src/view/build.rs`
```rust
#[cfg(target_os = "windows")]
fn main() {
    // 设置图标和子系统
    println!("cargo:rustc-link-arg-bins=/SUBSYSTEM:WINDOWS");
}
```

### macOS

**Info.plist** 配置:
- 应用图标: `.icns` 格式
- 最低系统版本: macOS 10.13+
- 高分辨率支持

### Linux

**依赖**:
- GTK3 (通过 GPUI)
- X11 或 Wayland
- 标准 C 库

## 🎨 图标准备

### Windows
- 格式: `.ico`
- 尺寸: 16x16, 32x32, 48x48, 256x256
- 位置: `assets/ico/canview.ico`

### macOS
- 格式: `.icns`
- 尺寸: 16x16 到 1024x1024
- 位置: `assets/ico/canview.icns`
- 生成: `iconutil -c icns icon.iconset`

### Linux
- 格式: `.png`
- 尺寸: 256x256 (推荐)
- 位置: `assets/ico/canview.png`

## 📊 打包对比

| 特性 | Windows | macOS | Linux |
|------|---------|-------|-------|
| 图形安装 | ✅ (.exe) | ✅ (.dmg) | ✅ (.deb/.rpm) |
| 便携版 | ✅ (.zip) | ✅ (.app) | ✅ (.AppImage) |
| 自动更新 | ✅ | ✅ | ⚠️ (需配置) |
| 代码签名 | ✅ | ✅ | ❌ |
| 系统集成 | ✅ | ✅ | ✅ |

## ✅ 测试清单

### 所有平台
- [ ] 编译成功
- [ ] 程序能启动
- [ ] 配置目录创建
- [ ] 信号库存储正常
- [ ] 文件选择对话框工作

### Windows
- [ ] 无控制台窗口
- [ ] 开始菜单快捷方式
- [ ] 桌面图标
- [ ] 卸载程序

### macOS
- [ ] .app 包结构正确
- [ ] 图标显示
- [ ] 拖拽安装
- [ ] Launchpad 显示

### Linux
- [ ] .deb 安装成功
- [ ] .rpm 安装成功
- [ ] 桌面快捷方式
- [ ] AppImage 可执行

## 🚀 发布流程

1. **更新版本号**
   ```bash
   # 所有脚本中的版本号
   ```

2. **编译所有平台**
   ```bash
   # Windows
   .\build-installer.ps1 -Version "1.0.0"
   
   # macOS
   ./package-macos.sh 1.0.0
   
   # Linux
   ./package-linux.sh 1.0.0
   ```

3. **测试安装包**
   - 在各平台虚拟机中测试

4. **创建 GitHub Release**
   ```bash
   gh release create v1.0.0 \
     release-package/*.exe \
     release-package/*.dmg \
     release-package/*.deb \
     release-package/*.rpm \
     release-package/*.AppImage \
     --title "CANVIEW v1.0.0" \
     --notes "Release notes here"
   ```

## 📚 相关资源

- **Inno Setup**: https://jrsoftware.org/
- **create-dmg**: https://github.com/create-dmg/create-dmg
- **AppImage**: https://appimage.org/
- **Debian 打包**: https://www.debian.org/doc/manuals/maint-guide/
- **RPM 打包**: https://rpm-packaging-guide.github.io/

---

**创建日期**: 2026-01-25  
**状态**: ✅ 完成  
**支持平台**: Windows, macOS, Linux
