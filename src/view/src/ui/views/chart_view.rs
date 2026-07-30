use crate::app::CanViewApp;
use gpui_component::input::{Input, InputState};
use crate::models::{DataPoint, Series};
use blf::LogObject;
use gpui::prelude::*;
use gpui::*;
use gpui_component::ActiveTheme;
use gpui_component::PixelsExt;
use gpui_component::v_flex;
use gpui_component::scroll::ScrollableElement;
use std::sync::Arc;
use chrono::{Timelike, Datelike};

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

    // Show the chart canvas as long as the user has selected signals — even if
    // none of those signals have data in the log, render_chart_canvas will draw
    // ⊘ no-data placeholder cards for them. Only fall back to render_empty_state
    // when no signals are selected at all.
    let has_data = !app.selected_signals.is_empty();

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
                .relative()  // for hover tooltip absolute positioning (sibling to scroll container, so it doesn't extend the scroll bounds)
                .child(render_toolbar(app, cx))
                .child(
                    if !has_data {
                        div()
                            .flex_1()
                            .p_4()
                            .child(render_empty_state(app))
                            .into_any_element()
                    } else {
                        render_chart_canvas(app, series_data.clone(), cx)
                    }
                )
                .child(
                    // Hover tooltip rendered as a sibling of the scroll container
                    // (NOT inside v_flex) so its absolute positioning doesn't
                    // extend the scroll content bounds (which caused infinite
                    // scroll-down past the last card).
                    if has_data {
                        render_hover_tooltip_overlay(app, &series_data, px(320.0))
                            .into_any_element()
                    } else {
                        div().into_any_element()
                    }
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


/// Render empty state when no signals are selected
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
                .child("尚未选择信号")
        )
        .child(
            div()
                .text_color(rgb(0x52525b))
                .text_sm()
                .child("请在左侧信号选择(Signals)中勾选信号，然后点击「绘制 (Plot)」按钮")
        )
        .child(
            div()
                .text_color(rgb(0x52525b))
                .text_xs()
                .child(format!("Debug: Messages={}, Selected={}", msg_count, sel_count))
        )
        .into_any_element()
}

/// Render the chart canvas — legend + zoom box + hover tooltip + per-signal canvas cards.
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

    // The v_flex itself is the scroll container: it has overflow_y_scroll +
    // track_scroll, and its children (legend + charts) take natural height so
    // the container grows beyond the viewport and scrolling kicks in. The
    // zoom-box overlay and hover line are positioned absolutely inside the
    // v_flex so they stay aligned with the visible charts.
    v_flex()
        .id("plot-scroll-container")
        .flex_1()
        .min_h_0()  // allow the flex child to shrink so its content can scroll
        .relative()
        .p_4()
        .gap_4()
        .overflow_y_scroll()
        .track_scroll(&app.plot_scroll_handle)
        .child(render_legend(&series_data))
        .children(app.selected_signals.iter().map(|signal_id| {
            let series = series_data.iter().find(|s| &s.name == signal_id);
            match series {
                Some(s) => render_single_chart(s, start_time, show_points, hover_x, hover_time).into_any_element(),
                None => render_no_data_chart(signal_id),
            }
        }))
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
        // Hover tooltip is rendered in render_plot_view as a sibling of the
        // scroll container (NOT inside v_flex) — see render_hover_tooltip_overlay.
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
                    this.plot_hover_y = Some(event.position.y);

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
                 this.plot_hover_y = None;
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

            // Mac 触控板会产生大量微小滚动事件，过低的阈值会导致轻微触碰就触发缩放。
            // 提高 threshold 并降低 zoom_factor，让缩放更"稳重"。
            if scroll_delta.abs() < 0.5 {
                return;
            }

            // Try reversed direction:
            // Scroll backward (toward user) = positive delta = zoom IN
            // Scroll forward (away from user) = negative delta = zoom OUT
            let zoom_in = scroll_delta > 0.0;

            // 单次缩放系数：1.1 比之前的 1.2 更平缓，避免一次滚动跳太多
            let zoom_factor = 1.1;

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
                        // 至少保留 2 个点可见：窗口宽度 >= 2 * 最小间隔，
                        // 即使窗口偏移到中间也至少能覆盖 2 个相邻点。
                        (smallest_gap * 2.0).max(1e-6)
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
                // Zoom out: expand toward full data range, centered on mouse.
                // 用一个更激进的步长：至少扩展 abs_range 的 10%，避免从极小 zoom
                // 花几十次滚动才能缩回来。
                let abs_range = abs_max_t - abs_min_t;
                let new_range = (current_range * zoom_factor).max(current_range + 0.1 * abs_range);

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

/// Helper to format time as absolute `YYYY-MM-DD HH:MM:SS.ffffff` (microsecond
/// precision, matching the log view's `%H:%M:%S%.6f`).
///
/// `time` is **Unix epoch seconds** (abs_ns / 1e9). The MergedView rewrites
/// every LogObject's `object_time_stamp` to absolute Unix nanoseconds during
/// multi-file merge (see `domain/multi_file.rs:160`), so we don't add
/// `start_time` here — `time` already is the wall-clock instant.
///
/// `start_time` is kept in the signature only for backward compatibility with
/// callers that still thread it through; it's not used.
fn format_time_relative_or_absolute(time: f64, _start_time: Option<chrono::NaiveDateTime>) -> String {
    use chrono::{TimeZone, Utc};
    let ns = (time * 1_000_000_000.0) as i64;
    let dt = Utc.timestamp_nanos(ns);
    dt.naive_utc().format("%Y-%m-%d %H:%M:%S%.6f").to_string()
}

/// Render the hover tooltip as an overlay sibling of the scroll container.
/// Rendered inside the right main area (which has `.relative()`), so absolute
/// positioning stays within the main area and does NOT extend the scroll
/// container's content bounds (which previously caused infinite scroll-down).
///
/// If hover state is None, returns an empty div (no tooltip shown).
fn render_hover_tooltip_overlay(
    app: &CanViewApp,
    series_data: &[Series],
    sidebar_width: Pixels,
) -> impl IntoElement {
    let (hover_x, hover_y, hover_time) = match (app.plot_hover_x, app.plot_hover_y, app.plot_hover_time) {
        (Some(x), Some(y), Some(t)) => (x, y, t),
        _ => return div().into_any_element(),
    };
    let chart_width = app.plot_width_px;
    // Right main area starts at sidebar_width (sidebar on left).
    // Local coordinates inside main area: x = hover_x - sidebar_width, y = hover_y - 0.
    // (render_plot_view fills the whole window; main area top = 0.)
    let local_x = hover_x - sidebar_width;
    render_hover_tooltip(local_x, hover_y, hover_time, series_data, sidebar_width, chart_width, app.start_time).into_any_element()
}

/// Render tooltip for hover over chart.
/// `local_x` is the mouse x inside the main area (after subtracting sidebar width).
/// `local_y` is the mouse y inside the main area (from window top, since main area top = 0).
fn render_hover_tooltip(
    local_x_in: Pixels,
    local_y_in: Pixels,
    hover_time: f64,
    series_data: &[Series],
    _sidebar_width: Pixels,
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
    // local_x is already the x relative to main area
    let local_x = local_x_in;

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
        .top(local_y_in)
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
                         .child(format!("{}: {:.2} {}", series.name.split(':').last().unwrap_or(&series.name), val, series.unit.as_deref().unwrap_or("")))
                 )
        }))
}

/// Render a placeholder card matching `render_single_chart`'s visual style
/// for a selected signal that has no data points in the current log.
fn render_no_data_chart(signal_id: &str) -> AnyElement {
    let display_name = signal_id.split(':').last().unwrap_or(signal_id);
    div()
        .flex()
        .flex_col()
        .h(px(240.0))
        .flex_shrink_0()
        .bg(rgb(0x18181b))
        .border_1()
        .border_color(rgb(0x27272a))
        .rounded_lg()
        .p_2()
        .items_center()
        .justify_center()
        .child(
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap_2()
                .child(div().text_xl().text_color(rgb(0x71717a)).child("⊘"))
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(0xa1a1aa))
                        .child(format!("No data for '{}'", display_name))
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(0x52525b))
                        .child("检查通道 ID 匹配 (DBC vs 日志) 或时间范围")
                )
        )
        .into_any_element()
}

/// Render a single chart for one signal — gpui canvas self-drawn, no grid.
fn render_single_chart(
    series: &Series,
    start_time: Option<chrono::NaiveDateTime>,
    show_points: bool,
    hover_x: Option<Pixels>,
    hover_time: Option<f64>,
) -> impl IntoElement {
    let points = Arc::new(series.points.clone());
    let color = series.color;
    let unit = series.unit.clone();
    let start_time_for_paint = start_time;
    let title = format!(
        "{} {} | {} pts",
        series.name.split(':').last().unwrap_or(&series.name),
        series.unit.as_ref().map(|u| format!("[{}]", u)).unwrap_or_default(),
        series.points.len(),
    );

    div()
        .flex()
        .flex_col()
        .h(px(240.0))
        .flex_shrink_0()
        .bg(rgb(0x18181b))
        .border_1()
        .border_color(rgb(0x27272a))
        .rounded_lg()
        .p_2()
        .child(div().text_xs().text_color(color).child(title))
        .child(
            div()
                .flex_1()
                .min_h_0()
                .py_1()
                .child({
                    let prepaint_points = points.clone();
                    gpui::canvas(
                        move |bounds, _window, _cx| {
                            // prepaint: 预计算坐标变换参数；空数据返回 None 跳过 paint
                            if prepaint_points.is_empty() {
                                return None;
                            }
                            let min_t = prepaint_points.first().unwrap().time;
                            let max_t = prepaint_points.last().unwrap().time;
                            let (mut min_v, mut max_v) = prepaint_points.iter().fold(
                                (f64::INFINITY, f64::NEG_INFINITY),
                                |(mn, mx), p| (mn.min(p.value), mx.max(p.value)),
                            );
                            // 当所有值相等（含全 0）时，扩展 Y 范围让 0 落在 X 轴底部，
                            // 而不是把折线压在 Y 轴顶部。
                            if max_v == min_v {
                                min_v = min_v.min(0.0);
                                max_v = max_v.max(0.0);
                                if max_v == min_v {
                                    // 仍相等（仅当 min_v=max_v=0）：给一个 1 的虚拟高度
                                    max_v = min_v + 1.0;
                                }
                            }
                            let v_range = (max_v - min_v).max(1e-9);
                            let t_range = (max_t - min_t).max(1e-9);
                            Some(ChartLayout {
                                bounds,
                                min_t,
                                min_v,
                                max_v,
                                t_range,
                                v_range,
                            })
                        },
                        move |_bounds, layout, window, cx| {
                            let layout = match layout {
                                Some(l) => l,
                                None => return,
                            };
                            let bounds = layout.bounds;
                            let pad_left = px(36.0);
                            let pad_right = px(4.0);
                            let pad_top = px(4.0);
                            let pad_bottom = px(14.0);
                            let chart_w = bounds.size.width - pad_left - pad_right;
                            let chart_h = bounds.size.height - pad_top - pad_bottom;

                            // 坐标系原点（左下角）和右上角
                            let origin_x = bounds.origin.x + pad_left;
                            let origin_y = bounds.origin.y + pad_top; // 顶部 Y
                            let x_axis_y = bounds.origin.y + pad_top + chart_h; // 底部 X 轴线 Y

                            let stroke_color = cx.theme().border;

                            // === 1. Y 轴线 ===
                            {
                                let mut b = PathBuilder::stroke(px(1.));
                                b.move_to(point(origin_x, origin_y));
                                b.line_to(point(origin_x, x_axis_y));
                                if let Ok(path) = b.build() {
                                    window.paint_path(path, stroke_color);
                                }
                            }

                            // === 2. X 轴线 ===
                            {
                                let mut b = PathBuilder::stroke(px(1.));
                                b.move_to(point(origin_x, x_axis_y));
                                b.line_to(point(bounds.origin.x + pad_left + chart_w, x_axis_y));
                                if let Ok(path) = b.build() {
                                    window.paint_path(path, stroke_color);
                                }
                            }

                            // === 3. Y 轴刻度 + 标签（动态数量，按 chart_h 估算）===
                            let n_y_ticks = calc_y_tick_count(chart_h.as_f32());
                            let mut y_labels: Vec<gpui_component::plot::label::Text> = Vec::with_capacity(n_y_ticks);
                            for i in 0..n_y_ticks {
                                // i=0 → max_v (top), i=n_y_ticks-1 → min_v (bottom)
                                let ratio = if n_y_ticks == 1 { 0.0 } else { i as f32 / (n_y_ticks - 1) as f32 };
                                let v = layout.max_v - (layout.max_v - layout.min_v) * ratio as f64;
                                let y_px = origin_y + chart_h * ratio;
                                // 短刻度线（向左 4px）
                                {
                                    let mut b = PathBuilder::stroke(px(1.));
                                    b.move_to(point(origin_x - px(4.), y_px));
                                    b.line_to(point(origin_x, y_px));
                                    if let Ok(path) = b.build() {
                                        window.paint_path(path, stroke_color);
                                    }
                                }
                                // 标签文本（在 origin_x - 6px 处右对齐，垂直居中于 y_px）
                                let label = format!("{:.1}", v);
                                // PlotLabel::paint 内部会再加 bounds.origin，所以这里必须
                                // 存画布相对坐标（已减去 bounds.origin），避免双重偏移。
                                y_labels.push(
                                    gpui_component::plot::label::Text::new(
                                        label,
                                        point(
                                            origin_x - px(6.) - bounds.origin.x,
                                            y_px - px(5.) - bounds.origin.y,
                                        ),
                                        cx.theme().muted_foreground,
                                    )
                                    .font_size(px(10.))
                                    .align(gpui::TextAlign::Right),
                                );
                            }
                            let y_plot_label = gpui_component::plot::PlotLabel::new(y_labels);
                            y_plot_label.paint(&bounds, window, cx);

                            // === 4. X 轴刻度 + 标签（动态数量）===
                            let n_ticks = calc_x_tick_count(chart_w.as_f32());
                            let mut x_labels: Vec<gpui_component::plot::label::Text> = Vec::with_capacity(n_ticks);
                            for i in 0..n_ticks {
                                let ratio = if n_ticks == 1 { 0.0 } else { i as f32 / (n_ticks - 1) as f32 };
                                let x_px = origin_x + chart_w * ratio;
                                // 短刻度线（向下 4px）
                                {
                                    let mut b = PathBuilder::stroke(px(1.));
                                    b.move_to(point(x_px, x_axis_y));
                                    b.line_to(point(x_px, x_axis_y + px(4.)));
                                    if let Ok(path) = b.build() {
                                        window.paint_path(path, stroke_color);
                                    }
                                }
                                // 时间标签：X 轴上省略年月日，用 HH:MM:SS.ffffff（与 log view 同精度）；
                                // 完整 YYYY-MM-DD HH:MM:SS.ffffff 在悬停 tooltip 里显示。
                                let t = layout.min_t + layout.t_range * ratio as f64;
                                let full_label = format_time_relative_or_absolute(t, start_time_for_paint);
                                let label_text = if full_label.starts_with("Time: ") {
                                    full_label
                                } else {
                                    // 截取 HH:MM:SS.ffffff（跳过 "YYYY-MM-DD " 的 11 字符）
                                    full_label.get(11..).unwrap_or(&full_label).to_string()
                                };
                                x_labels.push(
                                    gpui_component::plot::label::Text::new(
                                        label_text,
                                        point(
                                            x_px - bounds.origin.x,
                                            x_axis_y + px(4.) - bounds.origin.y,
                                        ),
                                        cx.theme().muted_foreground,
                                    )
                                    .font_size(px(10.))
                                    .align(gpui::TextAlign::Center),
                                );
                            }
                            let x_plot_label = gpui_component::plot::PlotLabel::new(x_labels);
                            x_plot_label.paint(&bounds, window, cx);

                            // === 5. 折线 ===
                            let mut builder = PathBuilder::stroke(px(1.0));
                            let mut started = false;
                            for p in points.iter() {
                                let x = origin_x
                                    + px(
                                        ((p.time - layout.min_t) / layout.t_range
                                            * chart_w.as_f32() as f64) as f32,
                                    );
                                let y = origin_y
                                    + px(
                                        ((layout.max_v - p.value) / layout.v_range
                                            * chart_h.as_f32() as f64) as f32,
                                    );
                                if !started {
                                    builder.move_to(point(x, y));
                                    started = true;
                                } else {
                                    builder.line_to(point(x, y));
                                }
                            }
                            if started {
                                if let Ok(path) = builder.build() {
                                    window.paint_path(path, color);
                                }
                            }

                            // === 6. data points（可选，show_points 为 true）===
                            if show_points {
                                for p in points.iter() {
                                    let x = origin_x
                                        + px(
                                            ((p.time - layout.min_t) / layout.t_range
                                                * chart_w.as_f32() as f64) as f32,
                                        );
                                    let y = origin_y
                                        + px(
                                            ((layout.max_v - p.value) / layout.v_range
                                                * chart_h.as_f32() as f64) as f32,
                                        );
                                    // 3px 实心方块（缩小后不堆叠）
                                    let dot_bounds = gpui::Bounds::new(
                                        point(x - px(1.5), y - px(1.5)),
                                        size(px(3.), px(3.)),
                                    );
                                    window.paint_quad(gpui::fill(dot_bounds, color));
                                }
                            }

                            // === 7. 悬停竖线 + 最近点数值标签 ===
                            // 鼠标在 plot 区时，每张卡片都画一条贯穿 top→x_axis_y 的竖线，
                            // 并在最近的数据点处显示一个带边框的小数值气泡。
                            if let (Some(hx), Some(ht)) = (hover_x, hover_time) {
                                // 只在 hover_time 落在 [min_t, max_t] 范围内才画
                                let max_t = layout.min_t + layout.t_range;
                                if ht >= layout.min_t && ht <= max_t {
                                    let line_x = origin_x
                                        + px(
                                            ((ht - layout.min_t) / layout.t_range
                                                * chart_w.as_f32() as f64) as f32,
                                        );
                                    // 仅当竖线 x 在 canvas 水平范围内时画
                                    if line_x >= origin_x && line_x <= origin_x + chart_w {
                                        // 竖线
                                        let mut b = PathBuilder::stroke(px(1.));
                                        b.move_to(point(line_x, origin_y));
                                        b.line_to(point(line_x, x_axis_y));
                                        if let Ok(path) = b.build() {
                                            window.paint_path(
                                                path,
                                                gpui::rgba(0xd4d4d8cc),
                                            );
                                        }
                                        // 找最近的数据点
                                        let idx = points
                                            .partition_point(|p| p.time < ht);
                                        let p_before = if idx > 0 {
                                            points.get(idx - 1)
                                        } else {
                                            None
                                        };
                                        let p_after = points.get(idx);
                                        let nearest = match (p_before, p_after) {
                                            (Some(b), Some(a)) => {
                                                let db = (ht - b.time).abs();
                                                let da = (a.time - ht).abs();
                                                if db < da { b } else { a }
                                            }
                                            (Some(b), None) => b,
                                            (None, Some(a)) => a,
                                            (None, None) => return,
                                        };
                                        // 高亮该点（4px 圆点，比 show_points 大）
                                        let dot_y = origin_y
                                            + px(
                                                ((layout.max_v - nearest.value)
                                                    / layout.v_range
                                                    * chart_h.as_f32() as f64)
                                                    as f32,
                                            );
                                        let dot_x = origin_x
                                            + px(
                                                ((nearest.time - layout.min_t) / layout.t_range
                                                    * chart_w.as_f32() as f64) as f32,
                                            );
                                        window.paint_quad(gpui::fill(
                                            gpui::Bounds::new(
                                                point(dot_x - px(2.), dot_y - px(2.)),
                                                size(px(4.), px(4.)),
                                            ),
                                            color,
                                        ));
                                        // 在高亮点上方显示数值标签
                                        let label_text = format!(
                                            "{:.2}{}",
                                            nearest.value,
                                            unit.as_deref().unwrap_or("")
                                        );
                                        // 标签放在点上方 6px，左对齐，避免遮挡折线
                                        let label_origin = point(
                                            dot_x + px(4.) - bounds.origin.x,
                                            dot_y - px(16.) - bounds.origin.y,
                                        );
                                        let label = gpui_component::plot::label::Text::new(
                                            label_text,
                                            label_origin,
                                            color,
                                        )
                                        .font_size(px(10.))
                                        .align(gpui::TextAlign::Left);
                                        let plot_label =
                                            gpui_component::plot::PlotLabel::new(vec![label]);
                                        plot_label.paint(&bounds, window, cx);
                                    }
                                }
                            }
                        },
                    )
                    .size_full()
                }),
        )
}

/// Canvas 自绘坐标变换参数
struct ChartLayout {
    bounds: gpui::Bounds<gpui::Pixels>,
    min_t: f64,
    t_range: f64,
    min_v: f64,
    max_v: f64,
    v_range: f64,
}


/// Render legend showing all series
fn render_legend(series_data: &[Series]) -> impl IntoElement {
    // Render an empty (zero-height) div when there are no series, so the
    // legend's bordered container doesn't show as an empty box above the
    // first chart when every selected signal has no data in the log.
    if series_data.is_empty() {
        return div().into_any_element();
    }
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
                        .child(series.name.split(':').last().unwrap_or(&series.name).to_string())
                )
        }))
        .into_any_element()
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

            // Convert timestamp to seconds. After MergedView::from_segments,
            // object_time_stamp is always absolute Unix nanoseconds (abs_ns),
            // regardless of the original object_flags. The flags-based
            // conversion (TimeTenMics / TimeOneNans) is done at merge time.
            let time = timestamp as f64 / 1_000_000_000.0;
            if collected == 0 {
                eprintln!("    🕐 Using absolute Unix nanosecond timestamp");
            }

            
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

        // time_labels field is now unused by render (the X axis draws via
        // format_time_relative_or_absolute directly). Leave it empty to avoid
        // building misleading strings; apply_zoom_to_full_data handles the
        // empty case via unwrap_or_default().
        let time_labels: Vec<String> = Vec::new();
        let _ = app.start_time; // silence unused warning

        eprintln!("    ✅ Final point count: {}", points.len());

        all_series.push(Series {
            name: sig_id.clone(),
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

    // Guard: if all series have 0 points (zoom window is between points), the
    // chart would be empty. Fall back to the full range so user never sees a
    // blank chart.
    let all_empty = filtered.iter().all(|s| s.points.is_empty());
    if all_empty {
        eprintln!("⚠️  Zoom window contains no points — resetting to full range");
        app.plot_zoom_start = None;
        app.plot_zoom_end = None;
        app.plot_data = app.plot_full_data.clone();
        return;
    }

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

/// 按 80px/刻度估算 X 轴刻度数量，clamp [2, 6]。
/// chart_w_px 是 canvas 内可用于画折线的水平像素（已扣除左右 padding）。
pub fn calc_x_tick_count(chart_w_px: f32) -> usize {
    let approx = (chart_w_px / 80.0).floor() as usize;
    approx.clamp(2, 6)
}

/// 按 50px/刻度估算 Y 轴刻度数量，clamp [3, 8]。
/// chart_h_px 是 canvas 内可用于画折线的垂直像素（已扣除上下 padding）。
/// 至少 3 个 (max / mid / min)，最多 8 个（避免标签过密）。
pub fn calc_y_tick_count(chart_h_px: f32) -> usize {
    let approx = (chart_h_px / 50.0).floor() as usize;
    approx.clamp(3, 8)
}

/// 时间标签 fallback（series.time_labels 为空时用）。
/// span 是 max_t - min_t（秒）。
/// < 60s → 三位小数秒；< 1h → 一位小数秒；否则 → 一位小数分钟。
pub fn format_time_label(t: f64, span: f64) -> String {
    if span < 60.0 {
        format!("{:.3}s", t)
    } else if span < 3600.0 {
        format!("{:.1}s", t)
    } else {
        format!("{:.1}min", t / 60.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies that the signal ID format produced by the sidebar matches what
    /// `extract_series_data` will set as `Series.name`. This is a structural
    /// test of the ID format — it doesn't run `extract_series_data` (which
    /// needs a full message log) but locks the format convention.
    #[test]
    fn signal_id_format_matches_series_name_convention() {
        let sidebar_signal_id = "CAN:1:256:EngineSpeed";
        let parts: Vec<&str> = sidebar_signal_id.split(':').collect();
        assert!(parts.len() >= 4, "signal id must have BUS:CH:MSG:NAME structure");
        // The Series.name after this task's change will equal the full signal_id;
        // display names extract the last segment:
        let display_name = sidebar_signal_id.split(':').last().unwrap();
        assert_eq!(display_name, "EngineSpeed");
    }

    #[test]
    fn calc_x_tick_count_basic() {
        assert_eq!(calc_x_tick_count(400.0), 5);
        assert_eq!(calc_x_tick_count(80.0), 2);   // 80px → 1 → clamp 2
        assert_eq!(calc_x_tick_count(40.0), 2);   // 40px → 0 → clamp 2
        assert_eq!(calc_x_tick_count(600.0), 6);  // 600/80=7.5 → 7 → clamp 6
        assert_eq!(calc_x_tick_count(160.0), 2);
        assert_eq!(calc_x_tick_count(320.0), 4);
    }

    #[test]
    fn calc_y_tick_count_basic() {
        assert_eq!(calc_y_tick_count(150.0), 3);  // 150/50=3 → clamp 3
        assert_eq!(calc_y_tick_count(100.0), 3);  // 100/50=2 → clamp 3
        assert_eq!(calc_y_tick_count(50.0), 3);   // 50/50=1 → clamp 3
        assert_eq!(calc_y_tick_count(300.0), 6);  // 300/50=6
        assert_eq!(calc_y_tick_count(500.0), 8);  // 500/50=10 → clamp 8
        assert_eq!(calc_y_tick_count(250.0), 5);
    }

    #[test]
    fn format_time_label_ranges() {
        assert_eq!(format_time_label(12.345, 30.0), "12.345s");
        assert_eq!(format_time_label(5.0, 300.0), "5.0s");
        assert_eq!(format_time_label(120.0, 4000.0), "2.0min");
        // Boundary cases: at exactly 60.0 the first branch (< 60.0) fails → second branch → "60.0s";
        // at exactly 3600.0 the second branch (< 3600.0) fails → else → 3600.0/60.0 = 60.0 → "60.0min".
        assert_eq!(format_time_label(60.0, 60.0), "60.0s");
        assert_eq!(format_time_label(3600.0, 3600.0), "60.0min");
    }
}
