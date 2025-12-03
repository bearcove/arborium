//! Demo WASM bindings for arborium syntax highlighting
//!
//! This is a thin wrapper around arborium's high-level Highlighter API
//! that exposes functionality to JavaScript via wasm-bindgen.

use arborium::{HIGHLIGHT_NAMES, Highlighter};
use wasm_bindgen::prelude::*;

thread_local! {
    /// Cached highlighter instance - created once per thread/worker
    static HIGHLIGHTER: std::cell::RefCell<Highlighter> = std::cell::RefCell::new(Highlighter::new());
}

/// Highlight source code and return HTML
#[wasm_bindgen]
pub fn highlight(language: &str, source: &str) -> Result<String, JsValue> {
    HIGHLIGHTER.with(|h| {
        let mut highlighter = h.borrow_mut();
        highlighter
            .highlight_to_html(language, source)
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
    })
}

/// Get list of supported languages
#[wasm_bindgen]
pub fn supported_languages() -> Vec<JsValue> {
    HIGHLIGHTER.with(|h| {
        h.borrow()
            .supported_languages()
            .into_iter()
            .map(JsValue::from_str)
            .collect()
    })
}

/// Get the highlight class names
#[wasm_bindgen]
pub fn highlight_names() -> Vec<JsValue> {
    HIGHLIGHT_NAMES
        .iter()
        .map(|s| JsValue::from_str(s))
        .collect()
}
