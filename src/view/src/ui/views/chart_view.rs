use crate::app::CanViewApp;
use gpui_component::input::{Input, InputState};
use crate::models::{DataPoint, Series};
use blf::LogObject;
use gpui::prelude::*;
use gpui::*;
use gpui_component::chart::LineChart;
use gpui_component::v_flex;
use gpui_component::scroll::ScrollableElement;
use std::sync::Arc;
use chrono::{Timelike, Datelike};

use super::plot_sidebar::{render_sidebar_item, SidebarItem};

/// Render the plot view with signal charts
pub fn render_plot_view(window: &mut Window, app: &mut CanViewApp, view: Entity<CanViewApp>, cx: &mut Context<CanViewApp>) -> impl IntoElement {
    // Safety check: prevent crash from invalid plot state after window operations
    let series_data = app.plot_data.clone();

    // Check if plot data size is reasonable
    if series_data.len() > 100 {
        eprintln!("Warning: Excessive plot data ({} series). This may indicate a state corruption issue.", series_data.len());
        return div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .bg(rgb(0x0f0f10))
            .child(div().px_4().py_2().text_sm().text_color(rgb(0xff0000))
                .child(format!("Error: Too many plot series ({}). Please reload the application.", series_data.len())));
    }

    let has_data = !series_data.is_empty();

    div()
        .size_full()
        .flex()
        .flex_row() // Side-by-side: Signals on left, Charts on right
        .bg(rgb(0x0f0f10))
        .child(
            // Left Sidebar: Signal Selection
            div()
                .w(px(320.0))
                .h_full()
                .border_r_1()
                .border_color(rgb(0x27272a))
                .flex()
                .flex_col()
                .child(super::plot_sidebar::render_signal_sidebar(window, app, view.clone(), cx))
        )
        .child(
            // Right Main Area: Charts
            div()
                .flex_1()
                .h_full()
                .flex()
                .flex_col()
                .child(render_toolbar(app, cx))
                .child(
                    div()
                        .flex_1()
                        .p_4()
                        .overflow_y_scrollbar()
                        .child(
                            if !has_data {
                                render_empty_state(app)
                            } else {
                                render_chart_canvas(app, series_data, cx)
                            }
                        )
                )
        )
}

/// Render the toolbar at the top of the plot area
fn render_toolbar(app: &CanViewApp, cx: &mut Context<CanViewApp>) -> impl IntoElement {
    let is_zoomed = app.plot_zoom_start.is_some() || app.plot_zoom_end.is_some();

    div()
        .flex()
        .items_center()
        .justify_between()
        .px_4()
        .py_1()
        .bg(rgb(0x18181b))
        .border_b_1()
        .border_color(rgb(0x27272a))
        .child(div().flex_1())
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .when(is_zoomed, |this| {
                    this.child(
                        div()
                            .px_2()
                            .py_1()
                            .bg(rgb(0x4b5563))
                            .rounded(px(4.0))
                            .cursor_pointer()
                            .hover(|s| s.bg(rgb(0x374151)))
                            .on_mouse_down(gpui::MouseButton::Left, cx.listener(|this: &mut CanViewApp, _, _, cx| {
                                this.plot_zoom_start = None;
                                this.plot_zoom_end = None;
                                crate::ui::views::chart_view::extract_and_update_series_data(this);
                                cx.notify();
                            }))
                            .child(div().text_xs().text_color(rgb(0xffffff)).child("Reset Zoom"))
                    )
                })
                .child(
                    div()
                        .px_2()
                        .py_1()
                        .bg(if app.show_plot_points { rgb(0x10b981) } else { rgb(0x6b7280) })
                        .rounded(px(4.0))
                        .cursor_pointer()
                        .hover(|s| s.bg(if app.show_plot_points { rgb(0x059669) } else { rgb(0x4b5563) }))
                        .on_mouse_down(gpui::MouseButton::Left, cx.listener(|this: &mut CanViewApp, _, _, cx| {
                            this.show_plot_points = !this.show_plot_points;
                            cx.notify();
                        }))
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(0xffffff))
                                .child(if app.show_plot_points { "Points: ON" } else { "Points: OFF" })
                        )
                )
        )
}


/// Render empty state when no data is available
fn render_empty_state(app: &CanViewApp) -> AnyElement {
    let msg_count = app.messages.len();
    let sel_count = app.selected_signals.len();

    div()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .size_full()
        .gap_3()
        .child(
            div()
                .text_color(rgb(0x71717a))
                .text_lg()
                .child("尚无数据显示")
        )
        .child(
            div()
                .text_color(rgb(0x52525b))
                .text_sm()
                .child("请在信号选择(Signals)中选择信号，点击Plot按钮加载数据")
        )
        .child(
            div()
                .text_color(rgb(0xef4444))
                .text_xs()
                .child(format!("Debug: Messages={}, Selected={}", msg_count, sel_count))
        )
        .into_any_element()
}

/// Render the chart canvas using gpui-component LineChart
fn render_chart_canvas(app: &CanViewApp, series_data: Arc<[Series]>, cx: &mut Context<CanViewApp>) -> AnyElement {
    let is_dragging = app.is_dragging_zoom;
    let drag_start = app.zoom_drag_start_x;
    let drag_current = app.zoom_drag_current_x;
    let start_time = app.start_time;
    let show_points = app.show_plot_points;
    let hover_x = app.plot_hover_x;
    let hover_time = app.plot_hover_time;

    // Constant offsets based on layout (Sidebar: 320px, Padding: 16px)
    let sidebar_offset = px(320.0); 
    let padding = px(16.0);
    let chart_start_x = sidebar_offset + padding;

    div()
        .size_full()
        .relative()
        .child(
            v_flex()
                .size_full()
                .gap_4()
                .child(render_legend(&series_data))
                .children(series_data.iter().map(|series| {
                    render_single_chart(series, start_time, show_points)
                }))
        )
        // Zoom Box Overlay Layer
        .when(is_dragging, |this| {
            if let (Some(start), Some(current)) = (drag_start, drag_current) {
                // Convert global mouse coordinates to local container coordinates
                let left_global = start.min(current);
                let right_global = start.max(current);
                
                let left_local = (left_global - chart_start_x).max(px(0.0));
                let width = right_global - left_global; // Width is the delta
                
                this.child(
                    div()
                        .absolute()
                        .top_0()
                        .left(left_local)
                        .w(width)
                        .h_full()
                        .bg(rgba(0x3b82f633)) // Light blue with transparency
                        .border_x_1()
                        .border_color(rgb(0x3b82f6))
                )
            } else {
                this
            }
        })
        // Hover Line and Tooltip
        .when_some(hover_x.zip(hover_time), |this, (hx, ht)| {
             // Calculate local position relative to canvas
             let local_x = hx - chart_start_x;
             let chart_width = app.plot_width_px;
             
             this.child(
                 div()
                     .absolute()
                     .top_0()
                     .left(local_x)
                     .w(px(1.0))
                     .h_full()
                     .bg(rgb(0xd4d4d8)) // Light gray line
                     .opacity(0.8)
             )
             .child(
                 render_hover_tooltip(hx, ht, &series_data, chart_start_x, chart_width, app.start_time)
             )
        })
        // Global Canvas Interactions
        .on_mouse_down(MouseButton::Left, cx.listener(move |this: &mut CanViewApp, event: &MouseDownEvent, _, cx| {
             // Check for double-click to reset zoom
             if event.click_count == 2 {
                 this.plot_zoom_start = None;
                 this.plot_zoom_end = None;
                 crate::ui::views::chart_view::extract_and_update_series_data(this);
                 eprintln!("🔄 Double-click detected - resetting zoom");
                 cx.notify();
                 return;
             }
             
             this.is_dragging_zoom = true;
             this.zoom_drag_start_x = Some(event.position.x);
             this.zoom_drag_current_x = Some(event.position.x);
             cx.notify();
        }))
        .on_mouse_move(cx.listener(move |this: &mut CanViewApp, event: &MouseMoveEvent, window, cx| {
            // Handle Zoom Dragging
            if this.is_dragging_zoom {
                this.zoom_drag_current_x = Some(event.position.x);
                cx.notify();
            }

            // Handle Hover
            // We need to calculate time based on mouse position
            let mouse_x = event.position.x;
            
            // Throttle: only notify if mouse moved more than 1px to reduce render load
            let prev_hover_x = this.plot_hover_x.unwrap_or(px(-9999.0));
            let moved_enough = (mouse_x - prev_hover_x).abs() > px(1.0);
            
            // Calculate visible width
            let window_width = window.bounds().size.width;
            let chart_width_px = window_width - chart_start_x - padding;
            this.plot_width_px = chart_width_px;
            
            // Bounds check
            if mouse_x >= chart_start_x && mouse_x <= (window_width - padding) {
                if moved_enough || this.is_dragging_zoom {
                    this.plot_hover_x = Some(mouse_x);
                    
                    // Determine time range
                    let (min_t, max_t) = if let (Some(s), Some(e)) = (this.plot_zoom_start, this.plot_zoom_end) {
                        (s, e)
                    } else {
                        let mut min_t = f64::MAX;
                        let mut max_t = f64::MIN;
                        for series in this.plot_data.iter() {
                            for p in series.points.iter() {
                                if p.time < min_t { min_t = p.time; }
                                if p.time > max_t { max_t = p.time; }
                            }
                        }
                        if min_t == f64::MAX { (0.0, 1.0) } else { (min_t, max_t) }
                    };
                    
                    // Interpolate
                    let rel_x = (mouse_x - chart_start_x) / chart_width_px;
                    let rel_x = f32::max(0.0, f32::min(1.0, rel_x)); // Clamp
                    
                    let time_range = max_t - min_t;
                    let time = min_t + time_range * rel_x as f64;
                    
                    this.plot_hover_time = Some(time);
                    cx.notify();
                }
            } else if this.plot_hover_x.is_some() {
                 // Clear hover if moved out (approximate)
                 this.plot_hover_x = None;
                 this.plot_hover_time = None;
                 cx.notify();
            }
        }))
        .on_mouse_up(MouseButton::Left, cx.listener(move |this: &mut CanViewApp, event: &MouseUpEvent, window, cx| {
            if !this.is_dragging_zoom { return; }
            
            if let Some(start_x) = this.zoom_drag_start_x {
                let end_x = event.position.x;
                let (x1, x2) = (start_x.min(end_x), start_x.max(end_x));
                
                // Only zoom if the selection is significant (> 10 pixels)
                if (x1 - x2).abs() > px(10.0) {
                    // Use stored full data time range for accurate mapping
                    let (min_t, max_t) = match (this.plot_full_time_min, this.plot_full_time_max) {
                        (Some(mn), Some(mx)) if mn < mx => (mn, mx),
                        _ => {
                            let mut min_t = f64::MAX;
                            let mut max_t = f64::MIN;
                            for series in this.plot_full_data.iter() {
                                for p in series.points.iter() {
                                    if p.time < min_t { min_t = p.time; }
                                    if p.time > max_t { max_t = p.time; }
                                }
                            }
                            if min_t == f64::MAX { (0.0, 1.0) } else { (min_t, max_t) }
                        }
                    };
                    
                    // Get current zoomed range for coordinate mapping
                    let (cur_min, cur_max) = if let (Some(s), Some(e)) = (this.plot_zoom_start, this.plot_zoom_end) {
                        (s, e)
                    } else {
                        (min_t, max_t)
                    };
                    
                    if cur_min < cur_max {
                        let window_width = window.bounds().size.width;
                        let plot_width = f32::from(window_width - chart_start_x - padding); 
                        
                        // Map relative x to time within current visible range
                        let start_rel = f32::from(x1 - chart_start_x) / plot_width; 
                        let end_rel = f32::from(x2 - chart_start_x) / plot_width;
                        
                        let cur_range = cur_max - cur_min;
                        let new_start = cur_min + cur_range * start_rel as f64;
                        let new_end = cur_min + cur_range * end_rel as f64;
                        
                        this.plot_zoom_start = Some(new_start.max(min_t));
                        this.plot_zoom_end = Some(new_end.min(max_t));
                        // Fast filter: just slice plot_full_data, no message re-scan
                        crate::ui::views::chart_view::apply_zoom_to_full_data(this);
                    }
                }
            }
            
            this.is_dragging_zoom = false;
            this.zoom_drag_start_x = None;
            this.zoom_drag_current_x = None;
            cx.notify();
        }))
        // Mouse wheel zoom
        .on_scroll_wheel(cx.listener(move |this: &mut CanViewApp, event: &ScrollWheelEvent, window, cx| {
            // Stop event propagation to prevent parent container from scrolling
            cx.stop_propagation();

            // Prevent zooming if no data
            if this.plot_data.is_empty() {
                return;
            }

            // Let's remove unused chart_width_px
            // Calculate chart dimensions
            let window_width = window.bounds().size.width;

            // Check if mouse is over the chart area
            let mouse_x = event.position.x;
            if mouse_x < chart_start_x || mouse_x > (window_width - padding) {
                return;
            }

            // Use stored full data time range (not from filtered plot_data!)
            // plot_data is filtered by zoom range, so computing min/max from it
            // would give us the zoomed range, not the full data range.
            let (abs_min_t, abs_max_t) = match (this.plot_full_time_min, this.plot_full_time_max) {
                (Some(min_t), Some(max_t)) if min_t < max_t => (min_t, max_t),
                _ => {
                    // Fallback: compute from plot_data (only correct when not zoomed)
                    let mut min_t = f64::MAX;
                    let mut max_t = f64::MIN;
                    for series in this.plot_data.iter() {
                        for p in series.points.iter() {
                            if p.time < min_t { min_t = p.time; }
                            if p.time > max_t { max_t = p.time; }
                        }
                    }
                    if min_t == f64::MAX { return; }
                    (min_t, max_t)
                }
            };

            // Get current zoomed range (or use full data range if not zoomed)
            let (current_min, current_max) = if let (Some(s), Some(e)) = (this.plot_zoom_start, this.plot_zoom_end) {
                (s, e)
            } else {
                (abs_min_t, abs_max_t)
            };

            let current_range = current_max - current_min;
            if current_range <= 0.0 {
                return;
            }

            // Determine zoom direction based on scroll delta
            let scroll_delta = match event.delta {
                gpui::ScrollDelta::Lines(point) => point.y,
                gpui::ScrollDelta::Pixels(pixels) => f32::from(pixels.y),
            };

            eprintln!("🖱️  scroll_delta = {:.4}", scroll_delta);

            // Lower threshold for better responsiveness
            if scroll_delta.abs() < 0.01 {
                return;
            }

            // Try reversed direction:
            // Scroll backward (toward user) = positive delta = zoom IN
            // Scroll forward (away from user) = negative delta = zoom OUT
            let zoom_in = scroll_delta > 0.0;

            let zoom_factor = 1.2;

            let (new_min, new_max) = if zoom_in {
                // Zoom in: reduce range
                let new_range = current_range / zoom_factor;

                // Minimum range = the smallest interval that still contains at
                // least one data point. Use a fraction of the full data range,
                // but never smaller than the time between two adjacent points
                // (approximated by abs_range / total_point_count).
                let abs_range = abs_max_t - abs_min_t;
                let total_points: usize = this.plot_full_data.iter().map(|s| s.points.len()).sum();
                let min_range = if total_points > 1 {
                    // smallest gap between any two adjacent points across all series
                    let mut smallest_gap = f64::MAX;
                    for series in this.plot_full_data.iter() {
                        let mut prev: Option<f64> = None;
                        for p in series.points.iter() {
                            if let Some(prev_t) = prev {
                                let gap = p.time - prev_t;
                                if gap > 0.0 && gap < smallest_gap {
                                    smallest_gap = gap;
                                }
                            }
                            prev = Some(p.time);
                        }
                    }
                    if smallest_gap == f64::MAX {
                        // all points at the same time → use 1/1000 of range
                        (abs_range / 1000.0).max(1e-6)
                    } else {
                        // allow zooming in to half the smallest gap, so at
                        // least one point remains visible
                        (smallest_gap * 0.5).max(1e-6)
                    }
                } else {
                    // less than 2 points total → no zoom-in needed
                    (abs_range / 1000.0).max(1e-6)
                };

                if new_range < min_range {
                    eprintln!("⚠️  Cannot zoom in further: minimum range ({:.6}s) reached", min_range);
                    return;
                }

                // Zoom centered on the mouse position, not the chart center,
                // so the data under the cursor stays put.
                let chart_width_px = window_width - padding - chart_start_x;
                let mouse_t_ratio = if current_max > current_min && chart_width_px > gpui::px(0.0) {
                    let r = (mouse_x - chart_start_x) / chart_width_px;
                    f32::max(0.0, f32::min(1.0, r)) as f64
                } else {
                    0.5
                };
                let focus_t = current_min + (current_max - current_min) * mouse_t_ratio;
                let new_min = focus_t - new_range * mouse_t_ratio;
                let new_max = focus_t + new_range * (1.0 - mouse_t_ratio);
                (new_min, new_max)
            } else {
                // Zoom out: expand toward full data range, centered on mouse
                let new_range = current_range * zoom_factor;

                let chart_width_px = window_width - padding - chart_start_x;
                let mouse_t_ratio = if current_max > current_min && chart_width_px > gpui::px(0.0) {
                    let r = (mouse_x - chart_start_x) / chart_width_px;
                    f32::max(0.0, f32::min(1.0, r)) as f64
                } else {
                    0.5
                };
                let focus_t = current_min + (current_max - current_min) * mouse_t_ratio;
                let mut new_min = focus_t - new_range * mouse_t_ratio;
                let mut new_max = focus_t + new_range * (1.0 - mouse_t_ratio);

                // Clamp to data boundaries
                new_min = new_min.max(abs_min_t);
                new_max = new_max.min(abs_max_t);

                (new_min, new_max)
            };

            let new_range = new_max - new_min;

            // Check if we've reached or exceeded the full range
            let abs_range = abs_max_t - abs_min_t;
            // Use a more lenient threshold (0.95 instead of 0.999) to make it easier to reset
            let is_at_or_near_full_range = new_range >= abs_range * 0.95;

            if is_at_or_near_full_range {
                // Reset to show full data
                this.plot_zoom_start = None;
                this.plot_zoom_end = None;
                eprintln!("🔍 Zoom OUT - reset to full range ({:.3}s)", abs_range);
            } else {
                this.plot_zoom_start = Some(new_min);
                this.plot_zoom_end = Some(new_max);
                eprintln!("🔍 Zoom {}: {:.3}s -> {:.3}s (span: {:.3}s)",
                    if zoom_in { "IN" } else { "OUT" },
                    new_min, new_max, new_range);
            }

            // Fast filter: slice plot_full_data without re-decoding messages
            crate::ui::views::chart_view::apply_zoom_to_full_data(this);
            cx.notify();
        }))
        .into_any_element()
}

/// Helper to format time as relative (seconds) or absolute (based on file start time)
fn format_time_relative_or_absolute(time: f64, start_time: Option<chrono::NaiveDateTime>) -> String {
    if let Some(st) = start_time {
        use chrono::{Timelike, Datelike};
        // Convert start_time to total seconds since midnight
        let start_hour = st.hour() as f64;
        let start_min = st.minute() as f64;
        let start_sec = st.second() as f64;
        let start_nano = st.nanosecond() as f64;
        let start_total_seconds = start_hour * 3600.0 + start_min * 60.0 + start_sec + start_nano / 1_000_000_000.0;
        
        // Calculate absolute time for this point
        let abs_seconds = start_total_seconds + time;
        
        // Handle day overflow (wrap at 24 hours) - purely for display
        let display_seconds = abs_seconds % 86400.0;
        
        let hours = (display_seconds / 3600.0).floor() as u32;
        let remaining = display_seconds % 3600.0;
        let minutes = (remaining / 60.0).floor() as u32;
        let seconds = remaining % 60.0;
        
        format!("{:04}-{:02}-{:02} {:02}:{:02}:{:06.3}", 
            st.year(), st.month(), st.day(), 
            hours, minutes, seconds)
    } else {
        format!("Time: {:.3}s", time)
    }
}

/// Render tooltip for hover over chart
fn render_hover_tooltip(
    hover_x: Pixels, 
    hover_time: f64, 
    series_data: &[Series],
    offset_x: Pixels,
    chart_width: Pixels,
    start_time: Option<chrono::NaiveDateTime>
) -> impl IntoElement {
    // Collect values near this time
    let mut values = Vec::new();
    for series in series_data {
        // Simple linear search or binary search
        if series.points.is_empty() { continue; }
        
        // Find closest point
        // Optimize: verify logic using partition_point for sorted data
        let idx = series.points.partition_point(|p| p.time < hover_time);
        
        let p_after = series.points.get(idx);
        let p_before = if idx > 0 { series.points.get(idx - 1) } else { None };
        
        // Find closest
        let val = match (p_before, p_after) {
            (Some(b), Some(a)) => {
                let diff_b = (hover_time - b.time).abs();
                let diff_a = (a.time - hover_time).abs();
                if diff_b < diff_a { b.value } else { a.value }
            },
            (Some(b), None) => b.value,
            (None, Some(a)) => a.value,
            (None, None) => 0.0,
        };
        
        values.push((series.clone(), val));
    }

    // Determine tooltip position
    // local coordinates
    let local_x = hover_x - offset_x;
    
    // Check if we need to flip to left
    let tooltip_width_estimate = px(220.0); // Rough estimate
    let space_right = chart_width - local_x;
    
    let (tooltip_left, _align_right) = if space_right < tooltip_width_estimate {
        // Not enough space, place to left of cursor
        (local_x - tooltip_width_estimate - px(10.0), true)
    } else {
        // Place to right
        (local_x + px(10.0), false)
    };
    
    div()
        .absolute()
        .top(px(20.0))
        .left(tooltip_left)
        .bg(rgba(0x18181be6)) // Dark bg with transparency
        .border_1()
        .border_color(rgb(0x3f3f46))
        .rounded_md()
        .shadow_md()
        .p_3()
        .flex()
        .flex_col()
        .gap_2()
        .child(
            div()
                .text_xs()
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(0xe4e4e7))
                .child(format_time_relative_or_absolute(hover_time, start_time))
        )
        .children(values.into_iter().map(|(series, val)| {
             div()
                 .flex()
                 .items_center()
                 .gap_2()
                 .child(
                     div()
                         .size(px(8.0))
                         .rounded_full()
                         .bg(series.color)
                 )
                 .child(
                     div()
                         .text_xs()
                         .text_color(rgb(0xd4d4d8))
                         .child(format!("{}: {:.2} {}", series.name, val, series.unit.as_deref().unwrap_or("")))
                 )
        }))
}

/// Render a single chart for one signal
fn render_single_chart(
    series: &Series, 
    _start_time: Option<chrono::NaiveDateTime>, 
    show_points: bool
) -> impl IntoElement {
    // Safety check: ensure we have points
    if series.points.is_empty() {
        return div()
            .flex()
            .flex_col()
            .h(px(250.0))
            .bg(rgb(0x18181b))
            .border_1()
            .border_color(rgb(0x27272a))
            .rounded_lg()
            .p_4()
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(0xa1a1aa))
                    .child(format!("No data points for '{}'. Check Channel ID match (DBC vs Log) or Time Range.", series.name))
            );
    }

    // Clone time labels for use in closure
    let time_labels = series.time_labels.clone();

    // Calculate time range for debug display
    let min_time = series.points.iter().map(|p| p.time).fold(f64::INFINITY, f64::min);
    let max_time = series.points.iter().map(|p| p.time).fold(f64::NEG_INFINITY, f64::max);
    let time_span = max_time - min_time;

    // Calculate optimal step for showing labels (aim for ~4 labels)
    let total_points = series.points.len();
    let label_step = if total_points <= 10 {
        1
    } else {
        total_points / 4
    };

    div()
        .flex()
        .flex_col()
        .h(px(250.0))
        .bg(rgb(0x18181b))
        .border_1()
        .border_color(rgb(0x27272a))
        .rounded_lg()
        .p_4()
        .child(
            div()
                .text_sm()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(series.color)
                .child(format!(
                    "{} {} | {} pts | {:.3}s-{:.3}s (span: {:.3}s)", 
                    series.name, 
                    series.unit.as_ref().map(|u| format!("[{}]", u)).unwrap_or_default(),
                    series.points.len(),
                    min_time,
                    max_time,
                    time_span
                ))
        )
        .child(
            div()
                .flex_1()
                .py_2()
                .child({
                    let mut chart = LineChart::<DataPoint, SharedString, f64>::new(series.points.clone())
                        .x(move |d| {
                            // Hack: Use zero-width characters to make each time string unique
                            // while keeping it visually identical for labels.
                            let mut unique_suffix = String::new();
                            let mut val = d.index;
                            // ERROR FIX: Increased from 10 to 20 bits. 
                            // 10 bits only supported 1024 points, causing collisions for 5753 points.
                            // 20 bits supports ~1 million points (2^20), insuring uniqueness.
                            for _ in 0..20 { 
                                unique_suffix.push(if val % 2 == 0 { '\u{200B}' } else { '\u{200C}' });
                                val /= 2;
                            }
                            
                            // Determine if we should show the label
                            let should_show = d.index == 0 || // Always show first
                                              d.index == total_points - 1 || // Always show last
                                              d.index % label_step == 0; // Show periodic
                            
                            let label_text = if should_show {
                                time_labels.get(d.index).map(|s| s.as_str()).unwrap_or("N/A")
                            } else {
                                "" // Empty string for hidden labels
                            };

                            format!("{}{}", label_text, unique_suffix).into()
                        })
                        .y(|d| d.value)
                        .stroke(series.color)
                        .linear()
                        .tick_margin(1); // Always "try" to draw every tick, but most will be empty strings
                    
                    // Add dots on data points if enabled
                    if show_points {
                        chart = chart.dot();
                    }
                    
                    chart
                })
        )
}


/// Render legend showing all series
fn render_legend(series_data: &[Series]) -> impl IntoElement {
    div()
        .flex()
        .flex_wrap()
        .gap_4()
        .p_3()
        .bg(rgb(0x18181b))
        .border_1()
        .border_color(rgb(0x27272a))
        .rounded_lg()
        .children(series_data.iter().map(|series| {
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .w(px(16.0))
                        .h(px(3.0))
                        .rounded_sm()
                        .bg(series.color)
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(0xa1a1aa))
                        .child(series.name.clone())
                )
        }))
}

/// Filter existing plot data by zoom range
/// This is used when messages is empty but plot_data has already been extracted
fn filter_series_data_by_zoom(app: &CanViewApp) -> Arc<[Series]> {
    eprintln!("🔍 Filtering existing plot data by zoom range...");

    let has_zoom = app.plot_zoom_start.is_some() || app.plot_zoom_end.is_some();

    if !has_zoom {
        // No zoom filter, return original data
        eprintln!("📊 No zoom filter, returning original plot_data ({} series)", app.plot_data.len());
        return app.plot_data.clone();
    }

    // Apply zoom filter to existing plot_data
    let filtered_series: Vec<Series> = app.plot_data.iter().map(|series| {
        let filtered_points: Vec<DataPoint> = series.points.iter()
            .filter(|p| {
                if let Some(start) = app.plot_zoom_start {
                    if p.time < start { return false; }
                }
                if let Some(end) = app.plot_zoom_end {
                    if p.time > end { return false; }
                }
                true
            })
            .map(|p| *p)
            .collect();

        eprintln!("  📈 Filtered '{}' from {} to {} points",
            series.name, series.points.len(), filtered_points.len());

        Series {
            name: series.name.clone(),
            color: series.color,
            unit: series.unit.clone(),
            points: filtered_points,
            time_labels: Vec::new(), // Will be regenerated by renderer
        }
    }).collect();

    eprintln!("✅ Filtering complete: {} series", filtered_series.len());
    Arc::from(filtered_series)
}

/// Extract series data from application state - SAFE VERSION
pub fn extract_series_data(app: &CanViewApp) -> Arc<[Series]> {
    eprintln!("🔍 Starting data extraction...");

    // If messages is empty but we have plot_data, filter the existing plot_data
    // This handles the case after window maximize/restore where messages is cleared
    if app.messages.is_empty() && !app.plot_data.is_empty() {
        eprintln!("📊 Messages empty, using existing plot_data ({} series)", app.plot_data.len());
        return filter_series_data_by_zoom(app);
    }

    // Limit processing to avoid stack overflow, but high enough for complete logs
    const MAX_SIGNALS: usize = 20;
    const MAX_MESSAGES: usize = 10_000_000;
    const MAX_RAW_POINTS: usize = 1_000_000; // Max points: 1M (needed for 10min @ 1ms = 600K points)
    const SAMPLING_INTERVAL_MS: f64 = 1.0; // 1ms sampling interval

    let signal_count = app.selected_signals.len().min(MAX_SIGNALS);
    let message_count = app.messages.len().min(MAX_MESSAGES);

    eprintln!("📊 Processing {} signals from {} messages", signal_count, message_count);

    if signal_count == 0 || message_count == 0 {
        eprintln!("⚠️  No data to process");
        return Arc::from([]);
    }

    let mut all_series: Vec<Series> = Vec::with_capacity(signal_count);
    
    let colors = [
        hsla(217.0 / 360.0, 0.91, 0.6, 1.0),
        hsla(142.0 / 360.0, 0.71, 0.45, 1.0),
        hsla(35.0 / 360.0, 0.92, 0.5, 1.0),
        hsla(280.0 / 360.0, 0.61, 0.5, 1.0),
        hsla(0.0 / 360.0, 0.84, 0.6, 1.0),
        hsla(180.0 / 360.0, 0.71, 0.5, 1.0),
    ];

    // Process each signal
    for (idx, sig_id) in app.selected_signals.iter().take(signal_count).enumerate() {
        eprintln!("  📡 Signal {}: {}", idx + 1, sig_id);
        
        // Parse signal ID: BUS:CHANNEL:MSG_ID:SIG_NAME or BUS:MSG_ID:SIG_NAME (compat)
        let parts: Vec<&str> = sig_id.split(':').collect();
        if parts.len() < 3 {
            eprintln!("    ❌ Skip: invalid format");
            continue;
        }

        let (bus_type, channel_filter, msg_id_str, sig_name) = if parts.len() >= 4 {
            let bus_type = parts[0];
            let channel_id = parts[1];
            let msg_id = parts[2];
            let sig_name = parts[3..].join(":");
            (bus_type, Some(channel_id.to_string()), msg_id.to_string(), sig_name)
        } else {
            (parts[0], None, parts[1].to_string(), parts[2].to_string())
        };

        let msg_id_res = if msg_id_str.starts_with("0x") {
            u32::from_str_radix(&msg_id_str[2..], 16)
        } else {
            msg_id_str.parse::<u32>()
        };
        
        let msg_id = match msg_id_res {
            Ok(id) => id,
            Err(_) => {
                eprintln!("    ❌ Skip: invalid message ID");
                continue;
            }
        };

        let target_channel = channel_filter.and_then(|s| s.parse::<u16>().ok());
        let mut unit = None;
        let mut points = Vec::new();
        let mut collected = 0;

        // Single pass: collect points with early termination if too many
        for msg in app.messages.iter().take(message_count) {
            // Early termination to prevent memory issues
            if collected >= MAX_RAW_POINTS {
                eprintln!("    ⚠️  Reached max raw points limit (1M), stopping collection");
                break;
            }

            if bus_type != "CAN" && bus_type != "LIN" {
                continue;
            }

            let (m_id, ch, timestamp, data, flags): (u32, u16, u64, &[u8], u32) = match msg {
                LogObject::CanMessage(m) => (m.id, m.channel, m.header.object_time_stamp, &m.data, m.header.object_flags),
                LogObject::CanMessage2(m) => (m.id, m.channel, m.header.object_time_stamp, &m.data, m.header.object_flags),
                LogObject::CanFdMessage(m) => (m.id, m.channel, m.header.object_time_stamp, &m.data, m.header.object_flags),
                LogObject::CanFdMessage64(m) => (m.id, m.channel as u16, m.header.object_time_stamp, &m.data, m.header.object_flags),
                LogObject::LinMessage(m) => (m.id as u32, m.channel, m.header.object_time_stamp, &m.data, m.header.object_flags),
                LogObject::LinMessage2(m) => (0u32, 0u16, m.header.object_time_stamp, &m.data, m.header.object_flags),
                _ => continue,
            };

            if m_id != msg_id {
                continue;
            }

            // Check channel filter
            if let Some(ch_filter) = target_channel {
                if ch != ch_filter {
                    continue;
                }
            }

            // Convert timestamp to seconds based on object_flags
            // TimeTenMics (0x01) = 10 microseconds per tick
            // TimeOneNans (0x02) = 1 nanosecond per tick
            let time = if flags & 0x01 != 0 {
                // 10 microseconds per tick
                if collected == 0 {
                    eprintln!("    🕐 Using 10 microsecond timestamp (flags: 0x{:08X})", flags);
                }
                timestamp as f64 / 100_000.0
            } else {
                // Default to nanoseconds (most common)
                if collected == 0 {
                    eprintln!("    🕐 Using nanosecond timestamp (flags: 0x{:08X})", flags);
                }
                timestamp as f64 / 1_000_000_000.0
            };

            
            // Apply zoom filter if active
            if let Some(start) = app.plot_zoom_start {
                if time < start { continue; }
            }
            if let Some(end) = app.plot_zoom_end {
                if time > end { continue; }
            }

            // Try to decode the signal
            if bus_type == "CAN" {
                if let Some(dbc) = app.dbc_channels.get(&ch) {
                    if let Some(dbc_msg) = dbc.messages.get(&m_id) {
                        if let Some(sig) = dbc_msg.signals.get(&sig_name) {
                            if unit.is_none() && !sig.unit.is_empty() {
                                unit = Some(sig.unit.clone());
                            }
                            let val = sig.decode(data);
                            points.push(DataPoint { time, value: val, index: 0 });
                            collected += 1;
                        }
                    }
                }
            } else if bus_type == "LIN" {
                if let Some(ldf) = app.ldf_channels.get(&ch) {
                    if let Some(frame) = ldf.frames.get(&m_id.to_string()) {
                        // Find signal in frame mappings
                        if let Some(mapping) = frame.signals.iter().find(|s| s.signal_name == sig_name) {
                            if let Some(sig) = ldf.signals.get(&mapping.signal_name) {
                                if unit.is_none() {
                                    unit = Some("".to_string()); // LIN usually doesn't have units in this parser
                                }
                                let val = sig.decode(data, mapping.offset) as f64;
                                points.push(DataPoint { time, value: val, index: 0 });
                                collected += 1;
                            }
                        }
                    }
                }
            }
        }

        eprintln!("    📈 Collected {} raw points", points.len());

        if points.is_empty() {
            eprintln!("    ⚠️  No points found for this signal");
            continue;
        }

        // Show time range
        let min_time = points.iter().map(|p| p.time).fold(f64::INFINITY, f64::min);
        let max_time = points.iter().map(|p| p.time).fold(f64::NEG_INFINITY, f64::max);
        eprintln!("    ⏱️  Time range: {:.3}s to {:.3}s (span: {:.3}s)", min_time, max_time, max_time - min_time);

        // Apply 1ms sampling if data is too dense
        if points.len() > 1 {
            let time_span = points.last().unwrap().time - points[0].time;
            let avg_interval_ms = (time_span * 1000.0) / (points.len() as f64);
            
            if avg_interval_ms < SAMPLING_INTERVAL_MS {
                eprintln!("    🔽 Applying 1ms sampling (avg interval: {:.3}ms)", avg_interval_ms);
                
                // Group points into 1ms buckets and keep one per bucket
                let mut sampled = Vec::new();
                let start_time = points[0].time;
                let mut current_bucket = 0i64;
                let mut bucket_point: Option<DataPoint> = None;
                
                for point in points.iter() {
                    let time_ms = (point.time - start_time) * 1000.0;
                    let bucket = (time_ms / SAMPLING_INTERVAL_MS).floor() as i64;
                    
                    if bucket != current_bucket {
                        // New bucket, save previous bucket's point
                        if let Some(p) = bucket_point {
                            sampled.push(p);
                        }
                        current_bucket = bucket;
                        bucket_point = Some(*point);
                    } else {
                        // Same bucket, keep the point closest to bucket start
                        if let Some(existing) = bucket_point {
                            let existing_offset = (existing.time - start_time) * 1000.0 - (current_bucket as f64 * SAMPLING_INTERVAL_MS);
                            let new_offset = time_ms - (bucket as f64 * SAMPLING_INTERVAL_MS);
                            if new_offset < existing_offset {
                                bucket_point = Some(*point);
                            }
                        }
                    }
                }
                
                // Don't forget the last bucket
                if let Some(p) = bucket_point {
                    sampled.push(p);
                }
                
                eprintln!("    ✅ Sampled to {} points", sampled.len());
                points = sampled;
            } else {
                eprintln!("    ✅ Data already sparse enough (avg interval: {:.3}ms)", avg_interval_ms);
            }
        }

        // Assign indices for label spacing
        for (i, p) in points.iter_mut().enumerate() {
            p.index = i;
        }

        // Pre-calculate time labels (absolute or relative)
        let mut time_labels = Vec::with_capacity(points.len());
        
        if let Some(start_time) = app.start_time {
            // Calculate precision based on time range
            let mut min_t = points[0].time;
            let mut max_t = points[0].time;
            for p in points.iter().skip(1) {
                if p.time < min_t { min_t = p.time; }
                if p.time > max_t { max_t = p.time; }
            }
            let range = (max_t - min_t).abs();
            let precision = if range < 0.01 { 4 }
                           else if range < 0.1 { 3 }
                           else if range < 1.0 { 2 }
                           else { 1 };
            
            // Convert start_time to total seconds since midnight
            let start_hour = start_time.hour() as f64;
            let start_min = start_time.minute() as f64;
            let start_sec = start_time.second() as f64;
            let start_nano = start_time.nanosecond() as f64;
            let start_total_seconds = start_hour * 3600.0 + start_min * 60.0 + start_sec + start_nano / 1_000_000_000.0;
            
            // Extract date components
            let year = start_time.year();
            let month = start_time.month();
            let day = start_time.day();
            
            // Convert each point to absolute time using pure math
            for point in points.iter() {
                let abs_seconds = start_total_seconds + point.time;
                
                // Handle day overflow (wrap at 24 hours)
                let abs_seconds = abs_seconds % 86400.0;
                
                let hours = (abs_seconds / 3600.0).floor() as u32;
                let remaining = abs_seconds % 3600.0;
                let minutes = (remaining / 60.0).floor() as u32;
                let seconds = remaining % 60.0;
                
                let label = match precision {
                    4 => format!("{:04}-{:02}-{:02} {:02}:{:02}:{:06.4}", year, month, day, hours, minutes, seconds),
                    3 => format!("{:04}-{:02}-{:02} {:02}:{:02}:{:06.3}", year, month, day, hours, minutes, seconds),
                    2 => format!("{:04}-{:02}-{:02} {:02}:{:02}:{:05.2}", year, month, day, hours, minutes, seconds),
                    1 => format!("{:04}-{:02}-{:02} {:02}:{:02}:{:04.1}", year, month, day, hours, minutes, seconds),
                    _ => format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", year, month, day, hours, minutes, seconds.floor() as u32),
                };
                time_labels.push(label);
            }
        } else {
            // Use relative time
            for point in points.iter() {
                time_labels.push(format!("{:.2}s", point.time));
            }
        }

        eprintln!("    ✅ Final point count: {}", points.len());

        all_series.push(Series {
            name: sig_name.to_string(),
            unit,
            points: points.into(),
            color: colors[idx % colors.len()],
            time_labels,
        });
    }

    eprintln!("✅ Extraction complete: {} series generated", all_series.len());
    Arc::from(all_series)
}

/// Fast zoom filter: slice plot_full_data by zoom range without re-decoding messages.
/// This is called every time zoom changes; it's O(points), not O(messages).
pub fn apply_zoom_to_full_data(app: &mut CanViewApp) {
    if app.plot_full_data.is_empty() {
        // plot_full_data hasn't been populated yet.
        // Do NOT call extract_and_update_series_data here — that would cause
        // mutual recursion (extract_and_update_series_data → apply_zoom_to_full_data).
        // Just keep plot_data empty; the caller is responsible for filling
        // plot_full_data first before calling us.
        app.plot_data = std::sync::Arc::from([]);
        return;
    }

    let zoom_start = app.plot_zoom_start;
    let zoom_end = app.plot_zoom_end;

    if zoom_start.is_none() && zoom_end.is_none() {
        // No zoom: display full data directly
        app.plot_data = app.plot_full_data.clone();
        return;
    }

    // Filter each series by zoom range and reassign point indices
    let filtered: Vec<crate::models::Series> = app.plot_full_data.iter().map(|series| {
        // Collect filtered points while remembering each point's original index
        // (original index → lookup in time_labels; new sequential index → label spacing)
        let orig_indexed: Vec<(usize, crate::models::DataPoint)> = series.points.iter()
            .filter(|p| {
                if let Some(s) = zoom_start { if p.time < s { return false; } }
                if let Some(e) = zoom_end   { if p.time > e { return false; } }
                true
            })
            .map(|p| (p.index, *p))  // save original index before we overwrite it
            .collect();

        // Build time_labels using the ORIGINAL index, then reassign sequential index
        let time_labels: Vec<String> = orig_indexed.iter()
            .map(|(orig_idx, _)| series.time_labels.get(*orig_idx).cloned().unwrap_or_default())
            .collect();

        let mut filtered_pts: Vec<crate::models::DataPoint> = orig_indexed.into_iter()
            .enumerate()
            .map(|(new_i, (_, mut p))| { p.index = new_i; p })
            .collect();

        // Sanity: ensure pts len matches labels len (they always should)
        debug_assert_eq!(filtered_pts.len(), time_labels.len());
        let _ = &mut filtered_pts; // suppress unused-mut warning

        crate::models::Series {
            name: series.name.clone(),
            color: series.color,
            unit: series.unit.clone(),
            points: filtered_pts,
            time_labels,
        }
    }).collect();

    app.plot_data = std::sync::Arc::from(filtered);
}

/// Wrapper that extracts series data and updates the full time range in app state.
/// Call this instead of extract_series_data directly.
pub fn extract_and_update_series_data(app: &mut CanViewApp) {
    // Always decode ALL data (ignoring zoom range) into plot_full_data
    let saved_start = app.plot_zoom_start.take();
    let saved_end = app.plot_zoom_end.take();

    app.plot_full_data = extract_series_data(app);

    app.plot_zoom_start = saved_start;
    app.plot_zoom_end = saved_end;

    // Store the full data time range for zoom-out reference
    {
        let mut min_t = f64::MAX;
        let mut max_t = f64::MIN;
        for series in app.plot_full_data.iter() {
            for p in series.points.iter() {
                if p.time < min_t { min_t = p.time; }
                if p.time > max_t { max_t = p.time; }
            }
        }
        if min_t < max_t {
            app.plot_full_time_min = Some(min_t);
            app.plot_full_time_max = Some(max_t);
            eprintln!("📏 Stored full time range: {:.3}s - {:.3}s", min_t, max_t);
        }
    }

    // Now apply zoom filter to set plot_data
    apply_zoom_to_full_data(app);
}
