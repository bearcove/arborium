//! HLSL grammar for tree-sitter
//!
//! This crate provides the hlsl language grammar for use with tree-sitter.

use tree_sitter_patched_arborium::Language;

unsafe extern "C" {
    fn tree_sitter_hlsl() -> Language;
}

/// Returns the hlsl tree-sitter language.
pub fn language() -> Language {
    unsafe { tree_sitter_hlsl() }
}

/// The highlights query for hlsl.
pub const HIGHLIGHTS_QUERY: &str = include_str!("../queries/highlights.scm");

/// The injections query for hlsl (empty - no injections available).
pub const INJECTIONS_QUERY: &str = "";

/// The locals query for hlsl (empty - no locals available).
pub const LOCALS_QUERY: &str = "";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grammar() {
        arborium_test_harness::test_grammar(
            language(),
            "hlsl",
            HIGHLIGHTS_QUERY,
            INJECTIONS_QUERY,
            LOCALS_QUERY,
            env!("CARGO_MANIFEST_DIR"),
        );
    }
}
