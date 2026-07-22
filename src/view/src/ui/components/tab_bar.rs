//! TabBar component
//!
//! Renders the 4 view tabs (Log / Signal Plot / Library) with active
//! state indicated by a bottom 2px indicator (Zed style).

use crate::app::{AppView, CanViewApp};
use crate::ui::theme::colors;
use crate::ui::theme::spacing;
use gpui::{prelude::*, *};

/// Tab labels and their AppView variants. Returns (label, view).
///
/// Note: "File" is rendered as a Button in TopBar, not in TabBar.
pub const TAB_ITEMS: [(&str, AppView); 3] = [
    ("Log", AppView::LogView),
    ("Signal Plot", AppView::PlotView),
    ("Library", AppView::LibraryView),
];

/// Render the tab bar (3 tabs).
pub fn render_tab_bar(app: &CanViewApp, view: Entity<CanViewApp>) -> impl IntoElement {
    let current = app.current_view;
    div()
        .flex()
        .items_center()
        .gap(px(2.))
        .h_full()
        .children(TAB_ITEMS.map(|(label, view_val)| {
            let active = current == view_val;
            let view_clone = view.clone();
            div()
                .px(spacing::SM)
                .h_full()
                .flex()
                .items_center()
                .cursor_pointer()
                .text_sm()
                .text_color(if active {
                    colors::TEXT_PRIMARY
                } else {
                    colors::TEXT_MUTED
                })
                .hover(move |s| {
                    if active {
                        s
                    } else {
                        s.text_color(colors::TEXT_SECONDARY).bg(colors::SURFACE0)
                    }
                })
                .when(active, |el| el.border_b_2().border_color(colors::PRIMARY))
                .child(label.to_string())
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    cx.stop_propagation();
                    view_clone.update(cx, |app, cx| {
                        app.current_view = view_val;
                        if view_val == AppView::PlotView {
                            crate::ui::views::chart_view::extract_and_update_series_data(app);
                        }
                        cx.notify();
                    });
                })
        }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_items_count() {
        assert_eq!(TAB_ITEMS.len(), 3);
    }

    #[test]
    fn test_tab_items_unique_views() {
        // All variants must be distinct
        assert_ne!(TAB_ITEMS[0].1, TAB_ITEMS[1].1);
        assert_ne!(TAB_ITEMS[1].1, TAB_ITEMS[2].1);
        assert_ne!(TAB_ITEMS[0].1, TAB_ITEMS[2].1);
    }

    #[test]
    fn test_tab_items_labels_not_empty() {
        for (label, _) in TAB_ITEMS.iter() {
            assert!(!label.is_empty());
        }
    }
}
