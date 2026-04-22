//! Signal decoding logic for CAN and LIN messages
//!
//! This module provides pure signal decoding functionality without UI dependencies.
//! It handles conversion from raw bytes to physical values using DBC/LDF definitions.

use parser::dbc::{DbcDatabase, Signal as DbcSignal};
use parser::ldf::{LdfDatabase, LdfFrame};
use std::collections::HashMap;

/// Decoded signal value with metadata
#[derive(Clone, Debug, PartialEq)]
pub struct DecodedSignal {
    /// Signal name
    pub name: String,
    /// Raw value from CAN/LIN data
    pub raw_value: f64,
    /// Physical (scaled) value
    pub physical_value: f64,
    /// Unit (if defined)
    pub unit: Option<String>,
    /// Signal start bit
    pub start_bit: u32,
    /// Signal length in bits
    pub length: u32,
    /// Byte order (0=Motorola, 1=Intel)
    pub byte_order: u8,
}

/// Signal value with timestamp
#[derive(Clone, Debug)]
pub struct SignalValue {
    /// Timestamp in seconds
    pub timestamp: f64,
    /// Decoded signal
    pub signal: DecodedSignal,
}

impl SignalValue {
    /// Create a new signal value
    pub fn new(timestamp: f64, signal: DecodedSignal) -> Self {
        Self { timestamp, signal }
    }
}

/// Result type for signal decoding
pub type DecodeResult<T> = Result<T, DecodeError>;

/// Errors that can occur during signal decoding
#[derive(Clone, Debug, PartialEq)]
pub enum DecodeError {
    /// Signal not found in database
    SignalNotFound(String),
    /// Invalid signal data
    InvalidData(String),
    /// Byte order not supported
    UnsupportedByteOrder(u8),
    /// Value out of range
    ValueOutOfRange { value: f64, min: f64, max: f64 },
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecodeError::SignalNotFound(name) => write!(f, "Signal '{}' not found", name),
            DecodeError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            DecodeError::UnsupportedByteOrder(order) => {
                write!(f, "Unsupported byte order: {}", order)
            }
            DecodeError::ValueOutOfRange { value, min, max } => {
                write!(f, "Value {} out of range [{}, {}]", value, min, max)
            }
        }
    }
}

impl std::error::Error for DecodeError {}

/// Signal decoder for CAN messages
pub struct SignalDecoder {
    /// DBC databases keyed by channel number
    dbc_channels: HashMap<u16, DbcDatabase>,
    /// LDF databases keyed by channel number
    ldf_channels: HashMap<u16, LdfDatabase>,
}

impl SignalDecoder {
    /// Create a new signal decoder
    pub fn new() -> Self {
        Self {
            dbc_channels: HashMap::new(),
            ldf_channels: HashMap::new(),
        }
    }

    /// Add a DBC database for a channel
    pub fn add_dbc_channel(&mut self, channel: u16, db: DbcDatabase) {
        self.dbc_channels.insert(channel, db);
    }

    /// Add an LDF database for a channel
    pub fn add_ldf_channel(&mut self, channel: u16, db: LdfDatabase) {
        self.ldf_channels.insert(channel, db);
    }

    /// Remove database for a channel
    pub fn remove_channel(&mut self, channel: u16) {
        self.dbc_channels.remove(&channel);
        self.ldf_channels.remove(&channel);
    }

    /// Clear all databases
    pub fn clear(&mut self) {
        self.dbc_channels.clear();
        self.ldf_channels.clear();
    }

    /// Get DBC database for a channel
    pub fn get_dbc_channel(&self, channel: u16) -> Option<&DbcDatabase> {
        self.dbc_channels.get(&channel)
    }

    /// Get LDF database for a channel
    pub fn get_ldf_channel(&self, channel: u16) -> Option<&LdfDatabase> {
        self.ldf_channels.get(&channel)
    }

    /// Decode all signals from a CAN message
    pub fn decode_can_message(
        &self,
        channel: u16,
        can_id: u32,
        data: &[u8],
        timestamp: f64,
    ) -> Vec<SignalValue> {
        let db = match self.dbc_channels.get(&channel) {
            Some(db) => db,
            None => return Vec::new(),
        };

        let message_def = match db.messages.get(&can_id) {
            Some(msg) => msg,
            None => return Vec::new(),
        };

        let mut results = Vec::new();

        for signal in message_def.signals.values() {
            match self.decode_signal(signal, data) {
                Ok(decoded) => {
                    results.push(SignalValue::new(timestamp, decoded));
                }
                Err(_) => {
                    // Skip signals that fail to decode
                    continue;
                }
            }
        }

        results
    }

    /// Decode all signals from a LIN message
    pub fn decode_lin_message(
        &self,
        channel: u16,
        lin_id: u8,
        data: &[u8],
        timestamp: f64,
    ) -> Vec<SignalValue> {
        let db = match self.ldf_channels.get(&channel) {
            Some(db) => db,
            None => return Vec::new(),
        };

        // Find the frame by ID
        let frame_def = match db.frames.values().find(|f| f.id as u8 == lin_id) {
            Some(frame) => frame,
            None => return Vec::new(),
        };

        let mut results = Vec::new();

        for signal_mapping in &frame_def.signals {
            // Get the actual signal from the database
            let signal = match db.signals.get(&signal_mapping.signal_name) {
                Some(sig) => sig,
                None => continue,
            };

            match self.decode_ldf_signal(signal, data, signal_mapping.offset) {
                Ok(decoded) => {
                    results.push(SignalValue::new(timestamp, decoded));
                }
                Err(_) => {
                    continue;
                }
            }
        }

        results
    }

    /// Decode a single DBC signal
    pub fn decode_signal(&self, signal: &DbcSignal, data: &[u8]) -> DecodeResult<DecodedSignal> {
        // Use the built-in decode method from DbcSignal
        let raw_value = signal.decode(data);

        // Calculate physical value using factor and offset
        let physical_value = (raw_value * signal.factor) + signal.offset;

        // Get unit
        let unit = if signal.unit.is_empty() {
            None
        } else {
            Some(signal.unit.clone())
        };

        Ok(DecodedSignal {
            name: signal.name.clone(),
            raw_value,
            physical_value,
            unit,
            start_bit: signal.start_bit,
            length: signal.signal_size,
            byte_order: signal.byte_order,
        })
    }

    /// Decode a single LDF signal
    pub fn decode_ldf_signal(
        &self,
        signal: &parser::ldf::LdfSignal,
        data: &[u8],
        offset: u32,
    ) -> DecodeResult<DecodedSignal> {
        // Use the built-in decode method from LdfSignal
        let raw_value = signal.decode(data, offset) as f64;

        // LDF signals don't have factor/offset, so physical = raw
        let physical_value = raw_value;

        Ok(DecodedSignal {
            name: signal.name.clone(),
            raw_value,
            physical_value,
            unit: None, // LDF signals don't have units in the parser
            start_bit: offset,
            length: signal.size,
            byte_order: 0, // LDF typically uses Motorola (big-endian)
        })
    }

    /// Get all signal names for a CAN message
    pub fn get_can_signal_names(&self, channel: u16, can_id: u32) -> Vec<String> {
        let db = match self.dbc_channels.get(&channel) {
            Some(db) => db,
            None => return Vec::new(),
        };

        match db.messages.get(&can_id) {
            Some(msg) => msg.signals.values().map(|s| s.name.clone()).collect(),
            None => Vec::new(),
        }
    }

    /// Get all signal names for a LIN frame
    pub fn get_lin_signal_names(&self, channel: u16, lin_id: u8) -> Vec<String> {
        let db = match self.ldf_channels.get(&channel) {
            Some(db) => db,
            None => return Vec::new(),
        };

        match db.frames.values().find(|f| f.id as u8 == lin_id) {
            Some(frame) => frame
                .signals
                .iter()
                .map(|s| s.signal_name.clone())
                .collect(),
            None => Vec::new(),
        }
    }
}

impl Default for SignalDecoder {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to format signal value for display
pub fn format_signal_value(signal: &DecodedSignal) -> String {
    let value_str = format!("{}", signal.physical_value);

    if let Some(unit) = &signal.unit {
        format!("{} {}", value_str, unit)
    } else {
        value_str
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use parser::dbc::{DbcDatabase, Message};
    use std::collections::HashMap;

    #[test]
    fn test_decode_signal() {
        let mut signals = HashMap::new();
        signals.insert(
            "TestSignal".to_string(),
            DbcSignal {
                name: "TestSignal".to_string(),
                start_bit: 0,
                signal_size: 8,
                byte_order: 1, // Intel
                value_type: '+',
                factor: 1.0,
                offset: 0.0,
                min: 0.0,
                max: 255.0,
                unit: "V".to_string(),
            },
        );

        let mut messages = HashMap::new();
        messages.insert(
            0x123,
            Message {
                id: 0x123,
                name: "TestMessage".to_string(),
                dlc: 8,
                transmitter: "Node1".to_string(),
                signals,
                comment: None,
            },
        );

        let db = DbcDatabase {
            messages,
            version: "1.0".to_string(),
            description: None,
        };

        let mut decoder = SignalDecoder::new();
        decoder.add_dbc_channel(1, db);

        let data = [0x64]; // 100 decimal
        let results = decoder.decode_can_message(1, 0x123, &data, 0.0);

        assert_eq!(results.len(), 1);
        let decoded = &results[0].signal;
        assert_eq!(decoded.name, "TestSignal");
        assert_eq!(decoded.raw_value, 100.0);
        assert_eq!(decoded.physical_value, 100.0);
        assert_eq!(decoded.unit, Some("V".to_string()));
    }

    #[test]
    fn test_format_signal_value() {
        let signal = DecodedSignal {
            name: "Speed".to_string(),
            raw_value: 100.0,
            physical_value: 100.0,
            unit: Some("km/h".to_string()),
            start_bit: 0,
            length: 8,
            byte_order: 1,
        };

        let formatted = format_signal_value(&signal);
        assert_eq!(formatted, "100 km/h");
    }

    #[test]
    fn test_decoder_empty() {
        let decoder = SignalDecoder::new();
        let data = [0x00];
        let results = decoder.decode_can_message(1, 0x123, &data, 0.0);
        assert_eq!(results.len(), 0);
    }
}
