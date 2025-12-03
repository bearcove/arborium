//! Vue injection tests.
//!
//! Tests that verify CSS and JavaScript injections work correctly in Vue SFCs.

#![cfg(feature = "lang-vue")]

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

/// Record all highlight events for Vue source
fn record_events(highlighter: &mut Highlighter, source: &str) -> Vec<Event> {
    // Pre-load all needed languages before extracting config references
    highlighter.get_config_mut("vue");
    highlighter.get_config_mut("css");
    highlighter.get_config_mut("javascript");
    highlighter.get_config_mut("typescript");

    // Now we can safely get immutable references
    let config = highlighter
        .get_config("vue")
        .expect("Vue language not found");

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
fn test_isolated_script() {
    let mut highlighter = Highlighter::new();
    let source = indoc! {r#"
        <script>
        export default {
            data() {
                return { name: "world" };
            }
        }
        </script>
    "#};
    let events = record_events(&mut highlighter, source);

    assert_has_highlights(&events, &["keyword"], "Vue script injection");
    assert_text_highlighted(&events, "export", "keyword", "Vue script injection");
}

#[test]
fn test_isolated_style() {
    let mut highlighter = Highlighter::new();
    let source = indoc! {r#"
        <style>
        .hello {
            color: blue;
            font-weight: bold;
        }
        </style>
    "#};
    let events = record_events(&mut highlighter, source);

    assert_has_highlights(&events, &["property"], "Vue style injection");
}

#[test]
fn test_scoped_style() {
    let mut highlighter = Highlighter::new();
    let source = indoc! {r#"
        <style scoped>
        .hello {
            color: red;
        }
        </style>
    "#};
    let events = record_events(&mut highlighter, source);

    assert_has_highlights(&events, &["property"], "Vue scoped style injection");
}

#[test]
fn test_full_sfc() {
    let mut highlighter = Highlighter::new();
    let source = indoc! {r#"
        <template>
            <div class="hello">
                <h1>{{ msg }}</h1>
            </div>
        </template>

        <script>
        export default {
            name: 'HelloWorld',
            props: {
                msg: String
            }
        }
        </script>

        <style scoped>
        .hello {
            text-align: center;
        }
        h1 {
            font-weight: normal;
        }
        </style>
    "#};
    let events = record_events(&mut highlighter, source);

    assert_has_highlights(&events, &["keyword"], "Vue SFC - JS");
    assert_has_highlights(&events, &["property"], "Vue SFC - CSS");
}

#[test]
fn test_typescript() {
    let mut highlighter = Highlighter::new();
    let source = indoc! {r#"
        <script lang="ts">
        import { defineComponent } from 'vue';

        interface Props {
            msg: string;
        }

        export default defineComponent({
            props: {
                msg: String
            }
        });
        </script>
    "#};
    let events = record_events(&mut highlighter, source);

    assert_has_highlights(&events, &["keyword"], "Vue TypeScript");
}

#[test]
fn test_highlighter_api() {
    let mut highlighter = Highlighter::new();
    let source = indoc! {r#"
        <script>
        export default {
            data() { return {}; }
        }
        </script>
        <style>
        .foo { color: blue; }
        </style>
    "#};

    let html = highlighter.highlight_to_html("vue", source).unwrap();

    assert!(
        html.contains("<a-k>export</a-k>"),
        "JS keyword should be highlighted. Got: {}",
        html
    );
}
