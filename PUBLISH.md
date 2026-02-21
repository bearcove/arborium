# Publishing Guide


## Core Crates (always present)

```
crates/
├── arborium/                  ← main crate with inventory
├── tree-sitter/               ← fork to avoid upstream build errors
└── tree-sitter-highlight/     ← fork to avoid upstream build errors
```

- These are always in the repo (not generated) and are published every release.
- The tree-sitter forks track upstream but include fixes so CI builds reliably across targets.

## Release Flow (current CI)

Everything is generated **once**, then a single tag publishes all crates + npm packages.

### 1. Prepare a release locally (optional but recommended)

```bash
# Pick a version and generate everything (core + all groups)
# This runs tree-sitter-cli for all grammars, but results are cached
# by tree-sitter-cli version + grammar.js hash, so reruns are cheap.

cargo xtask gen --version 0.3.0
```

`xtask gen` also records the current release version in a small metadata file.
CI will regenerate from source of truth as well, but running this locally lets
you review changes before tagging.

### 2. Tag and push (single tag)

```bash
git commit -am "Release v0.3.0"
git tag v0.3.0
git push origin main --tags
```

The `v0.3.0` tag triggers the CI workflow in `.github/workflows/ci.yml` which:

- Regenerates all grammars and crates with `arborium-xtask gen --version 0.3.0`
- Builds and tests the `arborium` crate
- Builds all WASM plugins into `dist/plugins`
- Publishes:
  - All grammar crates and core crates to crates.io via `cargo xtask publish crates`
  - All npm packages (per-language + `@arborium/arborium`) via `cargo xtask publish npm -o dist/plugins`

## Two Outputs, Two Registries

### 1. Native Rust Crates → crates.io

- ~98 grammar crates organized into ~10 hand-picked animal groups
- Core crates (`arborium`, `arborium-collection`, `tree-sitter-*`) always published
- Each group publishes independently from `langs/group-{animal}/` via
  `cargo ws publish --publish-as-is`
- We use `cargo ws publish` instead of `cargo publish --workspace` because the
  latter is still brittle with partial publishes; `cargo ws publish` can resume
  cleanly.
- **Retry-safe**: crates.io warns and skips already-published versions

### 2. WASM Plugins → npm

- All grammars with `generate-plugin: true` in
  `langs/group-{animal}/{lang}/def/arborium.kdl`
- Built via `cargo xtask build` for `wasm32-unknown-unknown` from the same group directory
- Published as per-language packages under the `@arborium` scope, e.g.
  `@arborium/rust`, `@arborium/javascript`, etc.
- **Published together with crates.io** in the same per-group CI job for version sync

## Publishing Strategy

- A single tag (`vX.Y.Z`) publishes **all** crates.io + npm artifacts together.
- Core crates (`arborium`, `tree-sitter-*`, etc.) and all
  `arborium-{lang}` crates publish once per release.

### crates.io (all crates)

Cargo handles already-published versions gracefully - it warns and continues:
```
warning: crate arborium-rust@0.3.0 already exists on crates.io
```

So retrying a publish job is safe; already-published crates are skipped.

### npm (all packages, via xtask)

npm is **not graceful** - it hard-fails with `EPUBLISHCONFLICT`:
```
npm ERR! code EPUBLISHCONFLICT
npm ERR! Cannot publish over existing version
```

**xtask publish** (per group) must:
- Check if version exists before publishing
- Distinguish `EPUBLISHCONFLICT` (skip, continue) from real errors (fail)
- Handle retries without re-publishing successes

The CI workflow builds all plugins into `dist/plugins` first, then one job
calls `xtask publish npm -o dist/plugins` to publish every npm package in one
go.

`xtask publish npm` reads the canonical version from `version.json` (written by
`xtask gen --version`) and enforces:

- Refuses to publish if the version is a dev one like `0.0.0-dev`
- Refuses to publish if any npm package version does not match `version.json`

This means you *must* run `cargo xtask gen --version X.Y.Z` before publishing
to npm, otherwise the command will fail instead of silently pushing the wrong
version (e.g., `0.0.0-dev`) to the registry.

## What's in Git vs Generated

### Source of Truth vs Generated

The **source of truth** for each language lives under its group and language,
inside a `def/` directory. Generated artifacts live next to it as `crate/`
(static Rust crate) and `npm/` (WASM plugin package).

```
langs/
├── group-acorn/                  (Web languages)
│   ├── rust/
│   │   ├── def/                  ← LANGUAGE DEFINITIONS (committed)
│   │   │   ├── arborium.kdl      ← SOURCE OF TRUTH
│   │   │   ├── grammar/
│   │   │   │   ├── grammar.js    ← tree-sitter grammar
│   │   │   │   └── scanner.c     ← custom scanner (if any)
│   │   │   ├── queries/
│   │   │   │   └── highlights.scm ← highlight queries
│   │   │   └── samples/          ← test samples
│   │   ├── crate/                ← Static linking crate (generated)
│   │   └── npm/                  ← WASM plugin package (generated)
│   ├── javascript/
│   ├── html/
│   └── [other web languages...]
├── group-birch/                  (Systems: C, C++, Rust, Go, Zig, etc.)
├── group-cedar/                  (JVM: Java, Scala, Kotlin, Clojure)
├── group-fern/                   (Functional: Haskell, OCaml, Elixir, etc.)
├── group-hazel/                  (Scripting: Python, Ruby, Bash, Lua, etc.)
├── group-maple/                  (Config/data: TOML, YAML, JSON, SQL, etc.)
├── group-moss/                   (Scientific: R, Julia, MATLAB, GLSL, etc.)
├── group-pine/                   (Modern: Swift, Dart, Zig, etc.)
├── group-sage/                   (Legacy/enterprise: C#, VB, Elisp, etc.)
└── group-willow/                 (Markup: Markdown, HTML templates, etc.)
```
### Generated (gitignored)

```
langs/group-{animal}/{lang}/crate/
├── Cargo.toml            ← GENERATED by xtask gen
├── build.rs              ← GENERATED by xtask gen
├── src/lib.rs            ← GENERATED by xtask gen
└── grammar/
    └── src/              ← GENERATED by xtask gen (tree-sitter generate)
        ├── parser.c
        ├── grammar.json
        └── ...

langs/group-{animal}/{lang}/npm/
├── Cargo.toml            ← GENERATED for cargo xtask build
├── src/
│   └── bindings.rs      ← GENERATED bindings
└── package.json          ← GENERATED npm package
```

### Non-generated crates (hand-written, committed)

These crates don't have `arborium.kdl` and are fully hand-written:
- `arborium` (main crate)
- `arborium-test-harness`
- `arborium-sysroot`
- `arborium-host`
- `arborium-wire`
- `arborium-plugin-runtime`

## What `xtask gen --version X.Y.Z` Does

1. **Updates core crate versions:**
   - `arborium/Cargo.toml` version = "X.Y.Z"
   - `arborium-collection/Cargo.toml` version = "X.Y.Z"

2. **Generates group workspace files:**
   - `langs/group-{animal}/Cargo.toml` with member crates and version "X.Y.Z"

3. **Generates grammar crate files from definitions:**
   - Reads `langs/group-{animal}/{lang}/def/arborium.kdl` and friends
   - Writes `langs/group-{animal}/{lang}/crate/Cargo.toml` with version "X.Y.Z"
   - Writes `build.rs` with correct C compilation setup
   - Writes `src/lib.rs` with language exports
   - Runs tree-sitter generate into `grammar/src/*`

4. **Generates WASM plugin packages:**
   - Writes `langs/group-{animal}/{lang}/npm/Cargo.toml` for plugin builds
   - Writes `langs/group-{animal}/{lang}/npm/package.json` for npm publishing
   - Writes `langs/group-{animal}/{lang}/npm/src/bindings.rs` generated bindings

When called without `--version`, uses `0.0.0-dev` (fine for local dev since path deps ignore versions).

## Workflows

### Local Development

```bash
# Edit arborium.kdl, grammar.js, queries, etc.

# Regenerate (uses 0.0.0-dev version, doesn't matter locally)
cargo xtask gen

# Build and test
cargo build
cargo test
```

### Release

See **Release Flow** above for the full tag + CI story.

## Artifacts Published

| Registry | Package | Count |
|----------|---------|-------|
| crates.io | `arborium` (core with inventory) | 1 |
| crates.io | `arborium-collection` (feature-gated) | 1 |
| crates.io | `arborium-{lang}` (static crates) | 98 |
| crates.io | `arborium-test-harness` | 1 |
| crates.io | `arborium-sysroot` | 1 |
| crates.io | `tree-sitter-patched-arborium` | 1 |
| crates.io | `tree-sitter-highlight-patched-arborium` | 1 |
| npmjs.com | `@arborium/arborium` (bundle) | 1 |
| npmjs.com | `@arborium/{lang}` (per-language WASM plugins) | 98 |

## Integrations

These projects use arborium for syntax highlighting:

| Integration | Description | Status |
|------------|-------------|--------|
| **dodeca** | Static site generator ([website](https://dodeca.bearcove.eu/), [GitHub](https://github.com/bearcove/dodeca)) | In use |
| **docs.rs** | Via `--html-in-header` for TOML/shell highlighting in rustdoc | Documented |

## Roadmap to 1.0

### Done

- [x] Core Rust library with 69+ language grammars
- [x] Theme system with 15+ bundled themes (Catppuccin, Dracula, Tokyo Night, etc.)
- [x] HTML rendering with compact custom elements (`<a-k>` vs `<span class="keyword">`)
- [x] ANSI output for terminal applications
- [x] Browser IIFE drop-in script (auto-highlights code blocks)
- [x] Browser ESM module for bundlers
- [x] WASM compilation target (`wasm32-unknown-unknown`)
- [x] docs.rs integration via `--html-in-header`
- [x] IIFE skips already-highlighted blocks (docs.rs compatibility)

### In Progress

- [ ] WASM plugin system
  - [x] Plugin runtime crate (`arborium-plugin-runtime`)
  - [x] Host runtime crate (`arborium-host`)
  - [x] Injection dependency resolution in browser
  - [ ] Dynamic grammar loading from bytes (currently loads from CDN)

### Blocking 1.0

- [x] **Tree-named grammar groups** - 10 groups finalized (acorn, birch, cedar, etc.)
- [x] **API stability review** - Documented above in "API Stability" section
- [x] **Language injection in browser** - HTML→JS→SQL nesting works
- [x] **Visual regression tests** - Playwright integration with CI

### Nice to Have (post-1.0)

- [ ] Standalone CLI tool for highlighting files
- [ ] Plugin discovery/registry mechanism (dynamic grammar loading)
- [ ] Performance benchmarks and optimization pass
- [ ] Incremental parsing with `apply-edit` support
- [ ] Sample files for all ~98 grammars (see `docs/samples_todo.md`)

## API Stability

| Crate | Stability | Notes |
|-------|-----------|-------|
| `arborium` | **Stable** | Main entry point, semver guarantees |
| `arborium-highlight` | **Stable** | Core highlighting traits and types |
| `arborium-theme` | **Stable** | Theme definitions and builtins |
| `arborium-{lang}` | **Stable** | Per-language grammar crates |
| `arborium-wire` | Internal | Plugin protocol, may change |
| `arborium-plugin-runtime` | Internal | Plugin internals, may change |
| `arborium-host` | Internal | WASM host, may change |
| `arborium-sysroot` | Internal | WASM build support, may change |

## Publishing TODO

- [ ] Update generate caching to tree-sitter-cli output only
- [ ] Standardize wasm-opt settings to -Oz
