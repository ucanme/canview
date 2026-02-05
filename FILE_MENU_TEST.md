# File Menu Testing Guide

## Quick Test Steps

### 1. Start the Application
Run the application and wait for the main window to appear.

### 2. Locate the "File" Button
Look at the top navigation bar (title bar area). You should see buttons labeled:
- `File` (leftmost)
- `Logs`
- `Library`
- `Plot`

### 3. Click the "File" Button
Click once on the "File" button with your mouse.

**Expected Behavior:**
- A dropdown menu should appear below the "File" button
- The menu should contain one item: "Open BLF..."
- The "File" button should appear highlighted/active
- The rest of the UI should have a semi-transparent overlay

**Console Output:**
You should see these debug messages in the console:
```
🖱️ File button clicked!
📂 Toggling show_file_menu: false -> true
🎨 Rendering file menu overlay
🎨 Rendering file dropdown menu
```

### 4. Click the "Open BLF..." Menu Item
Click on the "Open BLF..." text in the dropdown.

**Expected Behavior:**
- The dropdown menu should close
- A file picker dialog should open
- The dialog should show BLF and BIN files

**Console Output:**
```
📁 Open BLF menu item clicked
```

### 5. Click Outside the Menu
Reopen the menu and click somewhere outside the dropdown (e.g., on the content area or status bar).

**Expected Behavior:**
- The dropdown menu should close immediately
- No file dialog should open

**Console Output:**
```
❌ File menu overlay clicked - closing dropdown
```

### 6. Test Menu Persistence
- Click "File" to open menu
- Click on another tab (e.g., "Logs" or "Library")
- Menu should close when switching tabs

## Visual Verification Checklist

### When Menu is OPEN:
- [ ] Dropdown visible at `top: 40px, left: 16px`
- [ ] Menu has dark background `#1e1e2e`
- [ ] Menu has border `#3a3a4a`
- [ ] Menu has rounded corners
- [ ] Menu has shadow
- [ ] "Open BLF..." text is visible
- [ ] Hovering over "Open BLF..." highlights it with `#313147` background
- [ ] Cursor changes to pointer when hovering over menu item
- [ ] Rest of window has semi-transparent overlay
- [ ] "File" button shows active state (darker background)

### When Menu is CLOSED:
- [ ] No dropdown visible
- [ ] No overlay visible
- [ ] "File" button shows normal state (lighter text)
- [ ] Can interact with other UI elements normally

## Console Log Analysis

### Successful Open Sequence
```
🖱️ File button clicked!
📂 Toggling show_file_menu: false -> true
🎨 Rendering file menu overlay
🎨 Rendering file dropdown menu
```

### Successful Close Sequence (via Menu Item)
```
📁 Open BLF menu item clicked
Loading BLF...
```

### Successful Close Sequence (via Overlay)
```
❌ File menu overlay clicked - closing dropdown
```

## Troubleshooting

### Problem: Menu doesn't appear at all

**Check 1: Console Output**
If you see `🖱️ File button clicked!` but NO `🎨 Rendering` messages, the state isn't updating.

**Solution:** Check if `cx.notify()` is being called after state change.

**Check 2: Console Output**
If you see `🎨 Rendering` messages but menu isn't visible, it's a rendering/positioning issue.

**Solution:** The menu might be:
- Off-screen (check position values)
- Behind other elements (check z-order)
- Hidden by parent overflow (check `.overflow_hidden()`)

### Problem: Menu appears but disappears immediately

**Check 1: Event Propagation**
Something is clicking the overlay immediately.

**Solution:** Check if the menu itself is triggering the overlay's click event.

**Check 2: Parent Event Handlers**
A parent element has an `on_mouse_down` handler that's closing the menu.

**Solution:** Ensure `cx.stop_propagation()` is called on the menu's event handlers.

### Problem: Menu items don't respond to clicks

**Check 1: Event Handler Registration**
The click handler isn't attached.

**Solution:** Verify `.on_mouse_down()` is called on the menu item div.

**Check 2: Event Propagation**
Clicks are being intercepted by parent overlay.

**Solution:** Ensure `cx.stop_propagation()` is called BEFORE updating state.

### Problem: Menu is in wrong position

**Current Position:** `top: 40px, left: 16px`

**To Adjust:** Modify these values in the code:
```rust
.absolute()
.top(px(40.))   // Distance from top
.left(px(16.))  // Distance from left
```

## Code Location Reference

### File Button
**File:** `src/view/src/app/impls.rs`
**Line:** ~3220
```rust
btn_style(self.show_file_menu)
    .id("file_menu_btn")
    .on_mouse_down(gpui::MouseButton::Left, { ... })
```

### File Menu Dropdown
**File:** `src/view/src/app/impls.rs`
**Line:** ~3480-3575
- Overlay: Lines 3480-3495
- Dropdown container: Lines 3498-3575

### State Field
**File:** `src/view/src/app/state.rs`
**Line:** ~163
```rust
pub show_file_menu: bool,
```

## Expected Dimensions

- **Button height:** 24px
- **Button padding:** 12px (left/right)
- **Menu top position:** 40px (button height + title bar padding)
- **Menu left position:** 16px (title bar padding)
- **Menu width:** 200px
- **Menu item height:** ~32px (padding + text)
- **Menu border radius:** 6px

## Keyboard Shortcuts (Not Yet Implemented)

Future enhancements should include:
- `Alt+F` - Open File menu
- `↑`/`↓` - Navigate menu items
- `Enter` - Select highlighted item
- `Esc` - Close menu

## Performance Considerations

The current implementation re-renders the menu on every `cx.notify()`. For better performance:

1. Only notify when `show_file_menu` actually changes
2. Consider using `gpui::Entity<PopupMenu>` for complex menus
3. Memoize menu items if the list is long

## Next Steps After Testing

Once basic functionality is confirmed:

1. **Add more menu items:**
   - "Export..."
   - "Settings..."
   - "Exit"

2. **Add icons to menu items:**
   - Folder icon for "Open BLF..."
   - Download icon for "Export..."
   - Gear icon for "Settings..."

3. **Add keyboard shortcuts display:**
   - "Open BLF...     Ctrl+O"
   - "Export...       Ctrl+E"
   - "Settings...     Ctrl+,"

4. **Implement keyboard navigation:**
   - Arrow keys to move selection
   - Enter to select
   - Escape to close

5. **Add submenus support:**
   - "Recent Files" → submenu
   - "Import" → submenu (DBC, LDF, etc.)