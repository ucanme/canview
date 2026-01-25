use gpui::Rgba;

/// 单个数据点
#[derive(Debug, Clone, Copy)]
pub struct TimeSeriesPoint {
    /// 时间戳（秒），相对于开始时间
    pub timestamp: f64,
    /// 物理值
    pub value: f64,
}

/// 信号的时间序列数据
#[derive(Debug, Clone)]
pub struct SignalSeries {
    /// 信号名称
    pub name: String,
    /// 单位
    pub unit: String,
    /// 数据点序列
    pub points: Vec<TimeSeriesPoint>,
    /// 显示颜色
    pub color: Rgba,
    /// 是否可见
    pub visible: bool,
    /// 最小值（用于优化渲染）
    pub min_value: f64,
    /// 最大值（用于优化渲染）
    pub max_value: f64,
}

impl SignalSeries {
    pub fn new(name: String, unit: String, color: Rgba) -> Self {
        Self {
            name,
            unit,
            points: Vec::new(),
            color,
            visible: true,
            min_value: f64::MAX,
            max_value: f64::MIN,
        }
    }

    pub fn add_point(&mut self, timestamp: f64, value: f64) {
        self.points.push(TimeSeriesPoint { timestamp, value });
        if value < self.min_value {
            self.min_value = value;
        }
        if value > self.max_value {
            self.max_value = value;
        }
    }
}
