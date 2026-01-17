//! Arborium — High-performance syntax highlighting
//!
//! Arborium provides batteries-included syntax highlighting powered by tree-sitter.
//! It supports 60+ languages with automatic language injection (e.g., CSS/JS in HTML).
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use arborium::Highlighter;
//!
//! let mut hl = Highlighter::new();
//! let html = hl.highlight("rust", "fn main() {}")?;
//! // Output: <a-k>fn</a-k> <a-f>main</a-f>() {}
//! ```
//!
//! # HTML vs ANSI Output
//!
//! Use [`Highlighter`] for HTML output (web pages, documentation):
//!
//! ```rust,ignore
//! use arborium::{Highlighter, Config, HtmlFormat};
//!
//! // Default: custom elements (<a-k>, <a-f>, etc.)
//! let mut hl = Highlighter::new();
//!
//! // Or use class-based output for CSS compatibility
//! let config = Config {
//!     html_format: HtmlFormat::ClassNames,
//!     ..Default::default()
//! };
//! let mut hl = Highlighter::with_config(config);
//! ```
//!
//! Use [`AnsiHighlighter`] for terminal output:
//!
//! ```rust,ignore
//! use arborium::AnsiHighlighter;
//! use arborium::theme::builtin;
//!
//! let theme = builtin::catppuccin_mocha().clone();
//! let mut hl = AnsiHighlighter::new(theme);
//! let colored = hl.highlight("rust", "fn main() {}")?;
//! println!("{}", colored);
//! ```
//!
//! # Language Support
//!
//! Enable languages via feature flags:
//!
//! ```toml
//! [dependencies]
//! arborium = { version = "0.1", features = ["lang-rust", "lang-python"] }
//! ```
//!
//! Or enable all languages:
//!
//! ```toml
//! [dependencies]
//! arborium = { version = "0.1", features = ["all-languages"] }
//! ```
//!
//! ## Supported Languages
//!
//! ### Permissively Licensed (<%= permissive_grammars.len() %> languages, included by default)
//!
//! | Language | Feature Flag | License |
//! |----------|--------------|---------|
<% for grammar in permissive_grammars { %>
//! | <%= grammar.name %> | `<%= grammar.feature %>` | <%= grammar.license %> |
<% } %>
//!
//! ### GPL Licensed (<%= gpl_grammars.len() %> languages, opt-in)
//!
//! These require explicit opt-in via feature flags due to their copyleft license.
//!
//! | Language | Feature Flag | License |
//! |----------|--------------|---------|
<% for grammar in gpl_grammars { %>
//! | <%= grammar.name %> | `<%= grammar.feature %>` | <%= grammar.license %> |
<% } %>
//!
//! # Advanced Usage
//!
//! For building custom grammar providers or working with raw spans, see the
//! [`advanced`] module.

// Internal modules
mod error;
mod highlighter;
pub(crate) mod store;

// Public modules
pub mod advanced;

/// Theme system for ANSI output.
///
/// Re-exports types from `arborium-theme` for configuring syntax colors.
pub mod theme {
    pub use arborium_theme::theme::{builtin, Color, Modifiers, Style, Theme};
}

// Primary API exports
pub use error::Error;
pub use highlighter::{AnsiHighlighter, Highlighter};
pub use store::GrammarStore;

// Configuration types (re-exported from arborium-highlight)
pub use arborium_highlight::HtmlFormat;

/// Configuration for highlighting.
///
/// Controls injection depth and HTML output format.
#[derive(Debug, Clone)]
pub struct Config {
    /// Maximum depth for processing language injections.
    ///
    /// - `0`: No injections (just primary language)
    /// - `3`: Default, handles most cases (HTML with CSS/JS, Markdown with code blocks)
    /// - Higher: For deeply nested content
    pub max_injection_depth: u32,

    /// HTML output format.
    ///
    /// See [`HtmlFormat`] for options.
    pub html_format: HtmlFormat,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            max_injection_depth: 3,
            html_format: HtmlFormat::default(),
        }
    }
}

impl From<Config> for arborium_highlight::HighlightConfig {
    fn from(config: Config) -> Self {
        arborium_highlight::HighlightConfig {
            max_injection_depth: config.max_injection_depth,
            html_format: config.html_format,
        }
    }
}

// Tree-sitter re-export for advanced users
pub use arborium_tree_sitter as tree_sitter;

// WASM allocator (automatically enabled on WASM targets)
// Provides malloc/calloc/realloc/free symbols for tree-sitter's C code
#[cfg(target_family = "wasm")]
mod wasm;

// Highlight names constant
use arborium_theme::highlights;

/// Standard highlight names used for syntax highlighting.
///
/// These names are used to configure tree-sitter's `HighlightConfiguration`.
/// The indices correspond to HTML element tags (e.g., index 7 = `<a-k>` for keyword).
pub const HIGHLIGHT_NAMES: [&str; highlights::COUNT] = highlights::names();

/// Detect the language from a file path or name.
///
/// Extracts the file extension and maps it to a canonical language identifier.
/// Returns `None` if the extension is not recognized.
///
/// # Example
///
/// ```rust
/// use arborium::detect_language;
///
/// assert_eq!(detect_language("main.rs"), Some("rust"));
/// assert_eq!(detect_language("/path/to/script.py"), Some("python"));
/// assert_eq!(detect_language("styles.css"), Some("css"));
/// assert_eq!(detect_language("unknown.xyz"), None);
/// ```
pub fn detect_language(path: &str) -> Option<&'static str> {
    // Extract extension from path
    let ext = path
        .rsplit('.')
        .next()
        .filter(|e| !e.contains('/') && !e.contains('\\'))?;

    // Map extension to canonical language ID
    Some(match ext.to_lowercase().as_str() {
<% for (ext, lang) in extensions { %>
        "<%= ext %>" => "<%= lang %>",
<% } %>
        _ => return None,
    })
}

// =============================================================================
// Language grammar re-exports based on enabled features.
// Each module provides:
// - `language()` - Returns the tree-sitter Language
// - `HIGHLIGHTS_QUERY` - The highlight query string
// - `INJECTIONS_QUERY` - The injection query string
// - `LOCALS_QUERY` - The locals query string
// =============================================================================

<% for (crate_name, grammar_id) in grammars { %>
#[cfg(feature = "lang-<%= grammar_id %>")]
pub use <%= crate_name.replace('-', "_") %> as lang_<%= grammar_id.replace('-', "_") %>;

<% } %>

/// Returns the tree-sitter `Language` for the given language name.
///
/// This function only returns languages that are enabled via feature flags.
/// If no grammar matches the provided language name, it returns `None`.
///
/// # Example
///
/// ```rust,ignore
/// use arborium::get_language;
///
/// // Returns Some if the "lang-rust" feature is enabled
/// if let Some(lang) = get_language("rust") {
///     println!("Got Rust language: {:?}", lang);
/// }
///
/// // Returns None for unknown languages
/// assert!(get_language("unknown-language").is_none());
/// ```
pub fn get_language(name: &str) -> Option<tree_sitter::Language> {
    match name {
<% for (crate_name, grammar_id) in grammars { %>
        #[cfg(feature = "lang-<%= grammar_id %>")]
        "<%= grammar_id %>" => Some(<%= crate_name.replace('-', "_") %>::language().into()),
<% } %>
        _ => None,
    }
}
