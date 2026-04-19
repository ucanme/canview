cd assets
update_icons.bat
cargo build --release
```

### macOS/Linux 用户

```bash
cd assets
chmod +x update_icons.sh
./update_icons.sh
cargo build --release
```

---

## 📁 文件结构

```
assets/
├── svg/
│   ├── logo.svg              # 新的主Logo (200×200)
│   ├── logo-16x16.svg        # 16×16 尺寸
│   ├── logo-32x32.svg        # 32×32 尺寸
│   ├── logo-48x48.svg        # 48×48 尺寸
│   ├── logo-64x64.svg        # 64×64 尺寸
│   ├── logo-128x128.svg      # 128×128 尺寸
│   ├── logo-256x256.svg      # 256×256 尺寸
│   └── logo-512x512.svg      # 512×512 尺寸
├── ico/
│   └── canview.ico           # Windows 图标 (自动生成)
├── png/
│   ├── logo_16.png           # PNG 格式 (自动生成)
│   ├── logo_32.png
│   ├── logo_48.png
│   ├── logo_64.png
│   ├── logo_128.png
│   ├── logo_256.png
│   └── logo_512.png
├── update_icons.bat          # Windows 转换脚本
├── update_icons.sh           # macOS/Linux 转换脚本
└── update_icons.py           # Python 转换脚本
```

---

## 🔧 方法一：自动转换（推荐）

### 前置要求

#### Windows
需要安装 **ImageMagick**：
1. 下载：https://imagemagick.org/script/download.php
2. 安装时勾选 "Install legacy utilities (e.g. convert)"
3. 验证安装：`magick --version`

#### macOS/Linux
选择以下工具之一：

**选项 A: ImageMagick**
```bash
# macOS
brew install imagemagick

# Ubuntu/Debian
sudo apt-get install imagemagick

# Fedora
sudo dnf install imagemagick
```

**选项 B: Python 脚本**
```bash
pip install Pillow cairosvg
```

### 运行转换脚本

#### Windows (CMD)
```cmd
cd C:\Users\Administrator\RustroverProjects\canview\assets
update_icons.bat
```

#### macOS/Linux (Bash)
```bash
cd /path/to/canview/assets
chmod +x update_icons.sh
./update_icons.sh
```

#### Python (跨平台)
```bash
cd /path/to/canview/assets
python3 update_icons.py
```

### 脚本执行内容

脚本会自动完成以下步骤：

1. ✅ 读取 `svg/logo.svg` 源文件
2. ✅ 生成 7 种尺寸的 PNG 文件（16, 32, 48, 64, 128, 256, 512）
3. ✅ 创建 Windows ICO 文件（包含所有尺寸）
4. ✅ 创建 macOS ICNS 文件（仅 macOS）
5. ✅ 输出到 `assets/ico/` 和 `assets/png/` 目录

### 编译应用

转换完成后，重新编译应用以嵌入新图标：

```bash
cargo build --release
```

Windows 用户会在 `target/release/view.exe` 看到新的图标。

---

## 🎨 方法二：在线转换（无需安装工具）

如果不想安装工具，可以使用在线服务：

### 步骤 1: SVG 转 PNG

访问以下网站之一：
- https://cloudconvert.com/svg-to-png
- https://convertio.co/svg-png/
- https://www.aconvert.com/image/svg-to-png/

操作：
1. 上传 `assets/svg/logo.svg`
2. 选择需要的尺寸（建议：512, 256, 128, 64, 48, 32, 16）
3. 下载所有 PNG 文件
4. 保存到 `assets/png/` 目录

### 步骤 2: PNG 转 ICO（Windows）

访问：
- https://convertico.com/
- https://www.imgonline.com.ua/eng/convert-png-to-ico.php

操作：
1. 上传多个 PNG 文件（256, 128, 64, 48, 32, 16）
2. 生成 ICO 文件
3. 下载并保存为 `assets/ico/canview.ico`

### 步骤 3: PNG 转 ICNS（macOS）

访问：
- https://cloudconvert.com/svg-to-icns

操作：
1. 上传 `assets/svg/logo.svg`
2. 生成 ICNS 文件
3. 下载并保存为 `assets/canview.icns`

---

## 📦 应用图标到各平台

### Windows (EXE 图标)

**配置文件**: `src/view/build.rs`

```rust
#[cfg(target_os = "windows")]
fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_icon("../../assets/ico/canview.ico");
    res.compile().expect("Failed to compile resources");
}
```

**编译**:
```cmd
cargo build --release
```

**输出**: `target/release/view.exe` (已包含图标)

---

### macOS (.app 图标)

**创建 .app 包**:
```bash
mkdir -p CanView.app/Contents/{MacOS,Resources}

# 创建 Info.plist
cat > CanView.app/Contents/Info.plist << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" 
    "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>view</string>
    <key>CFBundleIconFile</key>
    <string>canview</string>
    <key>CFBundleIdentifier</key>
    <string>com.canview.app</string>
    <key>CFBundleName</key>
    <string>CANVIEW</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
</dict>
</plist>
EOF

# 复制文件
cp target/release/view CanView.app/Contents/MacOS/
cp assets/canview.icns CanView.app/Contents/Resources/
```

---

### Linux (桌面图标)

**安装图标**:
```bash
# 用户级安装
mkdir -p ~/.local/share/icons/hicolor/{256x256,512x512}/apps
cp assets/png/logo_256.png ~/.local/share/icons/hicolor/256x256/apps/canview.png
cp assets/png/logo_512.png ~/.local/share/icons/hicolor/512x512/apps/canview.png

# 创建 .desktop 文件
cat > ~/.local/share/applications/canview.desktop << 'EOF'
[Desktop Entry]
Name=CANVIEW
Comment=CAN/LIN Bus Data Analyzer
Exec=/path/to/target/release/view
Icon=canview
Terminal=false
Type=Application
Categories=Development;Electronics;
StartupNotify=true
EOF

# 刷新图标缓存
update-desktop-database ~/.local/share/applications
gtk-update-icon-cache ~/.local/share/icons/hicolor -f
```

---

## ✅ 验证图标更新

### Windows
1. 在文件资源管理器中查看 `target/release/view.exe`
2. 检查文件图标是否为新 Logo
3. 运行程序，查看任务栏图标

### macOS
1. 在 Finder 中查看 `CanView.app`
2. 检查应用图标是否更新
3. 将应用拖到 Dock 查看效果

### Linux
1. 在应用菜单中搜索 "CANVIEW"
2. 检查图标显示是否正确
3. 创建桌面快捷方式验证

---

## 🔍 故障排除

### 问题 1: ImageMagick 未找到

**错误信息**: `ERROR: ImageMagick not found!`

**解决方法**:
- Windows: 从官网下载安装 ImageMagick
- macOS: `brew install imagemagick`
- Linux: 使用包管理器安装

---

### 问题 2: 编译后图标未更新

**原因**: Windows 图标缓存

**解决方法**:
```cmd
# 清除图标缓存
ie4uinit.exe -show
# 或重启电脑
```

---

### 问题 3: ICO 文件无效

**原因**: 缺少必需的尺寸

**解决方法**: 确保 ICO 文件包含以下所有尺寸：
- 256×256
- 128×128
- 64×64
- 48×48
- 32×32
- 16×16

---

### 问题 4: SVG 转 PNG 质量差

**解决方法**:
- 使用 `rsvg-convert`（Linux/macOS）获得更好质量
- 或使用在线工具：https://cloudconvert.com/svg-to-png

---

## 📊 图标尺寸规格

### Windows ICO
| 尺寸 | 用途 |
|------|------|
| 256×256 | 高 DPI 显示 |
| 128×128 | 中等尺寸 |
| 64×64 | 桌面快捷方式 |
| 48×48 | 传统图标 |
| 32×32 | 任务栏/标题栏 |
| 16×16 | 最小尺寸 |

### macOS ICNS
| 尺寸 | 用途 |
|------|------|
| 512×512 | Retina 显示 |
| 256×256 | 标准 |
| 128×128 | 小尺寸 |
| 32×32 | Dock |
| 16×16 | 最小 |

### Linux PNG
推荐尺寸：512, 256, 128, 64, 48, 32

---

## 🎨 Logo 设计细节

**颜色方案**：
- 渐变线条：#9e9e9e → #757575 → #9e9e9e
- 背景：#1a1a1a（深灰）
- 边框：#4a4a4a
- 网格：#3a3a3a（40% 透明度）

**视觉元素**：
- 示波器屏幕框架
- 艺术化网格背景
- 双波形线（主波形 + 次要波形）
- 动画数据点（6个脉动圆点）
- 模糊滤镜效果

---

## 📞 技术支持

如有问题，请参考：
- **项目文档**: `assets/ICON_GUIDE.md`
- **ImageMagick 文档**: https://imagemagick.org/
- **Windows 资源编译**: `build.rs` 配置

---

## 📝 更新日志

### 2024-XX-XX
- ✅ 创建新的 SVG Logo（示波器风格）
- ✅ 生成 7 种尺寸的 SVG 文件
- ✅ 更新图标转换脚本
- ✅ 更新本文档

---

## 🚀 下一步

1. ✅ 运行转换脚本生成 PNG/ICO 文件
2. ✅ 编译应用：`cargo build --release`
3. ✅ 验证各平台图标显示
4. ✅ 提交更新到版本控制

---

**祝使用愉快！🎉**