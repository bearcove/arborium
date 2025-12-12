//! <%= grammar_id %> grammar plugin for arborium.

use wasm_bindgen::prelude::*;
use arborium_plugin_runtime::{HighlightConfig, PluginRuntime};
use arborium_wire::ParseResult as WireParseResult;
use std::cell::RefCell;

thread_local! {
    static RUNTIME: RefCell<Option<PluginRuntime>> = const { RefCell::new(None) };
}

fn get_or_init_runtime() -> &'static RefCell<Option<PluginRuntime>> {
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
        unsafe { &*(r as *const _) }
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
    get_or_init_runtime()
        .borrow_mut()
        .as_mut()
        .expect("runtime not initialized")
        .create_session()
}

/// Frees a parser session.
#[wasm_bindgen]
pub fn free_session(session: u32) {
    get_or_init_runtime()
        .borrow_mut()
        .as_mut()
        .expect("runtime not initialized")
        .free_session(session);
}

/// Sets the text for a parser session.
#[wasm_bindgen]
pub fn set_text(session: u32, text: &str) {
    get_or_init_runtime()
        .borrow_mut()
        .as_mut()
        .expect("runtime not initialized")
        .set_text(session, text);
}

/// Parses the text in a session and returns the result as a JS value.
///
/// The result is a JavaScript object representation of ParseResult containing spans and injections.
#[wasm_bindgen]
pub fn parse(session: u32) -> Result<JsValue, JsValue> {
    let result: Result<WireParseResult, _> = get_or_init_runtime()
        .borrow_mut()
        .as_mut()
        .expect("runtime not initialized")
        .parse(session);

    match result {
        Ok(r) => serde_wasm_bindgen::to_value(&r)
            .map_err(|e| JsValue::from_str(&format!("serialization error: {}", e))),
        Err(e) => Err(JsValue::from_str(&format!("parse error: {}", e.message))),
    }
}

/// Cancels an ongoing parse operation.
#[wasm_bindgen]
pub fn cancel(session: u32) {
    get_or_init_runtime()
        .borrow_mut()
        .as_mut()
        .expect("runtime not initialized")
        .cancel(session);
}
