//! Three-column library management layout (Enhanced with EnhancedTextInput)
//!
//! 使用新的 EnhancedTextInput 组件简化代码
//! - 左栏：库列表
//! - 中栏：版本列表
//! - 右栏：通道配置

use gpui::prelude::*;
use gpui::*;
use crate::models::{SignalLibrary, LibraryVersion, ChannelMapping};
use crate::app::LibraryDialogType;
use crate::ui::components::{EnhancedTextInputBuilder};
use crate::ui::components::enhanced_text_input::TextInputValidation;

/// 渲染三栏布局的库管理界面
pub fn render_library_management_view(
    libraries: &[SignalLibrary],
    selected_library_id: &Option<String>,
    mappings: &[ChannelMapping],
    show_new_library_input: bool,
    show_add_version_input: bool,
    new_library_name: &str,
    new_version_name: &str,
    focused_input: &Option<String>,
    _library_cursor_pos: usize,  // No longer needed - handled by EnhancedTextInput
    _version_cursor_pos: usize,  // No longer needed - handled by EnhancedTextInput
    cx: &mut Context<crate::CanViewApp>
) -> impl IntoElement {
    div()
        .flex_1()
        .flex()
        .flex_row()
        .gap_2()
        .p_4()
        .bg(rgb(0x0a0a0a))
        .child(render_left_column_enhanced(
            libraries,
            selected_library_id,
            mappings,
            show_new_library_input,
            new_library_name,
            focused_input,
            cx
        ))
        .child(render_middle_column_enhanced(
            libraries,
            selected_library_id,
            mappings,
            show_add_version_input,
            new_version_name,
            focused_input,
            cx
        ))
        .child(render_right_column(libraries, selected_library_id, mappings))
}

/// 左栏：库列表（使用 EnhancedTextInput）
fn render_left_column_enhanced(
    libraries: &[SignalLibrary],
    selected_library_id: &Option<String>,
    mappings: &[ChannelMapping],
    show_new_library_input: bool,
    new_library_name: &str,
    focused_input: &Option<String>,
    cx: &mut Context<crate::CanViewApp>
) -> impl IntoElement {
    let is_focused = focused_input.as_ref() == Some(&"new_library_input".to_string());
    let view = cx.entity().clone();

    div()
        .w(px(300.0))
        .flex()
        .flex_col()
        .gap_2()
        .child(
            // 标题
            div()
                .text_base()
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(0xffffff))
                .child("Libraries")
        )
        .child(
            // 过滤按钮
            div()
                .flex()
                .gap_1()
                .child(render_filter_button("All", true))
                .child(render_filter_button("CAN", true))
                .child(render_filter_button("LIN", false))
        )
        .when(show_new_library_input, |this| {
            this.child(
                div()
                    .flex()
                    .gap_2()
                    .child(
                        // 使用 EnhancedTextInput - 大幅简化代码！
                        EnhancedTextInputBuilder::new()
                            .text(new_library_name.to_string())
                            .placeholder("Library name...")
                            .focused(is_focused)
                            .validation(TextInputValidation::LibraryName)
                            .max_width(px(220.))
                            .min_width(px(150.))
                            .build(
                                "new_library_input_enhanced",
                                view.clone(),
                                {
                                    let view = view.clone();
                                    move |new_text, cx| {
                                        view.update(cx, |this, cx| {
                                            this.new_library_name = new_text.to_string();
                                            this.library_input_state.text = new_text.to_string();
                                            // 同步光标位置到末尾
                                            this.library_cursor_position = new_text.chars().count();
                                            this.library_input_state.cursor_position = this.library_cursor_position;
                                            eprintln!("✅ EnhancedTextInput library name changed: '{}', cursor={}",
                                                new_text, this.library_cursor_position);
                                            cx.notify();
                                        });
                                    }
                                },
                                {
                                    let view = view.clone();
                                    move |text, cx| {
                                        // Enter 键提交
                                        view.update(cx, |this, cx| {
                                            if !text.is_empty() {
                                                this.create_library(cx);
                                                this.is_editing_library_name = false;
                                                this.focused_library_input = None;
                                                eprintln!("✅ EnhancedTextInput library created: '{}'", text);
                                            }
                                        });
                                    }
                                },
                            )
                    )
                    .child(
                        div()
                            .px_3()
                            .py_1()
                            .bg(rgb(0x10b981))
                            .rounded(px(4.0))
                            .cursor_pointer()
                            .hover(|style| style.bg(rgb(0x059669)))
                            .text_color(rgb(0xffffff))
                            .text_sm()
                            .child("Create")
                            .on_mouse_down(gpui::MouseButton::Left, cx.listener(|this, _event, _window, cx| {
                                this.create_library(cx);
                                this.is_editing_library_name = false;
                                this.focused_library_input = None;
                                eprintln!("✅ Create button clicked");
                            }))
                    )
                    .child(
                        div()
                            .px_3()
                            .py_1()
                            .bg(rgb(0x6b7280))
                            .rounded(px(4.0))
                            .cursor_pointer()
                            .hover(|style| style.bg(rgb(0x4b5563)))
                            .text_color(rgb(0xffffff))
                            .text_sm()
                            .child("Cancel")
                            .on_mouse_down(gpui::MouseButton::Left, cx.listener(|this, _event, _window, cx| {
                                this.show_library_dialog = false;
                                this.new_library_name.clear();
                                this.library_input_state.text.clear();
                                this.is_editing_library_name = false;
                                this.focused_library_input = None;
                                cx.notify();
                                eprintln!("❌ Cancel button clicked");
                            }))
                    )
            )
        })
        .child(
            // 库列表
            div()
                .flex_1()
                .bg(rgb(0x1f1f1f))
                .border_1()
                .border_color(rgb(0x2a2a2a))
                .rounded(px(8.0))
                .when(libraries.is_empty(), |this| {
                    this.child(
                        div()
                            .p_4()
                            .items_center()
                            .justify_center()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(0x6b7280))
                                    .child("No libraries")
                            )
                    )
                    .child(render_add_library_button(cx))
                })
                .when(!libraries.is_empty(), |this| {
                    let mut list = this;
                    for library in libraries {
                        list = list.child(render_library_item(library, selected_library_id, mappings));
                    }
                    list.child(render_add_library_button(cx))
                })
        )
}

/// 中栏：版本列表（使用 EnhancedTextInput）
fn render_middle_column_enhanced(
    libraries: &[SignalLibrary],
    selected_library_id: &Option<String>,
    mappings: &[ChannelMapping],
    show_add_version_input: bool,
    new_version_name: &str,
    focused_input: &Option<String>,
    cx: &mut Context<crate::CanViewApp>
) -> impl IntoElement {
    let is_focused = focused_input.as_ref() == Some(&"new_version_input".to_string());
    let view = cx.entity().clone();

    // 找到选中的库
    let selected_library = selected_library_id
        .as_ref()
        .and_then(|id| libraries.iter().find(|lib| &lib.id == id));

    div()
        .w(px(300.0))
        .flex()
        .flex_col()
        .gap_2()
        .child(
            div()
                .text_base()
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(0xffffff))
                .child("Versions")
        )
        .when_some(selected_library, |this, library| {
            this.child(
                div()
                    .flex()
                    .gap_2()
                    .when(show_add_version_input, |this| {
                        this.child(
                            // 使用 EnhancedTextInput - 简化版本输入！
                            EnhancedTextInputBuilder::new()
                                .text(new_version_name.to_string())
                                .placeholder("v1.0.0")
                                .focused(is_focused)
                                .validation(TextInputValidation::VersionName)
                                .max_width(px(180.))
                                .min_width(px(120.))
                                .build(
                                    "new_version_input_enhanced",
                                    view.clone(),
                                    {
                                        let view = view.clone();
                                        move |new_text, cx| {
                                            view.update(cx, |this, cx| {
                                                this.new_version_name = new_text.to_string();
                                                // 同步光标位置
                                                this.new_version_cursor_position = new_text.len();
                                                eprintln!("✅ EnhancedTextInput version changed: '{}', cursor={}",
                                                    new_text, this.new_version_cursor_position);
                                                cx.notify();
                                            });
                                        }
                                    },
                                    {
                                        let view = view.clone();
                                        move |text, cx| {
                                            // Enter 键提交
                                            view.update(cx, |this, cx| {
                                                if !text.is_empty() {
                                                    this.add_library_version(cx);
                                                    this.focused_library_input = None;
                                                    eprintln!("✅ EnhancedTextInput version created: '{}'", text);
                                                }
                                            });
                                        }
                                    },
                                )
                        )
                        .child(
                            div()
                                .px_3()
                                .py_1()
                                .bg(rgb(0x10b981))
                                .rounded(px(4.0))
                                .cursor_pointer()
                                .hover(|style| style.bg(rgb(0x059669)))
                                .text_color(rgb(0xffffff))
                                .text_sm()
                                .child("Add")
                                .on_mouse_down(gpui::MouseButton::Left, {
                                    cx.listener(move |this, _event, _window, cx| {
                                        this.add_library_version(cx);
                                        this.focused_library_input = None;
                                        eprintln!("✅ Add version button clicked");
                                    })
                                })
                        )
                        .child(
                            div()
                                .px_3()
                                .py_1()
                                .bg(rgb(0x6b7280))
                                .rounded(px(4.0))
                                .cursor_pointer()
                                .hover(|style| style.bg(rgb(0x4b5563)))
                                .text_color(rgb(0xffffff))
                                .text_sm()
                                .child("Cancel")
                                .on_mouse_down(gpui::MouseButton::Left, cx.listener(|this, _event, _window, cx| {
                                    this.show_version_input = false;
                                    this.new_version_name.clear();
                                    this.focused_library_input = None;
                                    cx.notify();
                                    eprintln!("❌ Cancel version clicked");
                                }))
                        )
                    })
                    .child(
                        // 版本列表
                        div()
                            .flex_1()
                            .bg(rgb(0x1f1f1f))
                            .border_1()
                            .border_color(rgb(0x2a2a2a))
                            .rounded(px(8.0))
                            .when(library.versions.is_empty(), |this| {
                                this.child(
                                    div()
                                        .p_4()
                                        .items_center()
                                        .justify_center()
                                        .child(
                                            div()
                                                .text_sm()
                                                .text_color(rgb(0x6b7280))
                                                .child("No versions")
                                        )
                                )
                            })
                            .when(!library.versions.is_empty(), |this| {
                                let mut list = this;
                                for version in &library.versions {
                                    let is_active = library.active_version_name(mappings) == Some(version.name.clone());
                                    let version_name = version.name.clone();
                                    let lib_id = library.id.clone();

                                    list = list.child(
                                        div()
                                            .px_3()
                                            .py_2()
                                            .cursor_pointer()
                                            .hover(|style| style.bg(rgb(0x374151)))
                                            .when(is_active, |el| {
                                                el.border_1()
                                                    .border_color(rgb(0x10b981))
                                                    .border_l_1()
                                            })
                                            .rounded(px(4.0))
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
                                                            .text_color(rgb(0xffffff))
                                                            .child(version_name.clone())
                                                    )
                                                    .when(is_active, |el| {
                                                        el.child(
                                                            div()
                                                                .px_2()
                                                                .py_1()
                                                                .bg(rgb(0x10b981))
                                                                .rounded(px(2.0))
                                                                .text_xs()
                                                                .text_color(rgb(0xffffff))
                                                                .child("Active")
                                                        )
                                                    })
                                            )
                                            .child(
                                                div()
                                                    .flex()
                                                    .gap_1()
                                                    .child(
                                                        div()
                                                            .px_2()
                                                            .py_1()
                                                            .hover(|style| style.bg(rgb(0x3b82f6)))
                                                            .rounded(px(2.0))
                                                            .cursor_pointer()
                                                            .text_color(rgb(0x9ca3af))
                                                            .text_xs()
                                                            .child("✓")
                                                            .on_mouse_down(gpui::MouseButton::Left, {
                                                                let lib_id_clone = lib_id.clone();
                                                                let version_name_clone = version_name.clone();
                                                                cx.listener(move |this, _event, _window, cx| {
                                                                    this.load_library_version(&lib_id_clone, &version_name_clone, cx);
                                                                })
                                                            })
                                                    )
                                                    .child(
                                                        div()
                                                            .px_2()
                                                            .py_1()
                                                            .hover(|style| style.bg(rgb(0xef4444)))
                                                            .rounded(px(2.0))
                                                            .cursor_pointer()
                                                            .text_color(rgb(0x9ca3af))
                                                            .text_xs()
                                                            .child("✕")
                                                            .on_mouse_down(gpui::MouseButton::Left, {
                                                                let lib_id_clone = lib_id.clone();
                                                                let version_name_clone = version_name.clone();
                                                                cx.listener(move |this, _event, _window, cx| {
                                                                    this.delete_library_version(&lib_id_clone, &version_name_clone, cx);
                                                                })
                                                            })
                                                    )
                                            )
                                    );
                                }
                                list
                            })
                    )
            )
        })
        .when(selected_library.is_none(), |this| {
            this.child(
                div()
                    .flex_1()
                    .bg(rgb(0x1f1f1f))
                    .border_1()
                    .border_color(rgb(0x2a2a2a))
                    .rounded(px(8.0))
                    .items_center()
                    .justify_center()
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0x6b7280))
                            .child("Select a library to view versions")
                    )
            )
        })
        .child(render_add_version_button(cx))
}

/// 右栏：通道配置（保持不变）
fn render_right_column(
    _libraries: &[SignalLibrary],
    selected_library_id: &Option<String>,
    _mappings: &[ChannelMapping]
) -> impl IntoElement {
    div()
        .flex_1()
        .flex()
        .flex_col()
        .gap_2()
        .child(
            div()
                .text_base()
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(0xffffff))
                .child("Channel Configuration")
        )
        .when(selected_library_id.is_some(), |this| {
            this.child(
                div()
                    .flex_1()
                    .bg(rgb(0x1f1f1f))
                    .border_1()
                    .border_color(rgb(0x2a2a2a))
                    .rounded(px(8.0))
                    .items_center()
                    .justify_center()
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0x6b7280))
                            .child("Channel configuration UI (to be implemented)")
                    )
            )
        })
        .when(selected_library_id.is_none(), |this| {
            this.child(
                div()
                    .flex_1()
                    .bg(rgb(0x1f1f1f))
                    .border_1()
                    .border_color(rgb(0x2a2a2a))
                    .rounded(px(8.0))
                    .items_center()
                    .justify_center()
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0x6b7280))
                            .child("No library selected")
                    )
            )
        })
}

/// 渲染过滤按钮
fn render_filter_button(label: &'static str, is_active: bool) -> impl IntoElement {
    div()
        .px_3()
        .py_1()
        .rounded(px(4.0))
        .cursor_pointer()
        .when(is_active, |el| {
            el.bg(rgb(0x3b82f6))
        })
        .when(!is_active, |el| {
            el.hover(|style| style.bg(rgb(0x374151)))
        })
        .text_color(rgb(0xffffff))
        .text_sm()
        .child(label)
}

/// 渲染库项（保持不变）
fn render_library_item(
    library: &SignalLibrary,
    selected_library_id: &Option<String>,
    mappings: &[ChannelMapping]
) -> impl IntoElement {
    let is_selected = selected_library_id.as_ref() == Some(&library.id);
    let is_used = library.is_used(mappings);

    div()
        .px_3()
        .py_2()
        .cursor_pointer()
        .hover(|style| style.bg(rgb(0x374151)))
        .when(is_selected, |el| {
            el.bg(rgb(0x3b82f6))
        })
        .rounded(px(4.0))
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
                        .text_color(rgb(0xffffff))
                        .child(library.name.clone())
                )
                .when(is_used, |el| {
                    el.child(
                        div()
                            .px_2()
                            .py_1()
                            .bg(rgb(0x10b981))
                            .rounded(px(2.0))
                            .text_xs()
                            .text_color(rgb(0xffffff))
                            .child("In Use")
                    )
                })
        )
        .child(
            div()
                .px_2()
                .py_1()
                .hover(|style| style.bg(rgb(0xef4444)))
                .rounded(px(2.0))
                .cursor_pointer()
                .text_color(rgb(0x9ca3af))
                .text_xs()
                .child("✕")
        )
}

/// 渲染添加库按钮（保持不变）
fn render_add_library_button(cx: &mut Context<crate::CanViewApp>) -> impl IntoElement {
    div()
        .w_full()
        .px_3()
        .py_2()
        .mt_2()
        .border_1()
        .border_color(rgb(0x2a2a2a))
        .border_dashed()
        .rounded(px(4.0))
        .cursor_pointer()
        .hover(|style| style.bg(rgb(0x374151)))
        .items_center()
        .justify_center()
        .child(
            div()
                .text_sm()
                .text_color(rgb(0x6b7280))
                .child("+ Add Library")
        )
        .on_mouse_down(gpui::MouseButton::Left, cx.listener(|this, _event, _window, cx| {
            this.show_library_dialog = true;
            this.library_dialog_type = LibraryDialogType::Create;
            this.focused_library_input = Some("new_library_input".to_string());
            this.is_editing_library_name = true;
            cx.notify();
            eprintln!("➕ Add library button clicked");
        }))
}

/// 渲染添加版本按钮（保持不变）
fn render_add_version_button(cx: &mut Context<crate::CanViewApp>) -> impl IntoElement {
    div()
        .w_full()
        .px_3()
        .py_2()
        .mt_2()
        .border_1()
        .border_color(rgb(0x2a2a2a))
        .border_dashed()
        .rounded(px(4.0))
        .cursor_pointer()
        .hover(|style| style.bg(rgb(0x374151)))
        .items_center()
        .justify_center()
        .child(
            div()
                .text_sm()
                .text_color(rgb(0x6b7280))
                .child("+ Add Version")
        )
        .on_mouse_down(gpui::MouseButton::Left, cx.listener(|this, _event, _window, cx| {
            if this.selected_library_id.is_some() {
                this.show_version_input = true;
                this.focused_library_input = Some("new_version_input".to_string());
                cx.notify();
                eprintln!("➕ Add version button clicked");
            }
        }))
}
