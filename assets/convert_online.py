#!/usr/bin/env python3
"""
CANVIEW Icon Conversion Script
使用在线API转换SVG到PNG，然后生成ICO文件
"""

import json
import os
import subprocess
import sys
import tempfile
import urllib.request
import webbrowser
from pathlib import Path

# Configuration
SCRIPT_DIR = Path(__file__).parent
LOGO_SVG = SCRIPT_DIR / "svg" / "logo.svg"
PNG_DIR = SCRIPT_DIR / "png"
ICO_DIR = SCRIPT_DIR / "ico"
SIZES = [512, 256, 128, 64, 48, 32, 16]


def check_local_tools():
    """检查是否有可用的本地工具"""
    tools = {
        "inkscape": False,
        "magick": False,
        "convert": False,
    }

    # 检查inkscape
    try:
        result = subprocess.run(
            ["inkscape", "--version"], capture_output=True, text=True, timeout=5
        )
        if result.returncode == 0:
            tools["inkscape"] = True
            print(f"✅ 找到 Inkscape: {result.stdout.strip()}")
    except:
        pass

    # 检查ImageMagick
    try:
        result = subprocess.run(
            ["magick", "--version"], capture_output=True, text=True, timeout=5
        )
        if result.returncode == 0:
            tools["magick"] = True
            print(f"✅ 找到 ImageMagick")
    except:
        pass

    try:
        result = subprocess.run(
            ["convert", "--version"], capture_output=True, text=True, timeout=5
        )
        if result.returncode == 0:
            tools["convert"] = True
            print(f"✅ 找到 ImageMagick (convert)")
    except:
        pass

    return tools


def convert_with_inkscape():
    """使用Inkscape转换SVG"""
    print("\n🔄 使用 Inkscape 转换...")

    PNG_DIR.mkdir(exist_ok=True)
    ICO_DIR.mkdir(exist_ok=True)

    if not LOGO_SVG.exists():
        print(f"❌ 找不到文件: {LOGO_SVG}")
        return False

    try:
        # 转换各个尺寸
        for i, size in enumerate(SIZES, 1):
            print(f"   [{i}/{len(SIZES)}] 生成 {size}x{size} PNG...", end=" ")
            png_path = PNG_DIR / f"logo_{size}.png"

            result = subprocess.run(
                [
                    "inkscape",
                    str(LOGO_SVG),
                    "--export-type=png",
                    f"--export-filename={png_path}",
                    f"--export-width={size}",
                    f"--export-height={size}",
                ],
                capture_output=True,
                timeout=30,
            )

            if result.returncode == 0 and png_path.exists():
                print("✓")
            else:
                print(f"✗")
                return False

        print("✅ PNG转换完成!")
        return True

    except Exception as e:
        print(f"❌ Inkscape转换失败: {e}")
        return False


def convert_with_imagemagick():
    """使用ImageMagick转换SVG"""
    print("\n🔄 使用 ImageMagick 转换...")

    PNG_DIR.mkdir(exist_ok=True)
    ICO_DIR.mkdir(exist_ok=True)

    if not LOGO_SVG.exists():
        print(f"❌ 找不到文件: {LOGO_SVG}")
        return False

    try:
        cmd = (
            "magick"
            if subprocess.run(["magick", "--version"], capture_output=True).returncode
            == 0
            else "convert"
        )

        # 转换各个尺寸
        for i, size in enumerate(SIZES, 1):
            print(f"   [{i}/{len(SIZES)}] 生成 {size}x{size} PNG...", end=" ")
            png_path = PNG_DIR / f"logo_{size}.png"

            result = subprocess.run(
                [
                    cmd,
                    "-background",
                    "none",
                    str(LOGO_SVG),
                    "-resize",
                    f"{size}x{size}",
                    str(png_path),
                ],
                capture_output=True,
                timeout=30,
            )

            if result.returncode == 0 and png_path.exists():
                print("✓")
            else:
                print(f"✗")
                return False

        print("✅ PNG转换完成!")
        return True

    except Exception as e:
        print(f"❌ ImageMagick转换失败: {e}")
        return False


def create_ico_with_pillow():
    """使用Pillow创建ICO文件"""
    print("\n🔄 创建 ICO 文件...")

    try:
        from PIL import Image
    except ImportError:
        print("❌ 需要安装 Pillow: pip install pillow")
        return False

    try:
        # 收集PNG文件
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


def open_online_converter():
    """打开在线转换工具"""
    print("\n🌐 正在打开在线转换工具...")
    print("\n📋 按照以下步骤操作：\n")

    print("步骤 1: SVG 转 PNG")
    print("   1. 在新打开的网页中上传: svg/logo.svg")
    print("   2. 选择或输入以下尺寸（多个）:")
    for size in SIZES:
        print(f"      - {size}x{size}")
    print("   3. 下载所有PNG文件，保存到 assets/png/ 目录")
    print("   4. 文件命名为: logo_16.png, logo_32.png, ..., logo_512.png")

    print("\n步骤 2: 运行ICO生成脚本")
    print("   在assets目录运行: python convert_online.py --ico-only")

    # 打开在线转换工具
    urls = [
        "https://cloudconvert.com/svg-to-png",
        "https://convertio.co/svg-png/",
        "https://www.aconvert.com/image/svg-to-png/",
    ]

    print(f"\n正在打开转换工具...")
    for url in urls:
        print(f"   - {url}")

    # 打开第一个（最常用的）
    webbrowser.open(urls[0])

    return True


def main():
    """主函数"""
    print("=" * 70)
    print("CANVIEW 图标转换工具")
    print("=" * 70)
    print()

    # 检查命令行参数
    if len(sys.argv) > 1 and sys.argv[1] == "--ico-only":
        # 只生成ICO
        return 0 if create_ico_with_pillow() else 1

    # 检查本地工具
    print("🔍 检查可用的转换工具...")
    tools = check_local_tools()

    success = False

    # 尝试使用本地工具
    if tools["inkscape"]:
        success = convert_with_inkscape()
    elif tools["magick"] or tools["convert"]:
        success = convert_with_imagemagick()

    if success:
        # 创建ICO文件
        if create_ico_with_pillow():
            print("\n" + "=" * 70)
            print("✅ 转换完成!")
            print("=" * 70)
            print(f"\n📁 生成的文件:")
            print(f"   PNG: {PNG_DIR}/")
            print(f"   ICO: {ICO_DIR}/canview.ico")
            print(f"\n🚀 下一步:")
            print(f"   cd ..")
            print(f"   cargo build --release")
            print()
            return 0
        else:
            print("\n⚠️  PNG转换成功，但ICO创建失败")
            return 1
    else:
        # 没有本地工具，使用在线工具
        print("\n❌ 未找到本地转换工具")
        return 0 if open_online_converter() else 1


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
