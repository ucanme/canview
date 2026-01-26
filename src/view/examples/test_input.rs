//! Simple test for text input
use gpui::prelude::*;
use gpui::*;

fn main() {
    App::new().run(move |window, cx| {
        cx.show_window(window);
        window.set_title("Text Input Test", cx);
        window.resize(cx, px(800.0), px(600.0));

        let text_state = cx.entity().new(|cx| TextState::new(cx), cx);

        cx.focus(&text_state);

        window.set_content(
            div()
                .flex()
                .items_center()
                .justify_center()
                .size_full()
                .bg(rgb(0x1a1a1a))
                .child(
                    div()
                        .w(px(400.0))
                        .h(px(100.0))
                        .px_4()
                        .bg(rgb(0x2a2a2a))
                        .border_1()
                        .border_color(rgb(0x3b82f6))
                        .rounded(px(4.0))
                        .flex()
                        .items_center()
                        .id("test_input")
                        .focusable()
                        .on_key_down(cx.listener(|this, event: &KeyDownEvent, _window, cx| {
                            let keystroke = format!("{}", event.keystroke);
                            eprintln!("Key: '{}'", keystroke);
                            match keystroke.as_str() {
                                "backspace" => {
                                    eprintln!("Backspace");
                                }
                                "enter" => {
                                    eprintln!("Enter");
                                }
                                _ => {
                                    let first_char = keystroke.chars().next();
                                    if first_char.map(|c| !c.is_control()).unwrap_or(false) {
                                        eprintln!("✓ Inserted: '{}'", keystroke);
                                    }
                                }
                            }
                        }))
                        .child(
                            div()
                                .text_sm()
                                .text_color(rgb(0xffffff))
                                .child("Type here..."),
                        ),
                ),
        )
    })
}

struct TextState;

impl TextState {
    fn new(cx: &mut Context<Self>) -> Self {
        Self
    }
}
