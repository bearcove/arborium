//! Arborium — High-performance syntax highlighting
//!
//! Arborium provides batteries-included syntax highlighting powered by tree-sitter.
//! It supports 60+ languages with automatic language injection (e.g., CSS/JS in HTML).
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use arborium::Highlighter;
//!
//! let mut hl = Highlighter::new();
//! let html = hl.highlight("rust", "fn main() {}")?;
//! // Output: <a-k>fn</a-k> <a-f>main</a-f>() {}
//! ```
//!
//! # HTML vs ANSI Output
//!
//! Use [`Highlighter`] for HTML output (web pages, documentation):
//!
//! ```rust,ignore
//! use arborium::{Highlighter, Config, HtmlFormat};
//!
//! // Default: custom elements (<a-k>, <a-f>, etc.)
//! let mut hl = Highlighter::new();
//!
//! // Or use class-based output for CSS compatibility
//! let config = Config {
//!     html_format: HtmlFormat::ClassNames,
//!     ..Default::default()
//! };
//! let mut hl = Highlighter::with_config(config);
//! ```
//!
//! Use [`AnsiHighlighter`] for terminal output:
//!
//! ```rust,ignore
//! use arborium::AnsiHighlighter;
//! use arborium::theme::builtin;
//!
//! let theme = builtin::catppuccin_mocha().clone();
//! let mut hl = AnsiHighlighter::new(theme);
//! let colored = hl.highlight("rust", "fn main() {}")?;
//! println!("{}", colored);
//! ```
//!
//! # Language Support
//!
//! Enable languages via feature flags:
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
//! # Advanced Usage
//!
//! For building custom grammar providers or working with raw spans, see the
//! [`advanced`] module.

// Internal modules
mod error;
mod highlighter;
pub(crate) mod store;

// Public modules
pub mod advanced;

/// Theme system for ANSI output.
///
/// Re-exports types from `arborium-theme` for configuring syntax colors.
pub mod theme {
    pub use arborium_theme::theme::{Color, Modifiers, Style, Theme, builtin};
}

// Primary API exports
pub use error::Error;
pub use highlighter::{AnsiHighlighter, Highlighter};
pub use store::GrammarStore;

// Configuration types (re-exported from arborium-highlight)
pub use arborium_highlight::HtmlFormat;

/// Configuration for highlighting.
///
/// Controls injection depth and HTML output format.
#[derive(Debug, Clone)]
pub struct Config {
    /// Maximum depth for processing language injections.
    ///
    /// - `0`: No injections (just primary language)
    /// - `3`: Default, handles most cases (HTML with CSS/JS, Markdown with code blocks)
    /// - Higher: For deeply nested content
    pub max_injection_depth: u32,

    /// HTML output format.
    ///
    /// See [`HtmlFormat`] for options.
    pub html_format: HtmlFormat,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            max_injection_depth: 3,
            html_format: HtmlFormat::default(),
        }
    }
}

impl From<Config> for arborium_highlight::HighlightConfig {
    fn from(config: Config) -> Self {
        arborium_highlight::HighlightConfig {
            max_injection_depth: config.max_injection_depth,
            html_format: config.html_format,
        }
    }
}

// Tree-sitter re-export for advanced users
pub use arborium_tree_sitter as tree_sitter;

// WASM allocator (automatically enabled on WASM targets)
#[cfg(target_family = "wasm")]
use arborium_sysroot as _;

// Highlight names constant
use arborium_theme::highlights;

/// Standard highlight names used for syntax highlighting.
///
/// These names are used to configure tree-sitter's `HighlightConfiguration`.
/// The indices correspond to HTML element tags (e.g., index 7 = `<a-k>` for keyword).
pub const HIGHLIGHT_NAMES: [&str; highlights::COUNT] = highlights::names();

/// Detect the language from a file path or name.
///
/// Extracts the file extension and maps it to a canonical language identifier.
/// Returns `None` if the extension is not recognized.
///
/// # Example
///
/// ```rust
/// use arborium::detect_language;
///
/// assert_eq!(detect_language("main.rs"), Some("rust"));
/// assert_eq!(detect_language("/path/to/script.py"), Some("python"));
/// assert_eq!(detect_language("styles.css"), Some("css"));
/// assert_eq!(detect_language("unknown.xyz"), None);
/// ```
pub fn detect_language(path: &str) -> Option<&'static str> {
    // Extract extension from path
    let ext = path
        .rsplit('.')
        .next()
        .filter(|e| !e.contains('/') && !e.contains('\\'))?;

    // Map extension to canonical language ID
    Some(match ext.to_lowercase().as_str() {
        "ada" => "ada",

        "adoc" => "asciidoc",

        "agda" => "agda",

        "asciidoc" => "asciidoc",

        "asm" => "asm",

        "assembly" => "asm",

        "awk" => "awk",

        "bash" => "bash",

        "bat" => "batch",

        "batch" => "batch",

        "bazel" => "starlark",

        "bzl" => "starlark",

        "c" => "c",

        "c++" => "cpp",

        "c-sharp" => "c-sharp",

        "caddy" => "caddy",

        "capnp" => "capnp",

        "cfg" => "ini",

        "cjs" => "javascript",

        "cl" => "commonlisp",

        "clj" => "clojure",

        "clojure" => "clojure",

        "cmake" => "cmake",

        "cmd" => "batch",

        "commonlisp" => "commonlisp",

        "conf" => "ini",

        "cpp" => "cpp",

        "cs" => "c-sharp",

        "csharp" => "c-sharp",

        "css" => "css",

        "cts" => "typescript",

        "cxx" => "cpp",

        "d" => "d",

        "dart" => "dart",

        "devicetree" => "devicetree",

        "diff" => "diff",

        "dlang" => "d",

        "docker" => "dockerfile",

        "dockerfile" => "dockerfile",

        "dot" => "dot",

        "el" => "elisp",

        "elisp" => "elisp",

        "elixir" => "elixir",

        "elm" => "elm",

        "emacs-lisp" => "elisp",

        "erl" => "erlang",

        "erlang" => "erlang",

        "ex" => "elixir",

        "exs" => "elixir",

        "f#" => "fsharp",

        "fish" => "fish",

        "frag" => "glsl",

        "fs" => "fsharp",

        "fsharp" => "fsharp",

        "gleam" => "gleam",

        "glsl" => "glsl",

        "go" => "go",

        "golang" => "go",

        "gql" => "graphql",

        "graphql" => "graphql",

        "h" => "c",

        "haskell" => "haskell",

        "hcl" => "hcl",

        "hlsl" => "hlsl",

        "hpp" => "cpp",

        "hs" => "haskell",

        "htm" => "html",

        "html" => "html",

        "idr" => "idris",

        "idris" => "idris",

        "ini" => "ini",

        "j2" => "jinja2",

        "java" => "java",

        "javascript" => "javascript",

        "jinja" => "jinja2",

        "jinja2" => "jinja2",

        "jl" => "julia",

        "jq" => "jq",

        "js" => "javascript",

        "json" => "json",

        "jsonc" => "json",

        "jsx" => "javascript",

        "julia" => "julia",

        "kdl" => "kdl",

        "kotlin" => "kotlin",

        "kt" => "kotlin",

        "kts" => "kotlin",

        "lean" => "lean",

        "lisp" => "commonlisp",

        "lua" => "lua",

        "m" => "matlab",

        "markdown" => "markdown",

        "matlab" => "matlab",

        "md" => "markdown",

        "mdx" => "markdown",

        "meson" => "meson",

        "mjs" => "javascript",

        "ml" => "ocaml",

        "mm" => "objc",

        "mts" => "typescript",

        "mysql" => "sql",

        "nasm" => "x86asm",

        "nginx" => "nginx",

        "ninja" => "ninja",

        "nix" => "nix",

        "objc" => "objc",

        "objective-c" => "objc",

        "ocaml" => "ocaml",

        "patch" => "diff",

        "pbtxt" => "textproto",

        "perl" => "perl",

        "php" => "php",

        "pl" => "perl",

        "pm" => "perl",

        "postgres" => "sql",

        "postgresql" => "sql",

        "postscript" => "postscript",

        "powershell" => "powershell",

        "pro" => "prolog",

        "prolog" => "prolog",

        "ps" => "postscript",

        "ps1" => "powershell",

        "pwsh" => "powershell",

        "py" => "python",

        "py3" => "python",

        "python" => "python",

        "python3" => "python",

        "query" => "query",

        "r" => "r",

        "rb" => "ruby",

        "res" => "rescript",

        "rescript" => "rescript",

        "rkt" => "scheme",

        "rlang" => "r",

        "ron" => "ron",

        "rq" => "sparql",

        "rs" => "rust",

        "ruby" => "ruby",

        "rust" => "rust",

        "sass" => "scss",

        "scala" => "scala",

        "scheme" => "scheme",

        "scm" => "query",

        "scss" => "scss",

        "sh" => "bash",

        "shell" => "bash",

        "sparql" => "sparql",

        "sql" => "sql",

        "sqlite" => "sql",

        "ss" => "scheme",

        "ssh-config" => "ssh-config",

        "starlark" => "starlark",

        "sv" => "verilog",

        "svelte" => "svelte",

        "svg" => "xml",

        "swift" => "swift",

        "systemverilog" => "verilog",

        "terraform" => "hcl",

        "textpb" => "textproto",

        "textproto" => "textproto",

        "tf" => "hcl",

        "thrift" => "thrift",

        "tla" => "tlaplus",

        "tlaplus" => "tlaplus",

        "toml" => "toml",

        "ts" => "typescript",

        "tsx" => "tsx",

        "typ" => "typst",

        "typescript" => "typescript",

        "typst" => "typst",

        "ua" => "uiua",

        "uiua" => "uiua",

        "v" => "verilog",

        "vb" => "vb",

        "vbnet" => "vb",

        "verilog" => "verilog",

        "vert" => "glsl",

        "vhd" => "vhdl",

        "vhdl" => "vhdl",

        "vim" => "vim",

        "viml" => "vim",

        "vimscript" => "vim",

        "visualbasic" => "vb",

        "vue" => "vue",

        "x86" => "x86asm",

        "x86asm" => "x86asm",

        "xml" => "xml",

        "xsl" => "xml",

        "xslt" => "xml",

        "yaml" => "yaml",

        "yml" => "yaml",

        "yuri" => "yuri",

        "zig" => "zig",

        "zsh" => "zsh",

        _ => return None,
    })
}

// =============================================================================
// Language grammar re-exports based on enabled features.
// Each module provides:
// - `language()` - Returns the tree-sitter Language
// - `HIGHLIGHTS_QUERY` - The highlight query string
// - `INJECTIONS_QUERY` - The injection query string
// - `LOCALS_QUERY` - The locals query string
// =============================================================================

#[cfg(feature = "lang-ada")]
pub use arborium_ada as lang_ada;

#[cfg(feature = "lang-agda")]
pub use arborium_agda as lang_agda;

#[cfg(feature = "lang-asciidoc")]
pub use arborium_asciidoc as lang_asciidoc;

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

#[cfg(feature = "lang-markdown")]
pub use arborium_markdown as lang_markdown;

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

#[cfg(feature = "lang-postscript")]
pub use arborium_postscript as lang_postscript;

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
