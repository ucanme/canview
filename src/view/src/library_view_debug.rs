// 调试版本的中文输入处理
//
// 将此代码复制到 library_view.rs 中的对应位置来调试中文输入问题

// 在 on_key_down 处理函数开头添加：
move |event, _window, cx| {
    let keystroke = format!("{}", event.keystroke);
    let key_text = event.keystroke.key.as_str();

    // ========== 调试输出 ==========
    eprintln!("=== KEY DOWN EVENT DEBUG ===");
    eprintln!("keystroke (Display): '{}'", keystroke);
    eprintln!("keystroke len: {}", keystroke.len());
    eprintln!("keystroke bytes: {:?}", keystroke.as_bytes());
    eprintln!("key_text: '{}'", key_text);
    eprintln!("keystroke chars count: {}", keystroke.chars().count());
    eprintln!("keystroke chars: {:?}", keystroke.chars().collect::<Vec<_>>());

    // 检查每个字符的属性
    for (i, ch) in keystroke.chars().enumerate() {
        eprintln!("  char[{}] '{}' => is_control: {}, is_ascii: {}, is_ascii_alphanumeric: {}",
            i, ch, ch.is_control(), ch.is_ascii(), ch.is_ascii_alphanumeric());
    }

    // 检查是否是中文
    let has_chinese = keystroke.chars().any(|c| !c.is_ascii() && !c.is_control());
    eprintln!("has_chinese: {}", has_chinese);
    eprintln!("=============================\n");

    // 原有的处理逻辑
    match keystroke.as_str() {
        "backspace" => {
            // ... backspace 处理
        }
        "enter" => {
            // ... enter 处理
        }
        "escape" => {
            // ... escape 处理
        }
        _ => {
            // 字符输入处理
            let is_printable = if keystroke.len() == 1 {
                keystroke.chars().next().map(|c| !c.is_control()).unwrap_or(false)
            } else if keystroke.len() > 1 {
                !keystroke.to_lowercase().starts_with("backspace")
                    && !keystroke.to_lowercase().starts_with("delete")
                    && !keystroke.to_lowercase().starts_with("left")
                    && !keystroke.to_lowercase().starts_with("right")
                    && !keystroke.to_lowercase().starts_with("up")
                    && !keystroke.to_lowercase().starts_with("down")
                    && !keystroke.to_lowercase().starts_with("home")
                    && !keystroke.to_lowercase().starts_with("end")
                    && keystroke.chars().all(|c| !c.is_control())
            } else {
                false
            };

            eprintln!("is_printable: {}", is_printable);

            if is_printable {
                let is_valid_char = |c: char| -> bool {
                    !c.is_control() && (c.is_ascii_alphanumeric() || c == ' ' || !c.is_ascii())
                };

                let all_valid = keystroke.chars().all(is_valid_char);
                eprintln!("all_valid: {}", all_valid);

                if all_valid {
                    eprintln!("ACCEPTING INPUT: '{}'", keystroke);
                    view.update(cx, |this, cx| {
                        let mut chars: Vec<char> = this.new_library_name.chars().collect();
                        for (i, ch) in keystroke.chars().enumerate() {
                            chars.insert(this.library_cursor_position + i, ch);
                        }
                        this.new_library_name = chars.into_iter().collect();
                        this.library_cursor_position += keystroke.chars().count();
                        eprintln!("Library name is now: '{}'", this.new_library_name);
                        cx.notify();
                    });
                } else {
                    eprintln!("REJECTED: Not all characters are valid");
                }
            } else {
                eprintln!("REJECTED: Not printable");
            }
        }
    }
}

// ============================================================================
// 使用说明：
// ============================================================================
//
// 1. 将上述代码复制到 library_view.rs 中的 on_key_down 处理函数
// 2. 运行应用
// 3. 尝试输入中文
// 4. 查看终端输出的调试信息
// 5. 根据输出分析问题
//
// 预期输出（如果正常）：
//   当你输入 "测试" 时，应该看到：
//   - keystroke: '测试'
//   - keystroke len: 2 或更多（取决于 UTF-8 编码）
//   - has_chinese: true
//   - is_printable: true
//   - all_valid: true
//   - ACCEPTING INPUT
//
// 可能的问题：
// 1. 如果 keystroke 只显示单个字母（如 'c'），说明 GPUI 没有正确处理 IME
// 2. 如果 has_chinese 总是 false，说明中文没有通过 keystroke 传递
// 3. 如果 all_valid 是 false，检查验证逻辑
//
// ============================================================================
