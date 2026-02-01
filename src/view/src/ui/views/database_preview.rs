use crate::app::CanViewApp;
use crate::library::Database;
use gpui::prelude::*;
use gpui::*;
use gpui_component::scroll::ScrollableElement;
use gpui_component::input::Input;
use gpui_component::input::InputState;
use parser::{dbc::DbcDatabase, ldf::LdfDatabase};

pub fn render_database_preview(
    database: &Database,
    filter_text: &str,
    filter_input: Option<&Entity<InputState>>,
    id_prefix: Option<&str>,
    selected_signals: &[String],
    cx: &mut Context<CanViewApp>,
) -> impl IntoElement {
    let has_selection = !selected_signals.is_empty();
    
    div()
        .flex()
        .flex_col()
        .size_full()
        .bg(rgb(0x1e1e2e))
        .child(
            // Toolbar
            div()
                .flex()
                .items_center()
                .justify_between()
                .px_4()
                .py_2()
                .bg(rgb(0x181825))
                .border_b_1()
                .border_color(rgb(0x313244))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_4()
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(0xcdd6f4))
                                .child("Database Preview")
                        )
                        .child(
                            div()
                                .px_2()
                                .py_0p5()
                                .bg(rgb(0x313244))
                                .rounded(px(4.0))
                                .text_xs()
                                .text_color(rgb(0xa6adc8))
                                .child(if has_selection {
                                    format!("{} signals selected", selected_signals.len())
                                } else {
                                    "No signals selected".to_string()
                                })
                        )
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_4()
                        .child(
                            div()
                                .w(px(200.0))
                                .h(px(28.0))
                                .child(if let Some(input) = filter_input {
                                    div()
                                        .flex_1()
                                        .child(Input::new(input).appearance(true))
                                        .into_any_element()
                                } else {
                                    div().into_any_element()
                                }),
                        )
                        .child(if has_selection {
                            div()
                                .px_3()
                                .py_1()
                                .bg(rgb(0x89b4fa))
                                .text_color(rgb(0x11111b))
                                .text_sm()
                                .font_weight(FontWeight::BOLD)
                                .rounded(px(4.0))
                                .cursor_pointer()
                                .hover(|s| s.bg(rgb(0xb4befe)))
                                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                    this.plot_data = crate::ui::views::chart_view::extract_series_data(this);
                                    this.current_view = crate::app::AppView::PlotView;
                                    cx.notify();
                                }))
                                .child("Plot Selected")
                        } else {
                            div()
                        })
                ),
        )
        .child(
            // Content area
            div()
                .flex_1()
                .overflow_y_scrollbar()
                .child(match database {
                    Database::Dbc(dbc) => render_dbc_content(dbc, filter_text, id_prefix, selected_signals, cx).into_any_element(),
                    Database::Ldf(ldf) => render_ldf_content(ldf, filter_text, id_prefix, selected_signals, cx).into_any_element(),
                }),
        )
}

fn render_dbc_content(dbc: &DbcDatabase, filter_text: &str, id_prefix: Option<&str>, selected_signals: &[String], cx: &mut Context<CanViewApp>) -> impl IntoElement {
    let bus_prefix = id_prefix.unwrap_or("CAN");
    let lower_filter = filter_text.to_lowercase();
    let mut messages: Vec<_> = dbc.messages.values().collect();
    messages.sort_by_key(|m| m.id);

    let mut elements = Vec::new();

    for msg in messages {
        let matches_msg = msg.name.to_lowercase().contains(&lower_filter)
            || format!("0x{:x}", msg.id).to_lowercase().contains(&lower_filter);
        
        let matching_signals: Vec<_> = msg.signals.values()
            .filter(|s| s.name.to_lowercase().contains(&lower_filter))
            .collect();
        
        if !matches_msg && matching_signals.is_empty() && !filter_text.is_empty() {
            continue;
        }

        let mut signal_elements = Vec::new();
        let mut signals: Vec<_> = msg.signals.values().collect();
        signals.sort_by_key(|s| s.start_bit);
        
        let msg_id = msg.id;
        
        for sig in signals {
            let sig_name = sig.name.clone();
            let is_sig_match = sig_name.to_lowercase().contains(&lower_filter);
            let opacity = if !filter_text.is_empty() && !is_sig_match && !matches_msg { 0.3 } else { 1.0 };
            
            let signal_id = format!("{}:{}:{}", bus_prefix, msg_id, sig_name);
            let is_selected = selected_signals.contains(&signal_id);
            let sig_id_clone = signal_id.clone();

            signal_elements.push(
                div()
                    .flex()
                    .items_center()
                    .px_4()
                    .py_1()
                    .gap_4()
                    .opacity(opacity)
                    .hover(|s| s.bg(rgb(0x1a1a1a)))
                    .child(
                        // Selection Dot/Checkbox
                        div()
                            .id(format!("sig-check-{}", signal_id))
                            .w(px(16.0))
                            .h(px(16.0))
                            .rounded(px(2.0))
                            .border_1()
                            .border_color(if is_selected { hsla(217.0 / 360.0, 0.91, 0.76, 1.0) } else { hsla(234.0 / 360.0, 0.13, 0.31, 1.0) })
                            .bg(if is_selected { hsla(217.0 / 360.0, 0.91, 0.76, 1.0) } else { hsla(0.0, 0.0, 0.0, 0.0) })
                            .cursor_pointer()
                            .on_mouse_down(MouseButton::Left, cx.listener({
                                let sig_id_clone = sig_id_clone.clone();
                                move |this, _, _, cx| {
                                    cx.stop_propagation();
                                    eprintln!("🖱️ Toggling signal: {}", sig_id_clone);
                                    if let Some(pos) = this.selected_signals.iter().position(|s| s == &sig_id_clone) {
                                        this.selected_signals.remove(pos);
                                    } else {
                                        this.selected_signals.push(sig_id_clone.clone());
                                    }
                                    cx.notify();
                                }
                            }))
                    )
                    .child(
                        div()
                            .flex_1()
                            .text_sm()
                            .text_color(if is_selected { rgb(0xcdd6f4) } else { rgb(0xa6adc8) })
                            .child(sig_name),
                    )
                    .child(
                        div()
                            .w(px(100.0))
                            .text_xs()
                            .text_color(rgb(0x6c7086))
                            .child(format!("{} bits", sig.signal_size)),
                    )
            );
        }

        elements.push(
            div()
                .flex()
                .flex_col()
                .border_b_1()
                .border_color(rgb(0x1a1a1a))
                .child(
                    // Message Header
                    div()
                        .flex()
                        .items_center()
                        .px_4()
                        .py_2()
                        .bg(rgb(0x131313))
                        .gap_4()
                        .child(
                            div()
                                .w(px(80.0))
                                .text_xs()
                                .text_color(rgb(0x89b4fa))
                                .child(format!("0x{:X}", msg.id)),
                        )
                        .child(
                            div()
                                .flex_1()
                                .text_sm()
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(0xcdd6f4))
                                .child(msg.name.clone()),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(0x6c7086))
                                .child(format!("DLC: {}", msg.dlc)),
                        ),
                )
                .child(
                    // Signals List
                    div().flex().flex_col().children(signal_elements)
                )
        );
    }

    div().flex().flex_col().children(elements)
}

fn render_ldf_content(ldf: &LdfDatabase, filter_text: &str, id_prefix: Option<&str>, selected_signals: &[String], cx: &mut Context<CanViewApp>) -> impl IntoElement {
    let bus_prefix = id_prefix.unwrap_or("LIN");
    let lower_filter = filter_text.to_lowercase();
    let mut frames: Vec<_> = ldf.frames.values().collect();
    frames.sort_by_key(|f| f.id);

    let mut elements = Vec::new();

    for frame in frames {
         let matches_frame = frame.name.to_lowercase().contains(&lower_filter)
            || format!("0x{:x}", frame.id).to_lowercase().contains(&lower_filter);
        
        let matching_signals: Vec<_> = frame.signals.iter()
            .filter(|s| s.signal_name.to_lowercase().contains(&lower_filter))
            .collect();
        
        if !matches_frame && matching_signals.is_empty() && !filter_text.is_empty() {
            continue;
        }

        let mut signal_elements = Vec::new();
        let frame_id = frame.id;

        for mapping in &frame.signals {
            let sig_name = mapping.signal_name.clone();
            let is_sig_match = sig_name.to_lowercase().contains(&lower_filter);
            let opacity = if !filter_text.is_empty() && !is_sig_match && !matches_frame { 0.3 } else { 1.0 };
            
            let signal_id = format!("{}:{}:{}", bus_prefix, frame_id, sig_name);
            let is_selected = selected_signals.contains(&signal_id);
            let sig_id_clone = signal_id.clone();
            
            let sig_size = ldf.signals.get(&sig_name).map(|s| s.size).unwrap_or(0);

            signal_elements.push(
                div()
                    .flex()
                    .items_center()
                    .px_4()
                    .py_1()
                    .gap_4()
                    .opacity(opacity)
                    .hover(|s| s.bg(rgb(0x1a1a1a)))
                    .child(
                        // Selection Dot/Checkbox
                        div()
                            .w(px(16.0))
                            .h(px(16.0))
                            .rounded(px(2.0))
                            .border_1()
                            .border_color(if is_selected { hsla(41.0 / 360.0, 0.88, 0.78, 1.0) } else { hsla(234.0 / 360.0, 0.13, 0.31, 1.0) })
                            .bg(if is_selected { hsla(41.0 / 360.0, 0.88, 0.78, 1.0) } else { hsla(0.0, 0.0, 0.0, 0.0) })
                            .cursor_pointer()
                            .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                if let Some(pos) = this.selected_signals.iter().position(|s| s == &sig_id_clone) {
                                    this.selected_signals.remove(pos);
                                } else {
                                    this.selected_signals.push(sig_id_clone.clone());
                                }
                                cx.notify();
                            }))
                    )
                    .child(
                        div()
                            .flex_1()
                            .text_sm()
                            .text_color(if is_selected { rgb(0xcdd6f4) } else { rgb(0xa6adc8) })
                            .child(sig_name),
                    )
                    .child(
                        div()
                            .w(px(100.0))
                            .text_xs()
                            .text_color(rgb(0x6c7086))
                            .child(format!("{} bits", sig_size)),
                    )
            );
        }

        elements.push(
            div()
                .flex()
                .flex_col()
                .border_b_1()
                .border_color(rgb(0x1a1a1a))
                .child(
                    // Frame Header
                    div()
                        .flex()
                        .items_center()
                        .px_4()
                        .py_2()
                        .bg(rgb(0x131313))
                        .gap_4()
                        .child(
                            div()
                                .w(px(80.0))
                                .text_xs()
                                .text_color(rgb(0xf9e2af))
                                .child(format!("0x{:X}", frame.id)),
                        )
                        .child(
                            div()
                                .flex_1()
                                .text_sm()
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(0xcdd6f4))
                                .child(frame.name.clone()),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(0x6c7086))
                                .child(format!("Size: {}", frame.size)),
                        ),
                )
                .child(
                    // Signals List
                    div().flex().flex_col().children(signal_elements)
                )
        );
    }

    div().flex().flex_col().children(elements)
}
