use anyhow::Result;
use blf::{read_blf_from_file, LogObject};
use gpui::*;

// Application state
struct AppState {
    messages: Vec<LogObject>,
    status_msg: String,
}

// Main view struct
struct MainView {
    messages: Vec<LogObject>,
    status_msg: String,
}

impl MainView {
    fn new() -> Self {
        Self {
            messages: Vec::new(),
            status_msg: "Ready".to_string(),
        }
    }
}

impl Render for MainView {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        let status_msg = self.status_msg.clone();
        let messages = self.messages.clone();
        
        // Create a scrollable container for the table
        div()
            .flex()
            .flex_col()
            .size_full()
            .child(
                // Toolbar
                div()
                    .h_10()
                    .bg(rgb(0xf0f0f0))
                    .border_b_1()
                    .border_color(rgb(0xccc))
                    .flex()
                    .items_center()
                    .px_4()
                    .gap_4()
                    .child(
                        Button::new("open_btn", "Open BLF File")
                            .on_click(|_event, cx| {
                                cx.spawn(|view, mut cx| async move {
                                    if let Some(path) = rfd::FileDialog::new()
                                        .add_filter("BLF Files", &["blf", "bin"])
                                        .pick_file() {
                                        
                                        // Update status
                                        view.update(&mut cx, |main_view, cx| {
                                            main_view.status_msg = "Loading...".to_string();
                                            cx.notify();
                                        }).ok();
                                        
                                        // Load the BLF file
                                        match read_blf_from_file(&path) {
                                            Ok(result) => {
                                                view.update(&mut cx, |main_view, cx| {
                                                    main_view.messages = result.objects;
                                                    main_view.status_msg = 
                                                        format!("{} objects", main_view.messages.len());
                                                    cx.notify();
                                                }).ok();
                                            }
                                            Err(e) => {
                                                view.update(&mut cx, |main_view, cx| {
                                                    main_view.status_msg = format!("Error: {:?}", e);
                                                    cx.notify();
                                                }).ok();
                                            }
                                        }
                                    }
                                })
                                .detach();
                            })
                    )
                    .child(div().w_px().h_5().bg(rgb(0xccc)))
                    .child(div().text_sm().text_color(rgb(0x555)).child(status_msg))
            )
            .child(
                // Messages table container
                div()
                    .flex_1()
                    .overflow_scroll()
                    .bg(rgb(0xffffff))
                    .child(
                        v_flex()
                            .w_full()
                            .child(
                                // Table header
                                h_flex()
                                    .bg(rgb(0xfafafa))
                                    .border_b_2()
                                    .border_color(rgb(0xdddd))
                                    .sticky()
                                    .top_0()
                                    .child(div().w_32().p_2().text_sm().child("Time"))
                                    .child(div().w_20().p_2().text_sm().child("Type"))
                                    .child(div().w_20().p_2().text_sm().child("ID"))
                                    .child(div().w_16().p_2().text_sm().child("DLC"))
                                    .child(div().flex_1().p_2().text_sm().child("Data"))
                            )
                            .children(
                                // Render first 500 messages
                                messages.into_iter()
                                    .take(500)
                                    .enumerate()
                                    .map(|(i, msg)| render_message_row(i, msg))
                                    .collect::<Vec<_>>()
                            )
                    )
            )
    }
}

// Helper function to render a message row
fn render_message_row(_index: usize, msg: LogObject) -> Div {
    let (timestamp, type_name, id, dlc, data_hex) = match msg {
        LogObject::CanMessage(m) => (
            m.header.object_time_stamp,
            "CAN".to_string(),
            format!("0x{:X}", m.id),
            m.dlc.to_string(),
            m.data.iter().take(m.dlc as usize)
                .map(|b| format!("{:02X}", b))
                .collect::<Vec<_>>()
                .join(" ")
        ),
        LogObject::CanFdMessage(m) => (
            m.header.object_time_stamp,
            "CAN FD".to_string(),
            format!("0x{:X}", m.id),
            m.valid_payload_length.to_string(),
            m.data.iter().take(m.valid_payload_length as usize)
                .map(|b| format!("{:02X}", b))
                .collect::<Vec<_>>()
                .join(" ")
        ),
        LogObject::CanErrorFrame(m) => (
            m.header.object_time_stamp,
            "CAN Error".to_string(),
            "-".to_string(),
            "0".to_string(),
            format!("Length: {}", m.length)
        ),
        LogObject::CanDriverError(m) => (
            m.header.object_time_stamp,
            "CAN Driver Error".to_string(),
            "-".to_string(),
            "0".to_string(),
            format!("Code: {}, TX: {}, RX: {}", m.error_code, m.tx_errors, m.rx_errors)
        ),
        _ => {
            // For other types, return empty values
            (0, "Other".to_string(), "-".to_string(), "0".to_string(), "-".to_string())
        }
    };

    h_flex()
        .border_b_1()
        .border_color(rgb(0xeeee))
        .hover(|style| style.bg(rgb(0xf9f9f9)))
        .child(div().w_32().p_1_5().text_sm().font_family("monospace").child(timestamp.to_string()))
        .child(div().w_20().p_1_5().text_sm().child(type_name))
        .child(div().w_20().p_1_5().text_sm().font_family("monospace").text_color(rgb(0x0078d4)).child(id))
        .child(div().w_16().p_1_5().text_sm().child(dlc))
        .child(div().flex_1().p_1_5().text_sm().font_family("monospace").child(data_hex))
}

fn main() {
    App::new().run(|cx| {
        // Set up the main window
        cx.open_window(WindowOptions::default(), |cx| {
            // Create the main view
            cx.new_view(|_cx| MainView::new())
        })
        .unwrap();
    });
}