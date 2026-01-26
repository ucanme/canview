//! Library management UI components

use crate::CanViewApp;
use crate::library::LibraryManager;
use crate::models::{ChannelType, DatabaseType, LibraryVersion, SignalLibrary};
use gpui::prelude::*;
use gpui::*;
use gpui_component::input::Input;

/// Library list component
pub fn render_library_list(
    libraries: &[SignalLibrary],
    selected_id: &Option<String>,
    mappings: &[crate::models::ChannelMapping],
    cx: &mut Context<CanViewApp>,
) -> impl IntoElement {
    if libraries.is_empty() {
        return div()
            .flex_1()
            .p_4()
            .bg(rgb(0x1f1f1f))
            .border_1()
            .border_color(rgb(0x2a2a2a))
            .rounded(px(8.))
            .items_center()
            .justify_center()
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(0x6b7280))
                    .child("No libraries configured"),
            );
    }

    let mut list = div()
        .flex_1()
        .bg(rgb(0x1f1f1f))
        .border_1()
        .border_color(rgb(0x2a2a2a))
        .rounded(px(8.))
        .flex()
        .flex_col()
        .gap_1();

    for library in libraries {
        let is_selected = selected_id.as_ref() == Some(&library.id);
        let lib_id = library.id.clone();
        let lib_name = library.name.clone();
        let version_count = library.versions.len();
        let db_type = library.database_type();
        let db_icon = db_type.icon().to_string();
        let is_used = library.is_used(mappings);

        list = list.child(
            div()
                .px_3()
                .py_2()
                .cursor_pointer()
                .hover(|style| style.bg(rgb(0x374151)))
                .when(is_selected, |el| el.bg(rgb(0x3b82f6)))
                .rounded(px(4.))
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(div().text_sm().text_color(rgb(0xffffff)).child(db_icon))
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
                                        .child(lib_name.clone()),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(0x9ca3af))
                                        .child(format!("{} versions", version_count)),
                                ),
                        ),
                )
                .when(is_used, |el| {
                    el.child(
                        div()
                            .px_2()
                            .py_1()
                            .bg(rgb(0x10b981))
                            .rounded(px(3.))
                            .text_xs()
                            .text_color(rgb(0xffffff))
                            .child("In Use"),
                    )
                })
                .on_mouse_down(
                    gpui::MouseButton::Left,
                    cx.listener(move |this, _event, _window, cx| {
                        this.selected_library_id = Some(lib_id.clone());
                        cx.notify();
                    }),
                ),
        );
    }

    list
}

/// Version details component
pub fn render_version_details(
    library: &SignalLibrary,
    mappings: &[crate::models::ChannelMapping],
    cx: &mut Context<CanViewApp>,
) -> impl IntoElement {
    let versions = &library.versions;

    if versions.is_empty() {
        return div().flex_1().flex().items_center().justify_center().child(
            div()
                .text_sm()
                .text_color(rgb(0x6b7280))
                .child("No versions available"),
        );
    }

    div()
        .flex_1()
        .flex()
        .flex_col()
        .gap_2()
        .children(versions.iter().map(|version| {
            let is_active = library.active_version_name(mappings) == Some(version.name.clone());
            let lib_id = library.id.clone();
            let version_name = version.name.clone();

            div()
                .p_4()
                .bg(rgb(0x1f1f1f))
                .border_1()
                .border_color(rgb(0x2a2a2a))
                .rounded(px(8.))
                .when(is_active, |el| el.border_2().border_color(rgb(0x10b981)))
                .flex()
                .flex_col()
                .gap_2()
                .child(
                    div().flex().items_center().justify_between().child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(0xffffff))
                                    .child(version.name.clone()),
                            )
                            .when(is_active, |el| {
                                el.child(
                                    div()
                                        .px_2()
                                        .py_1()
                                        .bg(rgb(0x10b981))
                                        .rounded(px(3.))
                                        .text_xs()
                                        .text_color(rgb(0xffffff))
                                        .child("Active"),
                                )
                            }),
                    ),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(0x9ca3af))
                        .child(format!("Date: {}", version.date)),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(0x9ca3af))
                        .child(format!("Path: {}", version.path)),
                )
                .when(!version.description.is_empty(), |el| {
                    el.child(
                        div()
                            .text_sm()
                            .text_color(rgb(0x9ca3af))
                            .child(format!("Description: {}", version.description)),
                    )
                })
                .child(
                    div()
                        .flex()
                        .gap_2()
                        .child({
                            let lib_id_clone = lib_id.clone();
                            let version_name_clone = version_name.clone();
                            div()
                                .px_3()
                                .py_1()
                                .bg(rgb(0x3b82f6))
                                .rounded(px(4.))
                                .cursor_pointer()
                                .hover(|style| style.bg(rgb(0x2563eb)))
                                .text_color(rgb(0xffffff))
                                .text_sm()
                                .child("Load")
                                .on_mouse_down(
                                    gpui::MouseButton::Left,
                                    cx.listener(move |this, _event, _window, cx| {
                                        this.load_library_version(
                                            &lib_id_clone,
                                            &version_name_clone,
                                            cx,
                                        );
                                    }),
                                )
                        })
                        .child({
                            let lib_id_clone = lib_id.clone();
                            let version_name_clone = version_name.clone();
                            div()
                                .px_3()
                                .py_1()
                                .bg(rgb(0xef4444))
                                .rounded(px(4.))
                                .cursor_pointer()
                                .hover(|style| style.bg(rgb(0xdc2626)))
                                .text_color(rgb(0xffffff))
                                .text_sm()
                                .child("Delete")
                                .on_mouse_down(
                                    gpui::MouseButton::Left,
                                    cx.listener(move |this, _event, _window, cx| {
                                        this.delete_library_version(
                                            &lib_id_clone,
                                            &version_name_clone,
                                            cx,
                                        );
                                    }),
                                )
                        }),
                )
        }))
}

/// Library management dialog
pub fn render_library_dialog(
    show: bool,
    dialog_type: LibraryDialogType,
    library_name_input: Option<&gpui::Entity<gpui_component::input::InputState>>,
    version_name_input: Option<&gpui::Entity<gpui_component::input::InputState>>,
    cx: &mut Context<CanViewApp>,
) -> impl IntoElement {
    if !show {
        return div();
    }

    let title = match dialog_type {
        LibraryDialogType::Create => "Create New Library",
        LibraryDialogType::AddVersion => "Add New Version",
    };

    div()
        .absolute()
        .inset_0()
        .bg(rgb(0x000000))
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .w(px(600.))
                .bg(rgb(0x1f1f1f))
                .border_1()
                .border_color(rgb(0x2a2a2a))
                .rounded(px(12.))
                .p_6()
                .flex()
                .flex_col()
                .gap_4()
                .child(
                    div()
                        .text_xl()
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(0xffffff))
                        .child(title),
                )
                .child(match dialog_type {
                    LibraryDialogType::Create => {
                        // Inline create library form with gpui-component Input
                        div()
                            .flex()
                            .flex_col()
                            .gap_4()
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_2()
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(rgb(0xffffff))
                                            .child("Library Name"),
                                    )
                                    .child(if let Some(input) = library_name_input {
                                        Input::new(input).into_any_element()
                                    } else {
                                        div()
                                            .px_3()
                                            .py_2()
                                            .bg(rgb(0x374151))
                                            .border_1()
                                            .border_color(rgb(0x2a2a2a))
                                            .rounded(px(4.))
                                            .text_color(rgb(0xffffff))
                                            .child("Enter library name...")
                                            .into_any_element()
                                    }),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_2()
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(rgb(0xffffff))
                                            .child("Library Type"),
                                    )
                                    .child(
                                        div()
                                            .px_3()
                                            .py_2()
                                            .bg(rgb(0x374151))
                                            .border_1()
                                            .border_color(rgb(0x2a2a2a))
                                            .rounded(px(4.))
                                            .text_color(rgb(0xffffff))
                                            .child("CAN (DBC)"),
                                    ),
                            )
                            .into_any_element()
                    }
                    LibraryDialogType::AddVersion => {
                        // Inline add version form with gpui-component Input
                        div()
                            .flex()
                            .flex_col()
                            .gap_4()
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_2()
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(rgb(0xffffff))
                                            .child("Version Name"),
                                    )
                                    .child(if let Some(input) = version_name_input {
                                        Input::new(input).into_any_element()
                                    } else {
                                        div()
                                            .px_3()
                                            .py_2()
                                            .bg(rgb(0x374151))
                                            .border_1()
                                            .border_color(rgb(0x2a2a2a))
                                            .rounded(px(4.))
                                            .text_color(rgb(0xffffff))
                                            .child("Enter version name...")
                                            .into_any_element()
                                    }),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_2()
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(rgb(0xffffff))
                                            .child("Database File"),
                                    )
                                    .child(
                                        div()
                                            .px_3()
                                            .py_2()
                                            .bg(rgb(0x374151))
                                            .border_1()
                                            .border_color(rgb(0x2a2a2a))
                                            .rounded(px(4.))
                                            .text_color(rgb(0xffffff))
                                            .child("Select DBC/LDF file..."),
                                    ),
                            )
                            .into_any_element()
                    }
                })
                .child(
                    div().flex().gap_2().justify_end().child(
                        div()
                            .px_4()
                            .py_2()
                            .bg(rgb(0x6b7280))
                            .rounded(px(4.))
                            .cursor_pointer()
                            .hover(|style| style.bg(rgb(0x4b5563)))
                            .text_color(rgb(0xffffff))
                            .child("Cancel")
                            .on_mouse_down(
                                gpui::MouseButton::Left,
                                cx.listener(|this, _event, _window, cx| {
                                    this.show_library_dialog = false;
                                    cx.notify();
                                }),
                            ),
                    ),
                ),
        )
}

/// Dialog type enum
pub enum LibraryDialogType {
    Create,
    AddVersion,
}

/// Create library form
fn render_create_library_form(cx: &mut Context<CanViewApp>) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_4()
        .child(
            div()
                .flex()
                .flex_col()
                .gap_2()
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(rgb(0xffffff))
                        .child("Library Name"),
                )
                .child(
                    div()
                        .px_3()
                        .py_2()
                        .bg(rgb(0x374151))
                        .border_1()
                        .border_color(rgb(0x2a2a2a))
                        .rounded(px(4.))
                        .text_color(rgb(0xffffff))
                        .child("Enter library name..."),
                ),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap_2()
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(rgb(0xffffff))
                        .child("Library Type"),
                )
                .child(
                    div()
                        .flex()
                        .gap_2()
                        .child(
                            div()
                                .flex_1()
                                .px_3()
                                .py_2()
                                .bg(rgb(0x3b82f6))
                                .border_1()
                                .border_color(rgb(0x2563eb))
                                .rounded(px(4.))
                                .text_color(rgb(0xffffff))
                                .child("CAN"),
                        )
                        .child(
                            div()
                                .flex_1()
                                .px_3()
                                .py_2()
                                .bg(rgb(0x374151))
                                .border_1()
                                .border_color(rgb(0x2a2a2a))
                                .rounded(px(4.))
                                .text_color(rgb(0xffffff))
                                .child("LIN"),
                        ),
                ),
        )
        .child(
            div()
                .px_4()
                .py_2()
                .bg(rgb(0x10b981))
                .rounded(px(4.))
                .cursor_pointer()
                .hover(|style| style.bg(rgb(0x059669)))
                .text_color(rgb(0xffffff))
                .child("Create Library"),
        )
}

/// Add version form
fn render_add_version_form(cx: &mut Context<CanViewApp>) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_4()
        .child(
            div()
                .flex()
                .flex_col()
                .gap_2()
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(rgb(0xffffff))
                        .child("Version Name"),
                )
                .child(
                    div()
                        .px_3()
                        .py_2()
                        .bg(rgb(0x374151))
                        .border_1()
                        .border_color(rgb(0x2a2a2a))
                        .rounded(px(4.))
                        .text_color(rgb(0xffffff))
                        .child("v1.0"),
                ),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap_2()
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(rgb(0xffffff))
                        .child("Database File"),
                )
                .child(
                    div()
                        .px_3()
                        .py_2()
                        .bg(rgb(0x374151))
                        .border_1()
                        .border_color(rgb(0x2a2a2a))
                        .rounded(px(4.))
                        .text_color(rgb(0x9ca3af))
                        .child("Click to select file..."),
                ),
        )
        .child(
            div()
                .px_4()
                .py_2()
                .bg(rgb(0x10b981))
                .rounded(px(4.))
                .cursor_pointer()
                .hover(|style| style.bg(rgb(0x059669)))
                .text_color(rgb(0xffffff))
                .child("Add Version"),
        )
}
