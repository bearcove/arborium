//! TSX grammar for tree-sitter
//!
//! This crate provides the tsx language grammar for use with tree-sitter.

use tree_sitter_patched_arborium::Language;

unsafe extern "C" {
    fn tree_sitter_tsx() -> Language;
}

/// Returns the tsx tree-sitter language.
pub fn language() -> Language {
    unsafe { tree_sitter_tsx() }
}

/// The highlights query for tsx.
pub const HIGHLIGHTS_QUERY: &str = include_str!("../queries/highlights.scm");

/// The injections query for tsx (empty - no injections available).
pub const INJECTIONS_QUERY: &str = "";

/// The locals query for tsx.
pub const LOCALS_QUERY: &str = include_str!("../queries/locals.scm");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grammar() {
        arborium_test_harness::test_grammar(
            language(),
            "tsx",
            HIGHLIGHTS_QUERY,
            INJECTIONS_QUERY,
            LOCALS_QUERY,
            env!("CARGO_MANIFEST_DIR"),
        );
    }
}
