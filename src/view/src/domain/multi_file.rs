//! 多文件加载与全局合并视图
//!
//! FileSegment 表示一个已加载的 BLF 文件段；MergedView 表示多文件按全局绝对时间合并后的视图。
//!
//! 注意：本模块的类型与方法在 Task 2+ 才会被 UI 层引用，在此之前会有 dead_code 警告。
//! `#![allow(dead_code)]` 是临时措施，Task 2+ 接入后即可移除。

#![allow(dead_code)]

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
    pub time_min: Option<f64>, // 全局秒，UNIX epoch 基准
    pub time_max: Option<f64>,
}

/// 全局合并视图
#[derive(Clone)]
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
            Utc
                .with_ymd_and_hms(
                    st.year as i32,
                    st.month as u32,
                    st.day as u32,
                    st.hour as u32,
                    st.minute as u32,
                    st.second as u32,
                )
                .single()
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
                if abs_ns < min_ns {
                    min_ns = abs_ns;
                }
                if abs_ns > max_ns {
                    max_ns = abs_ns;
                }
            }
            (
                Some(min_ns as f64 / 1_000_000_000.0),
                Some(max_ns as f64 / 1_000_000_000.0),
            )
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

impl MergedView {
    /// 从多个 FileSegment 构建全局合并视图
    ///
    /// 按 (绝对时间秒, file_id, msg_idx) 稳定排序。
    /// messages 中每条 LogObject 是从 segment clone 过来的。
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

        // 多 segment：收集 (abs_sec, file_id, msg_idx, source_file_id, LogObject) 然后稳定排序
        let start_ns: Vec<i64> = segments
            .iter()
            .map(|s| {
                s.start_time
                    .map(|ndt| ndt.and_utc().timestamp_nanos_opt().unwrap_or(0))
                    .unwrap_or(0)
            })
            .collect();

        let mut entries: Vec<(f64, u32, usize, u32, LogObject)> = Vec::new();
        for (seg_idx, seg) in segments.iter().enumerate() {
            for (msg_idx, msg) in seg.messages.iter().enumerate() {
                let abs_ns = start_ns[seg_idx] + msg.timestamp() as i64;
                let abs_sec = abs_ns as f64 / 1_000_000_000.0;
                entries.push((abs_sec, seg.file_id, msg_idx, seg.file_id, msg.clone()));
            }
        }

        entries.sort_by(|a, b| {
            a.0.partial_cmp(&b.0)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.1.cmp(&b.1))
                .then(a.2.cmp(&b.2))
        });

        let total = entries.len();
        let mut messages_vec = Vec::with_capacity(total);
        let mut source_ids_vec = Vec::with_capacity(total);
        let mut min_sec = f64::INFINITY;
        let mut max_sec = f64::NEG_INFINITY;

        for (abs_sec, _fid, _idx, src_fid, msg) in entries {
            if abs_sec < min_sec {
                min_sec = abs_sec;
            }
            if abs_sec > max_sec {
                max_sec = abs_sec;
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use blf::{BlfResult, CanMessage, FileStatistics, ObjectHeader, SystemTime};
    use std::path::PathBuf;

    fn make_test_blf_result() -> BlfResult {
        BlfResult {
            file_stats: FileStatistics {
                statistics_size: 0,
                api_number: 0,
                application_id: 0,
                compression_level: 0,
                application_major: 0,
                application_minor: 0,
                file_size: 0,
                uncompressed_file_size: 0,
                object_count: 0,
                application_build: 0,
                measurement_start_time: SystemTime {
                    year: 2026,
                    month: 7,
                    day_of_week: 5,
                    day: 24,
                    hour: 10,
                    minute: 0,
                    second: 0,
                    milliseconds: 0,
                },
                last_object_time: SystemTime {
                    year: 2026,
                    month: 7,
                    day_of_week: 5,
                    day: 24,
                    hour: 10,
                    minute: 0,
                    second: 0,
                    milliseconds: 0,
                },
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

    #[test]
    fn test_merged_view_from_empty_segments() {
        let view = MergedView::from_segments(&[]);
        assert_eq!(view.messages.len(), 0);
        assert_eq!(view.source_file_ids.len(), 0);
        assert!(view.time_min.is_none());
        assert!(view.time_max.is_none());
        assert_eq!(view.version, 0);
    }

    fn make_seg(file_id: u32, msgs: Vec<u64>) -> Arc<FileSegment> {
        // msgs: Vec<timestamp_ns>。start_time 固定为 UNIX epoch（DateTime::from_timestamp(0, 0)）
        // 绝对时间 = 0 + msg.timestamp()，与 from_blf_result 计算方式一致
        let messages: Vec<LogObject> = msgs
            .iter()
            .map(|ts| {
                LogObject::CanMessage(CanMessage {
                    header: ObjectHeader {
                        object_time_stamp: *ts,
                        ..Default::default()
                    },
                    ..Default::default()
                })
            })
            .collect();
        // 与 from_blf_result 保持一致：start_ns = 0，abs_ns = start_ns + msg.timestamp()
        let (time_min, time_max) = if msgs.is_empty() {
            (None, None)
        } else {
            let mut min_ns = i64::MAX;
            let mut max_ns = i64::MIN;
            for ts in msgs.iter() {
                let abs_ns = *ts as i64;
                if abs_ns < min_ns {
                    min_ns = abs_ns;
                }
                if abs_ns > max_ns {
                    max_ns = abs_ns;
                }
            }
            (
                Some(min_ns as f64 / 1_000_000_000.0),
                Some(max_ns as f64 / 1_000_000_000.0),
            )
        };
        Arc::new(FileSegment {
            file_id,
            path: PathBuf::from(format!("/tmp/seg{}.blf", file_id)),
            file_name: format!("seg{}.blf", file_id),
            start_time: chrono::DateTime::from_timestamp(0, 0).map(|dt| dt.naive_utc()),
            messages: Arc::from(messages),
            errors: Vec::new(),
            bytes_total: 0,
            bytes_consumed: 0,
            object_count: msgs.len(),
            time_min,
            time_max,
        })
    }

    #[test]
    fn test_merged_view_single_segment() {
        let seg = make_seg(1, vec![100_000, 200_000, 300_000]);
        let view = MergedView::from_segments(&[seg]);
        assert_eq!(view.messages.len(), 3);
        assert_eq!(view.source_file_ids.len(), 3);
        assert_eq!(view.source_file_ids[0], 1);
        assert!(view.time_min.is_some());
        assert!(view.time_max.is_some());
    }

    #[test]
    fn test_merged_view_two_segments_sorted_by_time() {
        // seg1 时间戳: 100ns, 500ns
        // seg2 时间戳: 200ns, 600ns
        // 合并后顺序: 100(seg1), 200(seg2), 500(seg1), 600(seg2)
        let seg1 = make_seg(1, vec![100, 500]);
        let seg2 = make_seg(2, vec![200, 600]);
        let view = MergedView::from_segments(&[seg1, seg2]);
        assert_eq!(view.messages.len(), 4);
        assert_eq!(view.source_file_ids, Arc::from([1u32, 2, 1, 2]));
        assert_eq!(view.messages[0].timestamp(), 100);
        assert_eq!(view.messages[1].timestamp(), 200);
        assert_eq!(view.messages[2].timestamp(), 500);
        assert_eq!(view.messages[3].timestamp(), 600);
    }

    #[test]
    fn test_merged_view_after_remove_middle() {
        // 三个 segment: seg1 (100ns), seg2 (200ns), seg3 (300ns)
        // 移除 seg2 后: 100(seg1), 300(seg3)
        let seg1 = make_seg(1, vec![100]);
        let seg2 = make_seg(2, vec![200]);
        let seg3 = make_seg(3, vec![300]);
        let view_before = MergedView::from_segments(&[seg1.clone(), seg2, seg3.clone()]);
        assert_eq!(view_before.messages.len(), 3);

        let remaining: Vec<Arc<FileSegment>> = vec![seg1, seg3];
        let view_after = MergedView::from_segments(&remaining);
        assert_eq!(view_after.messages.len(), 2);
        assert_eq!(view_after.messages[0].timestamp(), 100);
        assert_eq!(view_after.messages[1].timestamp(), 300);
        assert_eq!(view_after.source_file_ids, Arc::from([1u32, 3]));
    }

    #[test]
    fn test_failed_segment_does_not_block_others() {
        // 一个失败 segment（messages 为空）+ 一个正常 segment
        // 失败 segment 不贡献 messages，但仍计入 source_file_ids 列表
        let good = make_seg(1, vec![100, 200]);
        let failed = Arc::new(FileSegment {
            file_id: 99,
            path: PathBuf::from("/tmp/bad.blf"),
            file_name: "bad.blf".to_string(),
            start_time: chrono::NaiveDateTime::from_timestamp_opt(0, 0),
            messages: Arc::from([]),
            errors: vec!["Parse error".to_string()],
            bytes_total: 1024,
            bytes_consumed: 0,
            object_count: 0,
            time_min: None,
            time_max: None,
        });
        let view = MergedView::from_segments(&[good, failed]);
        assert_eq!(view.messages.len(), 2);
        // 失败 segment 没贡献消息，source_file_ids 全是 good 的
        assert_eq!(view.source_file_ids, Arc::from([1u32, 1]));
    }

    #[test]
    fn test_duplicate_path_coexists_as_independent_segments() {
        // 同一路径加载两次：两个 segment 独立共存
        let seg1 = make_seg(1, vec![100]);
        let seg2 = make_seg(2, vec![200]);
        let view = MergedView::from_segments(&[seg1, seg2]);
        assert_eq!(view.messages.len(), 2);
        assert_eq!(view.source_file_ids, Arc::from([1u32, 2]));
    }

    #[test]
    fn test_merged_view_time_range_computed() {
        // 单 segment：merged 的 time_min/time_max 应等于 segment 的 time_min/time_max
        let seg = make_seg(1, vec![100, 500]);
        let view = MergedView::from_segments(&[seg]);
        assert!(view.time_min.is_some());
        assert!(view.time_max.is_some());
        assert!(view.time_min.unwrap() <= view.time_max.unwrap());
    }

    #[test]
    fn test_merge_three_segments_preserves_global_order() {
        // 三 segment 时间戳交错
        let seg1 = make_seg(1, vec![100, 700]);
        let seg2 = make_seg(2, vec![200, 800]);
        let seg3 = make_seg(3, vec![300, 900]);
        let view = MergedView::from_segments(&[seg1, seg2, seg3]);
        assert_eq!(view.messages.len(), 6);
        // 验证全局顺序
        assert_eq!(view.messages[0].timestamp(), 100);
        assert_eq!(view.messages[1].timestamp(), 200);
        assert_eq!(view.messages[2].timestamp(), 300);
        assert_eq!(view.messages[3].timestamp(), 700);
        assert_eq!(view.messages[4].timestamp(), 800);
        assert_eq!(view.messages[5].timestamp(), 900);
        // 验证 source_file_ids
        assert_eq!(view.source_file_ids, Arc::from([1u32, 2, 3, 1, 2, 3]));
    }

    #[test]
    fn test_merge_preserves_stable_order_for_same_timestamp() {
        // 同一 timestamp，应按 (file_id, msg_idx) 稳定排序
        let seg1 = make_seg(1, vec![100, 100]);
        let seg2 = make_seg(2, vec![100, 100]);
        let view = MergedView::from_segments(&[seg1, seg2]);
        assert_eq!(view.messages.len(), 4);
        // 4 条都是 100ns
        for msg in view.messages.iter() {
            assert_eq!(msg.timestamp(), 100);
        }
        // 排序：(100, fid=1, idx=0), (100, fid=1, idx=1), (100, fid=2, idx=0), (100, fid=2, idx=1)
        assert_eq!(view.source_file_ids, Arc::from([1u32, 1, 2, 2]));
    }
}
