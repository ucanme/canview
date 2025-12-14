#![allow(non_snake_case)]

use dioxus::prelude::*;
use blf::{read_blf_from_file, LogObject};


use dioxus::desktop::{Config, WindowBuilder};

fn main() {
    let config = Config::new()
        .with_window(WindowBuilder::new()
            .with_title("CanView")
            .with_resizable(true)
        );
    
    dioxus::desktop::launch::launch(App, vec![], config);
}

#[component]
fn App() -> Element {
    let mut messages = use_signal(|| Vec::<LogObject>::new());
    let mut status_msg = use_signal(|| "Ready".to_string());

    let handle_open_click = move |_| {
        spawn(async move {
            if let Some(path) = rfd::FileDialog::new().add_filter("BLF Files", &["blf", "bin"]).pick_file() {
                status_msg.set(format!("Loading..."));
                match read_blf_from_file(&path) {
                    Ok(result) => {
                        status_msg.set(format!("{} objects", result.objects.len()));
                        messages.set(result.objects);
                    }
                    Err(e) => status_msg.set(format!("Error: {:?}", e)),
                }
            }
        });
    };

    rsx! {
        div {
            style: "display: flex; flex-direction: column; height: 100vh; font-family: 'Segoe UI', sans-serif; overflow: hidden;",
            
            // Top Toolbar (Menu Bar)
            div {
                style: "height: 40px; background: #f0f0f0; border-bottom: 1px solid #ccc; display: flex; align-items: center; padding: 0 16px; gap: 16px; user-select: none;",
                
                button {
                    onclick: handle_open_click,
                    style: "background: #0078d4; color: white; border: none; font-size: 13px; cursor: pointer; padding: 6px 12px; border-radius: 4px; display: flex; align-items: center; gap: 8px; &:hover {{ background: #106ebe; }}",
                    span { "📂" } 
                    "Open BLF File"
                }
                
                div {
                    style: "width: 1px; height: 20px; background: #ccc;",
                }

                span {
                    style: "font-size: 13px; color: #555;",
                    "{status_msg}"
                }
            }

            // Content
            div {
                style: "flex: 1; overflow: auto; background: #fff;",
                table {
                    style: "width: 100%; border-collapse: collapse; font-size: 13px;",
                    thead {
                        tr {
                            style: "text-align: left; background: #fafafa; border-bottom: 2px solid #ddd; position: sticky; top: 0;",
                            th { style: "padding: 8px; width: 120px;", "Time" }
                            th { style: "padding: 8px; width: 80px;", "Type" }
                            th { style: "padding: 8px; width: 80px;", "ID" }
                            th { style: "padding: 8px; width: 60px;", "DLC" }
                            th { style: "padding: 8px;", "Data" }
                        }
                    }
                    tbody {
                        for (i, msg) in messages.read().iter().enumerate().take(500) {
                            MessageRow { key: "{i}", msg: msg.clone() }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn MessageRow(msg: LogObject) -> Element {
    // Extract data based on type
    let (timestamp, type_name, id, dlc, data_hex) = match &msg {
        LogObject::CanMessage(m) => (
            m.header.object_time_stamp,
            "CAN",
            format!("0x{:X}", m.id),
            m.dlc,
            // Format data as hex string
            m.data.iter().take(m.dlc as usize).map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ")
        ),
        LogObject::CanFdMessage(m) => (
             m.header.object_time_stamp,
            "CAN FD",
            format!("0x{:X}", m.id),
            m.valid_payload_length,
             m.data.iter().take(m.valid_payload_length as usize).map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ")
        ),
        LogObject::CanErrorFrame(m) => (
            m.header.object_time_stamp,
            "CAN Error",
            "-".to_string(),
            0,
            format!("Length: {}", m.length)
        ),
        LogObject::CanDriverError(m) => (
            m.header.object_time_stamp,
            "CAN Driver Error",
            "-".to_string(),
            0,
            format!("Code: {}, TX: {}, RX: {}", m.error_code, m.tx_errors, m.rx_errors)
        ),
        _ => {
            // For now, skip other types or show generic
            return rsx! {};
        }
    };

    rsx! {
        tr {
            style: "border-bottom: 1px solid #eee; &:hover {{ background: #f9f9f9; }}",
            td { style: "padding: 6px 8px; font-family: monospace;", "{timestamp}" }
            td { style: "padding: 6px 8px;", "{type_name}" }
            td { style: "padding: 6px 8px; font-family: monospace; color: #0078d4;", "{id}" }
            td { style: "padding: 6px 8px;", "{dlc}" }
            td { style: "padding: 6px 8px; font-family: monospace;", "{data_hex}" }
        }
    }
}
