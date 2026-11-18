//! JSON highlighting tests.

#![cfg(feature = "lang-json")]

use arborium::Highlighter;

#[test]
fn test_object_keys_distinct_from_string_values() {
    let mut highlighter = Highlighter::new();
    let html = highlighter
        .highlight("json", r#"{"schema_version": "1.1"}"#)
        .unwrap();

    assert!(
        html.contains("<a-pr>&quot;schema_version&quot;</a-pr>"),
        "object key should use the property slot: {html}"
    );
    assert!(
        html.contains("<a-s>&quot;1.1&quot;</a-s>"),
        "string value should use the string slot: {html}"
    );
}
