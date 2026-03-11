# arborium-example-json

This is a full example of an Arborium-compatible grammar crate built entirely from the outside.

It does not use Arborium's in-repo grammar generator. Instead, it packages a vendored tree-sitter parser payload and exports the small surface Arborium needs:

- `language()`
- `HIGHLIGHTS_QUERY`
- `INJECTIONS_QUERY`
- `LOCALS_QUERY`

## Files

```text
examples/arborium-example-json/
├── Cargo.toml
├── build.rs
├── src/lib.rs
├── grammar/src/parser.c
├── grammar/src/grammar.json
├── grammar/src/node-types.json
├── grammar/src/tree_sitter/*.h
├── queries/highlights.scm
└── examples/highlight_with_arborium.rs
```

## What It Shows

1. How to compile a vendored `parser.c` with `cc`.
2. How to export a `LanguageFn` and query constants.
3. How an Arborium user turns that crate into a `CompiledGrammar`.
4. How to register it in a `GrammarStore` and use `Highlighter::with_store(...)`.

## Run It

```bash
cargo run --manifest-path examples/arborium-example-json/Cargo.toml --example highlight_with_arborium
```

## Upstream Source

The parser payload and query in this example come from Arborium's vendored JSON grammar, which in turn tracks:

- Repository: <https://github.com/tree-sitter/tree-sitter-json>
- Commit: `001c28d7a29832b06b0e831ec77845553c89b56d`
- License: MIT

This example uses JSON because it is small and does not require an external scanner.
