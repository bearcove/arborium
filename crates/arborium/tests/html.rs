//! HTML injection tests.
//!
//! Tests that verify CSS and JavaScript injections work correctly in HTML.

#![cfg(all(
    feature = "lang-html",
    feature = "lang-css",
    feature = "lang-javascript"
))]

use arborium::Highlighter;
use arborium_highlight::Span;
use indoc::indoc;
use std::collections::HashSet;

/// Get all unique capture names from spans
fn get_captures(spans: &[Span]) -> HashSet<&str> {
    spans.iter().map(|s| s.capture.as_str()).collect()
}

/// Check that a specific text range has a specific capture
fn has_capture_at(spans: &[Span], source: &str, text: &str, capture: &str) -> bool {
    // Find position of text in source
    let Some(pos) = source.find(text) else {
        return false;
    };
    let start = pos as u32;
    let end = (pos + text.len()) as u32;

    // Check if any span covers this range with the expected capture
    spans
        .iter()
        .any(|s| s.start <= start && s.end >= end && s.capture == capture)
}

#[test]
fn test_isolated_style() {
    let mut highlighter = Highlighter::new();
    let source = indoc! {r#"
        <style>
            h1 { color: red; }
        </style>
    "#};
    let spans = highlighter.highlight_spans("html", source).unwrap();
    let captures = get_captures(&spans);

    assert!(
        captures.contains("property"),
        "HTML style injection should have property highlight. Found: {:?}",
        captures
    );
}

#[test]
fn test_isolated_script() {
    let mut highlighter = Highlighter::new();
    let source = indoc! {r#"
        <script>
            let x = 1;
            const y = "hello";
        </script>
    "#};
    let spans = highlighter.highlight_spans("html", source).unwrap();
    let captures = get_captures(&spans);

    assert!(
        captures.contains("keyword"),
        "HTML script injection should have keyword highlight. Found: {:?}",
        captures
    );
    assert!(
        has_capture_at(&spans, source, "let", "keyword"),
        "let should be highlighted as keyword"
    );
}

#[test]
fn test_mixed_content() {
    let mut highlighter = Highlighter::new();
    let source = indoc! {r#"
        <!DOCTYPE html>
        <html>
        <head>
            <style>
                body { margin: 0; }
            </style>
        </head>
        <body>
            <h1>Hello</h1>
            <script>
                console.log("world");
            </script>
        </body>
        </html>
    "#};
    let spans = highlighter.highlight_spans("html", source).unwrap();
    let captures = get_captures(&spans);

    assert!(captures.contains("tag"), "Should have tag highlights");
    assert!(
        captures.contains("property"),
        "Should have CSS property highlights"
    );
    assert!(captures.contains("string"), "Should have string highlights");
}

#[test]
fn test_empty_style_tag() {
    let mut highlighter = Highlighter::new();
    let source = "<style></style>";
    let spans = highlighter.highlight_spans("html", source).unwrap();
    // Should not panic, may or may not have spans
    let _ = spans;
}

#[test]
fn test_empty_script_tag() {
    let mut highlighter = Highlighter::new();
    let source = "<script></script>";
    let spans = highlighter.highlight_spans("html", source).unwrap();
    // Should not panic, may or may not have spans
    let _ = spans;
}

#[test]
fn test_inline_event_handler() {
    let mut highlighter = Highlighter::new();
    let source = r#"<button onclick="alert('hello')">Click</button>"#;
    let spans = highlighter.highlight_spans("html", source).unwrap();
    // Should not panic
    let _ = spans;
}

#[test]
fn test_highlighter_api() {
    let mut highlighter = Highlighter::new();
    let source = indoc! {r#"
        <script>
            const greeting = "hello";
        </script>
        <style>
            body { margin: 0; }
        </style>
    "#};

    let html = highlighter.highlight("html", source).unwrap();

    assert!(
        html.contains("<a-k>const</a-k>"),
        "JS keyword should be highlighted. Got: {}",
        html
    );
}
