#![doc = include_str!("../README.md")]

use tree_sitter_language::LanguageFn;

unsafe extern "C" {
    fn tree_sitter_json() -> *const ();
}

/// Returns the JSON tree-sitter language for this standalone example crate.
pub const fn language() -> LanguageFn {
    unsafe { LanguageFn::from_raw(tree_sitter_json) }
}

/// The highlights query for JSON.
pub const HIGHLIGHTS_QUERY: &str = include_str!("../queries/highlights.scm");

/// JSON has no injection query in this example crate.
pub const INJECTIONS_QUERY: &str = "";

/// JSON has no locals query in this example crate.
pub const LOCALS_QUERY: &str = "";
