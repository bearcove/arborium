//! TREE-SITTER QUERY grammar for tree-sitter
//!
//! This crate provides the query language grammar for use with tree-sitter.

use tree_sitter_patched_arborium::Language;

unsafe extern "C" {
    fn tree_sitter_query() -> Language;
}

/// Returns the query tree-sitter language.
pub fn language() -> Language {
    unsafe { tree_sitter_query() }
}

/// The highlights query for query.
pub const HIGHLIGHTS_QUERY: &str = include_str!("../queries/highlights.scm");

/// The injections query for query.
pub const INJECTIONS_QUERY: &str = include_str!("../queries/injections.scm");

/// The locals query for query (empty - no locals available).
pub const LOCALS_QUERY: &str = "";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grammar() {
        arborium_test_harness::test_grammar(
            language(),
            "query",
            HIGHLIGHTS_QUERY,
            INJECTIONS_QUERY,
            LOCALS_QUERY,
            env!("CARGO_MANIFEST_DIR"),
        );
    }
}
