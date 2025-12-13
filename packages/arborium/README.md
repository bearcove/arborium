# @arborium/arborium

High-performance syntax highlighting powered by tree-sitter and WebAssembly.

[![npm](https://img.shields.io/npm/v/@arborium/arborium)](https://www.npmjs.com/package/@arborium/arborium)
[![License](https://img.shields.io/npm/l/@arborium/arborium)](https://github.com/bearcove/arborium)

## Installation

```bash
npm install @arborium/arborium
```

## Usage

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
  data-theme="tokyo-night"
  data-selector="pre code"
></script>
```

### Option 2: ESM Module (Programmatic)

```typescript
import { loadGrammar, highlight } from '@arborium/arborium';

// Load a grammar (fetched from CDN on first use)
const grammar = await loadGrammar('rust');

// Highlight code
const html = grammar.highlight('fn main() { println!("Hello!"); }');

// Or use the convenience function
const html = await highlight('rust', code);
```

## Themes

This package includes 32 built-in themes. Import them individually:

```html
<link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/@arborium/arborium/dist/themes/tokyo-night.css">
```

Or import in JavaScript:

```javascript
import '@arborium/arborium/themes/tokyo-night.css';
```

### Available Themes

**Dark themes:** catppuccin-mocha, catppuccin-macchiato, catppuccin-frappe, dracula, tokyo-night, nord, one-dark, github-dark, gruvbox-dark, monokai, kanagawa-dragon, rose-pine-moon, ayu-dark, solarized-dark, ef-melissa-dark, melange-dark, cobalt2, zenburn, desert256, rustdoc-dark, rustdoc-ayu

**Light themes:** catppuccin-latte, github-light, gruvbox-light, ayu-light, solarized-light, melange-light, light-owl, lucius-light, dayfox, alabaster, rustdoc-light

## Theme Attribution

All themes are adaptations of color schemes from their original projects. We are grateful to the original theme authors:

| Theme | Source |
|-------|--------|
| Ayu | [ayu-theme/ayu-colors](https://github.com/ayu-theme/ayu-colors) |
| Catppuccin | [catppuccin/catppuccin](https://github.com/catppuccin/catppuccin) |
| Cobalt2 | [wesbos/cobalt2-vscode](https://github.com/wesbos/cobalt2-vscode) |
| Dayfox | [EdenEast/nightfox.nvim](https://github.com/EdenEast/nightfox.nvim) |
| Desert256 | [vim-scripts/desert256.vim](https://github.com/vim-scripts/desert256.vim) |
| Dracula | [draculatheme.com](https://draculatheme.com) |
| EF Melissa | [protesilaos.com/emacs/ef-themes](https://protesilaos.com/emacs/ef-themes) |
| GitHub | [primer/github-vscode-theme](https://github.com/primer/github-vscode-theme) |
| Gruvbox | [morhetz/gruvbox](https://github.com/morhetz/gruvbox) |
| Kanagawa | [rebelot/kanagawa.nvim](https://github.com/rebelot/kanagawa.nvim) |
| Light Owl | [sdras/night-owl-vscode-theme](https://github.com/sdras/night-owl-vscode-theme) |
| Lucius | [jonathanfilip/vim-lucius](https://github.com/jonathanfilip/vim-lucius) |
| Melange | [savq/melange-nvim](https://github.com/savq/melange-nvim) |
| Monokai | [monokai.pro](https://monokai.pro) |
| Nord | [nordtheme.com](https://www.nordtheme.com) |
| One Dark | [atom/one-dark-syntax](https://github.com/atom/one-dark-syntax) |
| Rosé Pine | [rosepinetheme.com](https://rosepinetheme.com) |
| Rustdoc | [rust-lang/rust](https://github.com/rust-lang/rust/tree/master/src/librustdoc/html/static/css/themes) |
| Solarized | [ethanschoonover.com/solarized](https://ethanschoonover.com/solarized/) |
| Tokyo Night | [enkia/tokyo-night-vscode-theme](https://github.com/enkia/tokyo-night-vscode-theme) |
| Zenburn | [jnurmine/Zenburn](https://github.com/jnurmine/Zenburn) |
| Alabaster | [tonsky/vscode-theme-alabaster](https://github.com/tonsky/vscode-theme-alabaster) |

## Links

- [GitHub Repository](https://github.com/bearcove/arborium)
- [Documentation](https://arborium.dev)
- [Rust Crate (arborium)](https://crates.io/crates/arborium)

## License

MIT OR Apache-2.0

The bundled themes are adaptations of color schemes from their respective projects. Please see each project's repository for their specific licensing terms.
