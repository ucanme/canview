// 测试中文输入验证逻辑
//
// 这个文件用于验证当前的字符验证逻辑是否正确支持中文

// 当前 library_view.rs 中的验证逻辑
fn is_valid_char_library_view(c: char) -> bool {
    !c.is_control() && (c.is_ascii_alphanumeric() || c == ' ' || !c.is_ascii())
}

// 推荐：使用 TextInputValidation::LibraryName 的验证逻辑
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TextInputValidation {
    LibraryName,
    VersionName,
    None,
}

impl TextInputValidation {
    pub fn is_valid_char(&self, ch: char) -> bool {
        match self {
            TextInputValidation::LibraryName => {
                // Support Chinese, English, numbers, spaces, and any Unicode
                !ch.is_control() && (ch.is_ascii_alphanumeric() || ch == ' ' || !ch.is_ascii())
            }
            TextInputValidation::VersionName => {
                // Only ASCII alphanumeric, dot, underscore, hyphen
                ch.is_ascii_alphanumeric() || ch == '.' || ch == '_' || ch == '-'
            }
            TextInputValidation::None => !ch.is_control(),
        }
    }
}

fn main() {
    println!("=== 测试中文字符验证 ===\n");

    // 测试中文字符
    let chinese_chars = vec!['测', '试', '中', '文', '你', '好', '库', '名'];

    println!("1. 测试单个中文字符:");
    for ch in chinese_chars {
        let result = is_valid_char_library_view(ch);
        println!("  '{}' -> {}", ch, result);
    }

    // 测试中文字符串
    let chinese_strings = vec!["测试", "中文", "测试CAN信号库", "你好世界", "库名称"];

    println!("\n2. 测试中文字符串:");
    for s in chinese_strings {
        let all_valid = s.chars().all(is_valid_char_library_view);
        println!("  '{}' -> {}", s, all_valid);
    }

    // 测试混合输入
    let mixed_strings = vec![
        ("Test测试123", "英文+中文+数字"),
        ("CAN总线测试", "英文+中文"),
        ("2024新版本", "数字+中文"),
        ("📊 数据分析库", "Emoji+中文"),
    ];

    println!("\n3. 测试混合输入:");
    for (s, desc) in mixed_strings {
        let all_valid = s.chars().all(is_valid_char_library_view);
        println!("  '{}' ({}) -> {}", s, desc, all_valid);
    }

    // 测试不应该接受的字符
    let invalid_chars = vec!['\n', '\t', '\r', '\x00'];

    println!("\n4. 测试控制字符（应该被拒绝）:");
    for ch in invalid_chars {
        let result = is_valid_char_library_view(ch);
        let display = if ch == '\n' {
            "\\n"
        } else if ch == '\t' {
            "\\t"
        } else if ch == '\r' {
            "\\r"
        } else {
            "\\x00"
        };
        println!("  '{}' -> {}", display, result);
    }

    // 测试 ASCII 字符
    let ascii_chars = vec!['a', 'Z', '0', '9', ' ', '-', '.'];

    println!("\n5. 测试 ASCII 字符:");
    for ch in ascii_chars {
        let result = is_valid_char_library_view(ch);
        println!("  '{}' -> {}", ch, result);
    }

    // 验证逻辑分析
    println!("\n=== 验证逻辑分析 ===");
    println!("\n当前验证逻辑:");
    println!("  !c.is_control() && (c.is_ascii_alphanumeric() || c == ' ' || !c.is_ascii())");
    println!("\n分解:");
    println!("  1. !c.is_control()        - 拒绝控制字符");
    println!("  2. c.is_ascii_alphanumeric() - 接受 ASCII 字母数字");
    println!("  3. c == ' '               - 接受空格");
    println!("  4. !c.is_ascii()          - 接受所有非 ASCII 字符（包括中文）");

    println!("\n对于中文字符 '测':");
    let ch = '测';
    println!("  is_control(): {}", ch.is_control());
    println!("  is_ascii_alphanumeric(): {}", ch.is_ascii_alphanumeric());
    println!("  is_ascii(): {}", ch.is_ascii());
    println!("  !is_ascii(): {}", !ch.is_ascii());
    println!("  最终结果: {}", is_valid_char_library_view(ch));

    println!("\n=== 结论 ===");
    println!("如果所有测试都显示 true，则验证逻辑是正确的。");
    println!("如果中文仍然无法输入，问题可能在于：");
    println!("  1. IME 输入事件处理");
    println!("  2. GPUI 的键盘事件捕获");
    println!("  3. 输入法组合窗口显示");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chinese_characters() {
        let validation = TextInputValidation::LibraryName;

        // 中文字符应该被接受
        assert!(validation.is_valid_char('测'), "中文字符 '测' 应该被接受");
        assert!(validation.is_valid_char('试'), "中文字符 '试' 应该被接受");
        assert!(validation.is_valid_char('中'), "中文字符 '中' 应该被接受");
        assert!(validation.is_valid_char('文'), "中文字符 '文' 应该被接受");
    }

    #[test]
    fn test_chinese_strings() {
        let validation = TextInputValidation::LibraryName;

        // 中文字符串应该被接受
        let strings = vec!["测试", "中文", "测试CAN信号库", "你好世界"];

        for s in strings {
            assert!(
                s.chars().all(|c| validation.is_valid_char(c)),
                "字符串 '{}' 应该被完全接受",
                s
            );
        }
    }

    #[test]
    fn test_mixed_input() {
        let validation = TextInputValidation::LibraryName;

        // 混合输入应该被接受
        let mixed = "Test测试123";
        assert!(
            mixed.chars().all(|c| validation.is_valid_char(c)),
            "混合输入 '{}' 应该被接受",
            mixed
        );
    }
}
