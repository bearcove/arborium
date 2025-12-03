//! ADA grammar for tree-sitter
//!
//! This crate provides the ada language grammar for use with tree-sitter.

use tree_sitter_patched_arborium::Language;

unsafe extern "C" {
    fn tree_sitter_ada() -> Language;
}

/// Returns the ada tree-sitter language.
pub fn language() -> Language {
    unsafe { tree_sitter_ada() }
}

/// The highlights query for ada.
pub const HIGHLIGHTS_QUERY: &str = include_str!("../queries/highlights.scm");

/// The injections query for ada (empty - no injections available).
pub const INJECTIONS_QUERY: &str = "";

/// The locals query for ada.
pub const LOCALS_QUERY: &str = include_str!("../queries/locals.scm");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grammar() {
        arborium_test_harness::test_grammar(
            language(),
            "ada",
            HIGHLIGHTS_QUERY,
            INJECTIONS_QUERY,
            LOCALS_QUERY,
            env!("CARGO_MANIFEST_DIR"),
        );
    }
}
