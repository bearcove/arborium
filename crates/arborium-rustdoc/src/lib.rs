//! Post-process rustdoc HTML output to add syntax highlighting for non-Rust code blocks.
//!
//! This crate provides tools to transform rustdoc-generated HTML, adding tree-sitter based
//! syntax highlighting for code blocks in languages other than Rust (which rustdoc already
//! highlights using rustc's parser).
//!
//! # Usage
//!
//! ```bash
//! arborium-rustdoc ./target/doc ./target/doc-highlighted
//! ```
//!
//! # How it works
//!
//! 1. **CSS Generation**: Generates theme CSS rules for arborium's custom elements
//!    and appends them to rustdoc's CSS file (`static.files/rustdoc-*.css`)
//!
//! 2. **HTML Transformation**: Uses lol_html to stream through each HTML file,
//!    finding `<pre class="language-*">` elements and replacing their content
//!    with syntax-highlighted HTML.
//!
//! # Theme Support
//!
//! Integrates with rustdoc's built-in theme system (light, dark, ayu) by generating
//! CSS rules scoped to `[data-theme="..."]` selectors.

mod css;
mod html;
mod processor;

pub use css::generate_rustdoc_theme_css;
pub use html::transform_html;
pub use processor::{ProcessOptions, Processor, ProcessorStats};
