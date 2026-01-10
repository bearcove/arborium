# arborium

Batteries-included [tree-sitter](https://tree-sitter.github.io/tree-sitter/) grammar collection with HTML rendering and WASM support.

[![Crates.io](https://img.shields.io/crates/v/arborium.svg)](https://crates.io/crates/arborium)
[![Documentation](https://docs.rs/arborium/badge.svg)](https://docs.rs/arborium)
[![License](https://img.shields.io/crates/l/arborium.svg)](LICENSE-MIT)

## Quick Start

### As a Rust library

```bash
cargo add arborium
```

By default, all permissively-licensed grammars are included (~70 languages). To select specific languages:

```bash
cargo add arborium --no-default-features --features lang-rust,lang-javascript
```

### As a CLI tool

```bash
cargo install arborium-cli
arborium file.rs  # Syntax highlight in your terminal
```

### In the browser

```html
<script src="https://cdn.jsdelivr.net/npm/@arborium/arborium@2/dist/arborium.iife.js"></script>
<!-- Auto-highlights all code blocks! -->
```

## Features

- **~70 language grammars** included out of the box
- **Permissively licensed** (MIT/Apache-2.0/CC0/Unlicense) grammars enabled by default
- **WASM support** with custom allocator fix
- **HTML rendering** with 32 built-in themes
- **Browser usage** via drop-in script tag or ESM module
- **CLI tool** (`arborium-cli`) - syntax highlighting for terminal and HTML
- **Feature flags** for fine-grained control over included languages

## Documentation

For complete documentation including:
- Full language support list
- Browser usage guide (drop-in script, ESM modules, compile to WASM)
- All 32 built-in themes
- HTML tag reference
- WASM build instructions
- Feature flags reference

See the **[arborium crate on crates.io](https://crates.io/crates/arborium)** or **[docs.rs](https://docs.rs/arborium)**.

## Repository Structure

- **[`crates/arborium/`](crates/arborium/)** - Main umbrella crate (start here!)
- **[`crates/arborium-cli/`](crates/arborium-cli/)** - Terminal syntax highlighter CLI
- **[`crates/arborium-*/`](crates/)** - Individual language grammar crates (~100 crates)
- **[`packages/arborium/`](packages/arborium/)** - NPM package for browser use
- **[`xtask/`](xtask/)** - Build automation and code generation

## Sponsors

Thanks to all individual sponsors:

<p>
<a href="https://github.com/sponsors/fasterthanlime">
<picture>
<source media="(prefers-color-scheme: dark)" srcset="./static/sponsors-v3/github-dark.svg">
<img src="./static/sponsors-v3/github-light.svg" height="40" alt="GitHub Sponsors">
</picture>
</a>
<a href="https://patreon.com/fasterthanlime">
<picture>
<source media="(prefers-color-scheme: dark)" srcset="./static/sponsors-v3/patreon-dark.svg">
<img src="./static/sponsors-v3/patreon-light.svg" height="40" alt="Patreon">
</picture>
</a>
</p>

...along with corporate sponsors:

<p>
<a href="https://zed.dev">
<picture>
<source media="(prefers-color-scheme: dark)" srcset="./static/sponsors-v3/zed-dark.svg">
<img src="./static/sponsors-v3/zed-light.svg" height="40" alt="Zed">
</picture>
</a>
<a href="https://depot.dev?utm_source=arborium">
<picture>
<source media="(prefers-color-scheme: dark)" srcset="./static/sponsors-v3/depot-dark.svg">
<img src="./static/sponsors-v3/depot-light.svg" height="40" alt="Depot">
</picture>
</a>
</p>

## License

This project is dual-licensed under [MIT](LICENSE-MIT) OR [Apache-2.0](LICENSE-APACHE).

The bundled grammar sources retain their original licenses - see [LICENSES.md](LICENSES.md) for details.

## Development

This project uses `cargo xtask` for most development and release tasks.

For detailed architecture, workflows, publishing order, and layout, see `DEVELOP.md`.

For a quick overview of available commands, run:

```bash
cargo xtask help
```
