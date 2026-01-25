# Cursor Position Tracking Implementation

## Overview
Implemented full cursor position tracking for the library name text input field. The cursor now moves dynamically as users type, delete, or navigate with arrow keys.

## Changes Made

### 1. State Management (main.rs)
**Added cursor position field:**
```rust
struct CanViewApp {
    // ... other fields
    library_cursor_position: usize,  // NEW: Tracks cursor position in library name input
}
```

**Initialization:**
```rust
// In both CanViewApp::new() and new_app state
library_cursor_position: 0,
```

### 2. Rendering Updates (library_view.rs)

**Function signature updated:**
```rust
pub fn render_library_management_view(
    // ... existing parameters
    cursor_position: usize,  // NEW: Accept cursor position
    cx: &mut gpui::Context<crate::CanViewApp>
) -> impl IntoElement
```

**Cursor rendering logic:**
- **Empty text**: Cursor appears at the beginning before the placeholder text
- **With text**: Text is split at cursor position into `before_cursor` and `after_cursor` parts
- Cursor is rendered as a blue vertical line (2px × 14px) between the two text parts

```rust
// Split text at cursor position
let text = new_library_name.chars().collect::<Vec<_>>();
let cursor_pos = cursor_position.min(text.len());

let before_cursor: String = text[..cursor_pos].iter().collect();
let after_cursor: String = text[cursor_pos..].iter().collect();

// Render: [before_cursor] [cursor] [after_cursor]
```

### 3. Keyboard Event Handling (library_view.rs)

**Enhanced keyboard handlers:**

| Key | Action | Cursor Behavior |
|-----|--------|-----------------|
| **Character input** | Insert character at cursor position | Moves cursor forward (+1) |
| **Backspace** | Delete character before cursor | Moves cursor backward (-1) |
| **Enter** | Create library | Resets cursor to 0 |
| **Escape** | Cancel editing | Resets cursor to 0 |
| **Left Arrow** | Move cursor left | Cursor position -1 |
| **Right Arrow** | Move cursor right | Cursor position +1 |
| **Home** | Move to start | Cursor position = 0 |
| **End** | Move to end | Cursor position = text length |

**Implementation details:**
```rust
// Character insertion
let mut chars: Vec<char> = this.new_library_name.chars().collect();
chars.insert(this.library_cursor_position, ch);
this.new_library_name = chars.into_iter().collect();
this.library_cursor_position += 1;

// Backspace
if this.library_cursor_position > 0 {
    let mut chars: Vec<char> = this.new_library_name.chars().collect();
    chars.remove(this.library_cursor_position - 1);
    this.new_library_name = chars.into_iter().collect();
    this.library_cursor_position -= 1;
}

// Arrow keys
"left" => {
    if this.library_cursor_position > 0 {
        this.library_cursor_position -= 1;
    }
}
"right" => {
    let text_len = this.new_library_name.chars().count();
    if this.library_cursor_position < text_len {
        this.library_cursor_position += 1;
    }
}
```

### 4. Cancel Button Update
Cancel button now resets cursor position:
```rust
this.new_library_name = String::new();
this.library_cursor_position = 0;  // NEW: Reset cursor
```

## How It Works

### Text Splitting Algorithm
1. Convert text string to `Vec<char>` for proper Unicode character handling
2. Clamp cursor position to valid range `[0, text.len()]`
3. Split character array at cursor position
4. Render three parts: `[before_cursor]` `[cursor]` `[after_cursor]`

### Cursor Position Tracking
- Cursor position represents the **insertion point** (0 = before first character)
- Position 0: Cursor appears before all text
- Position N: Cursor appears after N characters
- Position text.len(): Cursor appears after all text

### Character Insertion
1. Convert text to `Vec<char>`
2. Insert new character at `cursor_position` index
3. Increment `cursor_position` by 1
4. Convert back to `String`

### Character Deletion
1. Check `cursor_position > 0`
2. Convert text to `Vec<char>`
3. Remove character at `cursor_position - 1` (character before cursor)
4. Decrement `cursor_position` by 1
5. Convert back to `String`

## Benefits

1. **Natural text editing behavior**: Cursor behaves like standard text editors
2. **Arrow key navigation**: Users can move cursor left/right
3. **Home/End support**: Quick navigation to line start/end
4. **Unicode support**: Character-based splitting works with multi-byte characters
5. **Visual feedback**: Blue cursor clearly shows current position

## Testing

### Test Cases:
1. **Basic typing**: Type characters one by one, cursor should follow
2. **Navigation**: Use left/right arrows to move through text
3. **Insertion**: Move cursor to middle, type - should insert at cursor position
4. **Deletion**: Move cursor, press backspace - should delete character before cursor
5. **Home/End**: Press Home/End to jump to start/end
6. **Reset**: Press Escape or click Cancel - cursor should reset to 0

## Files Modified

1. `src/view/src/main.rs` - Added cursor_position field and initialization
2. `src/view/src/library_view.rs` - Updated rendering and keyboard handling

## Compilation

Build successful with no errors:
```bash
cargo build --release
```

Executable: `target/release/view.exe`

## Next Steps (Optional Enhancements)

1. **Cursor blink animation**: Add visual blinking effect for better visibility
2. **Text selection**: Support Shift+Arrow keys for text selection
3. **Ctrl+Arrow navigation**: Jump by words instead of characters
4. **Delete key**: Support Delete key (delete character after cursor)
5. **Mouse positioning**: Click to position cursor
