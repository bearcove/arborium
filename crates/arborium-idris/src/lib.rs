//! IDRIS grammar for tree-sitter
//!
//! This crate provides the idris language grammar for use with tree-sitter.

use tree_sitter_patched_arborium::Language;

unsafe extern "C" {
    fn tree_sitter_idris() -> Language;
}

/// Returns the idris tree-sitter language.
pub fn language() -> Language {
    unsafe { tree_sitter_idris() }
}

/// The highlights query for idris.
pub const HIGHLIGHTS_QUERY: &str = include_str!("../queries/highlights.scm");

/// The injections query for idris (empty - no injections available).
pub const INJECTIONS_QUERY: &str = "";

/// The locals query for idris (empty - no locals available).
pub const LOCALS_QUERY: &str = "";
