//! Message domain model
//!
//! Pure business model for CAN log messages.
//! No UI framework dependencies.

use chrono::{DateTime, NaiveDateTime};
use std::fmt;

/// CAN log message - domain layer representation
///
/// This is a pure domain model that represents a CAN log message
/// without any dependencies on UI frameworks.
#[derive(Clone, Debug, PartialEq)]
pub struct Message {
    /// Sequential index in the log
    pub index: usize,

    /// Timestamp of the message
    pub timestamp: NaiveDateTime,

    /// Channel number (1-8 for typical CAN buses)
    pub channel: u16,

    /// Message ID (CAN identifier)
    pub id: u32,

    /// Message type (CAN, Error, Status, etc.)
    pub message_type: MessageType,

    /// Direction (TX/RX)
    pub direction: Direction,

    /// Name (if available from database)
    pub name: Option<String>,

    /// Data length code (0-8 for CAN)
    pub dlc: u8,

    /// Data bytes (0-8 bytes)
    pub data: Vec<u8>,

    /// Extended data (for special message types)
    pub extended_data: Option<Vec<u8>>,

    /// Raw timestamp value (for precise calculations)
    pub raw_timestamp: f64,
}

impl Message {
    /// Create a new Message
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        index: usize,
        timestamp: NaiveDateTime,
        channel: u16,
        id: u32,
        message_type: MessageType,
        direction: Direction,
        name: Option<String>,
        dlc: u8,
        data: Vec<u8>,
        raw_timestamp: f64,
    ) -> Self {
        Self {
            index,
            timestamp,
            channel,
            id,
            message_type,
            direction,
            name,
            dlc,
            data,
            extended_data: None,
            raw_timestamp,
        }
    }

    /// Get the data length in bytes
    pub fn data_length(&self) -> usize {
        self.data.len()
    }

    /// Check if this is an extended frame (29-bit ID)
    pub fn is_extended(&self) -> bool {
        self.id > 0x7FF
    }

    /// Get formatted ID string
    pub fn formatted_id(&self, decimal: bool) -> String {
        if decimal {
            format!("{}", self.id)
        } else {
            format!("{:X}", self.id)
        }
    }

    /// Get data as hexadecimal string
    pub fn data_hex(&self) -> String {
        self.data
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// Message type enumeration
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MessageType {
    /// Standard CAN message
    Can,

    /// Error frame
    Error,

    /// Status information
    Status,

    /// Information log
    Info,

    /// Warning message
    Warning,

    /// Unknown/other type
    Other,
}

impl MessageType {
    /// Parse from string representation
    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "CAN" => MessageType::Can,
            "ERROR" => MessageType::Error,
            "STATUS" => MessageType::Status,
            "INFO" => MessageType::Info,
            "WARNING" => MessageType::Warning,
            _ => MessageType::Other,
        }
    }

    /// Get display name
    pub fn as_str(&self) -> &'static str {
        match self {
            MessageType::Can => "CAN",
            MessageType::Error => "ERR",
            MessageType::Status => "STAT",
            MessageType::Info => "INFO",
            MessageType::Warning => "WARN",
            MessageType::Other => "OTHER",
        }
    }
}

impl fmt::Display for MessageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Message direction (transmission or reception)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Direction {
    /// Transmitted message
    Tx,

    /// Received message
    Rx,

    /// Unknown direction
    Unknown,
}

impl Direction {
    /// Parse from string
    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "RX" | "R" => Direction::Rx,
            "TX" | "T" => Direction::Tx,
            _ => Direction::Unknown,
        }
    }

    /// Get display symbol
    pub fn as_symbol(&self) -> &'static str {
        match self {
            Direction::Rx => "Rx",
            Direction::Tx => "Tx",
            Direction::Unknown => "",
        }
    }
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_symbol())
    }
}

/// Message data payload (type-safe wrapper for bytes)
#[derive(Clone, Debug, PartialEq)]
pub struct MessageData {
    bytes: Vec<u8>,
    max_length: usize,
}

impl MessageData {
    /// Create new message data
    pub fn new(bytes: Vec<u8>, max_length: usize) -> Self {
        Self {
            bytes,
            max_length,
        }
    }

    /// Create from raw byte array
    pub fn from_raw(raw: &[u8], dlc: u8) -> Self {
        let length = (dlc as usize).min(8).min(raw.len());
        Self {
            bytes: raw[..length].to_vec(),
            max_length: 8,
        }
    }

    /// Get the byte slice
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Get the length
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        self.bytes
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_type_from_str() {
        assert_eq!(MessageType::from_str("CAN"), MessageType::Can);
        assert_eq!(MessageType::from_str("Error"), MessageType::Error);
        assert_eq!(MessageType::from_str("error"), MessageType::Error);
        assert_eq!(MessageType::from_str("unknown"), MessageType::Other);
    }

    #[test]
    fn test_message_type_display() {
        assert_eq!(MessageType::Can.to_string(), "CAN");
        assert_eq!(MessageType::Error.to_string(), "ERR");
    }

    #[test]
    fn test_direction() {
        assert_eq!(Direction::from_str("Rx"), Direction::Rx);
        assert_eq!(Direction::from_str("TX"), Direction::Tx);
        assert_eq!(Direction::Rx.as_symbol(), "Rx");
        assert_eq!(Direction::Tx.as_symbol(), "Tx");
    }

    #[test]
    fn test_message_data() {
        let data = MessageData::new(vec![0x01, 0x02, 0x03], 8);
        assert_eq!(data.len(), 3);
        assert!(!data.is_empty());
        assert_eq!(data.to_hex(), "01 02 03");
    }

    #[test]
    fn test_message_data_from_raw() {
        let raw = vec![0xAA, 0xBB, 0xCC, 0xDD];
        let data = MessageData::from_raw(&raw, 3);
        assert_eq!(data.len(), 3);
        assert_eq!(data.to_hex(), "AA BB CC");
    }

    #[test]
    fn test_message_formatted_id() {
        let msg = Message::new(
            0,
            NaiveDateTime::default(),
            1,
            0x123,
            MessageType::Can,
            Direction::Rx,
            None,
            8,
            vec![0; 8],
            0.0,
        );

        assert_eq!(msg.formatted_id(true), "291");
        assert_eq!(msg.formatted_id(false), "123");
    }

    #[test]
    fn test_message_is_extended() {
        let msg_std = Message::new(
            0,
            NaiveDateTime::default(),
            1,
            0x123,
            MessageType::Can,
            Direction::Rx,
            None,
            8,
            vec![0; 8],
            0.0,
        );
        assert!(!msg_std.is_extended());

        let msg_ext = Message::new(
            0,
            NaiveDateTime::default(),
            1,
            0x12345678,
            MessageType::Can,
            Direction::Rx,
            None,
            8,
            vec![0; 8],
            0.0,
        );
        assert!(msg_ext.is_extended());
    }
}
