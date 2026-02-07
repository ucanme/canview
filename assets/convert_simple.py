#!/usr/bin/env python3
"""
CANVIEW Icon Conversion Script (Simplified)
Converts logo.svg to PNG and ICO using svglib and Pillow
"""

import os
import sys
from pathlib import Path

try:
    from PIL import Image
    from reportlab.graphics import renderPM
    from svglib.svglib import svg2rlg

    HAS_DEPS = True
except ImportError as e:
    HAS_DEPS = False
    MISSING_DEP = str(e).split("'")[1] if "'" in str(e) else "svglib/reportlab"

# Configuration
SCRIPT_DIR = Path(__file__).parent
LOGO_SVG = SCRIPT_DIR / "svg" / "logo.svg"
PNG_DIR = SCRIPT_DIR / "png"
ICO_DIR = SCRIPT_DIR / "ico"
SIZES = [512, 256, 128, 64, 48, 32, 16]


def check_dependencies():
    """Check if required dependencies are installed"""
    if not HAS_DEPS:
        print(f"❌ Missing dependency: {MISSING_DEP}")
        print("\n📦 Install with:")
        print("   pip install svglib reportlab pillow")
        return False
    return True


def convert_svg_to_png():
    """Convert SVG to PNG at multiple sizes"""
    print("🔄 Converting SVG to PNG...")

    # Create directories
    PNG_DIR.mkdir(exist_ok=True)
    ICO_DIR.mkdir(exist_ok=True)

    # Check if logo.svg exists
    if not LOGO_SVG.exists():
        print(f"❌ Error: {LOGO_SVG} not found!")
        return False

    try:
        # Load SVG
        print(f"   Loading SVG: {LOGO_SVG}")
        drawing = svg2rlg(str(LOGO_SVG))

        # Convert to each size
        for i, size in enumerate(SIZES, 1):
            print(f"   [{i}/{len(SIZES)}] Generating {size}x{size} PNG...", end=" ")

            png_path = PNG_DIR / f"logo_{size}.png"

            # Render to PNG
            renderPM.drawToFile(
                drawing, str(png_path), fmt="PNG", dpi=int(size * 72 / 200)
            )

            # Resize if needed (svglib might not respect exact size)
            img = Image.open(png_path)
            if img.size != (size, size):
                img = img.resize((size, size), Image.Resampling.LANCZOS)
                img.save(png_path)

            print("✓")

        print("✅ PNG conversion complete!")
        return True

    except Exception as e:
        print(f"\n❌ Error converting SVG: {e}")
        return False


def create_ico_file():
    """Create ICO file from PNG images"""
    print("\n🔄 Creating ICO file for Windows...")

    try:
        # Load PNG images for ICO
        ico_images = []
        for size in [256, 128, 64, 48, 32, 16]:
            png_path = PNG_DIR / f"logo_{size}.png"
            if png_path.exists():
                img = Image.open(png_path)
                ico_images.append(img)

        if not ico_images:
            print("❌ No PNG files found!")
            return False

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


def main():
    """Main conversion workflow"""
    print("=" * 60)
    print("CANVIEW Icon Conversion Script (Simplified)")
    print("Using new logo from assets/svg/logo.svg")
    print("=" * 60)
    print()

    # Check dependencies
    if not check_dependencies():
        print("\n❌ Please install required dependencies:")
        print("   pip install svglib reportlab pillow")
        return 1

    # Convert SVG to PNG
    if not convert_svg_to_png():
        return 1

    # Create ICO file
    if not create_ico_file():
        return 1

    # Summary
    print("\n" + "=" * 60)
    print("✅ Conversion complete!")
    print("=" * 60)
    print(f"\n📁 Output files:")
    print(f"   PNG: {PNG_DIR}/")
    print(f"   ICO: {ICO_DIR}/canview.ico")
    print(f"\n📝 Source logo: {LOGO_SVG}")
    print(f"\n🚀 Next step:")
    print(f"   cargo build --release")
    print()

    return 0


if __name__ == "__main__":
    sys.exit(main())
