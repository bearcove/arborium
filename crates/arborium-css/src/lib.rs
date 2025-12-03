//! CSS grammar for tree-sitter
//!
//! This crate provides the css language grammar for use with tree-sitter.

use tree_sitter_patched_arborium::Language;

unsafe extern "C" {
    fn tree_sitter_css() -> Language;
}

/// Returns the css tree-sitter language.
pub fn language() -> Language {
    unsafe { tree_sitter_css() }
}

/// The highlights query for css.
pub const HIGHLIGHTS_QUERY: &str = include_str!("../queries/highlights.scm");

/// The injections query for css (empty - no injections available).
pub const INJECTIONS_QUERY: &str = "";

/// The locals query for css (empty - no locals available).
pub const LOCALS_QUERY: &str = "";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grammar() {
        arborium_test_harness::test_grammar(
            language(),
            "css",
            HIGHLIGHTS_QUERY,
            INJECTIONS_QUERY,
            LOCALS_QUERY,
            env!("CARGO_MANIFEST_DIR"),
        );
    }
}
