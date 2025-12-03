//! PERL grammar for tree-sitter
//!
//! This crate provides the perl language grammar for use with tree-sitter.

use tree_sitter_patched_arborium::Language;

unsafe extern "C" {
    fn tree_sitter_perl() -> Language;
}

/// Returns the perl tree-sitter language.
pub fn language() -> Language {
    unsafe { tree_sitter_perl() }
}

/// The highlights query for perl.
pub const HIGHLIGHTS_QUERY: &str = include_str!("../queries/highlights.scm");

/// The injections query for perl.
pub const INJECTIONS_QUERY: &str = include_str!("../queries/injections.scm");

/// The locals query for perl (empty - no locals available).
pub const LOCALS_QUERY: &str = "";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grammar() {
        arborium_test_harness::test_grammar(
            language(),
            "perl",
            HIGHLIGHTS_QUERY,
            INJECTIONS_QUERY,
            LOCALS_QUERY,
            env!("CARGO_MANIFEST_DIR"),
        );
    }
}
