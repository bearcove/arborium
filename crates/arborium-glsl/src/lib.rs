//! GLSL grammar for tree-sitter
//!
//! This crate provides the glsl language grammar for use with tree-sitter.

use tree_sitter_patched_arborium::Language;

unsafe extern "C" {
    fn tree_sitter_glsl() -> Language;
}

/// Returns the glsl tree-sitter language.
pub fn language() -> Language {
    unsafe { tree_sitter_glsl() }
}

/// The highlights query for glsl.
pub const HIGHLIGHTS_QUERY: &str = include_str!("../queries/highlights.scm");

/// The injections query for glsl (empty - no injections available).
pub const INJECTIONS_QUERY: &str = "";

/// The locals query for glsl (empty - no locals available).
pub const LOCALS_QUERY: &str = "";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grammar() {
        arborium_test_harness::test_grammar(
            language(),
            "glsl",
            HIGHLIGHTS_QUERY,
            INJECTIONS_QUERY,
            LOCALS_QUERY,
            env!("CARGO_MANIFEST_DIR"),
        );
    }
}
