# BLF Read Progress + Precise Error Reporting Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** StatusBar shows `521.0KB / 521.0KB (100%)` after loading a BLF; parse errors carry `structure.field` context like `FileStatistics.signature: Invalid BLF file magic string`.

**Architecture:** Add `Context { inner, ctx }` variant to `BlfParseError` + a `.context()` extension trait for `?` chains. Wrap ~20 read sites in 5 BLF files with `.context("StructName.field")`. Add `bytes_total` / `bytes_consumed` to `BlfResult` (parse returns consumed bytes). Add 2 state fields to `CanViewApp`; StatusBar renders a new byte-progress segment with a `format_bytes` helper.

**Tech Stack:** Rust nightly, GPUI, `byteorder` for BLF reads. Build with `cargo +nightly build -p view`. Local tests can't run (SIGBUS during link), so write `#[cfg(test)]` tests for CI.

## Global Constraints

- Rust nightly; build with `cargo +nightly build -p view 2>&1 | tail -3` (expects "Finished")
- `cargo test` SIGBUS during link locally (pre-existing); write unit tests for CI
- Baseline warnings: 330; new code must NOT add warnings. Check with `cargo +nightly clippy -p view 2>&1 | grep -c "^warning"` → expect ≤ 330
- New code in `view` crate MUST NOT contain `rgb(0x` literals — use `crate::ui::theme::colors::*`
- `BlfParseError` is in `src/blf/src/error.rs`; the `From<io::Error>` impl must remain (preserves `?` ergonomics)
- `BlfParser::parse` is at `src/blf/src/parser.rs:155` and currently returns `BlfParseResult<(Vec<LogObject>, Vec<BlfParseError>)>`
- Two callers of `parse`: `src/blf/src/file.rs:77` (`read_blf_from_file`) and `src/blf/src/file.rs:148` (`StreamingBlfReader::read_next_batch` — currently unused but must keep compiling)
- `BlfResult` is at `src/blf/src/file.rs:10` with fields `file_stats`, `objects`, `errors`
- Each commit must compile (`cargo +nightly build -p view`)
- Commit message style: lowercase prefix `feat(blf):` / `refactor(blf):` / `feat(ui):`

---

## File Structure

### Modified files

| File | Changes | Task |
|---|---|---|
| `src/blf/src/error.rs` | Add `Context` variant + `context()` method + `BlfResultContext` trait + Display recursion + Error::source update | Task 1 |
| `src/blf/src/file.rs` | Add `bytes_total` / `bytes_consumed` to `BlfResult`; change `parse` return to tuple with `u64`; update `read_blf_from_file` to set both fields; update `StreamingBlfReader::read_next_batch` for new tuple shape | Task 2 |
| `src/blf/src/file_statistics.rs` | Add `.context("FileStatistics.X")` to each read | Task 3 |
| `src/blf/src/object_header.rs` | Add `.context("ObjectHeader.X")` to each read | Task 3 |
| `src/blf/src/objects/log_container.rs` | Add `.context("LogContainer.X")` to each read | Task 3 |
| `src/blf/src/parser.rs` | Add `.context("BlfParser.X")` to each read; return consumed bytes via cursor position at end | Task 3 |
| `src/blf/src/file.rs` | Wrap `FileStatistics::read` and `parser.parse` calls with `.context()` | Task 3 |
| `src/view/src/app/state.rs` | Add `blf_bytes_total: u64`, `blf_bytes_consumed: u64` to `CanViewApp`; init in `new_with_maximized_state_and_bounds` and `new_state`-style constructors in `impls.rs` | Task 4 |
| `src/view/src/app/impls.rs` | Init new fields in 3 constructor locations (line ~35, ~272, ~589) | Task 4 |
| `src/view/src/app/impls.rs` | In `apply_blf_result` Ok path, set `blf_bytes_total` and `blf_bytes_consumed` from `result`; on Err, reset both to 0 | Task 4 |
| `src/view/src/ui/components/status_bar.rs` | Add `format_bytes` helper + `render_blf_progress_segment` function + insert into `render_status_bar` left side | Task 5 |

---

## Task 1: Add Context variant to BlfParseError

**Files:**
- Modify: `src/blf/src/error.rs` (full file)

**Interfaces:**
- Consumes: nothing
- Produces: `BlfParseError::Context { inner: Box<BlfParseError>, ctx: String }` variant; `BlfParseError::context(self, ctx: impl Into<String>) -> Self` method; `BlfResultContext<T>` trait with `fn context(self, ctx: impl Into<String>) -> BlfParseResult<T>`; Display recursion `"ctx: inner"`; `Error::source` returns `inner` for `Context`

- [ ] **Step 1: Verify baseline build passes**

Run: `cargo +nightly build -p view 2>&1 | tail -3`
Expected: `Finished` dev` profile [unoptimized + debuginfo] target(s)` with 330 warnings

- [ ] **Step 2: Replace `src/blf/src/error.rs` with the new version**

```rust
//! Defines error types for BLF parsing.

use std::fmt;
use std::io;
use std::error::Error;

/// Represents a parsing error that can occur while processing a BLF file.
#[derive(Debug)]
pub enum BlfParseError {
    /// An I/O error occurred while reading the data.
    IoError(io::Error),
    /// The file does not start with the expected "LOGG" magic string.
    InvalidFileMagic,
    /// A log container does not start with the expected "LOBJ" magic string.
    InvalidContainerMagic,
    /// The data ended unexpectedly while parsing an object.
    UnexpectedEof,
    /// An unknown or unsupported compression method was specified in a LogContainer.
    UnsupportedCompression(u16),
    /// An unknown object header version was encountered.
    UnknownHeaderVersion(u16),
    /// Wraps another error with a context string describing which
    /// structure/field was being read when the error occurred. Display
    /// prints the full chain: "FileStatistics.signature: ...inner...".
    Context {
        inner: Box<BlfParseError>,
        ctx: String,
    },
}

impl BlfParseError {
    /// Wrap this error with a context describing where it occurred.
    /// `err.context("FileStatistics.signature")` returns
    /// `BlfParseError::Context { inner: err, ctx: "FileStatistics.signature" }`.
    pub fn context(self, ctx: impl Into<String>) -> Self {
        Self::Context {
            inner: Box::new(self),
            ctx: ctx.into(),
        }
    }
}

/// Extension trait so `?` chains on `BlfParseResult` can add context inline:
/// `cursor.read_u32::<LittleEndian>().map_err(BlfParseError::IoError).context("X")?`.
pub trait BlfResultContext<T> {
    fn context(self, ctx: impl Into<String>) -> BlfParseResult<T>;
}

impl<T> BlfResultContext<T> for BlfParseResult<T> {
    fn context(self, ctx: impl Into<String>) -> BlfParseResult<T> {
        self.map_err(|e| e.context(ctx))
    }
}

impl fmt::Display for BlfParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BlfParseError::Context { inner, ctx } => write!(f, "{}: {}", ctx, inner),
            BlfParseError::IoError(e) => write!(f, "I/O error: {}", e),
            BlfParseError::InvalidFileMagic => write!(f, "Invalid BLF file magic string"),
            BlfParseError::InvalidContainerMagic => write!(f, "Invalid LOBJ container magic string"),
            BlfParseError::UnexpectedEof => write!(f, "Unexpected end of file"),
            BlfParseError::UnsupportedCompression(c) => write!(f, "Unsupported compression method: {}", c),
            BlfParseError::UnknownHeaderVersion(v) => write!(f, "Unknown object header version: {}", v),
        }
    }
}

impl Error for BlfParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            BlfParseError::Context { inner, .. } => Some(inner.as_ref()),
            BlfParseError::IoError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for BlfParseError {
    fn from(err: io::Error) -> Self {
        if err.kind() == io::ErrorKind::UnexpectedEof {
            BlfParseError::UnexpectedEof
        } else {
            BlfParseError::IoError(err)
        }
    }
}

/// A specialized `Result` type for BLF parsing.
pub type BlfParseResult<T> = Result<T, BlfParseError>;
```

- [ ] **Step 3: Add a unit test for the context chain**

Append to `src/blf/src/error.rs` at the end of the file:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_wraps_and_displays() {
        let err = BlfParseError::InvalidFileMagic.context("FileStatistics.signature");
        assert_eq!(format!("{}", err), "FileStatistics.signature: Invalid BLF file magic string");
    }

    #[test]
    fn test_context_chain_is_recursive() {
        let inner = BlfParseError::UnsupportedCompression(3);
        let mid = inner.context("LogContainer.compression_method");
        let outer = mid.context("BlfParser.parse");
        assert_eq!(
            format!("{}", outer),
            "BlfParser.parse: LogContainer.compression_method: Unsupported compression method: 3"
        );
    }

    #[test]
    fn test_context_source_returns_inner() {
        let err = BlfParseError::InvalidFileMagic.context("FileStatistics.signature");
        let source = std::error::Error::source(&err);
        assert!(source.is_some());
        assert_eq!(format!("{}", source.unwrap()), "Invalid BLF file magic string");
    }

    #[test]
    fn test_blf_result_context_trait() {
        let r: BlfParseResult<u32> = Err(BlfParseError::InvalidFileMagic);
        let wrapped = r.context("FileStatistics.signature");
        assert!(wrapped.is_err());
        let err = wrapped.unwrap_err();
        assert_eq!(format!("{}", err), "FileStatistics.signature: Invalid BLF file magic string");
    }
}
```

- [ ] **Step 4: Build to verify the new module compiles**

Run: `cargo +nightly build -p blf 2>&1 | tail -5`
Expected: `Finished` (warnings allowed, no errors)

Then verify the full view crate still builds (the new variant must not break existing `match` arms because no code matches exhaustively yet — only `Context` is new):

Run: `cargo +nightly build -p view 2>&1 | tail -5`
Expected: `Finished` with 330 warnings (or fewer if the new code removed unused warnings)

- [ ] **Step 5: Commit**

```bash
git add src/blf/src/error.rs
git commit -m "$(cat <<'EOF'
feat(blf): add Context variant to BlfParseError

Adds BlfParseError::Context { inner, ctx } for wrapping parse errors
with the structure/field name where they occurred. Adds:
- BlfParseError::context(self, ctx) -> Self helper
- BlfResultContext<T> trait so ?-chains can call .context("X")?
- Display recursion: "ctx: inner" for Context, otherwise unchanged
- Error::source returns inner for Context (enables anyhow chains)
- Unit tests for single wrap, nested wrap, source(), and the trait

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

## Task 2: Add bytes_total/consumed to BlfResult + parse returns consumed bytes

**Files:**
- Modify: `src/blf/src/file.rs` (struct `BlfResult` at line 10; `read_blf_from_file` at line 67; `StreamingBlfReader::read_next_batch` at line 148)
- Modify: `src/blf/src/parser.rs` (function `parse` at line 155)

**Interfaces:**
- Consumes: `BlfParseError::Context` from Task 1 (not strictly required, but the parse returns will be wrapped in Task 3)
- Produces: `BlfResult` with `bytes_total: u64` + `bytes_consumed: u64` fields; `BlfParser::parse` returns `BlfParseResult<(Vec<LogObject>, Vec<BlfParseError>, u64)>` where the u64 is cursor position at end

- [ ] **Step 1: Verify baseline build passes**

Run: `cargo +nightly build -p view 2>&1 | tail -3`
Expected: `Finished` with 330 warnings

- [ ] **Step 2: Update BlfResult struct**

In `src/blf/src/file.rs`, find `pub struct BlfResult` (line 10). Replace it with:

```rust
/// Represents the complete result of parsing a BLF file.
#[derive(Debug)]
pub struct BlfResult {
    /// The file statistics header.
    pub file_stats: FileStatistics,
    /// A vector of all parsed log objects.
    pub objects: Vec<LogObject>,
    /// A vector of any errors encountered during parsing.
    pub errors: Vec<BlfParseError>,
    /// Total file size in bytes (for progress reporting in the UI).
    pub bytes_total: u64,
    /// Bytes the parser consumed. Equals bytes_total on full success;
    /// less when parsing bailed early on a malformed structure.
    pub bytes_consumed: u64,
}
```

- [ ] **Step 3: Update BlfParser::parse to return consumed bytes**

In `src/blf/src/parser.rs`, find `pub fn parse(&self, data: &[u8]) -> BlfParseResult<(Vec<LogObject>, Vec<BlfParseError>)>` at line 155.

Change the signature to:
```rust
pub fn parse(&self, data: &[u8]) -> BlfParseResult<(Vec<LogObject>, Vec<BlfParseError>, u64)> {
```

Find the `Ok((all_objects, all_errors))` return at the end of the function (around line 271). Change it to return cursor position:

```rust
        let consumed = cursor.position();
        Ok((all_objects, all_errors, consumed))
```

Also find the early `Err` returns inside `parse` (if any — check the file) and ensure they don't break; the cursor position can be returned via Err path through Context wrapping in Task 3. For now, leave the existing Err paths unchanged.

- [ ] **Step 4: Update read_blf_from_file to compute bytes_total and bytes_consumed**

In `src/blf/src/file.rs`, find `read_blf_from_file` (line 67). Replace the function body with:

```rust
pub fn read_blf_from_file<P: AsRef<Path>>(path: P) -> BlfParseResult<BlfResult> {
    let data = fs::read(path.as_ref()).map_err(BlfParseError::IoError)?;
    let bytes_total = data.len() as u64;
    let mut cursor = Cursor::new(&data[..]);

    // 1. Parse the file statistics header. This will advance the cursor.
    let file_stats = FileStatistics::read(&mut cursor)?;
    let stats_consumed = cursor.position();

    // 2. Parse the log objects from the rest of the data slice.
    let parser = BlfParser::new();
    let remaining_data = &data[stats_consumed as usize..];
    let (objects, errors, parse_consumed) = parser.parse(remaining_data)?;

    let bytes_consumed = stats_consumed + parse_consumed;

    Ok(BlfResult {
        file_stats,
        objects,
        errors,
        bytes_total,
        bytes_consumed,
    })
}
```

- [ ] **Step 5: Update StreamingBlfReader::read_next_batch for new parse signature**

In `src/blf/src/file.rs`, find `read_next_batch` (line 130-153). Change line 148:

```rust
        let (objects, _errors) = self.parser.parse(&self.buffer)?;
```

to:

```rust
        let (objects, _errors, _consumed) = self.parser.parse(&self.buffer)?;
```

- [ ] **Step 6: Build to verify**

Run: `cargo +nightly build -p view 2>&1 | grep -E "^error" | head -10; echo "---"; cargo +nightly build -p view 2>&1 | tail -3`
Expected: no errors, `Finished` with 330 warnings

If errors appear, check for other callers of `parser.parse` (e.g. tests in `parser.rs`, `test_blf_tool.rs`). Update each to destructure the new 3-tuple.

- [ ] **Step 7: Search and fix any other callers of parse**

Run: `grep -rn "parser.parse\|\.parse(&self.buffer\|BlfParser.*parse" src/ tests/ --include="*.rs" 2>&1 | head -20`

For each match (except the two in `file.rs` already updated), update the destructure to add the third element `_consumed` (or `consumed` if needed).

Re-run: `cargo +nightly build -p view 2>&1 | tail -3` → expect `Finished`

- [ ] **Step 8: Commit**

```bash
git add src/blf/src/file.rs src/blf/src/parser.rs
git commit -m "$(cat <<'EOF'
feat(blf): track bytes_total and bytes_consumed in BlfResult

BlfResult gains two u64 fields: bytes_total (file size on disk) and
bytes_consumed (cursor position after parse). BlfParser::parse now
returns (objects, errors, consumed_bytes) so the caller can sum it
with the FileStatistics header size. StreamingBlfReader and any test
callers updated to destructure the new 3-tuple (consuming the third
element with _consumed where unused).

The values are not yet surfaced in the UI — Task 4 adds the StatusBar
display.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

## Task 3: Wrap reads with .context() in 5 BLF files

**Files:**
- Modify: `src/blf/src/file.rs` (wrap `FileStatistics::read` and `parser.parse` calls)
- Modify: `src/blf/src/file_statistics.rs` (`FileStatistics::read` body)
- Modify: `src/blf/src/object_header.rs` (`ObjectHeader::read` body)
- Modify: `src/blf/src/objects/log_container.rs` (`LogContainer::read` body)
- Modify: `src/blf/src/parser.rs` (read calls in `parse` and `parse_inner_objects`)

**Interfaces:**
- Consumes: `BlfResultContext` trait from Task 1, `BlfParseError::context` method from Task 1
- Produces: every parse error carries a `structure.field` context in its Display output

- [ ] **Step 1: Verify baseline build passes**

Run: `cargo +nightly build -p view 2>&1 | tail -3`
Expected: `Finished` with 330 warnings

- [ ] **Step 2: Add .context() to file.rs**

In `src/blf/src/file.rs`, find `read_blf_from_file` (now with the Task 2 body). Change:

```rust
    let file_stats = FileStatistics::read(&mut cursor)?;
```

to:

```rust
    let file_stats = FileStatistics::read(&mut cursor).context("FileStatistics")?;
```

And:

```rust
    let (objects, errors, parse_consumed) = parser.parse(remaining_data)?;
```

to:

```rust
    let (objects, errors, parse_consumed) = parser.parse(remaining_data).context("BlfParser")?;
```

Add the `BlfResultContext` trait to the import at the top of `file.rs`:

```rust
use crate::error::{BlfParseError, BlfParseResult, BlfResultContext};
```

(If the existing import is `use crate::{BlfParseError, BlfParseResult};`, update accordingly — verify by reading the imports.)

- [ ] **Step 3: Add .context() to file_statistics.rs**

In `src/blf/src/file_statistics.rs`, find `FileStatistics::read`. For each `cursor.read_uX::<LittleEndian>()?` and `cursor.read_u8()?` and similar, change `?` to `.map_err(BlfParseError::IoError).context("FileStatistics.<field>")?` where `<field>` is the semantic name of the field being read.

Concrete changes (read the function first to verify line numbers):

- `let signature = cursor.read_u32::<LittleEndian>()?;` → `let signature = cursor.read_u32::<LittleEndian>().map_err(BlfParseError::IoError).context("FileStatistics.signature")?;`
- `let statistics_size = cursor.read_u32::<LittleEndian>()?;` → `.map_err(BlfParseError::IoError).context("FileStatistics.statistics_size")?`
- `let api_number = cursor.read_u32::<LittleEndian>()?;` → `.context("FileStatistics.api_number")?`
- `let application_id = cursor.read_u8()?;` → `.context("FileStatistics.application_id")?`
- `let compression_level = cursor.read_u8()?;` → `.context("FileStatistics.compression_level")?`
- `let application_major = cursor.read_u8()?;` → `.context("FileStatistics.application_major")?`
- `let application_minor = cursor.read_u8()?;` → `.context("FileStatistics.application_minor")?`
- `let file_size = cursor.read_u64::<LittleEndian>()?;` → `.context("FileStatistics.file_size")?`
- `let uncompressed_file_size = cursor.read_u64::<LittleEndian>()?;` → `.context("FileStatistics.uncompressed_file_size")?`
- `let object_count = cursor.read_u32::<LittleEndian>()?;` → `.context("FileStatistics.object_count")?`
- `let application_build = cursor.read_u32::<LittleEndian>()?;` → `.context("FileStatistics.application_build")?`

Also wrap the SystemTime::read calls:
- `let measurement_start_time = SystemTime::read(cursor)?;` → `let measurement_start_time = SystemTime::read(cursor).context("FileStatistics.measurement_start_time")?;`
- `let last_object_time = SystemTime::read(cursor)?;` → `let last_object_time = SystemTime::read(cursor).context("FileStatistics.last_object_time")?;`

For the reserved/rest bytes read with `cursor.read_exact(&mut _rest)?`, wrap with `.map_err(BlfParseError::IoError).context("FileStatistics.reserved")?`.

Add the trait import at the top of `file_statistics.rs`:
```rust
use crate::error::BlfResultContext;
```
or include it in the existing crate import. The `BlfParseError` import already exists.

- [ ] **Step 4: Add .context() to object_header.rs**

In `src/blf/src/object_header.rs`, find `ObjectHeader::read`. Wrap each read:

- `let signature = cursor.read_u32::<LittleEndian>()?;` → `.map_err(BlfParseError::IoError).context("ObjectHeader.signature")?`
- `let header_size = cursor.read_u16::<LittleEndian>()?;` → `.context("ObjectHeader.header_size")?`
- `let header_version = cursor.read_u16::<LittleEndian>()?;` → `.context("ObjectHeader.header_version")?`
- `let object_size = cursor.read_u32::<LittleEndian>()?;` → `.context("ObjectHeader.object_size")?`
- `let object_type_raw = cursor.read_u32::<LittleEndian>()?;` → `.context("ObjectHeader.object_type")?`
- `let object_flags = cursor.read_u32::<LittleEndian>()?;` → `.context("ObjectHeader.object_flags")?`
- `let client_index = cursor.read_u16::<LittleEndian>()?;` → `.context("ObjectHeader.client_index")?`
- `let object_version = cursor.read_u16::<LittleEndian>()?;` → `.context("ObjectHeader.object_version")?`
- `let object_time_stamp = cursor.read_u64::<LittleEndian>()?;` → `.context("ObjectHeader.object_time_stamp")?`

If header_version == 2 reads additional fields (original_time_stamp, time_stamp_status), wrap those:
- `.context("ObjectHeader.original_time_stamp")?`
- `.context("ObjectHeader.time_stamp_status")?`

Add the trait import at the top.

- [ ] **Step 5: Add .context() to log_container.rs**

In `src/blf/src/objects/log_container.rs`, find `LogContainer::read`. Wrap:

- `let compression_method = cursor.read_u16::<LittleEndian>()?;` → `.map_err(BlfParseError::IoError).context("LogContainer.compression_method")?`
- `let _reserved1 = cursor.read_u16::<LittleEndian>()?;` → `.context("LogContainer.reserved1")?`
- `let _reserved2 = cursor.read_u32::<LittleEndian>()?;` → `.context("LogContainer.reserved2")?`
- `let uncompressed_size = cursor.read_u32::<LittleEndian>()? as usize;` → keep the `as usize` cast: `.map_err(BlfParseError::IoError).context("LogContainer.uncompressed_size")? as usize`
- `let _reserved3 = cursor.read_u32::<LittleEndian>()?;` → `.context("LogContainer.reserved3")?`
- `cursor.read_exact(&mut compressed_data)?;` → `.map_err(BlfParseError::IoError).context("LogContainer.data")?`
- `decoder.read_to_end(&mut uncompressed)?;` → `.map_err(BlfParseError::IoError).context("LogContainer.zlib_decode")?`

Add the trait import at the top.

- [ ] **Step 6: Add .context() to parser.rs**

In `src/blf/src/parser.rs`, find `BlfParser::parse` and `parse_inner_objects`.

In `parse`:
- The `LogContainer::read(&mut cursor, header.clone())?` call → wrap with `.context("BlfParser.LogContainer")?`
- Any `parse_inner_objects(&mut container_cursor)?` calls → `.context("BlfParser.parse_inner_objects")?`
- Any other `?` returning BlfParseError → check each and wrap appropriately

In `parse_inner_objects`:
- `let header = match ObjectHeader::read(cursor) { ... }` → the `ObjectHeader::read` already returns `BlfParseResult`, but the outer match has branches that return Err. For the `Err(e) => { all_errors.push(e); ... }` branch, wrap the pushed error with `.context("BlfParser.ObjectHeader")` before pushing.

Read the actual code in `parse_inner_objects` (lines 473-552) to identify each `?` and `Err(e)` push, and add `.context("BlfParser.X")` to each.

Add the trait import at the top of `parser.rs`:
```rust
use crate::error::BlfResultContext;
```

- [ ] **Step 7: Build to verify**

Run: `cargo +nightly build -p view 2>&1 | grep -E "^error" | head -10; echo "---"; cargo +nightly build -p view 2>&1 | tail -3`
Expected: no errors, `Finished` with 330 warnings (or fewer)

If you see "no method named `context` found for BlfParseResult", check that you added `use crate::error::BlfResultContext;` to the file.

- [ ] **Step 8: Verify clippy count**

Run: `cargo +nightly clippy -p view 2>&1 | grep -c "^warning"`
Expected: ≤ 330

- [ ] **Step 9: Commit**

```bash
git add src/blf/src/file.rs src/blf/src/file_statistics.rs src/blf/src/object_header.rs src/blf/src/objects/log_container.rs src/blf/src/parser.rs
git commit -m "$(cat <<'EOF'
refactor(blf): wrap reads with .context() across 5 BLF files

Every read in FileStatistics, ObjectHeader, LogContainer, BlfParser,
and the read_blf_from_file entry point now carries a "structure.field"
context via the new BlfResultContext trait. A failed read produces
errors like "FileStatistics.signature: Invalid BLF file magic string"
or "LogContainer.compression_method: Unsupported compression method: 3".

No new variants or type changes — only .context() calls added on
existing ?-chains and error-push sites.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

## Task 4: Surface bytes_total/consumed in CanViewApp state + apply_blf_result

**Files:**
- Modify: `src/view/src/app/state.rs` (struct `CanViewApp` field block; `new_with_maximized_state_and_bounds`)
- Modify: `src/view/src/app/impls.rs` (two `Self { ... }` literals that also need the new fields; `apply_blf_result` Ok and Err paths)

**Interfaces:**
- Consumes: `BlfResult::bytes_total` and `BlfResult::bytes_consumed` from Task 2
- Produces: `CanViewApp::blf_bytes_total: u64`, `CanViewApp::blf_bytes_consumed: u64` (pub); set in apply_blf_result Ok, reset to 0 in Err; status_msg shows `⚠ N parse errors` when errors is non-empty

- [ ] **Step 1: Verify baseline build passes**

Run: `cargo +nightly build -p view 2>&1 | tail -3`
Expected: `Finished` with 330 warnings

- [ ] **Step 2: Add fields to CanViewApp struct**

In `src/view/src/app/state.rs`, find the `CanViewApp` struct (around line 70). After the `pub library_picker_selected_version: std::collections::HashMap<String, String>,` line (added in the previous redesign), insert:

```rust
    // BLF file size and parser-consumed bytes (for StatusBar progress)
    pub blf_bytes_total: u64,
    pub blf_bytes_consumed: u64,
```

- [ ] **Step 3: Initialize the fields in new_with_maximized_state_and_bounds**

In `src/view/src/app/state.rs`, find `new_with_maximized_state_and_bounds` (around line 251). In the `Self { ... }` block, after `library_picker_selected_version: std::collections::HashMap::new(),`, insert:

```rust
            blf_bytes_total: 0,
            blf_bytes_consumed: 0,
```

- [ ] **Step 4: Initialize the fields in the other two constructors in impls.rs**

Run: `grep -n "library_picker_selected_version: std::collections::HashMap::new()," src/view/src/app/impls.rs`
Expected: 2 matches (around lines 36 and 590)

For each match, insert on the line after:
```rust
            blf_bytes_total: 0,
            blf_bytes_consumed: 0,
```

- [ ] **Step 5: Set the fields in apply_blf_result Ok path**

In `src/view/src/app/impls.rs`, find `apply_blf_result` (around line 230). In the `Ok(result) => { ... }` arm, find the existing line that sets `self.current_file_name = file_name;` and `self.library_picker_dismissed = false;`. After those lines, add:

```rust
                self.blf_bytes_total = result.bytes_total;
                self.blf_bytes_consumed = result.bytes_consumed;
```

Then find the existing `if error_count > 0 { ... } else { ... }` block (around line 247-260). The existing code sets `status_msg` to either a warning or success message. Leave that as-is — the warning already includes `Loaded N messages | M errors`. The new blf_bytes fields are read by StatusBar in Task 5.

- [ ] **Step 6: Reset the fields in apply_blf_result Err path**

In the same `apply_blf_result`, find the `Err(e) => { ... }` arm (around line 273). After `Self::display_blf_load_error(&e);`, add:

```rust
                self.blf_bytes_total = 0;
                self.blf_bytes_consumed = 0;
```

- [ ] **Step 7: Build to verify**

Run: `cargo +nightly build -p view 2>&1 | tail -3`
Expected: `Finished` with 330 warnings (the new fields will show "never read" warnings until Task 5 uses them — acceptable for this commit)

- [ ] **Step 8: Commit**

```bash
git add src/view/src/app/state.rs src/view/src/app/impls.rs
git commit -m "$(cat <<'EOF'
feat(app): surface blf_bytes_total/consumed in CanViewApp

Adds two u64 fields tracking the BLF file size and the parser's
consumed bytes. Set in apply_blf_result's Ok path from BlfResult;
reset to 0 in the Err path. Consumed by the StatusBar in Task 5
to render a "521.0KB / 521.0KB (100%)" progress segment.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

## Task 5: StatusBar displays bytes progress + format_bytes helper

**Files:**
- Modify: `src/view/src/ui/components/status_bar.rs` (add `format_bytes` fn + `render_blf_progress_segment` fn; insert into `render_status_bar`)

**Interfaces:**
- Consumes: `CanViewApp::blf_bytes_total`, `CanViewApp::blf_bytes_consumed` from Task 4
- Produces: a new StatusBar segment showing "521.0KB / 521.0KB (100%)" between the file name and message count

- [ ] **Step 1: Verify baseline build passes**

Run: `cargo +nightly build -p view 2>&1 | tail -3`
Expected: `Finished` with 330 warnings

- [ ] **Step 2: Add the format_bytes helper**

In `src/view/src/ui/components/status_bar.rs`, find the existing `format_count` function (near the top, around line 11). After it, add:

```rust
/// Format a byte count with units (1024-based): 0B, 1023B, 1.0KB, 1.5MB, 2.3GB.
pub fn format_bytes(n: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;
    if n >= GB {
        format!("{:.1}GB", n as f64 / GB as f64)
    } else if n >= MB {
        format!("{:.1}MB", n as f64 / MB as f64)
    } else if n >= KB {
        format!("{:.1}KB", n as f64 / KB as f64)
    } else {
        format!("{}B", n)
    }
}
```

- [ ] **Step 3: Add unit tests for format_bytes**

In `src/view/src/ui/components/status_bar.rs`, find the existing `#[cfg(test)] mod tests` block at the end. Append:

```rust
    #[test]
    fn test_format_bytes_zero() {
        assert_eq!(format_bytes(0), "0B");
    }

    #[test]
    fn test_format_bytes_small() {
        assert_eq!(format_bytes(1023), "1023B");
    }

    #[test]
    fn test_format_bytes_exact_kb() {
        assert_eq!(format_bytes(1024), "1.0KB");
    }

    #[test]
    fn test_format_bytes_kb_decimal() {
        assert_eq!(format_bytes(1536), "1.5KB");
    }

    #[test]
    fn test_format_bytes_exact_mb() {
        assert_eq!(format_bytes(1024 * 1024), "1.0MB");
    }

    #[test]
    fn test_format_bytes_mb_decimal() {
        assert_eq!(format_bytes(2_400_819), "2.3MB");
    }

    #[test]
    fn test_format_bytes_exact_gb() {
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.0GB");
    }

    #[test]
    fn test_format_bytes_gb_decimal() {
        assert_eq!(format_bytes(2_700_000_000), "2.5GB");
    }
```

- [ ] **Step 4: Add render_blf_progress_segment function**

In `src/view/src/ui/components/status_bar.rs`, find `render_file_segment` (around line 25). After it (or after `render_separator`), add:

```rust
/// Render the BLF bytes-progress segment: "521.0KB / 521.0KB (100%)".
/// Returns None when no file is loaded (blf_bytes_total == 0) so the
/// caller can skip rendering via .when_some().
fn render_blf_progress_segment(app: &CanViewApp) -> Option<impl IntoElement> {
    if app.blf_bytes_total == 0 {
        return None;
    }
    let total = format_bytes(app.blf_bytes_total);
    let consumed = format_bytes(app.blf_bytes_consumed);
    let pct = if app.blf_bytes_total > 0 {
        (app.blf_bytes_consumed as f64 / app.blf_bytes_total as f64) * 100.0
    } else {
        100.0
    };
    let text = format!("{} / {} ({:.1}%)", consumed, total, pct);
    // Color: green at 100%, yellow if < 100% (partial parse)
    let color = if pct >= 100.0 {
        colors::TEXT_SECONDARY
    } else {
        colors::WARNING
    };
    Some(div().text_color(color).child(text))
}
```

- [ ] **Step 5: Insert the segment into render_status_bar**

In `src/view/src/ui/components/status_bar.rs`, find `render_status_bar` (around line 230). Find the left side child block that currently has `render_file_segment`, `render_separator`, message count, etc.

Insert the BLF progress segment right after `render_file_segment` and its separator, before the message count:

```rust
        // Left side: Log/Plot toggle | file | BLF progress | msgs | DBC | LDF
        .child(
            div()
                .flex()
                .items_center()
                .gap(spacing::SM)
                .child(render_data_view_toggle(app, view.clone()))
                .child(render_separator())
                .child(render_file_segment(app))
                .when_some(render_blf_progress_segment(app), |el, seg| {
                    el.child(render_separator()).child(seg)
                })
                .child(render_separator())
                .child(
                    div()
                        .text_color(colors::TEXT_MUTED)
                        .child(format!("{} msgs", format_count(app.messages.len()))),
                )
                .child(render_separator())
                .child(
                    div()
                        .text_color(colors::TEXT_MUTED)
                        .child(format!("DBC: {}", app.dbc_channels.len())),
                )
                .child(render_separator())
                .child(
                    div()
                        .text_color(colors::TEXT_MUTED)
                        .child(format!("LDF: {}", app.ldf_channels.len())),
                ),
        )
```

Read the existing structure first — the `render_data_view_toggle(app, view.clone())` call is already present (added in earlier work). Just insert the `render_blf_progress_segment` block with surrounding separators between `render_file_segment` and the message count separator.

- [ ] **Step 6: Build to verify**

Run: `cargo +nightly build -p view 2>&1 | tail -3`
Expected: `Finished` with 330 warnings or fewer

- [ ] **Step 7: Verify clippy count**

Run: `cargo +nightly clippy -p view 2>&1 | grep -c "^warning"`
Expected: ≤ 330

- [ ] **Step 8: Run release build and launch**

Run: `cargo +nightly build --release -p view 2>&1 | tail -3`
Expected: `Finished` release` profile [optimized] target(s)`

Then rebuild the .app bundle and open:

```bash
pkill -f "CANVIEW.app\|target/release/view" 2>/dev/null
rm -rf target/release/CANVIEW.app
mkdir -p target/release/CANVIEW.app/Contents/{MacOS,Resources}
cp target/release/view target/release/CANVIEW.app/Contents/MacOS/canview
chmod +x target/release/CANVIEW.app/Contents/MacOS/canview
cp assets/ico/canview.icns target/release/CANVIEW.app/Contents/Resources/canview.icns
{
  printf '%s\n' '<?xml version="1.0" encoding="UTF-8"?>'
  printf '%s\n' '<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">'
  printf '%s\n' '<plist version="1.0">'
  printf '%s\n' '<dict>'
  printf '%s\n' '  <key>CFBundleDevelopmentRegion</key><string>en</string>'
  printf '%s\n' '  <key>CFBundleExecutable</key><string>canview</string>'
  printf '%s\n' '  <key>CFBundleIdentifier</key><string>com.canview.app</string>'
  printf '%s\n' '  <key>CFBundleInfoDictionaryVersion</key><string>6.0</string>'
  printf '%s\n' '  <key>CFBundleName</key><string>CANVIEW</string>'
  printf '%s\n' '  <key>CFBundlePackageType</key><string>APPL</string>'
  printf '%s\n' '  <key>CFBundleShortVersionString</key><string>0.1.0</string>'
  printf '%s\n' '  <key>CFBundleVersion</key><string>0.1.0</string>'
  printf '%s\n' '  <key>LSMinimumSystemVersion</key><string>10.13</string>'
  printf '%s\n' '  <key>NSHighResolutionCapable</key><true/>'
  printf '%s\n' '  <key>CFBundleIconFile</key><string>canview</string>'
  printf '%s\n' '</dict>'
  printf '%s\n' '</plist>'
} > target/release/CANVIEW.app/Contents/Info.plist
open target/release/CANVIEW.app
```

Expected: app opens, no panic.

- [ ] **Step 9: Manual verification (3 scenarios)**

1. Open sample.blf (1216 bytes, 21 messages). StatusBar should show "1.2KB / 1.2KB (100.0%)" in the left segment, between file name and message count.

2. Try opening test_corrupted.blf. StatusBar should show partial progress like "0.5KB / 1.2KB (41.7%)" in WARNING color (yellow), and status_msg on the right should display "⚠ N parse errors" (already implemented by the existing `error_count > 0` branch).

3. Try opening a non-BLF file (e.g. a .txt). StatusBar should NOT show the BLF progress segment (bytes_total = 0 → segment hidden). status_msg should display "❌ File Error: FileStatistics.signature: Invalid BLF file magic string" — confirming the context chain works.

If any scenario fails, debug before committing.

- [ ] **Step 10: Commit**

```bash
git add src/view/src/ui/components/status_bar.rs
git commit -m "$(cat <<'EOF'
feat(ui): show bytes progress in StatusBar with format_bytes helper

Adds a "521.0KB / 521.0KB (100.0%)" segment to the StatusBar left
side, between the file name and the message count. The segment is
hidden when no file is loaded (bytes_total == 0). Partial parses
(bytes_consumed < bytes_total) show in WARNING color.

format_bytes() formats 1024-based: 0B / 1023B / 1.0KB / 1.5MB /
1.0GB. Eight unit tests cover the boundaries.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

## Self-Review

**Spec coverage:**
- §1 architecture (BlfResult bytes fields, Context variant, data flow) → Tasks 1, 2, 4 ✓
- §2 error context (Context variant, helper, trait, Display recursion, 5-file wrap) → Tasks 1, 3 ✓
- §3 StatusBar display (new segment, format_bytes, no popover) → Task 5 ✓
- §4 implementation order (4 commits) → Tasks 1-5 ✓ (Task 5 is 2 commits worth but combined; spec said 4 but acceptable)
- §5 out of scope (no streaming, no popover, no RuntimeState) → respected ✓
- §6 risks (parse signature change, From<io::Error> kept) → Task 2 preserves From impl ✓

**Placeholder scan:** No TBD/TODO/implement-later. All code blocks show actual code. No "add appropriate error handling".

**Type consistency:**
- `BlfParseError::Context { inner: Box<BlfParseError>, ctx: String }` — used consistently
- `BlfResultContext<T>` trait — same name in Tasks 1, 3
- `BlfResult::bytes_total: u64` / `bytes_consumed: u64` — same in Tasks 2, 4
- `CanViewApp::blf_bytes_total: u64` / `blf_bytes_consumed: u64` — same in Tasks 4, 5
- `format_bytes(n: u64) -> String` — same in Task 5 step 2 and tests

**Scope check:** Single subsystem (BLF progress + errors). No decomposition needed.

---

## Execution Handoff

Plan complete and saved to `docs/superpowers/plans/2026-07-23-blf-progress-and-errors.md`. Two execution options:

1. **Subagent-Driven (recommended)** — I dispatch a fresh subagent per task, review between tasks, fast iteration
2. **Inline Execution** — Execute tasks in this session using executing-plans, batch execution with checkpoints

Which approach?
