//! GLEAM grammar for tree-sitter
//!
//! This crate provides the gleam language grammar for use with tree-sitter.

use tree_sitter_patched_arborium::Language;

unsafe extern "C" {
    fn tree_sitter_gleam() -> Language;
}

/// Returns the gleam tree-sitter language.
pub fn language() -> Language {
    unsafe { tree_sitter_gleam() }
}

/// The highlights query for gleam.
pub const HIGHLIGHTS_QUERY: &str = include_str!("../queries/highlights.scm");

/// The injections query for gleam.
pub const INJECTIONS_QUERY: &str = include_str!("../queries/injections.scm");

/// The locals query for gleam.
pub const LOCALS_QUERY: &str = include_str!("../queries/locals.scm");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grammar() {
        arborium_test_harness::test_grammar(
            language(),
            "gleam",
            HIGHLIGHTS_QUERY,
            INJECTIONS_QUERY,
            LOCALS_QUERY,
            env!("CARGO_MANIFEST_DIR"),
        );
    }
}
