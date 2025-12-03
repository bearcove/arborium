//! SVELTE grammar for tree-sitter
//!
//! This crate provides the svelte language grammar for use with tree-sitter.

use tree_sitter_patched_arborium::Language;

unsafe extern "C" {
    fn tree_sitter_svelte() -> Language;
}

/// Returns the svelte tree-sitter language.
pub fn language() -> Language {
    unsafe { tree_sitter_svelte() }
}

/// The highlights query for svelte.
pub const HIGHLIGHTS_QUERY: &str = include_str!("../queries/highlights.scm");

/// The injections query for svelte.
pub const INJECTIONS_QUERY: &str = include_str!("../queries/injections.scm");

/// The locals query for svelte.
pub const LOCALS_QUERY: &str = include_str!("../queries/locals.scm");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grammar() {
        arborium_test_harness::test_grammar(
            language(),
            "svelte",
            HIGHLIGHTS_QUERY,
            INJECTIONS_QUERY,
            LOCALS_QUERY,
            env!("CARGO_MANIFEST_DIR"),
        );
    }
}
