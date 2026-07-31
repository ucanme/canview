//! Plot sidebar — channel/message/signal tree with fold state.
//!
//! Migrated from `chart_view.rs`. The pure `extract_signal_items` function
//! builds the flattened, filtered list of sidebar items (unit-testable).
//! `render_signal_sidebar` wraps it in a `uniform_list`.

use crate::app::CanViewerApp;
use crate::models::ChannelMapping;
use gpui::prelude::*;
use gpui::*;

/// Items produced by `extract_signal_items` and consumed by `render_sidebar_item`.
#[derive(Clone, Debug)]
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
    // Spec-mandated; ch_id/msg_id used for future channel/message lookup.
    #[allow(dead_code)]
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
pub fn render_sidebar_item(item: &SidebarItem, view: Entity<CanViewerApp>) -> AnyElement {
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
                .h(px(24.0))
                .flex_shrink_0()
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
                .h(px(24.0))
                .flex_shrink_0()
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
                .h(px(24.0))
                .flex_shrink_0()
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
                                    // Manual edit: clear active set binding so the dropdown reverts to "Select a set…"
                                    this.active_signal_set = None;
                                    // Live plot: re-extract and update chart immediately on every selection change
                                    crate::ui::views::chart_view::extract_and_update_series_data(this);
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

/// Build the flattened, filtered list of sidebar items.
///
/// Pure function: reads only `&CanViewerApp`, no cx / window / side effects.
/// Search filter non-empty → force-expands matching channels/messages
/// without modifying `expanded_channels` / `expanded_messages`.
pub fn extract_signal_items(app: &CanViewerApp) -> Vec<SidebarItem> {
    let mut items = Vec::new();
    let filter_text = app.signal_filter_text.to_lowercase();
    let force_expand = !filter_text.is_empty();

    // Count selected signals per channel (for the ChannelHeader badge)
    let mut selected_by_channel: std::collections::HashMap<u16, usize> = std::collections::HashMap::new();
    for sig_id in &app.selected_signals {
        let parts: Vec<&str> = sig_id.split(':').collect();
        if let Some(ch) = parts.get(1).and_then(|s| s.parse::<u16>().ok()) {
            *selected_by_channel.entry(ch).or_insert(0) += 1;
        }
    }

    // === Loaded CAN (DBC) channels ===
    let mut dbc_keys: Vec<u16> = app.dbc_channels.keys().copied().collect();
    dbc_keys.sort();
    for ch_id in &dbc_keys {
        let ch_id = *ch_id;
        if let Some(dbc) = app.dbc_channels.get(&ch_id) {
            let ch_name = format!("Channel {} (CAN)", ch_id);
            let manual_expanded = app.expanded_channels.contains(&ch_id);
            let is_expanded = manual_expanded || force_expand;

            let mut channel_items: Vec<SidebarItem> = Vec::new();
            let mut channel_has_matches = filter_text.is_empty();

            let mut messages: Vec<_> = dbc.messages.values().collect();
            messages.sort_by_key(|m| m.id);

            for msg in messages {
                let matches_msg = msg.name.to_lowercase().contains(&filter_text)
                    || format!("0x{:x}", msg.id).to_lowercase().contains(&filter_text);
                let matching_signals: Vec<_> = msg.signals.values()
                    .filter(|s| s.name.to_lowercase().contains(&filter_text))
                    .collect();

                if matches_msg || !matching_signals.is_empty() {
                    channel_has_matches = true;
                    let msg_expanded =
                        app.expanded_messages.contains(&(ch_id, msg.id)) || force_expand;
                    channel_items.push(SidebarItem::MessageHeader {
                        name: msg.name.clone(),
                        id: msg.id,
                        is_can: true,
                        is_expanded: msg_expanded,
                        ch_id,
                    });

                    if msg_expanded {
                        let mut signals: Vec<_> = matching_signals.into_iter().collect();
                        signals.sort_by_key(|s| s.start_bit);
                        for sig in signals {
                            if filter_text.is_empty()
                                || sig.name.to_lowercase().contains(&filter_text)
                                || matches_msg
                            {
                                let signal_id = format!("CAN:{}:{}:{}", ch_id, msg.id, sig.name);
                                channel_items.push(SidebarItem::SignalItem {
                                    name: sig.name.clone(),
                                    id: signal_id.clone(),
                                    size: sig.signal_size,
                                    is_selected: app.selected_signals.contains(&signal_id),
                                    is_can: true,
                                    ch_id,
                                    msg_id: msg.id,
                                });
                            }
                        }
                    }
                }
            }

            if channel_has_matches {
                items.push(SidebarItem::ChannelHeader {
                    name: ch_name,
                    ch_id,
                    is_can: true,
                    is_loaded: true,
                    mapping: None,
                    is_expanded,
                    selected_count: selected_by_channel.get(&ch_id).copied().unwrap_or(0),
                });
                if is_expanded {
                    items.extend(channel_items);
                }
            }
        }
    }

    // === Loaded LIN (LDF) channels ===
    let mut ldf_keys: Vec<u16> = app.ldf_channels.keys().copied().collect();
    ldf_keys.sort();
    for ch_id in &ldf_keys {
        let ch_id = *ch_id;
        if let Some(ldf) = app.ldf_channels.get(&ch_id) {
            let ch_name = format!("Channel {} (LIN)", ch_id);
            let manual_expanded = app.expanded_channels.contains(&ch_id);
            let is_expanded = manual_expanded || force_expand;

            let mut channel_items: Vec<SidebarItem> = Vec::new();
            let mut channel_has_matches = filter_text.is_empty();

            let mut frames: Vec<_> = ldf.frames.values().collect();
            frames.sort_by_key(|f| f.id);

            for frame in frames {
                let matches_frame = frame.name.to_lowercase().contains(&filter_text)
                    || format!("0x{:x}", frame.id).to_lowercase().contains(&filter_text);
                let matching_signals: Vec<_> = frame.signals.iter()
                    .filter(|s| s.signal_name.to_lowercase().contains(&filter_text))
                    .collect();

                if matches_frame || !matching_signals.is_empty() {
                    channel_has_matches = true;
                    let msg_expanded =
                        app.expanded_messages.contains(&(ch_id, frame.id)) || force_expand;
                    channel_items.push(SidebarItem::MessageHeader {
                        name: frame.name.clone(),
                        id: frame.id,
                        is_can: false,
                        is_expanded: msg_expanded,
                        ch_id,
                    });

                    if msg_expanded {
                        for mapping in &frame.signals {
                            if filter_text.is_empty()
                                || mapping.signal_name.to_lowercase().contains(&filter_text)
                                || matches_frame
                            {
                                let signal_id = format!("LIN:{}:{}:{}", ch_id, frame.id, mapping.signal_name);
                                let sig_size = ldf.signals.get(&mapping.signal_name).map(|s| s.size).unwrap_or(0);
                                channel_items.push(SidebarItem::SignalItem {
                                    name: mapping.signal_name.clone(),
                                    id: signal_id.clone(),
                                    size: sig_size,
                                    is_selected: app.selected_signals.contains(&signal_id),
                                    is_can: false,
                                    ch_id,
                                    msg_id: frame.id,
                                });
                            }
                        }
                    }
                }
            }

            if channel_has_matches {
                items.push(SidebarItem::ChannelHeader {
                    name: ch_name,
                    ch_id,
                    is_can: false,
                    is_loaded: true,
                    mapping: None,
                    is_expanded,
                    selected_count: selected_by_channel.get(&ch_id).copied().unwrap_or(0),
                });
                if is_expanded {
                    items.extend(channel_items);
                }
            }
        }
    }

    // === Unloaded configured channels (only ChannelHeader, no children) ===
    let loaded_channels: std::collections::HashSet<u16> = items.iter()
        .filter_map(|i| if let SidebarItem::ChannelHeader { ch_id, .. } = i { Some(*ch_id) } else { None })
        .collect();

    for mapping in &app.app_config.mappings {
        if mapping.library_id.is_some()
            && mapping.version_name.is_some()
            && !loaded_channels.contains(&mapping.channel_id)
        {
            let ch_id = mapping.channel_id;
            let ch_type_str = if mapping.channel_type.is_can() { "CAN" } else { "LIN" };
            let ch_name = format!("Channel {} ({}) [Unloaded]", ch_id, ch_type_str);

            if filter_text.is_empty() || ch_name.to_lowercase().contains(&filter_text) {
                items.push(SidebarItem::ChannelHeader {
                    name: ch_name,
                    ch_id,
                    is_can: mapping.channel_type.is_can(),
                    is_loaded: false,
                    mapping: Some(mapping.clone()),
                    // Respect manual fold state + search force-expand
                    is_expanded: app.expanded_channels.contains(&ch_id) || force_expand,
                    selected_count: selected_by_channel.get(&ch_id).copied().unwrap_or(0),
                });
            }
        }
    }

    items
}

/// Items produced by `build_set_dropdown_items` for the signal-sets dropdown.
#[derive(Clone, Debug, PartialEq)]
pub enum SetDropdownItem {
    /// Disabled placeholder (no active library / no sets)
    Placeholder(String),
    /// A named set with its entry count
    Set { name: String, count: usize },
    /// "✕ Clear set selection" trailing item
    ClearActive,
}

/// Pure function: build the list of items shown in the signal-sets dropdown.
pub fn build_set_dropdown_items(app: &CanViewerApp) -> Vec<SetDropdownItem> {
    let Some(lib_id) = &app.app_config.active_library_id else {
        return vec![SetDropdownItem::Placeholder("先激活一个信号库".into())];
    };
    let Some(sets) = app.signal_set_store.sets_by_library.get(lib_id) else {
        return vec![SetDropdownItem::Placeholder("当前库无信号集".into())];
    };
    if sets.is_empty() {
        return vec![SetDropdownItem::Placeholder("当前库无信号集".into())];
    }
    let mut items: Vec<SetDropdownItem> = sets
        .iter()
        .map(|s| SetDropdownItem::Set {
            name: s.name.clone(),
            count: s.entries.len(),
        })
        .collect();
    if app.active_signal_set.is_some() {
        items.push(SetDropdownItem::ClearActive);
    }
    items
}

/// Render the signal selection sidebar: header + search box + virtualized list
/// + bottom action bar with "Clear all" and "Plot N signals" buttons.
pub fn render_signal_sidebar(
    _window: &mut Window,
    app: &CanViewerApp,
    view: Entity<CanViewerApp>,
    cx: &mut Context<CanViewerApp>,
) -> impl IntoElement {
    let items = extract_signal_items(app);
    let item_count = items.len();
    let selected_count = app.selected_signals.len();
    let items_arc = std::sync::Arc::new(items);

    div()
        .size_full()
        .flex()
        .flex_col()
        .relative()
        .bg(rgb(0x0a0a0b))
        .child(
            // Sidebar Header
            div()
                .px_4()
                .py_2()
                .bg(rgb(0x131314))
                .border_b_1()
                .border_color(rgb(0x27272a))
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(0xe4e4e7))
                                .child("信号选择 (Signals)")
                        )
                        .when_some(
                            app.active_signal_set.as_ref().and_then(|(lid, sname)| {
                                if app.app_config.active_library_id.as_ref() == Some(lid) {
                                    Some(sname.clone())
                                } else {
                                    None
                                }
                            }),
                            |this, set_name| {
                                this.child(
                                    div()
                                        .px_1p5()
                                        .py(px(1.0))
                                        .bg(rgb(0x3b82f6))
                                        .rounded(px(3.0))
                                        .text_xs()
                                        .text_color(rgb(0xffffff))
                                        .child(set_name)
                                )
                            }
                        )
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(0x71717a))
                        .child(format!("{}", item_count))
                )
        )
        .child(render_signal_set_dropdown_trigger(app, view.clone(), cx))
        .child(
            // Search Box
            div()
                .w_full()
                .px_4()
                .py_2()
                .border_b_1()
                .border_color(rgb(0x27272a))
                .flex()
                .items_center()
                .child(
                    if let Some(input) = &app.signal_search_input {
                        div()
                            .flex_1()
                            .h(px(32.0))
                            .flex()
                            .items_center()
                            .child(gpui_component::input::Input::new(input).appearance(true))
                            .into_any_element()
                    } else {
                        div()
                            .flex_1()
                            .h(px(32.0))
                            .flex()
                            .items_center()
                            .px_2()
                            .text_xs()
                            .text_color(rgb(0x888888))
                            .child("Search signals...")
                            .into_any_element()
                    }
                )
        )
        .child(
            // Virtualized list
            div()
                .flex_1()
                .child(
                    if item_count == 0 {
                        div()
                            .p_4()
                            .text_xs()
                            .text_color(rgb(0x52525b))
                            .text_center()
                            .child("No matches found")
                            .into_any_element()
                    } else {
                        let view_entity = view.clone();
                        gpui::uniform_list(
                            "signal-list",
                            item_count,
                            move |range, _window, _cx| {
                                let items = items_arc.clone();
                                range.map(|i| render_sidebar_item(&items[i], view_entity.clone()))
                                    .collect::<Vec<_>>()
                            }
                        )
                        .size_full()
                        .into_any_element()
                    }
                )
        )
        .child(
            // Bottom Action Bar: Clear all | Plot N signals | Save as set
            if app.show_save_set_input {
                // Inline rename input row
                render_save_set_input_row(app, view.clone(), cx)
            } else if selected_count > 0 {
                div()
                    .p_2()
                    .bg(rgb(0x131314))
                    .border_t_1()
                    .border_color(rgb(0x27272a))
                    .flex()
                    .gap_2()
                    .child(
                        // Clear all button
                        div()
                            .px_3()
                            .py_1p5()
                            .bg(rgb(0x3f3f46))
                            .rounded(px(4.0))
                            .cursor_pointer()
                            .hover(|s| s.bg(rgb(0x52525b)))
                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                this.clear_selected_signals(cx);
                            }))
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(0xffffff))
                                    .child(format!("清除全部 ({})", selected_count))
                            )
                    )
                    .child(
                        // Save as signal set button (flex-1 to fill the space previously held by the Plot button)
                        div()
                            .flex_1()
                            .px_3()
                            .py_1p5()
                            .bg(rgb(0x3f3f46))
                            .rounded(px(4.0))
                            .cursor_pointer()
                            .hover(|s| s.bg(rgb(0x52525b)))
                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                this.show_save_set_input = true;
                                this.pending_signal_set_name = Some(String::new());
                                cx.notify();
                            }))
                            .child(
                                div().text_xs().text_color(rgb(0xffffff)).child("保存为信号集…")
                            )
                    )
                    .into_any_element()
            } else {
                div().into_any_element()
            }
        )
        // Signal-sets dropdown popup — appended LAST so it paints on top of
        // all other sidebar content. Returns None when closed; .children() handles it gracefully.
        .children(render_signal_set_dropdown_popup(app, view.clone(), cx))
}

/// Render the signal-sets dropdown trigger row (the clickable bar between the
/// header and search box). The popup itself is rendered separately by
/// `render_signal_set_dropdown_popup` so it can be appended as the LAST child
/// of the sidebar root — that way it paints on top of the search box, signal
/// list, and bottom action bar instead of being stacked under them.
fn render_signal_set_dropdown_trigger(
    app: &CanViewerApp,
    _view: Entity<CanViewerApp>,
    cx: &mut Context<CanViewerApp>,
) -> impl IntoElement {
    let items = build_set_dropdown_items(app);
    let is_open = app.show_signal_set_dropdown;
    let active_set_label = app.active_signal_set.as_ref()
        .and_then(|(lid, sname)| {
            if app.app_config.active_library_id.as_ref() == Some(lid) {
                Some(sname.clone())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "选择信号集…".to_string());

    let is_disabled = matches!(items.first(), Some(SetDropdownItem::Placeholder(_)));

    div()
        .w_full()
        .px_4()
        .py_2()
        .border_b_1()
        .border_color(rgb(0x27272a))
        .flex()
        .items_center()
        .gap_2()
        .when(!is_disabled, |el| el.cursor_pointer().hover(|s| s.bg(rgb(0x1a1a1b))))
        .when(is_disabled, |el| el.opacity(0.5))
        .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
            this.show_signal_set_dropdown = !this.show_signal_set_dropdown;
            cx.notify();
        }))
        .child(
            div()
                .text_xs()
                .text_color(rgb(0x71717a))
                .w(px(56.0))
                .child("信号集:")
        )
        .child(
            div()
                .flex_1()
                .text_xs()
                .text_color(if is_disabled { rgb(0x52525b) } else { rgb(0xe4e4e7) })
                .child(active_set_label.clone())
        )
        .child(
            div()
                .text_xs()
                .text_color(rgb(0x71717a))
                .child(if is_disabled { "" } else if is_open { "▴" } else { "▾" })
        )
}

/// Render the signal-sets dropdown popup overlay. Must be appended as the
/// LAST child of the sidebar root so it paints on top of all other sidebar
/// content. Returns `None` when the dropdown is closed or disabled.
fn render_signal_set_dropdown_popup(
    app: &CanViewerApp,
    view: Entity<CanViewerApp>,
    cx: &mut Context<CanViewerApp>,
) -> Option<AnyElement> {
    if !app.show_signal_set_dropdown {
        return None;
    }
    let items = build_set_dropdown_items(app);
    if matches!(items.first(), Some(SetDropdownItem::Placeholder(_))) {
        return None;
    }

    let mut children: Vec<AnyElement> = Vec::new();
    for item in items.iter() {
        let view = view.clone();
        let elem = match item {
            SetDropdownItem::Placeholder(msg) => div()
                .px_4().py_2()
                .text_xs().text_color(rgb(0x52525b))
                .child(msg.clone())
                .into_any_element(),
            SetDropdownItem::Set { name, count } => {
                let name = name.clone();
                let count = *count;
                let name_for_closure = name.clone();
                let name_for_delete = name.clone();
                div()
                    .px_4().py_2()
                    .cursor_pointer().hover(|s| s.bg(rgb(0x1f1f22)))
                    .on_mouse_down(MouseButton::Left, {
                        let view = view.clone();
                        move |_, _, cx| {
                            view.update(cx, |this, cx| {
                                let lib_id = this.app_config.active_library_id.clone().unwrap_or_default();
                                crate::controllers::signal_set_controller::apply_signal_set(
                                    this, &lib_id, &name_for_closure, cx,
                                );
                                this.show_signal_set_dropdown = false;
                                cx.notify();
                            });
                        }
                    })
                    .flex().items_center().justify_between()
                    .child(div().text_xs().text_color(rgb(0xd4d4d8)).child(name.clone()))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(div().text_xs().text_color(rgb(0x71717a)).child(format!("({})", count)))
                            .child(
                                // Delete button — stop_propagation so the row's apply click doesn't fire
                                div()
                                    .px_1()
                                    .text_xs()
                                    .text_color(rgb(0x71717a))
                                    .hover(|s| s.text_color(rgb(0xef4444)))
                                    .on_mouse_down(MouseButton::Left, {
                                        let view = view.clone();
                                        move |_, _, cx| {
                                            cx.stop_propagation();
                                            view.update(cx, |this, cx| {
                                                let lib_id = this.app_config.active_library_id.clone().unwrap_or_default();
                                                crate::controllers::signal_set_controller::delete_signal_set(
                                                    this, &lib_id, &name_for_delete, cx,
                                                );
                                                cx.notify();
                                            });
                                        }
                                    })
                                    .child("✕")
                            )
                    )
                    .into_any_element()
            }
            SetDropdownItem::ClearActive => div()
                .px_4().py_2()
                .border_t_1().border_color(rgb(0x27272a))
                .cursor_pointer().hover(|s| s.bg(rgb(0x1f1f22)))
                .on_mouse_down(MouseButton::Left, {
                    let view = view.clone();
                    move |_, _, cx| {
                        view.update(cx, |this, cx| {
                            crate::controllers::signal_set_controller::clear_active_signal_set(this, cx);
                            this.show_signal_set_dropdown = false;
                            cx.notify();
                        });
                    }
                })
                .text_xs().text_color(rgb(0xef4444))
                .child("✕ 清除当前选择")
                .into_any_element(),
        };
        children.push(elem);
    }

    Some(
        // Click-outside overlay covers the entire sidebar. Clicks on the popup
        // itself don't close (the popup's own handlers stop propagation where
        // needed); clicks anywhere else hit this overlay and close the dropdown.
        div()
            .absolute()
            .top_0().left_0()
            .size_full()
            .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                this.show_signal_set_dropdown = false;
                cx.notify();
            }))
            // Popup card positioned just below the trigger row. Sidebar header
            // is ~32px (py_2 + xs text) and the trigger row is ~32px, so top=64
            // sits the popup right below the trigger. Adjust if padding changes.
            .child(
                div()
                    .absolute()
                    .top(px(64.0))
                    .left_0()
                    .right_0()
                    .bg(rgb(0x18181b))
                    .border_1()
                    .border_color(rgb(0x27272a))
                    .rounded_b(px(4.0))
                    .shadow_lg()
                    .flex().flex_col()
                    .on_mouse_down(MouseButton::Left, |_, _, cx| {
                        // Prevent clicks on the popup card from bubbling to the
                        // click-outside overlay behind it.
                        cx.stop_propagation();
                    })
                    .children(children)
            )
            .into_any_element()
    )
}

/// Render the inline "save as signal set" input row (replaces the bottom bar
/// when show_save_set_input is true). Enter saves (via InputState subscribe);
/// 取消 button aborts. The Input entity is created lazily in impls_rendering.rs
/// render loop (Sub-step 0b) and bound here.
fn render_save_set_input_row(
    app: &CanViewerApp,
    view: Entity<CanViewerApp>,
    _cx: &mut Context<CanViewerApp>,
) -> AnyElement {
    let input_entity = app.signal_set_name_input.clone();
    let view_for_cancel = view.clone();

    div()
        .p_2()
        .bg(rgb(0x131314))
        .border_t_1()
        .border_color(rgb(0x27272a))
        .flex()
        .gap_2()
        .child(
            if let Some(input) = input_entity {
                div()
                    .flex_1()
                    .h(px(32.0))
                    .flex()
                    .items_center()
                    .child(gpui_component::input::Input::new(&input).appearance(true))
                    .into_any_element()
            } else {
                div()
                    .flex_1()
                    .h(px(32.0))
                    .flex()
                    .items_center()
                    .px_2()
                    .text_xs()
                    .text_color(rgb(0x52525b))
                    .child("初始化输入…")
                    .into_any_element()
            }
        )
        .child(
            div()
                .px_3()
                .py_1p5()
                .bg(rgb(0x3f3f46))
                .rounded(px(4.0))
                .cursor_pointer()
                .hover(|s| s.bg(rgb(0x52525b)))
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    view_for_cancel.update(cx, |this, cx| {
                        this.show_save_set_input = false;
                        this.pending_signal_set_name = None;
                        cx.notify();
                    });
                })
                .child(div().text_xs().text_color(rgb(0xffffff)).child("取消"))
        )
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ChannelMapping, ChannelType};

    /// Helper: an app with one unloaded channel mapping (no DBC loaded).
    fn app_with_unloaded_channel(ch_id: u16, ch_type: ChannelType) -> CanViewerApp {
        let mut app = CanViewerApp::new_state();
        app.app_config.mappings.push(ChannelMapping {
            channel_type: ch_type,
            channel_id: ch_id,
            path: String::new(),
            description: String::new(),
            library_id: Some("lib1".to_string()),
            version_name: Some("v1.0".to_string()),
        });
        app
    }

    /// App with active library `lib_x` and a given list of (set_name, count) sets under it.
    fn app_with_sets(active_lib_id: Option<&str>, sets: Vec<(&str, usize)>, active_set: Option<(&str, &str)>) -> CanViewerApp {
        let mut app = CanViewerApp::new_state();
        if let Some(id) = active_lib_id {
            app.app_config.active_library_id = Some(id.to_string());
            let store_sets: Vec<_> = sets.into_iter().map(|(name, n)| crate::library::signal_sets::SignalSet {
                name: name.to_string(),
                entries: (0..n).map(|i| crate::library::signal_sets::SignalSetEntry {
                    channel_id: 1, msg_id: i as u32, signal_name: format!("sig{}", i),
                }).collect(),
            }).collect();
            app.signal_set_store.sets_by_library.insert(id.to_string(), store_sets);
        }
        if let Some((lid, sname)) = active_set {
            app.active_signal_set = Some((lid.to_string(), sname.to_string()));
        }
        app
    }

    #[test]
    fn build_set_dropdown_items_no_active_library() {
        let app = app_with_sets(None, Vec::new(), None);
        let items = build_set_dropdown_items(&app);
        assert_eq!(items.len(), 1);
        match &items[0] {
            SetDropdownItem::Placeholder(msg) => assert_eq!(msg, "先激活一个信号库"),
            other => panic!("expected Placeholder, got {:?}", other),
        }
    }

    #[test]
    fn build_set_dropdown_items_active_lib_no_sets() {
        let app = app_with_sets(Some("lib_x"), Vec::new(), None);
        let items = build_set_dropdown_items(&app);
        assert_eq!(items.len(), 1);
        match &items[0] {
            SetDropdownItem::Placeholder(msg) => assert_eq!(msg, "当前库无信号集"),
            other => panic!("expected Placeholder, got {:?}", other),
        }
    }

    #[test]
    fn build_set_dropdown_items_active_lib_with_sets_no_active() {
        let app = app_with_sets(Some("lib_x"), vec![("Engine", 2), ("Battery", 3)], None);
        let items = build_set_dropdown_items(&app);
        assert_eq!(items.len(), 2);
        assert!(matches!(&items[0], SetDropdownItem::Set { name, count: 2 } if name == "Engine"));
        assert!(matches!(&items[1], SetDropdownItem::Set { name, count: 3 } if name == "Battery"));
    }

    #[test]
    fn build_set_dropdown_items_with_active_set_appends_clear() {
        let app = app_with_sets(
            Some("lib_x"),
            vec![("Engine", 2)],
            Some(("lib_x", "Engine")),
        );
        let items = build_set_dropdown_items(&app);
        assert_eq!(items.len(), 2);
        assert!(matches!(&items[0], SetDropdownItem::Set { name, .. } if name == "Engine"));
        assert!(matches!(&items[1], SetDropdownItem::ClearActive));
    }

    #[test]
    fn build_set_dropdown_items_active_set_on_other_lib_still_lists() {
        // Edge case: active_signal_set points at a non-active library.
        // build_set_dropdown_items is driven by active_library_id for listing;
        // ClearActive is appended whenever active_signal_set.is_some(), regardless of lib match.
        // (The dropdown UI itself only shows ClearActive when the active set is on the
        // currently-active library, but that's a UI concern, not this pure function's.)
        let app = app_with_sets(
            Some("lib_x"),
            vec![("Engine", 2)],
            Some(("lib_other", "OtherSet")),
        );
        let items = build_set_dropdown_items(&app);
        assert_eq!(items.len(), 2);
        assert!(matches!(&items[1], SetDropdownItem::ClearActive));
    }

    #[test]
    fn extract_signal_items_empty_state_shows_all_channel_headers() {
        let app = app_with_unloaded_channel(1, ChannelType::CAN);
        let items = extract_signal_items(&app);
        // No filter, no expansion → exactly 1 ChannelHeader, no children
        assert_eq!(items.len(), 1);
        // Verify it's a ChannelHeader with the right ch_id
        if let SidebarItem::ChannelHeader { ch_id, is_expanded, .. } = &items[0] {
            assert_eq!(*ch_id, 1);
            assert!(!*is_expanded);
        } else {
            panic!("expected ChannelHeader, got {:?}", items[0]);
        }
    }

    #[test]
    fn extract_signal_items_search_expands_matching_channel() {
        let mut app = app_with_unloaded_channel(1, ChannelType::CAN);
        app.signal_filter_text = "channel 1".into(); // matches "Channel 1 (CAN) [Unloaded]"
        let items = extract_signal_items(&app);
        // Search matches → channel is_expanded=true (forced), still no children because no DBC
        if let SidebarItem::ChannelHeader { is_expanded, .. } = &items[0] {
            assert!(*is_expanded, "search should force-expand matching channel");
        } else {
            panic!("expected ChannelHeader");
        }
    }

    #[test]
    fn extract_signal_items_search_no_match_hides_channel() {
        let mut app = app_with_unloaded_channel(1, ChannelType::CAN);
        app.signal_filter_text = "xyznomatch".into();
        let items = extract_signal_items(&app);
        assert!(items.is_empty(), "non-matching filter should hide the channel");
    }

    #[test]
    fn extract_signal_items_search_clear_restores_manual_state() {
        let mut app = app_with_unloaded_channel(1, ChannelType::CAN);
        // Manually expand channel 1 (no accordion concern, only 1 channel)
        app.toggle_channel_expanded(1);
        // Search with non-matching text → channel disappears
        app.signal_filter_text = "xyznomatch".into();
        let items = extract_signal_items(&app);
        assert!(items.is_empty());
        // Clear search → channel reappears, expanded (manual state preserved)
        app.signal_filter_text = "".into();
        let items = extract_signal_items(&app);
        if let SidebarItem::ChannelHeader { is_expanded, .. } = &items[0] {
            assert!(*is_expanded, "manual expand state should be restored after search clear");
        }
    }

    #[test]
    fn extract_signal_items_selected_count_in_header() {
        let mut app = app_with_unloaded_channel(1, ChannelType::CAN);
        // Even without a DBC loaded, the ChannelHeader reports selected_count
        // computed from selected_signals matching "CAN:1:..." or "LIN:1:..."
        app.selected_signals.push("CAN:1:0x100:EngineSpeed".to_string());
        app.selected_signals.push("CAN:1:0x200:RPM".to_string());
        app.selected_signals.push("CAN:2:0x100:Other".to_string()); // different channel
        let items = extract_signal_items(&app);
        if let SidebarItem::ChannelHeader { selected_count, .. } = &items[0] {
            assert_eq!(*selected_count, 2, "channel 1 should report 2 selected signals");
        }
    }
}
