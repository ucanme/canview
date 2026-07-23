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
    /// Wraps another error with a context string describing which
    /// structure/field was being read when the error occurred. Display
    /// prints the full chain: "FileStatistics.signature: ...inner...".
    Context {
        inner: Box<BlfParseError>,
        ctx: String,
    },
}

impl BlfParseError {
    /// Wrap this error with a context describing where it occurred.
    /// `err.context("FileStatistics.signature")` returns
    /// `BlfParseError::Context { inner: err, ctx: "FileStatistics.signature" }`.
    pub fn context(self, ctx: impl Into<String>) -> Self {
        Self::Context {
            inner: Box::new(self),
            ctx: ctx.into(),
        }
    }
}

/// Extension trait so `?` chains on `BlfParseResult` can add context inline:
/// `cursor.read_u32::<LittleEndian>().map_err(BlfParseError::IoError).context("X")?`.
pub trait BlfResultContext<T> {
    fn context(self, ctx: impl Into<String>) -> BlfParseResult<T>;
}

impl<T> BlfResultContext<T> for BlfParseResult<T> {
    fn context(self, ctx: impl Into<String>) -> BlfParseResult<T> {
        self.map_err(|e| e.context(ctx))
    }
}

impl fmt::Display for BlfParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BlfParseError::Context { inner, ctx } => write!(f, "{}: {}", ctx, inner),
            BlfParseError::IoError(e) => write!(f, "I/O error: {}", e),
            BlfParseError::InvalidFileMagic => write!(f, "Invalid BLF file magic string"),
            BlfParseError::InvalidContainerMagic => write!(f, "Invalid LOBJ container magic string"),
            BlfParseError::UnexpectedEof => write!(f, "Unexpected end of file"),
            BlfParseError::UnsupportedCompression(c) => {
                write!(f, "Unsupported compression method: {}", c)
            }
            BlfParseError::UnknownHeaderVersion(v) => {
                write!(f, "Unknown object header version: {}", v)
            }
        }
    }
}

impl Error for BlfParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            BlfParseError::Context { inner, .. } => Some(inner.as_ref()),
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

/// A specialized `Result` type for BLF parsing.
pub type BlfParseResult<T> = Result<T, BlfParseError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_wraps_and_displays() {
        let err = BlfParseError::InvalidFileMagic.context("FileStatistics.signature");
        assert_eq!(
            format!("{}", err),
            "FileStatistics.signature: Invalid BLF file magic string"
        );
    }

    #[test]
    fn test_context_chain_is_recursive() {
        let inner = BlfParseError::UnsupportedCompression(3);
        let mid = inner.context("LogContainer.compression_method");
        let outer = mid.context("BlfParser.parse");
        assert_eq!(
            format!("{}", outer),
            "BlfParser.parse: LogContainer.compression_method: Unsupported compression method: 3"
        );
    }

    #[test]
    fn test_context_source_returns_inner() {
        let err = BlfParseError::InvalidFileMagic.context("FileStatistics.signature");
        let source = std::error::Error::source(&err);
        assert!(source.is_some());
        assert_eq!(
            format!("{}", source.unwrap()),
            "Invalid BLF file magic string"
        );
    }

    #[test]
    fn test_blf_result_context_trait() {
        let r: BlfParseResult<u32> = Err(BlfParseError::InvalidFileMagic);
        let wrapped = r.context("FileStatistics.signature");
        assert!(wrapped.is_err());
        let err = wrapped.unwrap_err();
        assert_eq!(
            format!("{}", err),
            "FileStatistics.signature: Invalid BLF file magic string"
        );
    }
}
