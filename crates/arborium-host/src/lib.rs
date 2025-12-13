//! Arborium WASM host for browser.
//!
//! Uses wasm-bindgen for JS interop and wasm-bindgen-futures for async.
//! Grammar plugins (WIT components) are loaded on demand via JS.
//!
//! This crate implements `GrammarProvider` to use the shared highlighting
//! engine from `arborium_highlight`, ensuring Rust and browser use the
//! same injection handling logic.
//!
//! ## JS Interface
//!
//! The host expects these functions to be available on `window.arboriumHost`:
//!
//! ```javascript
//! window.arboriumHost = {
//!     // Check if a language is available (sync, for fast rejection).
//!     isLanguageAvailable(language) { ... },
//!
//!     // Load a grammar plugin, returns a handle (async).
//!     async loadGrammar(language) { ... },
//!
//!     // Parse text using a grammar handle (sync).
//!     parse(handle, text) { ... },
//! };
//! ```

use std::collections::HashMap;

use wasm_bindgen::prelude::*;

use arborium_highlight::{
    AsyncHighlighter, Grammar, GrammarProvider, HighlightConfig as CoreConfig,
    HtmlFormat as CoreHtmlFormat, Injection, ParseResult, Span,
};

/// Grammar handle type (matches JS side)
type GrammarHandle = u32;

// JS functions imported from the host environment.
#[wasm_bindgen]
extern "C" {
    /// Check if a language grammar is available.
    #[wasm_bindgen(js_namespace = arboriumHost, js_name = isLanguageAvailable)]
    fn js_is_language_available(language: &str) -> bool;

    /// Load a grammar plugin, returns a handle.
    /// Returns a Promise resolving to a handle (u32), or 0 if not found.
    #[wasm_bindgen(js_namespace = arboriumHost, js_name = loadGrammar, catch)]
    async fn js_load_grammar(language: &str) -> Result<JsValue, JsValue>;

    /// Parse text using a grammar handle.
    /// Returns { spans: [...], injections: [...] }
    #[wasm_bindgen(js_namespace = arboriumHost, js_name = parse)]
    fn js_parse(handle: GrammarHandle, text: &str) -> JsValue;
}

/// Parse the JS result object into our ParseResult.
fn parse_js_result(value: JsValue) -> ParseResult {
    use js_sys::{Array, Object, Reflect};

    if value.is_undefined() || value.is_null() {
        return ParseResult::default();
    }

    let obj = Object::from(value);

    // Get spans array
    let spans_val = match Reflect::get(&obj, &"spans".into()) {
        Ok(v) => v,
        Err(_) => return ParseResult::default(),
    };
    let spans_arr = Array::from(&spans_val);

    let mut spans = Vec::with_capacity(spans_arr.length() as usize);
    for i in 0..spans_arr.length() {
        let span_obj = spans_arr.get(i);
        let start = Reflect::get(&span_obj, &"start".into())
            .ok()
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as u32;
        let end = Reflect::get(&span_obj, &"end".into())
            .ok()
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as u32;
        let capture = Reflect::get(&span_obj, &"capture".into())
            .ok()
            .and_then(|v| v.as_string())
            .unwrap_or_default();

        spans.push(Span {
            start,
            end,
            capture,
        });
    }

    // Get injections array
    let injections_val = match Reflect::get(&obj, &"injections".into()) {
        Ok(v) => v,
        Err(_) => {
            return ParseResult {
                spans,
                injections: vec![],
            };
        }
    };
    let injections_arr = Array::from(&injections_val);

    let mut injections = Vec::with_capacity(injections_arr.length() as usize);
    for i in 0..injections_arr.length() {
        let inj_obj = injections_arr.get(i);
        let start = Reflect::get(&inj_obj, &"start".into())
            .ok()
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as u32;
        let end = Reflect::get(&inj_obj, &"end".into())
            .ok()
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as u32;
        let language = Reflect::get(&inj_obj, &"language".into())
            .ok()
            .and_then(|v| v.as_string())
            .unwrap_or_default();
        let include_children = Reflect::get(&inj_obj, &"includeChildren".into())
            .ok()
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        injections.push(Injection {
            start,
            end,
            language,
            include_children,
        });
    }

    ParseResult { spans, injections }
}

/// A grammar that wraps a JS grammar handle.
///
/// When `parse()` is called, it calls into JS synchronously.
pub struct JsGrammar {
    handle: GrammarHandle,
}

impl JsGrammar {
    fn new(handle: GrammarHandle) -> Self {
        Self { handle }
    }
}

impl Grammar for JsGrammar {
    fn parse(&mut self, text: &str) -> ParseResult {
        let result = js_parse(self.handle, text);
        parse_js_result(result)
    }
}

/// Grammar provider that loads grammars from JS.
///
/// Implements `GrammarProvider` so we can use the shared `AsyncHighlighter`
/// from `arborium_highlight`.
pub struct JsGrammarProvider {
    /// Cached grammars by language name
    grammars: HashMap<String, JsGrammar>,
}

impl JsGrammarProvider {
    pub fn new() -> Self {
        Self {
            grammars: HashMap::new(),
        }
    }
}

impl Default for JsGrammarProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl GrammarProvider for JsGrammarProvider {
    type Grammar = JsGrammar;

    // This crate is only compiled for wasm32, so we use the non-Send version
    #[cfg(target_arch = "wasm32")]
    async fn get(&mut self, language: &str) -> Option<&mut Self::Grammar> {
        // Check if language is available (fast sync check)
        if !js_is_language_available(language) {
            return None;
        }

        // Check if we already have this grammar cached
        if self.grammars.contains_key(language) {
            return self.grammars.get_mut(language);
        }

        // Load the grammar from JS (async)
        let handle = match js_load_grammar(language).await {
            Ok(val) => val.as_f64().unwrap_or(0.0) as GrammarHandle,
            Err(_) => return None,
        };

        // 0 means not found
        if handle == 0 {
            return None;
        }

        // Cache and return
        self.grammars
            .insert(language.to_string(), JsGrammar::new(handle));
        self.grammars.get_mut(language)
    }

    // Stub for non-wasm32 targets (never used, just for compilation)
    #[cfg(not(target_arch = "wasm32"))]
    async fn get(&mut self, _language: &str) -> Option<&mut Self::Grammar> {
        unreachable!("arborium-host is only for wasm32")
    }
}

/// Configuration for highlighting.
#[wasm_bindgen]
pub struct HighlightConfig {
    max_injection_depth: u32,
    html_format: CoreHtmlFormat,
}

#[wasm_bindgen]
impl HighlightConfig {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            max_injection_depth: 3,
            html_format: CoreHtmlFormat::default(),
        }
    }

    #[wasm_bindgen(js_name = setMaxInjectionDepth)]
    pub fn set_max_injection_depth(&mut self, depth: u32) {
        self.max_injection_depth = depth;
    }

    /// Set HTML format to custom elements (default): `<a-k>`, `<a-f>`, etc.
    #[wasm_bindgen(js_name = setHtmlFormatCustomElements)]
    pub fn set_html_format_custom_elements(&mut self) {
        self.html_format = CoreHtmlFormat::CustomElements;
    }

    /// Set HTML format to custom elements with custom prefix.
    #[wasm_bindgen(js_name = setHtmlFormatCustomElementsWithPrefix)]
    pub fn set_html_format_custom_elements_with_prefix(&mut self, prefix: String) {
        self.html_format = CoreHtmlFormat::CustomElementsWithPrefix(prefix);
    }

    /// Set HTML format to class names: `<span class="keyword">`, etc.
    #[wasm_bindgen(js_name = setHtmlFormatClassNames)]
    pub fn set_html_format_class_names(&mut self) {
        self.html_format = CoreHtmlFormat::ClassNames;
    }

    /// Set HTML format to class names with custom prefix.
    #[wasm_bindgen(js_name = setHtmlFormatClassNamesWithPrefix)]
    pub fn set_html_format_class_names_with_prefix(&mut self, prefix: String) {
        self.html_format = CoreHtmlFormat::ClassNamesWithPrefix(prefix);
    }
}

impl Default for HighlightConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Highlight source code, resolving injections recursively.
///
/// This uses the shared `AsyncHighlighter` from `arborium_highlight`,
/// ensuring the same injection handling logic as Rust native.
#[wasm_bindgen]
pub async fn highlight(language: &str, source: &str) -> Result<String, JsValue> {
    highlight_with_config(language, source, HighlightConfig::default()).await
}

/// Highlight with custom configuration.
#[wasm_bindgen(js_name = highlightWithConfig)]
pub async fn highlight_with_config(
    language: &str,
    source: &str,
    config: HighlightConfig,
) -> Result<String, JsValue> {
    let core_config = CoreConfig {
        max_injection_depth: config.max_injection_depth,
        html_format: config.html_format.clone(),
    };

    let provider = JsGrammarProvider::new();
    let mut highlighter = AsyncHighlighter::with_config(provider, core_config);

    highlighter
        .highlight(language, source)
        .await
        .map_err(|e| JsValue::from_str(&format!("{}", e)))
}

/// Check if a language is available for highlighting.
#[wasm_bindgen(js_name = isLanguageAvailable)]
pub fn is_language_available(language: &str) -> bool {
    js_is_language_available(language)
}
