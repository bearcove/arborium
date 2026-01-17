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
//! # Offset Encoding
//!
//! Tree-sitter natively produces UTF-8 byte offsets. This runtime provides
//! two parsing methods:
//!
//! - [`PluginRuntime::parse`] returns UTF-8 byte offsets (for Rust string slicing)
//! - [`PluginRuntime::parse_utf16`] returns UTF-16 code unit indices (for JavaScript)
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
//!
//! // For Rust code (UTF-8 offsets):
//! let result = runtime.parse(session).unwrap();
//!
//! // For JavaScript interop (UTF-16 offsets):
//! let result = runtime.parse_utf16(session).unwrap();
//! ```

extern crate alloc;

#[cfg(target_family = "wasm")]
use arborium_sysroot as _;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use arborium_tree_sitter::{
    InputEdit, Language, LanguageFn, Parser, Point, Query, QueryCursor, QueryError,
    StreamingIterator, Tree,
};
use arborium_wire::{
    Edit, ParseError, Utf8Injection, Utf8ParseResult, Utf8Span, Utf16Injection, Utf16ParseResult,
    Utf16Span,
};

/// Batch convert UTF-8 byte offsets to UTF-16 code unit indices in a single pass.
///
/// This is O(n + m) where n is string length and m is number of offsets,
/// much better than O(n * m) for individual conversions.
///
/// The offsets slice must be sorted in ascending order.
fn batch_utf8_to_utf16(text: &str, offsets: &[usize]) -> Vec<u32> {
    let mut results = Vec::with_capacity(offsets.len());
    if offsets.is_empty() {
        return results;
    }

    let mut offset_idx = 0;
    let mut utf16_index = 0u32;
    let mut byte_index = 0usize;

    for c in text.chars() {
        // Emit results for all offsets at current byte position
        while offset_idx < offsets.len() && byte_index >= offsets[offset_idx] {
            results.push(utf16_index);
            offset_idx += 1;
        }

        if offset_idx >= offsets.len() {
            break;
        }

        byte_index += c.len_utf8();
        // Code points >= 0x10000 use surrogate pairs (2 UTF-16 code units)
        utf16_index += if c as u32 >= 0x10000 { 2 } else { 1 };
    }

    // Handle any remaining offsets at or past the end
    while offset_idx < offsets.len() {
        results.push(utf16_index);
        offset_idx += 1;
    }

    results
}

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
        language: LanguageFn,
        highlights_query: &str,
        injections_query: &str,
        locals_query: &str,
    ) -> Result<Self, QueryError> {
        let language: Language = language.into();
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

// Internal structs to hold raw byte offsets during parsing
struct RawSpan {
    start: usize,
    end: usize,
    capture: String,
    pattern_index: usize,
}

struct RawInjection {
    start: usize,
    end: usize,
    language: String,
    include_children: bool,
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

    /// Internal: execute query and collect raw spans/injections with byte offsets.
    fn parse_raw(
        &mut self,
        session_id: u32,
    ) -> Result<(String, Vec<RawSpan>, Vec<RawInjection>), ParseError> {
        let session = self
            .sessions
            .get_mut(&session_id)
            .ok_or_else(|| ParseError::new("invalid session id"))?;

        // Check for cancellation
        if session.cancelled.load(Ordering::Relaxed) {
            return Ok((String::new(), Vec::new(), Vec::new()));
        }

        let tree = session
            .tree
            .as_ref()
            .ok_or_else(|| ParseError::new("no text set for session"))?;

        let mut raw_spans: Vec<RawSpan> = Vec::new();
        let mut raw_injections: Vec<RawInjection> = Vec::new();

        let text = session.text.clone();
        let source = text.as_bytes();
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
                    return Ok((String::new(), Vec::new(), Vec::new()));
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
                    raw_injections.push(RawInjection {
                        start: node.start_byte(),
                        end: node.end_byte(),
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
                raw_spans.push(RawSpan {
                    start: node.start_byte(),
                    end: node.end_byte(),
                    capture: String::from(capture_name),
                    pattern_index: m.pattern_index,
                });
            }
        }

        Ok((text, raw_spans, raw_injections))
    }

    /// Parse the current text and return spans and injections with UTF-8 byte offsets.
    ///
    /// Use this when working with Rust strings, as `&source[start..end]` requires
    /// UTF-8 byte boundaries.
    ///
    /// If cancelled, returns an empty result.
    pub fn parse(&mut self, session_id: u32) -> Result<Utf8ParseResult, ParseError> {
        let (_text, raw_spans, raw_injections) = self.parse_raw(session_id)?;

        // Convert to UTF-8 spans (just cast the byte offsets)
        let mut spans: Vec<Utf8Span> = raw_spans
            .into_iter()
            .map(|s| Utf8Span {
                start: s.start as u32,
                end: s.end as u32,
                capture: s.capture,
                pattern_index: s.pattern_index as u32,
            })
            .collect();

        // Sort spans by start position for consistent output
        spans.sort_by_key(|s| (s.start, s.end));

        // Convert injections
        let injections: Vec<Utf8Injection> = raw_injections
            .into_iter()
            .map(|i| Utf8Injection {
                start: i.start as u32,
                end: i.end as u32,
                language: i.language,
                include_children: i.include_children,
            })
            .collect();

        Ok(Utf8ParseResult { spans, injections })
    }

    /// Parse the current text and return spans and injections with UTF-16 code unit indices.
    ///
    /// Use this when working with JavaScript, as `String.prototype.slice()` and
    /// DOM APIs use UTF-16 code unit indices.
    ///
    /// If cancelled, returns an empty result.
    pub fn parse_utf16(&mut self, session_id: u32) -> Result<Utf16ParseResult, ParseError> {
        let (text, raw_spans, raw_injections) = self.parse_raw(session_id)?;

        if raw_spans.is_empty() && raw_injections.is_empty() {
            return Ok(Utf16ParseResult::empty());
        }

        // Collect all byte offsets and batch convert to UTF-16
        let mut all_offsets: Vec<usize> =
            Vec::with_capacity((raw_spans.len() + raw_injections.len()) * 2);
        for span in &raw_spans {
            all_offsets.push(span.start);
            all_offsets.push(span.end);
        }
        for inj in &raw_injections {
            all_offsets.push(inj.start);
            all_offsets.push(inj.end);
        }
        all_offsets.sort_unstable();

        let utf16_offsets = batch_utf8_to_utf16(&text, &all_offsets);

        // Build a lookup from byte offset to UTF-16 offset
        // (using binary search since offsets are sorted)
        let lookup = |byte_offset: usize| -> u32 {
            let idx = all_offsets
                .binary_search(&byte_offset)
                .unwrap_or_else(|x| x);
            utf16_offsets.get(idx).copied().unwrap_or(0)
        };

        // Convert spans to UTF-16
        let mut spans: Vec<Utf16Span> = raw_spans
            .into_iter()
            .map(|s| Utf16Span {
                start: lookup(s.start),
                end: lookup(s.end),
                capture: s.capture,
                pattern_index: s.pattern_index as u32,
            })
            .collect();

        // Sort spans by start position for consistent output
        spans.sort_by_key(|s| (s.start, s.end));

        // Convert injections to UTF-16
        let injections: Vec<Utf16Injection> = raw_injections
            .into_iter()
            .map(|i| Utf16Injection {
                start: lookup(i.start),
                end: lookup(i.end),
                language: i.language,
                include_children: i.include_children,
            })
            .collect();

        Ok(Utf16ParseResult { spans, injections })
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
    fn test_batch_utf8_to_utf16_ascii() {
        // ASCII: 1 byte UTF-8 = 1 UTF-16 code unit
        let text = "hello";
        let offsets = [0, 1, 5];
        let result = batch_utf8_to_utf16(text, &offsets);
        assert_eq!(result, vec![0, 1, 5]);
    }

    #[test]
    fn test_batch_utf8_to_utf16_two_byte() {
        // é is 2 bytes in UTF-8, 1 UTF-16 code unit
        let text = "café";
        // c=0, a=1, f=2, é=3-4 (2 bytes)
        let offsets = [0, 3, 5];
        let result = batch_utf8_to_utf16(text, &offsets);
        assert_eq!(result, vec![0, 3, 4]); // byte 5 = UTF-16 index 4
    }

    #[test]
    fn test_batch_utf8_to_utf16_three_byte() {
        // 中 is 3 bytes in UTF-8, 1 UTF-16 code unit
        let text = "a中b";
        // a=0 (1 byte), 中=1-3 (3 bytes), b=4 (1 byte)
        let offsets = [0, 1, 4, 5];
        let result = batch_utf8_to_utf16(text, &offsets);
        assert_eq!(result, vec![0, 1, 2, 3]);
    }

    #[test]
    fn test_batch_utf8_to_utf16_four_byte_emoji() {
        // 🦀 is 4 bytes in UTF-8, 2 UTF-16 code units (surrogate pair)
        let text = "a🦀b";
        // a=0 (1 byte), 🦀=1-4 (4 bytes), b=5 (1 byte)
        let offsets = [0, 1, 5, 6];
        let result = batch_utf8_to_utf16(text, &offsets);
        assert_eq!(result, vec![0, 1, 3, 4]); // emoji takes 2 UTF-16 units
    }

    #[test]
    fn test_batch_utf8_to_utf16_mixed() {
        // Mix of ASCII, 2-byte, 3-byte, and 4-byte characters
        let text = "hi🌍世界";
        // h=0, i=1, 🌍=2-5 (4 bytes), 世=6-8 (3 bytes), 界=9-11 (3 bytes)
        let offsets = [0, 2, 6, 9, 12];
        let result = batch_utf8_to_utf16(text, &offsets);
        assert_eq!(result, vec![0, 2, 4, 5, 6]); // 🌍 = 2 UTF-16 units
    }

    #[test]
    fn test_batch_utf8_to_utf16_works_with_js_slice() {
        // This test verifies that the conversion produces indices
        // that would work correctly with JavaScript's String.slice()
        let text = "hello🌍world";

        // In JS: "hello🌍world".slice(0, 5) === "hello"
        // In JS: "hello🌍world".slice(5, 7) === "🌍" (emoji is 2 UTF-16 code units)
        // In JS: "hello🌍world".slice(7, 12) === "world"
        let offsets = [0, 5, 9, 14];
        let result = batch_utf8_to_utf16(text, &offsets);
        assert_eq!(result, vec![0, 5, 7, 12]);
    }

    #[test]
    fn test_batch_utf8_to_utf16_empty() {
        let text = "hello";
        let offsets: [usize; 0] = [];
        let result = batch_utf8_to_utf16(text, &offsets);
        assert!(result.is_empty());
    }

    // Integration tests that require a grammar - only available after grammar generation
    #[cfg(feature = "integration-tests")]
    mod integration {
        use super::super::*;

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

    /// Test Styx grammar - verifies pattern_index is correct for deduplication
    mod styx_tests {
        use super::super::*;

        fn print_spans(spans: &[Utf8Span], source: &str) {
            eprintln!("\n=== All spans ===");
            for span in spans {
                let text = &source[span.start as usize..span.end as usize];
                eprintln!(
                    "  [{:3}-{:3}] pattern={:2} capture={:20} text={:?}",
                    span.start, span.end, span.pattern_index, span.capture, text
                );
            }
            eprintln!();
        }

        #[test]
        fn test_styx_doc_comment() {
            let config = HighlightConfig::new(
                arborium_styx::language(),
                arborium_styx::HIGHLIGHTS_QUERY,
                arborium_styx::INJECTIONS_QUERY,
                arborium_styx::LOCALS_QUERY,
            )
            .expect("failed to create config");

            let mut runtime = PluginRuntime::new(config);
            let session = runtime.create_session();

            let source = "/// this is a doc comment\n";
            runtime.set_text(session, source);
            let result = runtime.parse(session).expect("parse failed");

            print_spans(&result.spans, source);

            // Should have a comment span covering the whole doc comment
            let comment_spans: Vec<_> = result
                .spans
                .iter()
                .filter(|s| s.capture.contains("comment"))
                .collect();

            assert!(
                !comment_spans.is_empty(),
                "Should have at least one comment span, got: {:?}",
                result.spans
            );

            // The comment span should cover "/// this is a doc comment"
            let comment = &comment_spans[0];
            let comment_text = &source[comment.start as usize..comment.end as usize];
            assert!(
                comment_text.contains("///") && comment_text.contains("this"),
                "Comment span should cover both '///' and text, got: {:?}",
                comment_text
            );

            runtime.free_session(session);
        }

        #[test]
        fn test_styx_key_value_pattern_index() {
            let config = HighlightConfig::new(
                arborium_styx::language(),
                arborium_styx::HIGHLIGHTS_QUERY,
                arborium_styx::INJECTIONS_QUERY,
                arborium_styx::LOCALS_QUERY,
            )
            .expect("failed to create config");

            let mut runtime = PluginRuntime::new(config);
            let session = runtime.create_session();

            let source = "name value\n";
            runtime.set_text(session, source);
            let result = runtime.parse(session).expect("parse failed");

            print_spans(&result.spans, source);

            // Find spans for "name" (the key)
            let name_spans: Vec<_> = result
                .spans
                .iter()
                .filter(|s| {
                    let text = &source[s.start as usize..s.end as usize];
                    text == "name"
                })
                .collect();

            eprintln!("Spans for 'name': {:?}", name_spans);

            // Should have both @string and @property for "name"
            let string_span = name_spans.iter().find(|s| s.capture == "string");
            let property_span = name_spans.iter().find(|s| s.capture == "property");

            assert!(string_span.is_some(), "Should have @string span for 'name'");
            assert!(
                property_span.is_some(),
                "Should have @property span for 'name'"
            );

            let string_idx = string_span.unwrap().pattern_index;
            let property_idx = property_span.unwrap().pattern_index;

            eprintln!(
                "@string pattern_index: {}, @property pattern_index: {}",
                string_idx, property_idx
            );

            // @property should have HIGHER pattern_index than @string
            // because it comes later in highlights.scm
            assert!(
                property_idx > string_idx,
                "@property (pattern_index={}) should be > @string (pattern_index={}) for deduplication to work correctly",
                property_idx,
                string_idx
            );

            runtime.free_session(session);
        }
    }
}
