# CANVIEW 应用图标设置指南

本指南说明如何为不同平台的 CANVIEW 应用设置图标。

## 📦 文件说明

### SVG 源文件
- `icon_512.svg` - 512x512，最高质量
- `icon_256.svg` - 256x256，标准尺寸
- `icon_128.svg` - 128x128，中等尺寸
- `icon_64.svg` - 64x64，小尺寸
- `icon_32.svg` - 32x32，最小尺寸

### 转换脚本
- `convert_icons.bat` - Windows 批处理脚本
- `convert_icons.sh` - macOS/Linux 脚本

---

## 🪟 Windows (EXE 图标)

### 方法一：使用 ImageMagick（推荐）

1. **安装 ImageMagick**
   - 下载：https://imagemagick.org/script/download.php
   - 选择 Windows 版本安装
   - 安装时勾选 "Install legacy utilities (e.g. convert)"

2. **运行转换脚本**
   ```cmd
   cd assets
   convert_icons.bat
   ```

3. **配置 Cargo.toml**

   在 `Cargo.toml` 中添加：

   ```toml
   [package]
   name = "canview"
   ...

   [target.'cfg(windows)'.build-dependencies]
   winres = "0.1"

   [[bin]]
   name = "canview"
   path = "src/view/src/main.rs"
   ```

4. **创建 build.rs**

   在项目根目录创建 `build.rs`：

   ```rust
   #[cfg(target_os = "windows")]
   fn main() {
       let mut res = winres::WindowsResource::new();
       res.set_icon("assets/ico/canview.ico");
       res.set_icon_with_id("assets/ico/canview.ico", 1);
       res.compile().expect("Failed to compile resources");
   }

   #[cfg(not(target_os = "windows"))]
   fn main() {}
   ```

5. **编译**

   ```cmd
   cargo build --release
   ```

   生成的 EXE 文件在 `target/release/view.exe`，已包含图标。

### 方法二：使用 Resource Hacker（手动）

1. 使用在线工具转换 SVG → PNG → ICO：
   - https://cloudconvert.com/svg-to-png
   - https://convertico.com/

2. 下载 Resource Hacker：
   - https://angusj.com/resourcehacker/

3. 打开 EXE 文件，替换图标：

4. 保存修改后的 EXE

---

## 🍎 macOS (.app 图标)

### 方法一：使用图标工具创建 ICNS

1. **安装 ImageMagick**
   ```bash
   brew install imagemagick
   ```

2. **运行转换脚本**
   ```bash
   cd assets
   chmod +x convert_icons.sh
   ./convert_icons.sh
   ```

   这会生成 `canview.icns` 文件。

3. **创建 .app 包结构**

   ```bash
   mkdir -p CanView.app/Contents/{MacOS,Resources}
   ```

4. **创建 Info.plist**

   在 `CanView.app/Contents/Info.plist`：

   ```xml
   <?xml version="1.0" encoding="UTF-8"?>
   <!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
   <plist version="1.0">
   <dict>
       <key>CFBundleExecutable</key>
       <string>canview</string>
       <key>CFBundleIconFile</key>
       <string>canview</string>
       <key>CFBundleIdentifier</key>
       <string>com.canview.app</string>
       <key>CFBundleName</key>
       <string>CANVIEW</string>
       <key>CFBundlePackageType</key>
       <string>APPL</string>
       <key>CFBundleShortVersionString</key>
       <string>1.0.0</string>
       <key>CFBundleVersion</key>
       <string>1</string>
   </dict>
   </plist>
   ```

5. **复制文件**

   ```bash
   cp target/release/canview CanView.app/Contents/MacOS/
   cp assets/canview.icns CanView.app/Contents/Resources/
   ```

6. **设置图标**

   ```bash
   /usr/bin/iconutil -c icns CanView.app/Contents/Resources/canview.icns
   ```

### 方法二：使用在线工具

访问 https://cloudconvert.com/svg-to-icns 直接转换。

---

## 🐧 Linux (桌面图标)

Linux 桌面环境使用 PNG 文件作为图标。

### 步骤：

1. **安装 ImageMagick**
   ```bash
   # Ubuntu/Debian
   sudo apt-get install imagemagick

   # Fedora
   sudo dnf install imagemagick

   # Arch Linux
   sudo pacman -S imagemagick
   ```

2. **生成 PNG 图标**
   ```bash
   cd assets
   ./convert_icons.sh
   ```

3. **安装图标**

   ```bash
   # 用户级安装
   mkdir -p ~/.local/share/icons/hicolor/256x256/apps
   mkdir -p ~/.local/share/icons/hicolor/512x512/apps
   cp assets/png/icon_256.png ~/.local/share/icons/hicolor/256x256/apps/canview.png
   cp assets/png/icon_512.png ~/.local/share/icons/hicolor/512x512/apps/canview.png

   # 系统级安装
   sudo cp assets/png/icon_256.png /usr/share/icons/hicolor/256x256/apps/canview.png
   sudo cp assets/png/icon_512.png /usr/share/icons/hicolor/512x256/apps/canview.png
   ```

4. **创建 .desktop 文件**

   在 `~/.local/share/applications/canview.desktop`：

   ```ini
   [Desktop Entry]
   Name=CANVIEW
   Comment=Bus Data Analyzer
   Exec=/path/to/canview
   Icon=canview
   Terminal=false
   Type=Application
   Categories=Development;Electronics;
   StartupNotify=true
   ```

5. **刷新图标缓存**
   ```bash
   update-desktop-database ~/.local/share/applications
   gtk-update-icon-cache ~/.local/share/icons/hicolor -f
   ```

---

## 🌐 在线转换工具

如果不想安装工具，可以使用在线服务：

### SVG 转 PNG
- https://cloudconvert.com/svg-to-png
- https://convertio.co/svg-png/
- https://www.aconvert.com/image/svg-to-png/

### PNG 转 ICO
- https://convertico.com/
- https://www.imgonline.com.ua/eng/convert-png-to-ico.php

### SVG 转 ICNS (macOS)
- https://cloudconvert.com/svg-to-icns
- https://www.icoconverter.com/

---

## 📋 图标规格要求

### Windows ICO
必须包含以下尺寸：
- 256x256 (主要)
- 128x128
- 64x64
- 48x48
- 32x32
- 16x16

### macOS ICNS
必须包含以下尺寸：
- 16x16
- 32x32 (@2x: 64x64)
- 128x128
- 256x256 (@2x: 512x512)
- 512x512
- 1024x1024 (@2x)

### Linux PNG
推荐尺寸：
- 512x512 (高DPI)
- 256x256 (标准)
- 128x128 (小尺寸)
- 64x64 (菜单)
- 48x48 (传统)
- 32x32 (面板)

---

## 🎨 图标设计规格

**当前设计特点：**
- **尺寸**: 512x512 基准
- **圆角**: 外框圆角 112px (512), 56px (256)
- **节点**: 5个圆形节点代表 CAN 总线设备
- **颜色**: 绿→蓝→靛→紫渐变 (#10b981 → #8b5cf6)
- **背景**: 深色渐变 (#1e293b → #0f172a)
- **波形**: 双线表示 CAN High/Low

---

## ✅ 验证图标

### Windows
1. 编译后查看 EXE 文件图标
2. 或在文件管理器中查看

### macOS
1. 查看 .app 包的图标显示
2. 或在 Finder 中查看

### Linux
1. 在应用菜单中查看
2. 或在文件管理器中查看

---

## 🔧 故障排除

### 问题：图标未更新
**解决**：清除图标缓存
- Windows: 删除 `%localappdata%\IconCache.db`
- macOS: `sudo rm -rf /Library/Caches/com.apple.iconservices*`
- Linux: `gtk-update-icon-cache -f`

### 问题：ICO 文件无效
**解决**：确保包含所有必需尺寸，使用 ImageMagick 重新生成

### 问题：编译失败
**解决**：确保 build.rs 在项目根目录，且 winres 依赖已添加

---

## 📞 支持

如有问题，请参考：
- ImageMagick 文档: https://imagemagick.org/
- Windows 资源: https://learn.microsoft.com/en-us/windows/win32/menurc/about-resource-files
- macOS 图标: https://developer.apple.com/design/human-interface-guidelines/app-icons
