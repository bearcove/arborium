//! Theme code generation - converts TOML themes to Rust code at build time.
//!
//! This module:
//! 1. Parses theme TOML files from crates/arborium-theme/themes/
//! 2. Generates builtin_generated.rs for the arborium-theme crate
//! 3. Exports parsed themes for use by serve.rs (CSS generation, etc.)
//!
//! This eliminates the runtime TOML dependency from arborium-theme and
//! avoids xtask depending on arborium-theme (which would be circular).

use crate::highlight_gen::{self, Highlights, NamedHighlight};
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
    /// Styles keyed by highlight name (e.g., "keyword", "punctuation.special")
    pub styles: HashMap<String, Style>,
}

impl Theme {
    /// Get style for a highlight name.
    pub fn style(&self, name: &str) -> Option<&Style> {
        self.styles.get(name)
    }

    /// Resolve style for a highlight, following parent chain if needed.
    pub fn resolve_style(&self, hl: &NamedHighlight, highlights: &Highlights) -> Option<Style> {
        // First try the exact name
        if let Some(style) = self.styles.get(&hl.name) {
            if !style.is_empty() {
                return Some(style.clone());
            }
        }

        // Follow parent chain
        let mut current_parent = hl.def.parent.as_deref();
        while let Some(parent_name) = current_parent {
            if let Some(style) = self.styles.get(parent_name) {
                if !style.is_empty() {
                    return Some(style.clone());
                }
            }
            // Get parent's parent
            current_parent = highlights
                .get(parent_name)
                .and_then(|p| p.def.parent.as_deref());
        }

        None
    }

    /// Generate CSS for this theme with the given selector prefix.
    pub fn to_css(&self, selector_prefix: &str, highlights: &Highlights) -> String {
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

        // --accent: use function color, fallback to keyword, fallback to foreground
        let accent = self
            .styles
            .get("function")
            .and_then(|s| s.fg)
            .or_else(|| self.styles.get("keyword").and_then(|s| s.fg))
            .or(self.foreground);
        if let Some(c) = accent {
            writeln!(css, "  --accent: {};", c.to_hex()).unwrap();
        }

        // --muted: use comment color
        if let Some(c) = self.styles.get("comment").and_then(|s| s.fg) {
            writeln!(css, "  --muted: {};", c.to_hex()).unwrap();
        }

        writeln!(css, "}}").unwrap();

        // Generate styles for each highlight tag with fallback resolution
        // Track emitted tags to avoid duplicates (some highlights share tags)
        let mut emitted_tags: std::collections::HashSet<&str> = std::collections::HashSet::new();

        for def in &highlights.defs {
            if def.def.tag.is_empty() || emitted_tags.contains(def.def.tag.as_str()) {
                continue;
            }

            // Resolve style with fallback
            if let Some(style) = self.resolve_style(def, highlights) {
                if style.is_empty() {
                    continue;
                }

                emitted_tags.insert(&def.def.tag);
                write!(css, "{selector_prefix} a-{}", def.def.tag).unwrap();
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
// Internal parsing functions
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

/// Parse a theme from TOML content.
pub fn parse_theme_toml(toml_str: &str) -> Result<Theme, String> {
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

    // Parse styles into HashMap
    let parse_style_value = |value: &toml::Value| -> Style {
        let mut style = Style::default();
        match value {
            toml::Value::String(s) => {
                if let Some((r, g, b)) = resolve_color(s) {
                    style.fg = Some(Color(r, g, b));
                }
            }
            toml::Value::Table(t) => {
                if let Some(fg) = t.get("fg").and_then(|v| v.as_str()) {
                    if let Some((r, g, b)) = resolve_color(fg) {
                        style.fg = Some(Color(r, g, b));
                    }
                }
                if let Some(bg) = t.get("bg").and_then(|v| v.as_str()) {
                    if let Some((r, g, b)) = resolve_color(bg) {
                        style.bg = Some(Color(r, g, b));
                    }
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

    // Collect all styles
    let mut styles = HashMap::new();

    // Known style keys - any key that looks like a highlight name
    for (key, value) in table {
        // Skip metadata keys
        if matches!(
            key.as_str(),
            "name" | "variant" | "source" | "background" | "foreground" | "palette"
        ) {
            continue;
        }

        // Skip ui.* keys (not highlights)
        if key.starts_with("ui.") {
            continue;
        }

        let style = parse_style_value(value);
        if !style.is_empty() {
            styles.insert(key.clone(), style);
        }
    }

    Ok(Theme {
        name,
        is_dark,
        source_url,
        background: background.map(|(r, g, b)| Color(r, g, b)),
        foreground: foreground.map(|(r, g, b)| Color(r, g, b)),
        styles,
    })
}

// ============================================================================
// Code generation
// ============================================================================

/// Generate Rust code for a color option.
fn gen_color_option(color: &Option<Color>) -> String {
    match color {
        Some(Color(r, g, b)) => format!("Some(Color::new({r}, {g}, {b}))"),
        None => "None".to_string(),
    }
}

/// Generate Rust code for a style.
fn gen_style(style: &Style) -> String {
    if style.is_empty() {
        return "Style::new()".to_string();
    }

    let mut parts = vec!["Style::new()".to_string()];

    if let Some(Color(r, g, b)) = style.fg {
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

// STYLE_SLOT_NAMES is no longer hardcoded - we use the highlight definitions directly.

/// Theme definition for code generation.
struct ThemeDef {
    fn_name: String,
    name: String,
    is_dark: bool,
    source_url: Option<String>,
    background: Option<Color>,
    foreground: Option<Color>,
    styles: HashMap<String, Style>,
}

/// Parse highlights and get the parent chain for fallback.
fn get_parent_chain(highlights: &Highlights, name: &str) -> Vec<String> {
    let mut chain = Vec::new();
    let mut current = highlights.get(name).and_then(|d| d.def.parent.as_deref());
    while let Some(parent) = current {
        chain.push(parent.to_string());
        current = highlights.get(parent).and_then(|d| d.def.parent.as_deref());
    }
    chain
}

/// Resolve style for a name using fallback chain.
fn resolve_style_for_slot(
    styles: &HashMap<String, Style>,
    name: &str,
    highlights: &Highlights,
) -> Style {
    // Try exact match first
    if let Some(style) = styles.get(name) {
        if !style.is_empty() {
            return style.clone();
        }
    }

    // Try parent chain
    for parent in get_parent_chain(highlights, name) {
        if let Some(style) = styles.get(&parent) {
            if !style.is_empty() {
                return style.clone();
            }
        }
    }

    Style::default()
}

/// Generate builtin_generated.rs from all theme TOML files.
pub fn generate_theme_code(crates_dir: &Utf8Path) -> Result<(), String> {
    let themes_dir = crates_dir.join("arborium-theme/themes");
    let output_path = crates_dir.join("arborium-theme/src/builtin_generated.rs");

    // Parse highlights for fallback resolution
    let highlights = highlight_gen::parse_highlights(crates_dir)?;

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

            let theme = parse_theme_toml(&content)
                .map_err(|e| format!("Failed to parse {:?}: {e}", path))?;

            themes.push(ThemeDef {
                fn_name,
                name: theme.name,
                is_dark: theme.is_dark,
                source_url: theme.source_url,
                background: theme.background,
                foreground: theme.foreground,
                styles: theme.styles,
            });
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
        writeln!(code, "/// {} theme.", def.name).unwrap();
        if let Some(ref url) = def.source_url {
            writeln!(code, "///").unwrap();
            writeln!(code, "/// Source: {url}").unwrap();
        }
        writeln!(code, "pub fn {}() -> Theme {{", def.fn_name).unwrap();
        writeln!(code, "    Theme {{").unwrap();
        writeln!(code, "        name: {:?}.to_string(),", def.name).unwrap();
        writeln!(code, "        is_dark: {},", def.is_dark).unwrap();

        match &def.source_url {
            Some(url) => {
                writeln!(code, "        source_url: Some({:?}.to_string()),", url).unwrap()
            }
            None => writeln!(code, "        source_url: None,").unwrap(),
        }

        writeln!(
            code,
            "        background: {},",
            gen_color_option(&def.background)
        )
        .unwrap();
        writeln!(
            code,
            "        foreground: {},",
            gen_color_option(&def.foreground)
        )
        .unwrap();

        writeln!(code, "        styles: [").unwrap();
        for (i, highlight_def) in highlights.defs.iter().enumerate() {
            let style = resolve_style_for_slot(&def.styles, &highlight_def.name, &highlights);
            let trailing = if i == highlights.defs.len() - 1 {
                ""
            } else {
                ","
            };
            writeln!(code, "            {}{}", gen_style(&style), trailing).unwrap();
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
