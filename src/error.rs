//! Error type shared across the crate.

use std::fmt;
use std::path::PathBuf;

/// Errors produced while reading input, computing distances, or building a tree.
#[derive(Debug)]
pub enum Error {
    /// An I/O failure, with the path involved when one is known.
    Io {
        /// File the operation was working on, if any.
        path: Option<PathBuf>,
        /// The underlying error.
        source: std::io::Error,
    },
    /// The input could not be parsed.
    Format(String),
    /// The input was well formed but not usable (for example, too few sequences).
    Invalid(String),
    /// Options given to the pipeline contradict each other.
    Options(String),
}

/// Convenience alias used throughout the crate.
pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub(crate) fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Error::Io { path: Some(path.into()), source }
    }
    pub(crate) fn format(msg: impl Into<String>) -> Self {
        Error::Format(msg.into())
    }
    pub(crate) fn invalid(msg: impl Into<String>) -> Self {
        Error::Invalid(msg.into())
    }
    pub(crate) fn options(msg: impl Into<String>) -> Self {
        Error::Options(msg.into())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io { path: Some(p), source } => write!(f, "{}: {}", p.display(), source),
            Error::Io { path: None, source } => write!(f, "{}", source),
            Error::Format(m) => write!(f, "input format error: {}", m),
            Error::Invalid(m) => write!(f, "invalid input: {}", m),
            Error::Options(m) => write!(f, "invalid options: {}", m),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(source: std::io::Error) -> Self {
        Error::Io { path: None, source }
    }
}
