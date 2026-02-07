//! Domain layer - Pure business logic without UI dependencies
//!
//! This module contains core business logic that is independent of the UI framework (GPUI).
//! All code here should:
//! - Not depend on gpui types
//! - Be testable without UI context
//! - Handle data processing, validation, transformation

pub mod config_manager;
pub mod log_processor;
pub mod signal_decoder;
pub mod time_handler;

// Re-export commonly used types
pub use config_manager::{ChannelConfig, ConfigManager};
pub use log_processor::{LogProcessor, LogStatistics, MessageFilter};
pub use signal_decoder::{DecodedSignal, SignalDecoder, SignalValue};
pub use time_handler::{TimeHandler, TimestampFormatter};
