//! ZIG grammar for tree-sitter
//!
//! This crate provides the zig language grammar for use with tree-sitter.

use tree_sitter_patched_arborium::Language;

unsafe extern "C" {
    fn tree_sitter_zig() -> Language;
}

/// Returns the zig tree-sitter language.
pub fn language() -> Language {
    unsafe { tree_sitter_zig() }
}

/// The highlights query for zig.
pub const HIGHLIGHTS_QUERY: &str = include_str!("../queries/highlights.scm");

/// The injections query for zig.
pub const INJECTIONS_QUERY: &str = include_str!("../queries/injections.scm");

/// The locals query for zig (empty - no locals available).
pub const LOCALS_QUERY: &str = "";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grammar() {
        arborium_test_harness::test_grammar(
            language(),
            "zig",
            HIGHLIGHTS_QUERY,
            INJECTIONS_QUERY,
            LOCALS_QUERY,
            env!("CARGO_MANIFEST_DIR"),
        );
    }
}
