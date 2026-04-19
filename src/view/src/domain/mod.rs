//! Domain layer - Pure business logic without UI dependencies
//!
//! This module contains core business logic that is independent of the UI framework (GPUI).
//! All code here should:
//! - Not depend on gpui types
//! - Be testable without UI context
//! - Handle data processing, validation, transformation

// Pure domain models
pub mod models;

// Business services
pub mod services;

// Legacy modules (to be refactored)
pub mod config_manager;
pub mod log_processor;
pub mod signal_decoder;
pub mod time_handler;

// Re-export domain models
pub use self::models::{
    Message, MessageType, MessageData, Direction,
    Channel, ChannelConfig, ChannelType, ChannelError,
    FilterCriteria, FilterType,
    IdFilter, ChannelFilter, MessageTypeFilter, DirectionFilter,
    SignalNameFilter, DataPatternFilter,
    DatabaseType, ChannelDatabase, LibraryVersion, SignalLibrary,
};

// Re-export services
pub use self::services::{
    MessageService, FilterService, StatisticsService,
};

// Re-export legacy types (for backward compatibility)
pub use config_manager::{ChannelConfig as LegacyChannelConfig, ConfigManager};
pub use log_processor::{LogProcessor, LogStatistics, MessageFilter};
pub use signal_decoder::{DecodedSignal, SignalDecoder, SignalValue};
pub use time_handler::{TimeHandler, TimestampFormatter};
