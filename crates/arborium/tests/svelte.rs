//! Svelte injection tests.
//!
//! Tests that verify CSS and JavaScript injections work correctly in Svelte components.

#![cfg(feature = "lang-svelte")]

use arborium::tree_sitter_highlight::{Highlight, HighlightEvent, Highlighter as TsHighlighter};
use arborium::{HIGHLIGHT_NAMES, Highlighter};
use indoc::indoc;

/// A recorded highlight event for testing
#[derive(Debug, Clone, PartialEq)]
enum Event {
    Source { text: String },
    Start { name: String },
    End,
}

/// Record all highlight events for Svelte source
fn record_events(highlighter: &mut Highlighter, source: &str) -> Vec<Event> {
    // Pre-load all needed languages before extracting config references
    highlighter.get_config_mut("svelte");
    highlighter.get_config_mut("css");
    highlighter.get_config_mut("javascript");
    highlighter.get_config_mut("typescript");

    // Now we can safely get immutable references
    let config = highlighter
        .get_config("svelte")
        .expect("Svelte language not found");

    let mut ts_highlighter = TsHighlighter::new();
    let highlights = ts_highlighter
        .highlight(config, source.as_bytes(), None, |lang| {
            highlighter.get_config(lang)
        })
        .expect("Failed to highlight");

    let mut events = Vec::new();
    for event in highlights {
        let event = event.expect("Highlight event error");
        match event {
            HighlightEvent::Source { start, end } => {
                events.push(Event::Source {
                    text: source[start..end].to_string(),
                });
            }
            HighlightEvent::HighlightStart(Highlight(i)) => {
                let name = if i < HIGHLIGHT_NAMES.len() {
                    HIGHLIGHT_NAMES[i].to_string()
                } else {
                    format!("unknown_{}", i)
                };
                events.push(Event::Start { name });
            }
            HighlightEvent::HighlightEnd => {
                events.push(Event::End);
            }
        }
    }
    events
}

/// Check that specific highlight names appear in the events
fn assert_has_highlights(events: &[Event], expected_names: &[&str], context: &str) {
    let found_names: std::collections::HashSet<_> = events
        .iter()
        .filter_map(|e| match e {
            Event::Start { name } => Some(name.as_str()),
            _ => None,
        })
        .collect();

    for expected in expected_names {
        assert!(
            found_names.contains(expected),
            "{}: Expected highlight '{}' not found. Found: {:?}",
            context,
            expected,
            found_names
        );
    }
}

/// Check that a specific text appears with a specific highlight
fn assert_text_highlighted(events: &[Event], text: &str, highlight: &str, context: &str) {
    let mut current_highlights: Vec<&str> = Vec::new();
    let mut found = false;

    for event in events {
        match event {
            Event::Start { name } => {
                current_highlights.push(name);
            }
            Event::End => {
                current_highlights.pop();
            }
            Event::Source { text: src } => {
                if src.contains(text) && current_highlights.contains(&highlight) {
                    found = true;
                    break;
                }
            }
        }
    }

    assert!(
        found,
        "{}: Text '{}' should be highlighted as '{}'. Events: {:?}",
        context, text, highlight, events
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
    let events = record_events(&mut highlighter, source);

    assert_has_highlights(&events, &["keyword"], "Svelte script injection");
    assert_text_highlighted(&events, "let", "keyword", "Svelte script injection");
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
    let events = record_events(&mut highlighter, source);

    assert_has_highlights(&events, &["keyword"], "Svelte function");
    assert_text_highlighted(&events, "function", "keyword", "Svelte function");
}

#[test]
fn test_nested_braces() {
    let mut highlighter = Highlighter::new();
    let source = indoc! {r#"
        <script>
            let obj = { a: { b: { c: 1 } } };
        </script>
    "#};
    let events = record_events(&mut highlighter, source);
    assert_has_highlights(&events, &["keyword"], "Svelte nested braces");
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
