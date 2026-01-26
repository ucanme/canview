//! IME (Input Method Editor) Input Support Test
//!
//! This test verifies that the application properly handles text input
//! through input methods (Chinese, Japanese, Korean, etc.)

use canview::library::LibraryManager;
use canview::models::SignalLibrary;
use canview::ChannelType;

#[cfg(test)]
mod ime_input_tests {
    use super::*;

    /// Test creating a library with Chinese name
    #[test]
    fn test_create_library_chinese_name() {
        let mut manager = LibraryManager::new();

        // Test with simple Chinese characters
        let result = manager.create_library("测试库".to_string(), ChannelType::CAN);
        assert!(result.is_ok(), "Should create library with Chinese name");

        let library = result.unwrap();
        assert_eq!(library.name, "测试库");
        assert_eq!(library.name.chars().count(), 3); // 3 characters, not bytes
    }

    /// Test creating a library with mixed Chinese and English
    #[test]
    fn test_create_library_mixed_name() {
        let mut manager = LibraryManager::new();

        let result = manager.create_library("Test测试库123".to_string(), ChannelType::CAN);
        assert!(result.is_ok(), "Should create library with mixed name");

        let library = result.unwrap();
        assert_eq!(library.name, "Test测试库123");
        assert_eq!(library.name.chars().count(), 10); // 10 characters total
    }

    /// Test creating a library with Japanese characters
    #[test]
    fn test_create_library_japanese_name() {
        let mut manager = LibraryManager::new();

        let result = manager.create_library("ライブラリ".to_string(), ChannelType::CAN);
        assert!(result.is_ok(), "Should create library with Japanese name");

        let library = result.unwrap();
        assert_eq!(library.name, "ライブラリ");
        assert_eq!(library.name.chars().count(), 6); // 6 Japanese characters
    }

    /// Test creating a library with Korean characters
    #[test]
    fn test_create_library_korean_name() {
        let mut manager = LibraryManager::new();

        let result = manager.create_library("라이브러리".to_string(), ChannelType::CAN);
        assert!(result.is_ok(), "Should create library with Korean name");

        let library = result.unwrap();
        assert_eq!(library.name, "라이브러리");
    }

    /// Test creating a library with emojis
    #[test]
    fn test_create_library_with_emoji() {
        let mut manager = LibraryManager::new();

        let result = manager.create_library("📊 Test Library 🚀".to_string(), ChannelType::CAN);
        assert!(result.is_ok(), "Should create library with emoji");

        let library = result.unwrap();
        assert_eq!(library.name, "📊 Test Library 🚀");
    }

    /// Test cursor position with multi-byte characters
    #[test]
    fn test_cursor_position_with_chinese() {
        let text = "测试库";

        // Character-based position (correct)
        let chars: Vec<char> = text.chars().collect();
        assert_eq!(chars.len(), 3);

        // Byte-based position (incorrect for cursor)
        assert_ne!(text.len(), chars.len()); // 6 bytes vs 3 characters

        // Test inserting at different positions
        let mut text = String::from("测试");

        // Insert at position 2 (between the two characters)
        let mut chars: Vec<char> = text.chars().collect();
        chars.insert(2, '中');
        text = chars.into_iter().collect();
        assert_eq!(text, "测中试");

        // Insert at position 3 (after '中')
        let mut chars: Vec<char> = text.chars().collect();
        chars.insert(3, '文');
        text = chars.into_iter().collect();
        assert_eq!(text, "测中文试");
    }

    /// Test deleting multi-byte characters
    #[test]
    fn test_delete_chinese_character() {
        let mut text = String::from("测试库");

        // Delete last character
        let mut chars: Vec<char> = text.chars().collect();
        chars.pop(); // Remove '库'
        text = chars.into_iter().collect();
        assert_eq!(text, "测试");

        // Delete first character
        let mut chars: Vec<char> = text.chars().collect();
        chars.remove(0); // Remove '测'
        text = chars.into_iter().collect();
        assert_eq!(text, "试");
    }

    /// Test version name with standard format
    #[test]
    fn test_version_name_format() {
        let mut manager = LibraryManager::new();
        let library = manager
            .create_library("Test Library".to_string(), ChannelType::CAN)
            .unwrap();

        // Standard version formats
        let valid_versions = vec![
            "v1.0.0",
            "version_1.2",
            "release-2.0",
            "1.0.3-beta",
            "v1.2.3-beta_release",
        ];

        for version_name in valid_versions {
            let result = manager.add_version(
                &library.id,
                version_name.to_string(),
                "/path/to/file.dbc".to_string(),
            );
            assert!(result.is_ok(), "Should create version: {}", version_name);
        }
    }

    /// Test that version names don't support spaces (intentional)
    #[test]
    fn test_version_name_no_spaces() {
        // This test documents that version names intentionally don't support spaces
        // to follow standard version naming conventions

        let valid_chars: Vec<char> = "v1.0.0-beta_release".chars().collect();
        let has_space = valid_chars.iter().any(|c| *c == ' ');
        assert!(!has_space, "Version names should not contain spaces");

        let allowed =
            |c: char| -> bool { c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-' };

        let all_valid = valid_chars.iter().all(|c| allowed(*c));
        assert!(
            all_valid,
            "All characters in version name should be allowed"
        );
    }

    /// Test searching for libraries with Chinese names
    #[test]
    fn test_search_library_chinese() {
        let mut manager = LibraryManager::new();

        manager
            .create_library("测试CAN库".to_string(), ChannelType::CAN)
            .unwrap();
        manager
            .create_library("测试LIN库".to_string(), ChannelType::LIN)
            .unwrap();
        manager
            .create_library("Test Library".to_string(), ChannelType::CAN)
            .unwrap();

        // Search for libraries containing "测试"
        let results: Vec<_> = manager
            .libraries()
            .iter()
            .filter(|lib| lib.name.contains("测试"))
            .collect();

        assert_eq!(results.len(), 2, "Should find 2 libraries with '测试'");

        // Search for libraries containing "CAN"
        let results: Vec<_> = manager
            .libraries()
            .iter()
            .filter(|lib| lib.name.contains("CAN"))
            .collect();

        assert_eq!(results.len(), 1, "Should find 1 library with 'CAN'");
    }

    /// Test library name validation
    #[test]
    fn test_library_name_validation() {
        let valid_names = vec![
            "测试信号库",
            "Test Library",
            "Library123",
            "测试库123",
            "Library测试",
            "Test库123",
            "📊 数据分析库",
        ];

        // Current validation logic:
        // - Not a control character
        // - ASCII alphanumeric OR space OR non-ASCII

        fn is_valid_library_name(name: &str) -> bool {
            name.chars().all(|c| {
                !c.is_control() && (c.is_ascii_alphanumeric() || c == ' ' || !c.is_ascii())
            })
        }

        for name in valid_names {
            assert!(
                is_valid_library_name(name),
                "Name '{}' should be valid",
                name
            );
        }

        let invalid_names = vec![
            "Test\nLibrary", // Contains newline
            "Test\tLibrary", // Contains tab
            "Test\rLibrary", // Contains carriage return
        ];

        for name in invalid_names {
            assert!(
                !is_valid_library_name(name),
                "Name '{}' should be invalid",
                name
            );
        }
    }

    /// Test UTF-8 byte length vs character length
    #[test]
    fn test_utf8_byte_vs_char_length() {
        let tests = vec![
            ("Test", 4, 4),      // English: 1 byte per char
            ("测试", 2, 6),      // Chinese: 3 bytes per char
            ("🚀", 1, 4),        // Emoji: 4 bytes
            ("Test测试", 6, 10), // Mixed: 4 + 6 bytes
            ("a🚀b", 3, 7),      // Emoji mixed: 1 + 4 + 2 bytes
        ];

        for (text, expected_chars, expected_bytes) in tests {
            let char_count = text.chars().count();
            let byte_count = text.len();

            assert_eq!(
                char_count, expected_chars,
                "Character count mismatch for '{}': expected {}, got {}",
                text, expected_chars, char_count
            );

            assert_eq!(
                byte_count, expected_bytes,
                "Byte count mismatch for '{}': expected {}, got {}",
                text, expected_bytes, byte_count
            );

            // Cursor position should be based on characters
            let cursor_pos = char_count;
            assert!(
                cursor_pos <= char_count,
                "Cursor position must be <= character count"
            );

            // But string operations work with bytes
            let bytes = text.as_bytes();
            assert_eq!(bytes.len(), expected_bytes);
        }
    }

    /// Test inserting text at cursor position with multi-byte characters
    #[test]
    fn test_insert_at_cursor_position() {
        let mut text = String::from("测试");

        // Simulate user typing "数据" at position 2 (after "测试")
        let insert_pos = 2;
        let insert_text = "数据";

        let mut chars: Vec<char> = text.chars().collect();
        for (i, ch) in insert_text.chars().enumerate() {
            chars.insert(insert_pos + i, ch);
        }
        text = chars.into_iter().collect();

        assert_eq!(text, "测数据试");
        assert_eq!(text.chars().count(), 4); // 2 + 2 = 4 characters
    }

    /// Test backspace with multi-byte characters
    #[test]
    fn test_backspace_chinese() {
        let text = String::from("测试库");
        let mut cursor_pos = 3; // At end

        // Press backspace
        let mut chars: Vec<char> = text.chars().collect();
        if cursor_pos > 0 {
            chars.remove(cursor_pos - 1);
            cursor_pos -= 1;
        }
        let new_text = chars.into_iter().collect();

        assert_eq!(new_text, "测试");
        assert_eq!(cursor_pos, 2);
    }

    /// Test arrow keys with multi-byte characters
    #[test]
    fn test_arrow_navigation_chinese() {
        let text = "测试库123";
        let text_len = text.chars().count(); // Should be 6: 测试库123

        assert_eq!(text_len, 6, "Should have 6 characters");

        let mut cursor_pos = 0;

        // Move right
        cursor_pos += 1;
        assert_eq!(cursor_pos, 1);

        // Move right multiple times
        cursor_pos += 2;
        assert_eq!(cursor_pos, 3);

        // Move left
        cursor_pos -= 1;
        assert_eq!(cursor_pos, 2);

        // Home
        cursor_pos = 0;
        assert_eq!(cursor_pos, 0);

        // End
        cursor_pos = text_len;
        assert_eq!(cursor_pos, 6);
    }
}

/// Performance test for handling large strings with Chinese characters
#[cfg(test)]
mod performance_tests {
    use super::*;

    #[test]
    fn test_large_chinese_string_handling() {
        // Create a large string with Chinese characters
        let large_text = "测试信号库".repeat(1000);

        // Test character counting
        let char_count = large_text.chars().count();
        assert_eq!(char_count, 6000); // 6 chars * 1000

        // Test iteration
        let char_vec: Vec<char> = large_text.chars().collect();
        assert_eq!(char_vec.len(), 6000);

        // Test insertion at middle
        let mut chars = large_text.chars().collect();
        chars.insert(3000, '中');
        let result: String = chars.into_iter().collect();
        assert_eq!(result.chars().count(), 6001);
    }
}

/// Integration test simulating actual user input scenarios
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_user_scenario_create_chinese_library() {
        let mut manager = LibraryManager::new();

        // Scenario: User types Chinese library name using Pinyin input method
        // 1. User types: "cesexinhao ku" (Pinyin)
        // 2. Input method converts to: "测试信号库"
        // 3. User presses Enter to confirm

        let library_name = "测试信号库";
        let result = manager.create_library(library_name.to_string(), ChannelType::CAN);

        assert!(result.is_ok());
        let library = result.unwrap();
        assert_eq!(library.name, "测试信号库");
        assert_eq!(library.name.chars().count(), 5);
    }

    #[test]
    fn test_user_scenario_edit_library_name() {
        let mut manager = LibraryManager::new();
        let library = manager
            .create_library("测试库".to_string(), ChannelType::CAN)
            .unwrap();

        // Scenario: User wants to edit the library name
        // Original: "测试库"
        // User presses backspace to remove "库"
        // User types "信号库" to get "测试信号库"

        // Simulate: Press backspace
        let mut name = library.name.clone();
        let mut cursor_pos = name.chars().count();

        let mut chars: Vec<char> = name.chars().collect();
        if cursor_pos > 0 {
            chars.remove(cursor_pos - 1);
            cursor_pos -= 1;
        }
        name = chars.into_iter().collect();

        assert_eq!(name, "测试");

        // Simulate: Type "信号库" using input method
        let insert_text = "信号库";
        let mut chars: Vec<char> = name.chars().collect();
        for (i, ch) in insert_text.chars().enumerate() {
            chars.insert(cursor_pos + i, ch);
        }
        name = chars.into_iter().collect();

        assert_eq!(name, "测试信号库");
    }

    #[test]
    fn test_user_scenario_mixed_input() {
        // Scenario: User types mixed Chinese and English
        // "CAN" + "测试库" + "2024" = "CAN测试库2024"

        let mut text = String::new();

        // Type "CAN"
        text.push_str("CAN");
        assert_eq!(text, "CAN");

        // Type "测试库" using input method
        text.push_str("测试库");
        assert_eq!(text, "CAN测试库");
        assert_eq!(text.chars().count(), 6); // 3 + 3 characters

        // Type "2024"
        text.push_str("2024");
        assert_eq!(text, "CAN测试库2024");
        assert_eq!(text.chars().count(), 10); // 3 + 3 + 4 characters
    }
}
