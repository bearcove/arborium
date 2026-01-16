//! Tree-sitter based highlighting with thread-safe grammar sharing.
//!
//! This module provides a split architecture for efficient multi-threaded highlighting:
//!
//! - [`CompiledGrammar`]: Thread-safe compiled queries, shareable via `Arc`
//! - [`ParseContext`]: Per-thread parser state, cheap to create
//!
//! # Single-threaded Usage
//!
//! ```rust,ignore
//! use arborium_highlight::tree_sitter::{CompiledGrammar, ParseContext};
//!
//! let grammar = Arc::new(CompiledGrammar::new(config)?);
//! let mut ctx = ParseContext::for_grammar(&grammar)?;
//! let result = grammar.parse(&mut ctx, "fn main() {}");
//! ```
//!
//! # Multi-threaded Usage
//!
//! ```rust,ignore
//! use std::sync::Arc;
//! use rayon::prelude::*;
//!
//! // Compile grammar once
//! let grammar = Arc::new(CompiledGrammar::new(config)?);
//!
//! // Each thread gets its own context
//! let results: Vec<_> = code_blocks.par_iter().map(|code| {
//!     let mut ctx = ParseContext::for_grammar(&grammar).unwrap();
//!     grammar.parse(&mut ctx, code)
//! }).collect();
//! ```

use crate::types::{Injection, ParseResult, Span};
use arborium_tree_sitter::{Language, Parser, Query, QueryCursor};
use streaming_iterator::StreamingIterator;

/// Configuration for creating a [`CompiledGrammar`].
pub struct GrammarConfig<'a> {
    /// The tree-sitter Language
    pub language: Language,
    /// The highlights query (required for syntax highlighting)
    pub highlights_query: &'a str,
    /// The injections query (for embedded languages)
    pub injections_query: &'a str,
    /// The locals query (for local variable tracking, currently unused)
    pub locals_query: &'a str,
}

/// Error when creating a grammar or parse context.
#[derive(Debug)]
pub enum GrammarError {
    /// Failed to set the parser language
    LanguageError,
    /// Failed to compile a query
    QueryError(String),
}

impl std::fmt::Display for GrammarError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GrammarError::LanguageError => write!(f, "Failed to set parser language"),
            GrammarError::QueryError(e) => write!(f, "Query compilation error: {}", e),
        }
    }
}

impl std::error::Error for GrammarError {}

/// Compiled grammar data that can be shared across threads.
///
/// This holds the compiled tree-sitter queries which are expensive to create
/// but cheap to use. Share via `Arc<CompiledGrammar>` for multi-threaded
/// highlighting.
///
/// # Thread Safety
///
/// `CompiledGrammar` is `Send + Sync` and can be freely shared across threads.
/// Each thread needs its own [`ParseContext`] to actually parse text.
pub struct CompiledGrammar {
    language: Language,
    highlights_query: Query,
    injections_query: Option<Query>,
    // Cached capture indices for injection query
    injection_content_idx: Option<u32>,
    injection_language_idx: Option<u32>,
}

// Safety: CompiledGrammar only contains Language and Query types from tree-sitter.
// Both types are documented as thread-safe (immutable after creation).
// We verify this at compile time with the assertions below.
unsafe impl Send for CompiledGrammar {}
unsafe impl Sync for CompiledGrammar {}

// Compile-time verification that the underlying types are Send + Sync
const _: () = {
    const fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Language>();
    assert_send_sync::<Query>();
};

impl CompiledGrammar {
    /// Create a new compiled grammar from configuration.
    ///
    /// This compiles the highlight and injection queries, which can be expensive.
    /// The resulting `CompiledGrammar` can be wrapped in `Arc` and shared across threads.
    pub fn new(config: GrammarConfig<'_>) -> Result<Self, GrammarError> {
        let highlights_query = Query::new(&config.language, config.highlights_query)
            .map_err(|e| GrammarError::QueryError(e.to_string()))?;

        let injections_query = if config.injections_query.is_empty() {
            None
        } else {
            Some(
                Query::new(&config.language, config.injections_query)
                    .map_err(|e| GrammarError::QueryError(e.to_string()))?,
            )
        };

        // Pre-compute injection capture indices
        let (injection_content_idx, injection_language_idx) =
            if let Some(ref query) = injections_query {
                let mut content_idx = None;
                let mut language_idx = None;
                for (i, name) in query.capture_names().iter().enumerate() {
                    match *name {
                        "injection.content" => content_idx = Some(i as u32),
                        "injection.language" => language_idx = Some(i as u32),
                        _ => {}
                    }
                }
                (content_idx, language_idx)
            } else {
                (None, None)
            };

        Ok(Self {
            language: config.language,
            highlights_query,
            injections_query,
            injection_content_idx,
            injection_language_idx,
        })
    }

    /// Get the tree-sitter language for this grammar.
    pub fn language(&self) -> &Language {
        &self.language
    }

    /// Parse text and return highlight spans and injection points.
    ///
    /// Requires a [`ParseContext`] which holds the mutable parser state.
    /// Each thread should have its own context.
    pub fn parse(&self, ctx: &mut ParseContext, text: &str) -> ParseResult {
        // Parse the text
        let tree = match ctx.parser.parse(text, None) {
            Some(tree) => tree,
            None => return ParseResult::default(),
        };

        let root_node = tree.root_node();
        let source = text.as_bytes();

        // Collect highlight spans
        let mut spans = Vec::new();

        let mut matches = ctx
            .cursor
            .matches(&self.highlights_query, root_node, source);

        while let Some(m) = matches.next() {
            for capture in m.captures {
                let capture_name = self.highlights_query.capture_names()[capture.index as usize];

                // Skip internal captures (start with _)
                if capture_name.starts_with('_') {
                    continue;
                }

                // Skip injection-related captures
                if capture_name.starts_with("injection.") {
                    continue;
                }

                let node = capture.node;
                spans.push(Span {
                    start: node.start_byte() as u32,
                    end: node.end_byte() as u32,
                    capture: capture_name.to_string(),
                    pattern_index: m.pattern_index as u32,
                });
            }
        }

        // Collect injections
        let mut injections = Vec::new();

        if let Some(ref injections_query) = self.injections_query {
            let mut matches = ctx.cursor.matches(injections_query, root_node, source);

            while let Some(m) = matches.next() {
                let mut content_node = None;
                let mut language_name = None;
                let mut include_children = false;

                // Check for #set! injection.language property
                for prop in injections_query.property_settings(m.pattern_index) {
                    match prop.key.as_ref() {
                        "injection.language" => {
                            if let Some(ref value) = prop.value {
                                language_name = Some(value.to_string());
                            }
                        }
                        "injection.include-children" => {
                            include_children = true;
                        }
                        _ => {}
                    }
                }

                // Get captures
                for capture in m.captures {
                    if Some(capture.index) == self.injection_content_idx {
                        content_node = Some(capture.node);
                    } else if Some(capture.index) == self.injection_language_idx {
                        // Language can come from captured text
                        if language_name.is_none() {
                            if let Ok(lang) = capture.node.utf8_text(source) {
                                language_name = Some(lang.to_string());
                            }
                        }
                    }
                }

                if let (Some(node), Some(lang)) = (content_node, language_name) {
                    injections.push(Injection {
                        start: node.start_byte() as u32,
                        end: node.end_byte() as u32,
                        language: lang,
                        include_children,
                    });
                }
            }
        }

        ParseResult { spans, injections }
    }
}

/// Per-thread parsing context.
///
/// This holds the mutable state needed for parsing: a [`Parser`] and [`QueryCursor`].
/// Creating a `ParseContext` is cheap compared to compiling queries.
///
/// # Usage
///
/// Each thread should have its own `ParseContext`. Create it once and reuse
/// for multiple parse calls.
///
/// ```rust,ignore
/// let mut ctx = ParseContext::for_grammar(&grammar)?;
///
/// // Reuse for multiple parses
/// let result1 = grammar.parse(&mut ctx, code1);
/// let result2 = grammar.parse(&mut ctx, code2);
/// ```
pub struct ParseContext {
    parser: Parser,
    cursor: QueryCursor,
}

impl ParseContext {
    /// Create a new parse context for a grammar.
    ///
    /// The parser is configured for the grammar's language.
    pub fn for_grammar(grammar: &CompiledGrammar) -> Result<Self, GrammarError> {
        let mut parser = Parser::new();
        parser
            .set_language(&grammar.language)
            .map_err(|_| GrammarError::LanguageError)?;

        Ok(Self {
            parser,
            cursor: QueryCursor::new(),
        })
    }

    /// Create a new parse context for a language.
    ///
    /// Use this when you need a context before having a grammar,
    /// or when switching between grammars with the same language.
    pub fn for_language(language: &Language) -> Result<Self, GrammarError> {
        let mut parser = Parser::new();
        parser
            .set_language(language)
            .map_err(|_| GrammarError::LanguageError)?;

        Ok(Self {
            parser,
            cursor: QueryCursor::new(),
        })
    }

    /// Reset the parser for a new language.
    ///
    /// Call this when switching to a grammar with a different language.
    pub fn set_language(&mut self, language: &Language) -> Result<(), GrammarError> {
        self.parser
            .set_language(language)
            .map_err(|_| GrammarError::LanguageError)
    }
}

// Backward compatibility aliases
#[doc(hidden)]
pub type TreeSitterGrammarConfig<'a> = GrammarConfig<'a>;
#[doc(hidden)]
pub type TreeSitterGrammarError = GrammarError;

#[cfg(test)]
mod tests {
    // Tests would go here but require actual tree-sitter grammars
}
