# CANVIEW Logo Resources

This directory contains the logo and icon assets for the CANVIEW application.

## 🎨 Logo Design

CANVIEW features a modern **oscilloscope-style** logo representing CAN/LIN bus signal analysis.

### Design Characteristics
- **Theme**: Oscilloscope screen with signal waveforms
- **Style**: Professional dark theme (#1a1a1a background)
- **Elements**: 
  - Oscilloscope screen frame
  - Grid background
  - Primary waveform (gray gradient)
  - Secondary waveform (dashed line)
  - 6 animated data points (representing CAN bus nodes)

---

## 📁 File Structure

### SVG Vector Logos
**Location**: `svg/`

All logos are in SVG format for scalability:

- **logo.svg** (200×200) - Main logo, default size
- **logo-16x16.svg** (16×16) - Favicon size
- **logo-32x32.svg** (32×32) - Windows taskbar
- **logo-48x48.svg** (48×48) - Windows shortcut
- **logo-64x64.svg** (64×64) - macOS Dock
- **logo-128x128.svg** (128×128) - Application icon
- **logo-256x256.svg** (256×256) - High-resolution icon
- **logo-512x512.svg** (512×512) - Ultra-high resolution

### PNG Icons
**Location**: `png/`

Pre-rendered PNG files for various uses:

- **logo_16.png** (229 bytes)
- **logo_32.png** (338 bytes)
- **logo_48.png** (480 bytes)
- **logo_64.png** (628 bytes)
- **logo_128.png** (1,245 bytes)
- **logo_256.png** (2,766 bytes)
- **logo_512.png** (5,602 bytes)

### Windows Icon
**Location**: `ico/canview.ico` (14 KB)

Contains all sizes (16-256 pixels) for Windows executable icons.

---

## 🚀 Quick Start

### Regenerating Icons

To regenerate PNG and ICO files from SVG sources:

```bash
cd assets
python draw_logo.py
```

This will create all PNG files and the Windows ICO file.

### Building Application with Icon

The icon is automatically embedded during build:

```bash
cargo build --release -p view
```

The Windows executable will have the icon embedded.

---

## 🎯 Usage Guidelines

### In Documentation

```markdown
![CANVIEW Logo](svg/logo.svg)
```

### In HTML/Web

```html
<!-- Standard size -->
<img src="svg/logo.svg" alt="CANVIEW Logo" width="200">

<!-- Specific size -->
<img src="svg/logo-256x256.svg" alt="CANVIEW Icon" width="256">
```

### In README Files

```markdown
<div align="center">

![CANVIEW Logo](svg/logo.svg)

*Modern CAN/LIN Bus Data Analyzer*

</div>
```

---

## 🛠️ Tools and Scripts

### Python Scripts

- **draw_logo.py** - Generate PNG and ICO files from scratch using Pillow
- **verify_icon.py** - Verify icon files and check application embedding
- **convert_online.py** - Online conversion tool (if local tools unavailable)

### Batch Scripts

- **refresh_icon_cache.bat** - Clear Windows icon cache

---

## 📋 Icon Specifications

### Windows ICO
Contains: 16×16, 32×32, 48×48, 64×64, 128×128, 256×256

### macOS ICNS
To create ICNS (mac only):
```bash
# Use iconutil
mkdir -p canview.iconset
cp png/logo_*.png canview.iconset/
iconutil -c icns canview.iconset
```

### Linux PNG
Recommended: 256×256 or 512×512 for desktop icons

---

## 🎨 Design Details

### Color Scheme

| Element | Color | Usage |
|---------|-------|-------|
| Background | #1a1a1a | Dark background |
| Border | #4a4a4a | Screen frame |
| Grid | #3a3a3a | Grid lines |
| Primary Waveform | #9e9e9e | Main signal |
| Secondary Waveform | #6d6d6d | Secondary signal |
| Data Points | #a0a0a0 | Signal nodes |

### Technical Implementation

- **Format**: Scalable Vector Graphics (SVG)
- **ViewBox**: 0 0 200 200 (base size)
- **Features**: 
  - `<defs>` for gradients and filters
  - `<animate>` tags for pulsing data points
  - Responsive scaling

---

## 📚 Documentation

For detailed logo usage, design specifications, and brand guidelines, see:

- **[LOGO_GUIDE.md](../LOGO_GUIDE.md)** - Complete logo design and usage guide
- **[UPDATE_ICONS.md](UPDATE_ICONS.md)** - Icon update instructions
- **[QUICK_START.md](QUICK_START.md)** - Quick start guide

---

## ✅ Verification

To verify icons are correctly embedded in the executable:

```bash
cd assets
python verify_icon.py
```

This will check:
- ✅ ICO file validity
- ✅ PNG file completeness
- ✅ EXE icon embedding
- ✅ Timestamp analysis

---

## 🔧 Platform-Specific Notes

### Windows
- Icon embedded via `src/view/build.rs`
- Uses `winres` crate
- No additional setup needed

### macOS
- Create ICNS from PNG files
- Use `iconutil` command
- Update Info.plist

### Linux
- Install PNG to icon directories
- Create .desktop file
- Update icon cache

---

## 📞 Support

For issues or questions:
- Check [LOGO_GUIDE.md](../LOGO_GUIDE.md)
- Run verification script: `python verify_icon.py`
- See [FIX_ICON_CACHE.md](FIX_ICON_CACHE.md) for icon display issues

---

**Last Updated**: 2024-02-07  
**Version**: 2.0 (Oscilloscope Style)