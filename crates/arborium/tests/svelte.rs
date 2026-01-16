//! Svelte injection tests.
//!
//! Tests that verify CSS and JavaScript injections work correctly in Svelte components.

#![cfg(all(
    feature = "lang-svelte",
    feature = "lang-css",
    feature = "lang-javascript",
    feature = "lang-typescript"
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
    let Some(pos) = source.find(text) else {
        return false;
    };
    let start = pos as u32;
    let end = (pos + text.len()) as u32;

    spans
        .iter()
        .any(|s| s.start <= start && s.end >= end && s.capture == capture)
}

// ========================================================================
// Script Injection Tests
// ========================================================================

#[test]
fn test_isolated_script() {
    let mut highlighter = Highlighter::new();
    let source = indoc! {r#"
        <script>
            let name = "world";
            export let count = 0;
        </script>
    "#};
    let html = highlighter.highlight("svelte", source).unwrap();

    assert!(
        html.contains("<a-k>"),
        "Svelte script should have keyword highlighting"
    );
}

#[test]
fn test_script_with_function() {
    let mut highlighter = Highlighter::new();
    let source = indoc! {r#"
        <script>
            function greet(name) {
                return `Hello, ${name}!`;
            }
        </script>
    "#};
    let html = highlighter.highlight("svelte", source).unwrap();

    assert!(
        html.contains("<a-k>"),
        "Svelte function should have keyword highlighting"
    );
}

#[test]
fn test_nested_braces() {
    let mut highlighter = Highlighter::new();
    let source = indoc! {r#"
        <script>
            let obj = { a: { b: { c: 1 } } };
        </script>
    "#};
    let html = highlighter.highlight("svelte", source).unwrap();
    assert!(
        html.contains("<a-k>"),
        "Svelte nested braces should have highlighting"
    );
}

// ========================================================================
// Style Injection Tests
// ========================================================================

#[test]
fn test_isolated_style() {
    let mut highlighter = Highlighter::new();
    let source = indoc! {r#"
        <style>
            h1 {
                color: red;
                font-size: 2em;
            }
        </style>
    "#};
    let spans = highlighter.highlight_spans("svelte", source).unwrap();
    let captures = get_captures(&spans);

    assert!(
        captures.contains("property"),
        "Svelte style injection should have property highlight. Found: {:?}",
        captures
    );
}

#[test]
fn test_style_with_multiple_selectors() {
    let mut highlighter = Highlighter::new();
    let source = indoc! {r#"
        <style>
            h1, h2, h3 {
                color: blue;
            }
            .container {
                margin: 0 auto;
                padding: 1rem;
            }
        </style>
    "#};
    let spans = highlighter.highlight_spans("svelte", source).unwrap();
    let captures = get_captures(&spans);

    assert!(
        captures.contains("property"),
        "Svelte multiple selectors should have property highlight. Found: {:?}",
        captures
    );
}

// ========================================================================
// Template Tests
// ========================================================================

#[test]
fn test_template_expressions() {
    let mut highlighter = Highlighter::new();
    let source = indoc! {r#"
        <h1>Hello {name}!</h1>
        <p>Count: {count + 1}</p>
    "#};
    let spans = highlighter.highlight_spans("svelte", source).unwrap();

    // Template expressions should produce spans
    assert!(!spans.is_empty(), "Svelte template should produce spans");
}

#[test]
fn test_only_template() {
    let mut highlighter = Highlighter::new();
    let source = indoc! {r#"
        <div>
            <h1>Hello World</h1>
            <p>No script or style tags here</p>
        </div>
    "#};
    let spans = highlighter.highlight_spans("svelte", source).unwrap();
    assert!(!spans.is_empty());
}

// ========================================================================
// Full Component Tests
// ========================================================================

#[test]
fn test_full_component() {
    let mut highlighter = Highlighter::new();
    let source = indoc! {r#"
        <script>
            export let name = "world";
            let count = 0;

            function increment() {
                count += 1;
            }
        </script>

        <main>
            <h1>Hello {name}!</h1>
            <button on:click={increment}>
                Clicked {count} times
            </button>
        </main>

        <style>
            main {
                text-align: center;
                padding: 1em;
            }

            h1 {
                color: #ff3e00;
            }

            button {
                background: #ff3e00;
                color: white;
            }
        </style>
    "#};
    let spans = highlighter.highlight_spans("svelte", source).unwrap();
    let captures = get_captures(&spans);

    // JS keywords
    assert!(
        captures.contains("keyword"),
        "Svelte full component should have JS keyword highlights"
    );
    assert!(
        has_capture_at(&spans, source, "export", "keyword"),
        "export should be highlighted as keyword"
    );
    assert!(
        has_capture_at(&spans, source, "function", "keyword"),
        "function should be highlighted as keyword"
    );

    // CSS properties
    assert!(
        captures.contains("property"),
        "Svelte full component should have CSS property highlights"
    );
}

// ========================================================================
// TypeScript Tests
// ========================================================================

#[test]
fn test_typescript_script() {
    let mut highlighter = Highlighter::new();
    let source = indoc! {r#"
        <script lang="ts">
            interface User {
                name: string;
                age: number;
            }

            let user: User = { name: "Alice", age: 30 };
        </script>
    "#};
    let spans = highlighter.highlight_spans("svelte", source).unwrap();
    let captures = get_captures(&spans);

    assert!(
        captures.contains("keyword"),
        "Svelte TypeScript should have keyword highlights. Found: {:?}",
        captures
    );
}

// ========================================================================
// High-Level API Tests
// ========================================================================

#[test]
fn test_highlighter_api() {
    let mut highlighter = Highlighter::new();
    let source = indoc! {r#"
        <script>
            let x = 1;
        </script>
        <style>
            h1 { color: red; }
        </style>
    "#};

    let html = highlighter.highlight("svelte", source).unwrap();

    // JS should be highlighted
    assert!(
        html.contains("<a-k>let</a-k>"),
        "JS keyword should be highlighted. Got: {}",
        html
    );

    // CSS should have highlighting tags
    assert!(
        html.contains("<a-"),
        "CSS should have highlighting. Got: {}",
        html
    );
}
