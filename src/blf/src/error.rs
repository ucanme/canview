//! Defines error types for BLF parsing.

use std::fmt;
use std::io;
use std::error::Error;

/// Represents a parsing error that can occur while processing a BLF file.
#[derive(Debug)]
pub enum BlfParseError {
    /// An I/O error occurred while reading the data.
    IoError(io::Error),
    /// The file does not start with the expected "LOGG" magic string.
    InvalidFileMagic,
    /// A log container does not start with the expected "LOBJ" magic string.
    InvalidContainerMagic,
    /// The data ended unexpectedly while parsing an object.
    UnexpectedEof,
    /// An unknown or unsupported compression method was specified in a LogContainer.
    UnsupportedCompression(u16),
    /// An unknown object header version was encountered.
    UnknownHeaderVersion(u16),
}

impl fmt::Display for BlfParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BlfParseError::IoError(e) => write!(f, "I/O error: {}", e),
            BlfParseError::InvalidFileMagic => write!(f, "Invalid BLF file magic string"),
            BlfParseError::InvalidContainerMagic => write!(f, "Invalid LOBJ container magic string"),
            BlfParseError::UnexpectedEof => write!(f, "Unexpected end of file"),
            BlfParseError::UnsupportedCompression(c) => write!(f, "Unsupported compression method: {}", c),
            BlfParseError::UnknownHeaderVersion(v) => write!(f, "Unknown object header version: {}", v),
        }
    }
}

impl Error for BlfParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            BlfParseError::IoError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for BlfParseError {
    fn from(err: io::Error) -> Self {
        if err.kind() == io::ErrorKind::UnexpectedEof {
            BlfParseError::UnexpectedEof
        } else {
            BlfParseError::IoError(err)
        }
    }
}

/// A specialized `Result` type for BLF parsing operations.
pub type BlfParseResult<T> = Result<T, BlfParseError>;
