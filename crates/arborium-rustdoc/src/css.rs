//! CSS theme generation for rustdoc integration.
//!
//! Generates CSS rules that integrate arborium's syntax highlighting with rustdoc's
//! theme system. The generated CSS uses `[data-theme="..."]` selectors to match
//! rustdoc's built-in themes.

use arborium_theme::builtin;
use std::fmt::Write;

/// Rustdoc's built-in themes and their corresponding arborium theme.
const RUSTDOC_THEMES: &[(&str, fn() -> &'static arborium_theme::Theme)] = &[
    ("light", builtin::rustdoc_light),
    ("dark", builtin::rustdoc_dark),
    ("ayu", builtin::rustdoc_ayu),
];

/// Generate CSS for all rustdoc themes.
///
/// Returns CSS that can be appended to rustdoc's main CSS file. The generated
/// rules are scoped to `[data-theme="..."]` selectors and target code blocks
/// with `language-*` classes.
pub fn generate_rustdoc_theme_css() -> String {
    let mut css = String::new();

    // Header comment
    writeln!(
        css,
        "\n/* arborium syntax highlighting for non-Rust code blocks */"
    )
    .unwrap();

    for (theme_name, theme_fn) in RUSTDOC_THEMES {
        let theme = theme_fn();

        // Generate CSS for this theme
        // We need to target: pre.language-* code a-*
        // The selector prefix scopes it to the specific rustdoc theme
        let selector = if *theme_name == "light" {
            // Light is the default, so we need both :root (no theme) and explicit light
            ":root:not([data-theme]), :root[data-theme=\"light\"]".to_string()
        } else {
            format!(":root[data-theme=\"{}\"]", theme_name)
        };

        // Use the theme's to_css method but we need to adjust the selector
        // to target our code blocks specifically
        let theme_css = generate_theme_css_for_rustdoc(theme, &selector);
        css.push_str(&theme_css);
    }

    css
}

/// Generate CSS rules for a single theme, targeting rustdoc's code block structure.
fn generate_theme_css_for_rustdoc(theme: &arborium_theme::Theme, selector_prefix: &str) -> String {
    use arborium_theme::HIGHLIGHTS;
    use std::collections::HashMap;

    let mut css = String::new();

    // Build a map from tag -> style for parent lookups
    let mut tag_to_style: HashMap<&str, &arborium_theme::Style> = HashMap::new();
    for (i, def) in HIGHLIGHTS.iter().enumerate() {
        if !def.tag.is_empty() {
            if let Some(style) = theme.style(i) {
                if !style.is_empty() {
                    tag_to_style.insert(def.tag, style);
                }
            }
        }
    }

    // Open the selector block
    // Target: pre elements with language-* class (but not .rust)
    writeln!(
        css,
        "{} pre[class^=\"language-\"] code, {} pre[class*=\" language-\"] code {{",
        selector_prefix, selector_prefix
    )
    .unwrap();

    // Generate rules for each highlight category
    for (i, def) in HIGHLIGHTS.iter().enumerate() {
        if def.tag.is_empty() {
            continue;
        }

        // Get style (own or parent)
        let style = theme.style(i).filter(|s| !s.is_empty()).or_else(|| {
            if !def.parent_tag.is_empty() {
                tag_to_style.get(def.parent_tag).copied()
            } else {
                None
            }
        });

        let Some(style) = style else {
            continue;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_theme_css() {
        let css = generate_rustdoc_theme_css();

        // Should contain all three theme selectors
        assert!(css.contains("data-theme=\"light\""));
        assert!(css.contains("data-theme=\"dark\""));
        assert!(css.contains("data-theme=\"ayu\""));

        // Should contain arborium custom element rules
        assert!(css.contains("a-k"));
        assert!(css.contains("a-s"));
        assert!(css.contains("a-c"));
    }
}
