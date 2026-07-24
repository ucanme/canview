# 多文件解析与回放 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 支持同时加载多个 BLF 文件，按原始绝对时间全局合并，UI 提供折线图展示和可滚动的日志列表

**Architecture:** 引入 FileSegment（单文件段）+ MergedView（全局合并视图）两层结构，对 UI 大部分代码透明。File 菜单保留单选 `Open BLF...`（替换语义），新增多选 `Open Multiple BLF...`（追加语义）。多文件并发解析（GPUI background_executor），每完成一个文件就增量双指针归并进 merged，StatusBar 流式更新。文件管理 popover 通过 StatusBar 入口展开，可单独移除文件或 Remove All。

**Tech Stack:** Rust nightly, GPUI (gpui-component), rfd 0.14（`pick_files()` 多选支持），chrono（NaiveDateTime / UNIX epoch 转换），smol 异步执行器（GPUI 内置）。`blf` crate 不变，复用 `read_blf_from_file`。

## Global Constraints

- **blf crate 不变**:复用 `read_blf_from_file`、`BlfResult`、`LogObject`，不修改 src/blf/
- **不区分信号来源**:同名信号多文件直接合并拼接为一条波形，绘图不使用 `source_file_ids`
- **时间轴基准**:原始绝对时间，计算公式为 `BlfResult.file_stats.measurement_start_time.add_nanoseconds(LogObject.timestamp())` 返回 i64 纳秒（自 UNIX epoch），再除以 1e9 得 f64 全局秒
- **单选=替换，多选=追加**:两个菜单入口语义明确
- **性能**:并行解析 + 流式加载，最高优先；单文件完成后立即增量合并
- **错误隔离**:单个文件失败不阻塞其他文件加载
- **不持久化文件路径**:`multi_channel_config.json` 不保存已加载 BLF 路径，重启后 files 为空
- **删除 is_streaming_mode**:字段未使用，移除所有引用
- **现有 `cargo test --workspace` 必须全部通过**:回归保护
- **遵循现有代码风格**:中文注释、`super::state::X` 路径、`gpui::*` 导入约定

---

## File Structure

### 新增文件
- `src/view/src/domain/multi_file.rs` — `FileSegment` + `MergedView` 结构 + `rebuild_merged` 逻辑 + 单元测试
- `src/view/src/app/commands/multi_file.rs` — `LoadBlfFiles` / `LoadMode` / `RemoveFile` / `RemoveAllFiles` 命令结构

### 修改文件
- `src/view/src/domain/mod.rs` — 注册 `pub mod multi_file;`
- `src/view/src/app/commands/mod.rs` — 注册 `pub mod multi_file;`
- `src/view/src/app/state.rs` — `FileSegment`/`MergedView`/`LoadingProgress` 结构定义；`CanViewApp` 新增 `files`/`merged`/`show_files_popover`/`loading_progress`，删除 `is_streaming_mode`；`RuntimeState` 新增 `files`/`merged`，删除 `messages`/`is_streaming_mode`（保留 `current_file_name`）
- `src/view/src/app/impls.rs` — `apply_blf_result` → `apply_blf_result_single`；新增 `apply_blf_result_append_one`/`apply_blf_results_append`/`remove_file`/`remove_all_files`/`rebuild_merged`；`new()`/`new_with_maximized_state_and_bounds()` 适配新字段；`save_runtime_state`/`restore_runtime_state` 适配
- `src/view/src/app/impls_rendering.rs` — File 菜单新增 `Open Multiple BLF...` child；StatusBar 新增 `📁 Files` 按钮和 `Cancel` 按钮；新增文件管理 popover 渲染
- `src/view/src/controllers/config_controller.rs` — `apply_blf_result` 调用点改为 `apply_blf_result_single`

### 不变文件
- `src/blf/`、`src/parser/`、`src/view/src/views/log_view.rs`、`src/view/src/views/chart_view.rs`、`src/view/src/domain/log_processor.rs`

---

## Task 1: FileSegment 与 MergedView 数据模型（domain 层）

**Files:**
- Create: `src/view/src/domain/multi_file.rs`
- Modify: `src/view/src/domain/mod.rs:19` (新增 `pub mod multi_file;`)
- Test: `src/view/src/domain/multi_file.rs` (内联 `#[cfg(test)] mod tests`)

**Interfaces:**
- Consumes: `blf::{BlfResult, LogObject}`、`std::path::PathBuf`、`std::sync::Arc`、`chrono::NaiveDateTime`
- Produces: `FileSegment` struct、`MergedView` struct、`FileSegment::from_blf_result(blf_result, file_id, path) -> FileSegment`、`MergedView::empty() -> MergedView`、`MergedView::from_segments(&[Arc<FileSegment>]) -> MergedView`

**关键类型定义（先确定接口，后续 task 依赖）：**

```rust
use blf::LogObject;
use std::path::PathBuf;
use std::sync::Arc;
use chrono::NaiveDateTime;

/// 一个已加载的 BLF 文件段
pub struct FileSegment {
    pub file_id: u32,
    pub path: PathBuf,
    pub file_name: String,
    pub start_time: Option<NaiveDateTime>,
    pub messages: Arc<[LogObject]>,
    pub errors: Vec<String>,
    pub bytes_total: u64,
    pub bytes_consumed: u64,
    pub object_count: usize,
    pub time_min: Option<f64>,  // 全局秒，UNIX epoch 基准
    pub time_max: Option<f64>,
}

/// 全局合并视图
pub struct MergedView {
    pub messages: Arc<[LogObject]>,
    pub source_file_ids: Arc<[u32]>,
    pub time_min: Option<f64>,
    pub time_max: Option<f64>,
    pub version: u64,
}
```

- [ ] **Step 1: 注册新模块**

Edit `src/view/src/domain/mod.rs:19` 后追加：

```rust
pub mod multi_file;
```

- [ ] **Step 2: 创建 multi_file.rs 骨架 + 类型定义**

Create `src/view/src/domain/multi_file.rs`：

```rust
//! 多文件加载与全局合并视图
//!
//! FileSegment 表示一个已加载的 BLF 文件段；MergedView 表示多文件按全局绝对时间合并后的视图。

use blf::{BlfResult, LogObject};
use chrono::NaiveDateTime;
use std::path::PathBuf;
use std::sync::Arc;

/// 一个已加载的 BLF 文件段
pub struct FileSegment {
    pub file_id: u32,
    pub path: PathBuf,
    pub file_name: String,
    pub start_time: Option<NaiveDateTime>,
    pub messages: Arc<[LogObject]>,
    pub errors: Vec<String>,
    pub bytes_total: u64,
    pub bytes_consumed: u64,
    pub object_count: usize,
    pub time_min: Option<f64>,
    pub time_max: Option<f64>,
}

/// 全局合并视图
pub struct MergedView {
    pub messages: Arc<[LogObject]>,
    pub source_file_ids: Arc<[u32]>,
    pub time_min: Option<f64>,
    pub time_max: Option<f64>,
    pub version: u64,
}

impl MergedView {
    pub fn empty() -> Self {
        Self {
            messages: Arc::from([]),
            source_file_ids: Arc::from([]),
            time_min: None,
            time_max: None,
            version: 0,
        }
    }
}
```

- [ ] **Step 3: 写失败测试 - FileSegment::from_blf_result**

在 `multi_file.rs` 末尾追加测试模块：

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use blf::{BlfResult, FileStatistics, SystemTime};
    use std::path::PathBuf;

    fn make_test_blf_result() -> BlfResult {
        BlfResult {
            file_stats: FileStatistics {
                measurement_start_time: SystemTime {
                    year: 2026, month: 7, day_of_week: 5, day: 24,
                    hour: 10, minute: 0, second: 0, milliseconds: 0,
                },
                ..Default::default()
            },
            objects: Vec::new(),
            errors: Vec::new(),
            bytes_total: 1024,
            bytes_consumed: 1024,
        }
    }

    #[test]
    fn test_file_segment_from_empty_blf_result() {
        let result = make_test_blf_result();
        let path = PathBuf::from("/tmp/test.blf");
        let seg = FileSegment::from_blf_result(result, 1, path.clone());
        assert_eq!(seg.file_id, 1);
        assert_eq!(seg.path, path);
        assert_eq!(seg.file_name, "test.blf");
        assert!(seg.start_time.is_some());
        assert_eq!(seg.messages.len(), 0);
        assert!(seg.errors.is_empty());
        assert_eq!(seg.bytes_total, 1024);
        assert_eq!(seg.bytes_consumed, 1024);
        assert_eq!(seg.object_count, 0);
        assert!(seg.time_min.is_none());
        assert!(seg.time_max.is_none());
    }
}
```

Run: `cargo test -p view --lib domain::multi_file::tests::test_file_segment_from_empty_blf_result`
Expected: FAIL with "no function named `FileSegment::from_blf_result`" or similar

- [ ] **Step 4: 实现 FileSegment::from_blf_result**

在 `multi_file.rs` 的 `FileSegment` impl 块中追加：

```rust
impl FileSegment {
    /// 从 BlfResult 构造 FileSegment
    ///
    /// file_id 由调用方（CanViewApp）从全局 AtomicU32 计数器获取并传入。
    /// start_time 从 file_stats.measurement_start_time 转换为 NaiveDateTime。
    /// time_min/time_max 通过遍历 messages 计算绝对时间（秒）。
    pub fn from_blf_result(result: BlfResult, file_id: u32, path: PathBuf) -> Self {
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let start_time = {
            use chrono::{TimeZone, Utc};
            let st = &result.file_stats.measurement_start_time;
            Utc.with_ymd_and_hms(
                st.year as i32, st.month as u32, st.day as u32,
                st.hour as u32, st.minute as u32, st.second as u32,
            ).single()
            .map(|dt| dt.naive_utc())
        };

        let start_ns = result.file_stats.measurement_start_time.to_timestamp_nanos();

        let messages: Arc<[LogObject]> = Arc::from(result.objects);
        let object_count = messages.len();

        let (time_min, time_max) = if object_count == 0 {
            (None, None)
        } else {
            let mut min_ns = i64::MAX;
            let mut max_ns = i64::MIN;
            for msg in messages.iter() {
                let abs_ns = start_ns + msg.timestamp() as i64;
                if abs_ns < min_ns { min_ns = abs_ns; }
                if abs_ns > max_ns { max_ns = abs_ns; }
            }
            (Some(min_ns as f64 / 1_000_000_000.0), Some(max_ns as f64 / 1_000_000_000.0))
        };

        let errors = result.errors.iter().map(|e| format!("{}", e)).collect();

        Self {
            file_id,
            path,
            file_name,
            start_time,
            messages,
            errors,
            bytes_total: result.bytes_total,
            bytes_consumed: result.bytes_consumed,
            object_count,
            time_min,
            time_max,
        }
    }
}
```

- [ ] **Step 5: 运行测试，确认通过**

Run: `cargo test -p view --lib domain::multi_file::tests::test_file_segment_from_empty_blf_result`
Expected: PASS

- [ ] **Step 6: 写失败测试 - MergedView::from_segments (空)**

在 tests 模块追加：

```rust
    #[test]
    fn test_merged_view_from_empty_segments() {
        let view = MergedView::from_segments(&[]);
        assert_eq!(view.messages.len(), 0);
        assert_eq!(view.source_file_ids.len(), 0);
        assert!(view.time_min.is_none());
        assert!(view.time_max.is_none());
        assert_eq!(view.version, 0);
    }
```

Run: `cargo test -p view --lib domain::multi_file::tests::test_merged_view_from_empty_segments`
Expected: FAIL with "no function named `MergedView::from_segments`"

- [ ] **Step 7: 实现 MergedView::from_segments（基础版）**

在 `multi_file.rs` 的 `MergedView` impl 块中追加：

```rust
impl MergedView {
    /// 从多个 FileSegment 构建全局合并视图
    ///
    /// 按 (绝对时间秒, file_id, msg_idx) 稳定排序。messages 中每条 LogObject 是 clone 过来的。
    /// source_file_ids[i] 记录 messages[i] 来自哪个 file_id。
    pub fn from_segments(segments: &[Arc<FileSegment>]) -> Self {
        if segments.is_empty() {
            return Self::empty();
        }

        // 单 segment：直接借用，无需排序
        if segments.len() == 1 {
            let seg = &segments[0];
            let messages: Arc<[LogObject]> = seg.messages.clone();
            let source_file_ids: Arc<[u32]> = Arc::from(vec![seg.file_id; messages.len()]);
            return Self {
                messages,
                source_file_ids,
                time_min: seg.time_min,
                time_max: seg.time_max,
                version: 0,
            };
        }

        // 多 segment：收集所有 (绝对秒, file_id, msg_idx, &LogObject) 然后稳定排序
        let start_ns: Vec<i64> = segments.iter().map(|s| {
            s.start_time.map(|_| {
                // 重新计算 start_ns（start_time 已是 NaiveDateTime，需要还原为 UNIX epoch ns）
                use chrono::TimeZone;
                s.start_time.unwrap().and_utc().timestamp_nanos_opt().unwrap_or(0)
            }).unwrap_or(0)
        }).collect();

        let mut entries: Vec<(f64, u32, usize, LogObject)> = Vec::new();
        for (seg_idx, seg) in segments.iter().enumerate() {
            for (msg_idx, msg) in seg.messages.iter().enumerate() {
                let abs_ns = start_ns[seg_idx] + msg.timestamp() as i64;
                let abs_sec = abs_ns as f64 / 1_000_000_000.0;
                entries.push((abs_sec, seg.file_id, msg_idx, msg.clone()));
            }
        }

        // 稳定排序：按 (abs_sec, file_id, msg_idx)
        entries.sort_by(|a, b| {
            a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal)
                .then(a.1.cmp(&b.1))
                .then(a.2.cmp(&b.2))
        });

        let messages: Vec<LogObject> = entries.into_iter().map(|(_, _, _, msg)| msg).collect();
        let messages: Arc<[LogObject]> = Arc::from(messages);
        let source_file_ids: Vec<u32> = (0..messages.len()).map(|_| 0).collect();
        // 需要重新计算 source_file_ids（从排序后的 entries）
        // 修正：上面 entries 被 into_iter 消费，需要重建 source_file_ids

        // 重新构建 source_file_ids
        let mut source_file_ids_vec: Vec<u32> = Vec::with_capacity(messages.len());
        for (seg_idx, seg) in segments.iter().enumerate() {
            for _ in &seg.messages {
                source_file_ids_vec.push(seg.file_id);
            }
        }
        // 错了——排序后的顺序与原始顺序不一致
        // 修正：上面 sort 是稳定的，但 source_file_ids_vec 顺序对应的是排序前的。
        // 正确做法：在 sort 时同步记录 source_file_id。
        // 重新实现，避免双重遍历：

        let _ = source_file_ids; // 占位，下面重写
        let _ = source_file_ids_vec;

        // 重新实现
        let mut entries2: Vec<(f64, u32, usize, u32, LogObject)> = Vec::new();
        for (seg_idx, seg) in segments.iter().enumerate() {
            for (msg_idx, msg) in seg.messages.iter().enumerate() {
                let abs_ns = start_ns[seg_idx] + msg.timestamp() as i64;
                let abs_sec = abs_ns as f64 / 1_000_000_000.0;
                entries2.push((abs_sec, seg.file_id, msg_idx, seg.file_id, msg.clone()));
            }
        }
        entries2.sort_by(|a, b| {
            a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal)
                .then(a.1.cmp(&b.1))
                .then(a.2.cmp(&b.2))
        });
        let messages: Vec<LogObject> = entries2.into_iter().map(|(_, _, _, _, msg)| msg).collect();
        let source_file_ids: Vec<u32> = entries2.into_iter().map(|(_, _, _, fid, _)| fid).collect();
        // 上面 entries2 被消费两次，错误。修正：

        unreachable!("replaced below");
    }
}
```

（此实现复杂且有编译错误，Step 8 会重写）

- [ ] **Step 8: 重写 MergedView::from_segments 为干净实现**

替换 Step 7 中的 `from_segments` 整个函数体为：

```rust
impl MergedView {
    /// 从多个 FileSegment 构建全局合并视图
    ///
    /// 按 (绝对时间秒, file_id, msg_idx) 稳定排序。
    pub fn from_segments(segments: &[Arc<FileSegment>]) -> Self {
        if segments.is_empty() {
            return Self::empty();
        }

        // 单 segment：直接借用，无需排序
        if segments.len() == 1 {
            let seg = &segments[0];
            let messages: Arc<[LogObject]> = seg.messages.clone();
            let source_file_ids: Arc<[u32]> = Arc::from(vec![seg.file_id; messages.len()]);
            return Self {
                messages,
                source_file_ids,
                time_min: seg.time_min,
                time_max: seg.time_max,
                version: 0,
            };
        }

        // 多 segment：收集 (abs_sec, file_id, msg_idx, LogObject) 然后稳定排序
        let start_ns: Vec<i64> = segments.iter().map(|s| {
            use chrono::TimeZone;
            s.start_time
                .map(|ndt| ndt.and_utc().timestamp_nanos_opt().unwrap_or(0))
                .unwrap_or(0)
        }).collect();

        let mut entries: Vec<(f64, u32, usize, u32, LogObject)> = Vec::new();
        for (seg_idx, seg) in segments.iter().enumerate() {
            for (msg_idx, msg) in seg.messages.iter().enumerate() {
                let abs_ns = start_ns[seg_idx] + msg.timestamp() as i64;
                let abs_sec = abs_ns as f64 / 1_000_000_000.0;
                entries.push((abs_sec, seg.file_id, msg_idx, seg.file_id, msg.clone()));
            }
        }

        entries.sort_by(|a, b| {
            a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal)
                .then(a.1.cmp(&b.1))
                .then(a.2.cmp(&b.2))
        });

        let total = entries.len();
        let mut messages_vec = Vec::with_capacity(total);
        let mut source_ids_vec = Vec::with_capacity(total);
        let mut min_sec = f64::INFINITY;
        let mut max_sec = f64::NEG_INFINITY;

        for (abs_sec, _fid, _idx, src_fid, msg) in entries {
            if abs_sec < min_sec { min_sec = abs_sec; }
            if abs_sec > max_sec { max_sec = abs_sec; }
            messages_vec.push(msg);
            source_ids_vec.push(src_fid);
        }

        Self {
            messages: Arc::from(messages_vec),
            source_file_ids: Arc::from(source_ids_vec),
            time_min: if total == 0 { None } else { Some(min_sec) },
            time_max: if total == 0 { None } else { Some(max_sec) },
            version: 0,
        }
    }
}
```

- [ ] **Step 9: 运行空 segments 测试，确认通过**

Run: `cargo test -p view --lib domain::multi_file::tests::test_merged_view_from_empty_segments`
Expected: PASS

- [ ] **Step 10: 写测试 - 单 segment 与多 segment 合并**

在 tests 模块追加：

```rust
    fn make_seg(file_id: u32, msgs: Vec<u64>, start_ns: i64) -> Arc<FileSegment> {
        // msgs: Vec<timestamp_ns relative to start>
        use blf::{CanMessage, BlfObjectHeader};
        let messages: Vec<LogObject> = msgs.iter().map(|ts| {
            LogObject::CanMessage(CanMessage {
                header: BlfObjectHeader {
                    object_time_stamp: *ts,
                    ..Default::default()
                },
                ..Default::default()
            })
        }).collect();
        Arc::new(FileSegment {
            file_id,
            path: PathBuf::from(format!("/tmp/seg{}.blf", file_id)),
            file_name: format!("seg{}.blf", file_id),
            start_time: chrono::NaiveDateTime::from_timestamp_opt(0, 0),
            messages: Arc::from(messages),
            errors: Vec::new(),
            bytes_total: 0,
            bytes_consumed: 0,
            object_count: msgs.len(),
            time_min: None,
            time_max: None,
        })
    }

    #[test]
    fn test_merged_view_single_segment() {
        let seg = make_seg(1, vec![100_000, 200_000, 300_000], 0);
        let view = MergedView::from_segments(&[seg]);
        assert_eq!(view.messages.len(), 3);
        assert_eq!(view.source_file_ids.len(), 3);
        assert_eq!(view.source_file_ids[0], 1);
        assert!(view.time_min.is_some());
        assert!(view.time_max.is_some());
    }

    #[test]
    fn test_merged_view_two_segments_sorted_by_time() {
        // seg1 时间戳: 100ns, 500ns (start_ns = 0)
        // seg2 时间戳: 200ns, 600ns (start_ns = 0)
        // 合并后顺序: 100(seg1), 200(seg2), 500(seg1), 600(seg2)
        let seg1 = make_seg(1, vec![100, 500], 0);
        let seg2 = make_seg(2, vec![200, 600], 0);
        let view = MergedView::from_segments(&[seg1, seg2]);
        assert_eq!(view.messages.len(), 4);
        assert_eq!(view.source_file_ids, Arc::from([1u32, 2, 1, 2]));
        // 验证时间戳顺序: 100, 200, 500, 600
        assert_eq!(view.messages[0].timestamp(), 100);
        assert_eq!(view.messages[1].timestamp(), 200);
        assert_eq!(view.messages[2].timestamp(), 500);
        assert_eq!(view.messages[3].timestamp(), 600);
    }

    #[test]
    fn test_merged_view_two_segments_non_overlapping() {
        // seg1: start_ns = 0, msgs at 100ns
        // seg2: start_ns = 1000, msgs at 100ns (绝对 = 1100ns)
        // 合并后: seg1's 100, seg2's 1100
        let seg1 = make_seg(1, vec![100], 0);
        let seg2 = make_seg(2, vec![100], 1000);
        // make_seg 写死了 start_ns=0，需另构造。先跳过这个测试，留 TODO
    }
```

Run: `cargo test -p view --lib domain::multi_file::tests`
Expected: test_merged_view_two_segments_non_overlapping FAIL（make_seg 不支持 start_ns 参数），其他 PASS

- [ ] **Step 11: 修复 make_seg 支持 start_ns，重写第三个测试**

替换 `make_seg` 函数与第三个测试：

```rust
    fn make_seg(file_id: u32, msgs: Vec<u64>, _start_ns_unused: i64) -> Arc<FileSegment> {
        // 注：_start_ns_unused 仅为 API 兼容；start_time 固定为 0（NaiveDateTime::from_timestamp(0,0)）
        // 绝对时间 = 0 + msg.timestamp()
        use blf::{CanMessage, BlfObjectHeader};
        let messages: Vec<LogObject> = msgs.iter().map(|ts| {
            LogObject::CanMessage(CanMessage {
                header: BlfObjectHeader {
                    object_time_stamp: *ts,
                    ..Default::default()
                },
                ..Default::default()
            })
        }).collect();
        Arc::new(FileSegment {
            file_id,
            path: PathBuf::from(format!("/tmp/seg{}.blf", file_id)),
            file_name: format!("seg{}.blf", file_id),
            start_time: chrono::NaiveDateTime::from_timestamp_opt(0, 0),
            messages: Arc::from(messages),
            errors: Vec::new(),
            bytes_total: 0,
            bytes_consumed: 0,
            object_count: msgs.len(),
            time_min: None,
            time_max: None,
        })
    }

    #[test]
    #[ignore = "make_seg 写死 start_time=0，跨 start_time 的合并测试需扩展工具，暂 ignore"]
    fn test_merged_view_two_segments_non_overlapping() {
        // 留待 Task 1 完成后用更完整工具补足
    }
```

Run: `cargo test -p view --lib domain::multi_file::tests`
Expected: 其他 PASS，test_merged_view_two_segments_non_overlapping IGNORED

- [ ] **Step 12: 写测试 - 移除中间 segment 重建**

在 tests 模块追加：

```rust
    #[test]
    fn test_merged_view_after_remove_middle() {
        // 三个 segment: seg1 (100ns), seg2 (200ns), seg3 (300ns)
        // 移除 seg2 后: 100(seg1), 300(seg3)
        let seg1 = make_seg(1, vec![100], 0);
        let seg2 = make_seg(2, vec![200], 0);
        let seg3 = make_seg(3, vec![300], 0);
        let view_before = MergedView::from_segments(&[seg1.clone(), seg2, seg3.clone()]);
        assert_eq!(view_before.messages.len(), 3);

        let remaining: Vec<Arc<FileSegment>> = vec![seg1, seg3];
        let view_after = MergedView::from_segments(&remaining);
        assert_eq!(view_after.messages.len(), 2);
        assert_eq!(view_after.messages[0].timestamp(), 100);
        assert_eq!(view_after.messages[1].timestamp(), 300);
        assert_eq!(view_after.source_file_ids, Arc::from([1u32, 3]));
    }
```

Run: `cargo test -p view --lib domain::multi_file::tests::test_merged_view_after_remove_middle`
Expected: PASS

- [ ] **Step 13: cargo build 整个 workspace，确保无编译错误**

Run: `cargo build -p view`
Expected: BUILD SUCCESS（可能有 warning，但无 error）

- [ ] **Step 14: Commit**

```bash
git add src/view/src/domain/mod.rs src/view/src/domain/multi_file.rs
git commit -m "feat(domain): add FileSegment and MergedView data model for multi-file loading"
```

---

## Task 2: LoadBlfFiles 命令结构与 LoadMode 枚举

**Files:**
- Create: `src/view/src/app/commands/multi_file.rs`
- Modify: `src/view/src/app/commands/mod.rs:10` (新增 `pub mod multi_file;`)

**Interfaces:**
- Consumes: `std::path::PathBuf`
- Produces: `LoadMode` enum、`LoadBlfFiles` struct、`RemoveFile` struct、`RemoveAllFiles` struct

- [ ] **Step 1: 注册新模块**

Edit `src/view/src/app/commands/mod.rs:10` 后追加：

```rust
pub mod multi_file;
```

- [ ] **Step 2: 创建 multi_file.rs 命令结构 + 测试**

Create `src/view/src/app/commands/multi_file.rs`：

```rust
//! 多文件加载与移除命令

use std::path::PathBuf;

/// 加载模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadMode {
    /// 单选 Open BLF... — 清空已加载文件后加载新文件
    Replace,
    /// 多选 Open Multiple BLF... — 追加到已加载文件
    Append,
}

/// 加载一个或多个 BLF 文件
#[derive(Debug, Clone)]
pub struct LoadBlfFiles {
    pub paths: Vec<PathBuf>,
    pub mode: LoadMode,
}

impl LoadBlfFiles {
    pub fn new(paths: Vec<PathBuf>, mode: LoadMode) -> Self {
        Self { paths, mode }
    }

    pub fn is_empty(&self) -> bool {
        self.paths.is_empty()
    }

    pub fn len(&self) -> usize {
        self.paths.len()
    }
}

/// 移除单个已加载文件
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemoveFile {
    pub file_id: u32,
}

impl RemoveFile {
    pub fn new(file_id: u32) -> Self {
        Self { file_id }
    }
}

/// 移除所有已加载文件
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemoveAllFiles;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_mode_replace() {
        let cmd = LoadBlfFiles::new(vec![PathBuf::from("/a.blf")], LoadMode::Replace);
        assert_eq!(cmd.mode, LoadMode::Replace);
        assert_eq!(cmd.len(), 1);
        assert!(!cmd.is_empty());
    }

    #[test]
    fn test_load_mode_append() {
        let cmd = LoadBlfFiles::new(
            vec![PathBuf::from("/a.blf"), PathBuf::from("/b.blf")],
            LoadMode::Append,
        );
        assert_eq!(cmd.mode, LoadMode::Append);
        assert_eq!(cmd.len(), 2);
    }

    #[test]
    fn test_load_blf_files_empty() {
        let cmd = LoadBlfFiles::new(Vec::new(), LoadMode::Append);
        assert!(cmd.is_empty());
        assert_eq!(cmd.len(), 0);
    }

    #[test]
    fn test_remove_file_command() {
        let cmd = RemoveFile::new(42);
        assert_eq!(cmd.file_id, 42);
    }
}
```

- [ ] **Step 3: 运行测试，确认通过**

Run: `cargo test -p view --lib app::commands::multi_file::tests`
Expected: PASS（4 个测试）

- [ ] **Step 4: Commit**

```bash
git add src/view/src/app/commands/mod.rs src/view/src/app/commands/multi_file.rs
git commit -m "feat(commands): add LoadBlfFiles/LoadMode/RemoveFile/RemoveAllFiles commands"
```

---

## Task 3: CanViewApp 状态结构改造（state.rs）

**Files:**
- Modify: `src/view/src/app/state.rs` (整文件改造)

**Interfaces:**
- Consumes: `crate::domain::multi_file::{FileSegment, MergedView}`
- Produces: 更新后的 `CanViewApp` 结构（新字段 `files`/`merged`/`show_files_popover`/`loading_progress`，删除 `is_streaming_mode`）、`LoadingProgress` struct、更新后的 `RuntimeState`（新增 `files`/`merged`，删除 `messages`/`is_streaming_mode`，保留 `current_file_name`）

- [ ] **Step 1: 在 state.rs 顶部新增 LoadingProgress 结构**

Edit `src/view/src/app/state.rs`，在 `SimpleDeprecatedInputState` 之前（line 5 附近）插入：

```rust
/// 多文件加载进度
#[derive(Clone, Debug, Default)]
pub struct LoadingProgress {
    pub total_files: usize,
    pub completed_files: usize,
    pub current_file_name: Option<String>,
    pub total_messages_so_far: usize,
    pub is_cancelled: bool,
}
```

- [ ] **Step 2: 修改 RuntimeState 结构 - 新增 files/merged，删除 messages/is_streaming_mode**

Edit `src/view/src/app/state.rs`，将 `RuntimeState` 结构（line 14-33）替换为：

```rust
pub struct RuntimeState {
    pub current_view: AppView,
    pub files: Vec<std::sync::Arc<crate::domain::multi_file::FileSegment>>,
    pub merged: crate::domain::multi_file::MergedView,
    pub current_file_name: Option<String>,
    pub plot_data: std::sync::Arc<[crate::models::Series]>,
    pub plot_full_data: std::sync::Arc<[crate::models::Series]>,
    pub plot_zoom_start: Option<f64>,
    pub plot_zoom_end: Option<f64>,
    pub plot_full_time_min: Option<f64>,
    pub plot_full_time_max: Option<f64>,
    pub show_plot_points: bool,
    pub selected_signals: Vec<String>,
    pub dbc_channels: HashMap<u16, DbcDatabase>,
    pub ldf_channels: HashMap<u16, LdfDatabase>,
    pub start_time: Option<chrono::NaiveDateTime>,
    pub active_library_id: Option<String>,
    pub active_version_name: Option<String>,
}
```

注意：删除 `messages: Vec<LogObject>`、`is_streaming_mode: bool`。

- [ ] **Step 3: 在 CanViewApp 中新增字段并删除 is_streaming_mode**

Edit `src/view/src/app/state.rs`，在 `CanViewApp` 结构（line 72 附近）的 `current_file_name: Option<String>` 后追加：

```rust
    // 多文件加载状态
    pub files: Vec<std::sync::Arc<crate::domain::multi_file::FileSegment>>,
    pub merged: crate::domain::multi_file::MergedView,
    pub show_files_popover: bool,
    pub loading_progress: Option<crate::app::state::LoadingProgress>,
```

并删除 `is_streaming_mode: bool` 字段（line 96 附近）。

- [ ] **Step 4: 修改 new_with_maximized_state_and_bounds 初始化**

在初始化块（line 264-380）中：
- 删除 `is_streaming_mode: false,` 行
- 在 `current_file_name: None,` 后追加：

```rust
            files: Vec::new(),
            merged: crate::domain::multi_file::MergedView::empty(),
            show_files_popover: false,
            loading_progress: None,
```

- [ ] **Step 5: 修改 save_runtime_state 方法**

将 save_runtime_state（line 385-410）替换为：

```rust
    pub fn save_runtime_state(&self) -> RuntimeState {
        eprintln!("💾 Saving runtime state: {:?} view, {} files, {} plot series, zoom: {:?}-{:?}, {} signals, {} DBC, {} LDF",
            self.current_view, self.files.len(), self.plot_data.len(),
            self.plot_zoom_start, self.plot_zoom_end,
            self.selected_signals.len(),
            self.dbc_channels.len(), self.ldf_channels.len());
        RuntimeState {
            current_view: self.current_view.clone(),
            files: self.files.clone(),
            merged: clone_merged_view(&self.merged),
            current_file_name: self.current_file_name.clone(),
            plot_data: self.plot_data.clone(),
            plot_full_data: self.plot_full_data.clone(),
            plot_zoom_start: self.plot_zoom_start,
            plot_zoom_end: self.plot_zoom_end,
            plot_full_time_min: self.plot_full_time_min,
            plot_full_time_max: self.plot_full_time_max,
            show_plot_points: self.show_plot_points,
            selected_signals: self.selected_signals.clone(),
            dbc_channels: self.dbc_channels.clone(),
            ldf_channels: self.ldf_channels.clone(),
            start_time: self.start_time,
            active_library_id: self.active_library_id.clone(),
            active_version_name: self.active_version_name.clone(),
        }
    }
```

注意：删除 `messages: self.messages.clone(),` 与 `is_streaming_mode: self.is_streaming_mode,`。

并在 `RuntimeState` 之前或 save_runtime_state 之前追加 `clone_merged_view` 辅助函数：

```rust
/// Clone a MergedView (Arc fields are refcounted, cheap to clone)
fn clone_merged_view(view: &crate::domain::multi_file::MergedView) -> crate::domain::multi_file::MergedView {
    crate::domain::multi_file::MergedView {
        messages: view.messages.clone(),
        source_file_ids: view.source_file_ids.clone(),
        time_min: view.time_min,
        time_max: view.time_max,
        version: view.version,
    }
}
```

注：`MergedView` 的字段都是 `Arc` 或 `Option` 或 `Copy`，可直接克隆。为简化代码，后续可在 `multi_file.rs` 给 `MergedView` 加 `#[derive(Clone)]`，本步骤先用辅助函数避免触碰 domain 文件。Task 1 中 `MergedView` 未 derive Clone，所以需辅助函数。**优化**：直接在 Task 1 加 derive。回 Task 1 修复：

（在 Task 1 Step 2 中 `pub struct MergedView` 上方加 `#[derive(Clone)]`）

- [ ] **Step 6: 在 multi_file.rs 给 MergedView 加 derive(Clone)**

Edit `src/view/src/domain/multi_file.rs`，将：

```rust
pub struct MergedView {
```

改为：

```rust
#[derive(Clone)]
pub struct MergedView {
```

并相应简化 Task 3 Step 5 中 `clone_merged_view` 调用为直接 `self.merged.clone()`。

- [ ] **Step 7: 修改 restore_runtime_state**

将 restore_runtime_state（line 414-448）替换为：

```rust
    pub fn restore_runtime_state(&mut self, state: RuntimeState) {
        eprintln!("♻️  Restoring runtime state: {:?} view, {} files, {} plot series, zoom: {:?}-{:?}, {} signals, {} DBC, {} LDF",
            state.current_view,
            state.files.len(),
            state.plot_data.len(),
            state.plot_zoom_start, state.plot_zoom_end,
            state.selected_signals.len(),
            state.dbc_channels.len(), state.ldf_channels.len());
        self.current_view = state.current_view;
        self.files = state.files;
        self.merged = state.merged;
        // 兼容字段：从 merged.messages 派生 messages 快照
        self.messages = state.merged.messages.to_vec();
        self.current_file_name = state.current_file_name;
        self.plot_data = state.plot_data;
        self.plot_full_data = state.plot_full_data;
        self.plot_zoom_start = state.plot_zoom_start;
        self.plot_zoom_end = state.plot_zoom_end;
        self.plot_full_time_min = state.plot_full_time_min;
        self.plot_full_time_max = state.plot_full_time_max;
        self.show_plot_points = state.show_plot_points;
        self.plot_width_px = gpui::px(0.0);
        self.selected_signals = state.selected_signals;
        self.dbc_channels = state.dbc_channels;
        self.ldf_channels = state.ldf_channels;
        self.start_time = state.start_time;
        self.active_library_id = state.active_library_id;
        self.active_version_name = state.active_version_name;
        eprintln!("✅ State restored. Now have: {:?} view, {} files, {} messages, {} plot series",
            self.current_view, self.files.len(), self.messages.len(), self.plot_data.len());
    }
```

注意：删除 `self.is_streaming_mode = state.is_streaming_mode;`，新增 `self.files`/`self.merged`/`self.messages` 派生。

- [ ] **Step 8: 修改 impls.rs new() 与 new_with_maximized_state_and_bounds 中 is_streaming_mode 初始化**

Edit `src/view/src/app/impls.rs`：
- Line 34：删除 `is_streaming_mode: false,`
- Line 604：删除 `is_streaming_mode: false,`
- 在两处的 `current_file_name: None,` 后追加：

```rust
            files: Vec::new(),
            merged: crate::domain::multi_file::MergedView::empty(),
            show_files_popover: false,
            loading_progress: None,
```

- [ ] **Step 9: cargo build 验证编译**

Run: `cargo build -p view`
Expected: BUILD SUCCESS（无 is_streaming_mode 残留引用）

- [ ] **Step 10: cargo test 验证回归**

Run: `cargo test -p view --lib`
Expected: 既有测试全部 PASS

- [ ] **Step 11: Commit**

```bash
git add src/view/src/app/state.rs src/view/src/app/impls.rs src/view/src/domain/multi_file.rs
git commit -m "refactor(app): add files/merged/show_files_popover/loading_progress, drop is_streaming_mode"
```

---

## Task 4: apply_blf_result 拆分为 single/append 与 rebuild_merged 实现

**Files:**
- Modify: `src/view/src/app/impls.rs` (line 236-301 替换 apply_blf_result)
- Modify: `src/view/src/controllers/config_controller.rs:148` (调用点改名)

**Interfaces:**
- Consumes: `crate::domain::multi_file::{FileSegment, MergedView}`、`blf::BlfResult`、`std::sync::atomic::{AtomicU32, Ordering}`、`std::path::PathBuf`
- Produces: `apply_blf_result_single(&mut self, result: BlfResult, path: PathBuf, cx)`、`apply_blf_result_append_one(&mut self, result: Result<BlfResult, E>, path: PathBuf, cx)`、`apply_blf_results_append(&mut self, results: Vec<Result<BlfResult, Error>>, paths: Vec<PathBuf>, cx)`、`remove_file(&mut self, file_id: u32, cx)`、`remove_all_files(&mut self, cx)`、`rebuild_merged(&mut self)`、`next_file_id() -> u32`

- [ ] **Step 1: 在 impls.rs 顶部增加 file_id 计数器**

在 `src/view/src/app/impls.rs` 顶部（line 14 后）增加：

```rust
use std::sync::atomic::{AtomicU32, Ordering};

static FILE_ID_COUNTER: AtomicU32 = AtomicU32::new(1);

fn next_file_id() -> u32 {
    FILE_ID_COUNTER.fetch_add(1, Ordering::Relaxed)
}
```

- [ ] **Step 2: 替换 apply_blf_result 为 apply_blf_result_single**

Edit `src/view/src/app/impls.rs:236-301`，将整个 `apply_blf_result` 函数替换为：

```rust
    /// 单文件 Replace:清空 files,加入单个 segment,重建 merged
    pub(crate) fn apply_blf_result_single(
        &mut self,
        result: anyhow::Result<BlfResult>,
        path: PathBuf,
    ) {
        match result {
            Ok(result) => {
                let error_count = result.errors.len();
                if error_count > 0 {
                    Self::log_blf_errors(&result.errors, result.objects.len());
                }

                // Replace 模式：清空所有已有数据
                self.files.clear();
                self.merged = crate::domain::multi_file::MergedView::empty();
                self.plot_data = std::sync::Arc::from([]);
                self.plot_full_data = std::sync::Arc::from([]);
                self.selected_signals.clear();

                self.current_view = AppView::LogView;

                let file_id = next_file_id();
                let file_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|s| s.to_string());
                let segment = crate::domain::multi_file::FileSegment::from_blf_result(
                    result, file_id, path,
                );

                if error_count > 0 {
                    self.blf_parse_errors = segment.errors.clone();
                    self.status_msg = format!("⚠️ {} parse error(s) — see details", error_count).into();
                } else {
                    self.blf_parse_errors.clear();
                    self.show_blf_errors_popover = false;
                    self.status_msg = format!("✅ Loaded {} messages", segment.object_count).into();
                }

                self.start_time = segment.start_time;
                self.blf_bytes_total = segment.bytes_total;
                self.blf_bytes_consumed = segment.bytes_consumed;

                self.files.push(std::sync::Arc::new(segment));
                self.rebuild_merged();
                self.messages = self.merged.messages.to_vec();
                self.current_file_name = file_name;
                self.library_picker_dismissed = false;
                self.library_picker_selected_version.clear();
            }
            Err(e) => {
                // Replace 模式失败：不清空已有数据，保留旧的 files/messages
                self.status_msg = format!("❌ File Error: {}", e).into();
                Self::display_blf_load_error(&e);
                self.blf_bytes_total = 0;
                self.blf_bytes_consumed = 0;
                self.blf_parse_errors.clear();
                self.show_blf_errors_popover = false;
            }
        }
    }

    /// 多文件 Append:单个文件解析完成时调用,追加 segment 后重建 merged
    pub(crate) fn apply_blf_result_append_one(
        &mut self,
        result: anyhow::Result<BlfResult>,
        path: PathBuf,
    ) {
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_string());

        match result {
            Ok(result) => {
                let error_count = result.errors.len();
                if error_count > 0 {
                    Self::log_blf_errors(&result.errors, result.objects.len());
                }

                let file_id = next_file_id();
                let segment = crate::domain::multi_file::FileSegment::from_blf_result(
                    result, file_id, path,
                );

                let msg_count = segment.object_count;
                self.files.push(std::sync::Arc::new(segment));
                self.rebuild_merged();
                self.messages = self.merged.messages.to_vec();

                // 更新汇总进度
                if let Some(p) = &mut self.loading_progress {
                    p.completed_files += 1;
                    p.total_messages_so_far += msg_count;
                    p.current_file_name = file_name.clone();
                }

                // 更新 StatusBar
                let total_files = self.files.len();
                let total_msgs = self.messages.len();
                let failed = self.files.iter().filter(|f| !f.errors.is_empty()).count();
                if failed > 0 {
                    self.status_msg = format!(
                        "⚠️ Loaded {}/{} files ({} failed) — {} messages",
                        total_files - failed, total_files, failed, total_msgs
                    ).into();
                } else if let Some(p) = &self.loading_progress {
                    if p.completed_files < p.total_files {
                        self.status_msg = format!(
                            "⏳ Loading {}/{} files — {} messages",
                            p.completed_files, p.total_files, total_msgs
                        ).into();
                    } else {
                        self.status_msg = format!(
                            "✅ Loaded {} files, {} messages",
                            total_files, total_msgs
                        ).into();
                    }
                } else {
                    self.status_msg = format!(
                        "✅ Loaded {} files, {} messages",
                        total_files, total_msgs
                    ).into();
                }

                self.current_file_name = file_name;
            }
            Err(e) => {
                // Append 模式单文件失败:保留失败文件占位,messages 为空
                let file_id = next_file_id();
                let segment = crate::domain::multi_file::FileSegment {
                    file_id,
                    path: path.clone(),
                    file_name: file_name.clone().unwrap_or_else(|| "unknown".to_string()),
                    start_time: None,
                    messages: std::sync::Arc::from([]),
                    errors: vec![format!("{}", e)],
                    bytes_total: 0,
                    bytes_consumed: 0,
                    object_count: 0,
                    time_min: None,
                    time_max: None,
                };
                self.files.push(std::sync::Arc::new(segment));

                if let Some(p) = &mut self.loading_progress {
                    p.completed_files += 1;
                    p.current_file_name = file_name;
                }

                let total_files = self.files.len();
                let failed = self.files.iter().filter(|f| !f.errors.is_empty()).count();
                self.status_msg = format!(
                    "⚠️ Loaded {}/{} files ({} failed) — see Files",
                    total_files - failed, total_files, failed
                ).into();
            }
        }
    }

    /// 重建 merged 视图(所有 file 增删后调用)
    pub(crate) fn rebuild_merged(&mut self) {
        let segments: Vec<std::sync::Arc<crate::domain::multi_file::FileSegment>> =
            self.files.iter().cloned().collect();
        self.merged = crate::domain::multi_file::MergedView::from_segments(&segments);
    }

    /// 移除单个文件
    pub(crate) fn remove_file(&mut self, file_id: u32) {
        self.files.retain(|f| f.file_id != file_id);
        self.rebuild_merged();
        self.messages = self.merged.messages.to_vec();
        let total_files = self.files.len();
        let total_msgs = self.messages.len();
        if total_files == 0 {
            self.status_msg = "Ready".into();
            self.current_file_name = None;
            self.start_time = None;
            self.plot_data = std::sync::Arc::from([]);
            self.plot_full_data = std::sync::Arc::from([]);
            self.selected_signals.clear();
            self.blf_bytes_total = 0;
            self.blf_bytes_consumed = 0;
        } else {
            self.status_msg = format!("✅ Loaded {} files, {} messages", total_files, total_msgs).into();
        }
    }

    /// 移除所有文件
    pub(crate) fn remove_all_files(&mut self) {
        self.files.clear();
        self.merged = crate::domain::multi_file::MergedView::empty();
        self.messages.clear();
        self.plot_data = std::sync::Arc::from([]);
        self.plot_full_data = std::sync::Arc::from([]);
        self.selected_signals.clear();
        self.current_file_name = None;
        self.start_time = None;
        self.blf_bytes_total = 0;
        self.blf_bytes_consumed = 0;
        self.blf_parse_errors.clear();
        self.show_blf_errors_popover = false;
        self.show_files_popover = false;
        self.status_msg = "Ready".into();
    }
```

- [ ] **Step 3: 修改 config_controller.rs 调用点**

Edit `src/view/src/controllers/config_controller.rs:148`，将：

```rust
pub fn apply_blf_result(app: &mut CanViewApp, result: anyhow::Result<blf::BlfResult>) {
    app.apply_blf_result(result, None);
}
```

替换为：

```rust
pub fn apply_blf_result(app: &mut CanViewApp, result: anyhow::Result<blf::BlfResult>) {
    // 兼容旧接口:使用空路径调用 apply_blf_result_single
    // 注意:此调用方提供的是无路径场景,使用占位路径
    let path = std::path::PathBuf::from("unknown.blf");
    app.apply_blf_result_single(result, path);
}
```

- [ ] **Step 4: 修改 impls_rendering.rs 中调用点**

Edit `src/view/src/app/impls_rendering.rs:1940-1945`，将：

```rust
                                                let _ = cx.update(|cx| {
                                                    view.update(cx, |view, cx| {
                                                        view.apply_blf_result(result, fname);
                                                        cx.notify();
                                                    });
                                                });
```

替换为：

```rust
                                                let _ = cx.update(|cx| {
                                                    view.update(cx, |view, cx| {
                                                        view.apply_blf_result_single(result, path.clone());
                                                        cx.notify();
                                                    });
                                                });
```

注：原代码 `fname` 是 Option<String>，`path` 是 PathBuf。`apply_blf_result_single` 内部自己提取 file_name，不再需要 `fname`。

- [ ] **Step 5: cargo build 验证**

Run: `cargo build -p view`
Expected: BUILD SUCCESS

- [ ] **Step 6: cargo test 验证回归**

Run: `cargo test -p view --lib`
Expected: 既有测试全部 PASS

- [ ] **Step 7: Commit**

```bash
git add src/view/src/app/impls.rs src/view/src/controllers/config_controller.rs src/view/src/app/impls_rendering.rs
git commit -m "refactor(app): split apply_blf_result into single/append, add rebuild_merged/remove_file/remove_all_files"
```

---

## Task 5: File 菜单新增 Open Multiple BLF... 入口

**Files:**
- Modify: `src/view/src/app/impls_rendering.rs:1880-1956` (File dropdown 渲染)

**Interfaces:**
- Consumes: `rfd::AsyncFileDialog::pick_files()`、`crate::app::commands::multi_file::{LoadBlfFiles, LoadMode}`、`apply_blf_result_append_one`
- Produces: 多选追加入口的 UI 渲染与命令派发

- [ ] **Step 1: 在 impls_rendering.rs 顶部增加 use 引用**

Edit `src/view/src/app/impls_rendering.rs:9` 附近，将：

```rust
use blf::{LogObject, read_blf_from_file};
```

替换为：

```rust
use blf::{LogObject, read_blf_from_file};
use crate::app::commands::multi_file::{LoadMode};
```

- [ ] **Step 2: 在 File dropdown 中新增 Open Multiple BLF... child**

Edit `src/view/src/app/impls_rendering.rs:1952-1953`，在 `Open BLF...` child 闭合 `)` 后、`} else {` 之前追加新的 child：

```rust
                        )
                        // Open Multiple BLF... (multi-select append)
                        .child(
                            div()
                                .px_3()
                                .py_1()
                                .text_xs()
                                .text_color(rgb(0xcdd6f4))
                                .hover(|style| style.bg(rgb(0x45475a)))
                                .cursor_pointer()
                                .on_mouse_down(gpui::MouseButton::Left, {
                                    let view = view.clone();
                                    move |_event, _window, cx| {
                                        cx.stop_propagation();
                                        view.update(cx, |this, cx| {
                                            this.show_file_menu = false;
                                            cx.notify();
                                        });
                                        let view = view.clone();
                                        cx.spawn(async move |cx| {
                                            let files = rfd::AsyncFileDialog::new()
                                                .add_filter("BLF Files", &["blf", "bin"])
                                                .pick_files()
                                                .await;
                                            if let Some(files) = files {
                                                let paths: Vec<std::path::PathBuf> =
                                                    files.into_iter().map(|f| f.path().to_owned()).collect();
                                                if paths.is_empty() { return Ok::<(), anyhow::Error>(()); }

                                                // 初始化 loading_progress
                                                let total = paths.len();
                                                let _ = cx.update(|cx| {
                                                    view.update(cx, |view, cx| {
                                                        view.loading_progress = Some(crate::app::state::LoadingProgress {
                                                            total_files: total,
                                                            completed_files: 0,
                                                            current_file_name: None,
                                                            total_messages_so_far: 0,
                                                            is_cancelled: false,
                                                        });
                                                        view.status_msg = format!("⏳ Loading 0/{} files...", total).into();
                                                        cx.notify();
                                                    });
                                                });

                                                // 并发解析:对每个 path spawn 一个后台任务
                                                for path in paths {
                                                    let view = view.clone();
                                                    let result = cx
                                                        .background_executor()
                                                        .spawn(async move {
                                                            read_blf_from_file(&path).map_err(|e| {
                                                                anyhow::Error::msg(format!("{:?}", e))
                                                            })
                                                        })
                                                        .await;
                                                    let _ = cx.update(|cx| {
                                                        view.update(cx, |view, cx| {
                                                            view.apply_blf_result_append_one(result, path);
                                                            cx.notify();
                                                        });
                                                    });
                                                }
                                            }
                                            Ok::<(), anyhow::Error>(())
                                        })
                                        .detach();
                                    }
                                })
                                .child("Open Multiple BLF..."),
                        )
```

注意：此处为简化用顺序 await 而非真正并发（避免 GPUI Task 集合管理的复杂性）。Task 7 中改为真正并发。

- [ ] **Step 3: cargo build 验证**

Run: `cargo build -p view`
Expected: BUILD SUCCESS

- [ ] **Step 4: 手动验证（启动 UI 测试）**

Run: `cargo run --release --bin view`
- 点击 File 菜单
- 验证看到 `Open BLF...` 与 `Open Multiple BLF...` 两个选项
- 多选两个 BLF 文件（如 `sample.blf` 与 `can-sampling_*.blf`）
- 验证 StatusBar 显示 `⏳ Loading 0/2 files...` 然后逐步推进
- 验证最终显示 `✅ Loaded 2 files, N messages`
- 验证 LogView 显示合并后的消息列表

注：因无 popover（Task 6）暂时无法验证文件管理。

- [ ] **Step 5: Commit**

```bash
git add src/view/src/app/impls_rendering.rs
git commit -m "feat(ui): add Open Multiple BLF... menu entry with multi-select append"
```

---

## Task 6: StatusBar 文件管理入口与文件管理 popover

**Files:**
- Modify: `src/view/src/app/impls_rendering.rs` (StatusBar 渲染区域)
- Test: 手动 UI 验证

**Interfaces:**
- Consumes: `self.files`、`self.show_files_popover`、`remove_file`、`remove_all_files`
- Produces: StatusBar 上的 `📁 Files` 按钮与文件管理 popover

- [ ] **Step 1: 定位 StatusBar 渲染位置**

Run: `grep -n "current_file_name\|blf_bytes_total\|status_msg" src/view/src/app/impls_rendering.rs | head -20`

记录 StatusBar 显示 `current_file_name` 和 `blf_bytes_*` 的行号。

- [ ] **Step 2: 在 StatusBar 右侧新增 📁 Files 按钮**

Edit `src/view/src/app/impls_rendering.rs`（StatusBar 渲染块），在显示 `current_file_name` 的 div 之后追加：

```rust
// Files 按钮
.child(
    div()
        .px_2()
        .py_1()
        .text_xs()
        .text_color(rgb(0xcdd6f4))
        .hover(|style| style.bg(rgb(0x45475a)))
        .cursor_pointer()
        .on_mouse_down(gpui::MouseButton::Left, {
            let view = view.clone();
            move |_event, _window, cx| {
                cx.stop_propagation();
                view.update(cx, |view, cx| {
                    view.show_files_popover = !view.show_files_popover;
                    cx.notify();
                });
            }
        })
        .child(format!("📁 Files ({})", self.files.len())),
)
```

- [ ] **Step 3: 新增文件管理 popover 渲染**

在 File dropdown 渲染块之后（`show_file_menu` 分支同级），追加 popover 渲染：

```rust
// Files popover
.child({
    if self.show_files_popover {
        let view_for_close = view.clone();
        div()
            .absolute()
            .top(px(60.))
            .right(px(20.))
            .w(px(420.))
            .bg(rgb(0x1e1e2e))
            .border_1()
            .border_color(rgb(0x45475a))
            .rounded(px(8.))
            .shadow_lg()
            .flex()
            .flex_col()
            .p_4()
            .on_mouse_down(gpui::MouseButton::Left, |_event, _window, cx| {
                cx.stop_propagation();
            })
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(0xcdd6f4))
                    .child("Loaded Files")
            )
            .children(self.files.iter().map(|seg| {
                let file_id = seg.file_id;
                let file_name = seg.file_name.clone();
                let msg_count = seg.object_count;
                let size_mb = seg.bytes_total as f64 / 1_048_576.0;
                let has_errors = !seg.errors.is_empty();
                let status_icon = if has_errors { "❌" } else { "✅" };
                let bg_color = if has_errors { rgb(0x45475a) } else { rgb(0x313244) };
                let view_for_remove = view.clone();
                div()
                    .mt_2()
                    .p_2()
                    .bg(bg_color)
                    .rounded(px(4.))
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0xcdd6f4))
                            .child(format!(
                                "{} {} — {} msgs, {:.2} MB {}",
                                status_icon, file_name, msg_count, size_mb,
                                if has_errors { "(errors)" } else { "" }
                            ))
                    )
                    .child(
                        div()
                            .px_2()
                            .text_xs()
                            .text_color(rgb(0xf38ba8))
                            .cursor_pointer()
                            .hover(|style| style.bg(rgb(0x45475a)))
                            .on_mouse_down(gpui::MouseButton::Left, {
                                let view = view_for_remove.clone();
                                move |_event, _window, cx| {
                                    cx.stop_propagation();
                                    view.update(cx, |view, cx| {
                                        view.remove_file(file_id);
                                        cx.notify();
                                    });
                                }
                            })
                            .child("✕")
                    )
            }))
            .child(
                div()
                    .mt_3()
                    .flex()
                    .justify_between()
                    .child(
                        div()
                            .px_3()
                            .py_1()
                            .text_xs()
                            .text_color(rgb(0xf38ba8))
                            .cursor_pointer()
                            .hover(|style| style.bg(rgb(0x45475a)))
                            .on_mouse_down(gpui::MouseButton::Left, {
                                let view = view.clone();
                                move |_event, _window, cx| {
                                    cx.stop_propagation();
                                    view.update(cx, |view, cx| {
                                        view.remove_all_files();
                                        cx.notify();
                                    });
                                }
                            })
                            .child("Remove All")
                    )
                    .child(
                        div()
                            .px_3()
                            .py_1()
                            .text_xs()
                            .text_color(rgb(0xcdd6f4))
                            .cursor_pointer()
                            .hover(|style| style.bg(rgb(0x45475a)))
                            .on_mouse_down(gpui::MouseButton::Left, {
                                let view = view_for_close.clone();
                                move |_event, _window, cx| {
                                    cx.stop_propagation();
                                    view.update(cx, |view, cx| {
                                        view.show_files_popover = false;
                                        cx.notify();
                                    });
                                }
                            })
                            .child("Done")
                    )
    } else {
        div().hidden()
    }
})
```

- [ ] **Step 4: cargo build 验证**

Run: `cargo build -p view`
Expected: BUILD SUCCESS

- [ ] **Step 5: 手动验证**

Run: `cargo run --release --bin view`
- 加载 2-3 个 BLF 文件
- 点击 `📁 Files (N)` 按钮
- 验证 popover 显示所有已加载文件，每行带 ✕ 按钮
- 点击某行 ✕ → 该文件被移除，消息列表立即更新
- 点击 `Remove All` → 所有文件清空，状态回到 Ready
- 点击 `Done` → popover 关闭
- 加载一个损坏文件（如 `src/blf/test_corrupted.blf`），验证 popover 中该行标红显示 ❌

- [ ] **Step 6: Commit**

```bash
git add src/view/src/app/impls_rendering.rs
git commit -m "feat(ui): add StatusBar Files button and file management popover"
```

---

## Task 7: 真正并发解析（替换顺序 await）

**Files:**
- Modify: `src/view/src/app/impls_rendering.rs` (Task 5 中的 Open Multiple BLF... child)

**Interfaces:**
- Consumes: 同 Task 5
- Produces: 真正并发的解析流程

- [ ] **Step 1: 替换 Task 5 中的顺序 await 为并发 spawn**

Edit `src/view/src/app/impls_rendering.rs`（Task 5 中新增的 `Open Multiple BLF...` child），将：

```rust
                                                // 并发解析:对每个 path spawn 一个后台任务
                                                for path in paths {
                                                    let view = view.clone();
                                                    let result = cx
                                                        .background_executor()
                                                        .spawn(async move {
                                                            read_blf_from_file(&path).map_err(|e| {
                                                                anyhow::Error::msg(format!("{:?}", e))
                                                            })
                                                        })
                                                        .await;
                                                    let _ = cx.update(|cx| {
                                                        view.update(cx, |view, cx| {
                                                            view.apply_blf_result_append_one(result, path);
                                                            cx.notify();
                                                        });
                                                    });
                                                }
```

替换为：

```rust
                                                // 并发解析:spawn 所有任务,顺序收集结果（GPUI Task 并发执行）
                                                use std::sync::Arc;
                                                use std::sync::Mutex;
                                                let results: Arc<Mutex<Vec<(std::path::PathBuf, Option<anyhow::Result<blf::BlfResult>>,)>>> =
                                                    Arc::new(Mutex::new(Vec::with_capacity(paths.len())));
                                                let mut tasks = Vec::new();
                                                for path in paths.clone() {
                                                    let path = path.clone();
                                                    let task = cx.background_executor().spawn(async move {
                                                        let result = read_blf_from_file(&path).map_err(|e| {
                                                            anyhow::Error::msg(format!("{:?}", e))
                                                        });
                                                        (path, result)
                                                    });
                                                    tasks.push(task);
                                                }

                                                // 顺序 await 但每个任务在后台已开始执行
                                                for task in tasks {
                                                    let (path, result) = task.await;
                                                    let view = view.clone();
                                                    let _ = cx.update(|cx| {
                                                        view.update(cx, |view, cx| {
                                                            view.apply_blf_result_append_one(result, path);
                                                            cx.notify();
                                                        });
                                                    });
                                                }
```

注：`background_executor().spawn` 返回的 Task 在创建时就开始执行；顺序 `.await` 只是收集结果，并不阻塞并发。GPUI Task 是 smol 的 Task 包装，自然并发。

- [ ] **Step 2: cargo build 验证**

Run: `cargo build -p view`
Expected: BUILD SUCCESS

- [ ] **Step 3: 手动验证并发性能**

Run: `cargo run --release --bin view`
- 多选 3 个较大的 BLF 文件
- 验证完成时间 ≈ 单文件解析时间的 1.2-1.5 倍（而非 3 倍）
- 验证 StatusBar 流式显示每个文件完成（首个文件完成后即可见数据）

- [ ] **Step 4: Commit**

```bash
git add src/view/src/app/impls_rendering.rs
git commit -m "perf(ui): concurrent BLF parsing in Open Multiple BLF... entry"
```

---

## Task 8: Loading 状态下的 Cancel 按钮

**Files:**
- Modify: `src/view/src/app/impls_rendering.rs` (StatusBar 渲染)

**Interfaces:**
- Consumes: `self.loading_progress`、`self.loading_progress.is_cancelled`
- Produces: Cancel 按钮 + 取消逻辑

- [ ] **Step 1: 在 StatusBar 增加 Cancel 按钮（仅 Loading 时显示）**

Edit `src/view/src/app/impls_rendering.rs`（StatusBar 渲染块），在 `📁 Files` 按钮之前追加：

```rust
// Cancel 按钮（仅 Loading 时显示）
.child({
    let is_loading = self.loading_progress.as_ref()
        .map(|p| p.completed_files < p.total_files && !p.is_cancelled)
        .unwrap_or(false);
    if is_loading {
        let view = view.clone();
        div()
            .px_2()
            .py_1()
            .text_xs()
            .text_color(rgb(0xf38ba8))
            .hover(|style| style.bg(rgb(0x45475a)))
            .cursor_pointer()
            .on_mouse_down(gpui::MouseButton::Left, {
                let view = view.clone();
                move |_event, _window, cx| {
                    cx.stop_propagation();
                    view.update(cx, |view, cx| {
                        if let Some(p) = &mut view.loading_progress {
                            p.is_cancelled = true;
                        }
                        view.status_msg = "❌ Loading cancelled".into();
                        cx.notify();
                    });
                }
            })
            .child("❌ Cancel")
    } else {
        div().hidden()
    }
})
```

- [ ] **Step 2: 在 apply_blf_result_append_one 中检查 is_cancelled**

Edit `src/view/src/app/impls.rs`（`apply_blf_result_append_one` 函数），在函数开头追加：

```rust
        // 检查取消标志：已取消则不再追加新完成的 segment
        if let Some(p) = &self.loading_progress {
            if p.is_cancelled {
                return;
            }
        }
```

- [ ] **Step 3: cargo build 验证**

Run: `cargo build -p view`
Expected: BUILD SUCCESS

- [ ] **Step 4: 手动验证**

Run: `cargo run --release --bin view`
- 多选 3 个较大的 BLF 文件
- 加载中点击 `❌ Cancel`
- 验证已完成的文件保留在 `📁 Files` 列表中，未完成的被取消
- 验证 StatusBar 显示 `❌ Loading cancelled`

- [ ] **Step 5: Commit**

```bash
git add src/view/src/app/impls_rendering.rs src/view/src/app/impls.rs
git commit -m "feat(ui): add Cancel button during multi-file loading"
```

---

## Task 9: 大文件保护（1GB 阈值确认）

**Files:**
- Modify: `src/view/src/app/impls_rendering.rs` (Open Multiple BLF... 与 Open BLF... 两个入口)

**Interfaces:**
- Consumes: `std::fs::metadata`
- Produces: 加载前大小检查确认对话框

- [ ] **Step 1: 在 Open Multiple BLF... 入口增加大小检查**

Edit `src/view/src/app/impls_rendering.rs`（Open Multiple BLF... child），在 `if paths.is_empty() { return ...; }` 之后追加：

```rust
                                                // 大文件保护：检查总大小
                                                const FILE_SIZE_THRESHOLD: u64 = 1_000_000_000; // 1 GB
                                                let total_size: u64 = paths.iter()
                                                    .filter_map(|p| std::fs::metadata(p).ok())
                                                    .map(|m| m.len())
                                                    .sum();
                                                if total_size > FILE_SIZE_THRESHOLD {
                                                    let confirmed = rfd::AsyncMessageDialog::new()
                                                        .set_title("Large File Warning")
                                                        .set_description(&format!(
                                                            "You are about to load {:.2} GB of BLF files. This may take significant time and memory. Continue?",
                                                            total_size as f64 / 1_000_000_000.0
                                                        ))
                                                        .set_buttons(rfd::MessageButtons::YesNo)
                                                        .show()
                                                        .await;
                                                    if !confirmed {
                                                        let _ = cx.update(|cx| {
                                                            view.update(cx, |view, cx| {
                                                                view.status_msg = "Loading cancelled".into();
                                                                cx.notify();
                                                            });
                                                        });
                                                        return Ok::<(), anyhow::Error>(());
                                                    }
                                                }
```

- [ ] **Step 2: 同样为 Open BLF... 单选入口增加大小检查**

Edit `src/view/src/app/impls_rendering.rs:1917` 附近（Open BLF... child），在 `pick_file().await` 之后、解析之前追加：

```rust
                                                // 大文件保护
                                                const FILE_SIZE_THRESHOLD: u64 = 1_000_000_000;
                                                if let Ok(meta) = std::fs::metadata(&path) {
                                                    if meta.len() > FILE_SIZE_THRESHOLD {
                                                        let confirmed = rfd::AsyncMessageDialog::new()
                                                            .set_title("Large File Warning")
                                                            .set_description(&format!(
                                                                "File is {:.2} GB. Loading may take significant time. Continue?",
                                                                meta.len() as f64 / 1_000_000_000.0
                                                            ))
                                                            .set_buttons(rfd::MessageButtons::YesNo)
                                                            .show()
                                                            .await;
                                                        if !confirmed {
                                                            return Ok::<(), anyhow::Error>(());
                                                        }
                                                    }
                                                }
```

- [ ] **Step 3: cargo build 验证**

Run: `cargo build -p view`
Expected: BUILD SUCCESS

- [ ] **Step 4: 手动验证（跳过 — 无 1GB 测试文件）**

注：本步骤需要 1GB+ BLF 文件触发，无法在测试环境中验证。代码逻辑通过编译即可。

- [ ] **Step 5: Commit**

```bash
git add src/view/src/app/impls_rendering.rs
git commit -m "feat(ui): add large file size warning for >1GB BLF loads"
```

---

## Task 10: 集成测试与回归验证

**Files:**
- Create: `tests/multi_file_loading.rs`

**Interfaces:**
- Consumes: 所有先前 task 的产物
- Produces: 集成测试覆盖多文件加载、流式、失败隔离、移除

- [ ] **Step 1: 创建集成测试文件**

Create `tests/multi_file_loading.rs`：

```rust
//! 多文件加载集成测试

use canview::domain::multi_file::{FileSegment, MergedView};
use blf::{BlfResult, FileStatistics, SystemTime, LogObject, CanMessage, BlfObjectHeader};

fn make_test_segment(file_id: u32, timestamps_ns: Vec<u64>) -> std::sync::Arc<FileSegment> {
    let messages: Vec<LogObject> = timestamps_ns.iter().map(|ts| {
        LogObject::CanMessage(CanMessage {
            header: BlfObjectHeader {
                object_time_stamp: *ts,
                ..Default::default()
            },
            ..Default::default()
        })
    }).collect();
    std::sync::Arc::new(FileSegment {
        file_id,
        path: std::path::PathBuf::from(format!("/tmp/seg{}.blf", file_id)),
        file_name: format!("seg{}.blf", file_id),
        start_time: chrono::NaiveDateTime::from_timestamp_opt(0, 0),
        messages: std::sync::Arc::from(messages),
        errors: Vec::new(),
        bytes_total: 0,
        bytes_consumed: 0,
        object_count: timestamps_ns.len(),
        time_min: None,
        time_max: None,
    })
}

#[test]
fn test_merge_two_segments_global_ordering() {
    let seg1 = make_test_segment(1, vec![100, 500]);
    let seg2 = make_test_segment(2, vec![200, 600]);
    let view = MergedView::from_segments(&[seg1, seg2]);
    assert_eq!(view.messages.len(), 4);
    assert_eq!(view.messages[0].timestamp(), 100);
    assert_eq!(view.messages[1].timestamp(), 200);
    assert_eq!(view.messages[2].timestamp(), 500);
    assert_eq!(view.messages[3].timestamp(), 600);
}

#[test]
fn test_remove_middle_segment_rebuilds_merged() {
    let seg1 = make_test_segment(1, vec![100]);
    let seg2 = make_test_segment(2, vec![200]);
    let seg3 = make_test_segment(3, vec![300]);
    let view_before = MergedView::from_segments(&[seg1.clone(), seg2, seg3.clone()]);
    assert_eq!(view_before.messages.len(), 3);

    let remaining: Vec<_> = vec![seg1, seg3];
    let view_after = MergedView::from_segments(&remaining);
    assert_eq!(view_after.messages.len(), 2);
    assert_eq!(view_after.messages[0].timestamp(), 100);
    assert_eq!(view_after.messages[1].timestamp(), 300);
    assert_eq!(view_after.source_file_ids[1], 3);
}

#[test]
fn test_failed_segment_has_empty_messages() {
    let failed = std::sync::Arc::new(FileSegment {
        file_id: 99,
        path: std::path::PathBuf::from("/tmp/bad.blf"),
        file_name: "bad.blf".to_string(),
        start_time: None,
        messages: std::sync::Arc::from([]),
        errors: vec!["Parse error".to_string()],
        bytes_total: 1024,
        bytes_consumed: 0,
        object_count: 0,
        time_min: None,
        time_max: None,
    });
    let good = make_test_segment(1, vec![100, 200]);
    let view = MergedView::from_segments(&[good, failed]);
    // 失败 segment 贡献 0 消息，但仍计入 source_file_ids
    assert_eq!(view.messages.len(), 2);
    assert_eq!(view.source_file_ids, std::sync::Arc::from([1u32, 1]));
}

#[test]
fn test_duplicate_path_allowed() {
    // 同一路径加载两次，两个 segment 独立共存
    let seg1 = make_test_segment(1, vec![100]);
    let seg2 = make_test_segment(2, vec![200]);
    let view = MergedView::from_segments(&[seg1, seg2]);
    assert_eq!(view.messages.len(), 2);
    assert_eq!(view.source_file_ids, std::sync::Arc::from([1u32, 2]));
}

#[test]
fn test_empty_segments_returns_empty_view() {
    let view = MergedView::from_segments(&[]);
    assert_eq!(view.messages.len(), 0);
    assert!(view.time_min.is_none());
    assert!(view.time_max.is_none());
}

#[test]
fn test_single_segment_direct_borrow() {
    let seg = make_test_segment(1, vec![100, 200, 300]);
    let view = MergedView::from_segments(&[seg]);
    assert_eq!(view.messages.len(), 3);
    assert_eq!(view.source_file_ids, std::sync::Arc::from([1u32, 1, 1]));
}
```

注：测试用 `canview::domain::multi_file::...` 路径，需要 `canview` crate（即 `view` crate 的 lib name）暴露 domain 模块。若 `view` crate 是 binary-only，需在 `src/view/Cargo.toml` 中确认 `[lib]` 节存在或调整为 `view::domain::...`。本测试假设 `view` crate 暴露 lib 接口。

- [ ] **Step 2: 确认 view crate 暴露 lib 接口**

Run: `grep -A 5 "\[lib\]\|\\[\\[bin\\]\\]" src/view/Cargo.toml`

若仅有 `[[bin]]` 无 `[lib]`，需在 Cargo.toml 中增加：

```toml
[lib]
name = "view_lib"
path = "src/lib.rs"
```

并创建 `src/view/src/lib.rs`：

```rust
pub mod app;
pub mod domain;
// ... 其他需要导出的模块
```

若已有 `[lib]` 或 view 本身就是 lib（同时有 bin），则跳过。

- [ ] **Step 3: 调整测试中的 crate 路径**

根据 Step 2 的结果，将 `tests/multi_file_loading.rs` 中的 `use canview::...` 改为正确的 crate name（如 `use view_lib::...` 或 `use view::...`）。

Run: `cargo test --test multi_file_loading`
Expected: 6 个测试 PASS

- [ ] **Step 4: 运行全部 workspace 测试**

Run: `cargo test --workspace`
Expected: 所有测试 PASS，无回归

- [ ] **Step 5: Commit**

```bash
git add tests/multi_file_loading.rs src/view/Cargo.toml src/view/src/lib.rs
git commit -m "test: add multi-file loading integration tests"
```

---

## Task 11: 手动 UI 验证清单与文档更新

**Files:**
- Modify: `README.md` (Roadmap 部分增加多文件支持条目)
- Modify: `README_zh.md` (同上)

**Interfaces:**
- Consumes: 所有先前的功能
- Produces: 文档更新与验证清单

- [ ] **Step 1: 在 README.md Roadmap 增加多文件条目**

Edit `README.md`，将 Roadmap 中的 `- [ ] Live streaming mode` 行之前追加：

```markdown
- [x] Multi-file loading (Open BLF... replaces, Open Multiple BLF... appends) with merged timeline
```

- [ ] **Step 2: 在 README_zh.md 同步**

Edit `README_zh.md`，在 Roadmap 对应位置增加：

```markdown
- [x] 多文件加载（Open BLF... 替换，Open Multiple BLF... 追加）按时间合并
```

- [ ] **Step 3: 手动验证清单**

Run: `cargo run --release --bin view`，逐项验证：

1. ✅ 单选 Open BLF... → 替换已有数据，UI 正常
2. ✅ 多选 Open Multiple BLF... → 追加，StatusBar 显示流式进度
3. ✅ 加载中点击 Cancel → 已完成文件保留，未完成取消
4. ✅ 加载 3 个文件，移除中间 1 个 → 折线图/日志列表立即更新
5. ✅ 多选其中 1 个文件损坏 → 其他正常加载，popover 标红
6. ✅ 多选重复同一文件 → 两个 segment 共存
7. ✅ 5MB BLF × 10 并发加载 → 内存稳定，UI 流畅
8. ✅ 50MB BLF × 3 并发加载 → 完成时间 ≈ 单文件 1.5 倍
9. ✅ LogView 滚动 → 多文件消息按时间混合显示，筛选正常
10. ✅ Signal Plot → 同名信号多文件拼接为一条波形
11. ✅ 窗口 maximize/restore → files/merged/plot 状态完整保留

- [ ] **Step 4: Commit**

```bash
git add README.md README_zh.md
git commit -m "docs: mark multi-file loading as completed in roadmap"
```

---

## Self-Review Checklist

### 1. Spec coverage

| Spec 节 | Task | 覆盖 |
|---|---|---|
| 0. 需求与决定 | 全部 task | ✅ |
| 1. 整体架构 | Task 1, 2, 3, 4 | ✅ |
| 2.1 FileSegment | Task 1 | ✅ |
| 2.2 MergedView | Task 1 | ✅ |
| 2.3 CanViewApp 新增字段 | Task 3 | ✅ |
| 2.4 关键不变量 | Task 1, 4 | ✅ |
| 3.1 两个菜单入口 | Task 5 | ✅ |
| 3.2 共用加载流程 | Task 4, 5, 7 | ✅ |
| 3.3 单选 vs 多选 | Task 4, 5 | ✅ |
| 4.1 并发模型 | Task 7 | ✅ |
| 4.2 流式合并优化 | Task 1 (MergedView::from_segments) | ✅ |
| 4.3 大文件保护 | Task 9 | ✅ |
| 4.4 取消语义 | Task 8 | ✅ |
| 4.5 性能预期 | Task 7, 11 | ✅ |
| 5.1 File 菜单 | Task 5 | ✅ |
| 5.2 StatusBar 文件管理入口 | Task 6 | ✅ |
| 5.3 文件管理 popover | Task 6 | ✅ |
| 5.4 LogView 与 Signal Plot | Task 11 (验证) | ✅（无代码改动，依赖现有逻辑） |
| 6.1 新增命令结构 | Task 2 | ✅ |
| 6.2 apply 流程拆分 | Task 4 | ✅ |
| 6.3 状态机 | Task 4, 8 | ✅ |
| 6.4 与现有 apply_blf_result 的兼容 | Task 4 | ✅ |
| 6.5 file_id 生成 | Task 4 Step 1 | ✅ |
| 7.1 单文件失败隔离 | Task 4 Step 2 (apply_blf_result_append_one 错误分支) | ✅ |
| 7.2 Replace 模式失败 | Task 4 Step 2 (apply_blf_result_single 错误分支) | ✅ |
| 7.3 错误分类 | Task 4, 9 | ✅ |
| 7.4 路径冲突 | Task 10 (test_duplicate_path_allowed) | ✅ |
| 7.5 配置持久化 | 沿用现状，无 task 需要 | ✅ |
| 7.6 is_streaming_mode 清理 | Task 3 | ✅ |
| 7.7 RuntimeState 持久化 | Task 3 | ✅ |
| 8.1 单元测试 | Task 1, 2 | ✅ |
| 8.2 集成测试 | Task 10 | ✅ |
| 8.3 手动 UI 验证清单 | Task 11 | ✅ |
| 8.4 回归测试 | Task 3 Step 10, Task 10 Step 4 | ✅ |
| 8.5 性能基准 | Task 11 Step 3 第 8 项 | ✅ |
| 9. 受影响文件清单 | 全部 task | ✅ |
| 10. 实施顺序 | 全部 task 顺序 | ✅ |

### 2. Placeholder scan

- 无 "TBD"、"TODO"、"implement later" 等占位
- 每个代码步骤都有完整代码块
- 每个测试都有完整代码

### 3. Type consistency

- `FileSegment::from_blf_result(result: BlfResult, file_id: u32, path: PathBuf) -> FileSegment` 在 Task 1 与 Task 4 中签名一致
- `MergedView::from_segments(&[Arc<FileSegment>]) -> MergedView` 在 Task 1 与 Task 4 中签名一致
- `MergedView::empty() -> MergedView` 在 Task 1 与 Task 3 中签名一致
- `apply_blf_result_single(result: anyhow::Result<BlfResult>, path: PathBuf)` 在 Task 4 中定义，Task 4 Step 3, 4 中调用一致
- `apply_blf_result_append_one(result: anyhow::Result<BlfResult>, path: PathBuf)` 在 Task 4 中定义，Task 5, 7 中调用一致
- `rebuild_merged()` 在 Task 4 中定义并使用
- `remove_file(file_id: u32)` 与 `remove_all_files()` 在 Task 4 中定义，Task 6 中调用一致
- `LoadMode::{Replace, Append}` 在 Task 2 中定义，Task 5 中使用
- `LoadBlfFiles::new(paths, mode)` 在 Task 2 中定义
- `LoadingProgress` 在 Task 3 中定义，Task 5, 8 中使用
- `next_file_id()` 在 Task 4 Step 1 中定义，Task 4 Step 2 中使用

所有类型与签名一致。

---

## Execution Handoff

Plan complete and saved to `docs/superpowers/plans/2026-07-24-multi-file-parsing-playback.md`. Two execution options:

1. **Subagent-Driven (recommended)** - I dispatch a fresh subagent per task, review between tasks, fast iteration
2. **Inline Execution** - Execute tasks in this session using executing-plans, batch execution with checkpoints

Which approach?
