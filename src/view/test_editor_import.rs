// 测试 GPUI 是否导出 Editor 组件

use gpui::*;

fn test_editor() {
    // 尝试使用 Editor
    let editor = Editor::new();
}

fn main() {
    println!("Testing GPUI Editor...");
    test_editor();
    println!("Success! Editor is available.");
}
