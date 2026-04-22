#!/usr/bin/env python3
"""
CANVIEW Icon Generator
直接使用Pillow绘制logo，无需SVG转换工具
"""

import math
import sys
from pathlib import Path

try:
    from PIL import Image, ImageDraw
except ImportError:
    print("❌ 需要安装 Pillow: pip install pillow")
    sys.exit(1)

# Configuration
SCRIPT_DIR = Path(__file__).parent
PNG_DIR = SCRIPT_DIR / "png"
ICO_DIR = SCRIPT_DIR / "ico"
SIZES = [512, 256, 128, 64, 48, 32, 16]


def draw_logo(size):
    """绘制指定尺寸的logo"""
    # 创建画布（深色背景）
    img = Image.new("RGB", (size, size), "#1a1a1a")
    draw = ImageDraw.Draw(img)

    # 计算缩放比例
    scale = size / 200

    # 示波器屏幕边框
    margin = int(20 * scale)
    border_width = int(3 * scale)
    corner_radius = int(10 * scale)

    # 绘制屏幕外框
    screen_rect = [margin, margin, size - margin, size - margin]

    # 使用圆角矩形效果（简化为普通矩形）
    draw.rectangle(screen_rect, outline="#4a4a4a", width=border_width)

    # 绘制网格线
    grid_color = "#3a3a3a"
    grid_positions = [
        int(60 * scale),
        int(100 * scale),
        int(140 * scale),
        int(60 * scale),
        int(100 * scale),
        int(140 * scale),
    ]

    # 水平网格线
    for y in grid_positions[:3]:
        y_pos = margin + int((y - 20) * scale)
        if margin < y_pos < size - margin:
            draw.line(
                [(margin, y_pos), (size - margin, y_pos)], fill=grid_color, width=1
            )

    # 垂直网格线
    for x in grid_positions[3:]:
        x_pos = margin + int((x - 20) * scale)
        if margin < x_pos < size - margin:
            draw.line(
                [(x_pos, margin), (x_pos, size - margin)], fill=grid_color, width=1
            )

    # 绘制主波形
    wave_color = "#9e9e9e"
    wave_width = max(1, int(4 * scale))

    # 波形路径点（归一化坐标）
    wave_points_normalized = [
        (30, 100),
        (40, 60),
        (50, 60),
        (60, 60),
        (80, 80),
        (90, 140),
        (100, 160),
        (110, 160),
        (120, 120),
        (130, 80),
        (140, 100),
        (150, 120),
        (160, 120),
        (170, 80),
    ]

    # 转换到实际尺寸
    main_wave_points = []
    for x, y in wave_points_normalized:
        px = margin + int((x - 20) * scale)
        py = margin + int((y - 20) * scale)
        main_wave_points.append((px, py))

    # 绘制主波形（使用平滑曲线）
    if len(main_wave_points) > 1:
        for i in range(len(main_wave_points) - 1):
            draw.line(
                [main_wave_points[i], main_wave_points[i + 1]],
                fill=wave_color,
                width=wave_width,
            )

    # 绘制次要波形（虚线效果）
    secondary_wave_color = "#6d6d6d"
    secondary_wave_width = max(1, int(2 * scale))

    secondary_wave_points_normalized = [
        (30, 110),
        (40, 80),
        (50, 80),
        (60, 80),
        (75, 100),
        (85, 130),
        (95, 150),
        (105, 150),
        (115, 110),
        (130, 90),
        (140, 110),
        (150, 130),
        (160, 130),
        (170, 100),
    ]

    secondary_wave_points = []
    for x, y in secondary_wave_points_normalized:
        px = margin + int((x - 20) * scale)
        py = margin + int((y - 20) * scale)
        secondary_wave_points.append((px, py))

    # 绘制次要波形
    if len(secondary_wave_points) > 1:
        for i in range(len(secondary_wave_points) - 1):
            # 创建虚线效果
            x1, y1 = secondary_wave_points[i]
            x2, y2 = secondary_wave_points[i + 1]
            distance = math.sqrt((x2 - x1) ** 2 + (y2 - y1) ** 2)
            dash_length = max(2, int(4 * scale))
            gap_length = max(1, int(2 * scale))

            if distance > 0:
                num_dashes = int(distance / (dash_length + gap_length))
                for j in range(num_dashes + 1):
                    start_ratio = j * (dash_length + gap_length) / distance
                    end_ratio = min(
                        1, (j * (dash_length + gap_length) + dash_length) / distance
                    )

                    dash_start = (
                        int(x1 + (x2 - x1) * start_ratio),
                        int(y1 + (y2 - y1) * start_ratio),
                    )
                    dash_end = (
                        int(x1 + (x2 - x1) * end_ratio),
                        int(y1 + (y2 - y1) * end_ratio),
                    )

                    draw.line(
                        [dash_start, dash_end],
                        fill=secondary_wave_color,
                        width=secondary_wave_width,
                    )

    # 绘制数据点（圆圈）
    data_points_normalized = [
        (45, 60, 3),  # x, y, radius
        (70, 140, 2.5),
        (110, 120, 3.5),
        (130, 80, 2),
        (160, 140, 3),
        (170, 80, 2.5),
    ]

    for x, y, r in data_points_normalized:
        cx = margin + int((x - 20) * scale)
        cy = margin + int((y - 20) * scale)
        radius = max(1, int(r * scale))

        # 外圈（光晕效果）
        if radius >= 2:
            draw.ellipse(
                [cx - radius - 1, cy - radius - 1, cx + radius + 1, cy + radius + 1],
                fill="#a0a0a0",
            )

        # 内圈
        draw.ellipse(
            [cx - radius, cy - radius, cx + radius, cy + radius], fill="#b0b0b0"
        )

    return img


def create_png_files():
    """创建所有尺寸的PNG文件"""
    print("🔄 生成PNG文件...")

    PNG_DIR.mkdir(exist_ok=True)
    ICO_DIR.mkdir(exist_ok=True)

    for i, size in enumerate(SIZES, 1):
        print(f"   [{i}/{len(SIZES)}] 生成 {size}x{size} PNG...", end=" ")

        try:
            img = draw_logo(size)
            png_path = PNG_DIR / f"logo_{size}.png"
            img.save(png_path, "PNG")
            print("✓")
        except Exception as e:
            print(f"✗ ({e})")
            return False

    print("✅ PNG文件生成完成!")
    return True


def create_ico_file():
    """创建ICO文件"""
    print("\n🔄 创建ICO文件...")

    try:
        # 收集PNG图像
        ico_images = []
        for size in [256, 128, 64, 48, 32, 16]:
            png_path = PNG_DIR / f"logo_{size}.png"
            if png_path.exists():
                img = Image.open(png_path)
                ico_images.append(img)

        if not ico_images:
            print("❌ 没有找到PNG文件")
            return False

        # 保存为ICO
        ico_path = ICO_DIR / "canview.ico"
        ico_images[0].save(
            ico_path,
            format="ICO",
            sizes=[(img.width, img.height) for img in ico_images],
        )

        print(f"✅ ICO文件已创建: {ico_path}")
        return True

    except Exception as e:
        print(f"❌ 创建ICO失败: {e}")
        return False


def main():
    """主函数"""
    print("=" * 70)
    print("CANVIEW 图标生成器")
    print("使用Pillow直接绘制logo（无需SVG转换工具）")
    print("=" * 70)
    print()

    # 创建PNG文件
    if not create_png_files():
        print("\n❌ PNG生成失败")
        return 1

    # 创建ICO文件
    if not create_ico_file():
        print("\n❌ ICO创建失败")
        return 1

    # 成功
    print("\n" + "=" * 70)
    print("✅ 图标生成完成!")
    print("=" * 70)
    print(f"\n📁 生成的文件:")
    print(f"   PNG: {PNG_DIR}/")
    print(f"   ICO: {ICO_DIR}/canview.ico")
    print(f"\n🚀 下一步:")
    print(f"   cd ..")
    print(f"   cargo build --release")
    print(f"\n📝 生成的尺寸:")
    for size in SIZES:
        png_file = PNG_DIR / f"logo_{size}.png"
        if png_file.exists():
            file_size = png_file.stat().st_size
            print(f"   ✓ {size:4d}x{size:<4d} - {file_size:,} bytes")
    print()

    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except KeyboardInterrupt:
        print("\n\n⚠️  用户取消")
        sys.exit(1)
    except Exception as e:
        print(f"\n❌ 发生错误: {e}")
        import traceback

        traceback.print_exc()
        sys.exit(1)
