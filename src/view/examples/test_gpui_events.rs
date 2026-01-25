// 测试 GPUI 支持的事件类型
//
// 编译这个文件来检查 GPUI 是否有 on_text 或其他处理 IME 的事件

use gpui::*;

fn test_available_events() -> Div {
    div()
        .id("test_input")
        .focusable()
        // 尝试不同的事件处理方法
        //
        // 方法 1: on_key_down (已知可以工作，但不捕获 IME)
        .on_key_down(|event, _window, cx| {
            eprintln!("on_key_down: {:?}", event);
        })
        // 方法 2: on_key_press (可能捕获 IME)
        //.on_key_press(|event, _window, cx| {
        //    eprintln!("on_key_press: {:?}", event);
        //})
        // 方法 3: on_text (专门处理文本输入)
        //.on_text(|text: &str, cx| {
        //    eprintln!("on_text: '{}'", text);
        //})
        // 方法 4: on_input (可能的名字)
        //.on_input(|input: &str, cx| {
        //    eprintln!("on_input: '{}'", input);
        //})
        // 方法 5: on_chars (可能的名字)
        //.on_chars(|chars: &[char], cx| {
        //    eprintln!("on_chars: {:?}", chars);
        //})
}

/*
使用说明：
=========

1. 取消注释上面的事件处理方法（一次一个）
2. 尝试编译：cargo check
3. 如果编译成功，说明 GPUI 支持该事件
4. 测试该事件是否能捕获 IME 输入

预期结果：
===========
- on_key_down: 编译成功 ✅，但不捕获 IME ❌
- on_key_press: 可能编译成功，可能捕获 IME ❓
- on_text: 可能编译成功，应该捕获 IME ❓
- on_input: 可能编译成功，可能捕获 IME ❓
- on_chars: 可能编译成功，可能捕获 IME ❓

找到能捕获 IME 的事件后，使用它替代 on_key_down 处理文本输入。
*/
