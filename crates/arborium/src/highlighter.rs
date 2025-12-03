//! High-level syntax highlighting API with automatic language injection support.
//!
//! This module provides a simple, batteries-included API for syntax highlighting
//! that automatically handles language injections (e.g., CSS in HTML `<style>` tags,
//! JavaScript in `<script>` tags).
//!
//! # Example
//!
//! ```rust,ignore
//! use arborium::highlighter::Highlighter;
//!
//! let mut highlighter = Highlighter::new();
//! let html = highlighter.highlight_to_html("svelte", r#"
//!     <script>
//!         let name = "world";
//!     </script>
//!     <h1>Hello {name}!</h1>
//!     <style>
//!         h1 { color: red; }
//!     </style>
//! "#)?;
//! ```

use std::collections::HashMap;
use std::io::{self, Write};

use crate::highlights;
use crate::html;
use crate::tree_sitter_highlight::Highlighter as TsHighlighter;
use crate::tree_sitter_highlight::{Highlight, HighlightConfiguration, HighlightEvent};

/// Error type for highlighting operations
#[derive(Debug)]
pub enum HighlightError {
    /// The requested language is not supported
    UnsupportedLanguage(String),
    /// Error compiling the grammar's queries
    QueryError(String),
    /// Error during highlighting
    HighlightError(String),
    /// IO error during rendering
    IoError(io::Error),
}

impl std::fmt::Display for HighlightError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HighlightError::UnsupportedLanguage(lang) => {
                write!(f, "Unsupported language: {}", lang)
            }
            HighlightError::QueryError(e) => write!(f, "Query error: {}", e),
            HighlightError::HighlightError(e) => write!(f, "Highlight error: {}", e),
            HighlightError::IoError(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl std::error::Error for HighlightError {}

impl From<io::Error> for HighlightError {
    fn from(e: io::Error) -> Self {
        HighlightError::IoError(e)
    }
}

/// High-level syntax highlighter with automatic injection support.
///
/// This struct manages highlight configurations for all enabled languages
/// and automatically handles language injections. Language configurations
/// are lazily initialized on first use to improve startup performance.
pub struct Highlighter {
    /// Language configurations (lazily populated)
    #[cfg(test)]
    pub(crate) configs: HashMap<&'static str, HighlightConfiguration>,
    #[cfg(not(test))]
    configs: HashMap<&'static str, HighlightConfiguration>,
    ts_highlighter: TsHighlighter,
    /// Cached highlight names for configuration
    names: Vec<String>,
}

impl Default for Highlighter {
    fn default() -> Self {
        Self::new()
    }
}

impl Highlighter {
    /// Create a new highlighter. Language configurations are lazily initialized
    /// on first use for better startup performance.
    pub fn new() -> Self {
        Self {
            configs: HashMap::new(),
            ts_highlighter: TsHighlighter::new(),
            names: crate::HIGHLIGHT_NAMES
                .iter()
                .map(|s| s.to_string())
                .collect(),
        }
    }

    /// Ensure a language configuration is loaded, initializing it if needed.
    /// Returns true if the language is available (even if init failed previously).
    fn ensure_config(&mut self, language: &str) -> bool {
        // First normalize to get the canonical name
        let normalized = Self::normalize_language(language);

        // Already loaded?
        if self.configs.contains_key(normalized) {
            return true;
        }

        // Find the static string for this language
        let static_name = Self::available_languages()
            .iter()
            .find(|&&s| s == normalized)
            .copied();

        let Some(static_name) = static_name else {
            return false;
        };

        // Try to create the config
        if let Some(config) = Self::create_config(static_name, &self.names) {
            self.configs.insert(static_name, config);
            return true;
        }

        false
    }

    /// Create a highlight configuration for a language.
    /// This is where the actual grammar initialization happens.
    #[allow(unused_variables)]
    fn create_config(language: &'static str, names: &[String]) -> Option<HighlightConfiguration> {
        macro_rules! try_lang {
            ($feature:literal, $module:ident, $primary:literal) => {
                #[cfg(feature = $feature)]
                if language == $primary {
                    return HighlightConfiguration::new(
                        crate::$module::language().into(),
                        $primary,
                        crate::$module::HIGHLIGHTS_QUERY,
                        crate::$module::INJECTIONS_QUERY,
                        crate::$module::LOCALS_QUERY,
                    )
                    .ok()
                    .map(|mut config| {
                        config.configure(names);
                        config
                    });
                }
            };
        }

        // Core languages for injections
        try_lang!("lang-javascript", lang_javascript, "javascript");
        try_lang!("lang-css", lang_css, "css");
        try_lang!("lang-typescript", lang_typescript, "typescript");

        // All other languages
        try_lang!("lang-ada", lang_ada, "ada");
        try_lang!("lang-agda", lang_agda, "agda");
        try_lang!("lang-asm", lang_asm, "asm");
        try_lang!("lang-awk", lang_awk, "awk");
        try_lang!("lang-bash", lang_bash, "bash");
        try_lang!("lang-batch", lang_batch, "batch");
        try_lang!("lang-c", lang_c, "c");
        try_lang!("lang-c-sharp", lang_c_sharp, "c-sharp");
        try_lang!("lang-caddy", lang_caddy, "caddy");
        try_lang!("lang-capnp", lang_capnp, "capnp");
        try_lang!("lang-clojure", lang_clojure, "clojure");
        try_lang!("lang-cmake", lang_cmake, "cmake");
        try_lang!("lang-commonlisp", lang_commonlisp, "commonlisp");
        try_lang!("lang-cpp", lang_cpp, "cpp");
        try_lang!("lang-d", lang_d, "d");
        try_lang!("lang-dart", lang_dart, "dart");
        try_lang!("lang-devicetree", lang_devicetree, "devicetree");
        try_lang!("lang-diff", lang_diff, "diff");
        try_lang!("lang-dockerfile", lang_dockerfile, "dockerfile");
        try_lang!("lang-dot", lang_dot, "dot");
        try_lang!("lang-elisp", lang_elisp, "elisp");
        try_lang!("lang-elixir", lang_elixir, "elixir");
        try_lang!("lang-elm", lang_elm, "elm");
        try_lang!("lang-erlang", lang_erlang, "erlang");
        try_lang!("lang-fish", lang_fish, "fish");
        try_lang!("lang-fsharp", lang_fsharp, "fsharp");
        try_lang!("lang-gleam", lang_gleam, "gleam");
        try_lang!("lang-glsl", lang_glsl, "glsl");
        try_lang!("lang-go", lang_go, "go");
        try_lang!("lang-graphql", lang_graphql, "graphql");
        try_lang!("lang-haskell", lang_haskell, "haskell");
        try_lang!("lang-hcl", lang_hcl, "hcl");
        try_lang!("lang-hlsl", lang_hlsl, "hlsl");
        try_lang!("lang-html", lang_html, "html");
        try_lang!("lang-ini", lang_ini, "ini");
        try_lang!("lang-java", lang_java, "java");
        try_lang!("lang-jinja2", lang_jinja2, "jinja2");
        try_lang!("lang-jq", lang_jq, "jq");
        try_lang!("lang-json", lang_json, "json");
        try_lang!("lang-julia", lang_julia, "julia");
        try_lang!("lang-kdl", lang_kdl, "kdl");
        try_lang!("lang-kotlin", lang_kotlin, "kotlin");
        try_lang!("lang-lean", lang_lean, "lean");
        try_lang!("lang-lua", lang_lua, "lua");
        try_lang!("lang-matlab", lang_matlab, "matlab");
        try_lang!("lang-meson", lang_meson, "meson");
        try_lang!("lang-nginx", lang_nginx, "nginx");
        try_lang!("lang-ninja", lang_ninja, "ninja");
        try_lang!("lang-nix", lang_nix, "nix");
        try_lang!("lang-objc", lang_objc, "objc");
        try_lang!("lang-ocaml", lang_ocaml, "ocaml");
        try_lang!("lang-perl", lang_perl, "perl");
        try_lang!("lang-php", lang_php, "php");
        try_lang!("lang-powershell", lang_powershell, "powershell");
        try_lang!("lang-prolog", lang_prolog, "prolog");
        try_lang!("lang-python", lang_python, "python");
        try_lang!("lang-query", lang_query, "query");
        try_lang!("lang-r", lang_r, "r");
        try_lang!("lang-rescript", lang_rescript, "rescript");
        try_lang!("lang-ron", lang_ron, "ron");
        try_lang!("lang-ruby", lang_ruby, "ruby");
        try_lang!("lang-rust", lang_rust, "rust");
        try_lang!("lang-scala", lang_scala, "scala");
        try_lang!("lang-scheme", lang_scheme, "scheme");
        try_lang!("lang-scss", lang_scss, "scss");
        try_lang!("lang-sparql", lang_sparql, "sparql");
        try_lang!("lang-sql", lang_sql, "sql");
        try_lang!("lang-ssh-config", lang_ssh_config, "ssh-config");
        try_lang!("lang-starlark", lang_starlark, "starlark");
        try_lang!("lang-svelte", lang_svelte, "svelte");
        try_lang!("lang-swift", lang_swift, "swift");
        try_lang!("lang-textproto", lang_textproto, "textproto");
        try_lang!("lang-thrift", lang_thrift, "thrift");
        try_lang!("lang-tlaplus", lang_tlaplus, "tlaplus");
        try_lang!("lang-toml", lang_toml, "toml");
        try_lang!("lang-tsx", lang_tsx, "tsx");
        try_lang!("lang-typst", lang_typst, "typst");
        try_lang!("lang-uiua", lang_uiua, "uiua");
        try_lang!("lang-vb", lang_vb, "vb");
        try_lang!("lang-verilog", lang_verilog, "verilog");
        try_lang!("lang-vhdl", lang_vhdl, "vhdl");
        try_lang!("lang-vue", lang_vue, "vue");
        try_lang!("lang-x86asm", lang_x86asm, "x86asm");
        try_lang!("lang-xml", lang_xml, "xml");
        try_lang!("lang-yaml", lang_yaml, "yaml");
        try_lang!("lang-yuri", lang_yuri, "yuri");
        try_lang!("lang-zig", lang_zig, "zig");

        None
    }

    /// Get the list of available languages (based on enabled features).
    /// This doesn't initialize any configurations.
    fn available_languages() -> &'static [&'static str] {
        &[
            #[cfg(feature = "lang-ada")]
            "ada",
            #[cfg(feature = "lang-agda")]
            "agda",
            #[cfg(feature = "lang-asm")]
            "asm",
            #[cfg(feature = "lang-awk")]
            "awk",
            #[cfg(feature = "lang-bash")]
            "bash",
            #[cfg(feature = "lang-batch")]
            "batch",
            #[cfg(feature = "lang-c")]
            "c",
            #[cfg(feature = "lang-c-sharp")]
            "c-sharp",
            #[cfg(feature = "lang-caddy")]
            "caddy",
            #[cfg(feature = "lang-capnp")]
            "capnp",
            #[cfg(feature = "lang-clojure")]
            "clojure",
            #[cfg(feature = "lang-cmake")]
            "cmake",
            #[cfg(feature = "lang-commonlisp")]
            "commonlisp",
            #[cfg(feature = "lang-cpp")]
            "cpp",
            #[cfg(feature = "lang-css")]
            "css",
            #[cfg(feature = "lang-d")]
            "d",
            #[cfg(feature = "lang-dart")]
            "dart",
            #[cfg(feature = "lang-devicetree")]
            "devicetree",
            #[cfg(feature = "lang-diff")]
            "diff",
            #[cfg(feature = "lang-dockerfile")]
            "dockerfile",
            #[cfg(feature = "lang-dot")]
            "dot",
            #[cfg(feature = "lang-elisp")]
            "elisp",
            #[cfg(feature = "lang-elixir")]
            "elixir",
            #[cfg(feature = "lang-elm")]
            "elm",
            #[cfg(feature = "lang-erlang")]
            "erlang",
            #[cfg(feature = "lang-fish")]
            "fish",
            #[cfg(feature = "lang-fsharp")]
            "fsharp",
            #[cfg(feature = "lang-gleam")]
            "gleam",
            #[cfg(feature = "lang-glsl")]
            "glsl",
            #[cfg(feature = "lang-go")]
            "go",
            #[cfg(feature = "lang-graphql")]
            "graphql",
            #[cfg(feature = "lang-haskell")]
            "haskell",
            #[cfg(feature = "lang-hcl")]
            "hcl",
            #[cfg(feature = "lang-hlsl")]
            "hlsl",
            #[cfg(feature = "lang-html")]
            "html",
            #[cfg(feature = "lang-ini")]
            "ini",
            #[cfg(feature = "lang-java")]
            "java",
            #[cfg(feature = "lang-javascript")]
            "javascript",
            #[cfg(feature = "lang-jinja2")]
            "jinja2",
            #[cfg(feature = "lang-jq")]
            "jq",
            #[cfg(feature = "lang-json")]
            "json",
            #[cfg(feature = "lang-julia")]
            "julia",
            #[cfg(feature = "lang-kdl")]
            "kdl",
            #[cfg(feature = "lang-kotlin")]
            "kotlin",
            #[cfg(feature = "lang-lean")]
            "lean",
            #[cfg(feature = "lang-lua")]
            "lua",
            #[cfg(feature = "lang-matlab")]
            "matlab",
            #[cfg(feature = "lang-meson")]
            "meson",
            #[cfg(feature = "lang-nginx")]
            "nginx",
            #[cfg(feature = "lang-ninja")]
            "ninja",
            #[cfg(feature = "lang-nix")]
            "nix",
            #[cfg(feature = "lang-objc")]
            "objc",
            #[cfg(feature = "lang-ocaml")]
            "ocaml",
            #[cfg(feature = "lang-perl")]
            "perl",
            #[cfg(feature = "lang-php")]
            "php",
            #[cfg(feature = "lang-powershell")]
            "powershell",
            #[cfg(feature = "lang-prolog")]
            "prolog",
            #[cfg(feature = "lang-python")]
            "python",
            #[cfg(feature = "lang-query")]
            "query",
            #[cfg(feature = "lang-r")]
            "r",
            #[cfg(feature = "lang-rescript")]
            "rescript",
            #[cfg(feature = "lang-ron")]
            "ron",
            #[cfg(feature = "lang-ruby")]
            "ruby",
            #[cfg(feature = "lang-rust")]
            "rust",
            #[cfg(feature = "lang-scala")]
            "scala",
            #[cfg(feature = "lang-scheme")]
            "scheme",
            #[cfg(feature = "lang-scss")]
            "scss",
            #[cfg(feature = "lang-sparql")]
            "sparql",
            #[cfg(feature = "lang-sql")]
            "sql",
            #[cfg(feature = "lang-ssh-config")]
            "ssh-config",
            #[cfg(feature = "lang-starlark")]
            "starlark",
            #[cfg(feature = "lang-svelte")]
            "svelte",
            #[cfg(feature = "lang-swift")]
            "swift",
            #[cfg(feature = "lang-textproto")]
            "textproto",
            #[cfg(feature = "lang-thrift")]
            "thrift",
            #[cfg(feature = "lang-tlaplus")]
            "tlaplus",
            #[cfg(feature = "lang-toml")]
            "toml",
            #[cfg(feature = "lang-tsx")]
            "tsx",
            #[cfg(feature = "lang-typescript")]
            "typescript",
            #[cfg(feature = "lang-typst")]
            "typst",
            #[cfg(feature = "lang-uiua")]
            "uiua",
            #[cfg(feature = "lang-vb")]
            "vb",
            #[cfg(feature = "lang-verilog")]
            "verilog",
            #[cfg(feature = "lang-vhdl")]
            "vhdl",
            #[cfg(feature = "lang-vue")]
            "vue",
            #[cfg(feature = "lang-x86asm")]
            "x86asm",
            #[cfg(feature = "lang-xml")]
            "xml",
            #[cfg(feature = "lang-yaml")]
            "yaml",
            #[cfg(feature = "lang-yuri")]
            "yuri",
            #[cfg(feature = "lang-zig")]
            "zig",
        ]
    }

    /// Get a reference to a language configuration by name.
    /// Note: This will NOT lazily initialize the config. Use `get_config_mut` for that.
    pub fn get_config(&self, language: &str) -> Option<&HighlightConfiguration> {
        // Normalize language name
        let normalized = Self::normalize_language(language);
        self.configs.get(normalized)
    }

    /// Get a reference to a language configuration by name, initializing it if needed.
    pub fn get_config_mut(&mut self, language: &str) -> Option<&HighlightConfiguration> {
        self.ensure_config(language);
        let normalized = Self::normalize_language(language);
        self.configs.get(normalized)
    }

    /// Normalize language name to canonical form
    fn normalize_language(language: &str) -> &str {
        match language {
            "js" | "jsx" | "mjs" | "cjs" => "javascript",
            "ts" | "mts" | "cts" => "typescript",
            "py" | "py3" | "python3" => "python",
            "rb" => "ruby",
            "rs" => "rust",
            "sh" | "shell" => "bash",
            "yml" => "yaml",
            "htm" => "html",
            "cs" | "csharp" => "c-sharp",
            "c++" | "cxx" | "hpp" => "cpp",
            "golang" => "go",
            "hs" => "haskell",
            "ex" | "exs" => "elixir",
            "erl" => "erlang",
            "kt" | "kts" => "kotlin",
            "ml" => "ocaml",
            "pl" | "pm" => "perl",
            "ps1" | "pwsh" => "powershell",
            "sass" => "scss",
            "tf" | "terraform" => "hcl",
            "bat" | "cmd" => "batch",
            "dockerfile" | "docker" => "dockerfile",
            "h" => "c",
            "lisp" | "cl" => "commonlisp",
            "el" | "emacs-lisp" => "elisp",
            "jl" => "julia",
            "m" => "matlab",
            "mm" | "objective-c" => "objc",
            "json" | "jsonc" => "json",
            "scm" => "query",
            "rlang" => "r",
            "res" => "rescript",
            "rq" => "sparql",
            "mysql" | "postgresql" | "postgres" | "sqlite" => "sql",
            "pbtxt" | "textpb" => "textproto",
            "tla" => "tlaplus",
            "typ" => "typst",
            "ua" => "uiua",
            "vbnet" | "visualbasic" => "vb",
            "v" | "sv" | "systemverilog" => "verilog",
            "vhd" => "vhdl",
            "nasm" | "x86" => "x86asm",
            "xsl" | "xslt" | "svg" => "xml",
            "jinja" | "j2" => "jinja2",
            "gql" => "graphql",
            "vert" | "frag" => "glsl",
            "conf" | "cfg" => "ini",
            "bzl" | "bazel" => "starlark",
            "patch" => "diff",
            "dlang" => "d",
            "f#" | "fs" => "fsharp",
            other => other,
        }
    }

    /// Check if a language is supported (based on enabled features).
    /// This doesn't initialize the language configuration.
    pub fn is_supported(&self, language: &str) -> bool {
        let normalized = Self::normalize_language(language);
        Self::available_languages().contains(&normalized)
    }

    /// Get list of supported languages (based on enabled features).
    /// This doesn't initialize any language configurations.
    pub fn supported_languages(&self) -> Vec<&'static str> {
        Self::available_languages().to_vec()
    }

    /// Highlight source code and return HTML string.
    ///
    /// This is the main entry point for highlighting. It automatically handles
    /// language injections (e.g., CSS/JS in HTML).
    pub fn highlight_to_html(
        &mut self,
        language: &str,
        source: &str,
    ) -> Result<String, HighlightError> {
        let mut output = Vec::new();
        self.highlight_to_html_writer(&mut output, language, source)?;
        String::from_utf8(output).map_err(|e| HighlightError::HighlightError(e.to_string()))
    }

    /// Highlight source code and write HTML to a writer.
    pub fn highlight_to_html_writer<W: Write>(
        &mut self,
        writer: &mut W,
        language: &str,
        source: &str,
    ) -> Result<(), HighlightError> {
        let normalized = Self::normalize_language(language);

        // Ensure the primary language is loaded
        if !self.ensure_config(language) {
            return Err(HighlightError::UnsupportedLanguage(language.to_string()));
        }

        // Pre-load common injection languages if they might be needed
        // (HTML can inject CSS, JS, etc.)
        if normalized == "html" || normalized == "svelte" || normalized == "vue" {
            self.ensure_config("css");
            self.ensure_config("javascript");
            self.ensure_config("typescript");
        }

        let config = self
            .configs
            .get(normalized)
            .ok_or_else(|| HighlightError::UnsupportedLanguage(language.to_string()))?;

        let highlights = self
            .ts_highlighter
            .highlight(config, source.as_bytes(), None, |lang| {
                self.configs.get(lang)
            })
            .map_err(|e| HighlightError::HighlightError(e.to_string()))?;

        let mut tag_stack: Vec<&'static str> = Vec::new();

        for event in highlights {
            let event = event.map_err(|e| HighlightError::HighlightError(e.to_string()))?;
            match event {
                HighlightEvent::Source { start, end } => {
                    html::write_escaped(writer, &source[start..end])?;
                }
                HighlightEvent::HighlightStart(Highlight(i)) => {
                    if let Some(tag) = highlights::tag(i) {
                        write!(writer, "<a-{tag}>")?;
                        tag_stack.push(tag);
                    }
                }
                HighlightEvent::HighlightEnd => {
                    if let Some(tag) = tag_stack.pop() {
                        write!(writer, "</a-{tag}>")?;
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    #[cfg(feature = "lang-rust")]
    fn test_rust_highlighting() {
        let mut highlighter = Highlighter::new();
        let html = highlighter
            .highlight_to_html("rust", "fn main() {}")
            .unwrap();
        assert!(html.contains("<a-k>fn</a-k>")); // keyword
        assert!(html.contains("<a-f>main</a-f>")); // function
    }

    #[test]
    #[cfg(feature = "lang-html")]
    fn test_html_css_injection() {
        let mut highlighter = Highlighter::new();
        let html = highlighter
            .highlight_to_html("html", r#"<style>h1 { color: red; }</style>"#)
            .unwrap();
        assert!(
            html.contains("color"),
            "CSS property 'color' should be in output"
        );
    }

    #[test]
    #[cfg(feature = "lang-html")]
    fn test_html_js_injection() {
        let mut highlighter = Highlighter::new();
        let html = highlighter
            .highlight_to_html("html", r#"<script>let x = 1;</script>"#)
            .unwrap();
        assert!(
            html.contains("<a-k>let</a-k>"),
            "JS keyword 'let' should be highlighted"
        );
    }

    // Note: Svelte and Vue tests are in svelte_tests.rs and injection_tests.rs
}
