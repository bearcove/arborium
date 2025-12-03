//! VIMSCRIPT grammar for tree-sitter
//!
//! This crate provides the vim language grammar for use with tree-sitter.

use tree_sitter_patched_arborium::Language;

unsafe extern "C" {
    fn tree_sitter_vim() -> Language;
}

/// Returns the vim tree-sitter language.
pub fn language() -> Language {
    unsafe { tree_sitter_vim() }
}

/// The highlights query for vim.
pub const HIGHLIGHTS_QUERY: &str = include_str!("../queries/highlights.scm");

/// The injections query for vim.
pub const INJECTIONS_QUERY: &str = include_str!("../queries/injections.scm");

/// The locals query for vim (empty - no locals available).
pub const LOCALS_QUERY: &str = "";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grammar() {
        arborium_test_harness::test_grammar(
            language(),
            "vim",
            HIGHLIGHTS_QUERY,
            INJECTIONS_QUERY,
            LOCALS_QUERY,
            env!("CARGO_MANIFEST_DIR"),
        );
    }
}
