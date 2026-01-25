//! Library management view - Dual pane layout
//!
//! Left pane: Library list with search and filtering
//! Right pane: Library details with versions and channel config

use crate::library::LibraryManager;
use crate::models::{ChannelDatabase, ChannelMapping, ChannelType, LibraryVersion, SignalLibrary};
use gpui::prelude::FluentBuilder;
use gpui::*;

/// Render the library management view
pub fn render_library_management_view(
    library_manager: &LibraryManager,
    selected_id: &Option<String>,
    mappings: &[ChannelMapping],
    new_library_name: String,
    cursor_position: usize,
    versions_expanded: bool,
    show_version_input: bool,
    new_version_name: String,
    new_version_cursor_position: usize,
    cx: &mut gpui::Context<crate::CanViewApp>,
) -> impl IntoElement {
    let libraries = &library_manager.libraries();

    div()
        .size_full()
        .flex()
        .flex_row()
        .gap_2()
        .px_2()
        .py_2()
        .bg(rgb(0x0c0c0e))
        // Left pane: Selected library info
        .child(
            div()
                .w(px(200.))
                .h_full()
                .flex()
                .flex_col()
                .gap_2()
                .child(render_library_header(
                    cx,
                    new_library_name.clone(),
                    cursor_position,
                ))
                .child(render_selected_library_info(
                    library_manager,
                    selected_id,
                    mappings,
                    versions_expanded,
                    show_version_input,
                    new_version_name,
                    new_version_cursor_position,
                    cx,
                )),
        )
        // Right pane: Library details
        .child(
            div()
                .flex_1()
                .h_full()
                .flex()
                .flex_col()
                .child(if let Some(lib_id) = selected_id {
                    if let Some(library) = library_manager.find_library(lib_id) {
                        render_library_detail(library, mappings, cx).into_any_element()
                    } else {
                        render_library_empty_state().into_any_element()
                    }
                } else {
                    render_library_empty_state().into_any_element()
                }),
        )
}

/// Render library section header with New button and input
fn render_library_header(
    cx: &mut gpui::Context<crate::CanViewApp>,
    new_library_name: String,
    cursor_position: usize,
) -> impl IntoElement {
    let view = cx.entity().clone();
    let is_editing = !new_library_name.is_empty();

    div()
        .flex()
        .items_center()
        .justify_between()
        .py_2()
        .child(
            div()
                .text_base()
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgb(0xcdd6f4))
                .child("Libraries")
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .when(!is_editing, |d| {
                    // Show "+ New" button when not editing
                    d.px_2()
                        .text_xs()
                        .text_color(rgb(0x646473))
                        .cursor_pointer()
                        .hover(|style| style.text_color(rgb(0xcdd6f4)))
                        .on_mouse_down(gpui::MouseButton::Left, {
                            let view = view.clone();
                            move |_event, _window, cx| {
                                view.update(cx, |this, cx| {
                                    this.new_library_name = String::from(" "); // Start with space to trigger input mode
                                    cx.notify();
                                });
                            }
                        })
                        .child("+ New")
                })
                .when(is_editing, |d| {
                    // Show text input when editing
                    d.child(
                        div()
                            .px_2()
                            .py_1()
                            .bg(rgb(0x1a1a1a))
                            .border_1()
                            .border_color(rgb(0x89b4fa))
                            .rounded(px(2.))
                            .flex()
                            .items_center()
                            .min_w(px(100.))
                            .max_w(px(200.))
                            .cursor_text()
                            .id("library_name_input")
                            .focusable()
                            .when(new_library_name.trim().is_empty(), |d| {
                                // Empty: show placeholder with cursor at beginning
                                d.child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_0()
                                        .child(
                                            div()
                                                .w(px(2.))
                                                .h(px(14.))
                                                .bg(rgb(0x89b4fa))
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(rgb(0x646473))
                                                .child("Library name...")
                                        )
                                )
                            })
                            .when(!new_library_name.trim().is_empty(), |d| {
                                // Has text: split at cursor position and render cursor between parts
                                let text = new_library_name.chars().collect::<Vec<_>>();
                                let cursor_pos = cursor_position.min(text.len());

                                let before_cursor: String = text[..cursor_pos].iter().collect();
                                let after_cursor: String = text[cursor_pos..].iter().collect();

                                d.child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_0()
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(rgb(0xcdd6f4))
                                                .child(before_cursor)
                                        )
                                        .child(
                                            div()
                                                .w(px(2.))
                                                .h(px(14.))
                                                .bg(rgb(0x89b4fa))
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(rgb(0xcdd6f4))
                                                .child(after_cursor)
                                        )
                                )
                            })
                            .on_key_down({
                                let view = view.clone();
                                let text = new_library_name.clone();
                                move |event, _window, cx| {
                                    let keystroke = format!("{}", event.keystroke);
                                    eprintln!("Library name input key_down: {}, text: {}", keystroke, text);

                                    match keystroke.as_str() {
                                        "backspace" => {
                                            view.update(cx, |this, cx| {
                                                if this.library_cursor_position > 0 {
                                                    let mut chars: Vec<char> = this.new_library_name.chars().collect();
                                                    chars.remove(this.library_cursor_position - 1);
                                                    this.new_library_name = chars.into_iter().collect();
                                                    this.library_cursor_position -= 1;
                                                    cx.notify();
                                                }
                                            });
                                        }
                                        "enter" => {
                                            view.update(cx, |this, cx| {
                                                this.create_new_library(cx);
                                                this.library_cursor_position = 0;
                                            });
                                        }
                                        "escape" => {
                                            view.update(cx, |this, cx| {
                                                this.new_library_name = String::new();
                                                this.library_cursor_position = 0;
                                                cx.notify();
                                            });
                                        }
                                        "left" => {
                                            view.update(cx, |this, cx| {
                                                if this.library_cursor_position > 0 {
                                                    this.library_cursor_position -= 1;
                                                    cx.notify();
                                                }
                                            });
                                        }
                                        "right" => {
                                            view.update(cx, |this, cx| {
                                                let text_len = this.new_library_name.chars().count();
                                                if this.library_cursor_position < text_len {
                                                    this.library_cursor_position += 1;
                                                    cx.notify();
                                                }
                                            });
                                        }
                                        "home" => {
                                            view.update(cx, |this, cx| {
                                                this.library_cursor_position = 0;
                                                cx.notify();
                                            });
                                        }
                                        "end" => {
                                            view.update(cx, |this, cx| {
                                                this.library_cursor_position = this.new_library_name.chars().count();
                                                cx.notify();
                                            });
                                        }
                                        _ => {
                                            // IMPROVED: Support multi-character input for IME
                                            // Check if this is a printable character or multi-char string
                                            let is_printable = if keystroke.len() == 1 {
                                                keystroke.chars().next().map(|c| !c.is_control()).unwrap_or(false)
                                            } else if keystroke.len() > 1 {
                                                // Multi-character string (possibly from IME)
                                                !keystroke.to_lowercase().starts_with("backspace")
                                                    && !keystroke.to_lowercase().starts_with("delete")
                                                    && !keystroke.to_lowercase().starts_with("left")
                                                    && !keystroke.to_lowercase().starts_with("right")
                                                    && !keystroke.to_lowercase().starts_with("up")
                                                    && !keystroke.to_lowercase().starts_with("down")
                                                    && !keystroke.to_lowercase().starts_with("home")
                                                    && !keystroke.to_lowercase().starts_with("end")
                                                    && keystroke.chars().all(|c| !c.is_control())
                                            } else {
                                                false
                                            };

                                            if is_printable {
                                                // Character validation: support Chinese, English, numbers, spaces
                                                let is_valid_char = |c: char| -> bool {
                                                    !c.is_control() && (c.is_ascii_alphanumeric() || c == ' ' || !c.is_ascii())
                                                };

                                                let all_valid = keystroke.chars().all(is_valid_char);

                                                if all_valid {
                                                    view.update(cx, |this, cx| {
                                                        // Insert at cursor position
                                                        let mut chars: Vec<char> = this.new_library_name.chars().collect();
                                                        for (i, ch) in keystroke.chars().enumerate() {
                                                            chars.insert(this.library_cursor_position + i, ch);
                                                        }
                                                        this.new_library_name = chars.into_iter().collect();
                                                        this.library_cursor_position += keystroke.chars().count();
                                                        eprintln!("Library name inserted '{}', text: '{}'", keystroke, this.new_library_name);
                                                        cx.notify();
                                                    });
                                                }
                                            }
                                        }
                                    }
                                }
                            })
                    )
                    .child(
                        div()
                            .px_2()
                            .text_xs()
                            .text_color(rgb(0x646473))
                            .cursor_pointer()
                            .hover(|style| style.text_color(rgb(0xcdd6f4)))
                            .on_mouse_down(gpui::MouseButton::Left, {
                                let view = view.clone();
                                move |_event, _window, cx| {
                                    view.update(cx, |this, cx| {
                                        this.create_new_library(cx);
                                    });
                                }
                            })
                            .child("Create")
                    )
                    .child(
                        div()
                            .px_2()
                            .text_xs()
                            .text_color(rgb(0x646473))
                            .cursor_pointer()
                            .hover(|style| style.text_color(rgb(0xf38ba8)))
                            .on_mouse_down(gpui::MouseButton::Left, {
                                let view = view.clone();
                                move |_event, _window, cx| {
                                    view.update(cx, |this, cx| {
                                        this.new_library_name = String::new();
                                        this.library_cursor_position = 0;
                                        cx.notify();
                                    });
                                }
                            })
                            .child("Cancel")
                    )
                })
        )
}

/// Render search box
fn render_library_search() -> impl IntoElement {
    div().px_2().child(
        div()
            .w_full()
            .px_2()
            .py_1()
            .bg(rgb(0x1a1a1a))
            .text_xs()
            .text_color(rgb(0x646473))
            .child("Search..."),
    )
}

/// Render library list
fn render_library_list(
    libraries: &[SignalLibrary],
    selected_id: &Option<String>,
    mappings: &[ChannelMapping],
    cx: &mut gpui::Context<crate::CanViewApp>,
) -> impl IntoElement {
    if libraries.is_empty() {
        return div().flex_1().flex().items_center().justify_center().child(
            div()
                .text_sm()
                .text_color(rgb(0x646473))
                .child("No libraries"),
        );
    }

    let mut list = div().flex_1().flex().flex_col().gap_1();

    for library in libraries {
        let is_selected = selected_id.as_ref() == Some(&library.id);
        let is_used = library.is_used(mappings);
        let lib_id = library.id.clone();
        let lib_name = library.name.clone();
        let version_count = library.versions.len();

        list = list.child(
            div()
                .px_2()
                .py_1()
                .flex()
                .items_center()
                .justify_between()
                .when(is_selected, |div| div.bg(rgb(0x1a1a1a)))
                .hover(|style| style.bg(rgb(0x151515)))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .flex_1()
                        .cursor_pointer()
                        .on_mouse_down(gpui::MouseButton::Left, {
                            let lib_id = lib_id.clone();
                            let view = cx.entity().clone();
                            move |_event, _window, cx| {
                                view.update(cx, |this, cx| {
                                    this.selected_library_id = Some(lib_id.clone());
                                    cx.notify();
                                });
                            }
                        })
                        .child(div().text_sm().text_color(rgb(0xcdd6f4)).child(lib_name))
                        .when(version_count > 0, |d: gpui::Div| {
                            d.child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0x646473))
                                    .child(version_count.to_string()),
                            )
                        }),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(0x646473))
                        .cursor_pointer()
                        .hover(|style| style.text_color(rgb(0xf38ba8)))
                        .on_mouse_down(gpui::MouseButton::Left, {
                            let view = cx.entity().clone();
                            let lib_id = lib_id.clone();
                            move |_event, _window, cx| {
                                view.update(cx, |this, cx| {
                                    this.delete_library(&lib_id, cx);
                                });
                            }
                        })
                        .child("×"),
                ),
        );
    }

    list
}

/// Render library detail view
fn render_library_detail(
    library: &SignalLibrary,
    mappings: &[ChannelMapping],
    cx: &mut gpui::Context<crate::CanViewApp>,
) -> impl IntoElement {
    // Find all mappings for this library
    let library_mappings: Vec<_> = mappings
        .iter()
        .filter(|m| m.library_id.as_ref() == Some(&library.id))
        .collect();

    div()
        .size_full()
        .flex()
        .flex_col()
        .gap_2()
        .px_4()
        // Library header
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .py_2()
                .border_b_1()
                .border_color(rgb(0x2a2a2a))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .text_base()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(rgb(0xcdd6f4))
                                .child(library.name.clone())
                        )
                        .child(
                            div()
                                .text_xs()
                                .px_2()
                                .py_1()
                                .rounded(px(2.))
                                .bg(match library.database_type() {
                                    crate::models::DatabaseType::DBC => rgb(0x89b4fa),
                                    crate::models::DatabaseType::LDF => rgb(0xfab387),
                                })
                                .text_color(rgb(0x1a1a1a))
                                .child(format!("{:?}", library.database_type()))
                        )
                )
                .child(
                    div()
                        .flex()
                        .gap_2()
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(0x646473))
                                .cursor_pointer()
                                .hover(|style| style.text_color(rgb(0xcdd6f4)))
                                .on_mouse_down(gpui::MouseButton::Left, {
                                    let view = cx.entity().clone();
                                    move |_event, _window, cx| {
                                        view.update(cx, |this, cx| {
                                            this.import_database_file(cx);
                                        });
                                    }
                                })
                                .child("Import")
                        )
                )
        )
        // Channel mappings header
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .py_2()
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(rgb(0xcdd6f4))
                        .child("Channel Mappings")
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(0x646473))
                        .child(format!("{} mapping(s)", library_mappings.len()))
                )
        )
        // Channel mappings list or empty state
        .child(
            div()
                .flex_1()
                .flex()
                .flex_col()
                .when(library_mappings.is_empty(), |d| {
                    d.child(
                        div()
                            .flex_1()
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(0x646473))
                                    .child("No channel mappings configured.\nClick a version in the left panel to create one.")
                            )
                    )
                })
                .when(!library_mappings.is_empty(), |d| {
                    let mut list = d.flex().flex_col().gap_2();

                    for (idx, mapping) in library_mappings.iter().enumerate() {
                        let view = cx.entity().clone();
                        let lib_id = library.id.clone();
                        let mapping_idx = idx;
                        let channel_type = mapping.channel_type;
                        let channel_id = mapping.channel_id;
                        let version_name = mapping.version_name.clone().unwrap_or_default();
                        let path = mapping.path.clone();
                        let description = mapping.description.clone();

                        list = list.child(
                            div()
                                .px_3()
                                .py_2()
                                .bg(rgb(0x1a1a1a))
                                .rounded(px(4.))
                                .flex()
                                .flex_col()
                                .gap_2()
                                // Mapping header
                                .child(
                                    div()
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
                                                        .text_sm()
                                                        .font_weight(FontWeight::MEDIUM)
                                                        .text_color(rgb(0xcdd6f4))
                                                        .child(format!("Channel {}", mapping.channel_id))
                                                )
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .px_2()
                                                        .py_1()
                                                        .rounded(px(2.))
                                                        .bg(match mapping.channel_type {
                                                            crate::models::ChannelType::CAN => rgb(0x89b4fa),
                                                            crate::models::ChannelType::LIN => rgb(0xfab387),
                                                        })
                                                        .text_color(rgb(0x1a1a1a))
                                                        .child(format!("{:?}", mapping.channel_type))
                                                )
                                        )
                                )
                                // Mapping details
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .px_2()
                                        // Version
                                        .child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .gap_2()
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(rgb(0x646473))
                                                        .child("Version:")
                                                )
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(rgb(0xcdd6f4))
                                                        .child(version_name)
                                                )
                                        )
                                        // Path
                                        .child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .gap_2()
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(rgb(0x646473))
                                                        .child("File:")
                                                )
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(rgb(0xcdd6f4))
                                                        .child(if path.is_empty() {
                                                            "Not configured".to_string()
                                                        } else {
                                                            path.clone()
                                                        })
                                                )
                                        )
                                        // Description
                                        .child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .gap_2()
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(rgb(0x646473))
                                                        .child("Description:")
                                                )
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(rgb(0xcdd6f4))
                                                        .child(if description.is_empty() {
                                                            "-".to_string()
                                                        } else {
                                                            description.clone()
                                                        })
                                                )
                                        )
                                )
                        );
                    }

                    list
                })
        )
}

/// Render selected library info with collapsible versions
fn render_selected_library_info(
    library_manager: &LibraryManager,
    selected_id: &Option<String>,
    mappings: &[ChannelMapping],
    versions_expanded: bool,
    show_version_input: bool,
    new_version_name: String,
    new_version_cursor_position: usize,
    cx: &mut gpui::Context<crate::CanViewApp>,
) -> impl IntoElement {
    let view = cx.entity().clone();

    if let Some(lib_id) = selected_id {
        if let Some(library) = library_manager.find_library(lib_id) {
            let lib_id_clone = lib_id.clone();
            let lib_name = library.name.clone();
            let version_count = library.versions.len();

            return div()
                .flex_1()
                .flex()
                .flex_col()
                .gap_1()
                // Library name header with collapse toggle
                .child(
                    div()
                        .px_2()
                        .py_2()
                        .flex()
                        .items_center()
                        .justify_between()
                        .bg(rgb(0x1a1a1a))
                        .rounded(px(4.))
                        .cursor_pointer()
                        .hover(|style| style.bg(rgb(0x252525)))
                        .on_mouse_down(gpui::MouseButton::Left, {
                            let view = view.clone();
                            move |_event, _window, cx| {
                                view.update(cx, |this, cx| {
                                    this.library_versions_expanded = !this.library_versions_expanded;
                                    cx.notify();
                                });
                            }
                        })
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .flex_1()
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(rgb(0xcdd6f4))
                                        .child(lib_name.clone())
                                )
                                .when(version_count > 0, |d| {
                                    d.child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(0x646473))
                                            .child(format!("({})", version_count))
                                    )
                                })
                                .child(
                                    div()
                                        .w(px(20.))
                                        .h(px(20.))
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .rounded(px(3.))
                                        .hover(|style| style.bg(rgb(0x3a3a3a)))
                                        .cursor_pointer()
                                        .on_mouse_down(gpui::MouseButton::Left, {
                                            let view = view.clone();
                                            let lib_id = lib_id.clone();
                                            move |_event, _window, cx| {
                                                view.update(cx, |this, cx| {
                                                    this.add_library_version(cx);
                                                });
                                            }
                                        })
                                        .child(
                                            div()
                                                .text_sm()
                                                .text_color(rgb(0x89b4fa))
                                                .font_weight(FontWeight::BOLD)
                                                .child("+")
                                        )
                                )
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(0x89b4fa))
                                .child(if versions_expanded { "▼" } else { "▶" })
                        )
                )
                // Collapsible versions list
                .when(versions_expanded && !library.versions.is_empty(), |d: gpui::Div| {
                    let mut versions = div()
                        .flex()
                        .flex_col()
                        .gap_0()
                        .px_2();

                    for version in &library.versions {
                        let is_active = mappings.iter().any(|m| {
                            m.library_id.as_ref() == Some(lib_id)
                                && m.version_name.as_ref().map(|s| s.as_str()) == Some(version.name.as_str())
                        });

                        let lib_id = lib_id_clone.clone();
                        let version_name = version.name.clone();
                        let channel_type = library.channel_type;

                        versions = versions.child(
                            div()
                                .px_2()
                                .py_1()
                                .flex()
                                .items_center()
                                .justify_between()
                                .hover(|style| style.bg(rgb(0x1a1a1a)))
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_2()
                                        .cursor_pointer()
                                        .flex_1()
                                        .on_mouse_down(gpui::MouseButton::Left, {
                                            let view = view.clone();
                                            let lib_id = lib_id.clone();
                                            let version_name = version_name.clone();
                                            let channel_type = channel_type;
                                            move |_event, _window, cx| {
                                                view.update(cx, |this, cx| {
                                                    let mapping = this.app_config.mappings.iter()
                                                        .position(|m| m.library_id.as_ref() == Some(&lib_id));

                                                    if let Some(idx) = mapping {
                                                        this.app_config.mappings[idx].version_name = Some(version_name.clone());
                                                    } else {
                                                        let new_mapping = crate::models::ChannelMapping {
                                                            channel_type,
                                                            channel_id: 1,
                                                            path: String::new(),
                                                            description: String::new(),
                                                            library_id: Some(lib_id.clone()),
                                                            version_name: Some(version_name.clone()),
                                                        };
                                                        this.app_config.mappings.push(new_mapping);
                                                    }
                                                    cx.notify();
                                                });
                                            }
                                        })
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(rgb(0xcdd6f4))
                                                .child(version.name.clone())
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(rgb(0x646473))
                                                .child(version.date.clone())
                                        )
                                )
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_2()
                                        .when(is_active, |d| {
                                            d.child(
                                                div()
                                                    .text_xs()
                                                    .text_color(rgb(0xa6e3a1))
                                                    .child("●")
                                            )
                                        })
                                )
                        );
                    }

                    d.child(versions)
                })
                .when(versions_expanded && library.versions.is_empty(), |d: gpui::Div| {
                    d.child(
                        div()
                            .px_2()
                            .py_2()
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0x646473))
                                    .child("No versions")
                            )
                    )
                })
                .when(show_version_input, |d| {
                    // Version input box
                    d.child(
                        div()
                            .px_2()
                            .child(
                                div()
                                    .w_full()
                                    .px_2()
                                    .py_1()
                                    .flex()
                                    .flex_col()
                                    .gap_1()
                                    .bg(rgb(0x1a1a1a))
                                    .border_1()
                                    .border_color(rgb(0x89b4fa))
                                    .rounded(px(2.))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(0x89b4fa))
                                            .child("Version name:")
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .gap_1()
                                            .child(
                                                div()
                                                    .flex_1()
                                                    .min_h(px(24.))
                                                    .px_2()
                                                    .py_1()
                                                    .bg(rgb(0x0c0c0e))
                                                    .border_1()
                                                    .border_color(rgb(0x2a2a2a))
                                                    .rounded(px(2.))
                                                    .flex()
                                                    .items_center()
                                                    .when(new_version_name.trim().is_empty(), |d| {
                                                        d.child(
                                                            div()
                                                                .text_xs()
                                                                .text_color(rgb(0x646473))
                                                                .child("v1.0")
                                                        )
                                                    })
                                                    .when(!new_version_name.trim().is_empty(), |d| {
                                                        let text = new_version_name.chars().collect::<Vec<_>>();
                                                        let cursor_pos = new_version_cursor_position.min(text.len());

                                                        let before_cursor: String = text[..cursor_pos].iter().collect();
                                                        let after_cursor: String = text[cursor_pos..].iter().collect();

                                                        d.child(
                                                            div()
                                                                .flex()
                                                                .items_center()
                                                                .gap_0()
                                                                .child(
                                                                    div()
                                                                        .text_xs()
                                                                        .text_color(rgb(0xcdd6f4))
                                                                        .child(before_cursor)
                                                                )
                                                                .child(
                                                                    div()
                                                                        .w(px(2.))
                                                                        .h(px(14.))
                                                                        .bg(rgb(0x89b4fa))
                                                                )
                                                                .child(
                                                                    div()
                                                                        .text_xs()
                                                                        .text_color(rgb(0xcdd6f4))
                                                                        .child(after_cursor)
                                                                )
                                                        )
                                                    })
                                                    .id("version_name_input")
                                                    .on_key_down({
                                                        let view = view.clone();
                                                        move |event, _window, cx| {
                                                            let keystroke = format!("{}", event.keystroke);

                                                            let handled = match keystroke.as_str() {
                                                                "enter" => {
                                                                    view.update(cx, |this, cx| {
                                                                        if !this.new_version_name.trim().is_empty() {
                                                                            this.create_library_version(this.new_version_name.clone(), cx);
                                                                        }
                                                                    });
                                                                    true
                                                                }
                                                                "escape" => {
                                                                    view.update(cx, |this, cx| {
                                                                        this.cancel_version_input(cx);
                                                                    });
                                                                    true
                                                                }
                                                                "backspace" => {
                                                                    view.update(cx, |this, cx| {
                                                                        if this.new_version_cursor_position > 0 {
                                                                            let mut chars: Vec<char> = this.new_version_name.chars().collect();
                                                                            chars.remove(this.new_version_cursor_position - 1);
                                                                            this.new_version_name = chars.into_iter().collect();
                                                                            this.new_version_cursor_position -= 1;
                                                                            cx.notify();
                                                                        }
                                                                    });
                                                                    true
                                                                }
                                                                "left" => {
                                                                    view.update(cx, |this, cx| {
                                                                        if this.new_version_cursor_position > 0 {
                                                                            this.new_version_cursor_position -= 1;
                                                                            cx.notify();
                                                                        }
                                                                    });
                                                                    true
                                                                }
                                                                "right" => {
                                                                    view.update(cx, |this, cx| {
                                                                        let text_len = this.new_version_name.chars().count();
                                                                        if this.new_version_cursor_position < text_len {
                                                                            this.new_version_cursor_position += 1;
                                                                            cx.notify();
                                                                        }
                                                                    });
                                                                    true
                                                                }
                                                                "home" => {
                                                                    view.update(cx, |this, cx| {
                                                                        this.new_version_cursor_position = 0;
                                                                        cx.notify();
                                                                    });
                                                                    true
                                                                }
                                                                "end" => {
                                                                    view.update(cx, |this, cx| {
                                                                        let text_len = this.new_version_name.chars().count();
                                                                        this.new_version_cursor_position = text_len;
                                                                        cx.notify();
                                                                    });
                                                                    true
                                                                }
                                                                _ => {
                                                                    // IMPROVED: Support multi-character input for IME
                                                                    // Check if this is a printable character or multi-char string
                                                                    let is_printable = if keystroke.len() == 1 {
                                                                        keystroke.chars().next().map(|c| !c.is_control()).unwrap_or(false)
                                                                    } else if keystroke.len() > 1 {
                                                                        // Multi-character string (possibly from IME)
                                                                        !keystroke.to_lowercase().starts_with("backspace")
                                                                            && !keystroke.to_lowercase().starts_with("delete")
                                                                            && !keystroke.to_lowercase().starts_with("left")
                                                                            && !keystroke.to_lowercase().starts_with("right")
                                                                            && !keystroke.to_lowercase().starts_with("up")
                                                                            && !keystroke.to_lowercase().starts_with("down")
                                                                            && !keystroke.to_lowercase().starts_with("home")
                                                                            && !keystroke.to_lowercase().starts_with("end")
                                                                            && keystroke.chars().all(|c| !c.is_control())
                                                                    } else {
                                                                        false
                                                                    };

                                                                    if is_printable {
                                                                        // Version names: only ASCII alphanumeric, dot, underscore, hyphen
                                                                        let is_valid_char = |c: char| -> bool {
                                                                            c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-'
                                                                        };

                                                                        let all_valid = keystroke.chars().all(is_valid_char);

                                                                        if all_valid {
                                                                            view.update(cx, |this, cx| {
                                                                                // Ensure cursor position is within bounds
                                                                                let text_len = this.new_version_name.chars().count();
                                                                                this.new_version_cursor_position = this.new_version_cursor_position.min(text_len);

                                                                                // Insert all characters at cursor position
                                                                                let mut chars: Vec<char> = this.new_version_name.chars().collect();
                                                                                for (i, ch) in keystroke.chars().enumerate() {
                                                                                    chars.insert(this.new_version_cursor_position + i, ch);
                                                                                }
                                                                                this.new_version_name = chars.into_iter().collect();
                                                                                this.new_version_cursor_position += keystroke.chars().count();
                                                                                eprintln!("Version name inserted '{}', text: '{}'", keystroke, this.new_version_name);
                                                                                cx.notify();
                                                                            });
                                                                            true
                                                                        } else {
                                                                            false
                                                                        }
                                                                    } else {
                                                                        false
                                                                    }
                                                                }
                                                            };
                                                        }
                                                    })
                                            )
                                            .child(
                                                div()
                                                    .px_2()
                                                    .py_1()
                                                    .flex()
                                                    .gap_1()
                                                    .child({
                                                        let view = view.clone();
                                                        let version_name = new_version_name.clone();
                                                        div()
                                                            .px_3()
                                                            .py_1()
                                                            .bg(rgb(0x89b4fa))
                                                            .rounded(px(2.))
                                                            .cursor_pointer()
                                                            .hover(|style| style.bg(rgb(0x6a9cda)))
                                                            .on_mouse_down(gpui::MouseButton::Left, move |_event, _window, cx| {
                                                                if !version_name.trim().is_empty() {
                                                                    view.update(cx, |this, cx| {
                                                                        this.create_library_version(version_name.clone(), cx);
                                                                    });
                                                                }
                                                            })
                                                            .child(
                                                                div()
                                                                    .text_xs()
                                                                    .text_color(rgb(0x1a1a1a))
                                                                    .font_weight(gpui::FontWeight::BOLD)
                                                                    .child("Create")
                                                            )
                                                    })
                                                    .child({
                                                        let view = view.clone();
                                                        div()
                                                            .px_3()
                                                            .py_1()
                                                            .bg(rgb(0x2a2a2a))
                                                            .rounded(px(2.))
                                                            .cursor_pointer()
                                                            .hover(|style| style.bg(rgb(0x3a3a3a)))
                                                            .on_mouse_down(gpui::MouseButton::Left, move |_event, _window, cx| {
                                                                view.update(cx, |this, cx| {
                                                                    this.cancel_version_input(cx);
                                                                });
                                                            })
                                                            .child(
                                                                div()
                                                                    .text_xs()
                                                                    .text_color(rgb(0xcdd6f4))
                                                                    .child("Cancel")
                                                            )
                                                    })
                                            )
                                    )
                            )
                    )
                })
                .into_any_element();
        }
    }

    // No library selected
    div()
        .flex_1()
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .text_sm()
                .text_color(rgb(0x646473))
                .child("No library selected"),
        )
        .into_any_element()
}

/// Render empty state when no library is selected
fn render_library_empty_state() -> impl IntoElement {
    div()
        .flex_1()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .child(
            div()
                .text_sm()
                .text_color(rgb(0x646473))
                .child("Select a library"),
        )
}
