#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
CANVIEW Icon Converter
Converts SVG icons to PNG format using Python
"""

import os
import sys
import subprocess
from pathlib import Path

# Fix Windows console encoding
if sys.platform == 'win32':
    sys.stdout.reconfigure(encoding='utf-8', errors='replace')
    sys.stderr.reconfigure(encoding='utf-8', errors='replace')

# Color palette for simple PNG generation (fallback)
COLORS = {
    'bg': (30, 41, 59),  # #1e293b
    'node1': (52, 211, 153),  # #34d399
    'node2': (96, 165, 250),  # #60a5fa
    'node3': (129, 140, 248),  # #818cf8
}

def check_pillow():
    """Check if PIL/Pillow is available"""
    try:
        from PIL import Image, ImageDraw
        return True
    except ImportError:
        return False

def generate_simple_icon(size, output_path):
    """Generate a simple CAN bus icon using PIL"""
    from PIL import Image, ImageDraw

    # Create image
    img = Image.new('RGBA', (size, size), (*COLORS['bg'], 255))
    draw = ImageDraw.Draw(img)

    # Calculate dimensions
    padding = size // 8
    content_size = size - 2 * padding

    # Draw waveform
    center_y = size // 2
    line_width = max(1, size // 64)

    # Draw 5 nodes
    num_nodes = 5
    node_spacing = content_size // (num_nodes + 1)
    node_radius = max(2, size // 32)

    for i in range(num_nodes):
        x = padding + node_spacing * (i + 1)
        y = center_y

        # Choose color based on position
        if i == 2:  # Center
            color = COLORS['node3']
            radius = int(node_radius * 1.5)
        elif i in [1, 3]:  # Inner
            color = COLORS['node2']
            radius = node_radius
        else:  # Outer
            color = COLORS['node1']
            radius = node_radius

        # Draw node circle
        draw.ellipse(
            [x - radius, y - radius, x + radius, y + radius],
            fill=(*color, 255)
        )

    img.save(output_path, 'PNG')
    print(f"✓ Generated {output_path.name} ({size}x{size})")

def main():
    """Main conversion function"""
    print("=" * 60)
    print("CANVIEW Icon Converter")
    print("=" * 60)
    print()

    # Check dependencies
    if not check_pillow():
        print("❌ Error: PIL/Pillow not installed!")
        print()
        print("Please install it using:")
        print("  pip install Pillow")
        print()
        print("Or install cairosvg for better SVG conversion:")
        print("  pip install cairosvg")
        print()
        return 1

    from PIL import Image

    # Create output directories
    png_dir = Path("png")
    ico_dir = Path("ico")
    png_dir.mkdir(exist_ok=True)
    ico_dir.mkdir(exist_ok=True)

    # Sizes to generate
    sizes = [512, 256, 128, 64, 48, 32]

    print("Generating PNG icons...")
    print()

    for size in sizes:
        output_path = png_dir / f"icon_{size}.png"
        generate_simple_icon(size, output_path)

    print()
    print("Creating ICO file...")

    # Create ICO file with multiple sizes
    try:
        from PIL import ImageDraw
        images = []

        # Generate 16x16 separately (too small for detailed drawing)
        img_16 = Image.new('RGBA', (16, 16), (*COLORS['bg'], 255))
        draw_16 = ImageDraw.Draw(img_16)
        draw_16.ellipse([6, 6, 10, 10], fill=(*COLORS['node3'], 255))
        images.append(img_16)

        # Add other sizes
        for size in [32, 48, 64, 128, 256]:
            img_path = png_dir / f"icon_{size}.png"
            if img_path.exists():
                img = Image.open(img_path)
                images.append(img)

        # Save as ICO
        ico_path = ico_dir / "canview.ico"
        images[0].save(
            ico_path,
            format='ICO',
            sizes=[(img.width, img.height) for img in images]
        )

        print(f"✓ Created {ico_path}")

    except Exception as e:
        print(f"❌ Error creating ICO: {e}")
        print("   PNG files are still available in ./png/ directory")

    print()
    print("=" * 60)
    print("✓ Conversion complete!")
    print("=" * 60)
    print()
    print("Output files:")
    print(f"  PNG files: {png_dir.absolute()}/")
    print(f"  ICO file:  {ico_dir.absolute()}/canview.ico")
    print()
    print("Note: These are simplified icons. For best quality, use:")
    print("  1. ImageMagick: convert_icons.bat (Windows)")
    print("  2. Online tools: https://cloudconvert.com/svg-to-png")
    print()

    return 0

if __name__ == "__main__":
    sys.exit(main())
