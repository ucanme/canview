# File Menu Dropdown Feature

## Overview

The top navigation bar has been redesigned to follow Zed IDE's menu pattern. The "Logs" tab has been replaced with a "File" dropdown menu that contains file-related actions like "Open BLF...".

## Changes

### Before
```
[Logs] [Library] [Plot] [Open BLF]
```

### After
```
[File ▼] [Logs] [Library] [Plot]
```

Clicking "File" opens a dropdown menu:
```
┌─────────────┐
│ Open BLF... │
└─────────────┘
```

## Implementation Details

### State Management

A new state field has been added to track the dropdown menu visibility:

**File**: `src/view/src/app/state.rs`

```rust
pub struct CanViewApp {
    // ... existing fields ...
    
    // File menu dropdown state
    pub show_file_menu: bool,
}
```

### UI Components

The File menu is implemented using GPUI's relative/absolute positioning system:

**File**: `src/view/src/app/impls.rs`

1. **Menu Button** - A styled button that toggles the dropdown
2. **Dropdown Container** - Absolutely positioned div that appears when `show_file_menu` is true
3. **Menu Items** - Clickable items that trigger actions

### Key Features

1. **Toggle Behavior**: Click "File" to open/close the menu
2. **Click Outside**: Clicking anywhere outside the menu closes it
3. **Auto-close**: Menu automatically closes when:
   - You click a menu item
   - You click on another tab (Logs/Library/Plot)
   - You click outside the menu area

4. **Visual Styling**:
   - Background: `#1e1e2e` (dark theme)
   - Border: `#3a3a4a`
   - Rounded corners with shadow
   - Hover effect on menu items: `#313147`

## Usage

### Opening a BLF File

1. Click the **"File"** button in the top menu bar
2. Select **"Open BLF..."** from the dropdown
3. Choose your `.blf` or `.bin` file
4. The file loads and displays in the Logs view

### Keyboard Shortcuts

Currently, there are no keyboard shortcuts for the File menu. This feature can be extended to add:
- `Ctrl+O` / `Cmd+O` for "Open BLF..."
- `Escape` to close the menu
- Arrow keys to navigate menu items

## Code Structure

### Menu Button Event Handler

```rust
.on_mouse_down(gpui::MouseButton::Left, {
    let view = view.clone();
    move |_event, _, cx| {
        cx.stop_propagation();
        view.update(cx, |this, cx| {
            this.show_file_menu = !this.show_file_menu;
            cx.notify();
        });
    }
})
```

### Menu Item Event Handler

```rust
.on_mouse_down(gpui::MouseButton::Left, {
    let view = view.clone();
    move |_event, _, cx| {
        cx.stop_propagation();
        view.update(cx, |this, cx| {
            this.show_file_menu = false;  // Close menu
            cx.notify();
        });
        
        // Trigger action (e.g., open file dialog)
        // ... action code ...
    }
})
```

### Click Outside Handler

```rust
.on_mouse_down(gpui::MouseButton::Left, {
    let view = view.clone();
    move |_event, _, cx| {
        view.update(cx, |this, cx| {
            if this.show_file_menu {
                this.show_file_menu = false;
                cx.notify();
            }
        });
    }
})
```

## Future Enhancements

### Planned Features

1. **Additional Menu Items**
   - "Recent Files" - Show recently opened BLF files
   - "Export to CSV..." - Export current data
   - "Save Configuration" - Quick save config
   - "Load Configuration" - Quick load config

2. **Keyboard Shortcuts**
   - Alt+F to focus File menu
   - Arrow keys to navigate
   - Enter to select
   - Escape to close

3. **Menu Categories**
   - File menu (currently implemented)
   - Edit menu (if we add editing features)
   - View menu (display options)
   - Help menu (documentation)

4. **Recent Files List**
   ```
   ┌──────────────────┐
   │ Open BLF...      │
   │ ──────────────── │
   │ Recent Files:    │
   │   can.blf        │
   │   test.blf       │
   └──────────────────┘
   ```

### Example: Adding a New Menu Item

To add a new menu item like "Export...":

```rust
.child(
    div()
        .px_3()
        .py_2()
        .text_sm()
        .text_color(rgb(0xcdd6f4))
        .hover(|s| s.bg(rgb(0x313147)))
        .cursor_pointer()
        .on_mouse_down(gpui::MouseButton::Left, {
            let view = view.clone();
            move |_event, _, cx| {
                cx.stop_propagation();
                view.update(cx, |this, cx| {
                    this.show_file_menu = false;
                    // Handle export action
                    this.export_data();
                    cx.notify();
                });
            }
        })
        .child("Export..."),
)
```

## Styling Guidelines

When adding new menu items, follow these style guidelines:

1. **Padding**: `px_3().py_2()` (horizontal: 12px, vertical: 8px)
2. **Text Size**: `text_sm()` (14px)
3. **Text Color**: `rgb(0xcdd6f4)` (light gray-white)
4. **Hover Color**: `rgb(0x313147)` (slightly darker than background)
5. **Cursor**: `cursor_pointer()` to indicate clickability

## Accessibility

### Current Implementation
- Mouse-only interaction
- Visual hover feedback

### Recommended Improvements
- Add keyboard navigation support
- Add ARIA labels for screen readers
- Add focus indicators
- Support keyboard shortcuts

## Testing

### Manual Testing Checklist

- [ ] Click "File" - menu opens
- [ ] Click "File" again - menu closes
- [ ] Click "Open BLF..." - file dialog opens
- [ ] Click outside menu - menu closes
- [ ] Click another tab - menu closes and tab switches
- [ ] Open BLF file - loads successfully
- [ ] Menu stays open when hovering over items
- [ ] Menu item highlights on hover

### Automated Testing

Future work should include unit tests for:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_menu_toggle() {
        let mut app = CanViewApp::new_state();
        
        // Initially closed
        assert!(!app.show_file_menu);
        
        // Toggle to open
        app.show_file_menu = true;
        assert!(app.show_file_menu);
        
        // Toggle to close
        app.show_file_menu = false;
        assert!(!app.show_file_menu);
    }

    #[test]
    fn test_file_menu_closes_on_tab_switch() {
        let mut app = CanViewApp::new_state();
        app.show_file_menu = true;
        app.current_view = AppView::LibraryView;
        
        // After tab switch, menu should close
        // (This would be tested in the actual event handler)
    }
}
```

## Related Files

- `src/view/src/app/state.rs` - State structure
- `src/view/src/app/impls.rs` - UI implementation
- `src/view/src/main.rs` - Application entry point

## References

- [Zed IDE UI Patterns](https://github.com/zed-industries/zed)
- [GPUI Documentation](https://github.com/zed-industries/zed/tree/main/crates/gpui)
- Material Design Menu Guidelines
- macOS Human Interface Guidelines - Menus

## Changelog

### Version 0.2.0 (Current)
- ✅ Replaced "Logs" tab with "File" dropdown
- ✅ Moved "Open BLF" button to File menu
- ✅ Added click-outside-to-close behavior
- ✅ Added auto-close on tab switch
- ✅ Styled dropdown menu to match dark theme

### Version 0.1.0 (Previous)
- Flat menu bar with "Logs", "Library", "Plot", "Open BLF" buttons

## Troubleshooting

### Issue: Menu doesn't close when clicking outside

**Solution**: Ensure the click-outside handler is attached to the parent container and `cx.stop_propagation()` is called on menu items to prevent event bubbling.

### Issue: Menu closes immediately after opening

**Solution**: Check that `cx.stop_propagation()` is called on the menu button click handler to prevent the click-outside handler from firing.

### Issue: Menu appears in wrong position

**Solution**: The dropdown uses absolute positioning with `top(px(28.))`. Adjust this value based on your menu bar height.

## Summary

The File menu dropdown improves the UI by:
1. **Better Organization** - File-related actions are grouped together
2. **Cleaner Layout** - Reduces top bar clutter
3. **Industry Standard** - Follows familiar IDE patterns (Zed, VS Code, etc.)
4. **Scalability** - Easy to add more menu items in the future
5. **User Experience** - Clear visual feedback and intuitive interactions

This implementation provides a solid foundation for expanding the menu system with additional features and menu categories in the future.