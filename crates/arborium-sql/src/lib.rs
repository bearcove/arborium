//! SQL grammar for tree-sitter
//!
//! This crate provides the sql language grammar for use with tree-sitter.

use tree_sitter_patched_arborium::Language;

unsafe extern "C" {
    fn tree_sitter_sql() -> Language;
}

/// Returns the sql tree-sitter language.
pub fn language() -> Language {
    unsafe { tree_sitter_sql() }
}

/// The highlights query for sql.
pub const HIGHLIGHTS_QUERY: &str = include_str!("../queries/highlights.scm");

/// The injections query for sql (empty - no injections available).
pub const INJECTIONS_QUERY: &str = "";

/// The locals query for sql (empty - no locals available).
pub const LOCALS_QUERY: &str = "";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grammar() {
        arborium_test_harness::test_grammar(
            language(),
            "sql",
            HIGHLIGHTS_QUERY,
            INJECTIONS_QUERY,
            LOCALS_QUERY,
            env!("CARGO_MANIFEST_DIR"),
        );
    }
}
