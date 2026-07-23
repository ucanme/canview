//! StatusBar component
//!
//! Renders the bottom status bar with file info (left) and server/library
//! status (right). Single 24px row.

use crate::app::{AppView, CanViewApp};
use crate::ui::theme::colors;
use crate::ui::theme::spacing;
use gpui::{prelude::*, *};

/// Format a count with thousands separators.
fn format_count(n: usize) -> String {
    let s = n.to_string();
    let bytes = s.as_bytes();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 && (bytes.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(*b as char);
    }
    out
}

/// Format a byte count with units (1024-based): 0B, 1023B, 1.0KB, 1.5MB, 2.3GB.
pub fn format_bytes(n: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;
    if n >= GB {
        format!("{:.1}GB", n as f64 / GB as f64)
    } else if n >= MB {
        format!("{:.1}MB", n as f64 / MB as f64)
    } else if n >= KB {
        format!("{:.1}KB", n as f64 / KB as f64)
    } else {
        format!("{}B", n)
    }
}

/// Render the file name segment (left side, segment 1).
fn render_file_segment(app: &CanViewApp) -> impl IntoElement {
    let text = app
        .current_file_name
        .clone()
        .unwrap_or_else(|| "No file loaded — File > Open BLF...".to_string());
    let color = if app.current_file_name.is_some() {
        colors::TEXT_SECONDARY
    } else {
        colors::TEXT_PLACEHOLDER
    };
    div()
        .flex()
        .items_center()
        .gap(px(6.))
        .child(div().text_color(colors::TEXT_MUTED).child("📂"))
        .child(div().text_color(color).child(text))
}

/// Render a vertical separator (1px wide, 12px tall).
fn render_separator() -> impl IntoElement {
    div().w(px(1.)).h(px(12.)).bg(colors::BORDER_SUBTLE)
}

/// Render the BLF bytes-progress segment: "521.0KB / 521.0KB (100.0%)".
/// Returns None when no file is loaded (blf_bytes_total == 0) so the
/// caller can skip rendering via .when_some().
fn render_blf_progress_segment(app: &CanViewApp) -> Option<impl IntoElement> {
    if app.blf_bytes_total == 0 {
        return None;
    }
    let total = format_bytes(app.blf_bytes_total);
    let consumed = format_bytes(app.blf_bytes_consumed);
    let pct = (app.blf_bytes_consumed as f64 / app.blf_bytes_total as f64) * 100.0;
    let text = format!("{} / {} ({:.1}%)", consumed, total, pct);
    // Color: green at 100%, yellow if < 100% (partial parse)
    let color = if pct >= 100.0 {
        colors::TEXT_SECONDARY
    } else {
        colors::WARNING
    };
    Some(div().text_color(color).child(text))
}

/// Render the server status segment (right side, segment 1).
fn render_server_segment(app: &CanViewApp, view: Entity<CanViewApp>) -> impl IntoElement {
    let running = app.server_handle.is_some();
    let url = app.share_url().map(|s| s.to_string()).unwrap_or_default();
    let view_for_click = view.clone();
    let url_for_copy = url.clone();
    let dot_color = if running {
        colors::SUCCESS
    } else {
        colors::DISABLED
    };
    let label = if running {
        format!("Server ON {}", url)
    } else {
        "Share disabled".to_string()
    };

    div()
        .flex()
        .items_center()
        .gap(px(6.))
        .cursor_pointer()
        .when(running, |el| el.hover(|s| s.bg(colors::SURFACE0)))
        .child(div().w(px(8.)).h(px(8.)).rounded_full().bg(dot_color))
        .child(div().text_color(colors::TEXT_MUTED).child(label))
        .when(running, |el| {
            el.on_mouse_down(MouseButton::Left, move |_, _, cx| {
                cx.stop_propagation();
                cx.write_to_clipboard(gpui::ClipboardItem::new_string(url_for_copy.clone()));
                view_for_click.update(cx, |app, cx| {
                    app.share_url_copied = true;
                    cx.notify();
                    // Reset after 2s
                    cx.spawn(async move |this, cx| {
                        smol::Timer::after(std::time::Duration::from_secs(2)).await;
                        if let Some(entity) = this.upgrade() {
                            entity.update(cx, |app, cx| {
                                app.share_url_copied = false;
                                cx.notify();
                            });
                        }
                    })
                    .detach();
                });
            })
        })
}

/// Render the library badge segment (right side, segment 3).
fn render_lib_badge_segment(app: &CanViewApp) -> impl IntoElement {
    if let (Some(lib_id), Some(ver)) = (&app.active_library_id, &app.active_version_name) {
        let lib_name = app
            .library_manager
            .find_library(lib_id)
            .map(|l| l.name.clone())
            .unwrap_or_else(|| lib_id.clone());
        let text = format!("📚 {} / {}", lib_name, ver);
        div()
            .text_color(colors::ACCENT_GREEN_LIGHT)
            .child(text)
            .into_any_element()
    } else {
        div().into_any_element()
    }
}

/// Render the status message segment (right side, segment 2).
///
/// Returns `Some(element)` when `app.status_msg` is non-empty, otherwise `None`
/// so the caller can skip the surrounding separators via `.when_some()`.
fn render_status_msg_segment(app: &CanViewApp) -> Option<impl IntoElement> {
    if app.status_msg.is_empty() {
        return None;
    }
    Some(
        div()
            .text_color(colors::TEXT_MUTED)
            .text_xs()
            .truncate()
            .child(app.status_msg.clone()),
    )
}

/// Render the current view name segment (right side, segment 4).
fn render_view_name_segment(view_val: AppView) -> impl IntoElement {
    let name = match view_val {
        AppView::LogView => "log mode",
        AppView::PlotView => "plot mode",
        AppView::LibraryView => "library mode",
        AppView::ConfigView => "config mode",
    };
    div().text_color(colors::TEXT_MUTED).child(name.to_string())
}

/// Render the data-view toggle (Log ⇄ Plot) in the center of the status bar.
///
/// Two mutually-exclusive buttons. Clicking one switches the current view to
/// that data view. When the app is in Library/Config view, both buttons are
/// rendered but neither is active — clicking either switches back to data
/// view (this is the "return to msg list" path).
fn render_data_view_toggle(app: &CanViewApp, view: Entity<CanViewApp>) -> impl IntoElement {
    let current = app.current_view;
    let log_active = current == AppView::LogView;
    let plot_active = current == AppView::PlotView;

    let view_for_log = view.clone();
    let view_for_plot = view.clone();

    div()
        .flex()
        .items_center()
        .gap(px(2.))
        .bg(colors::BG_DEFAULT)
        .rounded(px(4.))
        .border_1()
        .border_color(colors::BORDER_SUBTLE)
        .px(px(2.))
        // Log button
        .child(
            div()
                .px(spacing::SM)
                .py(px(1.))
                .rounded(px(3.))
                .cursor_pointer()
                .text_color(if log_active {
                    colors::TEXT_PRIMARY
                } else {
                    colors::TEXT_MUTED
                })
                .when(log_active, |el| {
                    el.bg(colors::PRIMARY).text_color(colors::BG_DEFAULT)
                })
                .hover(|s| {
                    if log_active {
                        s
                    } else {
                        s.bg(colors::SURFACE0).text_color(colors::TEXT_SECONDARY)
                    }
                })
                .child("Log")
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    cx.stop_propagation();
                    view_for_log.update(cx, |app, cx| {
                        app.current_view = AppView::LogView;
                        app.library_picker_dismissed = false;
                        cx.notify();
                    });
                }),
        )
        // Plot button
        .child(
            div()
                .px(spacing::SM)
                .py(px(1.))
                .rounded(px(3.))
                .cursor_pointer()
                .text_color(if plot_active {
                    colors::TEXT_PRIMARY
                } else {
                    colors::TEXT_MUTED
                })
                .when(plot_active, |el| {
                    el.bg(colors::PRIMARY).text_color(colors::BG_DEFAULT)
                })
                .hover(|s| {
                    if plot_active {
                        s
                    } else {
                        s.bg(colors::SURFACE0).text_color(colors::TEXT_SECONDARY)
                    }
                })
                .child("Plot")
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    cx.stop_propagation();
                    view_for_plot.update(cx, |app, cx| {
                        app.current_view = AppView::PlotView;
                        app.library_picker_dismissed = false;
                        crate::ui::views::chart_view::extract_and_update_series_data(app);
                        cx.notify();
                    });
                }),
        )
}

/// Render the StatusBar.
pub fn render_status_bar(app: &CanViewApp, view: Entity<CanViewApp>) -> impl IntoElement {
    let current_view = app.current_view;

    div()
        .h(px(24.))
        .bg(colors::BG_MUTED)
        .border_t_1()
        .border_color(colors::BORDER_SUBTLE)
        .flex()
        .items_center()
        .justify_between()
        .px(spacing::MD)
        .text_xs()
        // Left side: Log/Plot toggle | file | BLF progress | msgs | DBC | LDF
        .child(
            div()
                .flex()
                .items_center()
                .gap(spacing::SM)
                .child(render_data_view_toggle(app, view.clone()))
                .child(render_separator())
                .child(render_file_segment(app))
                .when_some(render_blf_progress_segment(app), |el, seg| {
                    el.child(render_separator()).child(seg)
                })
                .child(render_separator())
                .child(
                    div()
                        .text_color(colors::TEXT_MUTED)
                        .child(format!("{} msgs", format_count(app.messages.len()))),
                )
                .child(render_separator())
                .child(
                    div()
                        .text_color(colors::TEXT_MUTED)
                        .child(format!("DBC: {}", app.dbc_channels.len())),
                )
                .child(render_separator())
                .child(
                    div()
                        .text_color(colors::TEXT_MUTED)
                        .child(format!("LDF: {}", app.ldf_channels.len())),
                ),
        )
        // Right side: server | status_msg | lib badge | view name (separated by vertical bars)
        .child(
            div()
                .flex()
                .items_center()
                .gap(spacing::SM)
                .child(render_server_segment(app, view.clone()))
                .child(render_separator())
                .when_some(render_status_msg_segment(app), |el, status_seg| {
                    el.child(status_seg).child(render_separator())
                })
                .child(render_lib_badge_segment(app))
                .child(render_separator())
                .child(render_view_name_segment(current_view)),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_count_zero() {
        assert_eq!(format_count(0), "0");
    }

    #[test]
    fn test_format_count_small() {
        assert_eq!(format_count(123), "123");
    }

    #[test]
    fn test_format_count_thousands() {
        assert_eq!(format_count(12345), "12,345");
    }

    #[test]
    fn test_format_count_millions() {
        assert_eq!(format_count(1234567), "1,234,567");
    }

    #[test]
    fn test_format_count_exact_thousand() {
        assert_eq!(format_count(1000), "1,000");
    }

    #[test]
    fn test_format_bytes_zero() {
        assert_eq!(format_bytes(0), "0B");
    }

    #[test]
    fn test_format_bytes_small() {
        assert_eq!(format_bytes(1023), "1023B");
    }

    #[test]
    fn test_format_bytes_exact_kb() {
        assert_eq!(format_bytes(1024), "1.0KB");
    }

    #[test]
    fn test_format_bytes_kb_decimal() {
        assert_eq!(format_bytes(1536), "1.5KB");
    }

    #[test]
    fn test_format_bytes_exact_mb() {
        assert_eq!(format_bytes(1024 * 1024), "1.0MB");
    }

    #[test]
    fn test_format_bytes_mb_decimal() {
        assert_eq!(format_bytes(2_400_819), "2.3MB");
    }

    #[test]
    fn test_format_bytes_exact_gb() {
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.0GB");
    }

    #[test]
    fn test_format_bytes_gb_decimal() {
        assert_eq!(format_bytes(2_700_000_000), "2.5GB");
    }
}
