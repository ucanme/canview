//! Interactive IME (Input Method Editor) Test
//!
//! This example demonstrates how text input works in GPUI and helps
//! diagnose IME support issues.
//!
//! Run with:
//!   cargo run --example test_ime_input

use gpui::{prelude::*, *};

struct ImeTestApp {
    input_text: String,
    cursor_position: usize,
    focused: bool,
    event_log: Vec<String>,
}

impl ImeTestApp {
    fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            input_text: String::new(),
            cursor_position: 0,
            focused: false,
            event_log: vec![
                "IME Input Test Started".to_string(),
                "Try typing Chinese using Pinyin input method".to_string(),
                "Example: type 'nihao' and select '你好'".to_string(),
            ],
        }
    }

    fn log_event(&mut self, message: String) {
        self.event_log.push(message);
        if self.event_log.len() > 20 {
            self.event_log.remove(0);
        }
    }

    fn render_input_box(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let text = self.input_text.clone();
        let cursor_pos = self.cursor_position.min(text.chars().count());

        div()
            .flex()
            .flex_col()
            .gap_4()
            .p_4()
            .w(px(600.))
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::BOLD)
                    .text_color(rgb(0xcdd6f4))
                    .child("IME Input Test")
            )
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(0x646473))
                    .child("Instructions: Type Chinese using Pinyin input method (e.g., 'nihao' → '你好')")
            )
            // Input box
            .child(
                div()
                    .border_1()
                    .border_color(if self.focused {
                        rgb(0x89b4fa)
                    } else {
                        rgb(0x2a2a2a)
                    })
                    .rounded(px(4.))
                    .bg(rgb(0x1a1a1a))
                    .p_4()
                    .min_h(px(60.))
                    .cursor_text()
                    .focusable()
                    .id("ime_test_input")
                    .on_click({
                        let view = cx.entity().clone();
                        move |_event, _window, cx| {
                            view.update(cx, |this, cx| {
                                this.focused = true;
                                this.log_event("Input box clicked".to_string());
                                cx.focus(&"ime_test_input");
                                cx.notify();
                            });
                        }
                    })
                    .when(text.is_empty(), |div| {
                        div.child(
                            div()
                                .text_color(rgb(0x646473))
                                .child("Type here...")
                        )
                    })
                    .when(!text.is_empty(), |div| {
                        let chars: Vec<char> = text.chars().collect();
                        let before_cursor: String = chars[..cursor_pos].iter().collect();
                        let after_cursor: String = chars[cursor_pos..].iter().collect();

                        div.child(
                            div()
                                .flex()
                                .items_center()
                                .gap_0()
                                .child(
                                    div()
                                        .text_color(rgb(0xcdd6f4))
                                        .child(before_cursor)
                                )
                                .child(
                                    div()
                                        .w(px(2.))
                                        .h(px(18.))
                                        .bg(rgb(0x89b4fa))
                                )
                                .child(
                                    div()
                                        .text_color(rgb(0xcdd6f4))
                                        .child(after_cursor)
                                )
                        )
                    })
                    .on_key_down({
                        let view = cx.entity().clone();
                        move |event, _window, cx| {
                            let keystroke = format!("{}", event.keystroke);
                            let key_text = event.keystroke.key.as_str();

                            view.update(cx, |this, cx| {
                                // Log key events
                                this.log_event(format!(
                                    "on_key_down: keystroke='{}' key='{}' len={}",
                                    keystroke,
                                    key_text,
                                    keystroke.len()
                                ));

                                match keystroke.as_str() {
                                    "backspace" => {
                                        if this.cursor_position > 0 {
                                            let mut chars: Vec<char> = this.input_text.chars().collect();
                                            chars.remove(this.cursor_position - 1);
                                            this.input_text = chars.into_iter().collect();
                                            this.cursor_position -= 1;
                                            this.log_event(format!("Backspace: text='{}'", this.input_text));
                                        }
                                    }
                                    "enter" => {
                                        this.log_event(format!("Enter: Submitting '{}'", this.input_text));
                                    }
                                    "escape" => {
                                        this.input_text.clear();
                                        this.cursor_position = 0;
                                        this.log_event("Escape: Cleared".to_string());
                                    }
                                    "left" => {
                                        if this.cursor_position > 0 {
                                            this.cursor_position -= 1;
                                            this.log_event(format!("Left: cursor={}", this.cursor_position));
                                        }
                                    }
                                    "right" => {
                                        let text_len = this.input_text.chars().count();
                                        if this.cursor_position < text_len {
                                            this.cursor_position += 1;
                                            this.log_event(format!("Right: cursor={}", this.cursor_position));
                                        }
                                    }
                                    "home" => {
                                        this.cursor_position = 0;
                                        this.log_event("Home: cursor=0".to_string());
                                    }
                                    "end" => {
                                        this.cursor_position = this.input_text.chars().count();
                                        this.log_event(format!("End: cursor={}", this.cursor_position));
                                    }
                                    _ => {
                                        // Character input
                                        if keystroke.len() == 1 {
                                            if let Some(ch) = keystroke.chars().next() {
                                                if !ch.is_control() {
                                                    let mut chars: Vec<char> = this.input_text.chars().collect();
                                                    chars.insert(this.cursor_position, ch);
                                                    this.input_text = chars.into_iter().collect();
                                                    this.cursor_position += 1;
                                                    this.log_event(format!(
                                                        "Char '{}' (U+{:04X}): text='{}'",
                                                        ch,
                                                        ch as u32,
                                                        this.input_text
                                                    ));
                                                }
                                            }
                                        } else if keystroke.len() > 1 {
                                            // Multi-character input (possibly from IME?)
                                            this.log_event(format!(
                                                        "Multi-char keystroke (len={}): '{}', key_text: '{}'",
                                                        keystroke.len(),
                                                        keystroke,
                                                        key_text
                                            ));

                                            // Try inserting the entire keystroke
                                            if !keystroke.is_empty() {
                                                let mut chars: Vec<char> = this.input_text.chars().collect();
                                                for ch in keystroke.chars() {
                                                    chars.insert(this.cursor_position, ch);
                                                    this.cursor_position += 1;
                                                }
                                                this.input_text = chars.into_iter().collect();
                                                this.log_event(format!("Inserted: text='{}'", this.input_text));
                                            }
                                        }
                                    }
                                }
                                cx.notify();
                            });
                        }
                    })
            )
            // Current state info
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .text_base()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(rgb(0x89b4fa))
                            .child("Current State")
                    )
                    .child(
                        div()
                            .flex()
                            .gap_4()
                            .text_sm()
                            .text_color(rgb(0xcdd6f4))
                            .child(format!("Text: '{}'", self.input_text))
                            .child(format!("Chars: {}", self.input_text.chars().count()))
                            .child(format!("Bytes: {}", self.input_text.len()))
                            .child(format!("Cursor: {}", self.cursor_position))
                    )
            )
            // Event log
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .text_base()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(rgb(0x89b4fa))
                            .child("Event Log")
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .max_h(px(300.))
                            .overflow_y_scroll()
                            .text_xs()
                            .font_family("monospace")
                            .text_color(rgb(0x646473))
                            .children(self.event_log.iter().rev().map(|log| {
                                div()
                                    .border_b_1()
                                    .border_color(rgb(0x2a2a2a))
                                    .py_1()
                                    .child(log.clone())
                            }))
                    )
            )
            // Instructions
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .p_3()
                    .bg(rgb(0x1a1a1a))
                    .rounded(px(4.))
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(rgb(0xfab387))
                            .child("What to Test:")
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0xcdd6f4))
                            .child("1. Type 'nihao' using Pinyin input method")
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0xcdd6f4))
                            .child("2. Select '你好' from the candidate list")
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0xcdd6f4))
                            .child("3. Observe the event log to see what events are received")
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0xcdd6f4))
                            .child("4. If you see '你好' in the text box, IME works!")
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0xf38ba8))
                            .child("⚠️  If you only see 'nihao', IME is NOT working properly")
                    )
            )
    }
}

impl Render for ImeTestApp {
    fn render(&mut self, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .bg(rgb(0x0c0c0e))
            .child(self.render_input_box(_cx))
    }
}

fn main() {
    env_logger::init();

    let app = Application::new();
    app.run(move |cx| {
        cx.spawn(async move |cx| {
            let options = WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point::new(px(200.0), px(150.0)),
                    size: gpui::Size {
                        width: px(800.0),
                        height: px(900.0),
                    },
                })),
                titlebar: Some(TitlebarOptions {
                    title: Some("IME Input Test".into()),
                    appears_transparent: false,
                    traffic_light_position: None,
                }),
                kind: gpui::WindowKind::Normal,
                ..Default::default()
            };

            cx.open_window(options, |_window, cx| cx.new(|_cx| ImeTestApp::new()))?;

            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
}
