# Development Guide

This document covers the architecture and development workflow for arborium.

## Architecture Overview

### Crate Structure

Arborium consists of several types of crates:

**Pre-group crates** (publish first):
- `tree-sitter-patched-arborium` - Patched tree-sitter core
- `tree-sitter-highlight-patched-arborium` - Patched highlighting library
- `crates/arborium-sysroot` - WASM sysroot for grammar crates
- `crates/arborium-test-harness` - Test utilities for grammars

**Grammar crates** (in `langs/group-*/*/crate/`):
- Each grammar is an independent crate (e.g., `arborium-rust`, `arborium-svelte`)
- Only depends on pre-group crates, **not on other grammar crates**
- Organized into groups: acorn, birch, cedar, fern, hazel, maple, moss, pine, sage, willow
- Each grammar crate and its corresponding WASM plugin crate (in `npm/`) are independent,
  with their own `target/` directories for maximum build parallelism.

**Post-group crates** (publish last):
- `crates/arborium` - Umbrella crate with feature flags for all grammars

### Language Injections

Languages like HTML, Svelte, and Vue support **language injections** - embedding one
language inside another (e.g., JavaScript in `<script>` tags, CSS in `<style>` tags).

#### How Injections Work

Injection queries (in `def/queries/injections.scm`) reference other languages **by name**:

```scheme
; From svelte's injections.scm
((script_element
  (raw_text) @injection.content)
  (#set! injection.language "javascript"))

((style_element
  (raw_text) @injection.content)
 (#set! injection.language "css"))
```

The grammar itself has **no Cargo dependency** on the injected languages. The dependency
is purely nominal - it says "this region should be highlighted as javascript".

#### Injection Resolution by Platform

**Native Rust crate (`arborium`):**
- All grammars compiled into one binary via feature flags
- `Highlighter` struct has a HashMap of language configs
- Injection callback looks up languages from this HashMap
- Feature dependencies ensure required languages are included:
  ```toml
  lang-svelte = ["dep:arborium-svelte", "lang-javascript", "lang-css", "lang-typescript"]
  ```

**WASM demo:**
- Uses `arborium` compiled to WASM with `all-languages` feature
- Same as native - all grammars in one binary, injections resolved internally

**Individual npm packages (`@arborium/svelte`, etc.):**
- Each is a standalone WASM module with just that grammar
- **Cannot** resolve injections on their own
- Host application must:
  1. Load multiple grammar WASM modules
  2. Parse injection queries
  3. Route injection requests to appropriate grammar modules

### Publishing Order

Because grammar crates don't depend on each other (only on pre-group crates), they can
be published in any order after pre-group and before post-group:

```bash
# 1. Publish pre-group crates first
cargo xtask publish crates --group pre

# 2. Publish language groups (any order)
cargo xtask publish crates --group acorn
cargo xtask publish crates --group birch
# ... etc

# 3. Publish post-group crates last
cargo xtask publish crates --group post

# 4. Publish npm packages (no ordering constraints)
cargo xtask publish npm
```

## Development Workflow

### Adding a New Grammar

1. Create the grammar definition in `langs/group-<name>/<lang>/def/`
2. Run `cargo xtask gen` to generate crate files
3. Build or test the generated grammar crate, e.g.:
   `cargo check --manifest-path langs/group-<name>/<lang>/crate/Cargo.toml`
4. Run `cargo xtask build <lang>` to build the WASM plugin
5. Test with `cargo xtask serve`

### Modifying xtask

After modifying xtask code, the next `cargo xtask` invocation will recompile automatically.

Commands show "next steps" hints after completion to guide the workflow.

## Directory Layout

```
arborium/
├── crates/
│   ├── arborium/           # Umbrella crate
│   ├── arborium-sysroot/   # WASM sysroot
│   └── arborium-test-harness/
├── langs/
│   ├── group-acorn/        # Web languages (html, css, js, json, etc.)
│   ├── group-birch/        # Systems languages (c, cpp, rust, go, zig)
│   ├── group-cedar/        # JVM languages (java, scala, kotlin, clojure)
│   ├── group-fern/         # Functional languages (haskell, ocaml, elixir)
│   ├── group-hazel/        # Scripting languages (python, ruby, lua, bash)
│   ├── group-maple/        # Config/data languages (toml, yaml, json, etc.)
│   ├── group-moss/         # Scientific languages (r, julia, matlab)
│   ├── group-pine/         # Misc modern languages (swift, dart, rescript)
│   ├── group-sage/         # Legacy/enterprise (c-sharp, vb, elisp)
│   └── group-willow/       # Markup/templating (markdown, svelte, vue)
├── tree-sitter/            # Patched tree-sitter
├── tree-sitter-highlight/  # Patched highlighting
├── demo/                   # WASM demo site
└── xtask/                  # Build tooling
```

## Grammar Repository Structure

```
arborium/
├── crates/
│   └── arborium-{lang}/         # Per-language grammar crates
│       ├── arborium.kdl         ← SOURCE OF TRUTH (committed)
│       ├── grammar/
│       │   ├── grammar.js       ← tree-sitter grammar (committed)
│       │   ├── scanner.c        ← custom scanner if any (committed)
│       │   └── src/             ← GENERATED (gitignored)
│       ├── queries/
│       │   └── highlights.scm   ← highlight queries (committed)
│       ├── samples/             ← test samples (committed)
│       ├── Cargo.toml           ← GENERATED (gitignored)
│       ├── build.rs             ← GENERATED (gitignored)
│       └── src/lib.rs           ← GENERATED (gitignored)
├── demo/                        # WASM demo website
├── xtask/                       # Build tooling
└── Cargo.toml                   # Workspace root
```

### What's in Git vs Generated

| Location | In Git | Notes |
|----------|--------|-------|
| `arborium.kdl` | ✅ | Source of truth for grammar config |
| `grammar/grammar.js` | ✅ | Tree-sitter grammar definition |
| `grammar/scanner.c` | ✅ | Custom scanner (if any) |
| `queries/*.scm` | ✅ | Highlight/injection queries |
| `samples/*` | ✅ | Test samples |
| `Cargo.toml` | ❌ | Generated by `xtask gen` |
| `build.rs` | ❌ | Generated by `xtask gen` |
| `src/lib.rs` | ❌ | Generated by `xtask gen` |
| `grammar/src/*` | ❌ | Generated by `xtask gen` (tree-sitter) |

### Key Commands

```bash
# Regenerate all grammar crates (local dev)
cargo xtask gen

# Regenerate with specific version (for releases)
cargo xtask gen --version 1.1.11

# Regenerate specific grammar only
cargo xtask gen rust

# Build and serve WASM demo
cargo xtask serve --dev

# Build WASM plugins
cargo xtask plugins build
```

### Local Development Workflow

```bash
# 1. Edit grammar source files
#    - arborium.kdl (config, license, metadata)
#    - grammar/grammar.js (tree-sitter grammar)
#    - queries/highlights.scm (syntax highlighting)

# 2. Regenerate crate files
cargo xtask gen

# 3. Build and test
cargo build
cargo test -p arborium-rust
```

### Version Management

**Versions don't matter locally** - path dependencies ignore version numbers.

For releases, CI parses the version from the git tag and runs:
```bash
cargo xtask gen --version $VERSION
```

This updates all `Cargo.toml` files with the correct version before publishing.

See [PUBLISH.md](PUBLISH.md) for full release workflow details.

### arborium.kdl Format

Each grammar crate has an `arborium.kdl` file as its source of truth:

```kdl
repo "https://github.com/tree-sitter/tree-sitter-rust"
commit "abc123..."
license "MIT"

grammar {
    id "rust"
    name "Rust"
    tag "code"
    tier 1
    icon "devicon-plain:rust"
    aliases "rs"
    has-scanner #true
    generate-plugin #true

    sample {
        path "samples/example.rs"
        description "Example code"
        license "MIT"
    }
}
```

**Key fields:**
- `license` - SPDX license for the grammar (used in generated Cargo.toml)
- `generate-plugin #true` - Include in WASM plugin builds
- `has-scanner #true` - Grammar has external scanner (scanner.c)
- `tier` - 1-5, affects default feature inclusion
