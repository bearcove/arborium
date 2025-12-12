//! High-level syntax highlighting API with automatic language injection support.
//!
//! This module provides a simple, batteries-included API for syntax highlighting
//! that automatically handles language injections (e.g., CSS in HTML `<style>` tags,
//! JavaScript in `<script>` tags).
//!
//! # Example
//!
//! ```rust,ignore
//! use arborium::highlighter::Highlighter;
//!
//! let mut highlighter = Highlighter::new();
//! let html = highlighter.highlight_to_html("svelte", r#"
//!     <script>
//!         let name = "world";
//!     </script>
//!     <h1>Hello {name}!</h1>
//!     <style>
//!         h1 { color: red; }
//!     </style>
//! "#)?;
//! ```

use std::io::{self, Write};

use arborium_highlight::{
    AnsiOptions, HighlightConfig, HighlightError as CoreError, SyncHighlighter,
};

use crate::provider::StaticProvider;

/// Error type for highlighting operations
#[derive(Debug)]
pub enum HighlightError {
    /// The requested language is not supported
    UnsupportedLanguage(String),
    /// Error compiling the grammar's queries
    QueryError(String),
    /// Error during highlighting
    HighlightError(String),
    /// IO error during rendering
    IoError(io::Error),
}

impl std::fmt::Display for HighlightError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HighlightError::UnsupportedLanguage(lang) => {
                write!(f, "Unsupported language: {}", lang)
            }
            HighlightError::QueryError(e) => write!(f, "Query error: {}", e),
            HighlightError::HighlightError(e) => write!(f, "Highlight error: {}", e),
            HighlightError::IoError(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl std::error::Error for HighlightError {}

impl From<io::Error> for HighlightError {
    fn from(e: io::Error) -> Self {
        HighlightError::IoError(e)
    }
}

impl From<CoreError> for HighlightError {
    fn from(e: CoreError) -> Self {
        match e {
            CoreError::UnsupportedLanguage(lang) => HighlightError::UnsupportedLanguage(lang),
            CoreError::ParseError(msg) => HighlightError::HighlightError(msg),
        }
    }
}

/// High-level syntax highlighter with automatic injection support.
///
/// This struct wraps `SyncHighlighter<StaticProvider>` and provides a simple
/// API for highlighting source code with automatic language injection handling.
pub struct Highlighter {
    inner: SyncHighlighter<StaticProvider>,
}

impl Default for Highlighter {
    fn default() -> Self {
        Self::new()
    }
}

impl Highlighter {
    /// Create a new highlighter with default configuration.
    pub fn new() -> Self {
        Self {
            inner: SyncHighlighter::new(StaticProvider::new()),
        }
    }

    /// Create a new highlighter with custom configuration.
    pub fn with_config(config: HighlightConfig) -> Self {
        Self {
            inner: SyncHighlighter::with_config(StaticProvider::new(), config),
        }
    }

    /// Highlight source code and return HTML string.
    ///
    /// This is the main entry point for highlighting. It automatically handles
    /// language injections (e.g., CSS/JS in HTML).
    pub fn highlight_to_html(
        &mut self,
        language: &str,
        source: &str,
    ) -> Result<String, HighlightError> {
        Ok(self.inner.highlight(language, source)?)
    }

    /// Highlight source code and write HTML to a writer.
    pub fn highlight_to_html_writer<W: Write>(
        &mut self,
        writer: &mut W,
        language: &str,
        source: &str,
    ) -> Result<(), HighlightError> {
        let html = self.inner.highlight(language, source)?;
        writer.write_all(html.as_bytes())?;
        Ok(())
    }

    /// Highlight source code and return ANSI-colored text.
    ///
    /// This uses the theme system to generate proper 24-bit color ANSI codes.
    pub fn highlight_to_ansi(
        &mut self,
        language: &str,
        source: &str,
        theme: &arborium_theme::Theme,
    ) -> Result<String, HighlightError> {
        Ok(self.inner.highlight_to_ansi(language, source, theme)?)
    }

    /// Highlight source code and return ANSI-colored text with explicit
    /// rendering options (e.g., base background and width-aware wrapping).
    pub fn highlight_to_ansi_with_options(
        &mut self,
        language: &str,
        source: &str,
        theme: &arborium_theme::Theme,
        options: &AnsiOptions,
    ) -> Result<String, HighlightError> {
        Ok(self
            .inner
            .highlight_to_ansi_with_options(language, source, theme, options)?)
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    #[cfg(feature = "lang-commonlisp")]
    fn test_commonlisp_highlighting() {
        let mut highlighter = Highlighter::new();
        let html = highlighter
            .highlight_to_html("commonlisp", "(defun hello () (print \"Hello\"))")
            .unwrap();
        // Should contain some highlighted elements
        assert!(html.contains("<a-"), "Should contain highlight tags");
    }

    #[test]
    #[cfg(feature = "lang-rust")]
    fn test_ansi_highlighting() {
        let mut highlighter = Highlighter::new();
        let source = r#"
 fn main() {
     let message = "Hello, world!";
     println!("{}", message);
     
     if let Some(count) = Some(42) {
         for i in 0..count {
             println!("Iteration: {}", i);
         }
     }
 }
 "#;

        let theme = arborium_theme::theme::builtin::catppuccin_mocha();
        let ansi_output = highlighter
            .highlight_to_ansi("rust", source, theme)
            .unwrap();

        // Should contain ANSI escape sequences
        assert!(
            ansi_output.contains("\x1b["),
            "Should contain ANSI escape sequences"
        );

        // Print colored output (use --nocapture to see)
        println!("\n=== ANSI colored Rust code ===");
        println!("{}", ansi_output);
        println!("=== End of colored output ===\n");
    }

    #[test]
    #[cfg(feature = "lang-rust")]
    fn test_ansi_highlighting_with_background_and_wrapping() {
        let mut highlighter = Highlighter::new();

        // Short source that fits in width
        let source_short = r#"fn long_function_name(argument: i32) {
    println!("value: {}", argument);
}"#;

        // Long source that will wrap
        let source_long = r#"fn very_long_function_name_that_will_definitely_wrap(first_argument: i32, second_argument: String) {
    println!("This is a very long string that should cause wrapping: {}", first_argument);
}"#;

        // Test with multiple themes
        let themes = vec![
            (
                "Catppuccin Mocha",
                arborium_theme::theme::builtin::catppuccin_mocha(),
            ),
            (
                "GitHub Light",
                arborium_theme::theme::builtin::github_light(),
            ),
            ("Monokai", arborium_theme::theme::builtin::monokai()),
        ];

        for (name, theme) in &themes {
            println!("\n=== {} (Rust, with bg+padding) ===", name);

            // With background and padding (w=2, h=1 since chars aren't square)
            let mut options = AnsiOptions::default();
            options.use_theme_base_style = true;
            options.width = Some(50);
            options.pad_to_width = true;
            options.padding_x = 2;
            options.padding_y = 1;

            let ansi_output = highlighter
                .highlight_to_ansi_with_options("rust", source_short, theme, &options)
                .unwrap();

            assert!(ansi_output.contains("\x1b["));
            assert!(ansi_output.contains(&theme.ansi_base_style()));
            println!("{}", ansi_output);

            // Without background/padding
            println!("\n=== {} (Rust, no bg/padding) ===", name);
            let mut options2 = AnsiOptions::default();
            options2.width = Some(50);
            options2.pad_to_width = true;

            let ansi_output2 = highlighter
                .highlight_to_ansi_with_options("rust", source_short, theme, &options2)
                .unwrap();
            println!("{}", ansi_output2);
        }

        // Test wrapping with long source
        println!("\n=== WRAPPING TEST (Catppuccin Mocha) ===");
        let theme = arborium_theme::theme::builtin::catppuccin_mocha();
        let mut options = AnsiOptions::default();
        options.use_theme_base_style = true;
        options.width = Some(60);
        options.pad_to_width = true;
        options.padding_x = 2;
        options.padding_y = 1;

        let ansi_output = highlighter
            .highlight_to_ansi_with_options("rust", source_long, theme, &options)
            .unwrap();
        println!("{}", ansi_output);

        // Test with defaults (terminal width, no bg/padding)
        println!("\n=== DEFAULTS TEST (terminal width auto-detected) ===");
        let options = AnsiOptions::default();
        println!("Default options:");
        println!("  width: {:?}", options.width);
        println!("  pad_to_width: {}", options.pad_to_width);
        println!("  use_theme_base_style: {}", options.use_theme_base_style);
        println!(
            "  margin_x: {}, margin_y: {}",
            options.margin_x, options.margin_y
        );
        println!(
            "  padding_x: {}, padding_y: {}",
            options.padding_x, options.padding_y
        );
        println!("  border: {}", options.border);
        println!("  tab_width: {}", options.tab_width);

        let ansi_output = highlighter
            .highlight_to_ansi_with_options("rust", source_short, theme, &options)
            .unwrap();
        println!("\nOutput with defaults:");
        println!("{}", ansi_output);

        // Test with border
        println!("\n=== BORDER TEST (Catppuccin Mocha) ===");
        let mut options = AnsiOptions::default();
        options.use_theme_base_style = true;
        options.width = Some(50);
        options.pad_to_width = true;
        options.padding_x = 2;
        options.padding_y = 1;
        options.border = true;

        let ansi_output = highlighter
            .highlight_to_ansi_with_options("rust", source_short, theme, &options)
            .unwrap();
        println!("{}", ansi_output);
    }
}
