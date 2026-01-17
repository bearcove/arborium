//! Styx highlighting tests.
//!
//! Tests that verify Styx syntax highlighting works correctly,
//! especially that keys are highlighted differently from values.

#![cfg(feature = "lang-styx")]

use arborium::Highlighter;
use arborium_highlight::Span;
use indoc::indoc;
use std::collections::HashMap;

/// Get all spans that cover a specific text
fn get_spans_for_text<'a>(spans: &'a [Span], source: &str, text: &str) -> Vec<&'a Span> {
    let Some(pos) = source.find(text) else {
        return vec![];
    };
    let start = pos as u32;
    let end = (pos + text.len()) as u32;

    spans
        .iter()
        .filter(|s| s.start <= start && s.end >= end)
        .collect()
}

/// Get the winning capture for a specific text position (highest pattern_index)
fn get_winning_capture<'a>(spans: &'a [Span], source: &str, text: &str) -> Option<&'a str> {
    let matching = get_spans_for_text(spans, source, text);
    matching
        .into_iter()
        .max_by_key(|s| s.pattern_index)
        .map(|s| s.capture.as_str())
}

/// Debug: print all spans for a source
fn print_spans(spans: &[Span], source: &str) {
    println!("\n=== All spans ===");
    for span in spans {
        let text = &source[span.start as usize..span.end as usize];
        println!(
            "  [{:3}-{:3}] pattern={:2} capture={:20} text={:?}",
            span.start, span.end, span.pattern_index, span.capture, text
        );
    }
    println!();
}

#[test]
fn test_key_value_differentiation() {
    let mut highlighter = Highlighter::new();
    let source = indoc! {r#"
        name "Styx Showcase"
        version 1.0.0
    "#};

    let spans = highlighter.highlight_spans("styx", source).unwrap();
    print_spans(&spans, source);

    // The key "name" should be a property, not a string
    let name_capture = get_winning_capture(&spans, source, "name");
    println!("'name' winning capture: {:?}", name_capture);

    // The value "Styx Showcase" should be a string
    let value_capture = get_winning_capture(&spans, source, "Styx Showcase");
    println!("'Styx Showcase' winning capture: {:?}", value_capture);

    // Keys should be properties
    assert_eq!(
        name_capture,
        Some("property"),
        "Key 'name' should be highlighted as property, not {:?}",
        name_capture
    );

    // String values should be strings
    assert_eq!(
        value_capture,
        Some("string"),
        "Value 'Styx Showcase' should be highlighted as string, not {:?}",
        value_capture
    );
}

#[test]
fn test_nested_keys() {
    let mut highlighter = Highlighter::new();
    let source = indoc! {r#"
        server {
            host localhost
            port 8080
        }
    "#};

    let spans = highlighter.highlight_spans("styx", source).unwrap();
    print_spans(&spans, source);

    // All keys should be properties
    for key in ["server", "host", "port"] {
        let capture = get_winning_capture(&spans, source, key);
        println!("'{}' winning capture: {:?}", key, capture);
        assert_eq!(
            capture,
            Some("property"),
            "Key '{}' should be highlighted as property, not {:?}",
            key,
            capture
        );
    }

    // Values should be appropriate types
    let localhost_capture = get_winning_capture(&spans, source, "localhost");
    println!("'localhost' winning capture: {:?}", localhost_capture);
    // localhost is a bare scalar used as a value, so it should be string
    assert_eq!(
        localhost_capture,
        Some("string"),
        "Value 'localhost' should be highlighted as string"
    );
}

#[test]
fn test_tags_vs_keys() {
    let mut highlighter = Highlighter::new();
    let source = indoc! {r#"
        @ examples/showcase.schema.styx
        name "Test"
    "#};

    let spans = highlighter.highlight_spans("styx", source).unwrap();
    print_spans(&spans, source);

    // @ tag should be a label (or some tag-specific highlight)
    // The exact capture depends on highlights.scm
    let tag_spans = get_spans_for_text(&spans, source, "@ examples/showcase.schema.styx");
    println!("Tag spans: {:?}", tag_spans);

    // Key should still be property
    let name_capture = get_winning_capture(&spans, source, "name");
    assert_eq!(name_capture, Some("property"));
}

#[test]
fn test_pattern_index_ordering() {
    // This test examines the raw pattern indices to understand the precedence
    let mut highlighter = Highlighter::new();
    let source = "name value";

    let spans = highlighter.highlight_spans("styx", source).unwrap();
    print_spans(&spans, source);

    // Collect pattern indices by capture type
    let mut capture_patterns: HashMap<&str, Vec<u32>> = HashMap::new();
    for span in &spans {
        capture_patterns
            .entry(&span.capture)
            .or_default()
            .push(span.pattern_index);
    }

    println!("Capture pattern indices: {:?}", capture_patterns);

    // For proper precedence, property patterns should have HIGHER indices than string patterns
    // (later patterns in highlights.scm override earlier ones)
    if let (Some(string_indices), Some(property_indices)) = (
        capture_patterns.get("string"),
        capture_patterns.get("property"),
    ) {
        let max_string = string_indices.iter().max().unwrap_or(&0);
        let min_property = property_indices.iter().min().unwrap_or(&0);

        println!("Max string pattern index: {}", max_string);
        println!("Min property pattern index: {}", min_property);

        // Property patterns should come after string patterns
        assert!(
            min_property > max_string,
            "Property patterns (min={}) should have higher indices than string patterns (max={}). \
             This means @property rules should come AFTER @string rules in highlights.scm",
            min_property,
            max_string
        );
    }
}

#[test]
fn test_doc_comment() {
    let mut highlighter = Highlighter::new();
    // Grammar requires newline after doc comment
    let source = "/// this is a doc comment\n";

    let spans = highlighter.highlight_spans("styx", source).unwrap();
    print_spans(&spans, source);

    // The entire doc comment should be one span covering "/// this is a doc comment"
    let comment_spans: Vec<_> = spans
        .iter()
        .filter(|s| s.capture.contains("comment"))
        .collect();
    println!("Comment spans: {:?}", comment_spans);

    // Should have exactly one comment span
    assert!(
        !comment_spans.is_empty(),
        "Should have at least one comment span"
    );

    // The comment span should cover both "///" and the text
    let comment_span = comment_spans[0];
    let comment_text = &source[comment_span.start as usize..comment_span.end as usize];
    println!("Comment text: {:?}", comment_text);

    assert!(
        comment_text.contains("///"),
        "Comment span should include '///'"
    );
    assert!(
        comment_text.contains("this"),
        "Comment span should include the comment text 'this'"
    );
}
