//! Unified syntax highlighting for arborium.
//!
//! This crate provides the core highlighting engine that works with both:
//! - **Statically linked Rust grammars**: For CLI tools and servers
//! - **Dynamically loaded WASM plugins**: For browser contexts
//!
//! # Architecture
//!
//! The highlighting system is built around two key traits:
//!
//! - [`Grammar`]: What a grammar can do - parse text and return spans
//! - [`GrammarProvider`]: How grammars are obtained - this is where sync vs async differs
//!
//! The core highlighting logic is written once as async code in [`HighlighterCore`].
//! Two wrappers provide the sync and async APIs:
//!
//! - [`SyncHighlighter`]: Polls the async future once, panics if it yields
//! - [`AsyncHighlighter`]: Actually awaits provider calls
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use arborium_highlight::{SyncHighlighter, Grammar, GrammarProvider, ParseResult, Span};
//! use arborium_highlight::{HighlightConfig, HtmlFormat};
//!
//! // Define your grammar (implements Grammar trait)
//! struct MyGrammar { /* ... */ }
//! impl Grammar for MyGrammar {
//!     fn parse(&mut self, text: &str) -> ParseResult {
//!         // Parse and return spans + injections
//!         ParseResult::default()
//!     }
//! }
//!
//! // Define your provider (implements GrammarProvider trait)
//! struct MyProvider { /* ... */ }
//! impl GrammarProvider for MyProvider {
//!     type Grammar = MyGrammar;
//!     async fn get(&mut self, language: &str) -> Option<&mut Self::Grammar> {
//!         // Return grammar for language
//!         None
//!     }
//! }
//!
//! // Use with default configuration (custom elements: <a-k>, <a-f>, etc.)
//! let mut highlighter = SyncHighlighter::new(MyProvider { /* ... */ });
//! let html = highlighter.highlight("rust", "fn main() {}");
//! // Output: <a-k>fn</a-k> <a-f>main</a-f>() {}
//!
//! // Or use class-based output for compatibility with existing CSS
//! let config = HighlightConfig {
//!     html_format: HtmlFormat::ClassNames,
//!     ..Default::default()
//! };
//! let mut highlighter = SyncHighlighter::with_config(MyProvider { /* ... */ }, config);
//! let html = highlighter.highlight("rust", "fn main() {}");
//! // Output: <span class="keyword">fn</span> <span class="function">main</span>() {}
//! ```
//!
//! # HTML Output Formats
//!
//! Arborium supports multiple HTML output formats via [`HtmlFormat`]:
//!
//! - **`CustomElements`** (default): Compact custom elements like `<a-k>`, `<a-f>`, etc.
//! - **`CustomElementsWithPrefix(prefix)`**: Custom elements with your prefix, e.g., `<code-k>`
//! - **`ClassNames`**: Traditional `<span class="keyword">` for compatibility
//! - **`ClassNamesWithPrefix(prefix)`**: Namespaced classes like `<span class="arb-keyword">`
//!
//! See [`HtmlFormat`] for examples and use cases.

mod render;
mod types;

#[cfg(feature = "tree-sitter")]
pub mod tree_sitter;

pub use render::{
    AnsiOptions, html_escape, spans_to_ansi, spans_to_ansi_with_options, spans_to_html,
    write_spans_as_ansi, write_spans_as_html,
};
pub use types::{HighlightError, Injection, ParseResult, Span};

#[cfg(feature = "tree-sitter")]
pub use tree_sitter::{TreeSitterGrammar, TreeSitterGrammarConfig, TreeSitterGrammarError};

use std::future::Future;
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

/// A grammar that can parse text and produce highlight spans.
///
/// This is implemented by:
/// - Tree-sitter based parsers (for Rust)
/// - WASM plugin wrappers (for browser)
/// - Mock implementations (for testing)
///
/// # Implementation Notes
///
/// Parsing is always synchronous. The async part of highlighting is *getting* the grammar,
/// not using it. This is because tree-sitter parsing is inherently synchronous.
pub trait Grammar {
    /// Parse text and return spans + injection points.
    ///
    /// This is always synchronous - the async part is *getting* the grammar,
    /// not using it.
    fn parse(&mut self, text: &str) -> ParseResult;
}

/// Provides grammars for languages.
///
/// This trait abstracts over how grammars are obtained:
///
/// - **Static (Rust)**: Grammars are statically linked. `get()` returns
///   immediately without awaiting.
///
/// - **Dynamic (WASM)**: Grammars are loaded as WASM plugins. `get()` may
///   need to fetch and instantiate a plugin, which is async.
///
/// # Implementation Notes
///
/// For sync contexts (Rust CLI tools, servers), implement `get()` to return
/// immediately. The `SyncHighlighter` wrapper will panic if `get()` yields.
///
/// For async contexts (WASM/browser), `get()` can await plugin loading.
/// Use `AsyncHighlighter` wrapper.
pub trait GrammarProvider {
    /// The grammar type this provider returns.
    type Grammar: Grammar;

    /// Get a grammar for a language.
    ///
    /// Returns `None` if the language is not supported.
    ///
    /// # Sync vs Async
    ///
    /// This is an async method, but for sync providers (static Rust grammars),
    /// it should return `Ready` immediately without yielding. The caller
    /// (SyncHighlighter) will poll once and panic if it gets `Pending`.
    ///
    /// # Send Bound
    ///
    /// On native targets, the future must be `Send` for compatibility with
    /// async runtimes. On WASM, `Send` is not required (single-threaded).
    #[cfg(not(target_arch = "wasm32"))]
    fn get(&mut self, language: &str) -> impl Future<Output = Option<&mut Self::Grammar>> + Send;

    /// Get a grammar for a language (WASM version without Send bound).
    #[cfg(target_arch = "wasm32")]
    fn get(&mut self, language: &str) -> impl Future<Output = Option<&mut Self::Grammar>>;
}

/// HTML output format for syntax highlighting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HtmlFormat {
    /// Custom elements with default prefix: `<a-k>`, `<a-f>`, etc. (default)
    ///
    /// This is the most compact format and leverages custom HTML elements.
    ///
    /// # Example
    /// ```html
    /// <a-k>fn</a-k> <a-f>main</a-f>()
    /// ```
    CustomElements,

    /// Custom elements with custom prefix: `<prefix-k>`, `<prefix-f>`, etc.
    ///
    /// Useful for branding or avoiding conflicts with other custom elements.
    ///
    /// # Example
    /// ```html
    /// <!-- With prefix "code" -->
    /// <code-k>fn</code-k> <code-f>main</code-f>()
    /// ```
    CustomElementsWithPrefix(String),

    /// Traditional class-based spans: `<span class="keyword">`, etc.
    ///
    /// Compatible with existing tooling that expects class-based markup.
    ///
    /// # Example
    /// ```html
    /// <span class="keyword">fn</span> <span class="function">main</span>()
    /// ```
    ClassNames,

    /// Class-based spans with custom prefix: `<span class="prefix-keyword">`, etc.
    ///
    /// Useful for namespacing CSS classes.
    ///
    /// # Example
    /// ```html
    /// <!-- With prefix "arb" -->
    /// <span class="arb-keyword">fn</span> <span class="arb-function">main</span>()
    /// ```
    ClassNamesWithPrefix(String),
}

impl Default for HtmlFormat {
    fn default() -> Self {
        Self::CustomElements
    }
}

/// Configuration for highlighting.
#[derive(Debug, Clone)]
pub struct HighlightConfig {
    /// Maximum depth for processing language injections.
    ///
    /// - `0`: No injections (just primary language)
    /// - `3`: Default, handles most cases
    /// - Higher: For deeply nested content
    pub max_injection_depth: u32,

    /// HTML output format (custom elements vs class-based spans).
    pub html_format: HtmlFormat,
}

impl Default for HighlightConfig {
    fn default() -> Self {
        Self {
            max_injection_depth: 3,
            html_format: HtmlFormat::default(),
        }
    }
}

/// Internal async implementation - handles all the hard work.
///
/// The core logic is written once as async, then wrapped by `SyncHighlighter`
/// (which polls once and panics if it yields) and `AsyncHighlighter` (which
/// actually awaits).
struct HighlighterCore<P: GrammarProvider> {
    provider: P,
    config: HighlightConfig,
}

impl<P: GrammarProvider> HighlighterCore<P> {
    fn new(provider: P) -> Self {
        Self {
            provider,
            config: HighlightConfig::default(),
        }
    }

    fn with_config(provider: P, config: HighlightConfig) -> Self {
        Self { provider, config }
    }

    /// Highlight and return raw spans for the full document,
    /// including any recursively processed injections.
    async fn highlight_spans(
        &mut self,
        language: &str,
        source: &str,
    ) -> Result<Vec<Span>, HighlightError> {
        // 1. Get the primary grammar
        let grammar = self
            .provider
            .get(language)
            .await
            .ok_or_else(|| HighlightError::UnsupportedLanguage(language.into()))?;

        // 2. Parse the primary language
        let result = grammar.parse(source);

        // 3. Collect all spans (including from injections)
        let mut all_spans = result.spans;

        // 4. Process injections recursively
        if self.config.max_injection_depth > 0 {
            self.process_injections(
                source,
                result.injections,
                0,
                self.config.max_injection_depth,
                &mut all_spans,
            )
            .await;
        }

        Ok(all_spans)
    }

    /// The main highlight function - written once, used by both wrappers.
    async fn highlight(&mut self, language: &str, source: &str) -> Result<String, HighlightError> {
        let spans = self.highlight_spans(language, source).await?;
        Ok(spans_to_html(source, spans, &self.config.html_format))
    }

    /// Process injections recursively.
    async fn process_injections(
        &mut self,
        source: &str,
        injections: Vec<Injection>,
        base_offset: u32,
        remaining_depth: u32,
        all_spans: &mut Vec<Span>,
    ) {
        if remaining_depth == 0 {
            return;
        }

        for injection in injections {
            let start = injection.start as usize;
            let end = injection.end as usize;

            if end <= source.len() && start < end {
                // Try to get grammar for injected language
                if let Some(inj_grammar) = self.provider.get(&injection.language).await {
                    let injected_text = &source[start..end];
                    let result = inj_grammar.parse(injected_text);

                    // Adjust offsets and add spans
                    let adjusted_spans: Vec<Span> = result
                        .spans
                        .into_iter()
                        .map(|mut s| {
                            s.start += base_offset + injection.start;
                            s.end += base_offset + injection.start;
                            s
                        })
                        .collect();
                    all_spans.extend(adjusted_spans);

                    // Recurse into nested injections
                    if !result.injections.is_empty() {
                        // Box the recursive call to avoid infinite type size
                        Box::pin(self.process_injections(
                            injected_text,
                            result.injections,
                            base_offset + injection.start,
                            remaining_depth - 1,
                            all_spans,
                        ))
                        .await;
                    }
                }
                // If grammar not available, skip this injection silently
            }
        }
    }
}

/// Synchronous highlighter for Rust contexts.
///
/// Uses a sync provider where `get()` returns immediately.
/// Panics if the provider ever yields (returns Pending).
///
/// # Example
///
/// ```rust,ignore
/// use arborium_highlight::{SyncHighlighter, StaticProvider};
///
/// let mut highlighter = SyncHighlighter::new(StaticProvider::new());
/// let html = highlighter.highlight("rust", "fn main() {}")?;
/// ```
pub struct SyncHighlighter<P: GrammarProvider> {
    core: HighlighterCore<P>,
}

impl<P: GrammarProvider> SyncHighlighter<P> {
    /// Create a new synchronous highlighter with default configuration.
    pub fn new(provider: P) -> Self {
        Self {
            core: HighlighterCore::new(provider),
        }
    }

    /// Create a new synchronous highlighter with custom configuration.
    pub fn with_config(provider: P, config: HighlightConfig) -> Self {
        Self {
            core: HighlighterCore::with_config(provider, config),
        }
    }

    /// Get a mutable reference to the underlying provider.
    pub fn provider_mut(&mut self) -> &mut P {
        &mut self.core.provider
    }

    /// Highlight source code synchronously and return HTML.
    ///
    /// # Panics
    ///
    /// Panics if the provider's `get()` method yields (returns Pending).
    /// This indicates a bug - sync providers should never yield.
    pub fn highlight(&mut self, language: &str, source: &str) -> Result<String, HighlightError> {
        let future = self.core.highlight(language, source);

        // Pin the future on the stack
        let mut future = std::pin::pin!(future);

        // Create a no-op waker (we're not actually async)
        let waker = noop_waker();
        let mut cx = Context::from_waker(&waker);

        // Poll once - sync providers complete immediately
        match future.as_mut().poll(&mut cx) {
            Poll::Ready(result) => result,
            Poll::Pending => {
                panic!(
                    "SyncHighlighter: provider yielded. Use AsyncHighlighter for async providers."
                )
            }
        }
    }

    /// Highlight source code synchronously and return ANSI-colored text
    /// using the provided theme.
    ///
    /// This uses the same span computation as HTML output but renders
    /// with ANSI escape sequences.
    pub fn highlight_to_ansi(
        &mut self,
        language: &str,
        source: &str,
        theme: &arborium_theme::Theme,
    ) -> Result<String, HighlightError> {
        self.highlight_to_ansi_with_options(language, source, theme, &AnsiOptions::default())
    }

    /// Highlight source code synchronously and return ANSI-colored text
    /// using the provided theme and explicit ANSI rendering options.
    pub fn highlight_to_ansi_with_options(
        &mut self,
        language: &str,
        source: &str,
        theme: &arborium_theme::Theme,
        options: &AnsiOptions,
    ) -> Result<String, HighlightError> {
        let future = self.core.highlight_spans(language, source);

        let mut future = std::pin::pin!(future);
        let waker = noop_waker();
        let mut cx = Context::from_waker(&waker);

        match future.as_mut().poll(&mut cx) {
            Poll::Ready(Ok(spans)) => Ok(spans_to_ansi_with_options(source, spans, theme, options)),
            Poll::Ready(Err(e)) => Err(e),
            Poll::Pending => {
                panic!(
                    "SyncHighlighter: provider yielded. Use AsyncHighlighter for async providers."
                )
            }
        }
    }
}

/// Asynchronous highlighter for WASM/browser contexts.
///
/// Uses an async provider where `get()` may need to load plugins.
///
/// # Example
///
/// ```rust,ignore
/// use arborium_highlight::{AsyncHighlighter, WasmPluginProvider};
///
/// let mut highlighter = AsyncHighlighter::new(WasmPluginProvider::new());
/// let html = highlighter.highlight("rust", "fn main() {}").await?;
/// ```
pub struct AsyncHighlighter<P: GrammarProvider> {
    core: HighlighterCore<P>,
}

impl<P: GrammarProvider> AsyncHighlighter<P> {
    /// Create a new asynchronous highlighter with default configuration.
    pub fn new(provider: P) -> Self {
        Self {
            core: HighlighterCore::new(provider),
        }
    }

    /// Create a new asynchronous highlighter with custom configuration.
    pub fn with_config(provider: P, config: HighlightConfig) -> Self {
        Self {
            core: HighlighterCore::with_config(provider, config),
        }
    }

    /// Get a mutable reference to the underlying provider.
    pub fn provider_mut(&mut self) -> &mut P {
        &mut self.core.provider
    }

    /// Highlight source code asynchronously.
    pub async fn highlight(
        &mut self,
        language: &str,
        source: &str,
    ) -> Result<String, HighlightError> {
        self.core.highlight(language, source).await
    }
}

/// Create a no-op waker for sync polling.
fn noop_waker() -> Waker {
    const VTABLE: RawWakerVTable = RawWakerVTable::new(
        |_| RAW_WAKER, // clone
        |_| {},        // wake
        |_| {},        // wake_by_ref
        |_| {},        // drop
    );
    const RAW_WAKER: RawWaker = RawWaker::new(std::ptr::null(), &VTABLE);

    unsafe { Waker::from_raw(RAW_WAKER) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// Mock provider for testing - sync, returns immediately
    struct MockProvider {
        grammars: HashMap<&'static str, MockGrammar>,
    }

    impl GrammarProvider for MockProvider {
        type Grammar = MockGrammar;

        #[cfg(not(target_arch = "wasm32"))]
        async fn get(&mut self, language: &str) -> Option<&mut Self::Grammar> {
            self.grammars.get_mut(language)
        }

        #[cfg(target_arch = "wasm32")]
        async fn get(&mut self, language: &str) -> Option<&mut Self::Grammar> {
            self.grammars.get_mut(language)
        }
    }

    struct MockGrammar {
        result: ParseResult,
    }

    impl Grammar for MockGrammar {
        fn parse(&mut self, _text: &str) -> ParseResult {
            self.result.clone()
        }
    }

    #[test]
    fn test_basic_highlighting() {
        let provider = MockProvider {
            grammars: [(
                "test",
                MockGrammar {
                    result: ParseResult {
                        spans: vec![Span {
                            start: 0,
                            end: 2,
                            capture: "keyword".into(),
                        }],
                        injections: vec![],
                    },
                },
            )]
            .into(),
        };

        let mut highlighter = SyncHighlighter::new(provider);
        let html = highlighter.highlight("test", "fn").unwrap();
        assert_eq!(html, "<a-k>fn</a-k>");
    }

    #[test]
    fn test_injection() {
        let provider = MockProvider {
            grammars: [
                (
                    "outer",
                    MockGrammar {
                        result: ParseResult {
                            spans: vec![],
                            injections: vec![Injection {
                                start: 0,
                                end: 5,
                                language: "inner".into(),
                                include_children: false,
                            }],
                        },
                    },
                ),
                (
                    "inner",
                    MockGrammar {
                        result: ParseResult {
                            spans: vec![Span {
                                start: 0,
                                end: 5,
                                capture: "string".into(),
                            }],
                            injections: vec![],
                        },
                    },
                ),
            ]
            .into(),
        };

        let mut highlighter = SyncHighlighter::new(provider);
        let html = highlighter.highlight("outer", "hello").unwrap();
        assert_eq!(html, "<a-s>hello</a-s>");
    }

    #[test]
    fn test_unsupported_language() {
        let provider = MockProvider {
            grammars: HashMap::new(),
        };

        let mut highlighter = SyncHighlighter::new(provider);
        let result = highlighter.highlight("unknown", "code");
        assert!(matches!(
            result,
            Err(HighlightError::UnsupportedLanguage(_))
        ));
    }

    #[test]
    fn test_reuse_with_shorter_text() {
        // Regression test: reusing a highlighter with a shorter string
        // after a longer string should not panic with slice bounds errors.
        // This tests that we don't incorrectly use cached tree state.
        let provider = MockProvider {
            grammars: [(
                "test",
                MockGrammar {
                    result: ParseResult {
                        spans: vec![Span {
                            start: 0,
                            end: 2,
                            capture: "keyword".into(),
                        }],
                        injections: vec![],
                    },
                },
            )]
            .into(),
        };

        let mut highlighter = SyncHighlighter::new(provider);

        // First: longer string
        let _ = highlighter.highlight("test", "longer string here");

        // Second: shorter string - should not panic
        let _ = highlighter.highlight("test", "short");
    }

    #[test]
    fn test_span_coalescing() {
        let spans = vec![
            Span {
                start: 0,
                end: 3,
                capture: "keyword".into(),
            },
            Span {
                start: 3,
                end: 7,
                capture: "keyword.function".into(),
            },
        ];
        let html = spans_to_html("keyword", spans, &HtmlFormat::default());
        assert_eq!(html, "<a-k>keyword</a-k>");
    }
}
