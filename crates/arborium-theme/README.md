# arborium-theme

Theme and highlight definitions for [arborium](https://github.com/bearcove/arborium) syntax highlighting.

This crate provides:

- **Highlight definitions**: Mapping from tree-sitter capture names to short HTML tags (e.g., `keyword` -> `<a-k>`)
- **Theme types**: `Theme`, `Color`, `Style` for representing syntax highlighting themes
- **Built-in themes**: 32 popular color schemes ready to use

## Usage

```rust
use arborium_theme::{Theme, builtin, HIGHLIGHTS};

// Use a built-in theme
let theme = builtin::catppuccin_mocha();

// Generate CSS for the theme
let css = theme.to_css("[data-theme=\"mocha\"]");

// Access highlight definitions
for def in HIGHLIGHTS {
    println!("{} -> <a-{}>", def.name, def.tag);
}
```

## Built-in Themes

This crate includes 32 themes from popular color schemes. We are grateful to the original theme authors:

| Theme | Variant | Source |
|-------|---------|--------|
| Catppuccin Mocha | dark | [catppuccin/catppuccin](https://github.com/catppuccin/catppuccin) |
| Catppuccin Latte | light | [catppuccin/catppuccin](https://github.com/catppuccin/catppuccin) |
| Catppuccin Frappé | dark | [catppuccin/catppuccin](https://github.com/catppuccin/catppuccin) |
| Catppuccin Macchiato | dark | [catppuccin/catppuccin](https://github.com/catppuccin/catppuccin) |
| Dracula | dark | [draculatheme.com](https://draculatheme.com) |
| Tokyo Night | dark | [enkia/tokyo-night-vscode-theme](https://github.com/enkia/tokyo-night-vscode-theme) |
| Nord | dark | [www.nordtheme.com](https://www.nordtheme.com) |
| One Dark | dark | [atom/one-dark-syntax](https://github.com/atom/one-dark-syntax) |
| GitHub Dark | dark | [primer/github-vscode-theme](https://github.com/primer/github-vscode-theme) |
| GitHub Light | light | [primer/github-vscode-theme](https://github.com/primer/github-vscode-theme) |
| Gruvbox Dark | dark | [morhetz/gruvbox](https://github.com/morhetz/gruvbox) |
| Gruvbox Light | light | [morhetz/gruvbox](https://github.com/morhetz/gruvbox) |
| Monokai | dark | [monokai.pro](https://monokai.pro) |
| Kanagawa Dragon | dark | [rebelot/kanagawa.nvim](https://github.com/rebelot/kanagawa.nvim) |
| Rosé Pine Moon | dark | [rosepinetheme.com](https://rosepinetheme.com) |
| Ayu Dark | dark | [ayu-theme/ayu-colors](https://github.com/ayu-theme/ayu-colors) |
| Ayu Light | light | [ayu-theme/ayu-colors](https://github.com/ayu-theme/ayu-colors) |
| Solarized Dark | dark | [ethanschoonover.com/solarized](https://ethanschoonover.com/solarized/) |
| Solarized Light | light | [ethanschoonover.com/solarized](https://ethanschoonover.com/solarized/) |
| EF Melissa Dark | dark | [protesilaos.com/emacs/ef-themes](https://protesilaos.com/emacs/ef-themes) |
| Melange Dark | dark | [savq/melange-nvim](https://github.com/savq/melange-nvim) |
| Melange Light | light | [savq/melange-nvim](https://github.com/savq/melange-nvim) |
| Light Owl | light | [sdras/night-owl-vscode-theme](https://github.com/sdras/night-owl-vscode-theme) |
| Lucius Light | light | [jonathanfilip/vim-lucius](https://github.com/jonathanfilip/vim-lucius) |
| Rustdoc Light | light | [rust-lang/rust/blob/master/src/librustdoc/html/static/css/themes/light.css](https://github.com/rust-lang/rust/blob/master/src/librustdoc/html/static/css/themes/light.css) |
| Rustdoc Dark | dark | [rust-lang/rust/blob/master/src/librustdoc/html/static/css/themes/dark.css](https://github.com/rust-lang/rust/blob/master/src/librustdoc/html/static/css/themes/dark.css) |
| Rustdoc Ayu | dark | [rust-lang/rust/blob/master/src/librustdoc/html/static/css/themes/ayu.css](https://github.com/rust-lang/rust/blob/master/src/librustdoc/html/static/css/themes/ayu.css) |
| Dayfox | light | [EdenEast/nightfox.nvim](https://github.com/EdenEast/nightfox.nvim) |
| Alabaster | light | [tonsky/vscode-theme-alabaster](https://github.com/tonsky/vscode-theme-alabaster) |
| Cobalt2 | dark | [wesbos/cobalt2-vscode](https://github.com/wesbos/cobalt2-vscode) |
| Zenburn | dark | [jnurmine/Zenburn](https://github.com/jnurmine/Zenburn) |
| Desert256 | dark | [vim-scripts/desert256.vim](https://github.com/vim-scripts/desert256.vim) |

## License

This crate is licensed under MIT OR Apache-2.0.

The built-in themes are adaptations of color schemes from their respective projects. Please see each project's repository for their specific licensing terms.