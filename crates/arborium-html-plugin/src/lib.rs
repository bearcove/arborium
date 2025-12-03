//! html grammar plugin for arborium.
#![allow(unsafe_op_in_unsafe_fn)]

wit_bindgen::generate!({
    world: "grammar-plugin",
    path: "../../wit/grammar.wit",
});

use arborium_plugin_runtime::{HighlightConfig, PluginRuntime};
use arborium_wire::Edit as WireEdit;
use std::cell::RefCell;

// Import the generated types
use arborium::grammar::types::{Edit, Injection, ParseError, ParseResult, Span};

thread_local! {
    static RUNTIME: RefCell<Option<PluginRuntime>> = const { RefCell::new(None) };
}

fn get_or_init_runtime() -> &'static RefCell<Option<PluginRuntime>> {
    RUNTIME.with(|r| {
        let mut runtime = r.borrow_mut();
        if runtime.is_none() {
            let config = HighlightConfig::new(
                arborium_html::language(),
                arborium_html::HIGHLIGHTS_QUERY,
                arborium_html::INJECTIONS_QUERY,
                arborium_html::LOCALS_QUERY,
            )
            .expect("failed to create highlight config");
            *runtime = Some(PluginRuntime::new(config));
        }
        // SAFETY: We're returning a reference to thread-local storage which lives
        // for the duration of the WASM instance.
        unsafe { &*(r as *const _) }
    })
}

struct PluginImpl;

impl exports::arborium::grammar::plugin::Guest for PluginImpl {
    fn language_id() -> String {
        "html".to_string()
    }

    fn injection_languages() -> Vec<String> {
        // TODO: Parse injection queries to determine which languages are injected
        Vec::new()
    }

    fn create_session() -> u32 {
        get_or_init_runtime()
            .borrow_mut()
            .as_mut()
            .expect("runtime not initialized")
            .create_session()
    }

    fn free_session(session: u32) {
        get_or_init_runtime()
            .borrow_mut()
            .as_mut()
            .expect("runtime not initialized")
            .free_session(session);
    }

    fn set_text(session: u32, text: String) {
        get_or_init_runtime()
            .borrow_mut()
            .as_mut()
            .expect("runtime not initialized")
            .set_text(session, &text);
    }

    fn apply_edit(session: u32, text: String, edit: Edit) {
        let wire_edit = WireEdit {
            start_byte: edit.start_byte,
            old_end_byte: edit.old_end_byte,
            new_end_byte: edit.new_end_byte,
            start_row: edit.start_row,
            start_col: edit.start_col,
            old_end_row: edit.old_end_row,
            old_end_col: edit.old_end_col,
            new_end_row: edit.new_end_row,
            new_end_col: edit.new_end_col,
        };
        get_or_init_runtime()
            .borrow_mut()
            .as_mut()
            .expect("runtime not initialized")
            .apply_edit(session, &text, &wire_edit);
    }

    fn parse(session: u32) -> Result<ParseResult, ParseError> {
        let result = get_or_init_runtime()
            .borrow_mut()
            .as_mut()
            .expect("runtime not initialized")
            .parse(session);

        match result {
            Ok(r) => Ok(ParseResult {
                spans: r
                    .spans
                    .into_iter()
                    .map(|s| Span {
                        start: s.start,
                        end: s.end,
                        capture: s.capture,
                    })
                    .collect(),
                injections: r
                    .injections
                    .into_iter()
                    .map(|i| Injection {
                        start: i.start,
                        end: i.end,
                        language: i.language,
                        include_children: i.include_children,
                    })
                    .collect(),
            }),
            Err(e) => Err(ParseError { message: e.message }),
        }
    }

    fn cancel(session: u32) {
        get_or_init_runtime()
            .borrow_mut()
            .as_mut()
            .expect("runtime not initialized")
            .cancel(session);
    }
}

export!(PluginImpl);
