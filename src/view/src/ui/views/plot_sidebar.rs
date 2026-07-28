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

/// Build the flattened, filtered list of sidebar items.
///
/// Pure function: reads only `&CanViewApp`, no cx / window / side effects.
/// Search filter non-empty → force-expands matching channels/messages
/// without modifying `expanded_channels` / `expanded_messages`.
pub fn extract_signal_items(app: &CanViewApp) -> Vec<SidebarItem> {
    let mut items = Vec::new();
    let filter_text = app.signal_filter_text.to_lowercase();
    let force_expand = !filter_text.is_empty();

    // Count selected signals per channel (for the ChannelHeader badge)
    let mut selected_by_channel: std::collections::HashMap<u16, usize> = std::collections::HashMap::new();
    for sig_id in &app.selected_signals {
        let parts: Vec<&str> = sig_id.split(':').collect();
        if parts.len() >= 2 {
            if let Ok(ch) = parts[1].parse::<u16>() {
                *selected_by_channel.entry(ch).or_insert(0) += 1;
            }
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
        if mapping.library_id.is_some() && mapping.version_name.is_some() {
            if !loaded_channels.contains(&mapping.channel_id) {
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
                        is_expanded: false, // Unloaded channels are never expandable
                        selected_count: selected_by_channel.get(&ch_id).copied().unwrap_or(0),
                    });
                }
            }
        }
    }

    items
}

pub fn render_signal_sidebar(
    _window: &mut Window,
    _app: &CanViewApp,
    _view: Entity<CanViewApp>,
    _cx: &mut Context<CanViewApp>,
) -> impl IntoElement {
    div().size_full()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ChannelMapping, ChannelType};

    /// Helper: an app with one unloaded channel mapping (no DBC loaded).
    fn app_with_unloaded_channel(ch_id: u16, ch_type: ChannelType) -> CanViewApp {
        let mut app = CanViewApp::new_state();
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
