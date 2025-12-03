//! OCAML grammar for tree-sitter
//!
//! This crate provides the ocaml language grammar for use with tree-sitter.

use tree_sitter_patched_arborium::Language;

unsafe extern "C" {
    fn tree_sitter_ocaml() -> Language;
}

/// Returns the ocaml tree-sitter language.
pub fn language() -> Language {
    unsafe { tree_sitter_ocaml() }
}

/// The highlights query for ocaml.
pub const HIGHLIGHTS_QUERY: &str = include_str!("../queries/highlights.scm");

/// The injections query for ocaml (empty - no injections available).
pub const INJECTIONS_QUERY: &str = "";

/// The locals query for ocaml.
pub const LOCALS_QUERY: &str = include_str!("../queries/locals.scm");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grammar() {
        arborium_test_harness::test_grammar(
            language(),
            "ocaml",
            HIGHLIGHTS_QUERY,
            INJECTIONS_QUERY,
            LOCALS_QUERY,
            env!("CARGO_MANIFEST_DIR"),
        );
    }
}
