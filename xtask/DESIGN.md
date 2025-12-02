# xtask Design

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
│   │   ├── queries/          # highlight queries
│   │   └── samples/          # example files
│   ├── arborium-javascript/
│   ├── arborium-haskell/
│   └── ...
└── xtask/
```

**`arborium.kdl`** contains everything:
- Source: repo, commit, license
- Metadata: name, icon, aliases, tier, tag
- Description, trivia, links
- Sample definitions

No `grammars/` directory. No top-level `GRAMMARS.toml`. No `info.toml`.

Vendoring new grammars and checking for updates is a manual process.

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

All writes use compare-before-write: generate in memory, compare with disk, only write if different. This preserves mtimes and avoids thrashing the Cargo cache.

1. For each grammar in registry:
   - Regenerate `Cargo.toml` from template
   - Regenerate `build.rs` from template
   - Regenerate `src/lib.rs` from template
2. Regenerate main `crates/arborium/src/lib.rs` (re-exports all grammars)

### `cargo xtask serve`

Build and serve the WASM demo locally.

1. Run `generate`
2. Build all grammars for WASM target
3. Generate demo HTML/JS/CSS
4. Start HTTP server
