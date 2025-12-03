//! HTML injection tests.
//!
//! Tests that verify CSS and JavaScript injections work correctly in HTML.

#![cfg(feature = "lang-html")]

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

/// Record all highlight events for HTML source
fn record_events(highlighter: &mut Highlighter, source: &str) -> Vec<Event> {
    // Pre-load all needed languages before extracting config references
    highlighter.get_config_mut("html");
    highlighter.get_config_mut("css");
    highlighter.get_config_mut("javascript");

    // Now we can safely get immutable references
    let config = highlighter
        .get_config("html")
        .expect("HTML language not found");

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

#[test]
fn test_isolated_style() {
    let mut highlighter = Highlighter::new();
    let source = indoc! {r#"
        <style>
            h1 { color: red; }
        </style>
    "#};
    let events = record_events(&mut highlighter, source);

    assert_has_highlights(&events, &["property"], "HTML style injection");
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
    let events = record_events(&mut highlighter, source);

    assert_has_highlights(&events, &["keyword"], "HTML script injection");
    assert_text_highlighted(&events, "let", "keyword", "HTML script injection");
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
    let events = record_events(&mut highlighter, source);

    assert_has_highlights(
        &events,
        &["tag", "property", "string"],
        "HTML mixed content",
    );
}

#[test]
fn test_empty_style_tag() {
    let mut highlighter = Highlighter::new();
    let source = "<style></style>";
    let events = record_events(&mut highlighter, source);
    assert!(!events.is_empty());
}

#[test]
fn test_empty_script_tag() {
    let mut highlighter = Highlighter::new();
    let source = "<script></script>";
    let events = record_events(&mut highlighter, source);
    assert!(!events.is_empty());
}

#[test]
fn test_inline_event_handler() {
    let mut highlighter = Highlighter::new();
    let source = r#"<button onclick="alert('hello')">Click</button>"#;
    let events = record_events(&mut highlighter, source);
    assert!(!events.is_empty());
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

    let html = highlighter.highlight_to_html("html", source).unwrap();

    assert!(
        html.contains("<a-k>const</a-k>"),
        "JS keyword should be highlighted. Got: {}",
        html
    );
}
