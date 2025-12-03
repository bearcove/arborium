# xtask Design

## Status

### Completed

- **KDL migration**: All ~100 grammars converted from `info.toml` to `arborium.kdl`
- **Legacy cleanup**: Deleted all `info.toml` files, removed orphan directories
- **Lint command**: `cargo xtask lint` validates entire registry
- **Generate command**: `cargo xtask gen [--dry-run]` regenerates Cargo.toml, build.rs, src/lib.rs
- **Plan-execute pattern**: Two-phase approach with colored diff display and optional dry-run
- **KDL 2.0 syntax**: Fixed all files to use `#true`/`#false` and snake_case identifiers
- **Vendored grammars**: asciidoc, asciidoc_inline, markdown, markdown-inline added
- **Test harness**: `arborium-test-harness` reads samples from arborium.kdl and verifies highlighting works
- **Internal grammars**: `internal #true` flag to skip metadata linting for grammars like asciidoc_inline

### Not Started

- serve command (WASM build + static site)
- Query inheritance implementation
- Final cleanup (remove old xtask code, update README)

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
- Build config: has_scanner, c_symbol, grammar_path
- Query inheritance: which grammars' queries to prepend
- Description, trivia, links
- Sample definitions

No `grammars/` directory. No top-level `GRAMMARS.toml`. No `info.toml`.

Vendoring new grammars and checking for updates is a manual process.

## arborium.kdl Schema

**Important:** This uses KDL 2.0 syntax:
- Booleans are `#true` and `#false` (not bare `true`/`false`)
- Identifiers use snake_case (no hyphens)

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
    
    has_scanner #true
    c_symbol "rust_orchard"  // generates tree_sitter_rust_orchard()
    
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
    
    has_scanner #true
    grammar_path "xml"  // subdirectory within repo
    
    // ...metadata, samples...
}

grammar {
    id "dtd"
    name "DTD"
    tag "markup"
    tier 3
    
    has_scanner #true
    grammar_path "dtd"
    
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
4. Build.rs concatenates prepended queries + local query -> writes to OUT_DIR
5. lib.rs does `include_str!(concat!(env!("OUT_DIR"), "/highlights.scm"))`

The `prepend` entries add the crate as a dependency in generated Cargo.toml.
Order matters: first listed = first in output file.

## Plan-Execute Pattern

All commands that modify files use a two-phase approach:

### Phase 1: Plan

Compute all operations without touching disk:

```rust
pub enum Operation {
    CreateFile { path, content, description },
    UpdateFile { path, old_content, new_content, description },
    DeleteFile { path, description },
    CreateDir { path, description },
    RunCommand { command, args, working_dir, description },
    CopyFile { src, dest, description },
    GitClone { url, dest, commit, description },
}

pub struct Plan {
    name: String,
    operations: Vec<Operation>,
}

pub struct PlanSet {
    plans: Vec<Plan>,
}
```

### Phase 2: Display

Show what would change with colored diff output:

```
Plan: arborium-rust
  update Cargo.toml
    - version = "0.1.0"
    + version = "0.2.0"
  unchanged build.rs
  unchanged src/lib.rs

Plan: arborium-javascript
  create src/lib.rs
    + // Generated file
    + pub fn language() -> tree_sitter::Language { ... }
```

### Phase 3: Execute

If `--dry-run` is passed, exit after display. Otherwise, execute all operations.

This preserves mtimes on unchanged files, avoiding unnecessary Cargo rebuilds.

## Commands

### `cargo xtask lint`

Run all lints without generating anything.

1. Load and validate registry
2. **Sample validation:**
   - Sample files exist on disk
   - Sample files are not empty
   - Sample files have reasonable length (warn if < 25 lines)
3. **Config validation:**
   - Required fields present
   - Tier values in valid range
   - highlight.scm exists if grammar-src present

### `cargo xtask gen [--dry-run] [name]`

Regenerate crate code from `arborium.kdl` files.

For each grammar in registry:
- Generate `Cargo.toml` from template
- Generate `build.rs` from template
- Generate `src/lib.rs` from template

Optional `name` argument filters to a single crate.

### `cargo xtask serve` (TODO)

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
- ~90 grammars x ~5KB average = ~450KB uncompressed -> ~30-50KB compressed
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

## Next Steps

1. **Vendor command** - Implement `cargo xtask grammars vendor <name>` to automate
   the git clone + file copy workflow

2. **Update command** - Implement `cargo xtask grammars update` to check upstream
   repos for new commits

3. **Query inheritance** - Implement build.rs logic for prepending parent queries

4. **serve-demo** - Build WASM binaries and serve the interactive demo

5. **Final cleanup:**
   - Delete old xtask modules (vendor_grammar.rs, tiering.rs, config.rs)
   - Remove dead dependencies (toml, regex)
   - Update README with new workflow
