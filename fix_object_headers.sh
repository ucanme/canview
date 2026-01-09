#!/bin/bash
# 批量修复ObjectHeader测试代码的脚本

# 查找所有包含ObjectHeader初始化且缺少client_index和object_version的文件
# 并自动添加这两个字段

cd "$(dirname "$0")"

files_to_fix=(
    "src/blf/src/file.rs"
    "src/blf/src/parser.rs"
    "src/blf/src/bin/gen_test_blf.rs"
)

echo "开始批量修复ObjectHeader测试代码..."

for file in "${files_to_fix[@]}"; do
    if [ -f "$file" ]; then
        echo "处理文件: $file"
        # 使用sed添加缺失的字段（在object_flags行之后）
        sed -i '/object_flags:.*,$/a\            client_index: 0,\n            object_version: 0,' "$file"
        echo "  ✓ 已修复"
    else
        echo "  ✗ 文件不存在: $file"
    fi
done

echo "完成！现在运行测试..."
cargo test --package blf --lib 2>&1 | tail -30
