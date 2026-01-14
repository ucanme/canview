//! lib.rs
//
// A production-ready BLF (Binary Logging Format) parser library,
// translated from the C++ implementation.

//#![deny(missing_docs)]

#![allow(dead_code)] // Allow unused methods (e.g., write methods for future functionality)

mod blf_core;
mod file;
mod file_statistics;
mod objects;
mod parser;

#[cfg(test)]
mod test_utils;

pub use blf_core::*;
pub use file::*;
pub use file_statistics::*;
pub use objects::*;
pub use parser::*;
