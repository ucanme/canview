//! Plot sidebar — channel/message/signal tree with fold state.
//!
//! Migrated from `chart_view.rs`. The pure `extract_signal_items` function
//! builds the flattened, filtered list of sidebar items (unit-testable).
//! `render_signal_sidebar` wraps it in a `uniform_list`.

use crate::app::CanViewApp;
use crate::models::ChannelMapping;
use gpui::prelude::*;
use gpui::*;

/// Items produced by `extract_signal_items` and consumed by `render_sidebar_item`.
#[derive(Clone)]
pub enum SidebarItem {
    ChannelHeader {
        name: String,
        ch_id: u16,
        is_can: bool,
        is_loaded: bool,
        mapping: Option<ChannelMapping>,
        is_expanded: bool,
        /// How many signals under this channel are currently selected — shown as a badge.
        selected_count: usize,
    },
    MessageHeader {
        name: String,
        id: u32,
        is_can: bool,
        is_expanded: bool,
        ch_id: u16,
    },
    SignalItem {
        name: String,
        id: String,
        size: u32,
        is_selected: bool,
        is_can: bool,
        ch_id: u16,
        msg_id: u32,
    },
}

/// Render a single sidebar item. Click handlers for ChannelHeader / MessageHeader
/// toggle fold state via `toggle_channel_expanded` / `toggle_message_expanded`.
pub fn render_sidebar_item(item: &SidebarItem, view: Entity<CanViewApp>) -> AnyElement {
    match item {
        SidebarItem::ChannelHeader { name, ch_id, is_can, is_loaded, mapping, is_expanded, selected_count } => {
            let lib_id = mapping.as_ref().and_then(|m| m.library_id.clone()).unwrap_or_default();
            let ver_name = mapping.as_ref().and_then(|m| m.version_name.clone()).unwrap_or_default();
            let ch_id = *ch_id;
            let is_loaded = *is_loaded;
            let is_expanded = *is_expanded;
            let selected_count = *selected_count;
            let arrow = if is_expanded { "▾" } else { "▸" };

            div()
                .px_2()
                .py_1()
                .bg(rgb(0x18181b))
                .border_b_1()
                .border_color(rgb(0x27272a))
                .flex()
                .items_center()
                .justify_between()
                .cursor_pointer()
                .hover(|s| s.bg(rgb(0x1f1f22)))
                .on_mouse_down(gpui::MouseButton::Left, {
                    let view = view.clone();
                    move |_, _, cx| {
                        view.update(cx, |this, cx| {
                            this.toggle_channel_expanded(ch_id);
                            cx.notify();
                        });
                    }
                })
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1()
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(0x71717a))
                                .w(px(10.0))
                                .child(arrow)
                        )
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(if *is_can { rgb(0x3b82f6) } else { rgb(0xeab308) })
                                .child(name.clone())
                        )
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .when(selected_count > 0, |this| {
                            this.child(
                                div()
                                    .px_1p5()
                                    .py(px(1.0))
                                    .bg(rgb(0x3b82f6))
                                    .rounded(px(8.0))
                                    .text_xs()
                                    .text_color(rgb(0xffffff))
                                    .child(format!("{}", selected_count))
                            )
                        })
                        .when(!is_loaded, {
                            let lib_id = lib_id.clone();
                            let ver_name = ver_name.clone();
                            let view = view.clone();
                            move |this| {
                                this.child(
                                    div()
                                        .px_1p5()
                                        .py(px(1.0))
                                        .bg(rgb(0x313244))
                                        .rounded(px(3.0))
                                        .cursor_pointer()
                                        .hover(|s| s.bg(rgb(0x45475a)))
                                        .on_mouse_down(gpui::MouseButton::Left, move |_, _, cx| {
                                            view.update(cx, |this, cx| {
                                                this.load_library_version(&lib_id, &ver_name, cx);
                                            });
                                        })
                                        .child(div().text_color(rgb(0xcdd6f4)).text_xs().child("Load"))
                                )
                            }
                        })
                )
                .into_any_element()
        }
        SidebarItem::MessageHeader { name, id, is_can, is_expanded, ch_id } => {
            let ch_id = *ch_id;
            let msg_id = *id;
            let is_expanded = *is_expanded;
            let arrow = if is_expanded { "▾" } else { "▸" };

            div()
                .px_3()
                .py_0p5()
                .bg(rgb(0x111112))
                .flex()
                .items_center()
                .gap_2()
                .cursor_pointer()
                .hover(|s| s.bg(rgb(0x1a1a1b)))
                .on_mouse_down(gpui::MouseButton::Left, {
                    let view = view.clone();
                    move |_, _, cx| {
                        view.update(cx, |this, cx| {
                            this.toggle_message_expanded(ch_id, msg_id);
                            cx.notify();
                        });
                    }
                })
                .child(
                    div()
                        .w(px(10.0))
                        .text_xs()
                        .text_color(rgb(0x71717a))
                        .child(arrow)
                )
                .child(
                    div()
                        .w(px(60.0))
                        .text_xs()
                        .text_color(if *is_can { rgb(0x89b4fa) } else { rgb(0xf9e2af) })
                        .child(format!("0x{:X}", id))
                )
                .child(
                    div()
                        .flex_1()
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(rgb(0xd4d4d8))
                        .child(name.clone())
                )
                .into_any_element()
        }
        SidebarItem::SignalItem { name, id, size, is_selected, is_can, .. } => {
            let sig_id = id.clone();
            let is_selected = *is_selected;
            let size = *size;
            let is_can = *is_can;

            div()
                .px_4()
                .py_1()
                .flex()
                .items_center()
                .gap_2()
                .hover(|s| s.bg(rgb(0x1a1a1b)))
                .child(
                    div()
                        .w(px(12.0))
                        .h(px(12.0))
                        .rounded(px(2.0))
                        .border_1()
                        .border_color(if is_selected {
                            if is_can { rgb(0x3b82f6) } else { rgb(0xeab308) }
                        } else {
                            rgb(0x3f3f46)
                        })
                        .bg(if is_selected {
                            if is_can { rgb(0x3b82f6) } else { rgb(0xeab308) }
                        } else {
                            rgba(0x00000000)
                        })
                        .cursor_pointer()
                        .on_mouse_down(MouseButton::Left, {
                            let view = view.clone();
                            move |_, _, cx| {
                                view.update(cx, |this, cx| {
                                    if let Some(pos) = this.selected_signals.iter().position(|s| s == &sig_id) {
                                        this.selected_signals.remove(pos);
                                    } else {
                                        this.selected_signals.push(sig_id.clone());
                                    }
                                    cx.notify();
                                });
                            }
                        })
                )
                .child(
                    div()
                        .flex_1()
                        .text_xs()
                        .text_color(if is_selected { rgb(0xffffff) } else { rgb(0xa1a1aa) })
                        .child(name.clone())
                )
                .child(
                    div()
                        .text_color(rgb(0x52525b))
                        .text_xs()
                        .child(format!("{}b", size))
                )
                .into_any_element()
        }
    }
}

// Stubs — replaced in Tasks 4 and 6.
pub fn extract_signal_items(_app: &CanViewApp) -> Vec<SidebarItem> {
    Vec::new()
}

pub fn render_signal_sidebar(
    _window: &mut Window,
    _app: &CanViewApp,
    _view: Entity<CanViewApp>,
    _cx: &mut Context<CanViewApp>,
) -> impl IntoElement {
    div().size_full()
}
