//! CanViewApp Rendering Implementation
//!
//! This file contains all rendering methods for CanViewApp.
//! Separated from impls.rs to improve code organization and reduce file size.

use super::state::{AppView, CanViewApp, ScrollbarDragState};
use crate::ChannelType;
use crate::rendering::{calculate_column_widths, render_message_row_static_with_widths};
use blf::{LogObject, read_blf_from_file};
use gpui::{prelude::*, *};
use smol::Timer;
use gpui_component::input::{InputEvent, InputState};

impl CanViewApp {
    // ===== Rendering Methods =====
    // All rendering methods are organized in this section for better maintainability

    // ========== Message Rendering ==========
    fn render_message_row(&self, msg: &LogObject, index: usize) -> impl IntoElement {
        let (time_str, channel_id, msg_type, id_str, dlc_str, data_str, signals_str) = match msg {
            LogObject::CanMessage(can_msg) => {
                let timestamp = can_msg.header.object_time_stamp;
                let time_str = self.get_timestamp_string(timestamp);
                let data_hex = Self::format_data_hex(&can_msg.data, can_msg.dlc);
                let actual_data_len = can_msg.data.len().min(can_msg.dlc as usize);
                let signals = self.extract_can_signals(can_msg.channel, can_msg.id, &can_msg.data);

                (
                    time_str,
                    can_msg.channel,
                    "CAN".to_string(),
                    format!("0x{:03X}", can_msg.id),
                    actual_data_len.to_string(),
                    data_hex,
                    signals,
                )
            }
            LogObject::LinMessage(lin_msg) => {
                let timestamp = lin_msg.header.object_time_stamp;
                let time_str = self.get_timestamp_string(timestamp);
                let data_hex = Self::format_data_hex(&lin_msg.data, lin_msg.dlc);
                let actual_data_len = lin_msg.data.len().min(lin_msg.dlc as usize);
                let signals = self.extract_lin_signals(lin_msg.channel, lin_msg.id, &lin_msg.data);

                (
                    time_str,
                    lin_msg.channel,
                    "LIN".to_string(),
                    format!("0x{:02X}", lin_msg.id),
                    actual_data_len.to_string(),
                    data_hex,
                    signals,
                )
            }
            _ => (
                "Unknown".to_string(),
                0,
                "Other".to_string(),
                "-".to_string(),
                "-".to_string(),
                "-".to_string(),
                String::new(),
            ),
        };

        let bg_color = Self::get_zebra_bg_color(index);

        div()
            .flex()
            .w_full()
            .min_h(px(24.)) // Slightly taller for better readability
            .bg(bg_color)
            .border_b_1()
            .border_color(rgb(0x2a2a2a)) // Semi-transparent border like Zed
            .items_center()
            .text_sm() // Slightly larger text like Zed
            .text_color(rgb(0xcdd6f4)) // Zed's default text color
            .hover(|style| style.bg(rgb(0x1f1f1f))) // Subtle hover like Zed
            .cursor_pointer()
            .child(
                div()
                    .w(px(100.))
                    .px_3()
                    .py_1()
                    .text_color(rgb(0x646473)) // Zed's muted color
                    .child(time_str),
            )
            .child(
                div()
                    .w(px(40.))
                    .px_2()
                    .py_1()
                    .text_color(rgb(0x7dcfff)) // Zed's blue
                    .child(channel_id.to_string()),
            )
            .child(
                div()
                    .w(px(50.))
                    .px_2()
                    .py_1()
                    .text_color(rgb(0xa6e3a1)) // Zed's green
                    .child(msg_type),
            )
            .child(
                div()
                    .w(px(70.))
                    .px_2()
                    .py_1()
                    .text_color(rgb(0xf9e2af)) // Zed's yellow
                    .child(id_str),
            )
            .child(div().w(px(40.)).px_2().py_1().child(dlc_str))
            .child(
                div()
                    .w(px(150.))
                    .px_2()
                    .py_1()
                    .text_color(rgb(0xb4befe)) // Zed's purple
                    .child(data_str),
            )
            .child(
                div()
                    .flex_1()
                    .px_2()
                    .py_1()
                    .text_color(rgb(0x9399b2)) // Zed's comment color
                    .child(signals_str),
            )
    }

    fn render_library_view(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        use crate::ui::views::library_management::render_library_management_view;

        // TODO: wire up FilterBar for Library view in follow-up commit.
        // render_library_management_view takes &LibraryManager (not &CanViewApp),
        // so wiring requires either threading an Entity<CanViewApp> through its
        // signature or wrapping the call here with a FilterBar child added to
        // this outer div. Deferred to keep this commit's surface small.

        // Safety check: prevent stack overflow from invalid state
        let libraries = self.library_manager.libraries();
        if libraries.len() > 1000 {
            eprintln!("Warning: Too many libraries ({}), limiting to prevent stack overflow", libraries.len());
            // Return empty view if too many libraries
            return gpui::div()
                .flex_1()
                .size_full()
                .bg(rgb(0x0a0a0a))
                .child(gpui::div().px_4().py_2().text_sm().text_color(rgb(0xff0000))
                    .child(format!("Error: Too many libraries ({}). Please remove some libraries.", libraries.len())));
        }

        // Initialize input states if needed (only do this once)
        // Note: We can't create InputState here without window, so we'll handle it differently
        // The Input components will be created lazily when needed

        gpui::div()
            .flex_1()
            .size_full()
            .child(render_library_management_view(
                libraries,
                &self.selected_library_id,
                &self.selected_version_id, // Add selected version ID
                &self.app_config.mappings,
                self.show_library_dialog
                    && self.library_dialog_type == super::state::LibraryDialogType::Create,
                self.show_version_input,
                &self.new_library_name,
                &self.new_version_name,
                &None, // focused_library_input is deprecated
                self.library_cursor_position,
                self.new_version_cursor_position,
                self.library_name_input.as_ref(),
                self.version_name_input.as_ref(),
                self.show_add_channel_input,
                self.channel_id_input.as_ref(),
                self.channel_name_input.as_ref(),
                self.channel_db_path_input.as_ref(),
                &self.new_channel_db_path, // Add this parameter
                self.new_channel_type,     // Add channel type parameter
                self.server_handle.is_some(), // is_sharing
                self.copied_channel_id,       // copied_channel_id
                self.active_library_id.as_deref(),
                self.active_version_name.as_deref(),
                self.renaming_library_id.as_deref(),
                self.rename_library_input.as_ref(),
                self.renaming_version_name.as_deref(),
                self.rename_version_input.as_ref(),
                cx,
            ))
    }

    /// Filter messages based on ID and channel filters
    ///
    /// Helper method to apply both ID and channel filters to the message list.
    /// This is extracted from render_log_view to reduce complexity.
    fn filter_messages(&self) -> Vec<(usize, LogObject)> {
        match (self.id_filter, self.channel_filter) {
            (None, None) => self
                .messages
                .iter()
                .enumerate()
                .map(|(i, m)| (i, m.clone()))
                .collect(),
            (Some(filter_id), None) => self
                .messages
                .iter()
                .enumerate()
                .filter(|(_, msg)| match msg {
                    LogObject::CanMessage(can_msg) => can_msg.id == filter_id,
                    LogObject::CanMessage2(can_msg) => can_msg.id == filter_id,
                    LogObject::CanFdMessage(fd_msg) => fd_msg.id == filter_id,
                    LogObject::CanFdMessage64(fd_msg) => fd_msg.id == filter_id,
                    LogObject::LinMessage(lin_msg) => lin_msg.id as u32 == filter_id,
                    LogObject::LinMessage2(_) => false,
                    _ => false,
                })
                .map(|(i, m)| (i, m.clone()))
                .collect(),
            (None, Some(filter_channel)) => self
                .messages
                .iter()
                .enumerate()
                .filter(|(_, msg)| match msg {
                    LogObject::CanMessage(can_msg) => can_msg.channel == filter_channel,
                    LogObject::CanMessage2(can_msg) => can_msg.channel == filter_channel,
                    LogObject::CanFdMessage(fd_msg) => fd_msg.channel == filter_channel,
                    LogObject::CanFdMessage64(fd_msg) => fd_msg.channel as u16 == filter_channel,
                    LogObject::LinMessage(lin_msg) => lin_msg.channel == filter_channel,
                    LogObject::LinMessage2(_) => false,
                    _ => false,
                })
                .map(|(i, m)| (i, m.clone()))
                .collect(),
            (Some(filter_id), Some(filter_channel)) => self
                .messages
                .iter()
                .enumerate()
                .filter(|(_, msg)| match msg {
                    LogObject::CanMessage(can_msg) => {
                        can_msg.id == filter_id && can_msg.channel == filter_channel
                    }
                    LogObject::CanMessage2(can_msg) => {
                        can_msg.id == filter_id && can_msg.channel == filter_channel
                    }
                    LogObject::CanFdMessage(fd_msg) => {
                        fd_msg.id == filter_id && fd_msg.channel == filter_channel
                    }
                    LogObject::CanFdMessage64(fd_msg) => {
                        fd_msg.id == filter_id && fd_msg.channel as u16 == filter_channel
                    }
                    LogObject::LinMessage(lin_msg) => {
                        lin_msg.id as u32 == filter_id && lin_msg.channel == filter_channel
                    }
                    LogObject::LinMessage2(_) => false,
                    _ => false,
                })
                .map(|(i, m)| (i, m.clone()))
                .collect(),
        }
    }
    fn render_log_view(&self, view: Entity<CanViewApp>) -> impl IntoElement {
        // Clone view for use in multiple closures
        let view_clone1 = view.clone();
        let view_clone2 = view.clone();

        // Apply filters (both ID and Channel) using helper method
        let filtered_messages = self.filter_messages();

        // Save filtered message count BEFORE filtered_messages is moved
        let filtered_count = filtered_messages.len();

        let dbc_channels = self.dbc_channels.clone();
        let ldf_channels = self.ldf_channels.clone();
        let start_time = self.start_time;
        let scroll_handle = self.list_scroll_handle.clone();
        let id_display_decimal = self.id_display_decimal;
        let id_filter = self.id_filter;
        let id_filter_text = self.id_filter_text.clone();

        // Calculate column widths based on ALL messages (not filtered), to keep layout consistent
        let (time_width, ch_width, type_width, id_width, dlc_width) =
            calculate_column_widths(&self.messages, &dbc_channels, &ldf_channels, start_time);

        // Clone view for use in event handlers
        let view_for_mouse_move = view.clone();
        let view_for_mouse_down = view.clone();
        let view_for_mouse_up = view.clone();
        let view_for_scrollbar = view.clone();
        let view_for_keyboard = view.clone();

        // Clone for dialog display
        let _id_filter_text_for_dialog = id_filter_text.clone();

        div()
            .size_full()
            .flex()
            .flex_col()
            .relative()  // Add relative positioning for absolute children
            // Handle keyboard input for ID filter
            .on_key_down(move |event, _window, cx| {
                eprintln!("Global on_key_down: keystroke={}", event.keystroke);
                let view = view_for_keyboard.clone();
                let keystroke_str = format!("{}", event.keystroke);

                // Check if filter box is active
                let show_filter = view.read(cx).show_id_filter_input;

                // If filter box is active, handle input for it
                if show_filter {
                    match keystroke_str.as_str() {
                        "backspace" => {
                            view.update(cx, |app, cx| {
                                let mut text = app.id_filter_text.to_string();
                                if !text.is_empty() {
                                    text.pop();
                                    app.id_filter_text = text.into();
                                    eprintln!("  Filter text (backspace): {}", app.id_filter_text);
                                    cx.notify();
                                }
                            });
                            return;
                        }
                        "escape" => {
                            view.update(cx, |app, cx| {
                                app.show_id_filter_input = false;
                                eprintln!("  Filter box closed (escape)");
                                cx.notify();
                            });
                            return;
                        }
                        "enter" => {
                            view.update(cx, |app, cx| {
                                if let Ok(parsed_id) = u32::from_str_radix(app.id_filter_text.as_ref(), 10) {
                                    if !app.id_filter_text.is_empty() {
                                        app.id_filter = Some(parsed_id);
                                    }
                                }
                                app.show_id_filter_input = false;
                                eprintln!("  Filter applied (enter): id={:?}", app.id_filter);
                                cx.notify();
                            });
                            return;
                        }
                        _ => {
                            if keystroke_str.len() == 1 {
                                if let Some(ch) = keystroke_str.chars().next() {
                                    if ch.is_ascii_digit() {
                                        view.update(cx, |app, cx| {
                                            let mut text = app.id_filter_text.to_string();
                                            text.push(ch);
                                            app.id_filter_text = text.into();
                                            eprintln!("  Filter text: {}", app.id_filter_text);
                                            cx.notify();
                                        });
                                        return;
                                    }
                                }
                            }
                        }
                    }
                    return;
                }

                // Handle global shortcuts
                match keystroke_str.as_str() {
                    "backspace" => {
                        view.update(cx, |app, cx| {
                            let mut text = app.id_filter_text.to_string();
                            if !text.is_empty() {
                                text.pop();
                                let new_text = text.clone();
                                app.id_filter_text = text.into();

                                if new_text.is_empty() {
                                    app.id_filter = None;
                                } else if let Ok(parsed_id) = u32::from_str_radix(&new_text, 10) {
                                    app.id_filter = Some(parsed_id);
                                } else {
                                    app.id_filter = None;
                                }
                                cx.notify();
                            }
                        });
                    }
                    "escape" => {
                        view.update(cx, |app, cx| {
                            app.id_filter = None;
                            app.id_filter_text = "".into();
                            cx.notify();
                        });
                    }
                    _ => {
                        if keystroke_str.len() == 1 {
                            let ch = keystroke_str.chars().next().unwrap();
                            if ch.is_ascii_digit() {
                                view.update(cx, |app, cx| {
                                    let mut text = app.id_filter_text.to_string();
                                    text.push(ch);
                                    let new_text = text.clone();
                                    app.id_filter_text = text.into();

                                    if let Ok(parsed_id) = u32::from_str_radix(&new_text, 10) {
                                        app.id_filter = Some(parsed_id);
                                    }
                                    cx.notify();
                                });
                            }
                        }
                    }
                }
            })
            // Global mouse move handler for scrollbar dragging
            .on_mouse_move(move |event, _window, cx| {
                let drag_state = view_for_mouse_move.read(cx).scrollbar_drag_state.as_ref();
                let Some(drag) = drag_state else {
                    return;
                };

                // Check if left mouse button is still pressed
                // If not, clear the drag state to prevent ghost dragging
                if event.pressed_button != Some(MouseButton::Left) {
                    view_for_mouse_move.update(cx, |app, _cx| {
                        app.scrollbar_drag_state = None;
                    });
                    return;
                }

                let current_y = event.position.y;
                let container_h = view_for_mouse_move.read(cx).list_container_height;
                let row_h = 22.0;

                // Use filtered message count from drag state
                let filtered_count = drag.filtered_count;
                let total_content_height = filtered_count as f32 * row_h;
                let max_scroll_offset = (total_content_height - container_h).max(0.0);

                if max_scroll_offset <= 0.0 {
                    return;
                }

                // Calculate thumb dimensions with dynamic minimum size
                let thumb_ratio = (container_h / total_content_height).min(1.0);

                // Use same dynamic minimum thumb size
                let min_thumb_size = if filtered_count > 100 {
                    15.0
                } else if filtered_count > 50 {
                    20.0
                } else {
                    30.0
                };

                let thumb_h = (thumb_ratio * container_h).max(min_thumb_size);
                let track_h = (container_h - thumb_h).max(0.0);

                // Calculate thumb position based on mouse Y
                // Convert start_scroll_offset to thumb position at drag start
                let start_thumb_top = if max_scroll_offset > 0.0 {
                    (drag.start_scroll_offset / max_scroll_offset) * track_h
                } else {
                    0.0
                };

                // Calculate new thumb top based on mouse movement
                let delta_y = f32::from(current_y - drag.start_y);
                let new_thumb_top = (start_thumb_top + delta_y).clamp(0.0, track_h);

                // Convert thumb position back to scroll offset
                let scroll_progress = new_thumb_top / track_h;
                let new_scroll_offset = (scroll_progress * max_scroll_offset).clamp(0.0, max_scroll_offset);

                // Convert to item index based on FILTERED messages
                let visible_items = (container_h / row_h).ceil() as usize;
                let max_start_index = filtered_count.saturating_sub(visible_items);

                // Calculate target index based on scroll offset
                let target_index = ((new_scroll_offset / row_h).round() as usize).clamp(0, max_start_index);

                // Use Bottom strategy only when we're at the very end
                // This ensures the last row is visible at the bottom
                if target_index >= max_start_index.saturating_sub(1) {
                    view_for_mouse_move.read(cx).list_scroll_handle.scroll_to_item_strict(
                        filtered_count.saturating_sub(1),
                        gpui::ScrollStrategy::Bottom
                    );
                } else {
                    view_for_mouse_move.read(cx).list_scroll_handle.scroll_to_item_strict(target_index, gpui::ScrollStrategy::Top);
                }
                cx.notify(view_for_mouse_move.entity_id());
            })
            // Global mouse down handler
            .on_mouse_down(MouseButton::Left, move |_event, _window, cx| {
                view_for_mouse_down.update(cx, |app, _cx| {
                    eprintln!("Global mouse_down: show_id={}, show_ch={}",
                        app.show_id_filter_input, app.show_channel_filter_input);
                });
            })
            // Global mouse up handler - this will catch mouse up anywhere
            .on_mouse_up(MouseButton::Left, move |_event, _window, cx| {
                // Always clear drag state on mouse up, anywhere in the window
                view_for_mouse_up.update(cx, |app, _cx| {
                    app.scrollbar_drag_state = None;

                    // NOTE: Dropdown closing is now handled by the overlay, not here
                    // This prevents the dropdown from being closed immediately after opening
                    eprintln!("Global mouse_up: show_id={}, show_ch={}",
                        app.show_id_filter_input, app.show_channel_filter_input);
                });
                eprintln!("🔍 mouse_up: active_drag={}",
                    if cx.has_active_drag() { "Some" } else { "None" });
            })
            .child(
                // TODO: remove once FilterBar dropdowns are wired up
                // Zed-style header with calculated column widths and proper alignment
                div()
                    .w_full()
                    .h(px(28.))
                    .bg(rgb(0x1f1f1f))
                    .border_b_1()
                    .border_color(rgb(0x2a2a2a))
                    .flex()
                    .items_center()
                    .text_xs()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(rgb(0x9ca3af))
                    .child(
                        div()
                            .w(px(60.))
                            .px_3()
                            .py_1()
                            .flex()
                            .items_center()
                            .flex_shrink_0()
                            .whitespace_nowrap()
                            .overflow_hidden()
                            .child("#")
                    )
                    .child(
                        div()
                            .w(time_width)
                            .px_3()
                            .py_1()
                            .flex()
                            .items_center()
                            .flex_shrink_0()
                            .whitespace_nowrap()
                            .overflow_hidden()
                            .child("TIME")
                    )
                    .child(
                        {
                            let _view_for_ch_filter = view.clone();
                            div()
                                .w(ch_width)
                                .px_2()
                                .py_1()
                                .flex()
                                .items_center()
                                .flex_shrink_0()
                                .whitespace_nowrap()
                                .overflow_hidden()
                                .child("CH")
                                .child(
                                    div()
                                        .text_xs()
                                        .cursor_pointer()
                                        .text_color(if self.channel_filter.is_some() {
                                            rgb(0x60a5fa)
                                        } else {
                                            rgb(0x4b5563)
                                        })
                                        .hover(|style| style.bg(rgb(0x374151)))
                                        .rounded(px(2.))
                                        .ml_0p5()  // Small left margin to bring it closer to CH
                                        .pl_0()  // No left padding
                                        .pr_0()  // No right padding
                                        .py_0p5()
                                        .on_mouse_down(gpui::MouseButton::Left, {
                                            let view = view.clone();
                                            move |_event, _window, cx| {
                                                view.update(cx, |app, cx| {
                                                    // If filter is active, clicking clears it
                                                    // If filter is not active, clicking shows dropdown
                                                    if app.channel_filter.is_some() {
                                                        eprintln!("Clearing channel filter");
                                                        app.channel_filter = None;
                                                        app.channel_filter_text = "".into();
                                                        app.show_channel_filter_input = false;
                                                    } else {
                                                        eprintln!("Before: show_channel_filter_input={}", app.show_channel_filter_input);
                                                        app.show_channel_filter_input = !app.show_channel_filter_input;
                                                        eprintln!("After: show_channel_filter_input={}",
                                                            app.show_channel_filter_input);
                                                    }
                                                    cx.notify();
                                                });
                                            }
                                        })
                                        .child(if self.channel_filter.is_some() { "✓" } else { "⚙" })
                                )
                        }
                    )
                    .child(
                        div()
                            .w(type_width)
                            .px_2()
                            .py_1()
                            .flex()
                            .items_center()
                            .flex_shrink_0()
                            .whitespace_nowrap()
                            .overflow_hidden()
                            .child("TYPE")
                    )
                    .child(
                        div()
                            .w(id_width)
                            .pl_2()  // Only left padding
                            .pr_0()  // No right padding
                            .py_1()
                            .flex()
                            .items_center()
                            .flex_shrink_0()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .child(
                                        div()
                                            .cursor_pointer()
                                            .rounded(px(2.))
                                            .pl_1()  // Left padding only
                                            .pr_0()  // No right padding
                                            .py_0p5()
                                            .hover(|style| style.bg(rgb(0x374151)))
                                            .on_mouse_down(gpui::MouseButton::Left, {
                                                let view = view.clone();
                                                move |_, _, cx| {
                                                    view.update(cx, |app, cx| {
                                                        app.id_display_decimal = !app.id_display_decimal;
                                                        cx.notify();
                                                    });
                                                }
                                            })
                                            .child(
                                                div()
                                                    .flex()
                                                    .items_center()
                                                    .gap_0p5()
                                                    .child("ID")
                                                    .child(
                                                        div()
                                                            .text_xs()
                                                            .text_color(rgb(0x6b7280))
                                                            .child(if id_display_decimal { "10" } else { "16" })
                                                    )
                                            )
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .cursor_pointer()
                                            .text_color(if id_filter.is_some() {
                                                rgb(0x60a5fa)
                                            } else {
                                                rgb(0x4b5563)
                                            })
                                            .hover(|style| style.bg(rgb(0x374151)))
                                            .rounded(px(2.))
                                            .pl_1()  // Left padding only
                                            .pr_0()  // No right padding
                                            .py_0p5()
                                            .on_mouse_down(gpui::MouseButton::Left, {
                                                let view = view.clone();
                                                move |event, _, cx| {
                                                    eprintln!("Gear clicked! Position: {:?}", event.position);
                                                    view.update(cx, |app, cx| {
                                                        // If filter is active, clicking clears it
                                                        // If filter is not active, clicking shows dropdown
                                                        if app.id_filter.is_some() {
                                                            eprintln!("Clearing filter");
                                                            app.id_filter = None;
                                                            app.id_filter_text = "".into();
                                                            app.show_id_filter_input = false;
                                                        } else {
                                                            eprintln!("Before: show_id_filter_input={}", app.show_id_filter_input);
                                                            app.show_id_filter_input = !app.show_id_filter_input;
                                                            eprintln!("After: show_id_filter_input={}",
                                                                app.show_id_filter_input);
                                                        }
                                                        cx.notify();
                                                    });
                                                }
                                            })
                                            .child(if id_filter.is_some() { "✓" } else { "⚙" })
                                    )
                            )
                    )
                    .child(
                        div()
                            .w(dlc_width)
                            .px_2()
                            .py_1()
                            .flex()
                            .items_center()
                            .flex_shrink_0()
                            .whitespace_nowrap()
                            .overflow_hidden()
                            .child("DLC")
                    )
                    .child(
                        div()
                            .flex_1()  // DATA列使用flex_1()占据剩余空间
                            .px_2()
                            .py_1()
                            .flex()
                            .items_center()
                            .whitespace_nowrap()
                            .child("DATA")
                    ),
            )
            .child(
                // Content area with simple list
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .relative()
                    // Show placeholder when no messages
                    .when(self.messages.is_empty(), |parent| {
                        parent.child(
                            div()
                                .flex_1()
                                .flex()
                                .items_center()
                                .justify_center()
                                .child(
                                    div()
                                        .text_lg()
                                        .text_color(rgb(0x6b7280))
                                        .child("No messages loaded. Click '📂 Open BLF' to load a file.")
                                )
                        )
                    })
                    // Show messages - always use uniform_list for better performance
                    .when(!filtered_messages.is_empty(), |parent| {
                        let display_count = filtered_messages.len();
                        let view_entity = view.clone();

                        parent.child(
                            gpui::uniform_list(
                                "message-list",
                                display_count,
                                move |range: std::ops::Range<usize>, _window: &mut gpui::Window, cx: &mut gpui::App| {
                                    // Track scroll position by observing the visible range
                                    let first_visible = range.start;
                                    view_entity.update(cx, |v, _cx| {
                                        v.scroll_offset = px(first_visible as f32 * 22.0);
                                    });

                                    range
                                        .map(|index| {
                                            if let Some((orig_idx, msg)) = filtered_messages.get(index) {
                                                let selected = view_entity.read(cx).selected_row_index == Some(*orig_idx);
                                                render_message_row_static_with_widths(
                                                    msg,
                                                    *orig_idx,
                                                    time_width,
                                                    ch_width,
                                                    type_width,
                                                    id_width,
                                                    dlc_width,
                                                    &dbc_channels,
                                                    &ldf_channels,
                                                    start_time,
                                                    id_display_decimal,
                                                    view_entity.read(cx).show_id_filter_input,
                                                    view_entity.clone(),
                                                    selected,
                                                )
                                            } else {
                                                div().into_any_element()
                                            }
                                        })
                                        .collect::<Vec<_>>()
                                }
                            )
                            .track_scroll(&scroll_handle)
                            .flex_1()
                        )
                    })
                    .child({
                        // Calculate scrollbar dimensions based on FILTERED content
                        let row_height = 22.0;
                        let total_height = filtered_count as f32 * row_height;
                        let container_height = self.list_container_height;

                        // Smooth thumb height calculation - thumb represents proportion of visible content
                        let thumb_height_ratio = if total_height > 0.0 {
                            (container_height / total_height).min(1.0)
                        } else {
                            1.0
                        };

                        let max_scroll = (total_height - container_height).max(0.0);

                        // Improved dynamic minimum thumb size - scales smoothly with content
                        // Use a logarithmic scale for better UX across all dataset sizes
                        let min_thumb_size = if filtered_count <= 10 {
                            container_height  // Show full height for very small lists
                        } else if filtered_count <= 50 {
                            container_height * 0.5  // At least half visible for small lists
                        } else if filtered_count <= 200 {
                            40.0  // Reasonable minimum for medium lists
                        } else if filtered_count <= 1000 {
                            25.0  // Smaller for large lists
                        } else {
                            15.0  // Minimum for very large lists (still usable)
                        };

                        // Calculate thumb height with smooth transition
                        let ideal_thumb_height = thumb_height_ratio * container_height;
                        let thumb_height = ideal_thumb_height.max(min_thumb_size).min(container_height);
                        let thumb_height_px = px(thumb_height);

                        // Calculate scrollable track height (container minus thumb)
                        let track_height = (container_height - thumb_height).max(0.0);

                        // Calculate thumb position based on current scroll offset
                        let current_scroll_offset = f32::from(self.scroll_offset);
                        let thumb_top = if max_scroll > 0.0 && track_height > 0.0 {
                            // For very large datasets, scroll_offset may not reach max_scroll
                            // when using ScrollStrategy::Bottom. So we clamp the ratio.
                            let scroll_progress = (current_scroll_offset / max_scroll).min(1.0).max(0.0);

                            // Check if we're at the actual bottom
                            let container_h = self.list_container_height;
                            let row_h = 22.0_f32;
                            let visible_items = (container_h / row_h).ceil() as usize;
                            let max_start_index = filtered_count.saturating_sub(visible_items);
                            let current_start_index = (current_scroll_offset / row_h).round() as usize;

                            // If we're at the last page, force thumb to bottom
                            // This ensures the thumb visually reaches the end
                            if current_start_index >= max_start_index.saturating_sub(5) {
                                track_height
                            } else {
                                scroll_progress * track_height
                            }
                        } else {
                            0.0
                        };
                        let thumb_top_px = px(thumb_top);

                        let scroll_handle_clone = scroll_handle.clone();
                        let view_for_scrollbar_inner = view_for_scrollbar.clone();
                        let view_for_scroll_track = view_for_scrollbar.clone();

                        // Scrollbar container
                        div()
                            .absolute()
                            .right_0()
                            .top_0()
                            .bottom_0()  // Match the actual list container height
                            .w(px(12.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .bg(rgb(0x1a1a1a))
                            .child(
                                // Scrollbar track (clickable area)
                                div()
                                    .size_full()
                                    .relative()
                                    .on_mouse_down(gpui::MouseButton::Left, move |event, _window, cx| {
                                        let raw_click_y = f32::from(event.position.y);
                                        let offset_to_list = 84.0;
                                        let container_h = view_for_scroll_track.read(cx).list_container_height;
                                        let row_h = row_height;

                                        if filtered_count == 0 {
                                            return;
                                        }

                                        // Calculate thumb dimensions based on FILTERED messages with dynamic minimum size
                                        let total_content_height = filtered_count as f32 * row_h;
                                        let thumb_ratio = (container_h / total_content_height).min(1.0);

                                        // Use same improved minimum thumb size calculation as rendering
                                        let min_thumb_size = if filtered_count <= 10 {
                                            container_h
                                        } else if filtered_count <= 50 {
                                            container_h * 0.5
                                        } else if filtered_count <= 200 {
                                            40.0
                                        } else if filtered_count <= 1000 {
                                            25.0
                                        } else {
                                            15.0
                                        };

                                        let thumb_h = (thumb_ratio * container_h).max(min_thumb_size).min(container_h);
                                        let track_h = (container_h - thumb_h).max(0.0);

                                        // Adjust click position to be relative to container
                                        let click_y = (raw_click_y - offset_to_list).clamp(0.0, container_h);

                                        if track_h <= 0.0 {
                                            return;
                                        }

                                        // Calculate where thumb top should be based on click position
                                        // The click_y is in range [0, container_h], but thumb top can only be in [0, track_h]
                                        // When click_y is at bottom (container_h), thumb_top should be at track_h
                                        let scroll_ratio = click_y / container_h;
                                        let _desired_thumb_top = (scroll_ratio * track_h).clamp(0.0, track_h);

                                        // Calculate target index based on FILTERED messages
                                        let visible_items = (container_h / row_h).ceil() as usize;
                                        let max_start_index = filtered_count.saturating_sub(visible_items);

                                        let target_index = if max_start_index > 0 {
                                            (scroll_ratio * max_start_index as f32).round() as usize
                                        } else {
                                            0
                                        }.clamp(0, max_start_index);

                                        // Use Bottom strategy only when we're at the very end
                                        // This ensures the last row is visible at the bottom
                                        if target_index >= max_start_index.saturating_sub(1) {
                                            scroll_handle_clone.scroll_to_item_strict(
                                                filtered_count.saturating_sub(1),
                                                gpui::ScrollStrategy::Bottom
                                            );
                                        } else {
                                            scroll_handle_clone.scroll_to_item_strict(target_index, gpui::ScrollStrategy::Top);
                                        }
                                        cx.notify(view_for_scroll_track.entity_id());
                                    })
                                    .child(
                                        // Thumb with drag functionality
                                        div()
                                            .w(px(8.))
                                            .h(thumb_height_px)
                                            .top(thumb_top_px)
                                            .absolute()
                                            .bg(rgb(0x6a6a6a))
                                            .rounded(px(4.))
                                            .hover(|style| style.bg(rgb(0x7a7a7a)))
                                            .cursor_grab()
                                            .on_mouse_down(gpui::MouseButton::Left, {
                                                let view_for_thumb = view_for_scrollbar_inner.clone();
                                                move |event, _window, cx| {
                                                    cx.stop_propagation();
                                                    // Initialize drag state
                                                    let start_y = event.position.y;
                                                    let start_scroll_offset = f32::from(view_for_thumb.read(cx).scroll_offset);

                                                    // Set drag state
                                                    view_for_thumb.update(cx, |app, _cx| {
                                                    app.scrollbar_drag_state = Some(ScrollbarDragState {
                                                        start_y,
                                                        start_scroll_offset,
                                                        filtered_count,
                                                    });
                                                });

                                            }
                                            })
                                    )
                            )
                    })
            )
            // Full-screen overlay to catch clicks outside dropdown
            .when(self.show_id_filter_input || self.show_channel_filter_input, |parent| {
                let view_for_overlay = view.clone();
                parent.child(
                    div()
                        .absolute()
                        .inset_0()
                        .on_mouse_down(gpui::MouseButton::Left, move |_event, _window, cx| {
                            eprintln!("Overlay clicked - closing dropdowns");
                            view_for_overlay.update(cx, |app, cx| {
                                app.show_id_filter_input = false;
                                app.show_channel_filter_input = false;
                                cx.notify();
                            });
                        })
                )
            })
            // Filter dropdown - SHOW ALL IDs WITH SCROLL
            .when(self.show_id_filter_input, |parent| {
                // Calculate ALL unique IDs from messages
                let mut unique_ids = std::collections::HashSet::new();
                for msg in self.messages.iter() {  // Scan ALL messages
                    match msg {
                        LogObject::CanMessage(m) => { unique_ids.insert(m.id); }
                        LogObject::CanMessage2(m) => { unique_ids.insert(m.id); }
                        LogObject::CanFdMessage(m) => { unique_ids.insert(m.id); }
                        LogObject::CanFdMessage64(m) => { unique_ids.insert(m.id); }
                        LogObject::LinMessage(m) => { unique_ids.insert(m.id as u32); }
                        _ => {}
                    }
                }
                let mut id_list: Vec<u32> = unique_ids.into_iter().collect();
                id_list.sort();

                let filter_left = 60.0 + f32::from(time_width) + f32::from(ch_width) + f32::from(type_width) + f32::from(id_width) - 40.0;

                eprintln!("=== Filter dropdown rendering ===");
                eprintln!("  Found {} unique IDs", id_list.len());

                parent.child(
                    {
                        let id_list_clone = id_list.clone();
                        let view_for_scroll = view.clone();
                        let id_list_for_wheel = id_list.clone();
                        // Clone the scroll handle for use in closures
                        let filter_scroll_handle = self.filter_scroll_handle.clone();
                        let filter_scroll_handle_for_uniform = filter_scroll_handle.clone();

                        // Outer wrapper to catch and stop scroll event propagation
                        div()
                            .absolute()
                            .left(px(filter_left))
                            .top(px(32.))
                            .w(px(150.))
                            .h(px(300.))
                            .on_scroll_wheel(move |_event, _window, cx| {
                                // Stop propagation - don't let scroll events reach parent list
                                cx.stop_propagation();
                            })
                            .child(
                                div()
                                    .w_full()
                                    .h_full()
                                    .bg(rgb(0x1f2937))
                                    .border_1()
                                    .border_color(rgb(0x3b82f6))
                                    .rounded(px(4.))
                                    .shadow_lg()
                                    .flex()
                                    .flex_col()
                                    .overflow_hidden()  // Important: clip content
                                    // Track mouse enter/leave to know when we're over the dropdown
                                    // Don't handle mouse_down/mouse_up - let them bubble to parent
                                    .on_mouse_move({
                                        let view_for_scroll = view_for_scroll.clone();
                                        move |_event, _window, cx| {
                                            view_for_scroll.update(cx, |app, cx| {
                                                app.mouse_over_filter_dropdown = true;
                                                cx.notify();
                                            });
                                        }
                                    })
                                    // Capture wheel events at container level and manually scroll
                                    .on_scroll_wheel(move |event, _window, cx| {
                                        cx.stop_propagation();

                                        // Calculate scroll delta
                                        let delta_y = match event.delta {
                                            gpui::ScrollDelta::Lines(point) => point.y * 24.0,
                                            gpui::ScrollDelta::Pixels(pixels) => f32::from(pixels.y),
                                        };

                                        // Get current scroll offset
                                        let current_offset = view_for_scroll.read(cx).filter_scroll_offset;
                                        let current_offset_f32 = f32::from(current_offset);

                                        // Calculate new scroll position
                                        let row_height = 20.0;
                                        let total_items = id_list_for_wheel.len();
                                        let container_height = 300.0;
                                        let total_height = total_items as f32 * row_height;
                                        let max_scroll = (total_height - container_height).max(0.0);

                                        let new_offset = (current_offset_f32 - delta_y).clamp(0.0, max_scroll);

                                        // Update state
                                        view_for_scroll.update(cx, |app, cx| {
                                            app.filter_scroll_offset = px(new_offset);
                                            cx.notify();
                                        });

                                        // Manually scroll the uniform_list using the persistent handle
                                        let target_index = ((new_offset / row_height).round() as usize)
                                            .clamp(0, total_items.saturating_sub(1));

                                        filter_scroll_handle.scroll_to_item_strict(
                                            target_index,
                                            gpui::ScrollStrategy::Top
                                        );

                                        eprintln!("Manual scroll: delta={:.2}, offset={:.2} -> {:.2}, index={}",
                                            delta_y, current_offset_f32, new_offset, target_index);
                                    })
                                    .child(
                                uniform_list(
                                    "filter-dropdown",
                                    id_list_clone.len(),
                                    move |range: std::ops::Range<usize>, _window: &mut gpui::Window, _cx: &mut gpui::App| {
                                        range
                                            .map(|index| {
                                                let id = id_list_clone[index];
                                                div()
                                                    .w_full()
                                                    .h(px(20.))
                                                    .flex()
                                                    .items_center()
                                                    .px_3()
                                                    .text_sm()
                                                    .line_height(px(20.))
                                                    .text_color(rgb(0xffffff))
                                                    .hover(|style| style.bg(rgb(0x374151)))
                                                    .cursor_pointer()
                                                    // Block all mouse events from propagating to the main list
                                                    .on_mouse_move(move |_event, _window, _cx| {
                                                    })
                                                    .on_mouse_up(gpui::MouseButton::Left, move |_event, _window, _cx| {
                                                    })
                                                    .on_mouse_down(gpui::MouseButton::Left, {
                                                        let view = view_clone1.clone();
                                                        move |_event, _window, cx| {
                                                            cx.stop_propagation();
                                                            eprintln!("Selected ID: {}", id);
                                                            view.update(cx, |app, cx| {
                                                                app.id_filter = Some(id);
                                                                app.id_filter_text = id.to_string().into();
                                                                app.show_id_filter_input = false;
                                                                app.mouse_over_filter_dropdown = false;  // Reset hover flag
                                                                cx.notify();
                                                            });
                                                        }
                                                    })
                                                    .child(format!("ID: {}", id))
                                                    .into_any_element()
                                            })
                                            .collect::<Vec<_>>()
                                    },
                                )
                                .track_scroll(&filter_scroll_handle_for_uniform)
                                .flex_1()
                            )  // End of inner div
                            )  // End of outer wrapper div
                    }
                )
            })
            // Channel filter dropdown
            .when(self.show_channel_filter_input, |parent| {
                // Calculate ALL unique channels from messages
                let mut unique_channels = std::collections::HashSet::new();
                for msg in self.messages.iter() {
                    match msg {
                        LogObject::CanMessage(m) => { unique_channels.insert(m.channel); }
                        LogObject::CanMessage2(m) => { unique_channels.insert(m.channel); }
                        LogObject::CanFdMessage(m) => { unique_channels.insert(m.channel); }
                        LogObject::CanFdMessage64(m) => { unique_channels.insert(m.channel as u16); }
                        LogObject::LinMessage(m) => { unique_channels.insert(m.channel); }
                        LogObject::LinMessage2(_) => {}
                        _ => {}
                    }
                }
                let mut channel_list: Vec<u16> = unique_channels.into_iter().collect();
                channel_list.sort();

                let filter_left = 60.0 + f32::from(time_width) + 10.0; // Position after TIME column

                eprintln!("=== Channel filter dropdown rendering ===");
                eprintln!("  Found {} unique channels", channel_list.len());

                parent.child(
                    {
                        let channel_list_clone = channel_list.clone();
                        let view_for_scroll = view.clone();
                        let channel_list_for_wheel = channel_list.clone();
                        // Clone the scroll handle for use in closures
                        let filter_scroll_handle = self.channel_filter_scroll_handle.clone();
                        let filter_scroll_handle_for_uniform = filter_scroll_handle.clone();

                        // Outer wrapper to catch and stop scroll event propagation
                        div()
                            .absolute()
                            .left(px(filter_left))
                            .top(px(32.))
                            .w(px(150.))
                            .h(px(300.))
                            .on_scroll_wheel(move |_event, _window, cx| {
                                // Stop propagation - don't let scroll events reach parent list
                                cx.stop_propagation();
                            })
                            .child(
                                div()
                                    .w_full()
                                    .h_full()
                                    .bg(rgb(0x1f2937))
                                    .border_1()
                                    .border_color(rgb(0x3b82f6))
                                    .rounded(px(4.))
                                    .shadow_lg()
                                    .flex()
                                    .flex_col()
                                    .overflow_hidden()
                                    // Track mouse enter/leave to know when we're over the dropdown
                                    // Don't handle mouse_down/mouse_up - let them bubble to parent
                                    .on_mouse_move({
                                        let view_for_scroll = view_for_scroll.clone();
                                        move |_event, _window, cx| {
                                            view_for_scroll.update(cx, |app, cx| {
                                                app.mouse_over_filter_dropdown = true;
                                                cx.notify();
                                            });
                                        }
                                    })
                                    // Capture wheel events at container level and manually scroll
                                    .on_scroll_wheel(move |event, _window, cx| {
                                        cx.stop_propagation();

                                        // Calculate scroll delta
                                        let delta_y = match event.delta {
                                            gpui::ScrollDelta::Lines(point) => point.y * 24.0,
                                            gpui::ScrollDelta::Pixels(pixels) => f32::from(pixels.y),
                                        };

                                        // Get current scroll offset
                                        let current_offset = view_for_scroll.read(cx).channel_filter_scroll_offset;
                                        let current_offset_f32 = f32::from(current_offset);

                                        // Calculate new scroll position
                                        let row_height = 20.0;
                                        let total_items = channel_list_for_wheel.len();
                                        let container_height = 300.0;
                                        let total_height = total_items as f32 * row_height;
                                        let max_scroll = (total_height - container_height).max(0.0);

                                        let new_offset = (current_offset_f32 - delta_y).clamp(0.0, max_scroll);

                                        // Update state
                                        view_for_scroll.update(cx, |app, cx| {
                                            app.channel_filter_scroll_offset = px(new_offset);
                                            cx.notify();
                                        });

                                        // Manually scroll the uniform_list using the persistent handle
                                        let target_index = ((new_offset / row_height).round() as usize)
                                            .clamp(0, total_items.saturating_sub(1));

                                        filter_scroll_handle.scroll_to_item_strict(
                                            target_index,
                                            gpui::ScrollStrategy::Top
                                        );

                                        eprintln!("Channel filter scroll: delta={:.2}, offset={:.2} -> {:.2}, index={}",
                                            delta_y, current_offset_f32, new_offset, target_index);
                                    })
                                    .child(
                                uniform_list(
                                    "channel-filter-dropdown",
                                    channel_list_clone.len(),
                                    move |range: std::ops::Range<usize>, _window: &mut gpui::Window, _cx: &mut gpui::App| {
                                        range
                                            .map(|index| {
                                                let channel = channel_list_clone[index];
                                                div()
                                                    .w_full()
                                                    .h(px(20.))
                                                    .flex()
                                                    .items_center()
                                                    .px_3()
                                                    .text_sm()
                                                    .line_height(px(20.))
                                                    .text_color(rgb(0xffffff))
                                                    .hover(|style| style.bg(rgb(0x374151)))
                                                    .cursor_pointer()
                                                    // Block all mouse events from propagating to the main list
                                                    .on_mouse_move(move |_event, _window, _cx| {
                                                    })
                                                    .on_mouse_up(gpui::MouseButton::Left, move |_event, _window, _cx| {
                                                    })
                                                    .on_mouse_down(gpui::MouseButton::Left, {
                                                        let view = view_clone2.clone();
                                                        move |_event, _window, cx| {
                                                            cx.stop_propagation();
                                                            eprintln!("Selected Channel: {}", channel);
                                                            view.update(cx, |app, cx| {
                                                                app.channel_filter = Some(channel);
                                                                app.channel_filter_text = channel.to_string().into();
                                                                app.show_channel_filter_input = false;
                                                                app.mouse_over_filter_dropdown = false;  // Reset hover flag
                                                                cx.notify();
                                                            });
                                                        }
                                                    })
                                                    .child(format!("CH: {}", channel))
                                                    .into_any_element()
                                            })
                                            .collect::<Vec<_>>()
                                    },
                                )
                                .track_scroll(&filter_scroll_handle_for_uniform)
                                .flex_1()
                            )  // End of inner div
                            )  // End of outer wrapper div
                    }
                )
            })
    }

    #[allow(dead_code)]
    fn render_channel_filter_dropdown(
        &self,
        parent: gpui::Div,
        view: Entity<CanViewApp>,
        _ch_width: gpui::Pixels,
        time_width: gpui::Pixels,
    ) -> gpui::Div {
        parent.when(self.show_channel_filter_input, |parent| {
            // Calculate ALL unique channels from messages
            let mut unique_channels = std::collections::HashSet::new();
            for msg in self.messages.iter() {
                match msg {
                    LogObject::CanMessage(m) => {
                        unique_channels.insert(m.channel);
                    }
                    LogObject::CanMessage2(m) => {
                        unique_channels.insert(m.channel);
                    }
                    LogObject::CanFdMessage(m) => {
                        unique_channels.insert(m.channel);
                    }
                    LogObject::CanFdMessage64(m) => {
                        unique_channels.insert(m.channel as u16);
                    }
                    LogObject::LinMessage(m) => {
                        unique_channels.insert(m.channel);
                    }
                    LogObject::LinMessage2(_) => {}
                    _ => {}
                }
            }
            let mut channel_list: Vec<u16> = unique_channels.into_iter().collect();
            channel_list.sort();

            let filter_left = 60.0 + f32::from(time_width) + 10.0; // Position after TIME column

            eprintln!("=== Channel filter dropdown rendering ===");
            eprintln!("  Found {} unique channels", channel_list.len());

            parent.child({
                let channel_list_clone = channel_list.clone();
                let view_for_scroll = view.clone();
                let channel_list_for_wheel = channel_list.clone();
                // Clone the scroll handle for use in closures
                let filter_scroll_handle = self.channel_filter_scroll_handle.clone();
                let filter_scroll_handle_for_uniform = filter_scroll_handle.clone();
                let view_for_selection = view.clone();

                div()
                    .absolute()
                    .left(px(filter_left))
                    .top(px(32.))
                    .w(px(150.))
                    .h(px(300.))
                    .bg(rgb(0x1f2937))
                    .border_1()
                    .border_color(rgb(0x3b82f6))
                    .rounded(px(4.))
                    .shadow_lg()
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    // Track mouse enter/leave to know when we're over the dropdown
                    // Don't handle mouse_down/mouse_up - let them bubble to parent
                    .on_mouse_move({
                        let view_for_scroll = view_for_scroll.clone();
                        move |_event, _window, cx| {
                            view_for_scroll.update(cx, |app, cx| {
                                app.mouse_over_filter_dropdown = true;
                                cx.notify();
                            });
                        }
                    })
                    // Capture wheel events at container level and manually scroll
                    .on_scroll_wheel(move |event, _window, cx| {
                        cx.stop_propagation();

                        // Calculate scroll delta
                        let delta_y = match event.delta {
                            gpui::ScrollDelta::Lines(point) => point.y * 24.0,
                            gpui::ScrollDelta::Pixels(pixels) => f32::from(pixels.y),
                        };

                        // Get current scroll offset
                        let current_offset = view_for_scroll.read(cx).channel_filter_scroll_offset;
                        let current_offset_f32 = f32::from(current_offset);

                        // Calculate new scroll position
                        let row_height = 20.0;
                        let total_items = channel_list_for_wheel.len();
                        let container_height = 300.0;
                        let total_height = total_items as f32 * row_height;
                        let max_scroll = (total_height - container_height).max(0.0);

                        let new_offset = (current_offset_f32 - delta_y).clamp(0.0, max_scroll);

                        // Update state
                        view_for_scroll.update(cx, |app, cx| {
                            app.channel_filter_scroll_offset = px(new_offset);
                            cx.notify();
                        });

                        // Manually scroll the uniform_list using the persistent handle
                        let target_index = ((new_offset / row_height).round() as usize)
                            .clamp(0, total_items.saturating_sub(1));

                        filter_scroll_handle
                            .scroll_to_item_strict(target_index, gpui::ScrollStrategy::Top);

                        eprintln!(
                            "Channel filter scroll: delta={:.2}, offset={:.2} -> {:.2}, index={}",
                            delta_y, current_offset_f32, new_offset, target_index
                        );
                    })
                    .child(
                        uniform_list(
                            "channel-filter-dropdown",
                            channel_list_clone.len(),
                            move |range: std::ops::Range<usize>,
                                  _window: &mut gpui::Window,
                                  _cx: &mut gpui::App| {
                                range
                                    .map(|index| {
                                        let channel = channel_list_clone[index];
                                        div()
                                            .w_full()
                                            .h(px(20.))
                                            .flex()
                                            .items_center()
                                            .px_3()
                                            .text_sm()
                                            .line_height(px(20.))
                                            .text_color(rgb(0xffffff))
                                            .hover(|style| style.bg(rgb(0x374151)))
                                            .cursor_pointer()
                                            // Block all mouse events from propagating to the main list
                                            .on_mouse_move(move |_event, _window, _cx| {
                                            })
                                            .on_mouse_up(
                                                gpui::MouseButton::Left,
                                                move |_event, _window, _cx| {
                                                },
                                            )
                                            .on_mouse_down(gpui::MouseButton::Left, {
                                                let view = view_for_selection.clone();
                                                move |_event, _window, cx| {
                                                    cx.stop_propagation();
                                                    eprintln!("Selected Channel: {}", channel);
                                                    view.update(cx, |app: &mut CanViewApp, cx: &mut Context<CanViewApp>| {
                                                        app.channel_filter = Some(channel);
                                                        app.channel_filter_text =
                                                            channel.to_string().into();
                                                        app.show_channel_filter_input = false;
                                                        app.mouse_over_filter_dropdown = false; // Reset hover flag
                                                        cx.notify();
                                                    });
                                                }
                                            })
                                            .child(format!("CH: {}", channel))
                                            .into_any_element()
                                    })
                                    .collect::<Vec<_>>()
                            },
                        )
                        .track_scroll(&filter_scroll_handle_for_uniform)
                        .flex_1(),
                    )
            })
        })
    }

    /// Convert BLF timestamp to seconds based on object_flags
    fn convert_timestamp_to_seconds(timestamp: u64, flags: u32) -> f64 {
        if flags & 0x01 != 0 {
            // TimeTenMics: 10 microseconds per tick
            timestamp as f64 / 100_000.0
        } else {
            // TimeOneNans (default): 1 nanosecond per tick
            timestamp as f64 / 1_000_000_000.0
        }
    }
    fn render_config_view(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .p_6()
            .flex()
            .flex_col()
            .gap_4()
            .text_color(rgb(0xd1d5db))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_lg()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(rgb(0xffffff))
                            .child("Configuration"),
                    )
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .child({
                                use crate::ui::components::{Button, ButtonSize, ButtonVariant};
                                Button::new("Import Database")
                                    .size(ButtonSize::Small)
                                    .variant(ButtonVariant::Secondary)
                                    .build()
                                    .on_mouse_down(gpui::MouseButton::Left, {
                                        let view = cx.entity().clone();
                                        move |_event, _window, cx| {
                                            view.update(cx, |this, cx| {
                                                this.import_database_file(cx);
                                            });
                                        }
                                    })
                            })
                            .child({
                                use crate::ui::components::{Button, ButtonSize, ButtonVariant};
                                Button::new("Save Config")
                                    .size(ButtonSize::Small)
                                    .variant(ButtonVariant::Secondary)
                                    .build()
                                    .on_mouse_down(gpui::MouseButton::Left, {
                                        let view = cx.entity().clone();
                                        move |_event, _window, cx| {
                                            view.update(cx, |this, cx| {
                                                this.save_config(cx);
                                            });
                                        }
                                    })
                            }),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .bg(rgb(0x1f1f1f))
                    .border_1()
                    .border_color(rgb(0x2a2a2a))
                    .rounded(px(8.))
                    .flex()
                    .flex_col()
                    .gap_2()
                    .p_4()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(rgb(0xffffff))
                            .child("Channel Mappings"),
                    )
                    .child(div().flex_1().flex().flex_col().gap_2().children(
                        self.app_config.mappings.iter().map(|mapping| {
                            div()
                                .p_3()
                                .bg(rgb(0x374151))
                                .rounded(px(4.))
                                .flex()
                                .items_center()
                                .justify_between()
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .child(
                                            div()
                                                .text_sm()
                                                .font_weight(FontWeight::MEDIUM)
                                                .text_color(rgb(0xffffff))
                                                .child(format!(
                                                    "Channel {} ({})",
                                                    mapping.channel_id,
                                                    if mapping.channel_type == ChannelType::CAN {
                                                        "CAN"
                                                    } else {
                                                        "LIN"
                                                    }
                                                )),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(rgb(0x9ca3af))
                                                .child(mapping.path.clone()),
                                        ),
                                )
                        }),
                    )),
            )
            .child(
                // Status bar
                div()
                    .p_4()
                    .bg(rgb(0x1f1f1f))
                    .border_1()
                    .border_color(rgb(0x2a2a2a))
                    .rounded(px(8.))
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(rgb(0xffffff))
                            .child("System Status"),
                    )
                    .child(
                        div()
                            .flex()
                            .gap_4()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0x9ca3af))
                                    .child(format!("Messages: {}", self.messages.len())),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0x9ca3af))
                                    .child(format!("DBC: {}", self.dbc_channels.len())),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0x9ca3af))
                                    .child(format!("LIN: {}", self.ldf_channels.len())),
                            ),
                    ),
            )
    }
}

impl Render for CanViewApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Update container height based on current window size
        self.update_container_height(window);

        // Consume pending focus from add-channel Enter-key chain (set by PressEnter subscribe)
        if let Some(target) = self.pending_add_channel_focus.take() {
            use crate::app::PendingAddChannelFocus;
            match target {
                // Enter on channel_id → focus the channel_name input
                PendingAddChannelFocus::ChannelName => {
                    if let Some(name_input) = &self.channel_name_input {
                        name_input.update(cx, |state, cx| state.focus(window, cx));
                    }
                }
                // Enter on channel_name → user decided "let user choose next step"
                // (no auto-focus on ✓ Confirm button — user picks Browse or ✓ manually).
                // Re-focus the name input so it's not lost; the user then either clicks
                // "Select File..." or tabs to ✓ to submit.
                PendingAddChannelFocus::ChannelConfirm => {
                    if let Some(name_input) = &self.channel_name_input {
                        name_input.update(cx, |state, cx| state.focus(window, cx));
                    }
                }
            }
        }

        // Initialize import URL input if showing import dialog
        if self.show_import_dialog && self.import_url_input.is_none() {
            let input = cx.new(|cx| InputState::new(window, cx).placeholder("http://...?token=..."));
            cx.subscribe(&input, |this, input, event, cx| {
                if let InputEvent::Change = event {
                    this.import_url = input.read(cx).text().to_string();
                }
            })
            .detach();
            self.import_url_input = Some(input);
        }

        // Initialize signal search input if needed
        if self.signal_search_input.is_none() {
            let input = cx
                .new(|cx| InputState::new(window, cx).placeholder("查询信号/消息/ID (Search...)"));

            // 使用 observe 来监听输入状态的任何变化，而不仅仅是 Change 事件
            // CRITICAL: Prevent infinite rendering loop by using String comparison
            cx.observe(&input, |this, entity, cx| {
                let val = entity.read(cx).value().to_string();
                let current = this.signal_filter_text.to_string();
                // Use String comparison instead of SharedString to be absolutely sure
                if current != val {
                    eprintln!("🔍 Signal filter changed: '{}' -> '{}'", current, val);
                    this.signal_filter_text = val.into();
                    cx.notify();
                } else {
                    eprintln!("🔍 Signal filter unchanged: '{}'", current);
                }
            })
            .detach();

            self.signal_search_input = Some(input);
        }

        // Check for file dialog result (non-blocking poll)
        self.handle_file_dialog_result(cx);

        // Poll for import completion
        if self.poll_import(cx) {
            cx.notify();
        }

        let view = cx.entity().clone();

        // Pull Dock-icon drops into pending_drop_paths, then drain if no
        // load is in flight.
        crate::handlers::drag_drop::drain_dock_drop_queue(self);
        crate::handlers::drag_drop::drain_pending_drop(self, cx);

        div()
            .size_full()
            .flex()
            .flex_col()
            .on_key_down({
                let view = view.clone();
                move |event, _window, cx| {
                    eprintln!("=== ROOT LEVEL on_key_down ===");
                    eprintln!("keystroke: {}", event.keystroke);
                    eprintln!(
                        "show_id_filter_input: {}",
                        view.read(cx).show_id_filter_input
                    );

                    let keystroke_str = format!("{}", event.keystroke);

                    // Handle library dialog input
                    if keystroke_str.as_str() == "enter" {
                        let show_library_dialog = view.read(cx).show_library_dialog;
                        if show_library_dialog {
                            eprintln!("📥 Enter pressed in library dialog");

                            // Read input value BEFORE entering update block to avoid nested update conflict
                            let library_name = view
                                .read(cx)
                                .library_name_input
                                .as_ref()
                                .map(|i| i.read(cx).value().to_string())
                                .unwrap_or_default();

                            view.update(cx, |app, cx| {
                                eprintln!(
                                    "⏎ Creating library from ROOT handler: '{}'",
                                    library_name
                                );

                                if !library_name.trim().is_empty() {
                                    app.new_library_name = library_name.clone();
                                    app.create_library(cx);
                                }

                                // Close the dialog
                                app.show_library_dialog = false;
                                app.library_name_input = None;
                                cx.notify();
                            });
                            return;
                        }

                        // Handle version input
                        let show_version_input = view.read(cx).show_version_input;
                        if show_version_input {
                            eprintln!("📥 Enter pressed in version input");

                            // Read input value BEFORE entering update block to avoid nested update conflict
                            let version_name = view
                                .read(cx)
                                .version_name_input
                                .as_ref()
                                .map(|input| input.read(cx).value().to_string())
                                .unwrap_or_default();

                            view.update(cx, |app, cx| {
                                // Store the version name before calling add_library_version
                                app.new_version_name = version_name.clone();

                                eprintln!("⏎ Adding version from ROOT handler: '{}'", version_name);
                                app.add_library_version(cx);

                                // Close the input
                                app.show_version_input = false;
                                app.version_name_input = None;
                                cx.notify();
                            });
                            return;
                        }
                    }

                    // Only handle when filter is active
                    let show_filter = view.read(cx).show_id_filter_input;
                    if show_filter {
                        let keystroke_str = format!("{}", event.keystroke);
                        match keystroke_str.as_str() {
                            "backspace" => {
                                view.update(cx, |app, cx| {
                                    let mut text = app.id_filter_text.to_string();
                                    if !text.is_empty() {
                                        text.pop();
                                        app.id_filter_text = text.into();
                                        eprintln!(
                                            "Filter text (backspace): {}",
                                            app.id_filter_text
                                        );
                                        cx.notify();
                                    }
                                });
                            }
                            "escape" => {
                                view.update(cx, |app, cx| {
                                    app.show_id_filter_input = false;
                                    eprintln!("Filter closed (escape)");
                                    cx.notify();
                                });
                            }
                            "enter" => {
                                view.update(cx, |app, cx| {
                                    if let Ok(parsed_id) =
                                        u32::from_str_radix(app.id_filter_text.as_ref(), 10)
                                    {
                                        if !app.id_filter_text.is_empty() {
                                            app.id_filter = Some(parsed_id);
                                        }
                                    }
                                    app.show_id_filter_input = false;
                                    eprintln!("Filter applied (enter): id={:?}", app.id_filter);
                                    cx.notify();
                                });
                            }
                            _ => {
                                if keystroke_str.len() == 1 {
                                    if let Some(ch) = keystroke_str.chars().next() {
                                        if ch.is_ascii_digit() {
                                            view.update(cx, |app, cx| {
                                                let mut text = app.id_filter_text.to_string();
                                                text.push(ch);
                                                app.id_filter_text = text.into();
                                                eprintln!("Filter text: {}", app.id_filter_text);
                                                cx.notify();
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            })
            .child(crate::ui::components::render_top_bar(self, view.clone(), cx))
            .child(
                // Content area - Zed style
                div()
                    .id("content-area")
                    .flex_1()
                    .bg(rgb(0x0c0c0e)) // Zed's main background
                    .overflow_hidden()
                    .relative()
                    .drag_over::<gpui::ExternalPaths>(|style, _paths, _window, _cx| {
                        style.bg(gpui::rgba(0x00000022))
                    })
                    .on_drop(cx.listener(move |this, paths: &gpui::ExternalPaths, _window, cx| {
                        eprintln!("📥 on_drop fired with {} paths", paths.paths().len());
                        let stash: Vec<std::path::PathBuf> = paths.paths().to_vec();
                        crate::handlers::drag_drop::handle_drop(this, cx, stash);
                    }))
                    .child(match self.current_view {
                        AppView::LogView => {
                            self.render_log_view(cx.entity().clone()).into_any_element()
                        }
                        AppView::ConfigView => self.render_config_view(cx).into_any_element(),
                        AppView::LibraryView => self.render_library_view(cx).into_any_element(),
                        AppView::PlotView => {
                            crate::ui::views::chart_view::render_plot_view(window, self, cx.entity().clone(), cx)
                                .into_any_element()
                        }
                    })
                    // Library picker overlay — covers only the content area
                    .when_some(
                        crate::ui::components::render_library_picker_overlay(self, view.clone()),
                        |el, picker| el.child(picker),
                    ),
            )
            .child(crate::ui::components::render_status_bar(self, view.clone()))
            // Popovers rendered as siblings of the status bar (not inside it)
            // so the status bar's border_t_1 stacking context doesn't clip
            // the popover's top edge.
            .child(crate::ui::components::render_status_bar_popovers(self, view.clone()))
            .child({
                // Full-screen overlay to catch clicks outside file dropdown
                if self.show_file_menu {
                    let view_for_overlay = view.clone();
                    div()
                        .absolute()
                        .top_0()
                        .left_0()
                        .w_full()
                        .h_full()
                        .bg(rgba(0x00000033))
                        .on_mouse_down(gpui::MouseButton::Left, move |_event, _window, cx| {
                            view_for_overlay.update(cx, |app, cx| {
                                app.show_file_menu = false;
                                cx.notify();
                            });
                        })
                } else {
                    div().hidden()
                }
            })
            .child({
                // File dropdown menu - Zed style (simplified)
                if self.show_file_menu {
                    div()
                        .absolute()
                        .top(px(36.))
                        .left(px(16.))
                        .w(px(160.))
                        .bg(rgb(0x313244)) // BG_ELEVATED from Catppuccin
                        .border_1()
                        .border_color(rgb(0x45475a)) // BORDER_DEFAULT
                        .rounded(px(6.))
                        .shadow_lg()
                        .flex()
                        .flex_col()
                        .py_1()
                        .on_mouse_down(gpui::MouseButton::Left, |_event, _window, cx| {
                            cx.stop_propagation();
                        })
                        // Open BLF
                        .child(
                            div()
                                .px_3()
                                .py_1()
                                .text_xs()
                                .text_color(rgb(0xcdd6f4)) // TEXT_PRIMARY
                                .hover(|style| style.bg(rgb(0x45475a))) // BG_ACTIVE
                                .cursor_pointer()
                                .on_mouse_down(gpui::MouseButton::Left, {
                                    let view = view.clone();
                                    move |_event, _window, cx| {
                                        cx.stop_propagation();
                                        view.update(cx, |this, cx| {
                                            this.show_file_menu = false;
                                            cx.notify();
                                        });
                                        let view = view.clone();
                                        cx.spawn(async move |cx| {
                                            if let Some(file) = rfd::AsyncFileDialog::new()
                                                .add_filter("BLF Files", &["blf", "bin"])
                                                .pick_file()
                                                .await
                                            {
                                                let path = file.path().to_owned();
                                                let path_for_load = path.clone();

                                                // 大文件保护
                                                const FILE_SIZE_THRESHOLD: u64 = 1_000_000_000; // 1 GB
                                                if let Ok(meta) = std::fs::metadata(&path) {
                                                    if meta.len() > FILE_SIZE_THRESHOLD {
                                                        let confirmed = rfd::AsyncMessageDialog::new()
                                                            .set_title("Large File Warning")
                                                            .set_description(&format!(
                                                                "File is {:.2} GB. Loading may take significant time. Continue?",
                                                                meta.len() as f64 / 1_000_000_000.0
                                                            ))
                                                            .set_buttons(rfd::MessageButtons::YesNo)
                                                            .show()
                                                            .await;
                                                        if confirmed != rfd::MessageDialogResult::Yes {
                                                            return Ok::<(), anyhow::Error>(());
                                                        }
                                                    }
                                                }

                                                let _ = cx.update(|cx| {
                                                    view.update(cx, |view, _| {
                                                        view.status_msg = "Loading BLF...".into();
                                                    });
                                                });
                                                let result = cx
                                                    .background_executor()
                                                    .spawn(async move {
                                                        read_blf_from_file(&path_for_load).map_err(|e| {
                                                            anyhow::Error::msg(format!("{:?}", e))
                                                        })
                                                    })
                                                    .await;
                                                let _ = cx.update(|cx| {
                                                    view.update(cx, |view, cx| {
                                                        view.apply_blf_result_single(result, path.clone());
                                                        cx.notify();
                                                    });
                                                });
                                            }
                                            Ok::<(), anyhow::Error>(())
                                        })
                                        .detach();
                                    }
                                })
                                .child("Open BLF..."),
                        )
                        // Open Multiple BLF... (multi-select append)
                        .child(
                            div()
                                .px_3()
                                .py_1()
                                .text_xs()
                                .text_color(rgb(0xcdd6f4))
                                .hover(|style| style.bg(rgb(0x45475a)))
                                .cursor_pointer()
                                .on_mouse_down(gpui::MouseButton::Left, {
                                    let view = view.clone();
                                    move |_event, _window, cx| {
                                        cx.stop_propagation();
                                        view.update(cx, |this, cx| {
                                            this.show_file_menu = false;
                                            cx.notify();
                                        });
                                        let view = view.clone();
                                        cx.spawn(async move |cx| {
                                            let files = rfd::AsyncFileDialog::new()
                                                .add_filter("BLF Files", &["blf", "bin"])
                                                .pick_files()
                                                .await;
                                            if let Some(files) = files {
                                                let paths: Vec<std::path::PathBuf> =
                                                    files.into_iter().map(|f| f.path().to_owned()).collect();
                                                if paths.is_empty() { return Ok::<(), anyhow::Error>(()); }

                                                // 大文件保护：检查总大小
                                                const FILE_SIZE_THRESHOLD: u64 = 1_000_000_000; // 1 GB
                                                let total_size: u64 = paths.iter()
                                                    .filter_map(|p| std::fs::metadata(p).ok())
                                                    .map(|m| m.len())
                                                    .sum();
                                                if total_size > FILE_SIZE_THRESHOLD {
                                                    let confirmed = rfd::AsyncMessageDialog::new()
                                                        .set_title("Large File Warning")
                                                        .set_description(&format!(
                                                            "You are about to load {:.2} GB of BLF files. This may take significant time and memory. Continue?",
                                                            total_size as f64 / 1_000_000_000.0
                                                        ))
                                                        .set_buttons(rfd::MessageButtons::YesNo)
                                                        .show()
                                                        .await;
                                                    if confirmed != rfd::MessageDialogResult::Yes {
                                                        let _ = cx.update(|cx| {
                                                            view.update(cx, |view, cx| {
                                                                view.status_msg = "Loading cancelled".into();
                                                                cx.notify();
                                                            });
                                                        });
                                                        return Ok::<(), anyhow::Error>(());
                                                    }
                                                }

                                                // 初始化 loading_progress
                                                let total = paths.len();
                                                let _ = cx.update(|cx| {
                                                    view.update(cx, |view, cx| {
                                                        view.loading_progress = Some(crate::app::state::LoadingProgress {
                                                            total_files: total,
                                                            completed_files: 0,
                                                            current_file_name: None,
                                                            total_messages_so_far: 0,
                                                            is_cancelled: false,
                                                        });
                                                        view.status_msg = format!("⏳ Loading 0/{} files...", total).into();
                                                        cx.notify();
                                                    });
                                                });

                                                // 并发解析:spawn 所有任务,顺序收集结果（GPUI Task 并发执行）
                                                let mut tasks = Vec::new();
                                                for path in paths.clone() {
                                                    let path = path.clone();
                                                    let task = cx.background_executor().spawn(async move {
                                                        let result = read_blf_from_file(&path).map_err(|e| {
                                                            anyhow::Error::msg(format!("{:?}", e))
                                                        });
                                                        (path, result)
                                                    });
                                                    tasks.push(task);
                                                }

                                                // 顺序 await 但每个任务在后台已开始执行
                                                for task in tasks {
                                                    let (path, result) = task.await;
                                                    let view = view.clone();
                                                    let _ = cx.update(|cx| {
                                                        view.update(cx, |view, cx| {
                                                            view.apply_blf_result_append_one(result, path);
                                                            cx.notify();
                                                        });
                                                    });
                                                }
                                            }
                                            Ok::<(), anyhow::Error>(())
                                        })
                                        .detach();
                                    }
                                })
                                .child("Open Multiple BLF..."),
                        )
                } else {
                    div().hidden()
                }
            })
            .child({
                // Full-screen overlay to catch clicks outside help dropdown
                if self.show_help_menu {
                    let view_for_overlay = view.clone();
                    div()
                        .absolute()
                        .top_0()
                        .left_0()
                        .w_full()
                        .h_full()
                        .bg(rgba(0x00000033))
                        .on_mouse_down(gpui::MouseButton::Left, move |_event, _window, cx| {
                            view_for_overlay.update(cx, |app, cx| {
                                app.show_help_menu = false;
                                cx.notify();
                            });
                        })
                } else {
                    div().hidden()
                }
            })
            .child({
                // Help dropdown menu — About + GitHub + Feedback
                if self.show_help_menu {
                    let view_for_github = view.clone();
                    let view_for_feedback = view.clone();
                    let view_for_about = view.clone();
                    let version = env!("CANVIEW_VERSION");
                    div()
                        .absolute()
                        .top(px(36.))
                        .right(px(16.))
                        .w(px(220.))
                        .bg(rgb(0x313244))
                        .border_1()
                        .border_color(rgb(0x45475a))
                        .rounded(px(6.))
                        .shadow_lg()
                        .flex()
                        .flex_col()
                        .py_1()
                        .on_mouse_down(gpui::MouseButton::Left, |_event, _window, cx| {
                            cx.stop_propagation();
                        })
                        // About canview
                        .child(
                            div()
                                .px_3()
                                .py_1()
                                .text_xs()
                                .text_color(rgb(0xcdd6f4))
                                .hover(|style| style.bg(rgb(0x45475a)))
                                .cursor_pointer()
                                .on_mouse_down(gpui::MouseButton::Left, {
                                    let version = version.to_string();
                                    move |_event, _window, cx| {
                                        cx.stop_propagation();
                                        let v = version.clone();
                                        cx.spawn(async move |cx| {
                                            let _ = rfd::AsyncMessageDialog::new()
                                                .set_title("About canview")
                                                .set_description(&format!(
                                                    "canview v{}\n\nOpen-source cross-platform CAN/LIN bus data analysis tool\n\nhttps://github.com/ucanme/canview",
                                                    v
                                                ))
                                                .set_buttons(rfd::MessageButtons::Ok)
                                                .show()
                                                .await;
                                            Ok::<(), anyhow::Error>(())
                                        })
                                        .detach();
                                        view_for_about.update(cx, |app, cx| {
                                            app.show_help_menu = false;
                                            cx.notify();
                                        });
                                    }
                                })
                                .child(format!("About canview v{}", version)),
                        )
                        // Separator
                        .child(
                            div()
                                .h(px(1.))
                                .mx_2()
                                .my_1()
                                .bg(rgb(0x45475a)),
                        )
                        // View on GitHub
                        .child(
                            div()
                                .px_3()
                                .py_1()
                                .text_xs()
                                .text_color(rgb(0xcdd6f4))
                                .hover(|style| style.bg(rgb(0x45475a)))
                                .cursor_pointer()
                                .on_mouse_down(gpui::MouseButton::Left, move |_event, _window, cx| {
                                    cx.stop_propagation();
                                    cx.open_url("https://github.com/ucanme/canview");
                                    view_for_github.update(cx, |app, cx| {
                                        app.show_help_menu = false;
                                        cx.notify();
                                    });
                                })
                                .child("View on GitHub"),
                        )
                        // Send Feedback
                        .child(
                            div()
                                .px_3()
                                .py_1()
                                .text_xs()
                                .text_color(rgb(0xcdd6f4))
                                .hover(|style| style.bg(rgb(0x45475a)))
                                .cursor_pointer()
                                .on_mouse_down(gpui::MouseButton::Left, move |_event, _window, cx| {
                                    cx.stop_propagation();
                                    cx.open_url("mailto:admin@ucan.me?subject=canview%20Feedback");
                                    view_for_feedback.update(cx, |app, cx| {
                                        app.show_help_menu = false;
                                        cx.notify();
                                    });
                                })
                                .child("Send Feedback"),
                        )
                } else {
                    div().hidden()
                }
            })
            // Share dialog overlay
            .child({
                if self.show_share_dialog {
                    let url = self.share_url().unwrap_or("").to_string();
                    let url_for_copy = url.clone();
                    let url_for_open = url.clone();
                    let view_for_close = view.clone();
                    let view_for_copy = view.clone();
                    let copied = self.share_url_copied;
                    div()
                        .absolute()
                        .top(px(60.))
                        .right(px(20.))
                        .w(px(420.))
                        .bg(rgb(0x1e1e2e))
                        .border_1()
                        .border_color(rgb(0x45475a))
                        .rounded(px(8.))
                        .shadow_lg()
                        .flex()
                        .flex_col()
                        .p_4()
                        .gap_3()
                        .on_mouse_down(gpui::MouseButton::Left, |_event, _window, cx| {
                            cx.stop_propagation();
                        })
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(rgb(0xcdd6f4))
                                        .child("📡 Sharing Libraries"),
                                )
                                .child(
                                    div()
                                        .cursor_pointer()
                                        .text_sm()
                                        .text_color(rgb(0x6c7086))
                                        .hover(|s| s.text_color(rgb(0xcdd6f4)))
                                        .child("✕")
                                        .on_mouse_down(gpui::MouseButton::Left, move |_, _, cx| {
                                            cx.stop_propagation();
                                            view_for_close.update(cx, |app, cx| {
                                                app.show_share_dialog = false;
                                                app.share_url_copied = false;
                                                cx.notify();
                                            });
                                        }),
                                ),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(0xa6adc8))
                                .child("🌐 LAN Share URL (for other devices on the same network):"),
                        )
                        .child(
                            div()
                                .flex()
                                .gap_2()
                                .items_center()
                                .child(
                                    div()
                                        .flex_1()
                                        .bg(rgb(0x11111b))
                                        .rounded(px(4.))
                                        .px_3()
                                        .py_2()
                                        .text_xs()
                                        .text_color(rgb(0x89b4fa))
                                        .overflow_x_hidden()
                                        .child(url),
                                )
                                .child(
                                    div()
                                        .px_3()
                                        .py_2()
                                        .bg(if copied { rgb(0x3a5a40) } else { rgb(0x313244) })
                                        .rounded(px(4.))
                                        .cursor_pointer()
                                        .text_xs()
                                        .text_color(rgb(0xcdd6f4))
                                        .hover(|s| s.bg(if copied { rgb(0x4b7a52) } else { rgb(0x45475a) }))
                                        .on_mouse_down(gpui::MouseButton::Left, move |_, _window, cx| {
                                            cx.stop_propagation();
                                            cx.write_to_clipboard(gpui::ClipboardItem::new_string(url_for_copy.clone()));
                                            let reset_view = view_for_copy.clone();
                                            view_for_copy.update(cx, |app, cx| {
                                                app.share_url_copied = true;
                                                cx.notify();
                                            });
                                            cx.spawn(async move |cx| {
                                                Timer::after(std::time::Duration::from_secs(2)).await;
                                                let _ = cx.update(|cx| {
                                                    reset_view.update(cx, |app, cx| {
                                                        if app.show_share_dialog {
                                                            app.share_url_copied = false;
                                                            cx.notify();
                                                        }
                                                    })
                                                });
                                            })
                                            .detach();
                                        })
                                        .child(if copied { "✓ Copied" } else { "📋 Copy" }),
                                )
                                .child({
                                    div()
                                        .px_3()
                                        .py_2()
                                        .bg(rgb(0x313244))
                                        .rounded(px(4.))
                                        .cursor_pointer()
                                        .text_xs()
                                        .text_color(rgb(0xcdd6f4))
                                        .hover(|s| s.bg(rgb(0x45475a)))
                                        .on_mouse_down(gpui::MouseButton::Left, move |_, _window, cx| {
                                            cx.stop_propagation();
                                            cx.open_url(&url_for_open);
                                        })
                                        .child("🌐 Open")
                                }),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(0x6c7086))
                                .child("The server will stop when you close the app or click 'Stop Share'."),
                        )
                } else {
                    div().hidden()
                }
            })
            // Import dialog overlay
            .child({
                if self.show_import_dialog {
                    let view_for_close = view.clone();
                    let view_for_import = view.clone();
                    let status_msg = self.import_status.clone().unwrap_or_default();
                    let has_status = !status_msg.is_empty();

                    div()
                        .absolute()
                        .top(px(60.))
                        .right(px(20.))
                        .w(px(420.))
                        .bg(rgb(0x1e1e2e))
                        .border_1()
                        .border_color(rgb(0x45475a))
                        .rounded(px(8.))
                        .shadow_lg()
                        .flex()
                        .flex_col()
                        .p_4()
                        .gap_3()
                        .on_mouse_down(gpui::MouseButton::Left, |_event, _window, cx| {
                            cx.stop_propagation();
                        })
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(rgb(0xcdd6f4))
                                        .child("📥 Import Libraries"),
                                )
                                .child(
                                    div()
                                        .cursor_pointer()
                                        .text_sm()
                                        .text_color(rgb(0x6c7086))
                                        .hover(|s| s.text_color(rgb(0xcdd6f4)))
                                        .child("✕")
                                        .on_mouse_down(gpui::MouseButton::Left, move |_, _, cx| {
                                            cx.stop_propagation();
                                            view_for_close.update(cx, |app, cx| {
                                                app.show_import_dialog = false;
                                                app.import_url_input = None;
                                                cx.notify();
                                            });
                                        }),
                                ),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(0xa6adc8))
                                .child("Paste the share URL from another canview instance:"),
                        )
                        .child(
                            div()
                                .flex()
                                .gap_2()
                                .items_center()
                                .child(
                                    div()
                                        .flex_1()
                                        .child(if let Some(ref input) = self.import_url_input {
                                            gpui_component::input::Input::new(input)
                                                .cleanable(true)
                                                .into_any_element()
                                        } else {
                                            div()
                                                .bg(rgb(0x11111b))
                                                .rounded(px(4.))
                                                .px_3()
                                                .py_2()
                                                .text_xs()
                                                .text_color(rgb(0x6c7086))
                                                .child("http://...?token=...")
                                                .into_any_element()
                                        }),
                                )
                                .child(
                                    crate::ui::components::Button::new("Import")
                                        .size(crate::ui::components::ButtonSize::Small)
                                        .variant(crate::ui::components::ButtonVariant::Primary)
                                        .build()
                                        .id("do_import_btn")
                                        .on_mouse_down(gpui::MouseButton::Left, move |_, _, cx| {
                                            cx.stop_propagation();
                                            view_for_import.update(cx, |app, cx| {
                                                let url = app.import_url.clone();
                                                if !url.is_empty() {
                                                    app.start_import(url);
                                                } else {
                                                    app.import_status = Some("Please enter a URL".into());
                                                }
                                                cx.notify();
                                            });
                                        }),
                                ),
                        )
                        .child(if has_status {
                            div()
                                .text_xs()
                                .text_color(if status_msg.contains("failed") || status_msg.contains("Failed") {
                                    rgb(0xf38ba8)
                                } else {
                                    rgb(0xa6e3a1)
                                })
                                .child(status_msg)
                        } else {
                            div().hidden()
                        })
                } else {
                    div().hidden()
                }
            })
    }
}
