# arborium

Batteries-included [tree-sitter](https://tree-sitter.github.io/tree-sitter/) grammar collection with HTML rendering and WASM support.

[![Crates.io](https://img.shields.io/crates/v/arborium.svg)](https://crates.io/crates/arborium)
[![Documentation](https://docs.rs/arborium/badge.svg)](https://docs.rs/arborium)
[![License](https://img.shields.io/crates/l/arborium.svg)](LICENSE-MIT)

## Features

- **<%= permissive_grammars.len() + gpl_grammars.len() %> language grammars** included out of the box
- **<%= permissive_grammars.len() %> permissively licensed** (MIT/Apache-2.0/CC0/Unlicense) grammars enabled by default
- **WASM support** with custom allocator fix
- **Feature flags** for fine-grained control over included languages

## Usage

```toml
[dependencies]
arborium = "<%= version %>"
```

By default, all permissively-licensed grammars are included. To select specific languages:

```toml
[dependencies]
arborium = { version = "<%= version %>", default-features = false, features = ["lang-rust", "lang-javascript"] }
```

## Browser Usage

Arborium can be used in the browser in two ways:

### Option 1: Drop-in Script (Easiest)

Add a single script tag and arborium auto-highlights all code blocks:

```html
<script src="https://cdn.jsdelivr.net/npm/@arborium/arborium/dist/arborium.iife.js"></script>
```

That's it! Arborium will:
- Auto-detect languages from `class="language-*"` or `data-lang="*"` attributes
- Load grammar WASM plugins on-demand from jsDelivr CDN
- Inject the default theme CSS

**Configuration via data attributes:**

```html
<script
  src="https://cdn.jsdelivr.net/npm/@arborium/arborium/dist/arborium.iife.js"
  data-theme="mocha"
  data-selector="pre code"
  data-manual
></script>
```

**Configuration via JavaScript:**

```html
<script>
  window.Arborium = {
    theme: 'tokyo-night',
    selector: 'pre code, .highlight',
    cdn: 'jsdelivr',  // or 'unpkg' or a custom URL
    version: '1', // or 'latest'
  };
</script>
<script src="https://cdn.jsdelivr.net/npm/@arborium/arborium/dist/arborium.iife.js"></script>
```

**Manual highlighting:**

```html
<script src="..." data-manual></script>
<script>
  // Highlight all code blocks
  arborium.highlightAll();

  // Highlight a specific element
  arborium.highlightElement(document.querySelector('code'), 'rust');
</script>
```

### Option 2: ESM Module (Programmatic)

For bundlers (Vite, webpack, etc.) or ESM-native environments:

```bash
npm install @arborium/arborium
```

```typescript
import { loadGrammar, highlight } from '@arborium/arborium';

// Load a grammar (fetched from CDN on first use)
const grammar = await loadGrammar('rust');

// Highlight code
const html = grammar.highlight('fn main() { println!("Hello!"); }');

// Or use the convenience function
const html = await highlight('rust', code);
```

### Option 3: Compile Rust to WASM (Maximum Control)

For complete control and offline-first apps, compile the Rust crate directly to WASM:

```toml
[dependencies]
arborium = { version = "<%= version %>", default-features = false, features = ["lang-rust", "lang-javascript"] }
```

```bash
# Requires LLVM with WASM support (see FAQ below)
cargo build --target wasm32-unknown-unknown
```

This embeds selected grammars directly in your WASM binary - no CDN required at runtime.

### Themes

Arborium includes 32 built-in themes from popular color schemes.

**Dark themes:** `catppuccin-mocha`, `catppuccin-macchiato`, `catppuccin-frappe`, `dracula`, `tokyo-night`, `nord`, `one-dark`, `github-dark`, `gruvbox-dark`, `monokai`, `kanagawa-dragon`, `rose-pine-moon`, `ayu-dark`, `solarized-dark`, `ef-melissa-dark`, `melange-dark`, `cobalt2`, `zenburn`, `desert256`, `rustdoc-dark`, `rustdoc-ayu`

**Light themes:** `catppuccin-latte`, `github-light`, `gruvbox-light`, `ayu-light`, `solarized-light`, `melange-light`, `light-owl`, `lucius-light`, `dayfox`, `alabaster`, `rustdoc-light`

Import theme CSS:
```html
<link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/@arborium/arborium/dist/themes/tokyo-night.css">
```

Or let the IIFE bundle auto-inject it via the `data-theme` attribute.

**Theme Attribution:** All themes are adaptations of color schemes from their original projects. See the [arborium-theme crate README](https://github.com/bearcove/arborium/tree/main/crates/arborium-theme#built-in-themes) for full attribution and source links.

## Feature Flags

### Grammar Collections

| Feature | Description |
|---------|-------------|
| `mit-grammars` | All permissively licensed grammars (MIT, Apache-2.0, CC0) - **default** |
| `gpl-grammars` | GPL-licensed grammars (copyleft - may affect your project's license) |
| `all-grammars` | All grammars including GPL |

### Permissive Grammars (<%= permissive_grammars.len() %>)

These grammars use permissive licenses (MIT, Apache-2.0, CC0, Unlicense) and are included by default.

| Feature | Language | License | Source |
|---------|----------|---------|--------|
<% for grammar in permissive_grammars { %>
| `<%= grammar.feature %>` | <%= grammar.name %> | <%= grammar.license %> | <% if grammar.repo_url == "local" { %>local<% } else { %>[tree-sitter-<%= grammar.feature.strip_prefix("lang-").unwrap_or(&grammar.feature) %>](<%= grammar.repo_url %>)<% } %> |
<% } %>

### GPL-Licensed Grammars (<%= gpl_grammars.len() %>)

These grammars are **not included by default** due to their copyleft license.
Enabling them may have implications for your project's licensing.

| Feature | Language | License | Source |
|---------|----------|---------|--------|
<% for grammar in gpl_grammars { %>
| `<%= grammar.feature %>` | <%= grammar.name %> | <%= grammar.license %> | <% if grammar.repo_url == "local" { %>local<% } else { %>[tree-sitter-<%= grammar.feature.strip_prefix("lang-").unwrap_or(&grammar.feature) %>](<%= grammar.repo_url %>)<% } %> |
<% } %>

## HTML Tag Reference

Arborium renders syntax highlighting using custom HTML elements. When highlighting code, it wraps spans of text with tags like `<a-k>`, `<a-f>`, etc. These tags are styled by the theme CSS you choose.

### Tag Mappings

Each tag corresponds to a semantic code element. Here's the complete reference:

| Tag | Element Type | Description |
|-----|--------------|-------------|
| `<a-k>` | **Keyword** | Language keywords (if, else, while, class, fn, etc.) |
| `<a-f>` | **Function** | Function names and method calls |
| `<a-s>` | **String** | String literals and character literals |
| `<a-c>` | **Comment** | Comments (line and block) |
| `<a-t>` | **Type** | Type names and type annotations |
| `<a-v>` | **Variable** | Variable names and identifiers |
| `<a-co>` | **Constant** | Constants and boolean literals |
| `<a-n>` | **Number** | Numeric literals (integers and floats) |
| `<a-o>` | **Operator** | Operators (+, -, *, /, &&, etc.) |
| `<a-p>` | **Punctuation** | Delimiters and punctuation (parentheses, brackets, commas) |
| `<a-pr>` | **Property** | Object properties and struct fields |
| `<a-at>` | **Attribute** | Attributes and annotations (@, #[derive], etc.) |
| `<a-tg>` | **Tag** | HTML/XML tags |
| `<a-m>` | **Macro** | Macro names and invocations |
| `<a-l>` | **Label** | Labels and goto targets |
| `<a-ns>` | **Namespace** | Namespaces and modules |
| `<a-cr>` | **Constructor** | Constructor functions and type constructors |

### Markup Tags (Markdown, etc.)

| Tag | Element Type | Description |
|-----|--------------|-------------|
| `<a-tt>` | **Title** | Headings and titles |
| `<a-st>` | **Strong** | Bold text |
| `<a-em>` | **Emphasis** | Italic text |
| `<a-tu>` | **Link** | URLs and hyperlinks |
| `<a-tl>` | **Literal** | Code blocks and inline code |
| `<a-tx>` | **Strikethrough** | Strikethrough text |

### Diff Tags

| Tag | Element Type | Description |
|-----|--------------|-------------|
| `<a-da>` | **Diff Add** | Added lines in diffs |
| `<a-dd>` | **Diff Delete** | Deleted lines in diffs |

### Special Tags

| Tag | Element Type | Description |
|-----|--------------|-------------|
| `<a-eb>` | **Embedded** | Embedded language content |
| `<a-er>` | **Error** | Syntax errors |

### How It Works

Arborium uses tree-sitter grammars to parse code and identify semantic elements. Multiple capture names from tree-sitter queries (like `@keyword.function`, `@keyword.import`, `@conditional`) all map to the same theme slot. For example:

- `@keyword`, `@keyword.function`, `@include`, `@conditional` → all become `<a-k>` (keyword)
- `@function`, `@function.builtin`, `@method` → all become `<a-f>` (function)
- `@comment`, `@comment.documentation` → all become `<a-c>` (comment)

Adjacent spans with the same tag are automatically merged into a single element for efficiency.

### Styling Example

To create a custom theme, target these elements in your CSS:

```css
/* Keywords in blue */
a-k { color: #569cd6; }

/* Functions in yellow */
a-f { color: #dcdcaa; }

/* Strings in green */
a-s { color: #ce9178; }

/* Comments in gray */
a-c { color: #6a9955; font-style: italic; }

/* Types in cyan */
a-t { color: #4ec9b0; }
```

See the [included themes](https://github.com/bearcove/arborium/tree/main/packages/arborium/src/themes) for more examples.

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

## WASM Support

Arborium supports building for `wasm32-unknown-unknown`. This requires compiling C code (tree-sitter core and grammar parsers) to WebAssembly.

### macOS

On macOS, the built-in Apple clang does **not** support the `wasm32-unknown-unknown` target. You need to install LLVM via Homebrew:

```bash
brew install llvm
```

Then ensure the Homebrew LLVM is in your PATH when building:

```bash
export PATH="$(brew --prefix llvm)/bin:$PATH"
cargo build --target wasm32-unknown-unknown
```

## Development

This project uses `cargo xtask` for most development and release tasks.

For detailed architecture, workflows, publishing order, and layout, see `DEVELOP.md`.

For a quick overview of available commands, run:

```bash
cargo xtask help
```
