//! RESCRIPT grammar for tree-sitter
//!
//! This crate provides the rescript language grammar for use with tree-sitter.

use tree_sitter_patched_arborium::Language;

unsafe extern "C" {
    fn tree_sitter_rescript() -> Language;
}

/// Returns the rescript tree-sitter language.
pub fn language() -> Language {
    unsafe { tree_sitter_rescript() }
}

/// The highlights query for rescript.
pub const HIGHLIGHTS_QUERY: &str = include_str!("../queries/highlights.scm");

/// The injections query for rescript.
pub const INJECTIONS_QUERY: &str = include_str!("../queries/injections.scm");

/// The locals query for rescript.
pub const LOCALS_QUERY: &str = include_str!("../queries/locals.scm");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grammar() {
        arborium_test_harness::test_grammar(
            language(),
            "rescript",
            HIGHLIGHTS_QUERY,
            INJECTIONS_QUERY,
            LOCALS_QUERY,
            env!("CARGO_MANIFEST_DIR"),
        );
    }
}
