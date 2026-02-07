#!/usr/bin/env python3
"""
CANVIEW Icon Conversion Script
Converts the new logo.svg to PNG and ICO formats
"""

import os
import subprocess
import sys
from pathlib import Path

# Configuration
SCRIPT_DIR = Path(__file__).parent
LOGO_SVG = SCRIPT_DIR / "svg" / "logo.svg"
PNG_DIR = SCRIPT_DIR / "png"
ICO_DIR = SCRIPT_DIR / "ico"
SIZES = [512, 256, 128, 64, 48, 32, 16]


def check_dependencies():
    """Check if required dependencies are installed"""
    missing = []

    # Check for Pillow
    try:
        import PIL
        from PIL import Image
    except ImportError:
        missing.append("Pillow (PIL)")

    # Check for cairosvg (optional but recommended)
    try:
        import cairosvg
    except ImportError:
        missing.append("cairosvg (optional, for better SVG conversion)")

    if missing:
        print("❌ Missing dependencies:")
        for dep in missing:
            print(f"   - {dep}")
        print("\n📦 Install with:")
        print("   pip install Pillow cairosvg")
        return False

    return True


def convert_svg_to_png():
    """Convert SVG to PNG at multiple sizes"""
    import io

    import cairosvg
    from PIL import Image

    print("🔄 Converting SVG to PNG...")

    # Create directories
    PNG_DIR.mkdir(exist_ok=True)
    ICO_DIR.mkdir(exist_ok=True)

    # Check if logo.svg exists
    if not LOGO_SVG.exists():
        print(f"❌ Error: {LOGO_SVG} not found!")
        return False

    # Convert to each size
    for i, size in enumerate(SIZES, 1):
        print(f"   [{i}/{len(SIZES)}] Converting to {size}x{size} PNG...", end=" ")

        try:
            # Convert SVG to PNG
            png_data = cairosvg.svg2png(
                url=str(LOGO_SVG), output_width=size, output_height=size
            )

            # Save PNG
            png_path = PNG_DIR / f"logo_{size}.png"
            with open(png_path, "wb") as f:
                f.write(png_data)

            print("✓")
        except Exception as e:
            print(f"✗ ({e})")
            return False

    print("✅ PNG conversion complete!")
    return True


def create_ico_file():
    """Create ICO file from PNG images"""
    from PIL import Image

    print("\n🔄 Creating ICO file for Windows...")

    try:
        # Load PNG images for ICO (use most common sizes)
        ico_images = []
        for size in [256, 128, 64, 48, 32, 16]:
            png_path = PNG_DIR / f"logo_{size}.png"
            if png_path.exists():
                img = Image.open(png_path)
                ico_images.append(img)

        # Save as ICO
        ico_path = ICO_DIR / "canview.ico"
        ico_images[0].save(
            ico_path,
            format="ICO",
            sizes=[(img.width, img.height) for img in ico_images],
        )

        print(f"✅ ICO file created: {ico_path}")
        return True
    except Exception as e:
        print(f"❌ Error creating ICO: {e}")
        return False


def create_icns_file():
    """Create ICNS file for macOS (only works on macOS)"""
    if sys.platform != "darwin":
        print("\n⚠️  Skipping ICNS creation (macOS only)")
        return True

    print("\n🔄 Creating ICNS file for macOS...")

    iconset_dir = SCRIPT_DIR / "canview.iconset"
    iconset_dir.mkdir(exist_ok=True)

    # Create iconset structure
    icon_mappings = [
        ("logo_16.png", "icon_16x16.png"),
        ("logo_32.png", "icon_16x16@2x.png"),
        ("logo_32.png", "icon_32x32.png"),
        ("logo_64.png", "icon_32x32@2x.png"),
        ("logo_128.png", "icon_128x128.png"),
        ("logo_256.png", "icon_128x128@2x.png"),
        ("logo_256.png", "icon_256x256.png"),
        ("logo_512.png", "icon_256x256@2x.png"),
        ("logo_512.png", "icon_512x512.png"),
    ]

    try:
        for src, dst in icon_mappings:
            src_path = PNG_DIR / src
            dst_path = iconset_dir / dst
            if src_path.exists():
                import shutil

                shutil.copy2(src_path, dst_path)

        # Use iconutil to create ICNS
        result = subprocess.run(
            ["iconutil", "-c", "icns", str(iconset_dir)], capture_output=True, text=True
        )

        if result.returncode == 0:
            print("✅ ICNS file created: canview.icns")
            # Clean up iconset
            import shutil

            shutil.rmtree(iconset_dir)
            return True
        else:
            print(f"⚠️  iconutil failed: {result.stderr}")
            return False

    except Exception as e:
        print(f"❌ Error creating ICNS: {e}")
        return False


def main():
    """Main conversion workflow"""
    print("=" * 50)
    print("CANVIEW Icon Conversion Script")
    print("Using new logo from assets/svg/logo.svg")
    print("=" * 50)
    print()

    # Check dependencies
    if not check_dependencies():
        print("\n❌ Please install required dependencies first")
        return 1

    # Convert SVG to PNG
    if not convert_svg_to_png():
        print("\n❌ PNG conversion failed")
        return 1

    # Create ICO file
    if not create_ico_file():
        print("\n❌ ICO creation failed")
        return 1

    # Create ICNS file (macOS only)
    if not create_icns_file():
        print("\n⚠️  ICNS creation failed (non-critical)")

    # Summary
    print("\n" + "=" * 50)
    print("✅ Conversion complete!")
    print("=" * 50)
    print(f"\n📁 Output files:")
    print(f"   PNG: {PNG_DIR}/")
    print(f"   ICO: {ICO_DIR}/canview.ico")
    if sys.platform == "darwin":
        print(f"   ICNS: canview.icns")

    print(f"\n📝 Source logo: {LOGO_SVG}")
    print(f"\n🚀 Next steps:")
    print(f"   1. Build the application:")
    print(f"      cargo build --release")
    print(f"   2. The Windows EXE will have the new icon embedded")
    print(f"   3. For macOS/Linux, use the generated PNG/ICNS files")

    return 0


if __name__ == "__main__":
    sys.exit(main())
