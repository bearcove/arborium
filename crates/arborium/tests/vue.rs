//! Vue injection tests.
//!
//! Tests that verify CSS and JavaScript injections work correctly in Vue SFCs.

#![cfg(all(
    feature = "lang-vue",
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
    let spans = highlighter.highlight_spans("vue", source).unwrap();
    let captures = get_captures(&spans);

    assert!(
        captures.contains("keyword"),
        "Vue script injection should have keyword highlight. Found: {:?}",
        captures
    );
    assert!(
        has_capture_at(&spans, source, "export", "keyword"),
        "export should be highlighted as keyword"
    );
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
    let spans = highlighter.highlight_spans("vue", source).unwrap();
    let captures = get_captures(&spans);

    assert!(
        captures.contains("property"),
        "Vue style injection should have property highlight. Found: {:?}",
        captures
    );
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
    let spans = highlighter.highlight_spans("vue", source).unwrap();
    let captures = get_captures(&spans);

    assert!(
        captures.contains("property"),
        "Vue scoped style injection should have property highlight. Found: {:?}",
        captures
    );
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
    let spans = highlighter.highlight_spans("vue", source).unwrap();
    let captures = get_captures(&spans);

    assert!(
        captures.contains("keyword"),
        "Vue SFC should have JS keyword highlights"
    );
    assert!(
        captures.contains("property"),
        "Vue SFC should have CSS property highlights"
    );
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
    let spans = highlighter.highlight_spans("vue", source).unwrap();
    let captures = get_captures(&spans);

    assert!(
        captures.contains("keyword"),
        "Vue TypeScript should have keyword highlights. Found: {:?}",
        captures
    );
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

    let html = highlighter.highlight("vue", source).unwrap();

    assert!(
        html.contains("<a-k>export</a-k>"),
        "JS keyword should be highlighted. Got: {}",
        html
    );
}
