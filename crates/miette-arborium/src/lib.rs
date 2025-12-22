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

use arborium::{Highlighter, ThemedSpan, spans_to_themed};
use arborium_theme::{Style as ThemeStyle, Theme};
use miette::highlighters::Highlighter as MietteHighlighterTrait;
use owo_colors::Style;

/// A syntax highlighter for miette that uses arborium for tree-sitter based highlighting.
///
/// This highlighter can be installed globally using [`install_global`] or used directly
/// by setting it on miette's `GraphicalReportHandler`.
pub struct MietteHighlighter {
    inner: RwLock<Highlighter>,
    theme: Theme,
}

impl MietteHighlighter {
    /// Create a new miette highlighter with the default theme.
    pub fn new() -> Self {
        Self::with_theme(arborium_theme::builtin::catppuccin_mocha().clone())
    }

    /// Create a new miette highlighter with a custom theme.
    pub fn with_theme(theme: Theme) -> Self {
        Self {
            inner: RwLock::new(Highlighter::new()),
            theme,
        }
    }

    /// Returns whether a language is supported by this highlighter.
    pub fn is_supported(&self, language: &str) -> bool {
        // Check if arborium has this language available
        let mut inner = self.inner.write().unwrap();
        inner.highlight_spans(language, "").is_ok()
    }

    /// Detect language from a source name (file path or extension).
    ///
    /// This delegates to [`arborium::detect_language`] which is generated from
    /// the grammar registry, ensuring extensions stay in sync with supported languages.
    pub fn detect_language(source_name: &str) -> Option<&'static str> {
        arborium::detect_language(source_name)
    }

    /// Get a reference to the current theme.
    pub fn theme(&self) -> &Theme {
        &self.theme
    }

    /// Set a new theme.
    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
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

        // Get the full source text
        let source_text = std::str::from_utf8(source.data()).unwrap_or("").to_string();

        // Highlight the entire source once to get themed spans
        let themed_spans = if let Some(lang) = language {
            let mut inner = self.inner.write().unwrap();
            if let Ok(spans) = inner.highlight_spans(lang, &source_text) {
                spans_to_themed(spans)
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        Box::new(MietteHighlighterState {
            highlighter: self,
            themed_spans,
            line_start: 0,
        })
    }
}

struct MietteHighlighterState<'h> {
    highlighter: &'h MietteHighlighter,
    themed_spans: Vec<ThemedSpan>,
    line_start: usize,
}

impl miette::highlighters::HighlighterState for MietteHighlighterState<'_> {
    fn highlight_line<'s>(&mut self, line: &'s str) -> Vec<owo_colors::Styled<&'s str>> {
        // Handle empty lines
        if line.is_empty() {
            self.line_start += 1;
            return vec![Style::new().style(line)];
        }

        // If no themed spans, return unhighlighted
        if self.themed_spans.is_empty() {
            self.line_start += line.len() + 1;
            return vec![Style::new().style(line)];
        }

        // Find where this line ends in the full source
        let line_end = self.line_start + line.len();

        // Collect spans that overlap with this line
        let mut line_spans: Vec<&ThemedSpan> = self
            .themed_spans
            .iter()
            .filter(|span| {
                let span_start = span.start as usize;
                let span_end = span.end as usize;
                // Span overlaps if it starts before line ends and ends after line starts
                span_start < line_end && span_end > self.line_start
            })
            .collect();

        // Sort by start position
        line_spans.sort_by_key(|span| span.start);

        // Build styled spans for this line
        let mut result = Vec::new();
        let mut current_pos = 0;

        for span in line_spans {
            let span_start = span.start as usize;
            let span_end = span.end as usize;

            // Calculate positions relative to this line
            let rel_start = span_start.saturating_sub(self.line_start);
            let rel_end = (span_end - self.line_start).min(line.len());

            // Add unhighlighted text before this span
            if current_pos < rel_start && rel_start <= line.len() {
                result.push(Style::new().style(&line[current_pos..rel_start]));
            }

            // Add the highlighted span
            if rel_start < rel_end && rel_end <= line.len() {
                // Get style from theme
                let style =
                    if let Some(theme_style) = self.highlighter.theme.style(span.theme_index) {
                        convert_theme_style_to_owo(theme_style)
                    } else {
                        Style::new()
                    };

                result.push(style.style(&line[rel_start..rel_end]));
                current_pos = rel_end;
            }
        }

        // Add any remaining unhighlighted text
        if current_pos < line.len() {
            result.push(Style::new().style(&line[current_pos..]));
        }

        // Update line_start for next line (account for newline character)
        self.line_start = line_end + 1;

        // Return unhighlighted if we didn't produce any spans
        if result.is_empty() {
            vec![Style::new().style(line)]
        } else {
            result
        }
    }
}

/// Convert arborium's `ThemeStyle` to `owo_colors::Style`.
fn convert_theme_style_to_owo(theme_style: &ThemeStyle) -> Style {
    let mut style = Style::new();

    // Apply foreground color if present
    if let Some(fg) = theme_style.fg {
        style = style.truecolor(fg.r, fg.g, fg.b);
    }

    // Apply background color if present
    if let Some(bg) = theme_style.bg {
        style = style.on_truecolor(bg.r, bg.g, bg.b);
    }

    // Apply modifiers
    if theme_style.modifiers.bold {
        style = style.bold();
    }
    if theme_style.modifiers.italic {
        style = style.italic();
    }
    if theme_style.modifiers.underline {
        style = style.underline();
    }
    if theme_style.modifiers.strikethrough {
        style = style.strikethrough();
    }

    style
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

/// Install a custom themed highlighter as miette's global highlighter.
///
/// # Example
///
/// ```rust,ignore
/// fn main() {
///     let theme = arborium_theme::builtin::github_light().clone();
///     miette_arborium::install_global_with_theme(theme).ok();
///     // ... rest of your program ...
/// }
/// ```
pub fn install_global_with_theme(theme: Theme) -> Result<(), miette::InstallError> {
    miette::set_hook(Box::new(move |_| {
        Box::new(
            miette::MietteHandlerOpts::new()
                .with_syntax_highlighting(MietteHighlighter::with_theme(theme.clone()))
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
    fn test_theme_style_conversion() {
        use arborium_theme::Color;

        let theme_style = ThemeStyle::new().fg(Color::new(255, 0, 0)).bold().italic();

        let owo_style = convert_theme_style_to_owo(&theme_style);

        // We can't directly test the style, but we can verify it doesn't panic
        let _styled = owo_style.style("test");
    }
}
