//! PHP grammar for tree-sitter
//!
//! This crate provides the php language grammar for use with tree-sitter.

use tree_sitter_patched_arborium::Language;

unsafe extern "C" {
    fn tree_sitter_php() -> Language;
}

/// Returns the php tree-sitter language.
pub fn language() -> Language {
    unsafe { tree_sitter_php() }
}

/// The highlights query for php.
pub const HIGHLIGHTS_QUERY: &str = include_str!("../queries/highlights.scm");

/// The injections query for php.
pub const INJECTIONS_QUERY: &str = include_str!("../queries/injections.scm");

/// The locals query for php (empty - no locals available).
pub const LOCALS_QUERY: &str = "";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grammar() {
        arborium_test_harness::test_grammar(
            language(),
            "php",
            HIGHLIGHTS_QUERY,
            INJECTIONS_QUERY,
            LOCALS_QUERY,
            env!("CARGO_MANIFEST_DIR"),
        );
    }
}
