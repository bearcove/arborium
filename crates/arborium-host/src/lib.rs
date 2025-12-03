//! Arborium Host Component
//!
//! This WASM component orchestrates grammar plugins for syntax highlighting.
//! It manages documents, coordinates parsing across multiple languages,
//! and resolves language injections recursively.
//!
//! The host imports a `plugin-provider` interface from JS that allows it
//! to load and interact with grammar plugins. This design allows the host
//! to remain a pure WASM component while JS handles the actual plugin
//! instantiation.

use std::cell::RefCell;
use std::collections::BTreeMap;

wit_bindgen::generate!({
    world: "arborium-host",
    path: "../../wit/host.wit",
});

use crate::arborium::host::plugin_provider::{self, PluginHandle, PluginSession};
use crate::arborium::host::types::Edit;
use crate::exports::arborium::host::host::{Document, Guest, HighlightResult, HighlightSpan};

/// Global state for the host.
struct HostState {
    /// Next document ID to assign.
    next_doc_id: u32,
    /// Active documents.
    documents: BTreeMap<u32, DocumentState>,
    /// Cached plugin handles by language.
    plugins: BTreeMap<String, PluginHandle>,
}

/// State for a single document.
struct DocumentState {
    /// Primary language of this document.
    language: String,
    /// Plugin handle for the primary language.
    plugin: PluginHandle,
    /// Session on the primary plugin.
    session: PluginSession,
    /// Current text content.
    text: String,
    /// Child sessions for injected languages.
    injection_sessions: BTreeMap<String, (PluginHandle, PluginSession)>,
}

impl Default for HostState {
    fn default() -> Self {
        Self {
            next_doc_id: 1,
            documents: BTreeMap::new(),
            plugins: BTreeMap::new(),
        }
    }
}

thread_local! {
    static STATE: RefCell<HostState> = RefCell::new(HostState::default());
}

/// Get or load a plugin for a language.
/// This must be called outside of with_state to avoid borrow conflicts.
fn get_or_load_plugin(language: &str) -> Option<PluginHandle> {
    // First check if we already have it cached
    let cached = STATE.with(|state| state.borrow().plugins.get(language).copied());

    if let Some(handle) = cached {
        return Some(handle);
    }

    // Ask JS to load the plugin (outside of borrow)
    if let Some(handle) = plugin_provider::load_plugin(language) {
        STATE.with(|state| {
            state.borrow_mut().plugins.insert(language.into(), handle);
        });
        Some(handle)
    } else {
        None
    }
}

struct HostImpl;

impl Guest for HostImpl {
    fn create_document(language: String) -> Option<Document> {
        let plugin = get_or_load_plugin(&language)?;
        let session = plugin_provider::create_plugin_session(plugin);

        STATE.with(|state| {
            let mut state = state.borrow_mut();
            let doc_id = state.next_doc_id;
            state.next_doc_id += 1;

            state.documents.insert(
                doc_id,
                DocumentState {
                    language,
                    plugin,
                    session,
                    text: String::new(),
                    injection_sessions: BTreeMap::new(),
                },
            );

            Some(doc_id)
        })
    }

    fn free_document(doc: Document) {
        let doc_state = STATE.with(|state| state.borrow_mut().documents.remove(&doc));

        if let Some(doc_state) = doc_state {
            // Free injection sessions
            for (_, (plugin, session)) in doc_state.injection_sessions {
                plugin_provider::free_plugin_session(plugin, session);
            }
            // Free main session
            plugin_provider::free_plugin_session(doc_state.plugin, doc_state.session);
        }
    }

    fn set_text(doc: Document, text: String) {
        let info = STATE.with(|state| {
            let mut state = state.borrow_mut();
            if let Some(doc_state) = state.documents.get_mut(&doc) {
                doc_state.text = text.clone();
                Some((doc_state.plugin, doc_state.session))
            } else {
                None
            }
        });

        if let Some((plugin, session)) = info {
            plugin_provider::plugin_set_text(plugin, session, &text);
        }
    }

    fn apply_edit(doc: Document, text: String, edit: Edit) {
        let info = STATE.with(|state| {
            let mut state = state.borrow_mut();
            if let Some(doc_state) = state.documents.get_mut(&doc) {
                doc_state.text = text.clone();
                Some((doc_state.plugin, doc_state.session))
            } else {
                None
            }
        });

        if let Some((plugin, session)) = info {
            plugin_provider::plugin_apply_edit(plugin, session, &text, edit);
        }
    }

    fn highlight(doc: Document, max_depth: u32) -> HighlightResult {
        // Get document info
        let doc_info = STATE.with(|state| {
            let state = state.borrow();
            state.documents.get(&doc).map(|ds| {
                (
                    ds.plugin,
                    ds.session,
                    ds.language.clone(),
                    ds.text.clone(),
                )
            })
        });

        let Some((plugin, session, language, text)) = doc_info else {
            return HighlightResult { spans: Vec::new() };
        };

        let mut all_spans = Vec::new();

        // Parse the main document
        let result = plugin_provider::plugin_parse(plugin, session);

        let Ok(parse_result) = result else {
            return HighlightResult { spans: Vec::new() };
        };

        // Add spans from the main language
        for span in parse_result.spans {
            all_spans.push(HighlightSpan {
                start: span.start,
                end: span.end,
                capture: span.capture,
                language: language.clone(),
            });
        }

        // Process injections recursively
        if max_depth > 0 {
            for injection in parse_result.injections {
                if let Some(injection_spans) = process_injection(
                    doc,
                    &text,
                    &injection.language,
                    injection.start,
                    injection.end,
                    max_depth - 1,
                ) {
                    all_spans.extend(injection_spans);
                }
            }
        }

        // Sort spans by position
        all_spans.sort_by(|a, b| a.start.cmp(&b.start).then_with(|| b.end.cmp(&a.end)));

        HighlightResult { spans: all_spans }
    }

    fn cancel(doc: Document) {
        // Collect all sessions to cancel
        let sessions = STATE.with(|state| {
            let state = state.borrow();
            state.documents.get(&doc).map(|doc_state| {
                let mut sessions = vec![(doc_state.plugin, doc_state.session)];
                for (_, (plugin, session)) in &doc_state.injection_sessions {
                    sessions.push((*plugin, *session));
                }
                sessions
            })
        });

        if let Some(sessions) = sessions {
            for (plugin, session) in sessions {
                plugin_provider::plugin_cancel(plugin, session);
            }
        }
    }

    fn get_required_languages(doc: Document) -> Vec<String> {
        let info = STATE.with(|state| {
            let state = state.borrow();
            state
                .documents
                .get(&doc)
                .map(|ds| (ds.language.clone(), ds.plugin))
        });

        let Some((language, plugin)) = info else {
            return Vec::new();
        };

        let mut languages = vec![language];

        // Get injection languages from the plugin
        let injection_langs = plugin_provider::get_injection_languages(plugin);
        languages.extend(injection_langs);

        languages
    }
}

/// Process an injection and return highlight spans.
fn process_injection(
    doc: Document,
    text: &str,
    language: &str,
    start: u32,
    end: u32,
    remaining_depth: u32,
) -> Option<Vec<HighlightSpan>> {
    // Check if we already have a session for this injection language
    let existing_session = STATE.with(|state| {
        let state = state.borrow();
        state
            .documents
            .get(&doc)
            .and_then(|ds| ds.injection_sessions.get(language).copied())
    });

    let (plugin, session) = if let Some((p, s)) = existing_session {
        (p, s)
    } else {
        // Load the plugin and create a session
        let plugin = get_or_load_plugin(language)?;
        let session = plugin_provider::create_plugin_session(plugin);

        // Store the session
        STATE.with(|state| {
            let mut state = state.borrow_mut();
            if let Some(ds) = state.documents.get_mut(&doc) {
                ds.injection_sessions
                    .insert(language.into(), (plugin, session));
            }
        });

        (plugin, session)
    };

    // Extract the injected text
    let start_idx = start as usize;
    let end_idx = end as usize;
    if end_idx > text.len() || start_idx > end_idx {
        return None;
    }
    let injected_text: String = text[start_idx..end_idx].into();

    // Parse the injection
    plugin_provider::plugin_set_text(plugin, session, &injected_text);
    let result = plugin_provider::plugin_parse(plugin, session).ok()?;

    let mut spans = Vec::new();

    // Add spans with offset adjustment
    for span in result.spans {
        spans.push(HighlightSpan {
            start: span.start + start,
            end: span.end + start,
            capture: span.capture,
            language: language.into(),
        });
    }

    // Process nested injections if we have depth remaining
    if remaining_depth > 0 {
        for nested in result.injections {
            // The nested injection offsets are relative to the injected text,
            // so we need to add the parent injection's start offset
            if let Some(nested_spans) = process_injection(
                doc,
                text,
                &nested.language,
                nested.start + start,
                nested.end + start,
                remaining_depth - 1,
            ) {
                spans.extend(nested_spans);
            }
        }
    }

    Some(spans)
}

export!(HostImpl);
