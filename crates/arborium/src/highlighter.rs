//! High-level syntax highlighting API with thread-safe grammar sharing.
//!
//! This module provides highlighters that can be efficiently used across threads:
//!
//! - [`Highlighter`]: HTML output (custom elements or class-based spans)
//! - [`AnsiHighlighter`]: Terminal output with ANSI colors
//!
//! # Thread Safety
//!
//! Grammars are compiled once and shared via `Arc<GrammarStore>`. Each highlighter
//! has its own parse context (cheap to create). Use [`Highlighter::fork`] to create
//! a new highlighter that shares the grammar store.
//!
//! # Example
//!
//! ```rust,ignore
//! use arborium::Highlighter;
//! use rayon::prelude::*;
//!
//! // Create initial highlighter
//! let hl = Highlighter::new();
//!
//! // Parallel highlighting - each thread gets its own forked highlighter
//! let results: Vec<_> = code_blocks.par_iter().map(|code| {
//!     let mut hl = hl.fork();
//!     hl.highlight("rust", code)
//! }).collect();
//! ```

use std::io::Write;
use std::sync::Arc;

use arborium_highlight::tree_sitter::{CompiledGrammar, ParseContext};
use arborium_highlight::{AnsiOptions, Span, spans_to_ansi_with_options, spans_to_html};
use arborium_theme::Theme;

use crate::Config;
use crate::error::Error;
use crate::store::GrammarStore;

/// High-level syntax highlighter for HTML output.
///
/// This is the primary entry point for syntax highlighting. It produces HTML
/// output using custom elements (`<a-k>`, `<a-f>`, etc.) or traditional
/// `<span class="...">` elements depending on configuration.
///
/// # Thread Safety
///
/// The highlighter can be forked to create copies that share the grammar store
/// but have independent parse contexts. This enables efficient parallel highlighting.
///
/// ```rust,ignore
/// let hl = Highlighter::new();
///
/// // Fork for another thread
/// let hl2 = hl.fork();
/// std::thread::spawn(move || {
///     let mut hl = hl2;
///     hl.highlight("rust", code)
/// });
/// ```
pub struct Highlighter {
    store: Arc<GrammarStore>,
    ctx: Option<ParseContext>,
    config: Config,
}

impl Default for Highlighter {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for Highlighter {
    /// Clone creates a new highlighter sharing the grammar store.
    ///
    /// This is equivalent to [`fork`](Self::fork).
    fn clone(&self) -> Self {
        Self {
            store: self.store.clone(),
            ctx: None, // New context will be created on first use
            config: self.config.clone(),
        }
    }
}

impl Highlighter {
    /// Create a new highlighter with default configuration.
    ///
    /// Uses custom elements (`<a-k>`, `<a-f>`, etc.) for HTML output.
    pub fn new() -> Self {
        Self {
            store: Arc::new(GrammarStore::new()),
            ctx: None,
            config: Config::default(),
        }
    }

    /// Create a new highlighter with custom configuration.
    pub fn with_config(config: Config) -> Self {
        Self {
            store: Arc::new(GrammarStore::new()),
            ctx: None,
            config,
        }
    }

    /// Create a new highlighter with a shared grammar store.
    ///
    /// Use this when you want multiple highlighters to share compiled grammars.
    pub fn with_store(store: Arc<GrammarStore>) -> Self {
        Self {
            store,
            ctx: None,
            config: Config::default(),
        }
    }

    /// Create a new highlighter with a shared store and custom configuration.
    pub fn with_store_and_config(store: Arc<GrammarStore>, config: Config) -> Self {
        Self {
            store,
            ctx: None,
            config,
        }
    }

    /// Fork this highlighter, creating a new one that shares the grammar store.
    ///
    /// The forked highlighter has its own parse context, making it safe to use
    /// from another thread.
    pub fn fork(&self) -> Self {
        Self {
            store: self.store.clone(),
            ctx: None,
            config: self.config.clone(),
        }
    }

    /// Get the grammar store.
    ///
    /// Use this to create additional highlighters that share compiled grammars.
    pub fn store(&self) -> &Arc<GrammarStore> {
        &self.store
    }

    /// Highlight source code and return HTML string.
    ///
    /// This automatically handles language injections (e.g., CSS/JS in HTML,
    /// SQL in Python strings, etc.).
    pub fn highlight(&mut self, language: &str, source: &str) -> Result<String, Error> {
        let spans = self.highlight_spans(language, source)?;
        Ok(spans_to_html(source, spans, &self.config.html_format))
    }

    /// Highlight source code and write HTML directly to a writer.
    ///
    /// More efficient than [`highlight`](Self::highlight) when writing to a file or socket,
    /// as it avoids an intermediate string allocation.
    pub fn highlight_to_writer<W: Write>(
        &mut self,
        writer: &mut W,
        language: &str,
        source: &str,
    ) -> Result<(), Error> {
        let html = self.highlight(language, source)?;
        writer.write_all(html.as_bytes())?;
        Ok(())
    }

    /// Highlight and return raw spans (for custom rendering).
    pub fn highlight_spans(&mut self, language: &str, source: &str) -> Result<Vec<Span>, Error> {
        // Get the primary grammar
        let grammar = self
            .store
            .get(language)
            .ok_or_else(|| Error::UnsupportedLanguage {
                language: language.to_string(),
            })?;

        // Ensure we have a parse context
        self.ensure_context(&grammar)?;
        let ctx = self.ctx.as_mut().unwrap();

        // Set the language for this grammar
        ctx.set_language(grammar.language())
            .map_err(|_| Error::ParseError {
                language: language.to_string(),
                message: "Failed to set parser language".to_string(),
            })?;

        // Parse the primary language
        let result = grammar.parse(ctx, source);

        // Collect all spans (including from injections)
        let mut all_spans = result.spans;

        // Process injections recursively
        if self.config.max_injection_depth > 0 {
            self.process_injections(
                source,
                result.injections,
                0,
                self.config.max_injection_depth,
                &mut all_spans,
            )?;
        }

        Ok(all_spans)
    }

    /// Ensure we have a parse context, creating one if needed.
    fn ensure_context(&mut self, grammar: &CompiledGrammar) -> Result<(), Error> {
        if self.ctx.is_none() {
            self.ctx = Some(
                ParseContext::for_grammar(grammar).map_err(|e| Error::ParseError {
                    language: String::new(),
                    message: e.to_string(),
                })?,
            );
        }
        Ok(())
    }

    /// Process injections recursively.
    fn process_injections(
        &mut self,
        source: &str,
        injections: Vec<arborium_highlight::Injection>,
        base_offset: u32,
        remaining_depth: u32,
        all_spans: &mut Vec<Span>,
    ) -> Result<(), Error> {
        if remaining_depth == 0 {
            return Ok(());
        }

        for injection in injections {
            let start = injection.start as usize;
            let end = injection.end as usize;

            if start >= source.len() || end > source.len() || start >= end {
                continue;
            }

            let injected_source = &source[start..end];

            // Try to get grammar for injected language
            let Some(grammar) = self.store.get(&injection.language) else {
                continue;
            };

            // Set language for this grammar
            let ctx = self.ctx.as_mut().unwrap();
            if ctx.set_language(grammar.language()).is_err() {
                continue;
            }

            // Parse injected content
            let result = grammar.parse(ctx, injected_source);

            // Offset spans to document coordinates
            let offset = base_offset + injection.start;
            for mut span in result.spans {
                span.start += offset;
                span.end += offset;
                all_spans.push(span);
            }

            // Recurse into nested injections
            self.process_injections(
                injected_source,
                result.injections,
                offset,
                remaining_depth - 1,
                all_spans,
            )?;
        }

        Ok(())
    }
}

/// High-level syntax highlighter for ANSI terminal output.
///
/// This highlighter produces ANSI escape sequences for colored terminal output.
/// It owns a [`Theme`] which determines the colors used for each syntax element.
///
/// # Thread Safety
///
/// Like [`Highlighter`], this can be forked to share the grammar store.
pub struct AnsiHighlighter {
    inner: Highlighter,
    theme: Theme,
    options: AnsiOptions,
}

impl Clone for AnsiHighlighter {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            theme: self.theme.clone(),
            options: self.options.clone(),
        }
    }
}

impl AnsiHighlighter {
    /// Create a new ANSI highlighter with the given theme.
    pub fn new(theme: Theme) -> Self {
        Self {
            inner: Highlighter::new(),
            theme,
            options: AnsiOptions::default(),
        }
    }

    /// Create a new ANSI highlighter with custom configuration.
    pub fn with_config(theme: Theme, config: Config) -> Self {
        Self {
            inner: Highlighter::with_config(config),
            theme,
            options: AnsiOptions::default(),
        }
    }

    /// Create a new ANSI highlighter with custom configuration and rendering options.
    pub fn with_options(theme: Theme, config: Config, options: AnsiOptions) -> Self {
        Self {
            inner: Highlighter::with_config(config),
            theme,
            options,
        }
    }

    /// Create a new ANSI highlighter with a shared grammar store.
    pub fn with_store(store: Arc<GrammarStore>, theme: Theme) -> Self {
        Self {
            inner: Highlighter::with_store(store),
            theme,
            options: AnsiOptions::default(),
        }
    }

    /// Fork this highlighter, creating a new one that shares the grammar store.
    pub fn fork(&self) -> Self {
        Self {
            inner: self.inner.fork(),
            theme: self.theme.clone(),
            options: self.options.clone(),
        }
    }

    /// Get the grammar store.
    pub fn store(&self) -> &Arc<GrammarStore> {
        self.inner.store()
    }

    /// Get a reference to the current theme.
    pub fn theme(&self) -> &Theme {
        &self.theme
    }

    /// Set a new theme.
    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }

    /// Get a reference to the current ANSI rendering options.
    pub fn options(&self) -> &AnsiOptions {
        &self.options
    }

    /// Get a mutable reference to the ANSI rendering options.
    pub fn options_mut(&mut self) -> &mut AnsiOptions {
        &mut self.options
    }

    /// Highlight source code and return ANSI-colored string.
    ///
    /// This automatically handles language injections.
    pub fn highlight(&mut self, language: &str, source: &str) -> Result<String, Error> {
        let spans = self.inner.highlight_spans(language, source)?;
        Ok(spans_to_ansi_with_options(
            source,
            spans,
            &self.theme,
            &self.options,
        ))
    }

    /// Highlight source code and write ANSI output directly to a writer.
    pub fn highlight_to_writer<W: Write>(
        &mut self,
        writer: &mut W,
        language: &str,
        source: &str,
    ) -> Result<(), Error> {
        let ansi = self.highlight(language, source)?;
        writer.write_all(ansi.as_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    #[cfg(feature = "lang-rust")]
    fn test_highlighter_fork() {
        use crate::Highlighter;

        let hl = Highlighter::new();

        // Fork creates independent highlighters sharing the store
        let mut hl1 = hl.fork();
        let mut hl2 = hl.fork();

        // Both can highlight independently
        let html1 = hl1.highlight("rust", "fn main() {}").unwrap();
        let html2 = hl2.highlight("rust", "let x = 1;").unwrap();

        assert!(html1.contains("<a-"));
        assert!(html2.contains("<a-"));
    }

    #[test]
    #[cfg(feature = "lang-commonlisp")]
    fn test_commonlisp_highlighting() {
        use crate::Highlighter;

        let mut highlighter = Highlighter::new();
        let html = highlighter
            .highlight("commonlisp", "(defun hello () (print \"Hello\"))")
            .unwrap();
        assert!(html.contains("<a-"), "Should contain highlight tags");
    }

    #[test]
    #[cfg(feature = "lang-rust")]
    fn test_ansi_highlighting() {
        use arborium_theme::builtin;

        use crate::AnsiHighlighter;

        let theme = builtin::catppuccin_mocha().clone();
        let mut highlighter = AnsiHighlighter::new(theme);

        let source = r#"
fn main() {
    let message = "Hello, world!";
    println!("{}", message);
}
"#;

        let ansi_output = highlighter.highlight("rust", source).unwrap();

        println!("\n{ansi_output}");

        assert!(
            ansi_output.contains("\x1b["),
            "Should contain ANSI escape sequences"
        );
    }

    #[test]
    #[cfg(feature = "lang-rust")]
    fn test_ansi_with_options() {
        use arborium_highlight::AnsiOptions;
        use arborium_theme::builtin;

        use crate::AnsiHighlighter;

        let theme = builtin::catppuccin_mocha().clone();
        let config = crate::Config::default();
        let options = AnsiOptions {
            use_theme_base_style: true,
            width: Some(60),
            pad_to_width: true,
            padding_x: 2,
            padding_y: 1,
            border: true,
            ..Default::default()
        };

        let mut highlighter = AnsiHighlighter::with_options(theme, config, options);

        let source = r#"fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}"#;

        let ansi_output = highlighter.highlight("rust", source).unwrap();

        println!("\n{ansi_output}");

        assert!(ansi_output.contains("\x1b["));
    }

    #[test]
    #[cfg(feature = "lang-rust")]
    fn test_theme_switching() {
        use arborium_theme::builtin;

        use crate::AnsiHighlighter;

        let theme1 = builtin::catppuccin_mocha().clone();
        let mut highlighter = AnsiHighlighter::new(theme1);

        let source = "let x = 42;";
        let output1 = highlighter.highlight("rust", source).unwrap();

        // Switch theme
        highlighter.set_theme(builtin::github_light().clone());
        let output2 = highlighter.highlight("rust", source).unwrap();

        // Different themes should produce different output
        assert_ne!(output1, output2);
    }

    #[test]
    #[cfg(feature = "lang-rust")]
    fn test_shared_store() {
        // Create a store

        use std::sync::Arc;

        use crate::{GrammarStore, Highlighter};
        let store = Arc::new(GrammarStore::new());

        // Multiple highlighters sharing the store
        let mut hl1 = Highlighter::with_store(store.clone());
        let mut hl2 = Highlighter::with_store(store.clone());

        // Both use the same compiled grammar
        let _html1 = hl1.highlight("rust", "fn a() {}").unwrap();
        let _html2 = hl2.highlight("rust", "fn b() {}").unwrap();

        // Store should have the grammar cached
        assert!(store.get("rust").is_some());
    }

    #[test]
    #[cfg(feature = "lang-rust")]
    fn test_multithreaded_highlighting() {
        use std::thread;

        use crate::Highlighter;

        // Create a highlighter and share its store across threads
        let hl = Highlighter::new();
        let store = hl.store().clone();

        // Spawn multiple threads that highlight concurrently
        let handles: Vec<_> = (0..4)
            .map(|i| {
                let store = store.clone();
                thread::spawn(move || {
                    let mut hl = Highlighter::with_store(store);
                    let code = format!("fn thread{}() {{ let x = {}; }}", i, i * 10);
                    let html = hl.highlight("rust", &code).unwrap();
                    assert!(
                        html.contains("<a-"),
                        "Thread {} should produce highlighted output",
                        i
                    );
                    html
                })
            })
            .collect();

        // Wait for all threads and collect results
        let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

        // All threads should have produced valid output
        assert_eq!(results.len(), 4);
        for (i, html) in results.iter().enumerate() {
            assert!(
                html.contains(&format!("thread{}", i)),
                "Output should contain thread-specific content"
            );
        }
    }
}
