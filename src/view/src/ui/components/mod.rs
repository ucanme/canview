//! UI components

pub mod scrollbar;
pub mod filter_dropdown;
pub mod message_list;

use gpui::*;
use crate::models::AppView;
use crate::CanViewApp;

/// View button component
pub fn render_view_button(
    label: &str,
    view: AppView,
    current_view: AppView,
    cx: &mut Context<CanViewApp>,
) -> impl IntoElement {
    let is_active = current_view == view;
    let view_clone = view;

    div()
        .px_4()
        .py_2()
        .rounded(px(4.))
        .cursor_pointer()
        .when(is_active, |div| {
            div.bg(rgb(0x3b82f6))
        })
        .when(!is_active, |div| {
            div.hover(|style| style.bg(rgb(0x374151)))
        })
        .child(
            div()
                .text_sm()
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgb(0xffffff))
                .child(label)
        )
        .on_mouse_down(gpui::MouseButton::Left, {
            let view = cx.entity().clone();
            move |_event, _window, cx| {
                view.update(cx, |this, cx| {
                    this.current_view = view_clone;
                    cx.notify();
                });
            }
        })
}
