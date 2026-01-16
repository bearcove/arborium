//! Wire protocol types for arborium WASM plugins.
//!
//! This crate defines the data structures used for communication between
//! the arborium host and grammar plugins. All types use serde for
//! serialization with wasm-bindgen.
//!
//! # Offset Encoding
//!
//! Tree-sitter natively produces UTF-8 byte offsets. However, JavaScript
//! strings use UTF-16 encoding, so offsets need conversion for JS interop.
//!
//! This crate provides two sets of types:
//! - `Utf8*` types use UTF-8 byte offsets (for Rust code, string slicing)
//! - `Utf16*` types use UTF-16 code unit indices (for JavaScript `slice()`, editors)
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
use serde::{Deserialize, Serialize};

/// Wire protocol version.
///
/// Bump this when making breaking changes to the protocol.
/// Host and plugins must agree on this version.
pub const WIRE_VERSION: u32 = 2;

// ============================================================================
// UTF-8 types (native tree-sitter offsets, for Rust string slicing)
// ============================================================================

/// A span of highlighted text with UTF-8 byte offsets.
///
/// Use this when working with Rust strings, as `&source[start..end]` requires
/// UTF-8 byte boundaries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Utf8Span {
    /// UTF-8 byte offset where the span starts.
    pub start: u32,
    /// UTF-8 byte offset where the span ends (exclusive).
    pub end: u32,
    /// The capture name (e.g., "keyword", "function", "string").
    pub capture: String,
    /// Pattern index from the query (higher = later in highlights.scm = higher priority).
    #[serde(default)]
    pub pattern_index: u32,
}

/// An injection point with UTF-8 byte offsets.
///
/// Use this when working with Rust strings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Utf8Injection {
    /// UTF-8 byte offset where the injection starts.
    pub start: u32,
    /// UTF-8 byte offset where the injection ends (exclusive).
    pub end: u32,
    /// The language ID to inject (e.g., "javascript", "css").
    pub language: String,
    /// Whether to include the node children in the injection.
    pub include_children: bool,
}

/// Result of parsing text, with UTF-8 byte offsets.
///
/// This is the native format from tree-sitter and is suitable for
/// Rust code that needs to slice strings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Utf8ParseResult {
    /// Highlighted spans from this parse.
    pub spans: Vec<Utf8Span>,
    /// Injection points for other languages.
    pub injections: Vec<Utf8Injection>,
}

impl Utf8ParseResult {
    /// Create an empty parse result.
    pub fn empty() -> Self {
        Self {
            spans: Vec::new(),
            injections: Vec::new(),
        }
    }
}

// ============================================================================
// UTF-16 types (for JavaScript interop)
// ============================================================================

/// A span of highlighted text with UTF-16 code unit indices.
///
/// Use this when working with JavaScript, as `String.prototype.slice()`
/// and DOM APIs use UTF-16 code unit indices.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Utf16Span {
    /// UTF-16 code unit index where the span starts.
    pub start: u32,
    /// UTF-16 code unit index where the span ends (exclusive).
    pub end: u32,
    /// The capture name (e.g., "keyword", "function", "string").
    pub capture: String,
    /// Pattern index from the query (higher = later in highlights.scm = higher priority).
    #[serde(default)]
    pub pattern_index: u32,
}

/// An injection point with UTF-16 code unit indices.
///
/// Use this when working with JavaScript.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Utf16Injection {
    /// UTF-16 code unit index where the injection starts.
    pub start: u32,
    /// UTF-16 code unit index where the injection ends (exclusive).
    pub end: u32,
    /// The language ID to inject (e.g., "javascript", "css").
    pub language: String,
    /// Whether to include the node children in the injection.
    pub include_children: bool,
}

/// Result of parsing text, with UTF-16 code unit indices.
///
/// This format is suitable for JavaScript code that needs to use
/// `String.prototype.slice()` or integrate with editors.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Utf16ParseResult {
    /// Highlighted spans from this parse.
    pub spans: Vec<Utf16Span>,
    /// Injection points for other languages.
    pub injections: Vec<Utf16Injection>,
}

impl Utf16ParseResult {
    /// Create an empty parse result.
    pub fn empty() -> Self {
        Self {
            spans: Vec::new(),
            injections: Vec::new(),
        }
    }
}

// ============================================================================
// Legacy type aliases (for backwards compatibility during transition)
// ============================================================================

/// Legacy alias for [`Utf8Span`].
#[deprecated(since = "2.11.0", note = "Use Utf8Span or Utf16Span explicitly")]
pub type Span = Utf8Span;

/// Legacy alias for [`Utf8Injection`].
#[deprecated(
    since = "2.11.0",
    note = "Use Utf8Injection or Utf16Injection explicitly"
)]
pub type Injection = Utf8Injection;

/// Legacy alias for [`Utf8ParseResult`].
#[deprecated(
    since = "2.11.0",
    note = "Use Utf8ParseResult or Utf16ParseResult explicitly"
)]
pub type ParseResult = Utf8ParseResult;

// ============================================================================
// Other types (not offset-dependent)
// ============================================================================

/// An edit to apply to the text (for incremental parsing).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
