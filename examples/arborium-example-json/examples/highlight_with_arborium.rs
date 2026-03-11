use std::sync::Arc;

use arborium::advanced::{CompiledGrammar, GrammarConfig};
use arborium::{GrammarStore, Highlighter};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let grammar = Arc::new(CompiledGrammar::new(GrammarConfig {
        language: arborium_example_json::language().into(),
        highlights_query: arborium_example_json::HIGHLIGHTS_QUERY,
        injections_query: arborium_example_json::INJECTIONS_QUERY,
        locals_query: arborium_example_json::LOCALS_QUERY,
    })?);

    let store = Arc::new(GrammarStore::new());
    store.insert("json-outside", grammar);

    let mut highlighter = Highlighter::with_store(store);
    let html = highlighter.highlight("json-outside", include_str!("sample.json"))?;
    println!("{html}");

    Ok(())
}
