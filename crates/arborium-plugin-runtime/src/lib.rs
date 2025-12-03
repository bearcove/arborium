//! Runtime library for arborium grammar plugins.
//!
//! This crate provides the core functionality needed to implement
//! a tree-sitter grammar as a WASM component plugin. It handles:
//!
//! - Session management (create/free)
//! - Parser state and tree storage
//! - Query execution to produce Span and Injection records
//! - Incremental parsing via edit application
//! - Cancellation support
//!
//! # Example
//!
//! ```ignore
//! use arborium_plugin_runtime::{PluginRuntime, HighlightConfig};
//!
//! let config = HighlightConfig::new(
//!     my_language(),
//!     HIGHLIGHTS_QUERY,
//!     INJECTIONS_QUERY,
//!     LOCALS_QUERY,
//! ).unwrap();
//!
//! let mut runtime = PluginRuntime::new(config);
//! let session = runtime.create_session();
//! runtime.set_text(session, "fn main() {}");
//! let result = runtime.parse(session).unwrap();
//! ```

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use arborium_wire::{Edit, Injection, ParseError, ParseResult, Span};
use tree_sitter_patched_arborium::{
    InputEdit, Language, Parser, Point, Query, QueryCursor, QueryError, StreamingIterator, Tree,
};

/// Configuration for syntax highlighting.
///
/// Contains the compiled queries for highlights, injections, and locals.
pub struct HighlightConfig {
    language: Language,
    query: Query,
    injection_content_capture_index: Option<u32>,
    injection_language_capture_index: Option<u32>,
    locals_pattern_index: usize,
    highlights_pattern_index: usize,
}

impl HighlightConfig {
    /// Create a new highlight configuration.
    ///
    /// # Arguments
    /// * `language` - The tree-sitter language
    /// * `highlights_query` - Query for syntax highlighting captures
    /// * `injections_query` - Query for language injections
    /// * `locals_query` - Query for local variable tracking
    pub fn new(
        language: Language,
        highlights_query: &str,
        injections_query: &str,
        locals_query: &str,
    ) -> Result<Self, QueryError> {
        // Concatenate queries: injections, then locals, then highlights
        // Add newline separators to ensure queries don't merge incorrectly
        // if they don't end with newlines
        let mut query_source = String::new();
        query_source.push_str(injections_query);
        if !injections_query.is_empty() && !injections_query.ends_with('\n') {
            query_source.push('\n');
        }
        let locals_query_offset = query_source.len();
        query_source.push_str(locals_query);
        if !locals_query.is_empty() && !locals_query.ends_with('\n') {
            query_source.push('\n');
        }
        let highlights_query_offset = query_source.len();
        query_source.push_str(highlights_query);

        let query = Query::new(&language, &query_source)?;

        // Find pattern indices for each section
        let mut locals_pattern_index = 0;
        let mut highlights_pattern_index = 0;
        for i in 0..query.pattern_count() {
            let pattern_offset = query.start_byte_for_pattern(i);
            if pattern_offset < highlights_query_offset {
                highlights_pattern_index += 1;
                if pattern_offset < locals_query_offset {
                    locals_pattern_index += 1;
                }
            }
        }

        // Find injection capture indices
        let mut injection_content_capture_index = None;
        let mut injection_language_capture_index = None;
        for (i, name) in query.capture_names().iter().enumerate() {
            match *name {
                "injection.content" => injection_content_capture_index = Some(i as u32),
                "injection.language" => injection_language_capture_index = Some(i as u32),
                _ => {}
            }
        }

        Ok(Self {
            language,
            query,
            injection_content_capture_index,
            injection_language_capture_index,
            locals_pattern_index,
            highlights_pattern_index,
        })
    }

    /// Get the capture names from the query.
    pub fn capture_names(&self) -> &[&str] {
        self.query.capture_names()
    }
}

/// A parsing session that maintains parser state.
struct Session {
    parser: Parser,
    tree: Option<Tree>,
    text: String,
    cursor: QueryCursor,
    cancelled: AtomicBool,
}

impl Session {
    fn new(language: &Language) -> Self {
        let mut parser = Parser::new();
        parser
            .set_language(language)
            .expect("language should be valid");
        Self {
            parser,
            tree: None,
            text: String::new(),
            cursor: QueryCursor::new(),
            cancelled: AtomicBool::new(false),
        }
    }
}

/// Runtime for a grammar plugin.
///
/// Manages parsing sessions and executes queries to produce
/// highlight spans and injection points.
pub struct PluginRuntime {
    config: HighlightConfig,
    sessions: BTreeMap<u32, Session>,
    next_session_id: AtomicU32,
}

impl PluginRuntime {
    /// Create a new plugin runtime with the given highlight configuration.
    pub fn new(config: HighlightConfig) -> Self {
        Self {
            config,
            sessions: BTreeMap::new(),
            next_session_id: AtomicU32::new(1),
        }
    }

    /// Create a new parsing session.
    ///
    /// Returns a session handle that can be used with other methods.
    pub fn create_session(&mut self) -> u32 {
        let id = self.next_session_id.fetch_add(1, Ordering::Relaxed);
        let session = Session::new(&self.config.language);
        self.sessions.insert(id, session);
        id
    }

    /// Free a parsing session and its resources.
    pub fn free_session(&mut self, session_id: u32) {
        self.sessions.remove(&session_id);
    }

    /// Set the full text content for a session.
    ///
    /// This replaces any previous content and resets the parse tree.
    pub fn set_text(&mut self, session_id: u32, text: &str) {
        if let Some(session) = self.sessions.get_mut(&session_id) {
            session.text = String::from(text);
            session.tree = session.parser.parse(text, None);
            session.cancelled.store(false, Ordering::Relaxed);
        }
    }

    /// Apply an incremental edit to the session's text.
    ///
    /// The session must have had `set_text` called previously.
    pub fn apply_edit(&mut self, session_id: u32, new_text: &str, edit: &Edit) {
        if let Some(session) = self.sessions.get_mut(&session_id) {
            // Update the text
            session.text = String::from(new_text);

            // Apply the edit to the existing tree if we have one
            if let Some(tree) = &mut session.tree {
                let input_edit = InputEdit {
                    start_byte: edit.start_byte as usize,
                    old_end_byte: edit.old_end_byte as usize,
                    new_end_byte: edit.new_end_byte as usize,
                    start_position: Point::new(edit.start_row as usize, edit.start_col as usize),
                    old_end_position: Point::new(
                        edit.old_end_row as usize,
                        edit.old_end_col as usize,
                    ),
                    new_end_position: Point::new(
                        edit.new_end_row as usize,
                        edit.new_end_col as usize,
                    ),
                };
                tree.edit(&input_edit);
            }

            // Re-parse with the old tree for incremental parsing
            session.tree = session.parser.parse(&session.text, session.tree.as_ref());
            session.cancelled.store(false, Ordering::Relaxed);
        }
    }

    /// Request cancellation of an in-progress parse.
    pub fn cancel(&mut self, session_id: u32) {
        if let Some(session) = self.sessions.get(&session_id) {
            session.cancelled.store(true, Ordering::Relaxed);
        }
    }

    /// Parse the current text and return spans and injections.
    ///
    /// If cancelled, returns an empty result.
    pub fn parse(&mut self, session_id: u32) -> Result<ParseResult, ParseError> {
        let session = self
            .sessions
            .get_mut(&session_id)
            .ok_or_else(|| ParseError::new("invalid session id"))?;

        // Check for cancellation
        if session.cancelled.load(Ordering::Relaxed) {
            return Ok(ParseResult::empty());
        }

        let tree = session
            .tree
            .as_ref()
            .ok_or_else(|| ParseError::new("no text set for session"))?;

        let mut spans = Vec::new();
        let mut injections = Vec::new();

        let source = session.text.as_bytes();
        let root = tree.root_node();

        // Execute the query using streaming iterator
        let mut matches = session.cursor.matches(&self.config.query, root, source);

        let mut check_count = 0;
        const CANCELLATION_CHECK_INTERVAL: usize = 100;

        while let Some(m) = matches.next() {
            // Periodically check for cancellation
            check_count += 1;
            if check_count >= CANCELLATION_CHECK_INTERVAL {
                check_count = 0;
                if session.cancelled.load(Ordering::Relaxed) {
                    return Ok(ParseResult::empty());
                }
            }

            // Process injections (patterns before locals_pattern_index)
            if m.pattern_index < self.config.locals_pattern_index {
                let mut language_name: Option<&str> = None;
                let mut content_node = None;
                let mut include_children = false;

                for capture in m.captures {
                    if Some(capture.index) == self.config.injection_language_capture_index {
                        if let Ok(name) = capture.node.utf8_text(source) {
                            language_name = Some(name);
                        }
                    } else if Some(capture.index) == self.config.injection_content_capture_index {
                        content_node = Some(capture.node);
                    }
                }

                // Check for #set! predicates
                for prop in self.config.query.property_settings(m.pattern_index) {
                    match prop.key.as_ref() {
                        "injection.language" => {
                            if language_name.is_none() {
                                language_name = prop.value.as_ref().map(|v| v.as_ref());
                            }
                        }
                        "injection.include-children" => {
                            include_children = true;
                        }
                        _ => {}
                    }
                }

                if let (Some(lang), Some(node)) = (language_name, content_node) {
                    injections.push(Injection {
                        start: node.start_byte() as u32,
                        end: node.end_byte() as u32,
                        language: String::from(lang),
                        include_children,
                    });
                }

                continue;
            }

            // Skip locals patterns (between locals_pattern_index and highlights_pattern_index)
            if m.pattern_index < self.config.highlights_pattern_index {
                continue;
            }

            // Process highlights
            for capture in m.captures {
                let capture_name = self.config.query.capture_names()[capture.index as usize];

                // Skip internal captures (starting with underscore)
                if capture_name.starts_with('_') {
                    continue;
                }

                // Skip injection-related captures
                if capture_name.starts_with("injection.") {
                    continue;
                }

                // Skip local-related captures
                if capture_name.starts_with("local.") {
                    continue;
                }

                let node = capture.node;
                spans.push(Span {
                    start: node.start_byte() as u32,
                    end: node.end_byte() as u32,
                    capture: String::from(capture_name),
                });
            }
        }

        // Sort spans by start position for consistent output
        spans.sort_by_key(|s| (s.start, s.end));

        Ok(ParseResult { spans, injections })
    }

    /// Get the language provided by this plugin.
    pub fn language(&self) -> &Language {
        &self.config.language
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rust_code() {
        let config = HighlightConfig::new(
            arborium_rust::language(),
            arborium_rust::HIGHLIGHTS_QUERY,
            arborium_rust::INJECTIONS_QUERY,
            arborium_rust::LOCALS_QUERY,
        )
        .expect("failed to create config");

        let mut runtime = PluginRuntime::new(config);
        let session = runtime.create_session();

        runtime.set_text(session, "fn main() { let x = 42; }");
        let result = runtime.parse(session).expect("parse failed");

        // Should have some spans
        assert!(!result.spans.is_empty(), "expected some spans");

        // Check that we have keyword spans
        let has_keyword = result.spans.iter().any(|s| s.capture == "keyword");
        assert!(has_keyword, "expected keyword captures");

        // Check that we have function spans
        let has_function = result.spans.iter().any(|s| s.capture.contains("function"));
        assert!(has_function, "expected function captures");

        runtime.free_session(session);
    }

    #[test]
    fn test_incremental_edit() {
        let config = HighlightConfig::new(
            arborium_rust::language(),
            arborium_rust::HIGHLIGHTS_QUERY,
            arborium_rust::INJECTIONS_QUERY,
            arborium_rust::LOCALS_QUERY,
        )
        .expect("failed to create config");

        let mut runtime = PluginRuntime::new(config);
        let session = runtime.create_session();

        // Initial parse
        let initial = "fn main() {}";
        runtime.set_text(session, initial);
        let result1 = runtime.parse(session).expect("parse failed");

        // Apply edit: insert " let x = 1;" after "{"
        let new_text = "fn main() { let x = 1; }";
        let edit = Edit {
            start_byte: 11,
            old_end_byte: 11,
            new_end_byte: 23,
            start_row: 0,
            start_col: 11,
            old_end_row: 0,
            old_end_col: 11,
            new_end_row: 0,
            new_end_col: 23,
        };
        runtime.apply_edit(session, new_text, &edit);
        let result2 = runtime.parse(session).expect("parse failed");

        // After edit should have more spans
        assert!(result2.spans.len() > result1.spans.len());

        runtime.free_session(session);
    }

    #[test]
    fn test_cancellation() {
        let config = HighlightConfig::new(
            arborium_rust::language(),
            arborium_rust::HIGHLIGHTS_QUERY,
            arborium_rust::INJECTIONS_QUERY,
            arborium_rust::LOCALS_QUERY,
        )
        .expect("failed to create config");

        let mut runtime = PluginRuntime::new(config);
        let session = runtime.create_session();

        runtime.set_text(session, "fn main() {}");

        // Cancel before parsing
        runtime.cancel(session);

        let result = runtime.parse(session).expect("parse failed");

        // Should return empty result due to cancellation
        assert!(result.spans.is_empty());

        runtime.free_session(session);
    }
}
