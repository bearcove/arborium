use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn get_theme_css() -> String {
    arborium_theme::builtin::tokyo_night().to_css("pre")
}

#[wasm_bindgen]
pub fn highlight_code(code: &str, language: &str) -> Result<String, JsValue> {
    // Create a highlighter instance
    let mut highlighter = arborium::Highlighter::new();

    // Try to highlight the code
    let result = highlighter
        .highlight(language, code)
        .map_err(|e| JsValue::from_str(&format!("Highlighting error: {:?}", e)))?;

    Ok(result.to_string())
}

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    Ok(())
}
