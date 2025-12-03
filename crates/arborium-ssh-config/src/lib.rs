//! SSH CONFIG grammar for tree-sitter
//!
//! This crate provides the ssh-config language grammar for use with tree-sitter.

use tree_sitter_patched_arborium::Language;

unsafe extern "C" {
    fn tree_sitter_ssh_config() -> Language;
}

/// Returns the ssh-config tree-sitter language.
pub fn language() -> Language {
    unsafe { tree_sitter_ssh_config() }
}

/// The highlights query for ssh-config.
pub const HIGHLIGHTS_QUERY: &str = include_str!("../queries/highlights.scm");

/// The injections query for ssh-config.
pub const INJECTIONS_QUERY: &str = include_str!("../queries/injections.scm");

/// The locals query for ssh-config (empty - no locals available).
pub const LOCALS_QUERY: &str = "";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grammar() {
        arborium_test_harness::test_grammar(
            language(),
            "ssh-config",
            HIGHLIGHTS_QUERY,
            INJECTIONS_QUERY,
            LOCALS_QUERY,
            env!("CARGO_MANIFEST_DIR"),
        );
    }
}
