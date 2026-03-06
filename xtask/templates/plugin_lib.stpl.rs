//! <%= grammar_id %> grammar plugin for arborium.

use wasm_bindgen::prelude::*;
use arborium_plugin_runtime::{HighlightConfig, PluginRuntime};
use arborium_wire::{Utf8ParseResult, Utf16ParseResult};
use std::cell::RefCell;

thread_local! {
    static RUNTIME: RefCell<Option<PluginRuntime>> = const { RefCell::new(None) };
}

fn with_runtime<T>(f: impl FnOnce(&mut PluginRuntime) -> T) -> T {
    RUNTIME.with(|r| {
        let mut runtime = r.borrow_mut();
        if runtime.is_none() {
            // Use &* to handle both &str constants and LazyLock<String> statics
            let config = HighlightConfig::new(
                <%= grammar_crate_name_snake %>::language(),
                &*<%= grammar_crate_name_snake %>::HIGHLIGHTS_QUERY,
                <%= grammar_crate_name_snake %>::INJECTIONS_QUERY,
                <%= grammar_crate_name_snake %>::LOCALS_QUERY,
            )
            .expect("failed to create highlight config");
            *runtime = Some(PluginRuntime::new(config));
        }
        f(runtime.as_mut().expect("runtime not initialized"))
    })
}

/// Returns the language ID for this grammar plugin.
#[wasm_bindgen]
pub fn language_id() -> String {
    "<%= grammar_id %>".to_string()
}

/// Returns the list of languages this grammar can inject into (e.g., for embedded languages).
/// Most grammars return an empty array.
#[wasm_bindgen]
pub fn injection_languages() -> Vec<String> {
    vec![]
}

/// Creates a new parser session and returns its ID.
#[wasm_bindgen]
pub fn create_session() -> u32 {
    with_runtime(|runtime| runtime.create_session())
}

/// Frees a parser session.
#[wasm_bindgen]
pub fn free_session(session: u32) {
    with_runtime(|runtime| runtime.free_session(session));
}

/// Sets the text for a parser session.
#[wasm_bindgen]
pub fn set_text(session: u32, text: &str) {
    with_runtime(|runtime| runtime.set_text(session, text));
}

/// Parses the text in a session and returns spans with UTF-8 byte offsets.
///
/// Use this for Rust code that needs to slice strings with `&source[start..end]`.
/// For JavaScript interop, use `parse_utf16` instead.
#[wasm_bindgen]
pub fn parse(session: u32) -> Result<JsValue, JsValue> {
    let result: Result<Utf8ParseResult, _> = with_runtime(|runtime| runtime.parse(session));

    match result {
        Ok(r) => serde_wasm_bindgen::to_value(&r)
            .map_err(|e| JsValue::from_str(&format!("serialization error: {}", e))),
        Err(e) => Err(JsValue::from_str(&format!("parse error: {}", e.message))),
    }
}

/// Parses the text in a session and returns spans with UTF-16 code unit indices.
///
/// Use this for JavaScript code that needs to use `String.prototype.slice()`.
/// The offsets are compatible with JavaScript string APIs.
#[wasm_bindgen]
pub fn parse_utf16(session: u32) -> Result<JsValue, JsValue> {
    let result: Result<Utf16ParseResult, _> = with_runtime(|runtime| runtime.parse_utf16(session));

    match result {
        Ok(r) => serde_wasm_bindgen::to_value(&r)
            .map_err(|e| JsValue::from_str(&format!("serialization error: {}", e))),
        Err(e) => Err(JsValue::from_str(&format!("parse error: {}", e.message))),
    }
}

/// Cancels an ongoing parse operation.
#[wasm_bindgen]
pub fn cancel(session: u32) {
    with_runtime(|runtime| runtime.cancel(session));
}
