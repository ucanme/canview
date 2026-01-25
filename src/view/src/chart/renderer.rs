use gpui::*;
use super::data::SignalSeries;

pub struct ChartRenderer {
    series: Vec<SignalSeries>,
}

impl ChartRenderer {
    pub fn new(series: Vec<SignalSeries>) -> Self {
        Self { series }
    }

    pub fn render(self) -> impl IntoElement {
        let series = self.series.clone();
        
        canvas(
            move |_bounds, _window, _cx| series.clone(),
            |bounds, series, window, cx| {
                // Inline helper for grid
                let mut draw_grid = |bounds: Bounds<Pixels>, window: &mut Window, h_lines: usize, v_lines: usize| {
                    let x = bounds.origin.x;
                    let y = bounds.origin.y;
                    let w = bounds.size.width;
                    let h = bounds.size.height;
                    
                    let grid_color = rgb(0x2a2a2a);
                    let mut grid_path = PathBuilder::stroke(px(1.0));

                    // Horizontal lines
                    for i in 1..h_lines {
                        let y_pos = y + h * (i as f32 / h_lines as f32);
                        grid_path.move_to(point(x, y_pos));
                        grid_path.line_to(point(x + w, y_pos));
                    }

                    // Vertical lines
                    for i in 1..v_lines {
                        let x_pos = x + w * (i as f32 / v_lines as f32);
                        grid_path.move_to(point(x_pos, y));
                        grid_path.line_to(point(x_pos, y + h));
                    }
                    
                    window.paint_path(grid_path.build().unwrap(), grid_color);
                };

                // 1. Paint background
                let x = bounds.origin.x;
                let y = bounds.origin.y;
                let w = bounds.size.width;
                let h = bounds.size.height;
                
                let mut bg_path = PathBuilder::fill();
                bg_path.move_to(point(x, y));
                bg_path.line_to(point(x + w, y));
                bg_path.line_to(point(x + w, y + h));
                bg_path.line_to(point(x, y + h));
                bg_path.line_to(point(x, y));
                window.paint_path(bg_path.build().unwrap(), rgb(0x101010));

                // Paint border
                let mut border_path = PathBuilder::stroke(px(1.0));
                border_path.move_to(point(x, y));
                border_path.line_to(point(x + w, y));
                border_path.line_to(point(x + w, y + h));
                border_path.line_to(point(x, y + h));
                border_path.line_to(point(x, y));
                window.paint_path(border_path.build().unwrap(), rgb(0x454545));

                // Check for Empty Data
                if series.is_empty() || series.iter().all(|s| s.points.is_empty()) {
                    draw_grid(bounds, window, 10, 5);
                    return;
                }

                // 2. Calculate Ranges
                let mut min_time = f64::MAX;
                let mut max_time = f64::MIN;
                let mut min_val = f64::MAX;
                let mut max_val = f64::MIN;

                for s in &series {
                    if !s.visible { continue; }
                    for p in &s.points {
                        if p.timestamp < min_time { min_time = p.timestamp; }
                        if p.timestamp > max_time { max_time = p.timestamp; }
                        if p.value < min_val { min_val = p.value; }
                        if p.value > max_val { max_val = p.value; }
                    }
                }

                // Default Ranges
                if min_time == f64::MAX { min_time = 0.0; max_time = 10.0; }
                if min_val == f64::MAX { min_val = 0.0; max_val = 100.0; }
                
                // Avoid zero range
                if (max_time - min_time).abs() < 1e-6 { max_time += 1.0; }
                if (max_val - min_val).abs() < 1e-6 { max_val += 1.0; }

                // Padding (10%)
                let val_range = max_val - min_val;
                let padded_min_val = min_val - val_range * 0.1;
                let padded_max_val = max_val + val_range * 0.1;
                let effective_val_range = padded_max_val - padded_min_val;
                let time_range = max_time - min_time;

                // 3. Draw Grid
                draw_grid(bounds, window, 10, 5);

                // 4. Draw Lines
                for s in &series {
                    if !s.visible || s.points.is_empty() { continue; }

                    let mut path = PathBuilder::stroke(px(2.0));
                    let mut first = true;

                    for p in &s.points {
                        let x_ratio = (p.timestamp - min_time) / time_range;
                        // y axis usually grows down, so invert for graph
                        let y_ratio = (p.value - padded_min_val) / effective_val_range;

                        // Visibility Clip
                        if x_ratio < -0.1 || x_ratio > 1.1 { continue; }

                        let x_px = x + w * x_ratio as f32;
                        let y_px = y + h - (h * y_ratio as f32); // Invert Y

                        if first {
                            path.move_to(point(x_px, y_px));
                            first = false;
                        } else {
                            path.line_to(point(x_px, y_px));
                        }
                    }

                    window.paint_path(path.build().unwrap(), s.color);
                }
            }
        )
    }
}
