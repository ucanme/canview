# Window Maximize/Restore Fixes

## Phase 2 Summary

### Fixes Applied (Commits)

1. **Phase 2.23** (commit aab211c): LibraryManager Clone implementation
   - Added `#[derive(Clone)]` to LibraryManager struct
   - Changed `new_with_state()` signature to accept `Option<Bounds<Pixels>>` for display_bounds

2. **Phase 2.24** (commit a72b6d6): Window operation order fix
   - **Critical**: Fixed window operation order to prevent context invalidation
   - Changed from: `close old → open new` (BROKEN)
   - Changed to: `open new → close old` (CORRECT)

3. **Phase 2.25** (commit dc2326f): LibraryManager parameter usage fix
   - Fixed `new_with_state()` ignoring the library_manager parameter
   - Changed from: `library_manager: LibraryManager::new()` (BROKEN)
   - Changed to: `library_manager` (CORRECT - uses cloned parameter)

## Known Issues & Investigation

### Chart View Crash

**Symptom**: `STATUS_STACK_BUFFER_OVERRUN` (0xc0000409) when maximizing from chart view

**Possible Causes**:
1. Entity<InputState> fields may hold references to old window/context
2. Large HashMap clones (dbc_channels, ldf_channels) may cause stack overflow
3. Async operations or event handlers may reference destroyed objects
4. Scroll handles or other GPUI objects may not be safely transferable

**Fields NOT Cloned (reset to defaults)**:
- `plot_data: Arc<[Series]>` - reset to empty
- `signal_search_input: Option<Entity<InputState>>` - reset to None
- `library_name_input: Option<Entity<InputState>>` - reset to None
- `version_name_input: Option<Entity<InputState>>` - reset to None
- `channel_id_input: Option<Entity<InputState>>` - reset to None
- `channel_name_input: Option<Entity<InputState>>` - reset to None
- `channel_db_path_input: Option<Entity<InputState>>` - reset to None
- All scroll handles - reset to new instances
- Plot zoom/hover state - reset to defaults

### Comparison with v0.0.13

**v0.0.13 Behavior**:
- Did NOT preserve library_manager (recreated as empty)
- Did NOT preserve most UI state
- Used same window operation order (open new → close old)
- Successfully handled maximize/restore without crashes

**Current Implementation**:
- Preserves library_manager with all library data
- Preserves 14 core state fields
- Uses correct window operation order
- **Issue**: Crashes when maximizing from chart view after interaction

## Testing Recommendations

### Test 1: Basic Maximize/Restore
```
1. Start application
2. Click maximize button (top-right corner)
3. Click restore button
4. Repeat 5-10 times
Expected: No crashes, window size preserved
```

### Test 2: Library Data Preservation
```
1. Import a database file
2. Verify library data is loaded
3. Maximize window
4. Verify library data is still present
5. Restore window
6. Verify library data is still present
Expected: Library data preserved across window operations
```

### Test 3: Chart View (KNOWN ISSUE)
```
1. Navigate to chart view
2. Click on chart elements (select signals, zoom, etc.)
3. Click maximize button
Current: May crash with STATUS_STACK_BUFFER_OVERRUN
```

## Potential Solutions for Chart Crash

### Option A: Reset Chart State on Maximize
Safest approach - reset all chart-related fields to defaults:
```rust
// In toggle_maximize, don't clone these fields:
// - plot_data (already reset)
// - All plot_zoom/plot_hover fields (already reset)
```

### Option B: Use Arc for Large Data
Wrap large HashMaps in Arc to avoid deep copies:
```rust
pub dbc_channels: Arc<HashMap<u16, DbcDatabase>>,
pub ldf_channels: Arc<HashMap<u16, LdfDatabase>>,
```

### Option C: Entity Field Validation
Add validation to ensure Entity fields don't reference destroyed objects:
```rust
// Before cloning, validate all Entity fields are safe to transfer
```

## Next Steps

1. Test current fixes with basic maximize/restore
2. If chart crash persists, implement Option A (reset chart state)
3. Consider Option B for long-term performance improvement
4. Investigate specific chart operations that trigger the crash

## Files Modified

- `src/view/src/library/mod.rs` - Added Clone to LibraryManager
- `src/view/src/app/impls.rs` - Fixed window order, library_manager usage
- `src/view/src/app/state.rs` - No changes needed
