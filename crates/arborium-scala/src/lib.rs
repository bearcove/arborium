//! SCALA grammar for tree-sitter
//!
//! This crate provides the scala language grammar for use with tree-sitter.

use tree_sitter_patched_arborium::Language;

unsafe extern "C" {
    fn tree_sitter_scala() -> Language;
}

/// Returns the scala tree-sitter language.
pub fn language() -> Language {
    unsafe { tree_sitter_scala() }
}

/// The highlights query for scala.
pub const HIGHLIGHTS_QUERY: &str = include_str!("../queries/highlights.scm");

/// The injections query for scala (empty - no injections available).
pub const INJECTIONS_QUERY: &str = "";

/// The locals query for scala.
pub const LOCALS_QUERY: &str = include_str!("../queries/locals.scm");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grammar() {
        arborium_test_harness::test_grammar(
            language(),
            "scala",
            HIGHLIGHTS_QUERY,
            INJECTIONS_QUERY,
            LOCALS_QUERY,
            env!("CARGO_MANIFEST_DIR"),
        );
    }
}
