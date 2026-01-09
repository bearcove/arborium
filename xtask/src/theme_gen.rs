//! Theme code generation - converts TOML themes to Rust code at build time.
//!
//! This module:
//! 1. Parses theme TOML files from crates/arborium-theme/themes/
//! 2. Generates builtin_generated.rs for the arborium-theme crate
//! 3. Exports parsed themes for use by serve.rs (CSS generation, etc.)
//!
//! This eliminates the runtime TOML dependency from arborium-theme and
//! avoids xtask depending on arborium-theme (which would be circular).

use camino::Utf8Path;
use fs_err as fs;
use owo_colors::OwoColorize;
use std::collections::HashMap;
use std::fmt::Write;

// ============================================================================
// Public types for use by other xtask modules (serve.rs, etc.)
// ============================================================================

/// RGB color.
#[derive(Debug, Clone, Copy)]
pub struct Color(pub u8, pub u8, pub u8);

impl Color {
    pub fn to_hex(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.0, self.1, self.2)
    }

    pub fn lighten(&self, amount: f32) -> Color {
        let r = (self.0 as f32 + (255.0 - self.0 as f32) * amount).min(255.0) as u8;
        let g = (self.1 as f32 + (255.0 - self.1 as f32) * amount).min(255.0) as u8;
        let b = (self.2 as f32 + (255.0 - self.2 as f32) * amount).min(255.0) as u8;
        Color(r, g, b)
    }

    pub fn darken(&self, amount: f32) -> Color {
        let r = (self.0 as f32 * (1.0 - amount)) as u8;
        let g = (self.1 as f32 * (1.0 - amount)) as u8;
        let b = (self.2 as f32 * (1.0 - amount)) as u8;
        Color(r, g, b)
    }
}

/// Parsed style from TOML.
#[derive(Debug, Default, Clone)]
pub struct Style {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
}

impl Style {
    pub fn is_empty(&self) -> bool {
        self.fg.is_none()
            && self.bg.is_none()
            && !self.bold
            && !self.italic
            && !self.underline
            && !self.strikethrough
    }
}

/// Parsed theme from TOML.
#[derive(Debug)]
pub struct Theme {
    pub name: String,
    pub is_dark: bool,
    pub source_url: Option<String>,
    pub background: Option<Color>,
    pub foreground: Option<Color>,
    pub styles: Vec<Style>,
}

impl Theme {
    /// Get style at index.
    pub fn style(&self, index: usize) -> Option<&Style> {
        self.styles.get(index)
    }

    /// Generate CSS for this theme with the given selector prefix.
    pub fn to_css(&self, selector_prefix: &str) -> String {
        let mut css = String::new();

        writeln!(css, "{selector_prefix} {{").unwrap();

        // Background and foreground
        if let Some(bg) = &self.background {
            writeln!(css, "  background: {};", bg.to_hex()).unwrap();
            writeln!(css, "  --bg: {};", bg.to_hex()).unwrap();
            let surface = if self.is_dark {
                bg.lighten(0.08)
            } else {
                bg.darken(0.05)
            };
            writeln!(css, "  --surface: {};", surface.to_hex()).unwrap();
        }
        if let Some(fg) = &self.foreground {
            writeln!(css, "  color: {};", fg.to_hex()).unwrap();
            writeln!(css, "  --fg: {};", fg.to_hex()).unwrap();
        }

        // Find function and keyword colors for accent
        let function_idx = HIGHLIGHTS.iter().position(|h| h.name == "function");
        let keyword_idx = HIGHLIGHTS.iter().position(|h| h.name == "keyword");
        let comment_idx = HIGHLIGHTS.iter().position(|h| h.name == "comment");

        // --accent: use function color, fallback to keyword, fallback to foreground
        let accent = function_idx
            .and_then(|i| self.styles.get(i))
            .and_then(|s| s.fg)
            .or_else(|| {
                keyword_idx
                    .and_then(|i| self.styles.get(i))
                    .and_then(|s| s.fg)
            })
            .or(self.foreground);
        if let Some(c) = accent {
            writeln!(css, "  --accent: {};", c.to_hex()).unwrap();
        }

        // --muted: use comment color
        let muted = comment_idx
            .and_then(|i| self.styles.get(i))
            .and_then(|s| s.fg);
        if let Some(c) = muted {
            writeln!(css, "  --muted: {};", c.to_hex()).unwrap();
        }

        writeln!(css, "}}").unwrap();

        // Generate styles for each highlight tag
        // Track emitted tags to avoid duplicates (multiple HIGHLIGHTS can share the same tag)
        let mut emitted_tags: std::collections::HashSet<&str> = std::collections::HashSet::new();
        for (i, def) in HIGHLIGHTS.iter().enumerate() {
            if def.tag.is_empty() || emitted_tags.contains(def.tag) {
                continue;
            }
            if let Some(style) = self.styles.get(i) {
                if style.is_empty() {
                    continue;
                }
                emitted_tags.insert(def.tag);
                write!(css, "{selector_prefix} a-{}", def.tag).unwrap();
                css.push_str(" {\n");
                if let Some(fg) = &style.fg {
                    writeln!(css, "  color: {};", fg.to_hex()).unwrap();
                }
                if style.bold {
                    writeln!(css, "  font-weight: bold;").unwrap();
                }
                if style.italic {
                    writeln!(css, "  font-style: italic;").unwrap();
                }
                if style.underline {
                    writeln!(css, "  text-decoration: underline;").unwrap();
                }
                if style.strikethrough {
                    writeln!(css, "  text-decoration: line-through;").unwrap();
                }
                css.push_str("}\n");
            }
        }

        css
    }
}

/// Highlight definition with name, tag for HTML, and parent tag for fallback.
#[derive(Debug, Clone)]
pub struct HighlightDef {
    pub name: &'static str,
    pub tag: &'static str,
    pub parent_tag: &'static str,
}

/// All highlight definitions - this is the source of truth for theme slots.
pub static HIGHLIGHTS: &[HighlightDef] = &[
    HighlightDef {
        name: "attribute",
        tag: "at",
        parent_tag: "",
    },
    HighlightDef {
        name: "constant",
        tag: "co",
        parent_tag: "",
    },
    HighlightDef {
        name: "constant.builtin",
        tag: "co",
        parent_tag: "co",
    },
    HighlightDef {
        name: "constructor",
        tag: "cr",
        parent_tag: "",
    },
    HighlightDef {
        name: "function.builtin",
        tag: "fb",
        parent_tag: "f",
    },
    HighlightDef {
        name: "function",
        tag: "f",
        parent_tag: "",
    },
    HighlightDef {
        name: "function.method",
        tag: "f",
        parent_tag: "f",
    },
    HighlightDef {
        name: "keyword",
        tag: "k",
        parent_tag: "",
    },
    HighlightDef {
        name: "keyword.conditional",
        tag: "k",
        parent_tag: "k",
    },
    HighlightDef {
        name: "keyword.coroutine",
        tag: "k",
        parent_tag: "k",
    },
    HighlightDef {
        name: "keyword.debug",
        tag: "k",
        parent_tag: "k",
    },
    HighlightDef {
        name: "keyword.exception",
        tag: "k",
        parent_tag: "k",
    },
    HighlightDef {
        name: "keyword.function",
        tag: "k",
        parent_tag: "k",
    },
    HighlightDef {
        name: "keyword.import",
        tag: "k",
        parent_tag: "k",
    },
    HighlightDef {
        name: "keyword.operator",
        tag: "o",
        parent_tag: "k",
    },
    HighlightDef {
        name: "keyword.repeat",
        tag: "k",
        parent_tag: "k",
    },
    HighlightDef {
        name: "keyword.return",
        tag: "k",
        parent_tag: "k",
    },
    HighlightDef {
        name: "keyword.type",
        tag: "k",
        parent_tag: "k",
    },
    HighlightDef {
        name: "operator",
        tag: "o",
        parent_tag: "",
    },
    HighlightDef {
        name: "property",
        tag: "pr",
        parent_tag: "",
    },
    HighlightDef {
        name: "punctuation",
        tag: "p",
        parent_tag: "",
    },
    HighlightDef {
        name: "punctuation.bracket",
        tag: "p",
        parent_tag: "p",
    },
    HighlightDef {
        name: "punctuation.delimiter",
        tag: "p",
        parent_tag: "p",
    },
    HighlightDef {
        name: "punctuation.special",
        tag: "p",
        parent_tag: "p",
    },
    HighlightDef {
        name: "string",
        tag: "s",
        parent_tag: "",
    },
    HighlightDef {
        name: "string.special",
        tag: "s",
        parent_tag: "s",
    },
    HighlightDef {
        name: "tag",
        tag: "tg",
        parent_tag: "",
    },
    HighlightDef {
        name: "tag.delimiter",
        tag: "tg",
        parent_tag: "tg",
    },
    HighlightDef {
        name: "tag.error",
        tag: "err",
        parent_tag: "tg",
    },
    HighlightDef {
        name: "type",
        tag: "t",
        parent_tag: "",
    },
    HighlightDef {
        name: "type.builtin",
        tag: "t",
        parent_tag: "t",
    },
    HighlightDef {
        name: "type.qualifier",
        tag: "t",
        parent_tag: "t",
    },
    HighlightDef {
        name: "variable",
        tag: "v",
        parent_tag: "",
    },
    HighlightDef {
        name: "variable.builtin",
        tag: "v",
        parent_tag: "v",
    },
    HighlightDef {
        name: "variable.parameter",
        tag: "v",
        parent_tag: "v",
    },
    HighlightDef {
        name: "comment",
        tag: "c",
        parent_tag: "",
    },
    HighlightDef {
        name: "comment.documentation",
        tag: "c",
        parent_tag: "c",
    },
    HighlightDef {
        name: "macro",
        tag: "m",
        parent_tag: "",
    },
    HighlightDef {
        name: "label",
        tag: "l",
        parent_tag: "",
    },
    HighlightDef {
        name: "diff.addition",
        tag: "da",
        parent_tag: "",
    },
    HighlightDef {
        name: "diff.deletion",
        tag: "dd",
        parent_tag: "",
    },
    HighlightDef {
        name: "number",
        tag: "n",
        parent_tag: "",
    },
    HighlightDef {
        name: "text.literal",
        tag: "tl",
        parent_tag: "",
    },
    HighlightDef {
        name: "text.emphasis",
        tag: "em",
        parent_tag: "",
    },
    HighlightDef {
        name: "text.strong",
        tag: "st",
        parent_tag: "",
    },
    HighlightDef {
        name: "text.uri",
        tag: "tu",
        parent_tag: "",
    },
    HighlightDef {
        name: "text.reference",
        tag: "tr",
        parent_tag: "",
    },
    HighlightDef {
        name: "string.escape",
        tag: "se",
        parent_tag: "s",
    },
    HighlightDef {
        name: "text.title",
        tag: "tt",
        parent_tag: "",
    },
    HighlightDef {
        name: "text.strikethrough",
        tag: "ts",
        parent_tag: "",
    },
    HighlightDef {
        name: "spell",
        tag: "",
        parent_tag: "",
    },
    HighlightDef {
        name: "embedded",
        tag: "eb",
        parent_tag: "",
    },
    HighlightDef {
        name: "error",
        tag: "err",
        parent_tag: "",
    },
    HighlightDef {
        name: "namespace",
        tag: "ns",
        parent_tag: "",
    },
    HighlightDef {
        name: "include",
        tag: "k",
        parent_tag: "k",
    },
    HighlightDef {
        name: "storageclass",
        tag: "k",
        parent_tag: "k",
    },
    HighlightDef {
        name: "repeat",
        tag: "k",
        parent_tag: "k",
    },
    HighlightDef {
        name: "conditional",
        tag: "k",
        parent_tag: "k",
    },
    HighlightDef {
        name: "exception",
        tag: "k",
        parent_tag: "k",
    },
    HighlightDef {
        name: "preproc",
        tag: "pp",
        parent_tag: "",
    },
    HighlightDef {
        name: "none",
        tag: "",
        parent_tag: "",
    },
    HighlightDef {
        name: "character",
        tag: "ch",
        parent_tag: "s",
    },
    HighlightDef {
        name: "character.special",
        tag: "ch",
        parent_tag: "ch",
    },
    HighlightDef {
        name: "variable.member",
        tag: "pr",
        parent_tag: "v",
    },
    HighlightDef {
        name: "function.definition",
        tag: "f",
        parent_tag: "f",
    },
    HighlightDef {
        name: "type.definition",
        tag: "t",
        parent_tag: "t",
    },
    HighlightDef {
        name: "function.call",
        tag: "f",
        parent_tag: "f",
    },
    HighlightDef {
        name: "keyword.modifier",
        tag: "k",
        parent_tag: "k",
    },
    HighlightDef {
        name: "keyword.directive",
        tag: "k",
        parent_tag: "k",
    },
    HighlightDef {
        name: "string.regexp",
        tag: "sr",
        parent_tag: "s",
    },
    HighlightDef {
        name: "nospell",
        tag: "",
        parent_tag: "",
    },
    HighlightDef {
        name: "float",
        tag: "n",
        parent_tag: "n",
    },
    HighlightDef {
        name: "boolean",
        tag: "bo",
        parent_tag: "co",
    },
];

/// Parse all themes from the themes directory.
pub fn parse_all_themes(crates_dir: &Utf8Path) -> Result<Vec<Theme>, String> {
    let themes_dir = crates_dir.join("arborium-theme/themes");
    let mut themes = Vec::new();

    let entries =
        fs::read_dir(&themes_dir).map_err(|e| format!("Failed to read themes dir: {e}"))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read dir entry: {e}"))?;
        let path = entry.path();

        if path.extension().is_some_and(|e| e == "toml") {
            let content =
                fs::read_to_string(&path).map_err(|e| format!("Failed to read {:?}: {e}", path))?;

            let theme = parse_theme_toml(&content)
                .map_err(|e| format!("Failed to parse {:?}: {e}", path))?;

            themes.push(theme);
        }
    }

    // Sort by name for deterministic output
    themes.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(themes)
}

// ============================================================================
// Internal types and functions for code generation
// ============================================================================

/// Parse a hex color string like "#ff0000" or "ff0000" into (r, g, b).
fn parse_hex(s: &str) -> Option<(u8, u8, u8)> {
    let s = s.strip_prefix('#').unwrap_or(s);
    if s.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&s[0..2], 16).ok()?;
    let g = u8::from_str_radix(&s[2..4], 16).ok()?;
    let b = u8::from_str_radix(&s[4..6], 16).ok()?;
    Some((r, g, b))
}

/// Internal parsed style (uses tuples instead of Color for codegen).
#[derive(Debug, Default, Clone)]
struct ParsedStyle {
    fg: Option<(u8, u8, u8)>,
    bg: Option<(u8, u8, u8)>,
    bold: bool,
    italic: bool,
    underline: bool,
    strikethrough: bool,
}

impl ParsedStyle {
    fn is_empty(&self) -> bool {
        self.fg.is_none()
            && self.bg.is_none()
            && !self.bold
            && !self.italic
            && !self.underline
            && !self.strikethrough
    }

    fn to_style(&self) -> Style {
        Style {
            fg: self.fg.map(|(r, g, b)| Color(r, g, b)),
            bg: self.bg.map(|(r, g, b)| Color(r, g, b)),
            bold: self.bold,
            italic: self.italic,
            underline: self.underline,
            strikethrough: self.strikethrough,
        }
    }
}

/// Internal parsed theme for codegen.
#[derive(Debug)]
struct ParsedTheme {
    name: String,
    is_dark: bool,
    source_url: Option<String>,
    background: Option<(u8, u8, u8)>,
    foreground: Option<(u8, u8, u8)>,
    styles: Vec<ParsedStyle>,
}

impl ParsedTheme {
    fn to_theme(&self) -> Theme {
        Theme {
            name: self.name.clone(),
            is_dark: self.is_dark,
            source_url: self.source_url.clone(),
            background: self.background.map(|(r, g, b)| Color(r, g, b)),
            foreground: self.foreground.map(|(r, g, b)| Color(r, g, b)),
            styles: self.styles.iter().map(|s| s.to_style()).collect(),
        }
    }
}

/// The highlight definitions - must match arborium_theme::highlights::HIGHLIGHTS order.
const HIGHLIGHT_NAMES: &[&str] = &[
    "attribute",
    "constant",
    "constant.builtin",
    "constructor",
    "function.builtin",
    "function",
    "function.method",
    "keyword",
    "keyword.conditional",
    "keyword.coroutine",
    "keyword.debug",
    "keyword.exception",
    "keyword.function",
    "keyword.import",
    "keyword.operator",
    "keyword.repeat",
    "keyword.return",
    "keyword.type",
    "operator",
    "property",
    "punctuation",
    "punctuation.bracket",
    "punctuation.delimiter",
    "punctuation.special",
    "string",
    "string.special",
    "tag",
    "tag.delimiter",
    "tag.error",
    "type",
    "type.builtin",
    "type.qualifier",
    "variable",
    "variable.builtin",
    "variable.parameter",
    "comment",
    "comment.documentation",
    "macro",
    "label",
    "diff.addition",
    "diff.deletion",
    "number",
    "text.literal",
    "text.emphasis",
    "text.strong",
    "text.uri",
    "text.reference",
    "string.escape",
    "text.title",
    "text.strikethrough",
    "spell",
    "embedded",
    "error",
    "namespace",
    "include",
    "storageclass",
    "repeat",
    "conditional",
    "exception",
    "preproc",
    "none",
    "character",
    "character.special",
    "variable.member",
    "function.definition",
    "type.definition",
    "function.call",
    "keyword.modifier",
    "keyword.directive",
    "string.regexp",
    "nospell",
    "float",
    "boolean",
];

/// Extra mappings from Helix theme names to our names.
const EXTRA_MAPPINGS: &[(&str, &str)] = &[
    ("keyword.control", "keyword"),
    ("keyword.storage", "keyword"),
    ("comment.line", "comment"),
    ("comment.block", "comment"),
    ("function.macro", "macro"),
    ("diff.plus", "diff.addition"),
    ("diff.minus", "diff.deletion"),
];

/// Parse a theme from TOML content.
fn parse_theme(toml_str: &str) -> Result<ParsedTheme, String> {
    let value: toml::Value = toml_str
        .parse()
        .map_err(|e| format!("TOML parse error: {e}"))?;
    let table = value
        .as_table()
        .ok_or_else(|| "Expected table".to_string())?;

    let name = table
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let is_dark = table
        .get("variant")
        .and_then(|v| v.as_str())
        .map(|v| v != "light")
        .unwrap_or(true);

    let source_url = table
        .get("source")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // Extract palette for color lookups
    let palette: HashMap<&str, (u8, u8, u8)> = table
        .get("palette")
        .and_then(|v| v.as_table())
        .map(|t| {
            t.iter()
                .filter_map(|(k, v)| v.as_str().and_then(parse_hex).map(|c| (k.as_str(), c)))
                .collect()
        })
        .unwrap_or_default();

    let resolve_color =
        |s: &str| -> Option<(u8, u8, u8)> { parse_hex(s).or_else(|| palette.get(s).copied()) };

    // Extract background and foreground
    let mut background = None;
    let mut foreground = None;

    if let Some(bg) = table.get("ui.background") {
        if let Some(bg_table) = bg.as_table() {
            if let Some(bg_str) = bg_table.get("bg").and_then(|v| v.as_str()) {
                background = resolve_color(bg_str);
            }
        }
    }
    if let Some(bg_str) = table.get("background").and_then(|v| v.as_str()) {
        background = resolve_color(bg_str);
    }

    if let Some(fg) = table.get("ui.foreground") {
        if let Some(fg_str) = fg.as_str() {
            foreground = resolve_color(fg_str);
        } else if let Some(fg_table) = fg.as_table() {
            if let Some(fg_str) = fg_table.get("fg").and_then(|v| v.as_str()) {
                foreground = resolve_color(fg_str);
            }
        }
    }
    if let Some(fg_str) = table.get("foreground").and_then(|v| v.as_str()) {
        foreground = resolve_color(fg_str);
    }

    // Parse a style value (string or table)
    let parse_style_value = |value: &toml::Value| -> ParsedStyle {
        let mut style = ParsedStyle::default();
        match value {
            toml::Value::String(s) => {
                style.fg = resolve_color(s);
            }
            toml::Value::Table(t) => {
                if let Some(fg) = t.get("fg").and_then(|v| v.as_str()) {
                    style.fg = resolve_color(fg);
                }
                if let Some(bg) = t.get("bg").and_then(|v| v.as_str()) {
                    style.bg = resolve_color(bg);
                }
                if let Some(mods) = t.get("modifiers").and_then(|v| v.as_array()) {
                    for m in mods {
                        if let Some(s) = m.as_str() {
                            match s {
                                "bold" => style.bold = true,
                                "italic" => style.italic = true,
                                "underlined" | "underline" => style.underline = true,
                                "crossed_out" | "strikethrough" => style.strikethrough = true,
                                _ => {}
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        style
    };

    // Initialize styles array
    let mut styles: Vec<ParsedStyle> = (0..HIGHLIGHT_NAMES.len())
        .map(|_| ParsedStyle::default())
        .collect();

    // Parse each highlight rule
    for (i, name) in HIGHLIGHT_NAMES.iter().enumerate() {
        if let Some(rule) = table.get(*name) {
            styles[i] = parse_style_value(rule);
        }
    }

    // Handle extra mappings
    for (helix_name, our_name) in EXTRA_MAPPINGS {
        if let Some(rule) = table.get(*helix_name) {
            if let Some(i) = HIGHLIGHT_NAMES.iter().position(|n| n == our_name) {
                if styles[i].is_empty() {
                    styles[i] = parse_style_value(rule);
                }
            }
        }
    }

    Ok(ParsedTheme {
        name,
        is_dark,
        source_url,
        background,
        foreground,
        styles,
    })
}

/// Parse a theme from TOML content and return the public Theme type.
pub fn parse_theme_toml(toml_str: &str) -> Result<Theme, String> {
    parse_theme(toml_str).map(|p| p.to_theme())
}

/// Generate Rust code for a color option.
fn gen_color_option(color: &Option<(u8, u8, u8)>) -> String {
    match color {
        Some((r, g, b)) => format!("Some(Color::new({r}, {g}, {b}))"),
        None => "None".to_string(),
    }
}

/// Generate Rust code for a style.
fn gen_style(style: &ParsedStyle) -> String {
    if style.is_empty() {
        return "Style::new()".to_string();
    }

    let mut parts = vec!["Style::new()".to_string()];

    if let Some((r, g, b)) = style.fg {
        parts.push(format!(".fg(Color::new({r}, {g}, {b}))"));
    }

    if style.bold {
        parts.push(".bold()".to_string());
    }
    if style.italic {
        parts.push(".italic()".to_string());
    }
    if style.underline {
        parts.push(".underline()".to_string());
    }
    if style.strikethrough {
        parts.push(".strikethrough()".to_string());
    }

    parts.join("")
}

/// Theme definition for code generation.
struct ThemeDef {
    fn_name: String,
    theme: ParsedTheme,
}

/// Generate builtin_generated.rs from all theme TOML files.
pub fn generate_theme_code(crates_dir: &Utf8Path) -> Result<(), String> {
    let themes_dir = crates_dir.join("arborium-theme/themes");
    let output_path = crates_dir.join("arborium-theme/src/builtin_generated.rs");

    println!(
        "{} Generating theme Rust code from {}",
        "●".cyan(),
        themes_dir.cyan()
    );

    // Collect and parse all theme files
    let mut themes: Vec<ThemeDef> = Vec::new();

    let entries =
        fs::read_dir(&themes_dir).map_err(|e| format!("Failed to read themes dir: {e}"))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read dir entry: {e}"))?;
        let path = entry.path();

        if path.extension().is_some_and(|e| e == "toml") {
            let file_stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .ok_or_else(|| format!("Invalid file name: {:?}", path))?;

            // Convert file name to function name (e.g., "catppuccin-mocha" -> "catppuccin_mocha")
            let fn_name = file_stem.replace('-', "_");

            let content =
                fs::read_to_string(&path).map_err(|e| format!("Failed to read {:?}: {e}", path))?;

            let theme =
                parse_theme(&content).map_err(|e| format!("Failed to parse {:?}: {e}", path))?;

            themes.push(ThemeDef { fn_name, theme });
        }
    }

    // Sort by function name for deterministic output
    themes.sort_by(|a, b| a.fn_name.cmp(&b.fn_name));

    // Generate the Rust code
    let mut code = String::new();

    writeln!(
        code,
        "// Generated theme definitions - DO NOT EDIT MANUALLY."
    )
    .unwrap();
    writeln!(
        code,
        "// This file is generated by xtask from TOML theme files."
    )
    .unwrap();
    writeln!(code).unwrap();
    writeln!(code, "use super::{{Color, Style, Theme}};").unwrap();
    writeln!(code).unwrap();

    // Generate each theme as a static function
    for def in &themes {
        let theme = &def.theme;

        writeln!(code, "/// {} theme.", theme.name).unwrap();
        if let Some(ref url) = theme.source_url {
            writeln!(code, "///").unwrap();
            writeln!(code, "/// Source: {url}").unwrap();
        }
        writeln!(code, "pub fn {}() -> Theme {{", def.fn_name).unwrap();
        writeln!(code, "    Theme {{").unwrap();
        writeln!(code, "        name: {:?}.to_string(),", theme.name).unwrap();
        writeln!(code, "        is_dark: {},", theme.is_dark).unwrap();

        match &theme.source_url {
            Some(url) => {
                writeln!(code, "        source_url: Some({:?}.to_string()),", url).unwrap()
            }
            None => writeln!(code, "        source_url: None,").unwrap(),
        }

        writeln!(
            code,
            "        background: {},",
            gen_color_option(&theme.background)
        )
        .unwrap();
        writeln!(
            code,
            "        foreground: {},",
            gen_color_option(&theme.foreground)
        )
        .unwrap();

        writeln!(code, "        styles: [").unwrap();
        for (i, style) in theme.styles.iter().enumerate() {
            let trailing = if i == theme.styles.len() - 1 { "" } else { "," };
            writeln!(code, "            {}{}", gen_style(style), trailing).unwrap();
        }
        writeln!(code, "        ],").unwrap();
        writeln!(code, "    }}").unwrap();
        writeln!(code, "}}").unwrap();
        writeln!(code).unwrap();
    }

    // Generate all() function
    writeln!(code, "/// Get all built-in themes.").unwrap();
    writeln!(code, "pub fn all() -> Vec<Theme> {{").unwrap();
    writeln!(code, "    vec![").unwrap();
    for def in &themes {
        writeln!(code, "        {}(),", def.fn_name).unwrap();
    }
    writeln!(code, "    ]").unwrap();
    writeln!(code, "}}").unwrap();

    // Write the file
    fs::write(&output_path, &code).map_err(|e| format!("Failed to write output: {e}"))?;

    println!(
        "  {} Generated {} themes to {}",
        "✓".green(),
        themes.len(),
        output_path.cyan()
    );

    Ok(())
}
