# GPUI 原生图表实现方案

## 🎯 方案选择

使用 **GPUI 原生绘图 API** 而不是 plotters，原因：

| 特性 | GPUI 原生 | plotters |
|------|-----------|----------|
| 性能 | ⭐⭐⭐⭐⭐ GPU 加速 | ⭐⭐⭐ CPU 渲染 |
| 集成 | ⭐⭐⭐⭐⭐ 原生 | ⭐⭐ 需要图像转换 |
| 交互 | ⭐⭐⭐⭐⭐ 直接事件 | ⭐⭐ 需要坐标映射 |
| 流畅度 | ⭐⭐⭐⭐⭐ 矢量图形 | ⭐⭐⭐ 位图 |
| 文件大小 | ⭐⭐⭐⭐⭐ 无额外依赖 | ⭐⭐⭐ 需要 plotters |

## 🏗️ GPUI 绘图 API

### 核心 API

```rust
// 1. Canvas - 自定义绘制
canvas(
    |bounds, cx| {
        // 返回绘制内容
    },
    |bounds, element, cx| {
        // paint 回调 - 实际绘制
        cx.paint_layer(bounds, |cx| {
            // 绘制路径
            cx.paint_path(path, color);
            // 绘制矩形
            cx.paint_quad(quad);
        });
    }
)

// 2. PathBuilder - 构建路径
let mut path = PathBuilder::new();
path.move_to(point(x1, y1));
path.line_to(point(x2, y2));
path.curve_to(cp1, cp2, end);
let path = path.build();

// 3. 绘制路径
window.paint_path(path, color);

// 4. 绘制矩形
window.paint_quad(quad);
```

## 📊 图表组件设计

### 1. ChartCanvas 组件

```rust
// src/view/src/ui/components/chart_canvas.rs

use gpui::*;

pub struct ChartCanvas {
    time_series: Vec<SignalTimeSeries>,
    bounds: Bounds<Pixels>,
    zoom: f32,
    offset: Point<Pixels>,
}

impl ChartCanvas {
    pub fn new(time_series: Vec<SignalTimeSeries>) -> Self {
        Self {
            time_series,
            bounds: Bounds::default(),
            zoom: 1.0,
            offset: point(px(0.0), px(0.0)),
        }
    }
    
    fn render(&self, cx: &mut WindowContext) -> impl IntoElement {
        canvas(
            move |bounds, cx| {
                // 准备数据
                ChartElement {
                    time_series: self.time_series.clone(),
                    bounds,
                }
            },
            |bounds, element, cx| {
                // 绘制图表
                self.paint_chart(bounds, &element.time_series, cx);
            }
        )
    }
    
    fn paint_chart(
        &self,
        bounds: Bounds<Pixels>,
        time_series: &[SignalTimeSeries],
        cx: &mut WindowContext
    ) {
        // 1. 绘制背景
        self.paint_background(bounds, cx);
        
        // 2. 绘制网格
        self.paint_grid(bounds, cx);
        
        // 3. 绘制坐标轴
        self.paint_axes(bounds, cx);
        
        // 4. 绘制折线
        for series in time_series {
            self.paint_line(bounds, series, cx);
        }
        
        // 5. 绘制图例
        self.paint_legend(bounds, time_series, cx);
    }
}
```

### 2. 绘制折线

```rust
fn paint_line(
    &self,
    bounds: Bounds<Pixels>,
    series: &SignalTimeSeries,
    cx: &mut WindowContext
) {
    if series.points.is_empty() {
        return;
    }
    
    // 计算坐标转换
    let x_scale = bounds.size.width / self.time_range();
    let y_scale = bounds.size.height / self.value_range();
    
    // 构建路径
    let mut path = PathBuilder::new();
    
    // 第一个点
    let first = &series.points[0];
    let x = bounds.origin.x + px(first.timestamp as f32 * x_scale.0);
    let y = bounds.origin.y + bounds.size.height - px(first.value as f32 * y_scale.0);
    path.move_to(point(x, y));
    
    // 后续点
    for point in &series.points[1..] {
        let x = bounds.origin.x + px(point.timestamp as f32 * x_scale.0);
        let y = bounds.origin.y + bounds.size.height - px(point.value as f32 * y_scale.0);
        path.line_to(point(x, y));
    }
    
    // 绘制路径
    cx.paint_path(path.build(), series.color);
}
```

### 3. 绘制网格

```rust
fn paint_grid(
    &self,
    bounds: Bounds<Pixels>,
    cx: &mut WindowContext
) {
    let grid_color = rgb(0x2a2a2a);
    
    // 垂直网格线（时间）
    let num_vertical = 10;
    for i in 0..=num_vertical {
        let x = bounds.origin.x + bounds.size.width * (i as f32 / num_vertical as f32);
        
        let mut path = PathBuilder::new();
        path.move_to(point(x, bounds.origin.y));
        path.line_to(point(x, bounds.origin.y + bounds.size.height));
        
        cx.paint_path(path.build(), grid_color);
    }
    
    // 水平网格线（值）
    let num_horizontal = 8;
    for i in 0..=num_horizontal {
        let y = bounds.origin.y + bounds.size.height * (i as f32 / num_horizontal as f32);
        
        let mut path = PathBuilder::new();
        path.move_to(point(bounds.origin.x, y));
        path.line_to(point(bounds.origin.x + bounds.size.width, y));
        
        cx.paint_path(path.build(), grid_color);
    }
}
```

### 4. 绘制坐标轴

```rust
fn paint_axes(
    &self,
    bounds: Bounds<Pixels>,
    cx: &mut WindowContext
) {
    let axis_color = rgb(0x646473);
    
    // X 轴
    let mut x_axis = PathBuilder::new();
    x_axis.move_to(point(bounds.origin.x, bounds.origin.y + bounds.size.height));
    x_axis.line_to(point(
        bounds.origin.x + bounds.size.width,
        bounds.origin.y + bounds.size.height
    ));
    cx.paint_path(x_axis.build(), axis_color);
    
    // Y 轴
    let mut y_axis = PathBuilder::new();
    y_axis.move_to(point(bounds.origin.x, bounds.origin.y));
    y_axis.line_to(point(bounds.origin.x, bounds.origin.y + bounds.size.height));
    cx.paint_path(y_axis.build(), axis_color);
    
    // 刻度标签（使用 text）
    self.paint_axis_labels(bounds, cx);
}
```

### 5. 绘制图例

```rust
fn paint_legend(
    &self,
    bounds: Bounds<Pixels>,
    time_series: &[SignalTimeSeries],
    cx: &mut WindowContext
) {
    let legend_x = bounds.origin.x + px(20.0);
    let mut legend_y = bounds.origin.y + px(20.0);
    
    for series in time_series {
        // 绘制颜色块
        let color_box = Bounds {
            origin: point(legend_x, legend_y),
            size: size(px(20.0), px(10.0)),
        };
        cx.paint_quad(quad(
            color_box,
            corner_radii(px(2.0)),
            series.color,
            Edges::default(),
            Edges::default(),
        ));
        
        // 绘制文本（信号名称）
        // 使用 div() + text() 组件
        
        legend_y += px(20.0);
    }
}
```

## 🎨 完整示例

```rust
// src/view/src/ui/views/chart_view.rs

use gpui::*;
use crate::ui::components::chart_canvas::ChartCanvas;

pub fn render_chart_view(
    time_series: Vec<SignalTimeSeries>,
    cx: &mut ViewContext<CanViewApp>
) -> impl IntoElement {
    div()
        .flex()
        .size_full()
        .bg(rgb(0x0a0a0a))
        .child(
            // 左侧：信号选择器
            div()
                .w(px(250.0))
                .h_full()
                .bg(rgb(0x0f0f0f))
                .border_r_1()
                .border_color(rgb(0x1a1a1a))
                .child(render_signal_selector(cx))
        )
        .child(
            // 右侧：图表区域
            div()
                .flex_1()
                .h_full()
                .p_4()
                .child(
                    // 图表画布
                    canvas(
                        move |bounds, cx| {
                            ChartElement {
                                time_series: time_series.clone(),
                                bounds,
                            }
                        },
                        move |bounds, element, cx| {
                            paint_chart(bounds, &element.time_series, cx);
                        }
                    )
                    .size_full()
                )
        )
}

struct ChartElement {
    time_series: Vec<SignalTimeSeries>,
    bounds: Bounds<Pixels>,
}

fn paint_chart(
    bounds: Bounds<Pixels>,
    time_series: &[SignalTimeSeries],
    cx: &mut WindowContext
) {
    // 背景
    cx.paint_quad(quad(
        bounds,
        corner_radii(px(0.0)),
        rgb(0x0a0a0a),
        Edges::default(),
        Edges::default(),
    ));
    
    // 网格
    paint_grid(bounds, cx);
    
    // 折线
    for series in time_series {
        paint_line(bounds, series, cx);
    }
    
    // 坐标轴
    paint_axes(bounds, cx);
    
    // 图例
    paint_legend(bounds, time_series, cx);
}
```

## 🎯 交互功能

### 1. 缩放

```rust
.on_scroll_wheel(cx.listener(|this, event: &ScrollWheelEvent, cx| {
    let delta = event.delta.y;
    this.chart_zoom *= 1.0 + delta * 0.001;
    this.chart_zoom = this.chart_zoom.clamp(0.1, 10.0);
    cx.notify();
}))
```

### 2. 平移

```rust
.on_mouse_down(MouseButton::Left, cx.listener(|this, event, cx| {
    this.chart_dragging = true;
    this.drag_start = event.position;
}))
.on_mouse_move(cx.listener(|this, event, cx| {
    if this.chart_dragging {
        let delta = event.position - this.drag_start;
        this.chart_offset += delta;
        this.drag_start = event.position;
        cx.notify();
    }
}))
.on_mouse_up(MouseButton::Left, cx.listener(|this, event, cx| {
    this.chart_dragging = false;
}))
```

### 3. 悬停显示值

```rust
.on_mouse_move(cx.listener(|this, event, cx| {
    // 计算鼠标位置对应的时间和值
    let time = this.pixel_to_time(event.position.x);
    let value = this.pixel_to_value(event.position.y);
    
    // 查找最近的数据点
    if let Some(point) = this.find_nearest_point(time) {
        this.hover_info = Some(HoverInfo {
            time: point.timestamp,
            value: point.value,
            position: event.position,
        });
        cx.notify();
    }
}))
```

## 📊 性能优化

### 1. 数据抽样

```rust
fn downsample_points(points: &[TimeSeriesPoint], max_points: usize) -> Vec<TimeSeriesPoint> {
    if points.len() <= max_points {
        return points.to_vec();
    }
    
    let step = points.len() / max_points;
    points.iter()
        .step_by(step)
        .cloned()
        .collect()
}
```

### 2. 可见范围裁剪

```rust
fn clip_to_visible_range(
    points: &[TimeSeriesPoint],
    time_range: (f64, f64)
) -> Vec<TimeSeriesPoint> {
    points.iter()
        .filter(|p| p.timestamp >= time_range.0 && p.timestamp <= time_range.1)
        .cloned()
        .collect()
}
```

### 3. 路径缓存

```rust
struct ChartCache {
    path: Option<Path>,
    data_version: usize,
}

impl ChartCache {
    fn get_or_build(&mut self, points: &[TimeSeriesPoint]) -> &Path {
        if self.path.is_none() {
            self.path = Some(build_path(points));
        }
        self.path.as_ref().unwrap()
    }
}
```

## ✅ 优势总结

### GPUI 原生方案

1. **性能** ⭐⭐⭐⭐⭐
   - GPU 加速矢量渲染
   - 无图像编码/解码开销
   - 流畅的 60fps+

2. **集成** ⭐⭐⭐⭐⭐
   - 原生 GPUI 组件
   - 直接响应事件
   - 无需额外转换

3. **交互** ⭐⭐⭐⭐⭐
   - 精确的鼠标事件
   - 流畅的缩放平移
   - 实时悬停反馈

4. **维护** ⭐⭐⭐⭐⭐
   - 无额外依赖
   - 代码更简洁
   - 更易调试

## 🚀 实现步骤

1. **创建基础组件**
   - `chart_canvas.rs` - 图表画布
   - `signal_selector.rs` - 信号选择器

2. **实现绘制函数**
   - `paint_grid()` - 网格
   - `paint_axes()` - 坐标轴
   - `paint_line()` - 折线
   - `paint_legend()` - 图例

3. **添加交互**
   - 缩放
   - 平移
   - 悬停

4. **集成到主应用**
   - 添加 Chart 视图
   - 连接数据源

---

**方案**: GPUI 原生绘图 API  
**状态**: ✅ 推荐  
**优先级**: P0
