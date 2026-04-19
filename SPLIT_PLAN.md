# Phase 2.21: Split impls.rs into Multiple Files

## Goal
Reduce impls.rs from 3,576 to <3,000 lines by splitting into focused files.

## Strategy
Move all rendering methods (~2,000+ lines) to `impls_rendering.rs`

## Files to Create

### 1. `app/impls_rendering.rs`
Will contain all rendering methods:
- `render_message_row()` - Lines 378-493 (~115 lines)
- `render_library_view()` - Lines 662-774 (~112 lines)
- `render_log_view()` - Lines 776-1817 (~1041 lines)
- `render_channel_filter_dropdown()` - Lines 1819-1999 (~180 lines)
- `render_config_view()` - Lines 2001-2157 (~156 lines)
- `render()` - Lines 2159-2935 (~776 lines)

**Total**: ~2,380 lines

### 2. Update `app/mod.rs`
Add: `mod impls_rendering;`

## Expected Result
- impls.rs: 3,576 → ~1,196 lines (-2,380 lines, 66.6% reduction)
- impls_rendering.rs: ~2,380 lines (new file)
- **Total across both files**: ~3,576 lines (same code, better organized)
- **Main impls.rs line count**: ~1,196 lines (well under 3000!)

## Benefits
1. ✅ Achieves <3000 lines goal for main file
2. ✅ Clear separation of concerns (rendering vs logic)
3. ✅ Better code organization
4. ✅ Easier navigation and maintenance

## Implementation Steps
1. Extract all render methods to temp file
2. Create impls_rendering.rs with proper header/imports
3. Update mod.rs to declare new module
4. Verify compilation
5. Remove render methods from impls.rs
6. Final verification and commit

## Status: READY TO IMPLEMENT
