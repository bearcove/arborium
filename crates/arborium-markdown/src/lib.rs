//! Markdown grammar for tree-sitter
//!
//! This crate provides the markdown language grammar for use with tree-sitter.

use tree_sitter_patched_arborium::Language;

unsafe extern "C" {
    fn tree_sitter_markdown() -> Language;
}

/// Returns the markdown tree-sitter language.
pub fn language() -> Language {
    unsafe { tree_sitter_markdown() }
}

/// The highlights query for markdown.
pub const HIGHLIGHTS_QUERY: &str = include_str!("../queries/highlights.scm");

/// The injections query for markdown.
pub const INJECTIONS_QUERY: &str = include_str!("../queries/injections.scm");
