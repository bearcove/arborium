//! Theme support for syntax highlighting.
//!
//! This module provides a unified theme system that can generate both CSS and ANSI output.
//! Themes use the Helix editor format for compatibility and ease of use.
//!
//! # Theme Format
//!
//! Themes are TOML files with highlight rules and an optional color palette:
//!
//! ```toml
//! # Simple foreground color
//! "keyword" = "purple"
//!
//! # With modifiers
//! "comment" = { fg = "gray", modifiers = ["italic"] }
//!
//! # Using palette reference
//! "function" = { fg = "blue1" }
//!
//! [palette]
//! purple = "#c678dd"
//! gray = "#5c6370"
//! blue1 = "#61afef"
//! ```

use std::collections::HashMap;
use std::fmt::Write as FmtWrite;

/// RGB color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Parse a hex color string like "#ff0000" or "ff0000".
    pub fn from_hex(s: &str) -> Option<Self> {
        let s = s.strip_prefix('#').unwrap_or(s);
        if s.len() != 6 {
            return None;
        }
        let r = u8::from_str_radix(&s[0..2], 16).ok()?;
        let g = u8::from_str_radix(&s[2..4], 16).ok()?;
        let b = u8::from_str_radix(&s[4..6], 16).ok()?;
        Some(Self { r, g, b })
    }

    /// Convert to hex string with # prefix.
    pub fn to_hex(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    /// Lighten the color by a factor (0.0 to 1.0).
    pub fn lighten(&self, factor: f32) -> Self {
        let factor = factor.clamp(0.0, 1.0);
        Self {
            r: (self.r as f32 + (255.0 - self.r as f32) * factor).round() as u8,
            g: (self.g as f32 + (255.0 - self.g as f32) * factor).round() as u8,
            b: (self.b as f32 + (255.0 - self.b as f32) * factor).round() as u8,
        }
    }

    /// Darken the color by a factor (0.0 to 1.0).
    pub fn darken(&self, factor: f32) -> Self {
        let factor = factor.clamp(0.0, 1.0);
        Self {
            r: (self.r as f32 * (1.0 - factor)).round() as u8,
            g: (self.g as f32 * (1.0 - factor)).round() as u8,
            b: (self.b as f32 * (1.0 - factor)).round() as u8,
        }
    }
}

/// Text style modifiers.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Modifiers {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
}

/// A complete style for a highlight category.
#[derive(Debug, Clone, Default)]
pub struct Style {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub modifiers: Modifiers,
}

impl Style {
    pub const fn new() -> Self {
        Self {
            fg: None,
            bg: None,
            modifiers: Modifiers {
                bold: false,
                italic: false,
                underline: false,
                strikethrough: false,
            },
        }
    }

    pub const fn fg(mut self, color: Color) -> Self {
        self.fg = Some(color);
        self
    }

    pub const fn bold(mut self) -> Self {
        self.modifiers.bold = true;
        self
    }

    pub const fn italic(mut self) -> Self {
        self.modifiers.italic = true;
        self
    }

    pub const fn underline(mut self) -> Self {
        self.modifiers.underline = true;
        self
    }

    pub const fn strikethrough(mut self) -> Self {
        self.modifiers.strikethrough = true;
        self
    }

    /// Check if this style has any effect.
    pub fn is_empty(&self) -> bool {
        self.fg.is_none()
            && self.bg.is_none()
            && !self.modifiers.bold
            && !self.modifiers.italic
            && !self.modifiers.underline
            && !self.modifiers.strikethrough
    }
}

/// A complete syntax highlighting theme.
#[derive(Debug, Clone)]
pub struct Theme {
    /// Theme name for display.
    pub name: String,
    /// Whether this is a dark or light theme.
    pub is_dark: bool,
    /// URL to the original theme source (for attribution).
    pub source_url: Option<String>,
    /// Background color for the code block.
    pub background: Option<Color>,
    /// Foreground (default text) color.
    pub foreground: Option<Color>,
    /// Styles for each highlight category, indexed by HIGHLIGHT_NAMES.
    styles: [Style; crate::highlights::COUNT],
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            name: String::new(),
            is_dark: true,
            source_url: None,
            background: None,
            foreground: None,
            styles: std::array::from_fn(|_| Style::new()),
        }
    }
}

impl Theme {
    /// Create an empty theme.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Get the style for a highlight index.
    pub fn style(&self, index: usize) -> Option<&Style> {
        self.styles.get(index)
    }

    /// Set the style for a highlight index.
    pub fn set_style(&mut self, index: usize, style: Style) {
        if index < self.styles.len() {
            self.styles[index] = style;
        }
    }

    /// Parse a theme from Helix-style TOML.
    pub fn from_toml(toml_str: &str) -> Result<Self, ThemeError> {
        let value: toml::Value = toml_str
            .parse()
            .map_err(|e| ThemeError::Parse(format!("{e}")))?;
        let table = value
            .as_table()
            .ok_or(ThemeError::Parse("Expected table".into()))?;

        let mut theme = Theme::default();

        // Extract metadata
        if let Some(name) = table.get("name").and_then(|v| v.as_str()) {
            theme.name = name.to_string();
        }
        if let Some(variant) = table.get("variant").and_then(|v| v.as_str()) {
            theme.is_dark = variant != "light";
        }
        if let Some(source) = table.get("source").and_then(|v| v.as_str()) {
            theme.source_url = Some(source.to_string());
        }

        // Extract palette for color lookups
        let palette: HashMap<&str, Color> = table
            .get("palette")
            .and_then(|v| v.as_table())
            .map(|t| {
                t.iter()
                    .filter_map(|(k, v)| {
                        v.as_str()
                            .and_then(Color::from_hex)
                            .map(|c| (k.as_str(), c))
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Helper to resolve a color (either hex or palette reference)
        let resolve_color =
            |s: &str| -> Option<Color> { Color::from_hex(s).or_else(|| palette.get(s).copied()) };

        // Extract ui.background and ui.foreground
        if let Some(bg) = table.get("ui.background")
            && let Some(bg_table) = bg.as_table()
            && let Some(bg_str) = bg_table.get("bg").and_then(|v| v.as_str())
        {
            theme.background = resolve_color(bg_str);
        }
        // Also check for simple "background" key
        if let Some(bg_str) = table.get("background").and_then(|v| v.as_str()) {
            theme.background = resolve_color(bg_str);
        }

        if let Some(fg) = table.get("ui.foreground") {
            if let Some(fg_str) = fg.as_str() {
                theme.foreground = resolve_color(fg_str);
            } else if let Some(fg_table) = fg.as_table()
                && let Some(fg_str) = fg_table.get("fg").and_then(|v| v.as_str())
            {
                theme.foreground = resolve_color(fg_str);
            }
        }
        // Also check for simple "foreground" key
        if let Some(fg_str) = table.get("foreground").and_then(|v| v.as_str()) {
            theme.foreground = resolve_color(fg_str);
        }

        // Build mapping from Helix names to our indices using highlights module
        use crate::highlights::HIGHLIGHTS;

        // Parse each highlight rule - try main name and aliases
        for (i, def) in HIGHLIGHTS.iter().enumerate() {
            // Try main name
            if let Some(rule) = table.get(def.name) {
                let style = parse_style_value(rule, &resolve_color)?;
                theme.styles[i] = style;
                continue;
            }

            // Try aliases
            for alias in def.aliases {
                if let Some(rule) = table.get(*alias) {
                    let style = parse_style_value(rule, &resolve_color)?;
                    theme.styles[i] = style;
                    break;
                }
            }
        }

        // Also handle some common Helix-specific mappings that aren't direct matches
        let extra_mappings: &[(&str, &str)] = &[
            ("keyword.control", "keyword"),
            ("keyword.storage", "keyword"),
            ("comment.line", "comment"),
            ("comment.block", "comment"),
            ("function.macro", "macro"),
        ];

        for (helix_name, our_name) in extra_mappings {
            if let Some(rule) = table.get(*helix_name) {
                // Find our index
                if let Some(i) = HIGHLIGHTS.iter().position(|h| h.name == *our_name) {
                    // Only apply if we don't already have a style
                    if theme.styles[i].is_empty() {
                        let style = parse_style_value(rule, &resolve_color)?;
                        theme.styles[i] = style;
                    }
                }
            }
        }

        Ok(theme)
    }

    /// Generate CSS for this theme.
    ///
    /// Uses CSS nesting for compact output. The selector_prefix is prepended
    /// to scope the rules (e.g., `[data-theme="mocha"]`).
    pub fn to_css(&self, selector_prefix: &str) -> String {
        use crate::highlights::HIGHLIGHTS;
        use std::collections::HashMap;

        let mut css = String::new();

        writeln!(css, "{selector_prefix} {{").unwrap();

        // Background and foreground
        if let Some(bg) = &self.background {
            writeln!(css, "  background: {};", bg.to_hex()).unwrap();
            writeln!(css, "  --bg: {};", bg.to_hex()).unwrap();
            // Surface is background adjusted toward opposite (lighter for dark, darker for light)
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

        // Find indices for accent and muted colors
        let function_idx = HIGHLIGHTS.iter().position(|h| h.name == "function");
        let keyword_idx = HIGHLIGHTS.iter().position(|h| h.name == "keyword");
        let comment_idx = HIGHLIGHTS.iter().position(|h| h.name == "comment");

        // --accent: use function color, fallback to keyword, fallback to foreground
        let accent_color = function_idx
            .and_then(|i| self.styles[i].fg.as_ref())
            .or_else(|| keyword_idx.and_then(|i| self.styles[i].fg.as_ref()))
            .or(self.foreground.as_ref());
        if let Some(accent) = accent_color {
            writeln!(css, "  --accent: {};", accent.to_hex()).unwrap();
        }

        // --muted: use comment color, fallback to faded foreground
        let muted_color = comment_idx.and_then(|i| self.styles[i].fg.as_ref());
        if let Some(muted) = muted_color {
            writeln!(css, "  --muted: {};", muted.to_hex()).unwrap();
        } else if let Some(fg) = &self.foreground {
            let muted = if self.is_dark {
                fg.darken(0.3)
            } else {
                fg.lighten(0.3)
            };
            writeln!(css, "  --muted: {};", muted.to_hex()).unwrap();
        }

        // Build a map from tag -> style for parent lookups
        let mut tag_to_style: HashMap<&str, &Style> = HashMap::new();
        for (i, def) in HIGHLIGHTS.iter().enumerate() {
            if !def.tag.is_empty() && !self.styles[i].is_empty() {
                tag_to_style.insert(def.tag, &self.styles[i]);
            }
        }

        // Generate rules for each highlight category
        for (i, def) in HIGHLIGHTS.iter().enumerate() {
            if def.tag.is_empty() {
                continue; // Skip categories like "none" that have no tag
            }

            // Use own style, or fall back to parent style
            let style = if !self.styles[i].is_empty() {
                &self.styles[i]
            } else if !def.parent_tag.is_empty() {
                // Look up parent style
                tag_to_style
                    .get(def.parent_tag)
                    .copied()
                    .unwrap_or(&self.styles[i])
            } else {
                continue; // No style and no parent
            };

            if style.is_empty() {
                continue;
            }

            write!(css, "  a-{} {{", def.tag).unwrap();

            if let Some(fg) = &style.fg {
                write!(css, " color: {};", fg.to_hex()).unwrap();
            }
            if let Some(bg) = &style.bg {
                write!(css, " background: {};", bg.to_hex()).unwrap();
            }

            let mut decorations = Vec::new();
            if style.modifiers.underline {
                decorations.push("underline");
            }
            if style.modifiers.strikethrough {
                decorations.push("line-through");
            }
            if !decorations.is_empty() {
                write!(css, " text-decoration: {};", decorations.join(" ")).unwrap();
            }

            if style.modifiers.bold {
                write!(css, " font-weight: bold;").unwrap();
            }
            if style.modifiers.italic {
                write!(css, " font-style: italic;").unwrap();
            }

            writeln!(css, " }}").unwrap();
        }

        writeln!(css, "}}").unwrap();

        css
    }

    /// Generate ANSI escape sequence for a style.
    pub fn ansi_style(&self, index: usize) -> String {
        let Some(style) = self.styles.get(index) else {
            return String::new();
        };

        if style.is_empty() {
            return String::new();
        }

        let mut codes = Vec::new();

        if style.modifiers.bold {
            codes.push("1".to_string());
        }
        if style.modifiers.italic {
            codes.push("3".to_string());
        }
        if style.modifiers.underline {
            codes.push("4".to_string());
        }
        if style.modifiers.strikethrough {
            codes.push("9".to_string());
        }

        if let Some(fg) = &style.fg {
            codes.push(format!("38;2;{};{};{}", fg.r, fg.g, fg.b));
        }
        if let Some(bg) = &style.bg {
            codes.push(format!("48;2;{};{};{}", bg.r, bg.g, bg.b));
        }

        if codes.is_empty() {
            String::new()
        } else {
            format!("\x1b[{}m", codes.join(";"))
        }
    }

    /// Generate ANSI escape sequence for a style, inheriting base foreground/background if not set.
    ///
    /// When rendering with a base background color, we want individual styles to
    /// inherit that background unless they explicitly override it. Similarly, if a style
    /// doesn't define a foreground, use the base foreground. This avoids the background
    /// disappearing when switching between styled and unstyled text, and ensures colors
    /// are complete.
    pub fn ansi_style_with_base_bg(&self, index: usize) -> String {
        let Some(style) = self.styles.get(index) else {
            return String::new();
        };

        if style.is_empty() {
            return String::new();
        }

        let mut codes = Vec::new();

        if style.modifiers.bold {
            codes.push("1".to_string());
        }
        if style.modifiers.italic {
            codes.push("3".to_string());
        }
        if style.modifiers.underline {
            codes.push("4".to_string());
        }
        if style.modifiers.strikethrough {
            codes.push("9".to_string());
        }

        // Use style's foreground if defined, otherwise fall back to theme foreground
        if let Some(fg) = &style.fg {
            codes.push(format!("38;2;{};{};{}", fg.r, fg.g, fg.b));
        } else if let Some(fg) = &self.foreground {
            codes.push(format!("38;2;{};{};{}", fg.r, fg.g, fg.b));
        }

        // Use style's background if defined, otherwise fall back to theme background
        if let Some(bg) = &style.bg {
            codes.push(format!("48;2;{};{};{}", bg.r, bg.g, bg.b));
        } else if let Some(bg) = &self.background {
            codes.push(format!("48;2;{};{};{}", bg.r, bg.g, bg.b));
        }

        if codes.is_empty() {
            String::new()
        } else {
            format!("\x1b[{}m", codes.join(";"))
        }
    }

    /// Generate ANSI escape sequence for the theme's base foreground/background.
    ///
    /// This uses `background` and `foreground` and does not include any
    /// per-highlight styling or text modifiers.
    pub fn ansi_base_style(&self) -> String {
        let mut codes = Vec::new();

        if let Some(fg) = &self.foreground {
            codes.push(format!("38;2;{};{};{}", fg.r, fg.g, fg.b));
        }
        if let Some(bg) = &self.background {
            codes.push(format!("48;2;{};{};{}", bg.r, bg.g, bg.b));
        }

        if codes.is_empty() {
            String::new()
        } else {
            format!("\x1b[{}m", codes.join(";"))
        }
    }

    /// Generate ANSI escape sequence for border characters (half-blocks).
    ///
    /// Returns fg color only (no bg), slightly darker/lighter than theme background.
    /// The transparent half of the half-block char shows the terminal background.
    pub fn ansi_border_style(&self) -> String {
        let Some(bg) = &self.background else {
            return String::new();
        };

        // Border color: darker for dark themes, lighter for light themes
        let border = if self.is_dark {
            Color::new(
                bg.r.saturating_add(30),
                bg.g.saturating_add(30),
                bg.b.saturating_add(30),
            )
        } else {
            Color::new(
                bg.r.saturating_sub(30),
                bg.g.saturating_sub(30),
                bg.b.saturating_sub(30),
            )
        };

        format!("\x1b[38;2;{};{};{}m", border.r, border.g, border.b)
    }

    /// ANSI reset sequence.
    pub const ANSI_RESET: &'static str = "\x1b[0m";
}

/// Parse a style value from TOML (either string or table).
fn parse_style_value(
    value: &toml::Value,
    resolve_color: &impl Fn(&str) -> Option<Color>,
) -> Result<Style, ThemeError> {
    let mut style = Style::new();

    match value {
        // Simple string: just foreground color
        toml::Value::String(s) => {
            style.fg = resolve_color(s);
        }
        // Table with fg, bg, modifiers
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
                            "bold" => style.modifiers.bold = true,
                            "italic" => style.modifiers.italic = true,
                            "underlined" | "underline" => style.modifiers.underline = true,
                            "crossed_out" | "strikethrough" => style.modifiers.strikethrough = true,
                            _ => {}
                        }
                    }
                }
            }
        }
        _ => {}
    }

    Ok(style)
}

/// Error type for theme parsing.
#[derive(Debug)]
pub enum ThemeError {
    Parse(String),
}

impl std::fmt::Display for ThemeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThemeError::Parse(msg) => write!(f, "Theme parse error: {msg}"),
        }
    }
}

impl std::error::Error for ThemeError {}

// ============================================================================
// Built-in themes (include_str!'d from TOML files)
// ============================================================================

macro_rules! builtin_theme {
    ($name:ident, $file:literal) => {
        pub fn $name() -> &'static Theme {
            use std::sync::OnceLock;
            static THEME: OnceLock<Theme> = OnceLock::new();
            THEME.get_or_init(|| {
                Theme::from_toml(include_str!(concat!("../themes/", $file)))
                    .expect(concat!("Failed to parse built-in theme: ", $file))
            })
        }
    };
}

/// Built-in themes module.
pub mod builtin {
    use super::Theme;

    builtin_theme!(catppuccin_mocha, "catppuccin-mocha.toml");
    builtin_theme!(catppuccin_latte, "catppuccin-latte.toml");
    builtin_theme!(catppuccin_frappe, "catppuccin-frappe.toml");
    builtin_theme!(catppuccin_macchiato, "catppuccin-macchiato.toml");
    builtin_theme!(dracula, "dracula.toml");
    builtin_theme!(tokyo_night, "tokyo-night.toml");
    builtin_theme!(nord, "nord.toml");
    builtin_theme!(one_dark, "one-dark.toml");
    builtin_theme!(github_dark, "github-dark.toml");
    builtin_theme!(github_light, "github-light.toml");
    builtin_theme!(gruvbox_dark, "gruvbox-dark.toml");
    builtin_theme!(gruvbox_light, "gruvbox-light.toml");
    builtin_theme!(monokai, "monokai.toml");
    builtin_theme!(kanagawa_dragon, "kanagawa-dragon.toml");
    builtin_theme!(rose_pine_moon, "rose-pine-moon.toml");
    builtin_theme!(ayu_dark, "ayu-dark.toml");
    builtin_theme!(ayu_light, "ayu-light.toml");
    builtin_theme!(solarized_dark, "solarized-dark.toml");
    builtin_theme!(solarized_light, "solarized-light.toml");
    builtin_theme!(ef_melissa_dark, "ef-melissa-dark.toml");
    builtin_theme!(melange_dark, "melange-dark.toml");
    builtin_theme!(melange_light, "melange-light.toml");
    builtin_theme!(light_owl, "light-owl.toml");
    builtin_theme!(lucius_light, "lucius-light.toml");
    builtin_theme!(rustdoc_light, "rustdoc-light.toml");
    builtin_theme!(rustdoc_dark, "rustdoc-dark.toml");
    builtin_theme!(rustdoc_ayu, "rustdoc-ayu.toml");
    builtin_theme!(dayfox, "dayfox.toml");
    builtin_theme!(alabaster, "alabaster.toml");
    builtin_theme!(cobalt2, "cobalt2.toml");
    builtin_theme!(zenburn, "zenburn.toml");
    builtin_theme!(desert256, "desert256.toml");

    /// Get all built-in themes.
    pub fn all() -> Vec<&'static Theme> {
        vec![
            catppuccin_mocha(),
            catppuccin_latte(),
            catppuccin_frappe(),
            catppuccin_macchiato(),
            dracula(),
            tokyo_night(),
            nord(),
            one_dark(),
            github_dark(),
            github_light(),
            gruvbox_dark(),
            gruvbox_light(),
            monokai(),
            kanagawa_dragon(),
            rose_pine_moon(),
            ayu_dark(),
            ayu_light(),
            solarized_dark(),
            solarized_light(),
            ef_melissa_dark(),
            melange_dark(),
            melange_light(),
            light_owl(),
            lucius_light(),
            rustdoc_light(),
            rustdoc_dark(),
            rustdoc_ayu(),
            dayfox(),
            alabaster(),
            cobalt2(),
            zenburn(),
            desert256(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_from_hex() {
        assert_eq!(Color::from_hex("#ff0000"), Some(Color::new(255, 0, 0)));
        assert_eq!(Color::from_hex("00ff00"), Some(Color::new(0, 255, 0)));
        assert_eq!(Color::from_hex("#invalid"), None);
    }

    #[test]
    fn test_color_to_hex() {
        assert_eq!(Color::new(255, 0, 0).to_hex(), "#ff0000");
        assert_eq!(Color::new(0, 255, 0).to_hex(), "#00ff00");
    }
}
