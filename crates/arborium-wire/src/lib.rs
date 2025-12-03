//! Wire protocol types for arborium WASM plugins.
//!
//! This crate defines the data structures used for communication between
//! the arborium host and grammar plugins. All types use facet-postcard
//! for serialization (no serde on the WASM path).
//!
//! # Wire Version
//!
//! The `WIRE_VERSION` constant should be checked by both host and plugins
//! to ensure compatibility. If versions don't match, the host should
//! reject the plugin with a clear error message.

#![no_std]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use facet::Facet;

/// Wire protocol version.
///
/// Bump this when making breaking changes to the protocol.
/// Host and plugins must agree on this version.
pub const WIRE_VERSION: u32 = 1;

/// A span of highlighted text with a capture name.
#[derive(Debug, Clone, PartialEq, Eq, Facet)]
pub struct Span {
    /// Byte offset where the span starts.
    pub start: u32,
    /// Byte offset where the span ends (exclusive).
    pub end: u32,
    /// The capture name (e.g., "keyword", "function", "string").
    pub capture: String,
}

/// An injection point where another language should be parsed.
#[derive(Debug, Clone, PartialEq, Eq, Facet)]
pub struct Injection {
    /// Byte offset where the injection starts.
    pub start: u32,
    /// Byte offset where the injection ends (exclusive).
    pub end: u32,
    /// The language ID to inject (e.g., "javascript", "css").
    pub language: String,
    /// Whether to include the node children in the injection.
    pub include_children: bool,
}

/// Result of parsing text.
#[derive(Debug, Clone, PartialEq, Eq, Facet)]
pub struct ParseResult {
    /// Highlighted spans from this parse.
    pub spans: Vec<Span>,
    /// Injection points for other languages.
    pub injections: Vec<Injection>,
}

impl ParseResult {
    /// Create an empty parse result.
    pub fn empty() -> Self {
        Self {
            spans: Vec::new(),
            injections: Vec::new(),
        }
    }
}

/// An edit to apply to the text (for incremental parsing).
#[derive(Debug, Clone, PartialEq, Eq, Facet)]
pub struct Edit {
    /// Byte offset where the edit starts.
    pub start_byte: u32,
    /// Byte offset of the old end (before edit).
    pub old_end_byte: u32,
    /// Byte offset of the new end (after edit).
    pub new_end_byte: u32,
    /// Row where the edit starts.
    pub start_row: u32,
    /// Column where the edit starts.
    pub start_col: u32,
    /// Old end row (before edit).
    pub old_end_row: u32,
    /// Old end column (before edit).
    pub old_end_col: u32,
    /// New end row (after edit).
    pub new_end_row: u32,
    /// New end column (after edit).
    pub new_end_col: u32,
}

/// Error that can occur during parsing.
#[derive(Debug, Clone, PartialEq, Eq, Facet)]
pub struct ParseError {
    /// Error message.
    pub message: String,
}

impl ParseError {
    /// Create a new parse error.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// Check if a wire version is compatible with the current version.
///
/// Currently requires exact match. In the future, we might allow
/// backwards-compatible versions.
pub fn is_version_compatible(version: u32) -> bool {
    version == WIRE_VERSION
}

/// Re-export facet-postcard for consumers.
pub use facet_postcard;

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;
    use alloc::vec;

    #[test]
    fn test_parse_result_roundtrip() {
        let result = ParseResult {
            spans: vec![
                Span {
                    start: 0,
                    end: 5,
                    capture: "keyword".to_string(),
                },
                Span {
                    start: 6,
                    end: 10,
                    capture: "function".to_string(),
                },
            ],
            injections: vec![Injection {
                start: 20,
                end: 50,
                language: "javascript".to_string(),
                include_children: false,
            }],
        };

        // Round-trip through facet-postcard
        let bytes = facet_postcard::to_vec(&result).expect("serialize");
        let decoded: ParseResult = facet_postcard::from_bytes(&bytes).expect("deserialize");

        assert_eq!(result, decoded);
    }

    #[test]
    fn test_version_compatibility() {
        assert!(is_version_compatible(WIRE_VERSION));
        assert!(!is_version_compatible(WIRE_VERSION + 1));
        assert!(!is_version_compatible(0));
    }
}
