//! Tests for the `get_language` function.

#[test]
#[cfg(feature = "lang-rust")]
fn get_rust() {
    let lang = arborium::get_language("rust");
    assert!(lang.is_some(), "rust language should be available");
}

#[test]
fn get_unsupported() {
    let lang = arborium::get_language("bartholomew");
    assert!(lang.is_none(), "unknown language should return None");
}
