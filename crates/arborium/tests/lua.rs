//! Lua documentation-comment injection tests.

#![cfg(all(feature = "lang-lua", feature = "lang-emmyluadoc"))]

use arborium::Highlighter;
use arborium_highlight::Span;

fn has_exact_capture(spans: &[Span], source: &str, text: &str, capture: &str) -> bool {
    spans.iter().any(|span| {
        span.capture == capture && source.get(span.start as usize..span.end as usize) == Some(text)
    })
}

#[test]
fn highlights_emmylua_annotations_inside_lua_comments() {
    let source = "---@class Person\n---@field name string\nlocal person = {}\n";
    let spans = Highlighter::new().highlight_spans("lua", source).unwrap();

    assert!(has_exact_capture(&spans, source, "@class", "keyword"));
    assert!(has_exact_capture(
        &spans,
        source,
        "Person",
        "type.definition"
    ));
    assert!(has_exact_capture(&spans, source, "name", "variable.member"));
    assert!(has_exact_capture(&spans, source, "string", "type.builtin"));
}

#[test]
fn leaves_ordinary_lua_comments_as_comments() {
    let source = "-- @class NotDocumentation\nlocal value = 1\n";
    let spans = Highlighter::new().highlight_spans("lua", source).unwrap();

    assert!(!has_exact_capture(&spans, source, "@class", "keyword"));
}
