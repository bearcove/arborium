//! Arborium-powered syntax highlighter for miette diagnostics.
//!
//! This crate provides a [`MietteHighlighter`] that integrates arborium's tree-sitter
//! based syntax highlighting into miette's error reporting output.
//!
//! # Quick Start
//!
//! Install the highlighter globally and miette will automatically use it:
//!
//! ```rust,ignore
//! fn main() {
//!     // Install the highlighter (call once at startup)
//!     miette_arborium::install_global().ok();
//!
//!     // Now all miette errors will have syntax highlighting
//!     // ... your code ...
//! }
//! ```
//!
//! # Features
//!
//! - **Language detection**: Automatically detects language from file extension or accepts
//!   an explicit language name
//! - **Full arborium language support**: Supports all languages enabled via Cargo features
//!   (passthrough to arborium)
//! - **Minimal dependencies**: Uses arborium's tree-sitter-based highlighter
//! - **ANSI terminal output**: Renders highlighted code with terminal colors
//!
//! # Example
//!
//! ```rust,ignore
//! use miette::{Diagnostic, NamedSource, SourceSpan};
//! use thiserror::Error;
//!
//! #[derive(Error, Debug, Diagnostic)]
//! #[error("syntax error")]
//! struct SyntaxError {
//!     #[source_code]
//!     src: NamedSource<String>,
//!     #[label("unexpected token here")]
//!     span: SourceSpan,
//! }
//!
//! fn main() -> miette::Result<()> {
//!     miette_arborium::install_global().ok();
//!
//!     let source = r#"fn main() {
//!     let x = 42
//!     println!("{}", x);
//! }"#;
//!
//!     Err(SyntaxError {
//!         src: NamedSource::new("example.rs", source.to_string()),
//!         span: (32..33).into(),
//!     })?
//! }
//! ```

use std::sync::RwLock;

use arborium::Highlighter;
use miette::highlighters::Highlighter as MietteHighlighterTrait;
use owo_colors::Style;

/// A syntax highlighter for miette that uses arborium for tree-sitter based highlighting.
///
/// This highlighter can be installed globally using [`install_global`] or used directly
/// by setting it on miette's `GraphicalReportHandler`.
pub struct MietteHighlighter {
    inner: RwLock<Highlighter>,
}

impl MietteHighlighter {
    /// Create a new miette highlighter.
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(Highlighter::new()),
        }
    }

    /// Returns whether a language is supported by this highlighter.
    pub fn is_supported(&self, language: &str) -> bool {
        // Check if arborium has this language available
        // We do this by trying to highlight an empty string
        let mut inner = self.inner.write().unwrap();
        inner.highlight(language, "").is_ok()
    }

    /// Detect language from a source name (file path or extension).
    ///
    /// This delegates to [`arborium::detect_language`] which is generated from
    /// the grammar registry, ensuring extensions stay in sync with supported languages.
    pub fn detect_language(source_name: &str) -> Option<&'static str> {
        arborium::detect_language(source_name)
    }
}

impl Default for MietteHighlighter {
    fn default() -> Self {
        Self::new()
    }
}

impl MietteHighlighterTrait for MietteHighlighter {
    fn start_highlighter_state<'h>(
        &'h self,
        source: &dyn miette::SpanContents<'_>,
    ) -> Box<dyn miette::highlighters::HighlighterState + 'h> {
        // Try to detect language from source name
        let language = source.name().and_then(Self::detect_language);

        // Check if we support the language
        let language = match language {
            Some(lang) if self.is_supported(lang) => Some(lang),
            _ => None,
        };

        Box::new(MietteHighlighterState {
            highlighter: self,
            language,
        })
    }
}

struct MietteHighlighterState<'h> {
    highlighter: &'h MietteHighlighter,
    language: Option<&'static str>,
}

impl miette::highlighters::HighlighterState for MietteHighlighterState<'_> {
    fn highlight_line<'s>(&mut self, line: &'s str) -> Vec<owo_colors::Styled<&'s str>> {
        let Some(language) = self.language else {
            // No language detected, return unhighlighted
            return vec![Style::new().style(line)];
        };

        // Use arborium to highlight the line
        let mut inner = self.highlighter.inner.write().unwrap();
        let html = match inner.highlight(language, line) {
            Ok(html) => html,
            Err(_) => return vec![Style::new().style(line)],
        };

        // Parse the HTML output and convert to styled spans
        parse_html_to_spans(line, &html)
    }
}

/// Parse arborium's HTML output into miette styled spans.
///
/// Arborium outputs HTML like `<a-k>fn</a-k>` where `a-k` is short for "keyword".
/// We convert these to styled spans with appropriate ANSI colors.
fn parse_html_to_spans<'s>(original: &'s str, html: &str) -> Vec<owo_colors::Styled<&'s str>> {
    let mut spans = Vec::new();
    let mut current_pos = 0;
    let mut html_pos = 0;
    let html_bytes = html.as_bytes();

    while html_pos < html_bytes.len() {
        if html_bytes[html_pos] == b'<' {
            // Check if it's a closing tag
            if html_pos + 1 < html_bytes.len() && html_bytes[html_pos + 1] == b'/' {
                // Find end of closing tag
                if let Some(end) = html[html_pos..].find('>') {
                    html_pos += end + 1;
                    continue;
                }
            }

            // It's an opening tag - find the tag name
            if let Some(end) = html[html_pos..].find('>') {
                let tag = &html[html_pos + 1..html_pos + end];

                // Find the matching closing tag
                let close_tag = format!("</{}>", tag);
                if let Some(close_pos) = html[html_pos + end + 1..].find(&close_tag) {
                    let content_start = html_pos + end + 1;
                    let content_end = content_start + close_pos;
                    let content = &html[content_start..content_end];

                    // The content length in the original string
                    let content_len = content.len();

                    if current_pos + content_len <= original.len() {
                        let style = style_for_tag(tag);
                        spans.push(style.style(&original[current_pos..current_pos + content_len]));
                        current_pos += content_len;
                    }

                    html_pos = content_end + close_tag.len();
                    continue;
                }
            }
        }

        // Regular character - count how many chars until next tag or end
        let next_tag = html[html_pos..].find('<').unwrap_or(html.len() - html_pos);
        let chunk = &html[html_pos..html_pos + next_tag];

        // Handle HTML entities
        let decoded = decode_html_entities(chunk);
        let decoded_len = decoded.len();

        if !decoded.is_empty() && current_pos + decoded_len <= original.len() {
            spans.push(Style::new().style(&original[current_pos..current_pos + decoded_len]));
            current_pos += decoded_len;
        }

        html_pos += next_tag;
    }

    // Handle any remaining text
    if current_pos < original.len() {
        spans.push(Style::new().style(&original[current_pos..]));
    }

    // If we didn't produce any spans, return the original line unhighlighted
    if spans.is_empty() {
        return vec![Style::new().style(original)];
    }

    spans
}

/// Decode common HTML entities back to their characters.
fn decode_html_entities(s: &str) -> String {
    s.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}

/// Map an arborium tag name to an owo_colors style.
///
/// Arborium uses short tag names like `a-k` (keyword), `a-s` (string), etc.
fn style_for_tag(tag: &str) -> Style {
    use owo_colors::AnsiColors;

    match tag {
        // Keywords - bold magenta
        "a-k" | "a-kw" => Style::new().bold().color(AnsiColors::Magenta),

        // Strings - green
        "a-s" | "a-str" => Style::new().color(AnsiColors::Green),

        // Comments - dimmed cyan
        "a-c" | "a-cm" => Style::new().dimmed().color(AnsiColors::Cyan),

        // Functions - bold blue
        "a-f" | "a-fn" => Style::new().bold().color(AnsiColors::Blue),

        // Types - yellow
        "a-t" | "a-ty" => Style::new().color(AnsiColors::Yellow),

        // Variables - default
        "a-v" | "a-var" => Style::new(),

        // Numbers/constants - cyan
        "a-n" | "a-num" => Style::new().color(AnsiColors::Cyan),

        // Operators - white/default
        "a-o" | "a-op" => Style::new(),

        // Punctuation - dimmed
        "a-p" | "a-punc" => Style::new().dimmed(),

        // Attributes - italic cyan
        "a-a" | "a-attr" => Style::new().italic().color(AnsiColors::Cyan),

        // Macros - bold cyan
        "a-m" | "a-macro" => Style::new().bold().color(AnsiColors::Cyan),

        // Labels - bold
        "a-l" | "a-label" => Style::new().bold(),

        // Namespace/module - yellow
        "a-ns" => Style::new().color(AnsiColors::Yellow),

        // Property - cyan
        "a-prop" => Style::new().color(AnsiColors::Cyan),

        // Parameter - italic
        "a-param" => Style::new().italic(),

        // Built-in - bold yellow
        "a-builtin" => Style::new().bold().color(AnsiColors::Yellow),

        // Error - bold red
        "a-err" | "a-error" => Style::new().bold().color(AnsiColors::Red),

        // Default - no styling
        _ => Style::new(),
    }
}

/// Install the arborium highlighter as miette's global highlighter.
///
/// This should be called once at the start of your program.
///
/// # Example
///
/// ```rust,ignore
/// fn main() {
///     miette_arborium::install_global().ok();
///     // ... rest of your program ...
/// }
/// ```
pub fn install_global() -> Result<(), miette::InstallError> {
    miette::set_hook(Box::new(|_| {
        Box::new(
            miette::MietteHandlerOpts::new()
                .with_syntax_highlighting(MietteHighlighter::new())
                .build(),
        )
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_detection() {
        assert_eq!(MietteHighlighter::detect_language("foo.rs"), Some("rust"));
        assert_eq!(
            MietteHighlighter::detect_language("/path/to/file.py"),
            Some("python")
        );
        assert_eq!(
            MietteHighlighter::detect_language("script.js"),
            Some("javascript")
        );
        assert_eq!(MietteHighlighter::detect_language("no_extension"), None);
    }

    #[test]
    fn test_style_for_tag() {
        // Just verify that all expected tags return styles without panicking
        let tags = ["a-k", "a-s", "a-c", "a-f", "a-t", "a-n", "a-o", "a-p"];
        for tag in tags {
            let _ = style_for_tag(tag);
        }
    }
}
