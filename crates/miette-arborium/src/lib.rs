//! Arborium-powered syntax highlighter for miette diagnostics.
//!
//! This crate provides a `miette::Highlighter` implementation using arborium's
//! tree-sitter-based syntax highlighting, giving you accurate, language-aware
//! highlighting in your error messages.
//!
//! # Example
//!
//! ```rust,ignore
//! use miette::GraphicalReportHandler;
//! use miette_arborium::ArboriumHighlighter;
//!
//! let handler = GraphicalReportHandler::new()
//!     .with_syntax_highlighting(ArboriumHighlighter::new());
//! ```
//!
//! # Theme Support
//!
//! The highlighter uses Monokai by default, but you can customize it:
//!
//! ```rust,ignore
//! use miette_arborium::ArboriumHighlighter;
//! use arborium::theme::builtin;
//!
//! let highlighter = ArboriumHighlighter::with_theme(builtin::dracula());
//! ```
//!
//! # Feature Flags
//!
//! This crate mirrors arborium's language feature flags. By default, all
//! permissively-licensed languages are enabled. For smaller builds:
//!
//! ```toml
//! [dependencies]
//! miette-arborium = { version = "0.1", default-features = false, features = ["lang-rust", "lang-toml"] }
//! ```

use std::path::Path;
use std::sync::RwLock;

use arborium::Highlighter as ArboriumCoreHighlighter;
use arborium::theme::{Style, Theme, builtin};
use arborium::tree_sitter_highlight::{Highlight, HighlightConfiguration, HighlightEvent};
use miette::SpanContents;
use miette::highlighters::{Highlighter, HighlighterState};
use owo_colors::{Rgb, Style as OwoStyle, Styled};

/// A miette syntax highlighter powered by arborium's tree-sitter grammars.
///
/// This highlighter uses arborium to provide accurate, language-aware syntax
/// highlighting for source code snippets in miette diagnostic output.
pub struct ArboriumHighlighter {
    // Using RwLock because miette's Highlighter trait only gives us &self,
    // but we need to lazily initialize language configs via get_config_mut.
    // RwLock is needed for thread-safety (miette requires Sync + Send).
    highlighter: RwLock<ArboriumCoreHighlighter>,
    theme: &'static Theme,
}

impl Default for ArboriumHighlighter {
    fn default() -> Self {
        Self::new()
    }
}

impl ArboriumHighlighter {
    /// Create a new highlighter with the default theme (Monokai).
    pub fn new() -> Self {
        Self {
            highlighter: RwLock::new(ArboriumCoreHighlighter::new()),
            theme: builtin::monokai(),
        }
    }

    /// Create a new highlighter with a custom theme.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use miette_arborium::ArboriumHighlighter;
    /// use arborium::theme::builtin;
    ///
    /// let highlighter = ArboriumHighlighter::with_theme(builtin::dracula());
    /// ```
    pub fn with_theme(theme: &'static Theme) -> Self {
        Self {
            highlighter: RwLock::new(ArboriumCoreHighlighter::new()),
            theme,
        }
    }

    /// Detect language from SpanContents using language hint, filename, or first line.
    fn detect_language(&self, contents: &dyn SpanContents<'_>) -> Option<&'static str> {
        // First try the explicit language hint
        if let Some(lang) = contents.language()
            && self.highlighter.read().unwrap().is_supported(lang)
        {
            return Some(self.normalize_language(lang));
        }

        // Try to detect from filename extension
        if let Some(name) = contents.name()
            && let Some(lang) = self.language_from_path(name)
        {
            return Some(lang);
        }

        None
    }

    /// Normalize language name to arborium's canonical form.
    fn normalize_language(&self, lang: &str) -> &'static str {
        // This matches arborium's highlighter normalization
        match lang.to_lowercase().as_str() {
            "js" | "jsx" | "mjs" | "cjs" | "javascript" => "javascript",
            "ts" | "mts" | "cts" | "typescript" => "typescript",
            "py" | "py3" | "python3" | "python" => "python",
            "rb" | "ruby" => "ruby",
            "rs" | "rust" => "rust",
            "sh" | "shell" | "bash" => "bash",
            "yml" | "yaml" => "yaml",
            "htm" | "html" => "html",
            "cs" | "csharp" | "c#" | "c-sharp" => "c-sharp",
            "c++" | "cxx" | "hpp" | "cpp" => "cpp",
            "golang" | "go" => "go",
            "hs" | "haskell" => "haskell",
            "ex" | "exs" | "elixir" => "elixir",
            "erl" | "erlang" => "erlang",
            "kt" | "kts" | "kotlin" => "kotlin",
            "ml" | "ocaml" => "ocaml",
            "pl" | "pm" | "perl" => "perl",
            "ps1" | "pwsh" | "powershell" => "powershell",
            "sass" | "scss" => "scss",
            "tf" | "terraform" | "hcl" => "hcl",
            "bat" | "cmd" | "batch" => "batch",
            "dockerfile" | "docker" => "dockerfile",
            "h" | "c" => "c",
            "lisp" | "cl" | "commonlisp" => "commonlisp",
            "el" | "emacs-lisp" | "elisp" => "elisp",
            "jl" | "julia" => "julia",
            "m" | "matlab" => "matlab",
            "mm" | "objective-c" | "objc" => "objc",
            "json" | "jsonc" => "json",
            "scm" | "query" => "query",
            "rlang" | "r" => "r",
            "res" | "rescript" => "rescript",
            "rq" | "sparql" => "sparql",
            "mysql" | "postgresql" | "postgres" | "sqlite" | "sql" => "sql",
            "pbtxt" | "textpb" | "textproto" => "textproto",
            "tla" | "tlaplus" => "tlaplus",
            "typ" | "typst" => "typst",
            "ua" | "uiua" => "uiua",
            "vbnet" | "visualbasic" | "vb" => "vb",
            "v" | "sv" | "systemverilog" | "verilog" => "verilog",
            "vhd" | "vhdl" => "vhdl",
            "nasm" | "x86" | "x86asm" => "x86asm",
            "xsl" | "xslt" | "svg" | "xml" => "xml",
            "jinja" | "j2" | "jinja2" => "jinja2",
            "gql" | "graphql" => "graphql",
            "vert" | "frag" | "glsl" => "glsl",
            "conf" | "cfg" | "ini" => "ini",
            "bzl" | "bazel" | "starlark" => "starlark",
            "patch" | "diff" => "diff",
            "dlang" | "d" => "d",
            "f#" | "fs" | "fsharp" => "fsharp",
            "toml" => "toml",
            "css" => "css",
            "java" => "java",
            "scala" => "scala",
            "swift" => "swift",
            "lua" => "lua",
            "php" => "php",
            "nix" => "nix",
            "zig" => "zig",
            "gleam" => "gleam",
            "svelte" => "svelte",
            "vue" => "vue",
            "tsx" => "tsx",
            "ada" => "ada",
            "agda" => "agda",
            "asm" => "asm",
            "awk" => "awk",
            "capnp" => "capnp",
            "clojure" => "clojure",
            "cmake" => "cmake",
            "dart" => "dart",
            "dot" => "dot",
            "elm" => "elm",
            "fish" => "fish",
            "jq" => "jq",
            "kdl" => "kdl",
            "lean" => "lean",
            "meson" => "meson",
            "nginx" => "nginx",
            "ninja" => "ninja",
            "prolog" => "prolog",
            "ron" => "ron",
            "scheme" => "scheme",
            "thrift" => "thrift",
            "vim" => "vim",
            "zsh" => "zsh",
            other => {
                // Return the original if we don't have a mapping
                // This is a bit of a hack since we need 'static, but miette
                // will fall back to blank highlighting if unsupported
                match other {
                    "javascript" => "javascript",
                    "typescript" => "typescript",
                    "python" => "python",
                    "ruby" => "ruby",
                    "rust" => "rust",
                    "bash" => "bash",
                    "yaml" => "yaml",
                    "html" => "html",
                    "c-sharp" => "c-sharp",
                    "cpp" => "cpp",
                    "go" => "go",
                    "haskell" => "haskell",
                    "elixir" => "elixir",
                    "erlang" => "erlang",
                    "kotlin" => "kotlin",
                    "ocaml" => "ocaml",
                    "perl" => "perl",
                    "powershell" => "powershell",
                    "scss" => "scss",
                    "hcl" => "hcl",
                    "batch" => "batch",
                    "dockerfile" => "dockerfile",
                    _ => "text", // fallback
                }
            }
        }
    }

    /// Detect language from file path extension.
    fn language_from_path(&self, path: &str) -> Option<&'static str> {
        let ext = Path::new(path).extension()?.to_str()?;
        let lang = match ext.to_lowercase().as_str() {
            "rs" => "rust",
            "py" | "pyi" | "pyw" => "python",
            "js" | "mjs" | "cjs" => "javascript",
            "ts" | "mts" | "cts" => "typescript",
            "tsx" => "tsx",
            "jsx" => "javascript",
            "rb" | "rake" | "gemspec" => "ruby",
            "go" => "go",
            "java" => "java",
            "kt" | "kts" => "kotlin",
            "scala" | "sc" => "scala",
            "c" | "h" => "c",
            "cpp" | "cc" | "cxx" | "hpp" | "hxx" | "hh" => "cpp",
            "cs" => "c-sharp",
            "swift" => "swift",
            "m" | "mm" => "objc",
            "php" => "php",
            "pl" | "pm" => "perl",
            "sh" | "bash" => "bash",
            "zsh" => "zsh",
            "fish" => "fish",
            "ps1" | "psm1" | "psd1" => "powershell",
            "bat" | "cmd" => "batch",
            "lua" => "lua",
            "vim" => "vim",
            "ex" | "exs" => "elixir",
            "erl" | "hrl" => "erlang",
            "hs" | "lhs" => "haskell",
            "ml" | "mli" => "ocaml",
            "fs" | "fsi" | "fsx" => "fsharp",
            "clj" | "cljs" | "cljc" | "edn" => "clojure",
            "scm" | "ss" => "scheme",
            "lisp" | "cl" | "el" => "commonlisp",
            "nix" => "nix",
            "zig" => "zig",
            "gleam" => "gleam",
            "dart" => "dart",
            "html" | "htm" => "html",
            "css" => "css",
            "scss" | "sass" => "scss",
            "json" | "jsonc" => "json",
            "yaml" | "yml" => "yaml",
            "toml" => "toml",
            "xml" | "xsl" | "xslt" | "svg" => "xml",
            "md" | "markdown" => "markdown",
            "sql" => "sql",
            "graphql" | "gql" => "graphql",
            "dockerfile" => "dockerfile",
            "tf" | "hcl" => "hcl",
            "ini" | "cfg" | "conf" => "ini",
            "diff" | "patch" => "diff",
            "asm" | "s" => "asm",
            "svelte" => "svelte",
            "vue" => "vue",
            "ron" => "ron",
            "kdl" => "kdl",
            "jq" => "jq",
            "awk" => "awk",
            "cmake" => "cmake",
            "ninja" => "ninja",
            "meson" => "meson",
            "glsl" | "vert" | "frag" => "glsl",
            "hlsl" => "hlsl",
            "wgsl" => "wgsl",
            "proto" => "textproto",
            "thrift" => "thrift",
            "capnp" => "capnp",
            "ada" | "adb" | "ads" => "ada",
            "agda" => "agda",
            "elm" => "elm",
            "lean" => "lean",
            "tla" => "tlaplus",
            "typ" => "typst",
            "r" | "R" => "r",
            "jl" => "julia",
            "v" | "sv" => "verilog",
            "vhd" | "vhdl" => "vhdl",
            "prolog" | "pro" => "prolog",
            _ => return None,
        };

        if self.highlighter.read().unwrap().is_supported(lang) {
            Some(lang)
        } else {
            None
        }
    }
}

impl Highlighter for ArboriumHighlighter {
    fn start_highlighter_state<'h>(
        &'h self,
        source: &dyn SpanContents<'_>,
    ) -> Box<dyn HighlighterState + 'h> {
        // Detect the language
        let language = self.detect_language(source);

        // Get source data as string
        let source_str = match std::str::from_utf8(source.data()) {
            Ok(s) => s,
            Err(_) => return Box::new(BlankHighlighterState),
        };

        // If we have a supported language, prepare the highlighted lines
        if let Some(lang) = language {
            // Use get_config_mut to lazily initialize the language config
            // We need to do all highlighting work inside the lock scope
            let mut highlighter = self.highlighter.write().unwrap();
            if let Some(config) = highlighter.get_config_mut(lang) {
                return Box::new(ArboriumHighlighterState::new(
                    source_str, config, self.theme,
                ));
            }
        }

        // Fall back to blank highlighting
        Box::new(BlankHighlighterState)
    }
}

/// Stateful highlighter that caches highlighted lines.
struct ArboriumHighlighterState<'h> {
    /// Pre-computed styled segments for each line
    lines: Vec<Vec<StyledSegment>>,
    /// Current line index
    current_line: usize,
    /// Theme reference for style conversion
    theme: &'h Theme,
}

/// A segment of text with its highlight index.
struct StyledSegment {
    text: String,
    highlight: Option<usize>,
}

impl<'h> ArboriumHighlighterState<'h> {
    fn new(source: &str, config: &HighlightConfiguration, theme: &'h Theme) -> Self {
        let mut ts_highlighter = arborium::tree_sitter_highlight::Highlighter::new();

        // Parse the entire source and collect highlight events
        let highlights = match ts_highlighter.highlight(config, source.as_bytes(), None, |_| None) {
            Ok(h) => h,
            Err(_) => {
                return Self {
                    lines: Self::plain_lines(source),
                    current_line: 0,
                    theme,
                };
            }
        };

        // Build segments from highlight events
        let mut segments: Vec<StyledSegment> = Vec::new();
        let mut highlight_stack: Vec<usize> = Vec::new();

        for event in highlights {
            let event = match event {
                Ok(e) => e,
                Err(_) => continue,
            };

            match event {
                HighlightEvent::Source { start, end } => {
                    let text = &source[start..end];
                    if !text.is_empty() {
                        segments.push(StyledSegment {
                            text: text.to_string(),
                            highlight: highlight_stack.last().copied(),
                        });
                    }
                }
                HighlightEvent::HighlightStart(Highlight(i)) => {
                    highlight_stack.push(i);
                }
                HighlightEvent::HighlightEnd => {
                    highlight_stack.pop();
                }
            }
        }

        // Split segments by lines
        let lines = Self::split_into_lines(segments);

        Self {
            lines,
            current_line: 0,
            theme,
        }
    }

    /// Create plain (unstyled) lines from source.
    fn plain_lines(source: &str) -> Vec<Vec<StyledSegment>> {
        source
            .lines()
            .map(|line| {
                vec![StyledSegment {
                    text: line.to_string(),
                    highlight: None,
                }]
            })
            .collect()
    }

    /// Split segments into lines, handling segments that span multiple lines.
    fn split_into_lines(segments: Vec<StyledSegment>) -> Vec<Vec<StyledSegment>> {
        let mut lines: Vec<Vec<StyledSegment>> = vec![Vec::new()];

        for segment in segments {
            let mut remaining = segment.text.as_str();

            while let Some(newline_pos) = remaining.find('\n') {
                // Add text before newline to current line
                if newline_pos > 0 {
                    lines.last_mut().unwrap().push(StyledSegment {
                        text: remaining[..newline_pos].to_string(),
                        highlight: segment.highlight,
                    });
                }

                // Start a new line
                lines.push(Vec::new());
                remaining = &remaining[newline_pos + 1..];
            }

            // Add remaining text (after last newline or entire text if no newlines)
            if !remaining.is_empty() {
                lines.last_mut().unwrap().push(StyledSegment {
                    text: remaining.to_string(),
                    highlight: segment.highlight,
                });
            }
        }

        lines
    }

    /// Convert arborium Style to owo_colors Style.
    fn convert_style(arborium_style: &Style) -> OwoStyle {
        let mut owo_style = OwoStyle::new();

        if let Some(color) = arborium_style.fg {
            owo_style = owo_style.color(Rgb(color.r, color.g, color.b));
        }

        if arborium_style.modifiers.bold {
            owo_style = owo_style.bold();
        }

        if arborium_style.modifiers.italic {
            owo_style = owo_style.italic();
        }

        if arborium_style.modifiers.underline {
            owo_style = owo_style.underline();
        }

        if arborium_style.modifiers.strikethrough {
            owo_style = owo_style.strikethrough();
        }

        owo_style
    }
}

impl HighlighterState for ArboriumHighlighterState<'_> {
    fn highlight_line<'s>(&mut self, line: &'s str) -> Vec<Styled<&'s str>> {
        // Get the pre-computed segments for this line
        if self.current_line >= self.lines.len() {
            // If we've run out of pre-computed lines, return unstyled
            self.current_line += 1;
            return vec![OwoStyle::new().style(line)];
        }

        let segments = &self.lines[self.current_line];
        self.current_line += 1;

        // We need to map our pre-computed segments back to slices of the input line.
        // The segments should match the line content, but we need to return &'s str.
        let mut result = Vec::new();
        let mut pos = 0;

        for segment in segments {
            let segment_len = segment.text.len();
            if pos + segment_len > line.len() {
                // Safety check: if segment extends beyond line, just take what's available
                let available = &line[pos..];
                if !available.is_empty() {
                    let style = segment
                        .highlight
                        .and_then(|i| self.theme.style(i))
                        .map(Self::convert_style)
                        .unwrap_or_default();
                    result.push(style.style(available));
                }
                break;
            }

            let slice = &line[pos..pos + segment_len];
            let style = segment
                .highlight
                .and_then(|i| self.theme.style(i))
                .map(Self::convert_style)
                .unwrap_or_default();
            result.push(style.style(slice));
            pos += segment_len;
        }

        // If there's remaining content in the line that wasn't covered by segments
        if pos < line.len() {
            result.push(OwoStyle::new().style(&line[pos..]));
        }

        // If no segments were produced, return the whole line unstyled
        if result.is_empty() {
            result.push(OwoStyle::new().style(line));
        }

        result
    }
}

/// Blank highlighter state that returns unstyled text.
struct BlankHighlighterState;

impl HighlighterState for BlankHighlighterState {
    fn highlight_line<'s>(&mut self, line: &'s str) -> Vec<Styled<&'s str>> {
        vec![OwoStyle::new().style(line)]
    }
}

// Re-export theme module for convenience
pub use arborium::theme;

#[cfg(test)]
mod tests {
    use super::*;

    struct TestSpanContents<'a> {
        data: &'a [u8],
        language: Option<&'a str>,
        name: Option<&'a str>,
    }

    impl<'a> SpanContents<'a> for TestSpanContents<'a> {
        fn data(&self) -> &'a [u8] {
            self.data
        }

        fn span(&self) -> &miette::SourceSpan {
            static SPAN: std::sync::LazyLock<miette::SourceSpan> = std::sync::LazyLock::new(|| {
                miette::SourceSpan::new(miette::SourceOffset::from(0), 0)
            });
            &SPAN
        }

        fn line(&self) -> usize {
            0
        }

        fn column(&self) -> usize {
            0
        }

        fn line_count(&self) -> usize {
            1
        }

        fn language(&self) -> Option<&str> {
            self.language
        }

        fn name(&self) -> Option<&str> {
            self.name
        }
    }

    #[test]
    fn test_language_detection_from_hint() {
        let highlighter = ArboriumHighlighter::new();

        let contents = TestSpanContents {
            data: b"fn main() {}",
            language: Some("rust"),
            name: None,
        };

        let detected = highlighter.detect_language(&contents);
        assert_eq!(detected, Some("rust"));
    }

    #[test]
    fn test_language_detection_from_filename() {
        let highlighter = ArboriumHighlighter::new();

        let contents = TestSpanContents {
            data: b"fn main() {}",
            language: None,
            name: Some("main.rs"),
        };

        let detected = highlighter.detect_language(&contents);
        assert_eq!(detected, Some("rust"));
    }

    #[test]
    fn test_language_normalization() {
        let highlighter = ArboriumHighlighter::new();

        assert_eq!(highlighter.normalize_language("js"), "javascript");
        assert_eq!(highlighter.normalize_language("py"), "python");
        assert_eq!(highlighter.normalize_language("rs"), "rust");
        assert_eq!(highlighter.normalize_language("Rust"), "rust");
        assert_eq!(highlighter.normalize_language("PYTHON"), "python");
    }

    #[test]
    fn test_highlighting_produces_output() {
        let highlighter = ArboriumHighlighter::new();

        let source = "fn main() {\n    println!(\"Hello\");\n}";
        let contents = TestSpanContents {
            data: source.as_bytes(),
            language: Some("rust"),
            name: None,
        };

        let mut state = highlighter.start_highlighter_state(&contents);

        // Highlight each line
        let line1 = state.highlight_line("fn main() {");
        let line2 = state.highlight_line("    println!(\"Hello\");");
        let line3 = state.highlight_line("}");

        // Each line should produce at least one styled segment
        assert!(!line1.is_empty());
        assert!(!line2.is_empty());
        assert!(!line3.is_empty());
    }

    #[test]
    fn test_unsupported_language_fallback() {
        let highlighter = ArboriumHighlighter::new();

        let contents = TestSpanContents {
            data: b"some code",
            language: Some("unknown_language_xyz"),
            name: None,
        };

        // Should not panic, should return blank highlighter
        let mut state = highlighter.start_highlighter_state(&contents);
        let result = state.highlight_line("some code");
        assert!(!result.is_empty());
    }
}
