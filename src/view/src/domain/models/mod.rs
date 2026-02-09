//! Domain models - Pure business logic without UI dependencies
//!
//! This module contains core business models that are independent of the UI framework.
//! All models here:
//! - Do not depend on gpui types
//! - Are testable without UI context
//! - Represent business concepts clearly

pub mod message;
pub mod channel;
pub mod filter;

// Re-export commonly used types from domain layer
pub use self::message::{
    Message, MessageType, MessageData, Direction
};
pub use self::channel::{
    Channel, ChannelConfig, ChannelType, ChannelError
};
pub use self::filter::{
    FilterCriteria, FilterType,
    IdFilter, ChannelFilter, MessageTypeFilter, DirectionFilter,
    SignalNameFilter, DataPatternFilter
};

// Re-export library types from the existing models layer
// These are already pure domain models with proper separation
pub use crate::models::library::{
    DatabaseType, ChannelDatabase, LibraryVersion, SignalLibrary
};
