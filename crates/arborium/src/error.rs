//! Unified error type for arborium.

use std::fmt;
use std::io;

/// Error type for highlighting operations.
///
/// This is marked `#[non_exhaustive]` to allow adding new variants
/// in future versions without breaking changes.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// The requested language is not supported.
    ///
    /// This occurs when no grammar is available for the given language name.
    /// Language availability depends on which `lang-*` features are enabled.
    UnsupportedLanguage {
        /// The language that was requested.
        language: String,
    },

    /// An error occurred while parsing the source code.
    ///
    /// This typically indicates a problem with the grammar or an internal
    /// tree-sitter error.
    ParseError {
        /// The language being parsed.
        language: String,
        /// A description of what went wrong.
        message: String,
    },

    /// An error occurred while compiling a tree-sitter query.
    ///
    /// This indicates a problem with the grammar's highlight or injection queries.
    QueryError {
        /// The language whose query failed.
        language: String,
        /// A description of the query error.
        message: String,
    },

    /// An I/O error occurred during highlighting.
    ///
    /// This typically happens when writing to a `Write` destination fails.
    Io(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::UnsupportedLanguage { language } => {
                write!(f, "unsupported language: {}", language)
            }
            Error::ParseError { language, message } => {
                write!(f, "parse error for {}: {}", language, message)
            }
            Error::QueryError { language, message } => {
                write!(f, "query error for {}: {}", language, message)
            }
            Error::Io(e) => write!(f, "I/O error: {}", e),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}

/// Convert from the internal arborium-highlight error type.
impl From<arborium_highlight::HighlightError> for Error {
    fn from(e: arborium_highlight::HighlightError) -> Self {
        match e {
            arborium_highlight::HighlightError::UnsupportedLanguage(language) => {
                Error::UnsupportedLanguage { language }
            }
            arborium_highlight::HighlightError::ParseError(message) => Error::ParseError {
                language: String::new(), // We don't have the language here
                message,
            },
        }
    }
}
