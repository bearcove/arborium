//! MESON grammar for tree-sitter
//!
//! This crate provides the meson language grammar for use with tree-sitter.

use tree_sitter_patched_arborium::Language;

unsafe extern "C" {
    fn tree_sitter_meson() -> Language;
}

/// Returns the meson tree-sitter language.
pub fn language() -> Language {
    unsafe { tree_sitter_meson() }
}

/// The highlights query for meson.
pub const HIGHLIGHTS_QUERY: &str = include_str!("../queries/highlights.scm");

/// The injections query for meson (empty - no injections available).
pub const INJECTIONS_QUERY: &str = "";

/// The locals query for meson (empty - no locals available).
pub const LOCALS_QUERY: &str = "";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grammar() {
        arborium_test_harness::test_grammar(
            language(),
            "meson",
            HIGHLIGHTS_QUERY,
            INJECTIONS_QUERY,
            LOCALS_QUERY,
            env!("CARGO_MANIFEST_DIR"),
        );
    }
}
