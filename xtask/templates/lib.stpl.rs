//! <%= generated_disclaimer %>

#![doc = include_str!("../README.md")]

use tree_sitter_language::LanguageFn;

unsafe extern "C" {
    fn tree_sitter_<%= c_symbol %>() -> *const ();
}

/// Returns the <%= grammar_id %> tree-sitter [`LanguageFn`].
pub const fn language() -> LanguageFn {
    unsafe { LanguageFn::from_raw(tree_sitter_<%= c_symbol %>) }
}

<% if highlights_exists { %>
<% if !highlights_prepend.is_empty() { %>
/// The highlights query for <%= grammar_id %> (base query only).
/// Use [`HIGHLIGHTS_QUERY`] for the full query including inherited queries.
const HIGHLIGHTS_QUERY_BASE: &str = include_str!("../queries/highlights.scm");

/// The highlights query for <%= grammar_id %>.
/// Includes inherited queries from: <%= highlights_prepend.join(", ") %>.
pub static HIGHLIGHTS_QUERY: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
    let mut query = String::new();
<% for crate_name in &highlights_prepend { %>
    query.push_str(&<%= crate_name %>::HIGHLIGHTS_QUERY);
    query.push('\n');
<% } %>
    query.push_str(HIGHLIGHTS_QUERY_BASE);
    query
});
<% } else { %>
/// The highlights query for <%= grammar_id %>.
pub const HIGHLIGHTS_QUERY: &str = include_str!("../queries/highlights.scm");
<% } %>
<% } else { %>
/// The highlights query for <%= grammar_id %> (empty - no highlights available).
pub const HIGHLIGHTS_QUERY: &str = "";
<% } %>

<% if injections_exists { %>
/// The injections query for <%= grammar_id %>.
pub const INJECTIONS_QUERY: &str = include_str!("../queries/injections.scm");
<% } else { %>
/// The injections query for <%= grammar_id %> (empty - no injections available).
pub const INJECTIONS_QUERY: &str = "";
<% } %>

<% if locals_exists { %>
/// The locals query for <%= grammar_id %>.
pub const LOCALS_QUERY: &str = include_str!("../queries/locals.scm");
<% } else { %>
/// The locals query for <%= grammar_id %> (empty - no locals available).
pub const LOCALS_QUERY: &str = "";
<% } %>
<% if !tests_cursed { %>

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grammar() {
        arborium_test_harness::test_grammar(
            language(),
            "<%= grammar_id %>",
<% if !highlights_prepend.is_empty() { %>
            &HIGHLIGHTS_QUERY,
<% } else { %>
            HIGHLIGHTS_QUERY,
<% } %>
            INJECTIONS_QUERY,
            LOCALS_QUERY,
            env!("CARGO_MANIFEST_DIR"),
        );
    }

    #[test]
    fn test_corpus() {
        arborium_test_harness::test_corpus(language(), "<%= grammar_id %>", env!("CARGO_MANIFEST_DIR"));
    }
}
<% } %>
