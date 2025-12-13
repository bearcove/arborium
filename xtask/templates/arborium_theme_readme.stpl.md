# arborium-theme

Theme and highlight definitions for [arborium](https://github.com/bearcove/arborium) syntax highlighting.

This crate provides:

- **Highlight definitions**: Mapping from tree-sitter capture names to short HTML tags (e.g., `keyword` -> `<a-k>`)
- **Theme types**: `Theme`, `Color`, `Style` for representing syntax highlighting themes
- **Built-in themes**: <%= theme_count %> popular color schemes ready to use

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

This crate includes <%= theme_count %> themes from popular color schemes. We are grateful to the original theme authors:

| Theme | Variant | Source |
|-------|---------|--------|
<% for theme in themes { %>| <%= theme.name %> | <%= theme.variant %> | <% if let Some(ref url) = theme.source_url { %>[<%= theme.source_display %>](<%= url %>)<% } else { %>—<% } %> |
<% } %>
## License

This crate is licensed under MIT OR Apache-2.0.

The built-in themes are adaptations of color schemes from their respective projects. Please see each project's repository for their specific licensing terms.
