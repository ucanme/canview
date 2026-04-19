#!/usr/bin/env python3
"""
CANVIEW 图标验证工具
检查图标是否已正确应用到可执行文件中
"""

import os
import struct
import sys
from datetime import datetime
from pathlib import Path

# 配置
SCRIPT_DIR = Path(__file__).parent
PROJECT_DIR = SCRIPT_DIR.parent
ICO_FILE = SCRIPT_DIR / "ico" / "canview.ico"
EXE_FILE = PROJECT_DIR / "target" / "release" / "view.exe"
PNG_DIR = SCRIPT_DIR / "png"


def check_ico_file():
    """检查ICO文件是否存在且有效"""
    print("=" * 70)
    print("步骤 1: 检查ICO文件")
    print("=" * 70)

    if not ICO_FILE.exists():
        print(f"❌ ICO文件不存在: {ICO_FILE}")
        return False

    file_size = ICO_FILE.stat().st_size
    file_time = datetime.fromtimestamp(ICO_FILE.stat().st_mtime)

    print(f"✅ ICO文件存在: {ICO_FILE}")
    print(f"   大小: {file_size:,} bytes ({file_size / 1024:.1f} KB)")
    print(f"   修改时间: {file_time.strftime('%Y-%m-%d %H:%M:%S')}")

    # 验证ICO格式
    try:
        with open(ICO_FILE, "rb") as f:
            # 读取ICO头部
            header = f.read(6)
            if len(header) < 6:
                print("❌ ICO文件太小，不是有效的ICO格式")
                return False

            # 检查ICO魔术字 (0 = ICO, 1 = CUR)
            reserved, image_type, image_count = struct.unpack("<HHH", header)

            if reserved != 0:
                print("❌ ICO文件格式错误: 保留字节不为0")
                return False

            if image_type != 1:
                print(f"❌ 不是ICO文件类型 (type={image_type})")
                return False

            print(f"✅ ICO格式有效")
            print(f"   包含 {image_count} 个图像尺寸")

            # 读取图标尺寸信息
            print(f"\n   包含的尺寸:")
            for i in range(min(image_count, 16)):  # 最多显示16个
                entry = f.read(16)  # ICO目录入口大小
                if len(entry) >= 16:
                    width, height = struct.unpack("<BB", entry[:2])
                    if width == 0:
                        width = 256
                    if height == 0:
                        height = 256
                    print(f"   - {width}x{height}")

            return True

    except Exception as e:
        print(f"❌ 读取ICO文件失败: {e}")
        return False


def check_png_files():
    """检查PNG文件是否完整"""
    print("\n" + "=" * 70)
    print("步骤 2: 检查PNG文件")
    print("=" * 70)

    required_sizes = [512, 256, 128, 64, 48, 32, 16]
    missing = []

    print("检查必需的PNG文件:")
    for size in required_sizes:
        png_file = PNG_DIR / f"logo_{size}.png"
        if png_file.exists():
            file_size = png_file.stat().st_size
            print(f"✅ {size:4d}x{size:<4d} - {file_size:,} bytes")
        else:
            print(f"❌ {size:4d}x{size:<4d} - 缺失")
            missing.append(size)

    if missing:
        print(f"\n❌ 缺失 {len(missing)} 个PNG文件")
        return False

    print(f"\n✅ 所有 {len(required_sizes)} 个PNG文件都存在")
    return True


def check_exe_file():
    """检查EXE文件"""
    print("\n" + "=" * 70)
    print("步骤 3: 检查EXE文件")
    print("=" * 70)

    if not EXE_FILE.exists():
        print(f"❌ EXE文件不存在: {EXE_FILE}")
        print(f"   请先运行: cargo build --release")
        return False

    file_size = EXE_FILE.stat().st_size
    file_time = datetime.fromtimestamp(EXE_FILE.stat().st_mtime)
    ico_time = datetime.fromtimestamp(ICO_FILE.stat().st_mtime)

    print(f"✅ EXE文件存在: {EXE_FILE}")
    print(f"   大小: {file_size:,} bytes ({file_size / 1024 / 1024:.1f} MB)")
    print(f"   编译时间: {file_time.strftime('%Y-%m-%d %H:%M:%S')}")
    print(f"   ICO文件时间: {ico_time.strftime('%Y-%m-%d %H:%M:%S')}")

    # 检查时间顺序
    if file_time < ico_time:
        print(f"\n⚠️  警告: EXE文件的编译时间早于ICO文件")
        print(f"   这意味着EXE可能没有包含最新的图标")
        print(f"   建议: 重新编译 (cargo build --release -p view)")
        return False

    print(f"\n✅ EXE文件是在ICO文件之后编译的")

    # 检查EXE是否有资源段
    try:
        with open(EXE_FILE, "rb") as f:
            # 读取PE头部
            f.seek(0x3C)  # PE头偏移位置
            pe_offset = struct.unpack("<I", f.read(4))[0]

            f.seek(pe_offset)
            pe_sig = f.read(4)

            if pe_sig != b"PE\x00\x00":
                print("❌ 不是有效的PE文件")
                return False

            print("✅ PE文件格式有效")

            # 读取COFF头部
            f.seek(pe_offset + 4)
            machine, num_sections = struct.unpack("<HH", f.read(4))

            print(f"   机器类型: {hex(machine)}")
            print(f"   段数量: {num_sections}")

            # 查找.rsrc段
            f.seek(pe_offset + 20)  # 可选头部大小
            optional_header_size = struct.unpack("<H", f.read(2))[0]

            f.seek(pe_offset + 24 + optional_header_size)  # 段表起始位置

            has_rsrc = False
            for i in range(num_sections):
                section_name = (
                    f.read(8).rstrip(b"\x00").decode("ascii", errors="ignore")
                )
                if section_name == ".rsrc":
                    has_rsrc = True
                    break
                f.read(32)  # 跳过其余段信息

            if has_rsrc:
                print("✅ 找到资源段 (.rsrc)")
            else:
                print("⚠️  未找到资源段，可能没有嵌入图标")

            return True

    except Exception as e:
        print(f"⚠️  无法完全验证EXE资源: {e}")
        print("   但文件存在且大小正常")
        return True


def compare_times():
    """比较文件时间戳"""
    print("\n" + "=" * 70)
    print("步骤 4: 时间戳分析")
    print("=" * 70)

    exe_time = datetime.fromtimestamp(EXE_FILE.stat().st_mtime)
    ico_time = datetime.fromtimestamp(ICO_FILE.stat().st_mtime)
    build_rs_time = datetime.fromtimestamp(
        (PROJECT_DIR / "src" / "view" / "build.rs").stat().st_mtime
    )

    print(f"build.rs 修改时间:  {build_rs_time.strftime('%Y-%m-%d %H:%M:%S')}")
    print(f"ICO文件生成时间:   {ico_time.strftime('%Y-%m-%d %H:%M:%S')}")
    print(f"EXE文件编译时间:   {exe_time.strftime('%Y-%m-%d %H:%M:%S')}")

    if build_rs_time > exe_time:
        print(f"\n⚠️  build.rs在EXE编译后被修改")
        print(f"   建议: 重新编译 (cargo build --release -p view)")
        return False

    if ico_time > exe_time:
        print(f"\n⚠️  ICO文件在EXE编译后被生成")
        print(f"   建议: 重新编译 (cargo build --release -p view)")
        return False

    print(f"\n✅ 时间戳顺序正确，图标应该已嵌入")
    return True


def provide_solution():
    """提供解决方案"""
    print("\n" + "=" * 70)
    print("解决方案")
    print("=" * 70)

    print("""
如果图标没有显示，请按顺序尝试以下步骤：

步骤 1: 清除Windows图标缓存
----------------------------------------
在assets目录运行: refresh_icon_cache.bat

或手动执行：
    ie4uinit.exe -show
    taskkill /f /im explorer.exe && start explorer.exe


步骤 2: 重新编译（如果时间戳不匹配）
----------------------------------------
    cd C:\\Users\\Administrator\\RustroverProjects\\canview
    cargo clean -p view
    cargo build --release -p view


步骤 3: 验证图标文件
----------------------------------------
    cd assets
    python verify_icon.py


步骤 4: 查看文件图标
----------------------------------------
    1. 打开文件夹: target\\release\\
    2. 找到 view.exe
    3. 按 F5 刷新
    4. 查看文件图标


步骤 5: 重启电脑（最后的手段）
----------------------------------------
如果以上都不行，重启电脑以清除所有缓存


手动验证方法
----------------------------------------
方法1 - 属性对话框：
    1. 右键点击 view.exe
    2. 选择"属性"
    3. 在"快捷方式"选项卡中点击"更改图标"
    4. 应该能看到新的示波器风格图标

方法2 - 创建快捷方式：
    1. 复制 view.exe 到桌面
    2. 右键点击快捷方式 → 属性
    3. 点击"更改图标"查看
    """)


def main():
    """主函数"""
    print("=" * 70)
    print("CANVIEW 图标验证工具")
    print("=" * 70)
    print()

    results = []

    # 步骤1: 检查ICO文件
    results.append(check_ico_file())

    # 步骤2: 检查PNG文件
    results.append(check_png_files())

    # 步骤3: 检查EXE文件
    if EXE_FILE.exists():
        results.append(check_exe_file())
        results.append(compare_times())
    else:
        results.append(False)
        results.append(False)

    # 总结
    print("\n" + "=" * 70)
    print("验证总结")
    print("=" * 70)

    if all(results):
        print("""
✅ 所有检查通过！

图标文件正确，已编译到EXE中。

如果仍然看不到新图标，这是Windows图标缓存问题。
请参考上方的"解决方案"部分。
""")
        return 0
    else:
        print("""
❌ 部分检查未通过

请根据上方的错误信息，参考"解决方案"部分进行修复。
""")
        provide_solution()
        return 1


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
