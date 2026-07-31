pip install Pillow cairosvg
```

### 方式 3: 在线工具（无需安装）
- SVG 转 PNG: https://cloudconvert.com/svg-to-png
- PNG 转 ICO: https://convertico.com/

---

## ⚡ 三步更新图标

### 步骤 1: 转换图标

#### Windows (CMD)
```cmd
cd C:\Users\Administrator\RustroverProjects\can-viewer\assets
update_icons.bat
```

#### macOS/Linux (Bash)
```bash
cd /path/to/can-viewer/assets
chmod +x update_icons.sh
./update_icons.sh
```

#### Python (跨平台)
```bash
cd /path/to/can-viewer/assets
python3 update_icons.py
```

**脚本会自动生成：**
- ✅ `png/logo_16.png` ~ `png/logo_512.png` (7个尺寸)
- ✅ `ico/can-viewer.ico` (Windows 图标)
- ✅ `can-viewer.icns` (仅 macOS)

---

### 步骤 2: 编译应用

```bash
cd C:\Users\Administrator\RustroverProjects\can-viewer
cargo build --release
```

---

### 步骤 3: 验证图标

#### Windows
- 查看 `target/release/view.exe` 的图标
- 在文件资源管理器中检查

#### macOS
```bash
# 创建 .app 包（如果还没有）
mkdir -p can-viewer.app/Contents/{MacOS,Resources}
cp target/release/view can-viewer.app/Contents/MacOS/
cp assets/can-viewer.icns can-viewer.app/Contents/Resources/
```

#### Linux
```bash
# 安装桌面图标
mkdir -p ~/.local/share/icons/hicolor/256x256/apps
cp assets/png/logo_256.png ~/.local/share/icons/hicolor/256x256/apps/can-viewer.png
```

---

## 🎨 新 Logo 文件位置

```
assets/svg/
├── logo.svg          # 主 Logo (200×200)
├── logo-16x16.svg    # 最小尺寸
├── logo-32x32.svg    # 小尺寸
├── logo-48x48.svg    # 较小尺寸
├── logo-64x64.svg    # 中等尺寸
├── logo-128x128.svg  # 大尺寸
├── logo-256x256.svg  # 较大尺寸
└── logo-512x512.svg  # 最大尺寸
```

---

## 🔧 配置文件

### Windows 图标配置
文件：`src/view/build.rs`
```rust
#[cfg(target_os = "windows")]
fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_icon("../../assets/ico/can-viewer.ico");
    res.compile().expect("Failed to compile resources");
}
```

这个配置已经正确设置，无需修改！

---

## ❓ 常见问题

### Q: 没有安装 ImageMagick，有其他方法吗？
**A:** 可以使用在线转换工具：
1. 访问 https://cloudconvert.com/svg-to-png
2. 上传 `assets/svg/logo.svg`
3. 选择多个尺寸下载
4. 使用 https://convertico.com/ 转换为 ICO

### Q: 编译后图标没有更新？
**A:** Windows 图标缓存问题，尝试：
```cmd
ie4uinit.exe -show
```
或重启电脑。

### Q: 转换脚本失败？
**A:** 确保：
- ImageMagick 已正确安装
- 或使用 Python 脚本
- 或使用在线工具手动转换

---

## 📦 文件说明

| 文件 | 用途 |
|------|------|
| `update_icons.bat` | Windows 自动转换脚本 |
| `update_icons.sh` | macOS/Linux 自动转换脚本 |
| `update_icons.py` | Python 跨平台转换脚本 |
| `UPDATE_ICONS.md` | 详细说明文档 |
| `ICON_GUIDE.md` | 原始图标指南 |

---

## ✅ 检查清单

- [ ] 运行转换脚本生成 PNG/ICO/ICNS 文件
- [ ] 检查 `assets/ico/can-viewer.ico` 是否生成
- [ ] 检查 `assets/png/` 目录下的 PNG 文件
- [ ] 运行 `cargo build --release`
- [ ] 验证 EXE/App 图标已更新

---

## 🎉 完成！

现在你的 can-viewer 应用已经使用新的示波器风格 Logo 了！

**Logo 特点：**
- 🎨 示波器屏幕设计
- 📊 艺术化波形动画
- 🌈 渐变色彩效果
- ✨ 脉动数据点
- 🖥️ 深色主题

---

**需要帮助？** 查看 `UPDATE_ICONS.md` 获取详细说明。