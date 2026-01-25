# Comprehensive Refactoring Plan for main.rs

## Executive Summary

The current `main.rs` file contains **3,527 lines** with multiple responsibilities mixed together. This plan proposes splitting it into **8 focused modules** to improve maintainability, testability, and code organization while maintaining the existing public API and functionality.

---

## Current State Analysis

### Existing Structure
```
src/view/src/
├── main.rs (3,527 lines) ← TARGET FOR REFACTORING
├── library_view.rs (1,158 lines) ← Already extracted
├── library/mod.rs (468 lines) ← Already extracted
├── models/mod.rs (90 lines) ← Already extracted
├── ui/ ← Partially organized
│   ├── components/
│   └── views/
└── config/ ← Exists but needs expansion
```

### Code Distribution in main.rs

| Component | Lines | Percentage |
|-----------|-------|------------|
| State Structures | ~100 | 2.8% |
| Constructor/Initialization | ~200 | 5.7% |
| Library Management Methods | ~450 | 12.8% |
| Log View Rendering | ~1,400 | 39.7% |
| Config View Rendering | ~100 | 2.8% |
| Chart View Rendering | ~50 | 1.4% |
| Window/Layout Management | ~250 | 7.1% |
| Event Handlers | ~300 | 8.5% |
| Message Rendering/Formatting | ~600 | 17.0% |
| Configuration I/O | ~77 | 2.2% |

---

## Proposed Module Structure

```
src/view/src/
├── main.rs (~200 lines) ← Entry point only
├── app/
│   ├── mod.rs
│   ├── state.rs ← App state structures
│   └── core_impl.rs ← Core App impl (constructor, init)
├── handlers/
│   ├── mod.rs
│   ├── keyboard.rs ← Keyboard event handlers
│   ├── mouse.rs ← Mouse event handlers
│   └── file_io.rs ← File loading/saving handlers
├── views/
│   ├── mod.rs
│   ├── log_view.rs ← Log view rendering (~1,400 lines)
│   ├── config_view.rs ← Config view (delegates to library_view)
│   └── chart_view.rs ← Chart view
├── rendering/
│   ├── mod.rs
│   ├── message_renderer.rs ← Message formatting/rendering (~600 lines)
│   ├── filter_dropdowns.rs ← Filter dropdown rendering (~400 lines)
│   └── scrollbars.rs ← Custom scrollbar logic (~300 lines)
├── window/
│   ├── mod.rs
│   ├── management.rs ← Window state (maximize/restore)
│   └── layout.rs ← Container height calculations
├── config/
│   ├── mod.rs
│   ├── io.rs ← Config file loading/saving
│   └── startup.rs ← Startup configuration
└── ui/
    ├── mod.rs
    ├── components/ ← Existing components
    └── views/ ← Existing views
```

---

## Implementation Order

### Phase 1: Foundation (Low Risk) ✅ START HERE
1. Create module directory structure
2. Extract `app/state.rs` - No changes to logic, just move structs
3. Extract `app/core_impl.rs` - Constructor methods
4. Extract `config/io.rs` and `config/startup.rs` - Simple I/O operations

**Expected compilation issues:** Minimal
**Testing:** Smoke test - app should compile and run

### Phase 2: Event Handlers (Low-Medium Risk)
5. Extract `handlers/keyboard.rs` - Move keyboard handlers
6. Extract `handlers/mouse.rs` - Move mouse handlers
7. Extract `handlers/file_io.rs` - Move file operations

### Phase 3: Rendering Modules (Medium Risk)
8. Extract `rendering/message_renderer.rs` - Message formatting functions
9. Extract `rendering/scrollbars.rs` - Scrollbar logic
10. Extract `rendering/filter_dropdowns.rs` - Filter dropdowns

### Phase 4: View Modules (Medium-High Risk)
11. Extract `views/log_view.rs` - Large chunk of code
12. Extract `views/config_view.rs` - Simple delegation
13. Extract `views/chart_view.rs` - Simple placeholder

### Phase 5: Window Management (Low Risk)
14. Extract `window/management.rs` - Window state
15. Extract `window/layout.rs` - Layout calculations

### Phase 6: Cleanup (Low Risk)
16. Update `main.rs` to use all new modules
17. Remove dead code
18. Update module documentation
19. Run full test suite

---

## Success Criteria

- [ ] All existing functionality works identically
- [ ] Code compiles without warnings
- [ ] Each module is under 500 lines (except log_view which is ~1,400)
- [ ] All tests pass
- [ ] Documentation updated
- [ ] No regression in performance
- [ ] Code review approved

---

## Benefits of This Refactoring

### Maintainability
- Each module has a single, clear responsibility
- Easier to locate and fix bugs
- Changes are isolated to specific modules

### Testability
- Pure functions can be unit tested
- Event handlers can be tested independently
- Mock dependencies easier

### Code Organization
- Logical grouping of related functionality
- Clear module boundaries
- Easier onboarding for new developers

### Performance
- No performance overhead (Rust's zero-cost abstractions)
- Potential for future optimizations (e.g., parallel rendering)
- Better compilation times (incremental compilation)

---

**Total Estimated Time:** 3-4 weeks for full implementation

**Quick Start:** Begin with Phase 1 to establish the foundation with minimal risk.
