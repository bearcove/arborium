//! Core types for highlighting.

use std::fmt;

/// A span of highlighted text.
///
/// Spans come from grammar parsers and contain the raw capture name
/// (e.g., "keyword.function", "include", "string.special.symbol").
/// The capture name is later mapped to a theme slot for rendering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    /// Byte offset where the span starts (inclusive).
    pub start: u32,

    /// Byte offset where the span ends (exclusive).
    pub end: u32,

    /// The capture name from the grammar's highlight query.
    ///
    /// Examples: "keyword", "function.builtin", "include", "storageclass"
    /// All are mapped to theme slots via `arborium_theme::tag_for_capture()`.
    pub capture: String,

    /// Pattern index from the query (higher = later in highlights.scm = higher priority).
    ///
    /// When two spans have the exact same (start, end) range, the one with
    /// higher pattern_index wins during deduplication. This matches the
    /// tree-sitter convention where later patterns in a query override earlier ones.
    pub pattern_index: u32,
}

/// An injection point for embedded languages.
///
/// Injections are detected by the grammar's injection query. For example,
/// HTML can inject CSS and JavaScript into `<style>` and `<script>` tags.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Injection {
    /// Byte offset where the injection starts (inclusive).
    pub start: u32,

    /// Byte offset where the injection ends (exclusive).
    pub end: u32,

    /// The language to inject (e.g., "javascript", "css").
    pub language: String,

    /// Whether to include the node's children in the injection range.
    pub include_children: bool,
}

/// Result of parsing a document with a grammar.
#[derive(Debug, Clone, Default)]
pub struct ParseResult {
    /// Highlighted spans from this parse.
    pub spans: Vec<Span>,

    /// Injection points for other languages.
    pub injections: Vec<Injection>,
}

/// Errors that can occur during highlighting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HighlightError {
    /// The requested language is not supported.
    UnsupportedLanguage(String),

    /// An error occurred during parsing.
    ParseError(String),
}

impl fmt::Display for HighlightError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HighlightError::UnsupportedLanguage(lang) => {
                write!(f, "unsupported language: {}", lang)
            }
            HighlightError::ParseError(msg) => {
                write!(f, "parse error: {}", msg)
            }
        }
    }
}

impl std::error::Error for HighlightError {}
