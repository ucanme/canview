# BLF Read Progress + Precise Error Reporting — Design Spec

- **日期**:2026-07-23
- **目标**:StatusBar 显示 BLF 加载字节进度,解析错误信息含"结构名.字段名"前缀
- **范围**:`blf` crate 的 `BlfParseError` + `BlfResult` + 5 个 read 文件加 `.context()`;`view` crate 的 `CanViewApp` + `StatusBar` 显示

---

## 1. 整体架构

两条独立改动:

1. **字节进度**:`BlfResult` 加 `bytes_total: u64` + `bytes_consumed: u64`,在 `read_blf_from_file` 末尾计算。加载完成后 StatusBar 显示,不涉及实时进度(无后台线程)。

2. **错误上下文**:`BlfParseError` 加 `Context { inner: Box<BlfParseError>, ctx: String }` 变体,通过 `.context("FileStatistics.signature")?` 给 `?` 链上每个 read 加上下文。Display 递归打印完整链:`FileStatistics.signature: Invalid BLF file magic string`。

### 数据流

```
read_blf_from_file(path)
  → fs::read(path) → data: Vec<u8>, bytes_total = data.len() as u64
  → FileStatistics::read(&mut cursor).context("FileStatistics")? → file_stats
    (cursor advances past FileStatistics header)
  → remaining_data = &data[cursor.position() as usize..]
  → BlfParser::parse(remaining_data).context("BlfParser")? → (objects, errors)
  → BlfParser tracks its own internal cursor; on success returns consumed bytes
    via a new field in BlfParseResult tuple: (objects, errors, bytes_consumed_in_remaining)
  → bytes_consumed = cursor.position() (FileStatistics) + bytes_consumed_in_remaining
  → Ok(BlfResult { file_stats, objects, errors, bytes_total, bytes_consumed })

apply_blf_result → CanViewApp
  → app.blf_bytes_total = bytes_total
  → app.blf_bytes_consumed = bytes_consumed
  → app.status_msg 显示 "⚠ N parse errors" if errors 非空
  → StatusBar reads app.blf_bytes_total + app.blf_bytes_consumed
  → 显示 "521.0KB / 521.0KB (100%)" 或 "500.0KB / 521.0KB (95.9%)"
```

### bytes_consumed 计算

`BlfParser::parse` 当前签名 `fn parse(&self, data: &[u8]) -> BlfParseResult<(Vec<LogObject>, Vec<BlfParseError>)>`,内部 cursor 不暴露。两个选项:

1. **改 parse 返回元组**:增加 `u64` 字段表示 consumed 字节数。需要更新所有调用方(主要 `read_blf_from_file`)。
2. **不修改 parse,在 read_blf_from_file 用另一个 cursor 追踪**:不实际,因为 parse 自己创建 cursor。

采用选项 1:把 `parse` 返回类型改成 `BlfParseResult<(Vec<LogObject>, Vec<BlfParseError>, u64)>`,第三个 u64 是 cursor 在 data 末尾的位置(consumed bytes)。Err 路径:parse 内部循环遇错会 advance cursor 并 continue,所以 consumed 始终是 cursor 在循环结束时的位置。

### 新 state 字段(CanViewApp)

```rust
pub blf_bytes_total: u64,
pub blf_bytes_consumed: u64,
```

初始化为 0 / 0,`apply_blf_result` Ok 路径设置,Err 路径清零。**不放进 RuntimeState**(临时状态,跨窗口重建不重要)。错误数量通过现有 `status_msg` 显示 `⚠ N parse errors`,不存 `Vec<String>`。

---

## 2. 错误上下文实现

### BlfParseError 新变体

```rust
pub enum BlfParseError {
    IoError(io::Error),
    InvalidFileMagic,
    InvalidContainerMagic,
    UnexpectedEof,
    UnsupportedCompression(u16),
    UnknownHeaderVersion(u16),
    /// Wraps another error with a context string describing which
    /// structure/field was being read when the error occurred.
    Context {
        inner: Box<BlfParseError>,
        ctx: String,
    },
}
```

### Helper 方法

```rust
impl BlfParseError {
    /// Wrap this error with a context describing where it occurred.
    pub fn context(self, ctx: impl Into<String>) -> Self {
        Self::Context {
            inner: Box::new(self),
            ctx: ctx.into(),
        }
    }
}
```

### Extension trait

```rust
pub trait BlfResultContext<T> {
    fn context(self, ctx: impl Into<String>) -> BlfParseResult<T>;
}

impl<T> BlfResultContext<T> for BlfParseResult<T> {
    fn context(self, ctx: impl Into<String>) -> BlfParseResult<T> {
        self.map_err(|e| e.context(ctx))
    }
}
```

### Display 递归打印

```rust
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
```

### Error::source

```rust
impl Error for BlfParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            BlfParseError::Context { inner, .. } => Some(inner.as_ref()),
            BlfParseError::IoError(e) => Some(e),
            _ => None,
        }
    }
}
```

### 加 .context() 的位置

| 文件 | 函数 | 加 context 的字段 |
|---|---|---|
| `file.rs` | `read_blf_from_file` | `FileStatistics::read` + `parser.parse` |
| `file_statistics.rs` | `FileStatistics::read` | signature / statistics_size / api_number / application_id / compression_level / application_major / application_minor / file_size / uncompressed_file_size / object_count / application_build / measurement_start_time / last_object_time |
| `object_header.rs` | `ObjectHeader::read` | signature / header_size / header_version / object_size / object_type / object_flags / client_index / object_version / object_time_stamp / original_time_stamp / time_stamp_status |
| `log_container.rs` | `LogContainer::read` | compression_method / reserved1 / reserved2 / uncompressed_size / reserved3 / data |
| `parser.rs` | `BlfParser::parse` + `parse_inner_objects` | ObjectHeader::read + 各 message 类型 read(CanMessage::read 等) |

### 典型示例

```rust
// file_statistics.rs (现状)
let signature = cursor.read_u32::<LittleEndian>()?;
if signature != FILE_SIGNATURE {
    return Err(BlfParseError::InvalidFileMagic);
}
let statistics_size = cursor.read_u32::<LittleEndian>()?;

// 改后
let signature = cursor
    .read_u32::<LittleEndian>()
    .map_err(BlfParseError::IoError)
    .context("FileStatistics.signature")?;
if signature != FILE_SIGNATURE {
    return Err(BlfParseError::InvalidFileMagic.context("FileStatistics.signature"));
}
let statistics_size = cursor
    .read_u32::<LittleEndian>()
    .map_err(BlfParseError::IoError)
    .context("FileStatistics.statistics_size")?;
```

注意:`read_u32::<LittleEndian>()?` 现有写法把 io::Error 转 BlfParseError::IoError,但没上下文。改后用 `.map_err(BlfParseError::IoError).context(...)?` 给 io 错误加 context,业务错误用 `.context(...)` 包。

---

## 3. StatusBar 显示

### 新增显示段

StatusBar 左侧现有:`[Log | Plot] | ?? 文件名 | msg 数 | DBC | LDF`

加一个新的"BLF 加载状态"段,放在文件名段后:

```
[Log | Plot] | ?? convert.blf  521.0KB / 521.0KB (100%) | 72,821 msgs | DBC: 3 | LDF: 2
```

如果有解析错误:

```
[Log | Plot] | ?? convert.blf  500.0KB / 521.0KB (95.9%) | 72,821 msgs | DBC: 3 | LDF: 2
```
(右侧 status_msg 显示 `⚠ 3 parse errors`)

### 显示规则

| 加载状态 | 显示 |
|---|---|
| 未加载文件 | 不显示这一段 |
| 100% 成功 | `521.0KB / 521.0KB (100%)` |
| 部分成功 | `500.0KB / 521.0KB (95.9%)`(status_msg 显示 `⚠ N parse errors`) |
| 完全失败 | 不显示这段;status_msg 显示 `❌ File Error: FileStatistics.signature: Invalid BLF file magic string` |

### 字节格式化

```rust
fn format_bytes(n: u64) -> String {
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

放在 `status_bar.rs` 模块,加单元测试。

### 百分比

```rust
let pct = if bytes_total > 0 {
    (bytes_consumed as f64 / bytes_total as f64) * 100.0
} else {
    100.0
};
format!("{:.1}%", pct)
```

### state 字段(CanViewApp 新增)

```rust
pub blf_bytes_total: u64,
pub blf_bytes_consumed: u64,
```

不加 popover/详细错误列表 — 错误数量通过 `status_msg = format!("⚠ {} parse errors", n)` 显示,详细错误信息打到 stderr(eprintln! in apply_blf_result's log_blf_errors call)供开发者用 Console.app 查看。

---

## 4. 实施步骤与测试

### 实施顺序(4 个 commit)

| # | commit 标题 | 内容 |
|---|---|---|
| 1 | `feat(blf): add Context variant to BlfParseError` | 加 Context 变体 + context() helper + BlfResultContext trait + Display 递归 + Error::source 更新 |
| 2 | `feat(blf): add bytes_total/consumed to BlfResult` | BlfResult 加两字段 + 改 `BlfParser::parse` 返回元组加 consumed 字节 + read_blf_from_file 末尾组合计算 |
| 3 | `refactor(blf): wrap reads with .context() in 5 files` | file.rs + file_statistics.rs + object_header.rs + log_container.rs + parser.rs 各 read 调用加 .context()(含 parse 内部各 read) |
| 4 | `feat(ui): show bytes progress + error count in StatusBar` | CanViewApp 加 blf_bytes_total/consumed + apply_blf_result 设置 + status_msg 显示错误数 + StatusBar 加 format_bytes 显示段 + 单元测试 |

### 测试

- **编译**:`cargo +nightly build -p view` 通过
- **Clippy**:`cargo +nightly clippy -p view` warnings ≤ 330(baseline)
- **运行验证**:
  1. 加载 sample.blf(1216 bytes,21 条消息)→ StatusBar 显示 `1.2KB / 1.2KB (100%)`
  2. 加载 test_corrupted.blf → StatusBar 显示部分进度 + status_msg 显示 `⚠ N parse errors`
  3. 加载非 BLF 文件(随便选个 .txt) → status_msg 显示 `❌ File Error: FileStatistics.signature: Invalid BLF file magic string`
- **单元测试**:
  - `format_bytes` 测试:0B / 1B / 1023B / 1024B / 1.5KB / 1.0MB / 1.0GB
  - `BlfParseError::context` 链测试:`InvalidFileMagic.context("FileStatistics.signature")` 的 Display 输出 = `"FileStatistics.signature: Invalid BLF file magic string"`

### 验收标准

- StatusBar 在加载文件后显示字节进度段
- 错误信息含 "结构名.字段名" 前缀
- `cargo +nightly build -p view` 通过
- 4 个 commit 各自独立可编译
- `format_bytes` 单元测试覆盖 0/1B/1.5KB/1.0MB/1.0GB 边界

---

## 5. 不在本次范围

- 不改 read_blf_from_file 为流式读取(不引入实时进度)
- 不加 popover/详细错误列表 UI
- 不动 `RuntimeState` 保存 blf_bytes_*
- 不改现有 `BlfParseError` 变体的语义(只新增 Context)
- 不改 `StreamingBlfReader`(未使用)
- 不动 BLF parser 的整体结构,只在 read 调用处加 .context

---

## 6. 风险与缓解

| 风险 | 缓解 |
|---|---|
| 加 .context() 让代码冗长 | 用 helper trait 让 `?` 链保持一行 |
| Context 嵌套过深 | Display 递归打印,不限制嵌套层数 |
| 现有 `?` 自动转换 io::Error → IoError 的 From impl 被破坏 | 保留 From impl,改用 `.map_err(BlfParseError::IoError).context(...)?` 显式两步 |
| bytes_consumed 计算不准(parser 中途 Err 返回) | parse 完成后读 cursor.position();Err 路径用 try 的 cursor 位置(需要把 cursor 移到外层) |
| 单元测试不能跑(SIGBUS) | 测试编译过即可,CI 跑实际执行 |
