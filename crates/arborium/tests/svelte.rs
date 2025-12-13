//! Svelte injection tests.
//!
//! Tests that verify CSS and JavaScript injections work correctly in Svelte components.

#![cfg(feature = "lang-svelte")]

use arborium::Highlighter;
use indoc::indoc;

/// Check that HTML contains specific highlight tags
fn assert_has_tag(html: &str, tag: &str, context: &str) {
    assert!(
        html.contains(tag),
        "{}: Expected tag '{}' not found in HTML",
        context,
        tag
    );
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

    assert_has_tag(
        &html,
        "<a-k>",
        "Svelte script should have keyword highlighting",
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

    assert_has_tag(
        &html,
        "<a-k>",
        "Svelte function should have keyword highlighting",
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
    assert_has_tag(
        &html,
        "<a-k>",
        "Svelte nested braces should have highlighting",
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
    let events = record_events(&mut highlighter, source);

    assert_has_highlights(&events, &["property"], "Svelte style injection");
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
    let events = record_events(&mut highlighter, source);

    assert_has_highlights(&events, &["property"], "Svelte multiple selectors");
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
    let events = record_events(&mut highlighter, source);

    // Template expressions should produce events
    assert!(!events.is_empty(), "Svelte template should produce events");
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
    let events = record_events(&mut highlighter, source);
    assert!(!events.is_empty());
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
    let events = record_events(&mut highlighter, source);

    // JS keywords
    assert_has_highlights(&events, &["keyword"], "Svelte full component - JS");
    assert_text_highlighted(&events, "export", "keyword", "Svelte full component");
    assert_text_highlighted(&events, "function", "keyword", "Svelte full component");

    // CSS properties
    assert_has_highlights(&events, &["property"], "Svelte full component - CSS");
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
    let events = record_events(&mut highlighter, source);

    assert_has_highlights(&events, &["keyword"], "Svelte TypeScript");
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

    let html = highlighter.highlight_to_html("svelte", source).unwrap();

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
