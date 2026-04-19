//! Three-column library management layout
//!
//! 按照LIBRARY_MANAGEMENT_REDESIGN.md文档实现的三栏布局：
//! - 左栏：库列表
//! - 中栏：版本列表
//! - 右栏：通道配置

use crate::app::LibraryDialogType;
use crate::models::{ChannelDatabase, ChannelMapping, LibraryVersion, SignalLibrary};
use gpui::prelude::*;
use gpui::*;
use gpui_component::input::{Input, InputState};

/// 渲染三栏布局的库管理界面 - Zed IDE 风格
///
/// 严格的三栏布局，中间用控制线隔开：
/// - 左栏：库列表（固定宽度）
/// - 中栏：版本列表（固定宽度）
/// - 右栏：通道配置（自适应宽度）
pub fn render_library_management_view(
    libraries: &[SignalLibrary],
    selected_library_id: &Option<String>,
    selected_version_id: &Option<String>, // Add selected version ID parameter
    mappings: &[ChannelMapping],
    show_new_library_input: bool,
    show_add_version_input: bool,
    new_library_name: &str,
    new_version_name: &str,
    focused_input: &Option<String>,
    library_cursor_pos: usize,
    version_cursor_pos: usize,
    library_name_input: Option<&gpui::Entity<gpui_component::input::InputState>>,
    version_name_input: Option<&gpui::Entity<gpui_component::input::InputState>>,
    show_add_channel_input: bool,
    channel_id_input: Option<&gpui::Entity<gpui_component::input::InputState>>,
    channel_name_input: Option<&gpui::Entity<gpui_component::input::InputState>>,
    channel_db_path_input: Option<&gpui::Entity<gpui_component::input::InputState>>,
    new_channel_db_path: &str, // Add this parameter to avoid reading entity in render
    new_channel_type: crate::models::ChannelType, // Add channel type parameter
    is_sharing: bool, // Whether the share server is currently running
    cx: &mut Context<crate::CanViewApp>,
) -> impl IntoElement {
    div()
        .flex_1()
        .flex()
        .flex_row()
        .bg(rgb(0x0a0a0a))
        .overflow_hidden() // 防止主容器滚动
        // 左栏：库列表
        .child(render_left_column(
            libraries,
            selected_library_id,
            mappings,
            show_new_library_input,
            new_library_name,
            focused_input,
            library_cursor_pos,
            library_name_input,
            is_sharing,
            cx,
        ))
        // 垂直分割线 1 - Zed IDE subtle divider
        .child(
            div()
                .w(px(1.0))
                .h_full()
                .bg(rgb(0x252525)) // Zed-style subtle divider
                .flex_shrink_0(),
        )
        // 中栏：版本列表
        .child(render_middle_column(
            libraries,
            selected_library_id,
            selected_version_id,
            mappings,
            show_add_version_input,
            new_version_name,
            focused_input,
            version_cursor_pos,
            version_name_input,
            cx,
        ))
        // 垂直分割线 2 - Zed IDE subtle divider
        .child(
            div()
                .w(px(1.0))
                .h_full()
                .bg(rgb(0x252525)) // Zed-style subtle divider
                .flex_shrink_0(),
        )
        // 右栏：通道配置
        .child(render_right_column(
            libraries,
            selected_library_id,
            selected_version_id,
            mappings,
            show_add_channel_input,
            channel_id_input,
            channel_name_input,
            channel_db_path_input,
            new_channel_db_path,
            new_channel_type,
            cx,
        ))
}

/// 左栏：库列表 - Zed IDE 风格
fn render_left_column(
    libraries: &[SignalLibrary],
    selected_library_id: &Option<String>,
    mappings: &[ChannelMapping],
    show_new_library_input: bool,
    new_library_name: &str,
    _focused_input: &Option<String>,
    _cursor_pos: usize,
    library_name_input: Option<&gpui::Entity<gpui_component::input::InputState>>,
    is_sharing: bool,
    cx: &mut Context<crate::CanViewApp>,
) -> impl IntoElement {
    div()
        .w(px(280.0)) // 稍微窄一点，更紧凑
        .h_full()
        .flex()
        .flex_col()
        .overflow_hidden()
        .child(
            // 固定的顶部区域：标题
            div()
                .flex()
                .items_center()
                .justify_between()
                .px_3()
                .py_2()
                .border_b_1()
                .border_color(rgb(0x252525))
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(rgb(0x6c7086)) // Zed muted
                        .child("LIBRARIES"),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(0x6c7086)) // Zed muted
                        .child(format!("{}", libraries.len())),
                ),
        )
        .child(
            // 可滚动的库列表
            div()
                .flex_1()
                .overflow_hidden()
                .px_2()
                .py_2()
                .when(libraries.is_empty() && !show_new_library_input, |this| {
                    this.child(
                        div().px_3().py_8().items_center().justify_center().child(
                            div()
                                .flex()
                                .flex_col()
                                .items_center()
                                .gap_3()
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(0x646473))
                                        .child("No libraries yet"),
                                )
                                .child(render_add_library_button(cx)),
                        ),
                    )
                })
                .when(!libraries.is_empty() || show_new_library_input, |this| {
                    let mut list = this;
                    // 如果正在添加，先显示输入行
                    if show_new_library_input {
                        list = list.child(render_add_library_input_row(
                            library_name_input,
                            new_library_name,
                            cx,
                        ));
                    }
                    // 然后显示所有库项
                    for library in libraries {
                        list = list.child(render_library_item(
                            library,
                            selected_library_id,
                            mappings,
                            cx,
                        ));
                    }
                    // 最后显示添加按钮
                    list.child(render_add_library_button(cx))
                }),
        )
        // Bottom action bar: Share & Import
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .px_3()
                .py_2()
                .border_t_1()
                .border_color(rgb(0x252525))
                .child({
                    div()
                        .flex_1()
                        .px_2()
                        .py_1()
                        .text_xs()
                        .text_color(if is_sharing { rgb(0xa6e3a1) } else { rgb(0xcdd6f4) })
                        .bg(rgb(0x1e1e2e))
                        .rounded(px(4.0))
                        .cursor_pointer()
                        .text_center()
                        .hover(|s| s.bg(rgb(0x313244)))
                        .on_mouse_down(gpui::MouseButton::Left, cx.listener(|this, _, _, cx| {
                            if this.server_handle.is_some() {
                                this.stop_share_server();
                            } else {
                                this.start_share_server();
                            }
                            cx.notify();
                        }))
                        .child(if is_sharing { "📡 Stop Share" } else { "📡 Share" })
                })
                .child(
                    div()
                        .flex_1()
                        .px_2()
                        .py_1()
                        .text_xs()
                        .text_color(rgb(0xcdd6f4))
                        .bg(rgb(0x1e1e2e))
                        .rounded(px(4.0))
                        .cursor_pointer()
                        .text_center()
                        .hover(|s| s.bg(rgb(0x313244)))
                        .on_mouse_down(gpui::MouseButton::Left, cx.listener(|this, _, _, cx| {
                            this.show_import_dialog = !this.show_import_dialog;
                            if this.show_import_dialog {
                                this.import_status = None;
                            } else {
                                this.import_url_input = None;
                            }
                            cx.notify();
                        }))
                        .child("📥 Import"),
                ),
        )
}

/// 渲染内联添加库输入行 - 完全融入列表
fn render_add_library_input_row(
    library_name_input: Option<&gpui::Entity<gpui_component::input::InputState>>,
    _new_library_name: &str,
    cx: &mut Context<crate::CanViewApp>,
) -> impl IntoElement {
    div()
        .px_3()
        .py_1p5()
        .h(px(32.))
        .border_1()
        .border_color(rgb(0x252525))
        .flex()
        .items_center()
        .on_key_down(cx.listener(|this, event: &KeyDownEvent, _window, cx| {
            if event.keystroke.key == "escape" {
                // Close the input without saving
                this.show_library_dialog = false;
                this.library_name_input = None;
                cx.notify();
            }
        }))
        .child(if let Some(input) = library_name_input {
            div()
                .flex_1()
                .child(Input::new(input).appearance(true))
                .into_any_element()
        } else {
            div()
                .text_color(gpui::rgb(0xffffff))
                .text_sm()
                .child("Library name...")
                .into_any_element()
        })
}

/// 渲染单个库项 - Zed IDE 风格
fn render_library_item(
    library: &SignalLibrary,
    selected_library_id: &Option<String>,
    mappings: &[ChannelMapping],
    cx: &mut Context<crate::CanViewApp>,
) -> impl IntoElement {
    let is_selected = selected_library_id.as_ref() == Some(&library.id);
    let is_used = library.is_used(mappings);
    let db_type = library.database_type();
    let icon = db_type.icon();
    let library_id = library.id.clone();

    div()
        .id(format!("lib-{}", library_id))
        .px_3()
        .py_1p5()
        .h(px(32.))
        .cursor_pointer()
        .hover(|style| style.bg(rgb(0x1a1a1a))) // 更微妙的悬停
        .when(is_selected, |el| {
            el.bg(rgb(0x252525)) // 选中时更深的背景
                .border_l_1()
                .border_color(rgb(0x89b4fa)) // Zed blue accent
        })
        .flex()
        .items_center()
        .justify_between()
        .on_mouse_down(
            gpui::MouseButton::Left,
            cx.listener({
                let library_id = library_id.clone();
                move |this, _event, _window, cx| {
                    cx.stop_propagation();
                    eprintln!("🖱️ Selected library: {}", library_id);
                    this.selected_library_id = Some(library_id.clone());
                    // Reset selected version when library changes
                    this.selected_version_id = None;
                    // Reset add channel input when switching libraries
                    this.hide_add_channel_input(cx);
                    cx.notify();
                }
            }),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .text_xs()
                        .text_color(if is_selected {
                            rgb(0x89b4fa) // Zed blue
                        } else {
                            rgb(0x6c7086) // Zed muted
                        })
                        .child(icon.to_string()),
                )
                .child(
                    div().flex().flex_col().gap_0().child(
                        div()
                            .text_sm()
                            .text_color(rgb(0xcdd6f4)) // Zed text
                            .child(library.name.clone()),
                    ),
                ),
        )
        .when(is_used, |el| {
            el.child(
                div()
                    .text_xs()
                    .text_color(rgb(0x6c7086)) // 使用文字标记
                    .child(format!("{}", library.versions.len())),
            )
        })
}

/// 渲染添加库按钮 - Zed IDE 风格
fn render_add_library_button(cx: &mut Context<crate::CanViewApp>) -> impl IntoElement {
    div()
        .id("add-lib-btn")
        .px_3()
        .py_1p5()
        .h(px(32.))
        .cursor_pointer()
        .hover(|style| style.bg(rgb(0x252525)))
        .flex()
        .items_center()
        .gap_2()
        .on_mouse_down(
            gpui::MouseButton::Left,
            cx.listener(|this, _event, window, cx| {
                cx.stop_propagation();
                eprintln!("🖱️ Add library button clicked");
                // Initialize input state when showing input
                if this.library_name_input.is_none() {
                    let input = cx
                        .new(|cx| InputState::new(window, cx).placeholder("Enter library name..."));

                    this.library_name_input = Some(input.clone());

                    // Subscribe to input events - store subscription to keep it alive
                    let _subscription = cx.subscribe(
                        &input,
                        |this: &mut crate::CanViewApp,
                         _input_entity,
                         event: &gpui_component::input::InputEvent,
                         cx| {
                            match event {
                                gpui_component::input::InputEvent::PressEnter { .. } => {
                                    eprintln!(
                                        "📥 Received PressEnter event from library_name_input"
                                    );
                                    let name = this
                                        .library_name_input
                                        .as_ref()
                                        .map(|i| i.read(cx).value().to_string())
                                        .unwrap_or_default();
                                    eprintln!("⏎ Creating library: '{}'", name);

                                    if !name.trim().is_empty() {
                                        this.new_library_name = name.clone();
                                        this.create_library(cx);
                                    }

                                    // Close the dialog
                                    this.show_library_dialog = false;
                                    this.library_name_input = None;
                                    cx.notify();
                                }
                                gpui_component::input::InputEvent::Change => {
                                    // Sync text to state
                                    let name = this
                                        .library_name_input
                                        .as_ref()
                                        .map(|i| i.read(cx).value().to_string())
                                        .unwrap_or_default();
                                    this.new_library_name = name;
                                }
                                _ => {}
                            }
                        },
                    );

                    eprintln!("✅ Created input and subscribed to events");
                }
                this.show_library_dialog = true;
                this.library_dialog_type = LibraryDialogType::Create;
                cx.notify();
            }),
        )
        .child(
            div()
                .text_sm()
                .text_color(rgb(0x6c7086)) // Zed muted
                .child("+"),
        )
        .child(
            div()
                .text_sm()
                .text_color(rgb(0x9399b2)) // Zed muted text
                .child("New Library"),
        )
}

/// 中栏：版本列表 - Zed IDE 风格
fn render_middle_column(
    libraries: &[SignalLibrary],
    selected_library_id: &Option<String>,
    selected_version_id: &Option<String>, // Add selected version ID parameter
    _mappings: &[ChannelMapping],
    show_add_version_input: bool,
    new_version_name: &str,
    _focused_input: &Option<String>,
    _cursor_pos: usize,
    version_name_input: Option<&gpui::Entity<gpui_component::input::InputState>>,
    cx: &mut Context<crate::CanViewApp>,
) -> impl IntoElement {
    // 找到选中的库
    let selected_library = selected_library_id
        .as_ref()
        .and_then(|id| libraries.iter().find(|lib| &lib.id == id));

    let version_count = selected_library.map(|l| l.versions.len()).unwrap_or(0);

    div()
        .w(px(280.0)) // 与左栏相同宽度
        .h_full()
        .flex()
        .flex_col()
        .overflow_hidden()
        .child(
            // 固定的顶部区域：标题
            div()
                .flex()
                .items_center()
                .justify_between()
                .px_3()
                .py_2()
                .border_b_1()
                .border_color(rgb(0x252525))
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(rgb(0x6c7086)) // Zed muted
                        .child("VERSIONS"),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(0x6c7086)) // Zed muted
                        .child(format!("{}", version_count)),
                ),
        )
        .child(
            // 可滚动的版本列表
            div()
                .flex_1()
                .overflow_hidden()
                .px_2()
                .py_2()
                .when(selected_library.is_none(), |this| {
                    this.child(
                        div().px_3().py_8().items_center().justify_center().child(
                            div().flex().flex_col().items_center().gap_3().child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0x646473))
                                    .child("Select a library first"),
                            ),
                        ),
                    )
                })
                .when_some(selected_library, |this, library| {
                    let mut list = this;
                    // 显示现有版本列表
                    for version in &library.versions {
                        let version_name = version.name.clone();
                        let is_selected = selected_version_id.as_ref() == Some(&version_name);
                        list =
                            list.child(render_version_item(version, version_name, is_selected, cx));
                    }
                    // 添加内联版本输入行（当show_add_version_input为true时）
                    if show_add_version_input {
                        list = list.child(render_add_version_input_row(
                            version_name_input,
                            new_version_name,
                            cx,
                        ));
                    }
                    // 添加"Add Version"按钮
                    list.child(render_add_version_button(cx))
                }),
        )
}

/// 渲染单个版本项 - Zed IDE 风格
fn render_version_item(
    version: &LibraryVersion,
    version_name: String,
    is_selected: bool,
    cx: &mut Context<crate::CanViewApp>,
) -> impl IntoElement {
    let stats = version.get_stats();

    div()
        .id(format!("ver-{}", version_name))
        .px_3()
        .py_1p5()
        .h(px(32.))
        .cursor_pointer()
        .when(is_selected, |el| {
            el.bg(rgb(0x252525))
                .border_l_1()
                .border_color(rgb(0x89b4fa)) // Zed blue
        })
        .hover(|style| style.bg(rgb(0x1a1a1a)))
        .flex()
        .items_center()
        .justify_between()
        .on_mouse_down(
            gpui::MouseButton::Left,
            cx.listener({
                let version_name = version_name.clone();
                move |this, _event, _window, cx| {
                    cx.stop_propagation();
                    eprintln!("🖱️ Selected version: {}", version_name);
                    this.selected_version_id = Some(version_name.clone());
                    this.status_msg = format!("Selected version: {}", version_name).into();
                    // Ensure add channel input is hidden when determining selection
                    this.hide_add_channel_input(cx);
                    cx.notify();
                }
            }),
        )
        .child(
            div().flex().flex_col().gap_0().child(
                div()
                    .text_sm()
                    .text_color(rgb(0xcdd6f4))
                    .child(version.name.clone()),
            ),
        )
        .child(
            div()
                .text_xs()
                .text_color(rgb(0x6c7086)) // Zed muted
                .child(format!("{}", stats.total_channels)),
        )
}

/// 右栏：通道配置 - Zed IDE 风格
fn render_right_column(
    libraries: &[SignalLibrary],
    selected_library_id: &Option<String>,
    selected_version_id: &Option<String>, // Add selected version ID parameter
    _mappings: &[ChannelMapping],
    show_add_channel_input: bool,
    channel_id_input: Option<&gpui::Entity<gpui_component::input::InputState>>,
    channel_name_input: Option<&gpui::Entity<gpui_component::input::InputState>>,
    channel_db_path_input: Option<&gpui::Entity<gpui_component::input::InputState>>,
    new_channel_db_path: &str, // Add this parameter to avoid reading entity in render
    new_channel_type: crate::models::ChannelType, // Use the new channel type being added
    cx: &mut Context<crate::CanViewApp>,
) -> impl IntoElement {
    // 找到选中的库和版本
    let selected_library = selected_library_id
        .as_ref()
        .and_then(|id| libraries.iter().find(|lib| &lib.id == id));

    // 使用选中的版本名称而不是最新版本
    let selected_version = selected_library.and_then(|lib| {
        selected_version_id
            .as_ref()
            .and_then(|version_name| lib.versions.iter().find(|v| &v.name == version_name))
    });

    let channel_count = selected_version
        .map(|v| v.channel_databases.len())
        .unwrap_or(0);

    // Use the passed new_channel_type parameter directly instead of getting from library
    let channel_type = new_channel_type;

    // Use the passed parameter instead of reading entity - this avoids borrow conflicts
    let (path_text, path_is_empty) = if show_add_channel_input {
        let is_empty = new_channel_db_path.is_empty();
        let text = if is_empty {
            "No file selected".to_string()
        } else {
            new_channel_db_path.to_string()
        };
        (text, is_empty)
    } else {
        (String::new(), true)
    };

    // Clone entity BEFORE entering .when_some() closure for file picker
    let entity_clone = if show_add_channel_input {
        Some(cx.entity().clone())
    } else {
        None
    };

    div()
        .flex_1() // 自适应剩余宽度
        .h_full()
        .flex()
        .flex_col()
        .overflow_hidden()
        .child(
            // 固定的顶部区域：标题
            div()
                .flex()
                .flex_col()
                .gap_2()
                .p_4()
                .border_b_1()
                .border_color(rgb(0x1a1a1a))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(rgb(0xcdd6f4)) // Zed 文本色
                                .child("Channel Configuration"),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_3()
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(0x646473)) // Zed muted
                                        .child(format!("{} channels", channel_count)),
                                )
                                .when_some(selected_version, |this, version| {
                                    let lib_id = selected_library_id.clone().unwrap_or_default();
                                    let ver_name = version.name.clone();
                                    this.child(
                                        div()
                                            .px_2()
                                            .py_1()
                                            .bg(rgb(0x3b82f6))
                                            .rounded(px(4.0))
                                            .cursor_pointer()
                                            .hover(|s| s.bg(rgb(0x2563eb)))
                                            .id("apply-version-btn")
                                            .on_mouse_down(gpui::MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                                cx.stop_propagation();
                                                this.apply_version_to_mappings(&lib_id, &ver_name, cx);
                                            }))
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .font_weight(FontWeight::BOLD)
                                                    .text_color(rgb(0xffffff))
                                                    .child("Apply to Plot")
                                            )
                                    )
                                }),
                        ),
                ),
        )
        .child(
            // 可滚动的通道列表
            div()
                .flex_1()
                .overflow_hidden()
                .flex()
                .flex_col()
                .when_some(selected_version, |this, _version| {
                    // 添加表头
                    this.child(
                        div()
                            .px_3()
                            .py_2()
                            .bg(rgb(0x0c0c0e))
                            .border_b_1()
                            .border_color(rgb(0x1a1a1a))
                            .flex()
                            .items_center()
                            .gap_3()
                            .child(
                                // Type 列
                                div().w(px(60.0)).flex_shrink_0().child(
                                    div()
                                        .text_xs()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(rgb(0x646473))
                                        .child("Type"),
                                ),
                            )
                            .child(
                                // CH 列（通道编号）
                                div().w(px(50.0)).flex_shrink_0().child(
                                    div()
                                        .text_xs()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(rgb(0x646473))
                                        .child("CH"),
                                ),
                            )
                            .child(
                                // Name 列
                                div().w(px(120.0)).flex_shrink_0().child(
                                    div()
                                        .text_xs()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(rgb(0x646473))
                                        .child("Name"),
                                ),
                            )
                            .child(
                                // Database Path 列
                                div().flex_1().min_w_0().child(
                                    div()
                                        .text_xs()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(rgb(0x646473))
                                        .child("Database Path"),
                                ),
                            )
                            .child(div().w(px(16.)).flex_shrink_0()),
                    )
                })
                .child(
                    div()
                        .flex_1()
                        .overflow_hidden()
                        .px_2()
                        .py_2()
                        .when(selected_version.is_none(), |this| {
                            this.child(
                                div().px_3().py_8().items_center().justify_center().child(
                                    div().flex().flex_col().items_center().gap_3().child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(0x646473))
                                            .child("Select a library and version to view channels"),
                                    ),
                                ),
                            )
                        })
                        .when_some(selected_version, |this, version| {
                            let mut list = this;
                            // 显示现有通道列表
                            for channel_db in &version.channel_databases {
                                list = list.child(render_channel_item(channel_db, cx));
                            }
                            // 显示输入框（如果show_add_channel_input为true）
                            if show_add_channel_input {
                                list = list.child(render_add_channel_input_row_with_path(
                                    channel_id_input,
                                    channel_name_input,
                                    channel_db_path_input,
                                    channel_type,
                                    path_text.clone(),    // Use pre-read value
                                    path_is_empty,        // Use pre-read value
                                    entity_clone.clone(), // Use pre-cloned entity
                                    cx,
                                ));
                            }
                            // 显示"Add Channel"按钮
                            list.child(render_add_channel_button(cx))
                        }),
                ),
        )
}

/// 渲染单个通道项 - 完整的单行列表显示
fn render_channel_item(
    channel_db: &ChannelDatabase,
    cx: &mut Context<crate::CanViewApp>,
) -> impl IntoElement {
    let path = channel_db.database_path.clone();
    let channel_name = channel_db.channel_name.clone();
    let channel_type = channel_db.channel_type;

    // Copy channel_id to avoid borrow issues in closure
    let channel_id = channel_db.channel_id;

    div()
        .px_3()
        .py_1()
        .mb_1()
        .h(px(32.))
        .bg(rgb(0x0c0c0e)) // Zed panel bg
        .border_1()
        .border_color(rgb(0x1a1a1a))
        .rounded(px(3.0))
        .flex()
        .items_center()
        .gap_3()
        .child(
            // 通道类型 - 固定宽度
            div().w(px(60.0)).flex_shrink_0().child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(if channel_type == crate::models::ChannelType::CAN {
                        rgb(0xa6e3a1) // Green for CAN
                    } else {
                        rgb(0xf9e2af) // Yellow for LIN
                    })
                    .child(if channel_type == crate::models::ChannelType::CAN {
                        "CAN"
                    } else {
                        "LIN"
                    }),
            ),
        )
        .child(
            // 通道ID - 固定宽度，只显示数字
            div().w(px(50.0)).flex_shrink_0().child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(rgb(0x89b4fa)) // Zed blue for ID
                    .child(format!("{}", channel_db.channel_id)),
            ),
        )
        .child(
            // 通道名称 - 固定宽度
            div().w(px(120.0)).flex_shrink_0().child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(rgb(0xcdd6f4)) // Zed text
                    .child(channel_name),
            ),
        )
        .child(
            div().flex_1().min_w_0().child(
                div()
                    .text_sm()
                    .text_color(rgb(0x646473)) // Zed muted
                    .truncate()
                    .child({
                        let normalized = path.replace('\\', "/");
                        if let Some(idx) = normalized.find("libraries/") {
                            normalized[idx..].to_string()
                        } else {
                            path
                        }
                    }),
            ),
        )
        .child(
            // 删除按钮
            div()
                .w(px(16.))
                .h(px(16.))
                .cursor_pointer()
                .hover(|style| style.bg(rgb(0x382828)))
                .rounded(px(2.))
                .flex()
                .items_center()
                .justify_center()
                .flex_shrink_0()
                .text_color(rgb(0x646473)) // Zed muted
                .hover(|style| style.text_color(rgb(0xf38ba8))) // Red on hover
                .child("🗑")
                .on_mouse_down(
                    gpui::MouseButton::Left,
                    cx.listener(move |this, _event, _window, cx| {
                        this.delete_channel(channel_id, cx);
                    }),
                ),
        )
}

fn render_add_channel_button(cx: &mut Context<crate::CanViewApp>) -> impl IntoElement {
    div()
        .px_3()
        .py_2()
        .mt_1()
        .border_1()
        .border_dashed()
        .border_color(rgb(0x45475a))
        .rounded(px(3.0))
        .cursor_pointer()
        .hover(|style| style.bg(rgb(0x1a1f2e)))
        .flex()
        .items_center()
        .justify_center()
        .gap_2()
        .on_mouse_down(
            gpui::MouseButton::Left,
            cx.listener(|this, _event, window, cx| {
                eprintln!("🖱️ Add Channel button clicked");

                // Clear previous values
                this.new_channel_db_path.clear();
                this.new_channel_id.clear();
                this.new_channel_name.clear();

                // Create fresh InputState entities in the event handler (not in render)
                let id_input = cx.new(|cx| InputState::new(window, cx).placeholder("Channel ID"));
                cx.subscribe(&id_input, |this, input, event, cx| {
                    if let gpui_component::input::InputEvent::Change = event {
                        this.new_channel_id = input.read(cx).text().to_string();
                    }
                })
                .detach();
                this.channel_id_input = Some(id_input);

                let name_input =
                    cx.new(|cx| InputState::new(window, cx).placeholder("Channel name"));
                cx.subscribe(&name_input, |this, input, event, cx| {
                    if let gpui_component::input::InputEvent::Change = event {
                        this.new_channel_name = input.read(cx).text().to_string();
                    }
                })
                .detach();
                this.channel_name_input = Some(name_input);

                this.show_add_channel_input = true;
                cx.notify();
                eprintln!("✅ show_add_channel_input = true");
            }),
        )
        .child(
            div().flex().items_center().gap_2().child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(rgb(0x7dcfff))
                    .child("+ Add Channel"),
            ),
        )
}

/// 渲染内联添加通道输入行 - 带预读取的路径和entity（避免在渲染时读取entity）
fn render_add_channel_input_row_with_path(
    channel_id_input: Option<&gpui::Entity<gpui_component::input::InputState>>,
    channel_name_input: Option<&gpui::Entity<gpui_component::input::InputState>>,
    _channel_db_path_input: Option<&gpui::Entity<gpui_component::input::InputState>>,
    channel_type: crate::models::ChannelType,
    path_text: String,
    path_is_empty: bool,
    entity_clone: Option<gpui::Entity<crate::CanViewApp>>, // Pre-cloned entity
    cx: &mut Context<crate::CanViewApp>,
) -> impl IntoElement {
    div()
        .px_3()
        .py_1()
        .h(px(32.))
        .border_1()
        .border_color(rgb(0x252525))
        .flex()
        .items_center()
        .gap_3()
        .on_key_down(cx.listener(|this, event: &KeyDownEvent, _window, cx| {
            if event.keystroke.key == "escape" {
                // Close the input without saving
                this.show_add_channel_input = false;
                this.channel_id_input = None;
                this.channel_name_input = None;
                this.channel_db_path_input = None;
                this.new_channel_db_path.clear(); // Clear selected path
                cx.notify();
            } else if event.keystroke.key == "enter" {
                // Save the channel configuration
                this.save_channel_config(cx);
            }
        }))
        .child(
            // 类型选择器 - 可点击切换，宽度与表头对齐
            div().w(px(60.0)).flex_shrink_0().child(
                div()
                    .px_2()
                    .py_1()
                    .bg(rgb(0x1a1a1a))
                    .rounded(px(2.0))
                    .text_color(if channel_type == crate::models::ChannelType::CAN {
                        rgb(0xa6e3a1) // Green for CAN
                    } else {
                        rgb(0xf9e2af) // Yellow for LIN
                    })
                    .text_sm()
                    .font_weight(FontWeight::MEDIUM)
                    .cursor_pointer()
                    .hover(|style| style.bg(rgb(0x2a2a2a)))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(if channel_type == crate::models::ChannelType::CAN {
                        "CAN"
                    } else {
                        "LIN"
                    })
                    .on_mouse_down(
                        gpui::MouseButton::Left,
                        cx.listener(|this, _event, _window, cx| {
                            // 切换通道类型
                            this.new_channel_type =
                                if this.new_channel_type == crate::models::ChannelType::CAN {
                                    crate::models::ChannelType::LIN
                                } else {
                                    crate::models::ChannelType::CAN
                                };
                            cx.notify();
                        }),
                    ),
            ),
        )
        .child(
            // 通道ID输入
            div()
                .w(px(50.0))
                .flex_shrink_0()
                .child(if let Some(input) = channel_id_input {
                    div()
                        .flex_1()
                        .child(Input::new(input))
                        .into_any_element()
                } else {
                    div()
                        .text_color(gpui::rgb(0xffffff))
                        .text_sm()
                        .child("ID...")
                        .into_any_element()
                }),
        )
        .child(
            // 通道名称输入
            div()
                .w(px(120.0))
                .flex_shrink_0()
                .child(if let Some(input) = channel_name_input {
                    div()
                        .flex_1()
                        .child(Input::new(input))
                        .into_any_element()
                } else {
                    div()
                        .text_color(gpui::rgb(0xffffff))
                        .text_sm()
                        .child("Name...")
                        .into_any_element()
                }),
        )
        .child(
            // 数据库路径显示 - 只读，通过Browse按钮选择
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .gap_2()
                .items_center()
                .child(
                    // 显示已选择的路径或提示文本
                    div().flex_1().min_w_0().child(
                        div()
                            .text_sm()
                            .text_color(if path_is_empty {
                                rgb(0x646473) // 灰色提示
                            } else {
                                rgb(0xcdd6f4) // 白色文本
                            })
                            .truncate()
                            .child(path_text),
                    ),
                )
                .child({
                    // 文件选择按钮 - 选择后自动保存
                    // 使用预先clone的entity，避免在 .when_some() 内部读取
                    if let Some(this) = entity_clone {
                        div()
                            .px_3()
                            .py_1()
                            .cursor_pointer()
                            .hover(|style| style.bg(rgb(0x313244)))
                            .rounded(px(2.0))
                            .border_1()
                            .border_color(rgb(0x45475a))
                            .text_color(rgb(0x7dcfff))
                            .text_xs()
                            .child("Select File...")
                            .on_mouse_down(gpui::MouseButton::Left, move |_event, _window, app| {
                                let this = this.clone();
                                app.spawn(async move |cx| {
                                    let dialog = rfd::AsyncFileDialog::new();
                                    
                                    let dialog = match channel_type {
                                        crate::models::ChannelType::CAN => dialog.add_filter("DBC Files", &["dbc"]),
                                        crate::models::ChannelType::LIN => dialog.add_filter("LDF Files", &["ldf"]),
                                    };

                                    if let Some(file) = dialog
                                        .pick_file()
                                        .await
                                    {
                                        let path_str = file.path().to_string_lossy().to_string();
                                        this.update(cx, |view, _cx| {
                                            // 保存文件路径
                                            view.new_channel_db_path = path_str.clone();
                                            eprintln!("📁 File selected: {}", path_str);

                                            // Auto-fill channel name from filename if empty
                                            if view.new_channel_name.is_empty() {
                                                if let Some(stem) = std::path::Path::new(&path_str).file_stem() {
                                                    view.new_channel_name = stem.to_string_lossy().to_string();
                                                }
                                            }

                                            // 自动保存通道配置
                                            // view.save_channel_config(cx); // Removed auto-save to allow user to input ID/Name after file selection
                                        });
                                    }
                                    Ok::<(), anyhow::Error>(())
                                })
                                .detach();
                            })
                    } else {
                        // Fallback if no entity clone (shouldn't happen)
                        div().child("Error: No entity")
                    }
                }),
        )
        .child(
            // 操作按钮：确认和取消
            div()
                .flex()
                .items_center()
                .gap_1()
                .ml_2()
                .flex_shrink_0()
                .child(
                    // 确认按钮
                    div()
                        .w(px(20.))
                        .h(px(20.))
                        .cursor_pointer()
                        .hover(|style| style.bg(rgb(0x313244)))
                        .rounded(px(3.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(
                            div()
                                .text_sm()
                                .text_color(rgb(0xa6e3a1)) // Green
                                .child("✓"),
                        )
                        .on_mouse_down(
                            gpui::MouseButton::Left,
                            cx.listener(|this, _, _, cx| {
                                eprintln!("🖱️ Confirm button clicked");
                                this.save_channel_config(cx);
                            }),
                        ),
                )
                .child(
                    // 取消按钮
                    div()
                        .w(px(20.))
                        .h(px(20.))
                        .cursor_pointer()
                        .hover(|style| style.bg(rgb(0x313244)))
                        .rounded(px(3.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(
                            div()
                                .text_sm()
                                .text_color(rgb(0xf38ba8)) // Red
                                .child("✕"),
                        )
                        .on_mouse_down(
                            gpui::MouseButton::Left,
                            cx.listener(|this, _, _, cx| {
                                this.hide_add_channel_input(cx);
                            }),
                        ),
                ),
        )
}

/// 渲染内联添加版本输入行 - 完全融入列表
fn render_add_version_input_row(
    version_name_input: Option<&gpui::Entity<gpui_component::input::InputState>>,
    _new_version_name: &str,
    cx: &mut Context<crate::CanViewApp>,
) -> impl IntoElement {
    div()
        .px_3()
        .py_1p5()
        .h(px(32.))
        .border_1()
        .border_color(rgb(0x252525))
        .flex()
        .items_center()
        .gap_2()
        .on_key_down(cx.listener(|this, event: &KeyDownEvent, _window, cx| {
            if event.keystroke.key == "escape" {
                // Close the input without saving
                this.show_version_input = false;
                this.version_name_input = None;
                cx.notify();
            }
        }))
        .child(
            div()
                .flex_1()
                .child(if let Some(input) = version_name_input {
                    div()
                        .flex_1()
                        .child(Input::new(input).appearance(true))
                        .into_any_element()
                } else {
                    div()
                        .text_color(gpui::rgb(0xffffff))
                        .text_sm()
                        .child("Version name...")
                        .into_any_element()
                }),
        )
}

/// 渲染添加版本按钮 - 简洁单行形式
fn render_add_version_button(cx: &mut Context<crate::CanViewApp>) -> impl IntoElement {
    div()
        .id("add-version-btn")
        .px_3()
        .py_1() // 减少padding，更紧凑
        .mb_1()
        .h(px(32.)) // 固定单行高度
        .border_1()
        .border_dashed()
        .border_color(rgb(0x45475a)) // Zed border
        .rounded(px(3.0))
        .cursor_pointer()
        .hover(|style| {
            style.bg(rgb(0x1a1f2e)) // Zed green hint on hover
        })
        .flex()
        .items_center()
        .gap_2()
        .on_mouse_down(
            gpui::MouseButton::Left,
            cx.listener(|this, _event, window, cx| {
                cx.stop_propagation();
                eprintln!("🖱️ Add version button clicked");
                // Initialize input state when showing input
                if this.version_name_input.is_none() {
                    let input = cx.new(|cx| {
                        InputState::new(window, cx)
                            .placeholder("Enter version name (e.g., v1.0)...")
                    });

                    this.version_name_input = Some(input);
                }
                this.show_version_input = true;
                cx.notify();
            }),
        )
        .child(
            div()
                .text_sm()
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgb(0x9399b2)) // Zed muted
                .child("+ Add Version"),
        )
}
