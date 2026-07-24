//! StatusBar component
//!
//! Renders the bottom status bar with file info (left) and server/library
//! status (right). Single 24px row.

use crate::app::{AppView, CanViewApp};
use crate::ui::theme::colors;
use crate::ui::theme::radius;
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
    // Clamp at 100% — consumed should never exceed total, but float math
    // can introduce tiny overshoot (e.g. 100.01%). The min() in the
    // parser already prevents this at the source; the cap here is a
    // last-line-of-defense so users never see >100%.
    let pct = ((app.blf_bytes_consumed as f64 / app.blf_bytes_total as f64) * 100.0).min(100.0);
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
/// When `blf_parse_errors` is non-empty, a clickable "details" link is
/// appended. The popover itself is rendered separately by
/// render_blf_errors_popover (as a sibling of the status bar) so it isn't
/// clipped by this segment's truncate().
fn render_status_msg_segment(app: &CanViewApp, view: Entity<CanViewApp>) -> Option<impl IntoElement> {
    if app.status_msg.is_empty() {
        return None;
    }
    let has_errors = !app.blf_parse_errors.is_empty();
    let view_for_details = view.clone();

    Some(
        div()
            .flex()
            .items_center()
            .gap(px(4.))
            .text_color(colors::TEXT_MUTED)
            .text_xs()
            .truncate()
            .child(app.status_msg.clone())
            .when(has_errors, |el| {
                el.child(
                    div()
                        .text_color(colors::WARNING)
                        .cursor_pointer()
                        .hover(|s| s.text_color(colors::TEXT_PRIMARY))
                        .child("› details")
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            cx.stop_propagation();
                            view_for_details.update(cx, |app, cx| {
                                app.show_blf_errors_popover = !app.show_blf_errors_popover;
                                cx.notify();
                            });
                        }),
                )
            }),
    )
}

/// Render the BLF errors popover as a standalone element (sibling of the
/// status bar) so it isn't clipped by the status_msg segment's truncate().
/// Returns None when no errors or when the popover is closed.
fn render_blf_errors_popover(app: &CanViewApp, view: Entity<CanViewApp>) -> Option<impl IntoElement> {
    if app.blf_parse_errors.is_empty() || !app.show_blf_errors_popover {
        return None;
    }
    let errors_count = app.blf_parse_errors.len();
    let errors: Vec<String> = app.blf_parse_errors.clone();
    let view_for_close = view.clone();
    let view_for_click_outside = view.clone();

    Some(
        div()
            .absolute()
            .top_0()
            .left_0()
            .w_full()
            .h_full()
            .bg(rgba(0x00000055))
            .flex()
            .items_end()
            .justify_end()
            .p(spacing::LG)
            .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                view_for_click_outside.update(cx, |app, cx| {
                    app.show_blf_errors_popover = false;
                    cx.notify();
                });
            })
            .child(
                div()
                    .w(px(460.))
                    .max_h(px(360.))
                    .bg(colors::BG_ELEVATED)
                    .border_1()
                    .border_color(colors::BORDER_DEFAULT)
                    .rounded(px(6.))
                    .shadow_lg()
                    .flex()
                    .flex_col()
                    .p(spacing::SM)
                    .gap(spacing::XS)
                    .on_mouse_down(MouseButton::Left, |_, _, cx| {
                        cx.stop_propagation();
                    })
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .text_color(colors::TEXT_PRIMARY)
                                    .text_sm()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(format!("BLF parse errors ({})", errors_count)),
                            )
                            .child(
                                div()
                                    .cursor_pointer()
                                    .text_color(colors::TEXT_MUTED)
                                    .hover(|s| s.text_color(colors::TEXT_PRIMARY))
                                    .child("✕")
                                    .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                        cx.stop_propagation();
                                        view_for_close.update(cx, |app, cx| {
                                            app.show_blf_errors_popover = false;
                                            cx.notify();
                                        });
                                    }),
                            ),
                    )
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .gap(px(2.))
                            .children(
                                errors
                                    .iter()
                                    .enumerate()
                                    .map(|(i, e)| {
                                        div()
                                            .text_color(colors::TEXT_MUTED)
                                            .text_xs()
                                            .child(format!("{}. {}", i + 1, e))
                                    })
                                    .collect::<Vec<_>>(),
                            ),
                    ),
            ),
    )
}

/// Render the 📁 Files (N) button segment (right side).
///
/// 仅在 `app.files` 非空时返回 `Some`，与 `render_blf_progress_segment` 的
/// “无文件则隐藏” 模式对称。点击切换 `app.show_files_popover`，由
/// `render_files_popover` 渲染对应的浮层。
fn render_files_button_segment(app: &CanViewApp, view: Entity<CanViewApp>) -> Option<impl IntoElement> {
    if app.files.is_empty() {
        return None;
    }
    let view_for_click = view.clone();
    Some(
        div()
            .flex()
            .items_center()
            .gap(px(4.))
            .px(spacing::SM)
            .py(px(1.))
            .rounded(px(3.))
            .text_color(colors::TEXT_MUTED)
            .cursor_pointer()
            .hover(|s| s.bg(colors::SURFACE0).text_color(colors::TEXT_PRIMARY))
            .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                cx.stop_propagation();
                view_for_click.update(cx, |app, cx| {
                    app.show_files_popover = !app.show_files_popover;
                    cx.notify();
                });
            })
            .child(format!("📁 Files ({})", app.files.len())),
    )
}

/// Render the file-management popover as a sibling of the status bar.
///
/// 镜像 `render_blf_errors_popover` 的定位方式：全屏绝对定位 + 半透明遮罩
/// 处理外部点击关闭；实际内容容器锚定到右下角，避免被 status_msg 段的
/// `truncate()` 裁切。仅当 `show_files_popover && !files.is_empty()` 时
/// 返回 `Some`；当文件在 popover 打开期间被全部移除时，popover 自动消失。
fn render_files_popover(app: &CanViewApp, view: Entity<CanViewApp>) -> Option<impl IntoElement> {
    if !app.show_files_popover || app.files.is_empty() {
        return None;
    }
    let files_count = app.files.len();
    // 预先把每行渲染成独立的 element，避免 GPUI children() 对 'static 生命周期
    // 的要求与局部 rows borrow 冲突（E0597）。每行的 on_mouse_down 闭包通过
    // move 捕获自己的 file_id + view clone，互不借用 rows。
    let row_elements: Vec<Div> = app
        .files
        .iter()
        .map(|seg| {
            let file_id = seg.file_id;
            let file_name = seg.file_name.clone();
            let msg_count = seg.object_count;
            let bytes_total = seg.bytes_total;
            let has_errors = !seg.errors.is_empty();
            let status_icon = if has_errors { "❌" } else { "✅" };
            let status_color = if has_errors {
                colors::WARNING
            } else {
                colors::SUCCESS
            };
            let row_bg = if has_errors {
                colors::SURFACE0
            } else {
                colors::BG_DEFAULT
            };
            let view_for_remove = view.clone();
            div()
                .p(spacing::XS)
                .bg(row_bg)
                .rounded(radius::MD)
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.))
                        .text_xs()
                        .truncate()
                        .child(div().text_color(status_color).child(status_icon))
                        .child(div().text_color(colors::TEXT_PRIMARY).child(file_name))
                        .child(
                            div()
                                .text_color(colors::TEXT_MUTED)
                                .child(format!(
                                    "{} msgs, {}",
                                    format_count(msg_count),
                                    format_bytes(bytes_total),
                                )),
                        )
                        .when(has_errors, |el| {
                            el.child(div().text_color(colors::WARNING).child("(errors)"))
                        }),
                )
                .child(
                    div()
                        .px(spacing::SM)
                        .text_xs()
                        .text_color(colors::ERROR)
                        .cursor_pointer()
                        .hover(|s| s.bg(colors::SURFACE0))
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            cx.stop_propagation();
                            view_for_remove.update(cx, |app, cx| {
                                app.remove_file(file_id);
                                cx.notify();
                            });
                        })
                        .child("✕"),
                )
        })
        .collect();
    let view_for_remove_all = view.clone();
    let view_for_done = view.clone();
    let view_for_title_close = view.clone();
    let view_for_click_outside = view.clone();

    Some(
        div()
            .absolute()
            .top_0()
            .left_0()
            .w_full()
            .h_full()
            .bg(rgba(0x00000055))
            .flex()
            .items_end()
            .justify_end()
            .p(spacing::LG)
            // 点击遮罩外部关闭 popover
            .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                view_for_click_outside.update(cx, |app, cx| {
                    app.show_files_popover = false;
                    cx.notify();
                });
            })
            .child(
                div()
                    .w(px(460.))
                    .max_h(px(360.))
                    .bg(colors::BG_ELEVATED)
                    .border_1()
                    .border_color(colors::BORDER_DEFAULT)
                    .rounded(radius::LG)
                    .shadow_lg()
                    .flex()
                    .flex_col()
                    .p(spacing::SM)
                    .gap(spacing::XS)
                    // 阻止 popover 内部点击冒泡到遮罩的关闭 handler
                    .on_mouse_down(MouseButton::Left, |_, _, cx| {
                        cx.stop_propagation();
                    })
                    // 标题栏：文件数 + ✕ 关闭
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .text_color(colors::TEXT_PRIMARY)
                                    .text_sm()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(format!("Loaded Files ({})", files_count)),
                            )
                            .child(
                                div()
                                    .cursor_pointer()
                                    .text_color(colors::TEXT_MUTED)
                                    .hover(|s| s.text_color(colors::TEXT_PRIMARY))
                                    .child("✕")
                                    .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                        cx.stop_propagation();
                                        view_for_title_close.update(cx, |app, cx| {
                                            app.show_files_popover = false;
                                            cx.notify();
                                        });
                                    }),
                            ),
                    )
                    // 文件列表（每行：状态图标 + 名称 + msgs + size + ✕ 移除）
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .gap(px(2.))
                            .children(row_elements),
                    )
                    // 底部：Remove All（左）+ Done（右）
                    .child(
                        div()
                            .mt(px(2.))
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .px(spacing::MD)
                                    .py(px(1.))
                                    .text_xs()
                                    .text_color(colors::ERROR)
                                    .cursor_pointer()
                                    .rounded(px(3.))
                                    .hover(|s| s.bg(colors::SURFACE0))
                                    .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                        cx.stop_propagation();
                                        view_for_remove_all.update(cx, |app, cx| {
                                            app.remove_all_files();
                                            cx.notify();
                                        });
                                    })
                                    .child("Remove All"),
                            )
                            .child(
                                div()
                                    .px(spacing::MD)
                                    .py(px(1.))
                                    .text_xs()
                                    .text_color(colors::TEXT_PRIMARY)
                                    .cursor_pointer()
                                    .rounded(px(3.))
                                    .hover(|s| s.bg(colors::SURFACE0))
                                    .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                        cx.stop_propagation();
                                        view_for_done.update(cx, |app, cx| {
                                            app.show_files_popover = false;
                                            cx.notify();
                                        });
                                    })
                                    .child("Done"),
                            ),
                    ),
            ),
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
        .relative()
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
        // Right side: server | status_msg | lib badge | files btn | view name (separated by vertical bars)
        .child(
            div()
                .flex()
                .items_center()
                .gap(spacing::SM)
                .child(render_server_segment(app, view.clone()))
                .child(render_separator())
                .when_some(render_status_msg_segment(app, view.clone()), |el, status_seg| {
                    el.child(status_seg).child(render_separator())
                })
                .child(render_lib_badge_segment(app))
                .when_some(render_files_button_segment(app, view.clone()), |el, files_btn| {
                    el.child(render_separator()).child(files_btn)
                })
                .child(render_separator())
                .child(render_view_name_segment(current_view)),
        )
        // Popover as a sibling of the status bar row, so it isn't clipped
        // by the status_msg segment's truncate().
        .when_some(
            render_blf_errors_popover(app, view.clone()),
            |el, popover| el.child(popover),
        )
        // 文件管理 popover，与 blf_errors_popover 同级渲染，避免被 status bar 裁切
        .when_some(render_files_popover(app, view.clone()), |el, popover| {
            el.child(popover)
        })
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
