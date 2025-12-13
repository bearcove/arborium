//! Advanced API for custom highlighting implementations.
//!
//! This module re-exports low-level types from `arborium-highlight` for users who need:
//!
//! - Direct access to [`CompiledGrammar`] and [`ParseContext`] for custom workflows
//! - Raw [`Span`] data for custom rendering
//! - Building browser/WASM highlighters with dynamic grammar loading
//!
//! # Architecture
//!
//! The highlighting system is built around separating shareable and per-thread state:
//!
//! - [`CompiledGrammar`]: Thread-safe compiled queries (share via `Arc`)
//! - [`ParseContext`]: Per-thread parser state (cheap to create)
//!
//! # Example: Direct Grammar Usage
//!
//! ```rust,ignore
//! use std::sync::Arc;
//! use arborium::advanced::{CompiledGrammar, ParseContext, GrammarConfig};
//!
//! // Compile grammar (expensive, do once)
//! let config = GrammarConfig {
//!     language: arborium::lang_rust::language().into(),
//!     highlights_query: &arborium::lang_rust::HIGHLIGHTS_QUERY,
//!     injections_query: arborium::lang_rust::INJECTIONS_QUERY,
//!     locals_query: arborium::lang_rust::LOCALS_QUERY,
//! };
//! let grammar = Arc::new(CompiledGrammar::new(config)?);
//!
//! // Create parse context (cheap, per-thread)
//! let mut ctx = ParseContext::for_grammar(&grammar)?;
//!
//! // Parse
//! let result = grammar.parse(&mut ctx, "fn main() {}");
//! println!("Found {} spans", result.spans.len());
//! ```

// Core tree-sitter types
pub use arborium_highlight::tree_sitter::{
    CompiledGrammar, GrammarConfig, GrammarError, ParseContext,
};

// Data types
pub use arborium_highlight::{Injection, ParseResult, Span};

// Low-level rendering utilities
pub use arborium_highlight::{
    html_escape, spans_to_ansi, spans_to_ansi_with_options, spans_to_html, write_spans_as_html,
};

// ANSI rendering options
pub use arborium_highlight::AnsiOptions;
