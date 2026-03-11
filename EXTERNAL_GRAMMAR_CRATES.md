# External Arborium Grammar Crates

This guide is for packaging a grammar crate that works with Arborium without adding the language to Arborium itself.

If you want to add a language to this repository's generated built-in registry, use [ADDING_GRAMMARS.md](ADDING_GRAMMARS.md) instead.

## What Arborium Needs

Arborium does not require a special provider trait or code generator for third-party grammars. A crate only needs to expose the same pieces Arborium's generated grammar crates expose:

- `language() -> tree_sitter_language::LanguageFn`
- `HIGHLIGHTS_QUERY: &str`
- `INJECTIONS_QUERY: &str`
- `LOCALS_QUERY: &str`

From there, an Arborium user compiles that into a `CompiledGrammar` and registers it in a `GrammarStore`.

```rust
use std::sync::Arc;

use arborium::advanced::{CompiledGrammar, GrammarConfig};
use arborium::GrammarStore;

let grammar = Arc::new(CompiledGrammar::new(GrammarConfig {
    language: arborium_mylanguage::language().into(),
    highlights_query: arborium_mylanguage::HIGHLIGHTS_QUERY,
    injections_query: arborium_mylanguage::INJECTIONS_QUERY,
    locals_query: arborium_mylanguage::LOCALS_QUERY,
})?);

let store = GrammarStore::new();
store.insert("mylanguage", grammar);
```

## Minimal Crate Layout

```text
arborium-mylanguage/
├── Cargo.toml
├── build.rs
├── src/
│   └── lib.rs
├── grammar/
│   └── src/
│       ├── parser.c
│       ├── grammar.json
│       ├── node-types.json
│       └── tree_sitter/
│           ├── alloc.h
│           ├── array.h
│           └── parser.h
└── queries/
    ├── highlights.scm
    ├── injections.scm      # optional
    └── locals.scm          # optional
```

Notes:

- `parser.c` is the only required generated C source for grammars without an external scanner.
- If your grammar has an external scanner, also ship `grammar/scanner.c` and compile it from `build.rs`.
- `grammar.json` and `node-types.json` are not required at runtime, but they are useful to keep in the crate because they describe the parser you shipped.
- You can generate `parser.c` with the tree-sitter CLI before publishing, then ship the generated C output in the crate. Consumers do not need the CLI.

## Cargo.toml

```toml
[package]
name = "arborium-mylanguage"
version = "0.1.0"
edition = "2024"
rust-version = "1.85"

[lib]
path = "src/lib.rs"

[dependencies]
tree-sitter-language = "0.1"

[build-dependencies]
cc = "1"
```

If you want to keep an in-crate usage example that registers the grammar into Arborium, add:

```toml
arborium = { version = "2", default-features = false }
```

## build.rs

Compile the vendored parser exactly once as C code:

```rust
fn main() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src_dir = manifest_dir.join("grammar/src");
    let grammar_dir = manifest_dir.join("grammar");

    println!("cargo:rerun-if-changed={}", src_dir.join("parser.c").display());

    let mut build = cc::Build::new();
    build
        .include(&src_dir)
        .include(src_dir.join("tree_sitter"))
        .warnings(false)
        .file(src_dir.join("parser.c"));

    let scanner = grammar_dir.join("scanner.c");
    if scanner.exists() {
        println!("cargo:rerun-if-changed={}", scanner.display());
        build.include(&grammar_dir).file(scanner);
    }

    build.compile("tree_sitter_mylanguage");
}
```

For grammars with a C++ scanner, switch the `cc` builder to C++ mode and compile `scanner.cc`.

## src/lib.rs

Export the tree-sitter language symbol and the queries:

```rust
use tree_sitter_language::LanguageFn;

unsafe extern "C" {
    fn tree_sitter_mylanguage() -> *const ();
}

pub const fn language() -> LanguageFn {
    unsafe { LanguageFn::from_raw(tree_sitter_mylanguage) }
}

pub const HIGHLIGHTS_QUERY: &str = include_str!("../queries/highlights.scm");
pub const INJECTIONS_QUERY: &str = include_str!("../queries/injections.scm");
pub const LOCALS_QUERY: &str = include_str!("../queries/locals.scm");
```

If your grammar has no injections or locals query, use `""`.

## Registering It With Arborium

```rust
use std::sync::Arc;

use arborium::advanced::{CompiledGrammar, GrammarConfig};
use arborium::{GrammarStore, Highlighter};

let grammar = Arc::new(CompiledGrammar::new(GrammarConfig {
    language: arborium_mylanguage::language().into(),
    highlights_query: arborium_mylanguage::HIGHLIGHTS_QUERY,
    injections_query: arborium_mylanguage::INJECTIONS_QUERY,
    locals_query: arborium_mylanguage::LOCALS_QUERY,
})?);

let store = Arc::new(GrammarStore::new());
store.insert("mylanguage", grammar);

let mut hl = Highlighter::with_store(store);
let html = hl.highlight("mylanguage", "your source here")?;
```

`GrammarStore::insert` uses the same normalization rules as `get`. Unknown names are stored exactly as provided. Built-in aliases normalize to their canonical slot, so inserting `"js"` targets `"javascript"`.

## Full Example

See [examples/arborium-example-json](examples/arborium-example-json/README.md) for a complete standalone crate that:

- vendors a generated parser payload
- exports Arborium-compatible symbols
- compiles with `cc`
- registers itself into a `GrammarStore`
- highlights a JSON sample through `Highlighter::with_store(...)`

Run it with:

```bash
cargo run --manifest-path examples/arborium-example-json/Cargo.toml --example highlight_with_arborium
```
