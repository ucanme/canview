use crate::app::CanViewApp;
use crate::models::{DataPoint, Series};
use blf::LogObject;
use gpui::prelude::*;
use gpui::*;
use gpui_component::chart::LineChart;
use gpui_component::{h_flex, v_flex};
use gpui_component::scroll::ScrollableElement;
use std::sync::Arc;

/// Render the plot view with signal charts
pub fn render_plot_view(app: &CanViewApp, cx: &mut Context<CanViewApp>) -> impl IntoElement {
    let series_data = app.plot_data.clone();
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
                .child(render_signal_sidebar(app, cx))
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
                                render_empty_state()
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
        .child(
            div()
                .flex()
                .items_center()
                .gap_3()
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(0xe4e4e7))
                        .child("信号波形图 (Signal Plotter)")
                )
                .when(is_zoomed, |this| {
                    this.child(
                        div()
                            .px_2()
                            .py(px(2.0))
                            .bg(rgb(0x3f3f46))
                            .rounded(px(4.0))
                            .child(div().text_xs().text_color(rgb(0xa1a1aa)).child("Zoom Active"))
                    )
                })
        )
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
                                this.plot_data = crate::ui::views::chart_view::extract_series_data(this);
                                cx.notify();
                            }))
                            .child(div().text_xs().text_color(rgb(0xffffff)).child("Reset Zoom"))
                    )
                })
                .child(
                    div()
                        .px_2()
                        .py_1()
                        .bg(rgb(0x312e81))
                        .rounded(px(4.0))
                        .cursor_pointer()
                        .hover(|s| s.bg(rgb(0x1e1b4b)))
                        .on_mouse_down(gpui::MouseButton::Left, cx.listener(|this: &mut CanViewApp, _, _, cx| {
                            // Simple zoom into middle 50%
                             let (current_start, current_end) = if this.plot_zoom_start.is_some() || this.plot_zoom_end.is_some() {
                                (this.plot_zoom_start.unwrap_or(0.0), this.plot_zoom_end.unwrap_or(3600.0)) // FIXME: get real end
                             } else {
                                // Find min/max time from all points if possible, or use a default
                                (0.0, 100.0) // Placeholder
                             };
                             
                             // Better: calculate range from current plot_data
                             let mut min_t = f64::MAX;
                             let mut max_t = f64::MIN;
                             for series in this.plot_data.iter() {
                                 for p in series.points.iter() {
                                     if p.time < min_t { min_t = p.time; }
                                     if p.time > max_t { max_t = p.time; }
                                 }
                             }
                             
                             if min_t < max_t {
                                 let range = max_t - min_t;
                                 this.plot_zoom_start = Some(min_t + range * 0.25);
                                 this.plot_zoom_end = Some(max_t - range * 0.25);
                                 this.plot_data = crate::ui::views::chart_view::extract_series_data(this);
                                 cx.notify();
                             }
                        }))
                        .child(div().text_xs().text_color(rgb(0xffffff)).child("Zoom Selection"))
                )
                .child(
                    div()
                        .px_2()
                        .py_1()
                        .bg(rgb(0x3b82f6))
                        .rounded(px(4.0))
                        .cursor_pointer()
                        .hover(|s| s.bg(rgb(0x2563eb)))
                        .on_mouse_down(gpui::MouseButton::Left, cx.listener(|this: &mut CanViewApp, _, _, cx| {
                            this.plot_data = crate::ui::views::chart_view::extract_series_data(this);
                            cx.notify();
                        }))
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(0xffffff))
                                .child("Redraw Plot")
                        )
                )
        )
}

/// Render the signal selection sidebar
fn render_signal_sidebar(app: &CanViewApp, cx: &mut Context<CanViewApp>) -> impl IntoElement {
    // 1. Get all databases already loaded in memory
    let mut loaded_channels = std::collections::HashSet::new();
    for ch_id in app.dbc_channels.keys() { loaded_channels.insert(*ch_id); }
    for ch_id in app.ldf_channels.keys() { loaded_channels.insert(*ch_id); }

    // 2. Identify channels from config that are NOT loaded
    let mut configured_but_unloaded = Vec::new();
    for mapping in &app.app_config.mappings {
        if mapping.library_id.is_some() && mapping.version_name.is_some() {
            if !loaded_channels.contains(&mapping.channel_id) {
                configured_but_unloaded.push(mapping.clone());
            }
        }
    }

    div()
        .size_full()
        .flex()
        .flex_col()
        .bg(rgb(0x0f0f10))
        .child(
            div()
                .px_4()
                .py_2()
                .border_b_1()
                .border_color(rgb(0x27272a))
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(0xe4e4e7))
                        .child("信号选择 (Signals)")
                )
        )
        .child(
            div()
                .flex_1()
                .overflow_y_scrollbar()
                .p_2()
                .flex()
                .flex_col()
                .gap_4()
                .children({
                    let mut elements: Vec<AnyElement> = Vec::new();
                    
                    // Show Configured but Unloaded channels first
                    for mapping in configured_but_unloaded {
                        let lib_id = mapping.library_id.clone().unwrap_or_default();
                        let ver_name = mapping.version_name.clone().unwrap_or_default();
                        let ch_id = mapping.channel_id;
                        let ch_type_str = if mapping.channel_type.is_can() { "CAN" } else { "LIN" };
                        
                        elements.push(
                            div()
                                .flex()
                                .flex_col()
                                .gap_2()
                                .p_3()
                                .bg(rgb(0x18181b))
                                .border_1()
                                .border_color(rgb(0x27272a))
                                .rounded_lg()
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .justify_between()
                                        .child(
                                            div()
                                                .text_xs()
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(rgb(0x71717a))
                                                .child(format!("Channel {} ({})", ch_id, ch_type_str))
                                        )
                                        .child(
                                            div()
                                                .px_2()
                                                .py_1()
                                                .bg(rgb(0x313244))
                                                .rounded(px(4.0))
                                                .cursor_pointer()
                                                .hover(|s| s.bg(rgb(0x45475a)))
                                                .on_mouse_down(gpui::MouseButton::Left, {
                                                    let lib_id = lib_id.clone();
                                                    let ver_name = ver_name.clone();
                                                    cx.listener(move |this, _, _, cx| {
                                                        this.load_library_version(&lib_id, &ver_name, cx);
                                                    })
                                                })
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(rgb(0xcdd6f4))
                                                        .child("Load Database")
                                                )
                                        )
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(0x52525b))
                                        .child(format!("{}: {}", lib_id, ver_name))
                                )
                                .into_any_element()
                        );
                    }

                    // Show Loaded databases
                    let mut dbc_keys: Vec<_> = app.dbc_channels.keys().collect();
                    dbc_keys.sort();
                    for ch_id in dbc_keys {
                        if let Some(dbc) = app.dbc_channels.get(ch_id) {
                            let name = format!("Channel {} (CAN)", ch_id);
                            let db = crate::library::Database::Dbc(dbc.clone());
                            elements.push(render_database_entry(name, db, app, cx).into_any_element());
                        }
                    }

                    let mut ldf_keys: Vec<_> = app.ldf_channels.keys().collect();
                    ldf_keys.sort();
                    for ch_id in ldf_keys {
                        if let Some(ldf) = app.ldf_channels.get(ch_id) {
                            let name = format!("Channel {} (LIN)", ch_id);
                            let db = crate::library::Database::Ldf(ldf.clone());
                            elements.push(render_database_entry(name, db, app, cx).into_any_element());
                        }
                    }
                    
                    elements
                })
                .when(app.dbc_channels.is_empty() && app.ldf_channels.is_empty() && app.app_config.mappings.is_empty(), |this| {
                    this.child(
                        div()
                            .p_4()
                            .flex()
                            .flex_col()
                            .items_center()
                            .justify_center()
                            .gap_2()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0x71717a))
                                    .child("未配置信号库")
                            )
                            .child(
                                div()
                                    .px_3()
                                    .py_1()
                                    .bg(rgb(0x313244))
                                    .rounded(px(4.0))
                                    .cursor_pointer()
                                    .hover(|s| s.bg(rgb(0x45475a)))
                                    .on_mouse_down(gpui::MouseButton::Left, cx.listener(|this, _, _, cx| {
                                        this.current_view = crate::app::AppView::LibraryView;
                                        cx.notify();
                                    }))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(0xcdd6f4))
                                            .child("前往 Library 加载")
                                    )
                            )
                    )
                })
        )
}

/// Helper to render a single database entry in the sidebar
fn render_database_entry(name: String, db: crate::library::Database, app: &CanViewApp, cx: &mut Context<CanViewApp>) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_1()
        .child(
            div()
                .px_1()
                .text_xs()
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(0x3b82f6))
                .child(name.clone())
        )
        .child(
            crate::ui::views::database_preview::render_database_preview(
                &db,
                &app.signal_filter_text,
                None,
                Some(&if name.contains("CAN") { format!("CAN:{}", name.split_whitespace().nth(1).unwrap_or("0")) } else { format!("LIN:{}", name.split_whitespace().nth(1).unwrap_or("0")) }),
                &app.selected_signals,
                cx
            )
        )
}

/// Render empty state when no data is available
fn render_empty_state() -> AnyElement {
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
        .into_any_element()
}

/// Render the chart canvas using gpui-component LineChart
fn render_chart_canvas(app: &CanViewApp, series_data: Arc<[Series]>, cx: &mut Context<CanViewApp>) -> AnyElement {
    let is_dragging = app.is_dragging_zoom;
    let drag_start = app.zoom_drag_start_x;
    let drag_current = app.zoom_drag_current_x;

    div()
        .size_full()
        .relative()
        .child(
            v_flex()
                .size_full()
                .gap_4()
                .child(render_legend(&series_data))
                .children(series_data.iter().map(|series| {
                    render_single_chart(series)
                }))
        )
        // Zoom Box Overlay Layer
        .when(is_dragging, |this| {
            if let (Some(start), Some(current)) = (drag_start, drag_current) {
                let left = start.min(current);
                let width = (start - current).abs();
                this.child(
                    div()
                        .absolute()
                        .top_0()
                        .left(left)
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
        // Global Canvas Interactions for Zooming
        .on_mouse_down(MouseButton::Left, cx.listener(|this: &mut CanViewApp, event: &MouseDownEvent, _, cx| {
             this.is_dragging_zoom = true;
             this.zoom_drag_start_x = Some(event.position.x);
             this.zoom_drag_current_x = Some(event.position.x);
             cx.notify();
        }))
        .on_mouse_move(cx.listener(|this: &mut CanViewApp, event: &MouseMoveEvent, _, cx| {
            if this.is_dragging_zoom {
                this.zoom_drag_current_x = Some(event.position.x);
                cx.notify();
            }
        }))
        .on_mouse_up(MouseButton::Left, cx.listener(|this: &mut CanViewApp, event: &MouseUpEvent, window, cx| {
            if !this.is_dragging_zoom { return; }
            
            if let Some(start_x) = this.zoom_drag_start_x {
                let end_x = event.position.x;
                let (x1, x2) = (start_x.min(end_x), start_x.max(end_x));
                
                // Only zoom if the selection is significant (> 10 pixels)
                if (x1 - x2).abs() > px(10.0) {
                    // Let's find the current min/max time in the visible plot_data
                    let mut min_t = f64::MAX;
                    let mut max_t = f64::MIN;
                    for series in this.plot_data.iter() {
                        for p in series.points.iter() {
                            if p.time < min_t { min_t = p.time; }
                            if p.time > max_t { max_t = p.time; }
                        }
                    }

                    if min_t < max_t {
                        // Assuming the plot area is roughly the width of the canvas minus sidebars
                        // Sidebar is 320px. Chart area starts after that.
                        let window_width = window.bounds().size.width;
                        let plot_width = f32::from(window_width - px(320.0) - px(32.0)); // 320 sidebar + some padding
                        
                        // Map relative x to time
                        let start_rel = f32::from(x1 - px(320.0) - px(16.0)) / plot_width; // 16px is p_4 of the main area
                        let end_rel = f32::from(x2 - px(320.0) - px(16.0)) / plot_width;
                        
                        let t_range = max_t - min_t;
                        let new_start = min_t + t_range * start_rel as f64;
                        let new_end = min_t + t_range * end_rel as f64;
                        
                        this.plot_zoom_start = Some(new_start.max(0.0));
                        this.plot_zoom_end = Some(new_end);
                        this.plot_data = crate::ui::views::chart_view::extract_series_data(this);
                    }
                }
            }
            
            this.is_dragging_zoom = false;
            this.zoom_drag_start_x = None;
            this.zoom_drag_current_x = None;
            cx.notify();
        }))
        .into_any_element()
}

/// Render a single chart for one signal
fn render_single_chart(series: &Series) -> impl IntoElement {
    // Find the time range for this specific series to determine precision
    let mut min_t = f64::MAX;
    let mut max_t = f64::MIN;
    for p in series.points.iter() {
        if p.time < min_t { min_t = p.time; }
        if p.time > max_t { max_t = p.time; }
    }
    let range = max_t - min_t;
    
    // Adjust precision based on range
    let precision = if range < 0.01 { 4 }
                    else if range < 0.1 { 3 }
                    else if range < 1.0 { 2 }
                    else { 1 };

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
                .child(format!("{} {}", series.name, series.unit.as_ref().map(|u| format!("[{}]", u)).unwrap_or_default()))
        )
        .child(
            div()
                .flex_1()
                .py_2()
                .child(
                    LineChart::<DataPoint, SharedString, f64>::new(series.points.clone())
                        .x(move |d| {
                            let time_str = format!("{:.precision$}s", d.time, precision = precision);
                            
                            // Hack: Use zero-width characters to make each time string unique
                            // while keeping it visually identical for labels.
                            // This ensures ScalePoint treats them as distinct positions.
                            let mut unique_suffix = String::new();
                            let mut val = d.index;
                            for _ in 0..10 { // 10 bits is enough for 1024 points
                                unique_suffix.push(if val % 2 == 0 { '\u{200B}' } else { '\u{200C}' });
                                val /= 2;
                            }
                            format!("{}{}", time_str, unique_suffix).into()
                        })
                        .y(|d| d.value)
                        .stroke(series.color)
                        .linear()
                        .tick_margin(if series.points.len() > 10 { series.points.len() / 2 } else { 1 }) // Only 2 labels per chart for maximum legibility
                )
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

/// Extract series data from application state - SAFE VERSION
pub fn extract_series_data(app: &CanViewApp) -> Arc<[Series]> {
    eprintln!("=== Extract Series Data (SAFE) ===");
    
    // Limit processing to avoid stack overflow, but high enough for complete logs
    const MAX_SIGNALS: usize = 20;
    const MAX_MESSAGES: usize = 10_000_000;
    
    let signal_count = app.selected_signals.len().min(MAX_SIGNALS);
    let message_count = app.messages.len().min(MAX_MESSAGES);
    
    eprintln!("Processing {} signals from {} messages", signal_count, message_count);
    
    if signal_count == 0 || message_count == 0 {
        eprintln!("No data to process");
        return Arc::from([]);
    }

    // Use Vec to allocate on heap
    let mut all_series: Vec<Series> = Vec::new();
    
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
        eprintln!("  Signal {}: {}", idx, sig_id);
        
        // Parse signal ID: BUS:CHANNEL:MSG_ID:SIG_NAME or BUS:MSG_ID:SIG_NAME (compat)
        let parts: Vec<&str> = sig_id.split(':').collect();
        if parts.len() < 3 {
            eprintln!("    Skip: invalid format");
            continue;
        }

        let (bus_type, channel_filter, msg_id_str, sig_name) = if parts.len() >= 4 {
            (parts[0], Some(parts[1]), parts[2], parts[3])
        } else {
            (parts[0], None, parts[1], parts[2])
        };

        let msg_id_res = if msg_id_str.starts_with("0x") {
            u32::from_str_radix(&msg_id_str[2..], 16)
        } else {
            msg_id_str.parse::<u32>()
        };
        
        let msg_id = match msg_id_res {
            Ok(id) => id,
            Err(_) => continue,
        };

        let target_channel = channel_filter.and_then(|s| s.parse::<u16>().ok());
        let mut unit = None;
        let mut points = Vec::new();

        // Scan messages for this signal
        for msg in app.messages.iter().take(message_count) {
            match bus_type {
                "CAN" => {
                        let (m_id, ch, timestamp, data) = match msg {
                            LogObject::CanMessage(m) => (m.id, m.channel, m.header.object_time_stamp, m.data.as_slice()),
                            LogObject::CanMessage2(m) => (m.id, m.channel, m.header.object_time_stamp, m.data.as_slice()),
                            LogObject::CanFdMessage(m) => (m.id, m.channel, m.header.object_time_stamp, m.data.as_slice()),
                            LogObject::CanFdMessage64(m) => (m.id, m.channel as u16, m.header.object_time_stamp, m.data.as_slice()),
                            _ => continue,
                        };

                    if m_id == msg_id {
                        let time = timestamp as f64 / 1_000_000.0;
                        
                        // Apply zoom filter if active
                        if let Some(start) = app.plot_zoom_start {
                            if time < start { continue; }
                        }
                        if let Some(end) = app.plot_zoom_end {
                            if time > end { continue; }
                        }

                        if let Some(ch_filter) = target_channel {
                            if ch != ch_filter {
                                continue;
                            }
                        }
                        if let Some(dbc) = app.dbc_channels.get(&ch) {
                            if let Some(dbc_msg) = dbc.messages.get(&m_id) {
                                if let Some(sig) = dbc_msg.signals.get(sig_name) {
                                    // Extract unit if not already done
                                    if unit.is_none() && !sig.unit.is_empty() {
                                        unit = Some(sig.unit.clone());
                                    }
                                    
                                    let val = sig.decode(data);
                                    points.push(DataPoint { time, value: val, index: 0 });
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        // Downsample if too many points
        if points.len() > 1000 {
            let step = points.len() / 1000;
            points = points.into_iter().step_by(step).take(1000).collect();
        }
        
        // Assign indices after downsampling for correct label spacing
        for (i, p) in points.iter_mut().enumerate() {
            p.index = i;
        }

        if !points.is_empty() {
            all_series.push(Series {
                name: sig_name.to_string(),
                unit,
                points: points.into(),
                color: colors[idx % colors.len()],
            });
        }
    }

    Arc::from(all_series)
}
