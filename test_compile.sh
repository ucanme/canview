#!/bin/bash
# Minimal compilation test for canview

echo "=== CanView Compilation Test ==="
echo ""

cd /mnt/c/Users/Administrator/RustroverProjects/canview

echo "Step 1: Check if text_input_v2.rs exists..."
if [ -f "src/view/src/ui/components/text_input_v2.rs" ]; then
    echo "✅ text_input_v2.rs exists"
else
    echo "❌ text_input_v2.rs NOT found"
    exit 1
fi

echo ""
echo "Step 2: Check mod.rs exports..."
if grep -q "pub mod text_input_v2" src/view/src/ui/components/mod.rs; then
    echo "✅ text_input_v2 module declared"
else
    echo "❌ text_input_v2 module NOT declared in mod.rs"
    exit 1
fi

echo ""
echo "Step 3: Check library_view.rs imports..."
if grep -q "use crate::ui::components::text_input_v2" src/view/src/library_view.rs; then
    echo "✅ library_view.rs imports text_input_v2"
else
    echo "❌ library_view.rs does NOT import text_input_v2"
    exit 1
fi

echo ""
echo "Step 4: Check Cargo.toml binary config..."
if grep -q '\[\[bin\]\]' src/view/Cargo.toml; then
    echo "✅ Cargo.toml has [[bin]] configuration"
else
    echo "⚠️  Cargo.toml missing [[bin]] configuration (adding it now)"
    echo "" >> src/view/Cargo.toml
    echo "[[bin]]" >> src/view/Cargo.toml
    echo "name = \"view\"" >> src/view/Cargo.toml
    echo "path = \"src/main.rs\"" >> src/view/Cargo.toml
fi

echo ""
echo "Step 5: Clean previous build..."
cargo clean -p view

echo ""
echo "Step 6: Build view package..."
if cargo build -p view --release 2>&1 | tee build.log; then
    echo ""
    echo "✅✅✅ BUILD SUCCESSFUL ✅✅✅"
    echo ""

    # Check if executable was created
    if [ -f "target/release/view.exe" ]; then
        echo "✅ Executable created: target/release/view.exe"
        ls -lh target/release/view.exe
    else
        echo "⚠️  Executable not found in target/release/"
        echo "   Looking in subdirectories..."
        find target -name "view.exe" 2>/dev/null
    fi

    echo ""
    echo "To run the application:"
    echo "  cargo run -p view --release"
    echo ""
else
    echo ""
    echo "❌❌❌ BUILD FAILED ❌❌❌"
    echo ""
    echo "Check build.log for details"
    echo ""
    tail -50 build.log
    exit 1
fi
