# File Menu Quick Start Guide

## What's New?

The top navigation bar has been updated! Instead of a separate "Open BLF" button, we now have a **File** dropdown menu (following Zed IDE's design pattern).

**Before:**
```
[Logs] [Library] [Plot] [Open BLF]
```

**After:**
```
[File ▼] [Logs] [Library] [Plot]
```

## How to Open a BLF File

### Option 1: Using the File Menu (Recommended)

1. **Click the "File" button** in the top-left corner
2. **Select "Open BLF..."** from the dropdown menu
3. **Choose your `.blf` or `.bin` file** from the file picker
4. **View the results!** The file loads automatically

### Visual Guide

```
┌──────────────────────────────────────────────┐
│ [File ▼] [Logs] [Library] [Plot]             │
└──────────────────────────────────────────────┘
           ↓ (click)
┌────────────────┐
│ Open BLF...    │
└────────────────┘
```

## Menu Behavior

### Opening/Closing the Menu
- **Click "File"** → Opens the menu
- **Click "File" again** → Closes the menu
- **Click outside** → Closes the menu
- **Click another tab** → Closes the menu and switches view
- **Click "Open BLF..."** → Closes menu and opens file dialog

### Menu Item Highlighting
When you hover over menu items, they highlight in a darker color:
- **Normal**: `#1e1e2e` (dark background)
- **Hover**: `#313147` (slightly darker)

## Example Workflow

Here's a typical workflow using the new File menu:

### 1. Start the Application
```bash
cargo run
```

### 2. Open a BLF File
- Click **File** → **Open BLF...**
- Select your file (e.g., `test_corrupted.blf`)
- Wait for loading to complete

### 3. Check the Results
- **Logs view**: Shows all parsed CAN messages
- **Status bar**: Displays loading results
  - Success: `"BLF 解析成功: 1000 个对象"`
  - Partial success: `"BLF 解析完成: 20 对象成功 | 3 个错误 (首个: ...)"`
  - Failure: `"❌ BLF 解析失败: ..."`
- **Console**: Shows detailed error information (if any)

## Testing with the Test File

We've included a test file with intentional errors to demonstrate error handling:

### Test File: `test_corrupted.blf`

**Location**: Project root directory

**Contents**:
- ✅ 20 valid CAN messages (ID: 0x100-0x113)
- ❌ 4 types of corrupted data at the end

**Expected Results**:
```
Status Bar: "BLF 解析完成: 20 对象成功 | 3 个错误 (首个: Invalid LOBJ container magic string)"
```

**To Test**:
1. Click **File** → **Open BLF...**
2. Select `test_corrupted.blf`
3. Verify:
   - Logs view shows 20 messages ✅
   - Status bar shows errors ✅
   - Console shows detailed error info ✅

## Keyboard Shortcuts (Future)

Currently, the File menu only supports mouse interaction. Planned keyboard shortcuts:

- `Alt+F` → Focus File menu
- `Ctrl+O` / `Cmd+O` → Quick open BLF file
- `Escape` → Close menu
- `↑/↓` → Navigate menu items
- `Enter` → Select highlighted item

## Troubleshooting

### Menu Won't Open
- **Issue**: Clicking "File" does nothing
- **Solution**: Restart the application

### Menu Won't Close
- **Issue**: Menu stays open when clicking elsewhere
- **Solution**: Click the "File" button again to toggle it closed

### File Dialog Doesn't Open
- **Issue**: Clicking "Open BLF..." doesn't show file picker
- **Solution**: Check console for error messages, ensure the file picker library is working

### Nothing Happens After Selecting File
- **Issue**: File dialog closes but nothing loads
- **Solution**: 
  1. Check the status bar for error messages
  2. Check the console for detailed error information
  3. Verify the file format is valid BLF

## Tips & Tricks

### 1. Quick Access
The File menu is always available from any view, so you can open a new BLF file anytime without switching views.

### 2. Multiple Files
You can open multiple BLF files in sequence - each new file replaces the previous one.

### 3. Error Recovery
If a file has partial errors (like `test_corrupted.blf`), the valid data is still displayed. Check the status bar for error details.

### 4. File Location
For easy access, keep your BLF files in:
- Project root directory
- A dedicated `blf_files/` folder
- Any easily accessible location

## Future Enhancements

Coming soon to the File menu:

- **Recent Files** - Quick access to recently opened files
- **Export to CSV** - Export current data
- **Save Configuration** - Quick config save
- **Load Configuration** - Quick config load

## Getting Help

If you encounter issues:

1. **Check the console** - Detailed error messages are printed there
2. **Check the status bar** - Shows current status and error summaries
3. **Review the logs** - Full error details in the console output
4. **Try the test file** - Use `test_corrupted.blf` to verify the feature works

## Related Documentation

- `FILE_MENU_UPDATE.md` - Detailed technical documentation
- `CORRUPTED_BLF_TEST.md` - Test file documentation
- `ERROR_HANDLING_SUMMARY.md` - Error handling features
- `README.md` - General project documentation

---

**Version**: 1.0.0  
**Last Updated**: 2025-01-15  
**Status**: ✅ Feature Complete and Tested