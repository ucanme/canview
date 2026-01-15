// src/view/src/playback.rs
//! 信号回放引擎
//!
//! 提供基于时间戳的消息回放功能,支持变速播放、暂停、seek等操作

use std::collections::VecDeque;
use std::time::{Duration, Instant};
use blf::LogObject;

/// 回放状态
#[derive(Debug, Clone)]
pub struct PlaybackState {
    // 播放控制
    is_playing: bool,
    current_time: Duration,
    playback_speed: f32,

    // 时间范围
    start_time: Duration,
    end_time: Duration,
    total_duration: Duration,

    // 循环模式
    loop_mode: bool,

    // 内部状态
    message_index: usize,
    last_update: Instant,
    pending_messages: VecDeque<LogObject>,

    // 时间索引 (加速seek)
    time_index: TimeIndex,
}

/// 时间索引 - 用于快速定位消息
#[derive(Debug, Clone)]
pub struct TimeIndex {
    timestamps: Vec<Duration>,    // 排序的时间戳
    message_indices: Vec<usize>,  // 对应的消息索引
}

impl TimeIndex {
    /// 从消息列表构建时间索引
    pub fn build(messages: &[LogObject]) -> Self {
        let mut timestamps = Vec::with_capacity(messages.len());
        let mut message_indices = Vec::with_capacity(messages.len());

        for (index, msg) in messages.iter().enumerate() {
            let ts = msg.timestamp();
            if ts > 0 {
                timestamps.push(Duration::from_nanos(ts));
                message_indices.push(index);
            }
        }

        Self {
            timestamps,
            message_indices,
        }
    }

    /// 二分查找定位消息
    pub fn find_message(&self, time: Duration) -> usize {
        match self.timestamps.binary_search(&time) {
            Ok(idx) => self.message_indices[idx],
            Err(idx) => {
                if idx == 0 {
                    self.message_indices[0]
                } else if idx >= self.message_indices.len() {
                    *self.message_indices.last().unwrap_or(&0)
                } else {
                    self.message_indices[idx]
                }
            }
        }
    }

    /// 获取时间范围内的消息
    pub fn get_range(&self, start: Duration, end: Duration) -> Vec<usize> {
        let start_idx = match self.timestamps.binary_search(&start) {
            Ok(i) | Err(i) => i,
        };
        let end_idx = match self.timestamps.binary_search(&end) {
            Ok(i) => i + 1,
            Err(i) => i,
        };

        self.message_indices[start_idx..end_idx.min(self.message_indices.len())]
            .to_vec()
    }
}

impl PlaybackState {
    /// 创建新的回放状态
    pub fn new(messages: &[LogObject]) -> Self {
        let time_index = TimeIndex::build(messages);

        let start_time = time_index.timestamps.first()
            .copied()
            .unwrap_or(Duration::ZERO);

        let end_time = time_index.timestamps.last()
            .copied()
            .unwrap_or(Duration::ZERO);

        let total_duration = end_time.saturating_sub(start_time);

        Self {
            is_playing: false,
            current_time: Duration::ZERO,
            playback_speed: 1.0,
            start_time,
            end_time,
            total_duration,
            loop_mode: false,
            message_index: 0,
            last_update: Instant::now(),
            pending_messages: VecDeque::new(),
            time_index,
        }
    }

    /// 开始/恢复播放
    pub fn play(&mut self) {
        self.is_playing = true;
        self.last_update = Instant::now();
    }

    /// 暂停播放
    pub fn pause(&mut self) {
        self.is_playing = false;
    }

    /// 停止播放并重置到开始
    pub fn stop(&mut self) {
        self.is_playing = false;
        self.current_time = Duration::ZERO;
        self.message_index = 0;
        self.pending_messages.clear();
    }

    /// 跳转到指定时间
    pub fn seek(&mut self, time: Duration) {
        let target_time = time.min(self.total_duration);
        self.current_time = target_time;
        self.message_index = self.time_index.find_message(
            self.start_time.saturating_add(target_time)
        );
        self.pending_messages.clear();
    }

    /// 设置播放速度
    pub fn set_speed(&mut self, speed: f32) {
        self.playback_speed = speed.clamp(0.1, 10.0);
    }

    /// 切换循环模式
    pub fn toggle_loop(&mut self) {
        self.loop_mode = !self.loop_mode;
    }

    /// 更新回放状态 (每帧调用)
    pub fn update(&mut self, messages: &[LogObject], dt: Duration) -> Vec<usize> {
        if !self.is_playing {
            return Vec::new();
        }

        // 根据播放速度计算虚拟时间增量
        let virtual_dt = dt.mul_f32(self.playback_speed);
        self.current_time = self.current_time.saturating_add(virtual_dt);

        // 检查是否到达终点
        if self.current_time >= self.total_duration {
            if self.loop_mode {
                self.seek(Duration::ZERO);
            } else {
                self.pause();
                return Vec::new();
            }
        }

        // 获取当前时间窗口内的消息索引
        let current_absolute_time = self.start_time.saturating_add(self.current_time);
        let next_absolute_time = current_absolute_time.saturating_add(virtual_dt);

        self.time_index.get_range(current_absolute_time, next_absolute_time)
    }

    /// 获取当前进度 (0.0 - 1.0)
    pub fn progress(&self) -> f32 {
        if self.total_duration.is_zero() {
            0.0
        } else {
            self.current_time.as_secs_f32() / self.total_duration.as_secs_f32()
        }
    }

    /// 格式化时间显示
    pub fn format_time(&self) -> String {
        format_time(self.current_time)
    }

    /// 格式化总时间显示
    pub fn format_total_time(&self) -> String {
        format_time(self.total_duration)
    }

    /// 获取是否正在播放
    pub fn is_playing(&self) -> bool {
        self.is_playing
    }

    /// 获取播放速度
    pub fn playback_speed(&self) -> f32 {
        self.playback_speed
    }

    /// 获取循环模式
    pub fn loop_mode(&self) -> bool {
        self.loop_mode
    }
}

/// 格式化时间显示为 HH:MM:SS.mmm
fn format_time(duration: Duration) -> String {
    let total_ms = duration.as_millis();
    let ms = (total_ms % 1000) as u32;
    let total_seconds = (total_ms / 1000) as u32;
    let seconds = total_seconds % 60;
    let total_minutes = total_seconds / 60;
    let minutes = total_minutes % 60;
    let hours = total_minutes / 60;

    if hours > 0 {
        format!("{:02}:{:02}:{:02}.{:03}", hours, minutes, seconds, ms)
    } else {
        format!("{:02}:{:02}.{:03}", minutes, seconds, ms)
    }
}

// Duration扩展方法
trait DurationExt {
    fn mul_f32(self, factor: f32) -> Duration;
}

impl DurationExt for Duration {
    fn mul_f32(self, factor: f32) -> Duration {
        Duration::from_millis((self.as_millis() as f64 * factor as f64) as u64)
    }
}
