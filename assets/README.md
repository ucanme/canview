# CANVIEW Logo Resources

This directory contains the logo and icon assets for the CANVIEW application.

## 🚀 Quick Start

### Converting Icons for Your Platform

**Windows:**
```cmd
convert_icons.bat
```

**macOS/Linux:**
```bash
chmod +x convert_icons.sh
./convert_icons.sh
```

See [ICON_GUIDE.md](ICON_GUIDE.md) for detailed platform-specific instructions.

---

## Files

### App Icons (for executables)
- **icon_512.svg** - 512x512, highest quality
- **icon_256.svg** - 256x256, standard size
- **icon_128.svg** - 128x128, medium size
- **icon_64.svg** - 64x64, small size
- **icon_32.svg** - 32x32, minimal size

### Logo Variants
- **logo_modern.svg** - Main modern logo with glow effects (512x512)
  - Dark theme version
  - Contains "CANVIEW" title and "BUS DATA ANALYZER" subtitle
  - Recommended for: GitHub, documentation, marketing materials

- **logo_modern_light.svg** - Light theme version (512x512)
  - Same design as logo_modern but for light backgrounds
  - Recommended for: Light-themed documents and presentations

- **logo_icon_only.svg** - Icon without text (256x256)
  - Pure CAN bus waveform visualization
  - Recommended for: In-app icons, UI elements

- **logo_simple.svg** - Simplified design (512x512)
  - Clean version with bracket-style frame
  - Recommended for: App icons, desktop shortcuts

- **logo.svg** - Classic design (256x256)
  - Original design with binary data display
  - Recommended for: General use

### Special Purpose Icons
- **favicon.svg** - Browser tab icon (64x64)
  - Optimized for small sizes
  - Recommended for: Website favicon, browser tab

- **app_logo.svg** - In-app logo (32x32)
  - Small version for UI header
  - Embedded in the application code

### Conversion Scripts
- **convert_icons.bat** - Windows icon conversion script
- **convert_icons.sh** - macOS/Linux icon conversion script

### Documentation
- **ICON_GUIDE.md** - Detailed guide for setting up icons on each platform
- **README.md** - This file

---

## Design Elements

### Color Scheme
- **Primary Gradient**: Green → Blue → Indigo → Purple
  - #10b981 (emerald-500)
  - #3b82f6 (blue-500)
  - #6366f1 (indigo-500)
  - #8b5cf6 (violet-500)

- **Background Colors**:
  - Dark: #1e293b (slate-800) → #0f172a (slate-900)
  - Light: #f8fafc (slate-50) → #e2e8f0 (slate-200)

### Symbolism
- **Waveform**: Represents CAN bus digital signals (High/Low lines)
- **Nodes**: Represents devices/nodes on the CAN network
- **Pulses**: Square waves represent digital data transmission
- **Circular Frame**: Represents the "view" aspect of the analyzer

---

## Usage

### Converting to Other Formats

#### Using ImageMagick
```bash
# Install ImageMagick first
# Windows: https://imagemagick.org/script/download.php
# macOS: brew install imagemagick
# Linux: sudo apt-get install imagemagick

# SVG to PNG
convert logo_modern.svg logo_modern.png

# SVG to ICO (Windows)
convert logo_modern.svg -define icon:auto-resize=256,128,64,48,32,16 canview.ico

# Or use the provided scripts
./convert_icons.sh  # macOS/Linux
convert_icons.bat   # Windows
```

#### Using Online Tools
If you don't want to install ImageMagick:
- SVG to PNG: https://cloudconvert.com/svg-to-png
- PNG to ICO: https://convertico.com/
- SVG to ICNS: https://cloudconvert.com/svg-to-icns

### Platform-Specific Setup

#### Windows (EXE Icon)
1. Convert SVG → ICO using provided scripts
2. Add `winres` dependency to Cargo.toml
3. Create `build.rs` in project root
4. Copy ICO file to `assets/ico/canview.ico`
5. Build with `cargo build --release`

See [ICON_GUIDE.md](ICON_GUIDE.md) for detailed Windows setup.

#### macOS (.app Icon)
1. Convert SVG → ICNS using provided scripts
2. Create .app bundle structure
3. Copy ICNS to `CanView.app/Contents/Resources/`
4. Update Info.plist

See [ICON_GUIDE.md](ICON_GUIDE.md) for detailed macOS setup.

#### Linux (Desktop Icon)
1. Convert SVG → PNG using provided scripts
2. Copy PNG to icon directories
3. Create .desktop file
4. Update icon cache

See [ICON_GUIDE.md](ICON_GUIDE.md) for detailed Linux setup.

---

## Implementation in Code

The in-app logo is rendered using GPUI div elements at:
- **File**: `src/view/src/main.rs`
- **Location**: Top-left header bar
- **Size**: 24x24 pixels
- **Style**: 5 colored dots representing CAN bus nodes

The logo uses the color gradient:
- Center node: #818cf8 (indigo-400)
- Inner nodes: #60a5fa (blue-400)
- Outer nodes: #34d399 (emerald-400)

---

## Icon Specifications

### Windows ICO
Required sizes:
- 256x256 (main)
- 128x128
- 64x64
- 48x48
- 32x32
- 16x16

### macOS ICNS
Required sizes:
- 16x16
- 32x32 (@2x: 64x64)
- 128x128
- 256x256 (@2x: 512x512)
- 512x512

### Linux PNG
Recommended sizes:
- 512x512 (high DPI)
- 256x256 (standard)
- 128x128
- 64x64
- 48x48

---

## Credits
Designed for CANVIEW - Bus Data Analyzer

## License
Part of the CANVIEW project

