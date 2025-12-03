//! HTML rendering for syntax-highlighted code.
//!
//! This module provides utilities to render tree-sitter highlighted code
//! as HTML using custom web elements for compact, semantic markup.
//!
//! # Custom Web Elements
//!
//! Instead of CSS classes, this module uses custom HTML elements for highlighting.
//! All elements use the `a-` prefix (for Arborium), followed by a short category code:
//!
//! - `<a-k>` = keyword
//! - `<a-t>` = type
//! - `<a-f>` = function
//! - `<a-s>` = string
//! - `<a-c>` = comment
//! - etc.
//!
//! This approach:
//! - Produces very compact output
//! - Works without JavaScript (custom elements are valid HTML)
//! - Easy to style with CSS: `a-k { color: purple; }`

use crate::highlights;
use crate::tree_sitter_highlight::{
    Highlight, HighlightConfiguration, HighlightEvent, Highlighter,
};
use std::io::{self, Write};

/// Get the custom element tag suffix for a highlight index.
/// Returns None for indices without tags (like "none" or "nospell") or out-of-range indices.
#[inline]
pub fn highlight_tag(highlight_index: usize) -> Option<&'static str> {
    highlights::tag(highlight_index)
}

/// Render highlighted code to HTML using custom web elements.
///
/// This function takes source code and a configured `HighlightConfiguration`,
/// and writes HTML output with custom elements like `<a-k>` for highlighted regions.
///
/// # Arguments
///
/// * `w` - Writer to output HTML to
/// * `highlighter` - A mutable `Highlighter` instance (reuse for performance)
/// * `config` - Configured `HighlightConfiguration` for the source language
/// * `source` - Source code to highlight
/// * `injection_callback` - Optional callback for language injection (e.g., in markdown code blocks)
///
/// # Example
///
/// ```rust,ignore
/// use arborium::{html, HIGHLIGHT_NAMES};
/// use tree_sitter_highlight::{Highlighter, HighlightConfiguration};
///
/// let mut config = /* create config */;
/// config.configure(&HIGHLIGHT_NAMES.iter().map(|s| s.to_string()).collect::<Vec<_>>());
///
/// let mut highlighter = Highlighter::new();
/// let mut output = Vec::new();
/// html::render(&mut output, &mut highlighter, &config, "fn main() {}", |_| None).unwrap();
/// // Output: <a-k>fn</a-k> <a-f>main</a-f>() {}
/// ```
pub fn render<'a>(
    w: &mut dyn Write,
    highlighter: &'a mut Highlighter,
    config: &'a HighlightConfiguration,
    source: &'a str,
    injection_callback: impl FnMut(&str) -> Option<&'a HighlightConfiguration> + 'a,
) -> io::Result<()> {
    let highlights = highlighter
        .highlight(config, source.as_bytes(), None, injection_callback)
        .map_err(|e| io::Error::other(e.to_string()))?;

    // Stack to track open tags for proper closing
    let mut tag_stack: Vec<&'static str> = Vec::new();

    for event in highlights {
        let event = event.map_err(|e| io::Error::other(e.to_string()))?;
        match event {
            HighlightEvent::Source { start, end } => {
                write_escaped(w, &source[start..end])?;
            }
            HighlightEvent::HighlightStart(Highlight(i)) => {
                if let Some(tag) = highlight_tag(i) {
                    write!(w, "<a-{tag}>")?;
                    tag_stack.push(tag);
                }
            }
            HighlightEvent::HighlightEnd => {
                if let Some(tag) = tag_stack.pop() {
                    write!(w, "</a-{tag}>")?;
                }
            }
        }
    }

    Ok(())
}

/// Render highlighted code wrapped in a code block structure.
///
/// This outputs a complete code block with:
/// - A `<figure>` wrapper with appropriate ARIA attributes
/// - A `<code>` element containing the highlighted source
///
/// # Arguments
///
/// * `w` - Writer to output HTML to
/// * `highlighter` - A mutable `Highlighter` instance (reuse for performance)
/// * `config` - Configured `HighlightConfiguration` for the source language
/// * `source` - Source code to highlight
/// * `lang` - Language tag for the code block (e.g., "rust", "python")
/// * `injection_callback` - Optional callback for language injection
pub fn render_block<'a>(
    w: &mut dyn Write,
    highlighter: &'a mut Highlighter,
    config: &'a HighlightConfiguration,
    source: &'a str,
    lang: &str,
    injection_callback: impl FnMut(&str) -> Option<&'a HighlightConfiguration> + 'a,
) -> io::Result<()> {
    write!(
        w,
        r#"<figure role="region" aria-label="{lang} code block" class="code-block" translate="no" data-lang="{lang}"><code class="scroll-wrapper">"#
    )?;
    render(w, highlighter, config, source, injection_callback)?;
    write!(w, "</code></figure>")?;
    Ok(())
}

/// Write HTML-escaped text.
///
/// Escapes `<`, `>`, and `&` characters for safe HTML output.
pub fn write_escaped(w: &mut dyn Write, input: &str) -> io::Result<()> {
    let mut start: Option<usize> = None;

    for (i, c) in input.char_indices() {
        match c {
            '<' | '>' | '&' => {
                if let Some(start_idx) = start.take() {
                    write!(w, "{}", &input[start_idx..i])?;
                }
                match c {
                    '<' => write!(w, "&lt;")?,
                    '>' => write!(w, "&gt;")?,
                    '&' => write!(w, "&amp;")?,
                    _ => {}
                };
            }
            _ => {
                if start.is_none() {
                    start = Some(i)
                }
            }
        }
    }
    if let Some(start_idx) = start.take() {
        write!(w, "{}", &input[start_idx..])?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_escaped() {
        let mut out = Vec::new();
        write_escaped(&mut out, "The Vec<u8> type").unwrap();
        assert_eq!(std::str::from_utf8(&out).unwrap(), "The Vec&lt;u8&gt; type");

        out.clear();
        write_escaped(&mut out, "ParseResult<&str> Or Result<Vec<_>> && false").unwrap();
        assert_eq!(
            std::str::from_utf8(&out).unwrap(),
            "ParseResult&lt;&amp;str&gt; Or Result&lt;Vec&lt;_&gt;&gt; &amp;&amp; false"
        );
    }

    #[test]
    fn test_write_escaped_empty() {
        let mut out = Vec::new();
        write_escaped(&mut out, "").unwrap();
        assert_eq!(std::str::from_utf8(&out).unwrap(), "");
    }

    #[test]
    fn test_write_escaped_no_special() {
        let mut out = Vec::new();
        write_escaped(&mut out, "hello world").unwrap();
        assert_eq!(std::str::from_utf8(&out).unwrap(), "hello world");
    }
}
