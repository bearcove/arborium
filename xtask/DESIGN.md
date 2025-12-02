# xtask Design

## Dependencies

| Purpose | Crate |
|---------|-------|
| Brotli compression | `brotli` |
| Gzip compression | `flate2` |
| HTTP requests (icons) | `reqwest` (blocking) |
| Templating | `minijinja` |
| KDL parsing | `facet-kdl` (git) |
| JSON serialization | `facet-json` (git) |
| WASM parsing | `wasmparser` |
| Parallelism | `rayon` |
| Logging | `tracing` + `tracing-subscriber` |
| Colored output | `owo-colors` |
| Drawing boxes | `boxen` |
| Fancy diagnostics (with spans) | `miette` |
| Everyday errors | `rootcause` |

All format parsing uses the facet ecosystem. `facet-kdl` provides `Spanned<T>` 
wrappers to preserve source locations, enabling precise Miette diagnostics:

```rust
#[derive(Facet)]
struct GrammarConfig {
    id: Spanned<String>,
    tier: Option<Spanned<u8>>,
}

// Lint error with source span:
// error: tier must be between 1 and 5
//   ┌─ crates/arborium-foo/arborium.kdl:5:10
//   │
// 5 │     tier 99
//   │          ^^ invalid tier
```

## Target State

```
arborium/
├── crates/
│   ├── arborium/             # main crate, re-exports all grammars
│   │   ├── Cargo.toml
│   │   └── src/lib.rs
│   ├── arborium-rust/        # one crate per grammar
│   │   ├── arborium.kdl      # single source of truth
│   │   ├── Cargo.toml        # generated
│   │   ├── build.rs          # generated
│   │   ├── src/lib.rs        # generated
│   │   ├── grammar-src/      # vendored from upstream
│   │   ├── queries/          # highlight queries (local to this grammar)
│   │   └── samples/          # example files
│   ├── arborium-javascript/
│   ├── arborium-haskell/
│   └── ...
└── xtask/
```

**`arborium.kdl`** contains everything:
- Source: repo, commit, license, authors
- Metadata: name, icon, aliases, tier, tag
- Build config: has-scanner, c-symbol, grammar-path
- Query inheritance: which grammars' queries to prepend
- Description, trivia, links
- Sample definitions

No `grammars/` directory. No top-level `GRAMMARS.toml`. No `info.toml`.

Vendoring new grammars and checking for updates is a manual process.

## arborium.kdl Schema

### Simple grammar (most common)

```kdl
repo "https://github.com/tree-sitter/tree-sitter-rust"
commit "261b20226c04ef601adbdf185a800512a5f66291"
license "MIT"
authors "Maxim Sokolov"

grammar {
    id "rust"
    name "Rust"
    tag "code"
    tier 1
    icon "devicon-plain:rust"
    aliases "rs"
    
    has-scanner true
    c-symbol "rust_orchard"  // generates tree_sitter_rust_orchard()
    
    inventor "Graydon Hoare"
    year 2010
    description "Systems language focused on safety and performance without GC"
    link "https://en.wikipedia.org/wiki/Rust_(programming_language)"
    trivia "Hoare began Rust as a side project at Mozilla in 2006"
    
    sample {
        path "samples/example.rs"
        description "Clippy lint implementation"
        link "https://github.com/rust-lang/rust/blob/main/..."
        license "MIT OR Apache-2.0"
    }
}
```

### Multi-grammar crate (e.g., tree-sitter-xml exports XML and DTD)

```kdl
repo "https://github.com/tree-sitter-grammars/tree-sitter-xml"
commit "863dbc381f44f6c136a399e684383b977bb2beaa"
license "MIT"
authors "ObserverOfTime"

grammar {
    id "xml"
    name "XML"
    tag "markup"
    tier 3
    icon "devicon-plain:xml"
    aliases "xsl" "xslt" "svg"
    
    has-scanner true
    grammar-path "xml"  // subdirectory within repo
    
    // ...metadata, samples...
}

grammar {
    id "dtd"
    name "DTD"
    tag "markup"
    tier 3
    
    has-scanner true
    grammar-path "dtd"
    
    // ...metadata, samples...
}
```

### Query inheritance (e.g., TypeScript extends JavaScript)

Languages like TypeScript, TSX, Svelte need to prepend highlight queries from parent
languages. Only highlights need inheritance - injections are orthogonal (each grammar
declares where *other* languages appear within it), and locals we don't bother with.

```kdl
grammar {
    id "typescript"
    name "TypeScript"
    // ...
    
    queries {
        highlights {
            prepend crate="arborium-javascript"
        }
    }
}
```

The `grammar=` attribute is optional when the crate has exactly one grammar.
If the crate has multiple grammars and `grammar=` is not specified, generation fails
with an error like: "arborium-xml has multiple grammars (xml, dtd), specify which one".

```kdl
// Explicit grammar for multi-grammar crates:
queries {
    highlights {
        prepend crate="arborium-xml" grammar="dtd"
    }
}
```

More complex example (Svelte inherits from multiple languages):

```kdl
grammar {
    id "svelte"
    name "Svelte"
    // ...
    
    queries {
        highlights {
            prepend crate="arborium-javascript"
            prepend crate="arborium-css"
            prepend crate="arborium-html"
        }
    }
}
```

**How query inheritance works:**

1. Every grammar crate uses `links = "arborium-{id}"` in Cargo.toml
2. Every build.rs emits `cargo:queries-dir={manifest_dir}/queries`
3. Dependent crates read `DEP_ARBORIUM_{ID}_QUERIES_DIR` in their build.rs
4. Build.rs concatenates prepended queries + local query → writes to OUT_DIR
5. lib.rs does `include_str!(concat!(env!("OUT_DIR"), "/highlights.scm"))`

The `prepend` entries add the crate as a dependency in generated Cargo.toml.
Order matters: first listed = first in output file.

## Startup: Registry Loading & Linting

Every xtask invocation starts the same way:

1. Find workspace root (walk up looking for `Cargo.toml` with `[workspace]`)
2. Crawl `crates/` directory, find all `arborium-*/` directories
3. Parse every `arborium.kdl` into a registry
4. Lint the entire registry:
   - Missing required fields in `arborium.kdl`
   - Sample files that don't exist on disk
   - Sample files that are empty or just comments
   - Highlight queries referencing invalid node types
   - Missing `grammar-src/` or required files
5. Report diagnostics (warnings and errors) with Miette
6. If any errors: refuse to continue, exit non-zero
7. If only warnings: continue with the command

## Commands

### `cargo xtask generate`

Regenerate all crate code from `arborium.kdl` files.

```
cargo xtask generate [--dry-run]
```

Uses a two-phase approach: plan, then execute.

**Phase 1: Plan**

Compute all file writes without touching disk:

```rust
struct GeneratePlan {
    files: Vec<PlannedWrite>,
}

struct PlannedWrite {
    path: Utf8PathBuf,
    content: String,
    changed: bool,  // compared against current disk content
}
```

For each grammar in registry:
- Generate `Cargo.toml` from template
- Generate `build.rs` from template
- Generate `src/lib.rs` from template

Also generate main `crates/arborium/src/lib.rs` (re-exports all grammars).

**Phase 2: Report**

Show what would be written:

```
  unchanged crates/arborium-rust/Cargo.toml
  unchanged crates/arborium-rust/build.rs
  write     crates/arborium-rust/src/lib.rs
  ...
```

**Phase 3: Execute**

If `--dry-run` is passed, exit here. Otherwise, write only the changed files.

This preserves mtimes on unchanged files, avoiding unnecessary Cargo rebuilds.

### `cargo xtask serve`

Build and serve the WASM demo locally.

1. Run `generate`
2. Build all grammars for WASM target
3. Generate `registry.json` from registry (using facet-json)
4. Serve static files + registry.json + WASM binaries

**Demo architecture:**

The demo is a static site with minimal generation:

```
demo/
├── index.html      # static, checked in
├── demo.js         # static, checked in
├── demo.css        # static, checked in
├── registry.json   # generated from CrateRegistry
└── wasm/           # generated WASM binaries
    ├── arborium-rust.wasm
    ├── arborium-javascript.wasm
    └── ...
```

`registry.json` contains all grammar metadata plus inlined sample content:

```json
{
  "grammars": [
    {
      "id": "rust",
      "crate": "arborium-rust",
      "name": "Rust",
      "icon": "devicon-plain:rust",
      "tier": 1,
      "tag": "code",
      "description": "Systems language focused on...",
      "samples": [
        {
          "path": "samples/example.rs",
          "description": "Clippy lint implementation",
          "content": "fn main() { ... }"
        }
      ]
    }
  ]
}
```

Sample content is inlined because:
- Code compresses extremely well (brotli gets ~10-15x on source code)
- ~90 grammars × ~5KB average = ~450KB uncompressed → ~30-50KB compressed
- Simpler than lazy loading (no fetch waterfall, works offline, no CORS issues)

The static HTML/JS/CSS are checked into the repo and not generated. The JS
fetches `registry.json` on load and builds the UI dynamically.

**Additional demo build steps:**

1. **Check WASM env imports** - Run `wasm-objdump` on the built WASM to detect
   `env.*` imports that won't work in browsers. Fail early before slow optimizations.

2. **Pre-compress files** - Generate `.br` (brotli) and `.gz` (gzip) versions of
   large files (WASM, JS, HTML). The dev server serves these with appropriate
   `Content-Encoding` headers.

3. **Fetch icons** - Download SVG icons from Iconify API based on `icon` fields
   in the registry. Cache in `.icon-cache.json` to avoid repeated fetches.

4. **Generate theme CSS** - Generate CSS from `arborium::theme::builtin` themes
   for the demo's theme switcher.

### `cargo xtask lint`

Run all lints without generating anything.

1. Load and validate registry (same as startup)
2. **Sample validation:**
   - Sample files exist on disk
   - Sample files are not empty
   - Sample files are not HTTP error pages (failed downloads)
   - Sample files have reasonable length (warn if < 25 lines)
3. **Highlight validation:**
   - Run highlighting on each sample with its grammar
   - Warn if zero highlights produced (query might be broken)
   - Report query parse errors

## Cleanup (after migration complete)

Once all grammars have `arborium.kdl` and everything works:

1. **Delete legacy files:**
   - Remove all `crates/arborium-*/info.toml`
   - Remove top-level `GRAMMARS.toml`
   - Remove `grammars/` directory entirely
   - Remove `grammar-crate-config.toml` files

2. **Delete old xtask code:**
   - `vendor_grammar.rs` - manual vendoring now
   - `tiering.rs` - no more npm dependency ordering
   - `config.rs` - info.toml parsing
   - Old command handlers

3. **Update documentation:**
   - README: explain new structure
   - README: how to add a new grammar (create `arborium.kdl`, vendor sources)
   - README: how to update a grammar (bump commit, re-vendor)

4. **Remove dead dependencies from xtask:**
   - `toml` (was for info.toml/GRAMMARS.toml parsing)
   - `regex` (was for template substitution)
   - Any npm/node-related tooling
