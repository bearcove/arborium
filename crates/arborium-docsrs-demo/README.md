# arborium-docsrs-demo

This crate demonstrates [arborium](https://github.com/bearcove/arborium) syntax
highlighting on docs.rs.

docs.rs highlights Rust code natively, but leaves other languages (TOML, shell,
JSON, YAML, SQL, etc.) unhighlighted. This crate uses arborium's IIFE script
via `--html-in-header` to highlight everything else.

## How it works

1. Create `arborium-header.html`:
   ```html
   <script defer src="https://cdn.jsdelivr.net/npm/@arborium/arborium@1.2.3/dist/arborium.iife.js"></script>
   ```

2. Add to `Cargo.toml`:
   ```toml
   [package.metadata.docs.rs]
   rustdoc-args = ["--html-in-header", "arborium-header.html"]
   ```

That's it! The IIFE automatically:
- Detects code blocks by `class="language-*"`
- Skips Rust blocks (they have semantic `<a>` links from rustdoc)
- Highlights everything else with tree-sitter grammars

## See the demo

Visit the [module documentation](https://docs.rs/arborium-docsrs-demo) to see
highlighted TOML, shell, JSON, YAML, SQL, and more.