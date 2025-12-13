//! HTML transformation using lol_html.
//!
//! Transforms rustdoc HTML to add syntax highlighting for non-Rust code blocks.

use arborium::Highlighter;
use lol_html::html_content::ContentType;
use lol_html::{ElementContentHandlers, HtmlRewriter, Selector, Settings};
use std::borrow::Cow;
use std::cell::RefCell;
use std::rc::Rc;

/// Result of transforming an HTML file.
#[derive(Debug, Default, Clone)]
pub struct TransformResult {
    /// Number of code blocks that were highlighted.
    pub blocks_highlighted: usize,
    /// Number of code blocks that were skipped (already Rust, or unknown language).
    pub blocks_skipped: usize,
    /// Languages that were encountered but not supported.
    pub unsupported_languages: Vec<String>,
}

/// State shared between lol_html handlers.
#[derive(Default)]
struct TransformState {
    /// The language of the current code block (if any).
    current_lang: Option<String>,
    /// Accumulated text content from the current code block.
    collected_text: String,
    /// Statistics about the transformation.
    result: TransformResult,
    /// The highlighter (wrapped for sharing).
    highlighter: Option<Highlighter>,
}

/// Transform rustdoc HTML, adding syntax highlighting to non-Rust code blocks.
///
/// Uses lol_html for streaming HTML transformation.
pub fn transform_html(
    html: &str,
    highlighter: Highlighter,
) -> Result<(String, TransformResult), TransformError> {
    // Shared state wrapped in Rc<RefCell<>> for the closure dance
    let state = Rc::new(RefCell::new(TransformState {
        highlighter: Some(highlighter),
        ..Default::default()
    }));

    let mut output = Vec::new();

    let state_for_pre = state.clone();
    let state_for_code_el = state.clone();
    let state_for_code_text = state.clone();

    {
        let mut rewriter = HtmlRewriter::new(
            Settings {
                element_content_handlers: vec![
                    // Handler for <pre class="language-*"> - extract language
                    (
                        Cow::<Selector>::Owned("pre[class*='language-']".parse().unwrap()),
                        ElementContentHandlers::default().element(move |el: &mut lol_html::html_content::Element| {
                            let mut state = state_for_pre.borrow_mut();

                            let class = el.get_attribute("class").unwrap_or_default();

                            // Skip if it has "rust" class (already highlighted by rustdoc)
                            // Use word boundary check to avoid false positives like "language-rustscript"
                            if class.split_whitespace().any(|c| c == "rust") {
                                state.result.blocks_skipped += 1;
                                state.current_lang = None;
                                return Ok(());
                            }

                            // Extract language from class
                            state.current_lang = extract_language_from_class(&class);

                            Ok(())
                        }),
                    ),
                    // Handler for <code> inside language pre - collect text and replace
                    (
                        Cow::<Selector>::Owned("pre[class*='language-'] code".parse().unwrap()),
                        ElementContentHandlers::default()
                            .element({
                                let state_ref = state_for_code_el.clone();
                                move |el: &mut lol_html::html_content::Element| {
                                    // Check if we have a language set
                                    let has_lang = state_ref.borrow().current_lang.is_some();
                                    if !has_lang {
                                        return Ok(());
                                    }

                                    // Clear collected text for this block
                                    state_ref.borrow_mut().collected_text.clear();

                                    // Set up end tag handler
                                    let state_for_end = state_ref.clone();
                                    if let Some(handlers) = el.end_tag_handlers() {
                                        handlers.push(Box::new(move |end| {
                                            let mut state = state_for_end.borrow_mut();

                                            let lang = match &state.current_lang {
                                                Some(l) => l.clone(),
                                                None => return Ok(()),
                                            };

                                            // Decode HTML entities
                                            let decoded =
                                                decode_html_entities(&state.collected_text);

                                            // Highlight the code
                                            let highlighter = state.highlighter.as_mut().unwrap();
                                            match highlighter.highlight_to_html(&lang, &decoded) {
                                                Ok(highlighted) => {
                                                    // Insert highlighted content before </code>
                                                    end.before(&highlighted, ContentType::Html);
                                                    state.result.blocks_highlighted += 1;
                                                }
                                                Err(
                                                    arborium::highlighter::HighlightError::UnsupportedLanguage(
                                                        _,
                                                    ),
                                                ) => {
                                                    // Language not supported - keep original
                                                    if !state
                                                        .result
                                                        .unsupported_languages
                                                        .contains(&lang)
                                                    {
                                                        state
                                                            .result
                                                            .unsupported_languages
                                                            .push(lang.clone());
                                                    }
                                                    // Re-insert the original text
                                                    end.before(
                                                        &state.collected_text,
                                                        ContentType::Html,
                                                    );
                                                    state.result.blocks_skipped += 1;
                                                }
                                                Err(_) => {
                                                    // Other error - keep original
                                                    end.before(
                                                        &state.collected_text,
                                                        ContentType::Html,
                                                    );
                                                    state.result.blocks_skipped += 1;
                                                }
                                            }

                                            // Reset for next block
                                            state.current_lang = None;
                                            state.collected_text.clear();

                                            Ok(())
                                        }));
                                    }

                                    Ok(())
                                }
                            })
                            .text(move |text: &mut lol_html::html_content::TextChunk| {
                                let mut state = state_for_code_text.borrow_mut();

                                // Only collect if we're in a language block
                                if state.current_lang.is_some() {
                                    state.collected_text.push_str(text.as_str());
                                    text.remove(); // Remove original - we'll re-insert highlighted
                                }

                                Ok(())
                            }),
                    ),
                ],
                ..Settings::new()
            },
            |c: &[u8]| output.extend_from_slice(c),
        );

        rewriter
            .write(html.as_bytes())
            .map_err(TransformError::Rewrite)?;
        rewriter.end().map_err(TransformError::Rewrite)?;
    }

    // Extract final result
    let result = state.borrow().result.clone();
    let output_str = String::from_utf8(output)
        .map_err(|e| TransformError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?;

    Ok((output_str, result))
}

/// Extract language name from a class attribute like "language-toml" or "language-json".
fn extract_language_from_class(class: &str) -> Option<String> {
    for part in class.split_whitespace() {
        if let Some(lang) = part.strip_prefix("language-") {
            if !lang.is_empty() && lang != "rust" {
                return Some(lang.to_string());
            }
        }
    }
    None
}

fn decode_html_entities(s: &str) -> String {
    // Note: &amp; must be decoded LAST to avoid double-decoding
    // e.g., "&lt;" should become "<", not "&<"
    s.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&#x27;", "'")
        .replace("&amp;", "&")
}

/// Errors that can occur during HTML transformation.
#[derive(Debug)]
pub enum TransformError {
    /// Error from lol_html rewriter.
    Rewrite(lol_html::errors::RewritingError),
    /// IO error.
    Io(std::io::Error),
}

impl std::fmt::Display for TransformError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransformError::Rewrite(e) => write!(f, "HTML rewrite error: {}", e),
            TransformError::Io(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl std::error::Error for TransformError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_language_from_class() {
        assert_eq!(
            extract_language_from_class("language-toml"),
            Some("toml".to_string())
        );
        assert_eq!(
            extract_language_from_class("language-json foo"),
            Some("json".to_string())
        );
        assert_eq!(extract_language_from_class("language-rust"), None);
        assert_eq!(extract_language_from_class("foo bar"), None);
    }

    #[test]
    fn test_decode_html_entities() {
        assert_eq!(decode_html_entities("&lt;div&gt;"), "<div>");
        assert_eq!(decode_html_entities("foo &amp; bar"), "foo & bar");
    }
}
