//! Arborium - Batteries-included tree-sitter grammar collection
//!
//! This crate provides:
//! - Re-exports of individual grammar crates via feature flags
//! - HTML rendering with CSS classes for syntax highlighting
//! - WASM support with custom allocator (automatically enabled on WASM targets)
//!
//! # Usage
//!
//! Enable the languages you need via feature flags:
//!
//! ```toml
//! [dependencies]
//! arborium = { version = "0.1", features = ["lang-rust", "lang-python"] }
//! ```
//!
//! Or enable all languages:
//!
//! ```toml
//! [dependencies]
//! arborium = { version = "0.1", features = ["all-languages"] }
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use arborium::{html, lang_rust, HIGHLIGHT_NAMES};
//! use arborium::tree_sitter_highlight::{Highlighter, HighlightConfiguration};
//!
//! // Create a highlight configuration for Rust
//! let mut config = HighlightConfiguration::new(
//!     lang_rust::language().into(),
//!     "rust",
//!     lang_rust::HIGHLIGHTS_QUERY,
//!     lang_rust::INJECTIONS_QUERY,
//!     lang_rust::LOCALS_QUERY,
//! ).unwrap();
//! config.configure(&HIGHLIGHT_NAMES.iter().map(|s| s.to_string()).collect::<Vec<_>>());
//!
//! // Render to HTML
//! let mut highlighter = Highlighter::new();
//! let mut output = Vec::new();
//! html::render(&mut output, &mut highlighter, &config, "fn main() {}", |_| None).unwrap();
//! ```

pub use tree_sitter_highlight_patched_arborium as tree_sitter_highlight;
pub use tree_sitter_patched_arborium as tree_sitter;

pub mod ansi;
pub mod highlighter;
pub mod highlights;
pub mod html;
pub mod theme;

pub use highlighter::Highlighter;

#[cfg(target_family = "wasm")]
mod wasm;

// Language grammar re-exports based on enabled features.
// Each module provides:
// - `language()` - Returns the tree-sitter Language
// - `HIGHLIGHTS_QUERY` - The highlight query string
// - `INJECTIONS_QUERY` - The injection query string
// - `LOCALS_QUERY` - The locals query string

#[cfg(feature = "lang-ada")]
pub use arborium_ada as lang_ada;

#[cfg(feature = "lang-agda")]
pub use arborium_agda as lang_agda;

#[cfg(feature = "lang-asm")]
pub use arborium_asm as lang_asm;

#[cfg(feature = "lang-awk")]
pub use arborium_awk as lang_awk;

#[cfg(feature = "lang-bash")]
pub use arborium_bash as lang_bash;

#[cfg(feature = "lang-batch")]
pub use arborium_batch as lang_batch;

#[cfg(feature = "lang-c")]
pub use arborium_c as lang_c;

#[cfg(feature = "lang-c-sharp")]
pub use arborium_c_sharp as lang_c_sharp;

#[cfg(feature = "lang-caddy")]
pub use arborium_caddy as lang_caddy;

#[cfg(feature = "lang-capnp")]
pub use arborium_capnp as lang_capnp;

#[cfg(feature = "lang-clojure")]
pub use arborium_clojure as lang_clojure;

#[cfg(feature = "lang-cmake")]
pub use arborium_cmake as lang_cmake;

#[cfg(feature = "lang-commonlisp")]
pub use arborium_commonlisp as lang_commonlisp;

#[cfg(feature = "lang-cpp")]
pub use arborium_cpp as lang_cpp;

#[cfg(feature = "lang-css")]
pub use arborium_css as lang_css;

#[cfg(feature = "lang-d")]
pub use arborium_d as lang_d;

#[cfg(feature = "lang-dart")]
pub use arborium_dart as lang_dart;

#[cfg(feature = "lang-devicetree")]
pub use arborium_devicetree as lang_devicetree;

#[cfg(feature = "lang-diff")]
pub use arborium_diff as lang_diff;

#[cfg(feature = "lang-dockerfile")]
pub use arborium_dockerfile as lang_dockerfile;

#[cfg(feature = "lang-dot")]
pub use arborium_dot as lang_dot;

#[cfg(feature = "lang-elisp")]
pub use arborium_elisp as lang_elisp;

#[cfg(feature = "lang-elixir")]
pub use arborium_elixir as lang_elixir;

#[cfg(feature = "lang-elm")]
pub use arborium_elm as lang_elm;

#[cfg(feature = "lang-erlang")]
pub use arborium_erlang as lang_erlang;

#[cfg(feature = "lang-fish")]
pub use arborium_fish as lang_fish;

#[cfg(feature = "lang-fsharp")]
pub use arborium_fsharp as lang_fsharp;

#[cfg(feature = "lang-gleam")]
pub use arborium_gleam as lang_gleam;

#[cfg(feature = "lang-glsl")]
pub use arborium_glsl as lang_glsl;

#[cfg(feature = "lang-go")]
pub use arborium_go as lang_go;

#[cfg(feature = "lang-graphql")]
pub use arborium_graphql as lang_graphql;

#[cfg(feature = "lang-haskell")]
pub use arborium_haskell as lang_haskell;

#[cfg(feature = "lang-hcl")]
pub use arborium_hcl as lang_hcl;

#[cfg(feature = "lang-hlsl")]
pub use arborium_hlsl as lang_hlsl;

#[cfg(feature = "lang-html")]
pub use arborium_html as lang_html;

#[cfg(feature = "lang-idris")]
pub use arborium_idris as lang_idris;

#[cfg(feature = "lang-ini")]
pub use arborium_ini as lang_ini;

#[cfg(feature = "lang-java")]
pub use arborium_java as lang_java;

#[cfg(feature = "lang-javascript")]
pub use arborium_javascript as lang_javascript;

#[cfg(feature = "lang-jinja2")]
pub use arborium_jinja2 as lang_jinja2;

#[cfg(feature = "lang-jq")]
pub use arborium_jq as lang_jq;

#[cfg(feature = "lang-json")]
pub use arborium_json as lang_json;

#[cfg(feature = "lang-julia")]
pub use arborium_julia as lang_julia;

#[cfg(feature = "lang-kdl")]
pub use arborium_kdl as lang_kdl;

#[cfg(feature = "lang-kotlin")]
pub use arborium_kotlin as lang_kotlin;

#[cfg(feature = "lang-lean")]
pub use arborium_lean as lang_lean;

#[cfg(feature = "lang-lua")]
pub use arborium_lua as lang_lua;

#[cfg(feature = "lang-matlab")]
pub use arborium_matlab as lang_matlab;

#[cfg(feature = "lang-meson")]
pub use arborium_meson as lang_meson;

#[cfg(feature = "lang-nginx")]
pub use arborium_nginx as lang_nginx;

#[cfg(feature = "lang-ninja")]
pub use arborium_ninja as lang_ninja;

#[cfg(feature = "lang-nix")]
pub use arborium_nix as lang_nix;

#[cfg(feature = "lang-objc")]
pub use arborium_objc as lang_objc;

#[cfg(feature = "lang-ocaml")]
pub use arborium_ocaml as lang_ocaml;

#[cfg(feature = "lang-perl")]
pub use arborium_perl as lang_perl;

#[cfg(feature = "lang-php")]
pub use arborium_php as lang_php;

#[cfg(feature = "lang-powershell")]
pub use arborium_powershell as lang_powershell;

#[cfg(feature = "lang-prolog")]
pub use arborium_prolog as lang_prolog;

#[cfg(feature = "lang-python")]
pub use arborium_python as lang_python;

#[cfg(feature = "lang-query")]
pub use arborium_query as lang_query;

#[cfg(feature = "lang-r")]
pub use arborium_r as lang_r;

#[cfg(feature = "lang-rescript")]
pub use arborium_rescript as lang_rescript;

#[cfg(feature = "lang-ron")]
pub use arborium_ron as lang_ron;

#[cfg(feature = "lang-ruby")]
pub use arborium_ruby as lang_ruby;

#[cfg(feature = "lang-rust")]
pub use arborium_rust as lang_rust;

#[cfg(feature = "lang-scala")]
pub use arborium_scala as lang_scala;

#[cfg(feature = "lang-scheme")]
pub use arborium_scheme as lang_scheme;

#[cfg(feature = "lang-scss")]
pub use arborium_scss as lang_scss;

#[cfg(feature = "lang-sparql")]
pub use arborium_sparql as lang_sparql;

#[cfg(feature = "lang-sql")]
pub use arborium_sql as lang_sql;

#[cfg(feature = "lang-ssh-config")]
pub use arborium_ssh_config as lang_ssh_config;

#[cfg(feature = "lang-starlark")]
pub use arborium_starlark as lang_starlark;

#[cfg(feature = "lang-svelte")]
pub use arborium_svelte as lang_svelte;

#[cfg(feature = "lang-swift")]
pub use arborium_swift as lang_swift;

#[cfg(feature = "lang-textproto")]
pub use arborium_textproto as lang_textproto;

#[cfg(feature = "lang-thrift")]
pub use arborium_thrift as lang_thrift;

#[cfg(feature = "lang-tlaplus")]
pub use arborium_tlaplus as lang_tlaplus;

#[cfg(feature = "lang-toml")]
pub use arborium_toml as lang_toml;

#[cfg(feature = "lang-tsx")]
pub use arborium_tsx as lang_tsx;

#[cfg(feature = "lang-typescript")]
pub use arborium_typescript as lang_typescript;

#[cfg(feature = "lang-typst")]
pub use arborium_typst as lang_typst;

#[cfg(feature = "lang-uiua")]
pub use arborium_uiua as lang_uiua;

#[cfg(feature = "lang-vb")]
pub use arborium_vb as lang_vb;

#[cfg(feature = "lang-verilog")]
pub use arborium_verilog as lang_verilog;

#[cfg(feature = "lang-vhdl")]
pub use arborium_vhdl as lang_vhdl;

#[cfg(feature = "lang-vim")]
pub use arborium_vim as lang_vim;

#[cfg(feature = "lang-vue")]
pub use arborium_vue as lang_vue;

#[cfg(feature = "lang-x86asm")]
pub use arborium_x86asm as lang_x86asm;

#[cfg(feature = "lang-xml")]
pub use arborium_xml as lang_xml;

#[cfg(feature = "lang-yaml")]
pub use arborium_yaml as lang_yaml;

#[cfg(feature = "lang-yuri")]
pub use arborium_yuri as lang_yuri;

#[cfg(feature = "lang-zig")]
pub use arborium_zig as lang_zig;

#[cfg(feature = "lang-zsh")]
pub use arborium_zsh as lang_zsh;

/// Standard highlight names used for syntax highlighting.
///
/// These names are used to configure tree-sitter's `HighlightConfiguration`.
/// The indices correspond to HTML element tags (e.g., index 7 = `<a-k>` for keyword).
///
/// See [`highlights::HIGHLIGHTS`] for the complete definitions including HTML tags.
pub const HIGHLIGHT_NAMES: [&str; highlights::COUNT] = highlights::names();
