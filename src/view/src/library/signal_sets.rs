//! Signal sets — named collections of (channel, msg_id, signal) tuples per library.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SignalSetEntry {
    pub channel_id: u16,
    pub msg_id: u32,
    pub signal_name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SignalSet {
    pub name: String,
    pub entries: Vec<SignalSetEntry>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SignalSetStore {
    #[serde(default)]
    pub sets_by_library: HashMap<String, Vec<SignalSet>>,
}

/// Parse a `selected_signals`-format string ("BUS:CH:MSG_ID:SIG_NAME")
/// into a `SignalSetEntry`. Returns None on malformed input.
///
/// `bus` is discarded — the parent library's `channel_type` decides bus.
pub fn parse_signal_id(sig_id: &str) -> Option<SignalSetEntry> {
    let parts: Vec<&str> = sig_id.split(':').collect();
    if parts.len() < 4 {
        return None;
    }
    let bus = parts[0];
    if bus != "CAN" && bus != "LIN" {
        return None;
    }
    let channel_id = parts[1].parse::<u16>().ok()?;
    let msg_id_str = parts[2];
    let msg_id = if let Some(hex) = msg_id_str.strip_prefix("0x") {
        u32::from_str_radix(hex, 16).ok()?
    } else {
        msg_id_str.parse::<u32>().ok()?
    };
    let signal_name = parts[3..].join(":");
    if signal_name.is_empty() {
        return None;
    }
    Some(SignalSetEntry {
        channel_id,
        msg_id,
        signal_name,
    })
}

/// Rebuild `selected_signals`-format strings from a set + parent library's channel_type.
pub fn build_selected_signals_from_set(
    set: &SignalSet,
    channel_type: crate::models::ChannelType,
) -> Vec<String> {
    let bus = match channel_type {
        crate::models::ChannelType::CAN => "CAN",
        crate::models::ChannelType::LIN => "LIN",
    };
    set.entries
        .iter()
        .map(|e| format!("{}:{}:{}:{}", bus, e.channel_id, e.msg_id, e.signal_name))
        .collect()
}

/// Resolve the path to `signal_sets.json` next to `multi_channel_config.json`.
pub fn signal_set_store_path(config_file_path: Option<&Path>) -> PathBuf {
    if let Some(dir) = config_file_path.and_then(|p| p.parent()) {
        return dir.join("signal_sets.json");
    }
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            return exe_dir.join("signal_sets.json");
        }
    }
    PathBuf::from("signal_sets.json")
}

/// Load the store from disk. Missing file → empty store.
pub fn load_signal_set_store(config_file_path: Option<&Path>) -> SignalSetStore {
    let path = signal_set_store_path(config_file_path);
    if !path.exists() {
        return SignalSetStore::default();
    }
    match std::fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(e) => {
            eprintln!("⚠️  Failed to read signal_sets.json: {}", e);
            SignalSetStore::default()
        }
    }
}

/// Save the store to disk. Errors return a String.
pub fn save_signal_set_store(
    store: &SignalSetStore,
    config_file_path: Option<&Path>,
) -> Result<(), String> {
    let path = signal_set_store_path(config_file_path);
    let content = serde_json::to_string_pretty(store)
        .map_err(|e| format!("Failed to serialize signal sets: {}", e))?;
    std::fs::write(&path, content).map_err(|e| {
        format!("Failed to write {}: {}", path.display(), e)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ChannelType;

    #[test]
    fn test_parse_signal_id_valid_can() {
        let entry = parse_signal_id("CAN:1:0x100:EngineSpeed").unwrap();
        assert_eq!(entry.channel_id, 1);
        assert_eq!(entry.msg_id, 256);
        assert_eq!(entry.signal_name, "EngineSpeed");
    }

    #[test]
    fn test_parse_signal_id_valid_lin() {
        let entry = parse_signal_id("LIN:2:0x20:Speed").unwrap();
        assert_eq!(entry.channel_id, 2);
        assert_eq!(entry.msg_id, 32);
        assert_eq!(entry.signal_name, "Speed");
    }

    #[test]
    fn test_parse_signal_id_decimal_msg_id() {
        let entry = parse_signal_id("CAN:1:256:Speed").unwrap();
        assert_eq!(entry.msg_id, 256);
    }

    #[test]
    fn test_parse_signal_id_invalid_bus() {
        assert!(parse_signal_id("J1939:1:0x100:Speed").is_none());
    }

    #[test]
    fn test_parse_signal_id_empty_signal_name() {
        assert!(parse_signal_id("CAN:1:0x100:").is_none());
    }

    #[test]
    fn test_parse_signal_id_too_few_parts() {
        assert!(parse_signal_id("CAN:1:0x100").is_none());
        assert!(parse_signal_id("CAN:1").is_none());
        assert!(parse_signal_id("CAN").is_none());
    }

    #[test]
    fn test_parse_signal_id_bad_channel() {
        assert!(parse_signal_id("CAN:abc:0x100:Speed").is_none());
    }

    #[test]
    fn test_parse_signal_id_bad_msg_id() {
        assert!(parse_signal_id("CAN:1:0xGG:Speed").is_none());
    }

    #[test]
    fn test_build_selected_signals_from_set_empty() {
        let set = SignalSet { name: "empty".into(), entries: Vec::new() };
        assert!(build_selected_signals_from_set(&set, ChannelType::CAN).is_empty());
    }

    #[test]
    fn test_build_selected_signals_from_set_can() {
        let set = SignalSet {
            name: "s".into(),
            entries: vec![SignalSetEntry {
                channel_id: 1,
                msg_id: 256,
                signal_name: "EngineSpeed".into(),
            }],
        };
        let out = build_selected_signals_from_set(&set, ChannelType::CAN);
        assert_eq!(out, vec!["CAN:1:256:EngineSpeed".to_string()]);
    }

    #[test]
    fn test_build_selected_signals_from_set_lin() {
        let set = SignalSet {
            name: "s".into(),
            entries: vec![SignalSetEntry {
                channel_id: 2,
                msg_id: 32,
                signal_name: "Speed".into(),
            }],
        };
        let out = build_selected_signals_from_set(&set, ChannelType::LIN);
        assert_eq!(out, vec!["LIN:2:32:Speed".to_string()]);
    }

    #[test]
    fn test_build_selected_signals_preserves_order() {
        let set = SignalSet {
            name: "s".into(),
            entries: vec![
                SignalSetEntry { channel_id: 1, msg_id: 256, signal_name: "A".into() },
                SignalSetEntry { channel_id: 1, msg_id: 512, signal_name: "B".into() },
                SignalSetEntry { channel_id: 2, msg_id: 768, signal_name: "C".into() },
            ],
        };
        let out = build_selected_signals_from_set(&set, ChannelType::CAN);
        assert_eq!(out[0], "CAN:1:256:A");
        assert_eq!(out[1], "CAN:1:512:B");
        assert_eq!(out[2], "CAN:2:768:C");
    }

    #[test]
    fn test_store_roundtrip() {
        let mut store = SignalSetStore::default();
        store.sets_by_library.insert(
            "lib_a".into(),
            vec![
                SignalSet {
                    name: "set1".into(),
                    entries: vec![SignalSetEntry {
                        channel_id: 1,
                        msg_id: 256,
                        signal_name: "EngineSpeed".into(),
                    }],
                },
                SignalSet {
                    name: "set2".into(),
                    entries: vec![
                        SignalSetEntry { channel_id: 1, msg_id: 256, signal_name: "A".into() },
                        SignalSetEntry { channel_id: 2, msg_id: 512, signal_name: "B".into() },
                    ],
                },
            ],
        );
        store.sets_by_library.insert("lib_b".into(), Vec::new());

        let json = serde_json::to_string(&store).unwrap();
        let back: SignalSetStore = serde_json::from_str(&json).unwrap();
        assert_eq!(store, back);
    }
}
