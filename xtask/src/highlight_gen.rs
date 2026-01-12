//! Highlight parsing - loads highlights.toml for theme generation.
//!
//! This module parses highlights.toml from crates/arborium-theme/
//! and provides types for CSS generation with fallback resolution.

use camino::Utf8Path;
use fs_err as fs;
use std::collections::HashMap;

/// A highlight definition parsed from TOML.
#[derive(Debug, Clone)]
pub struct HighlightDef {
    /// The canonical name (e.g., "keyword.function")
    pub name: String,
    /// Short tag for HTML elements (e.g., "kf" -> `<a-kf>`)
    /// Empty string means no styling.
    pub tag: String,
    /// Parent name for style fallback (e.g., "keyword")
    pub parent: Option<String>,
    /// Alternative capture names that map to this highlight
    pub aliases: Vec<String>,
}

/// All parsed highlight definitions.
#[derive(Debug)]
pub struct Highlights {
    /// Definitions in order of appearance in TOML
    pub defs: Vec<HighlightDef>,
    /// Map from name to index for quick lookup
    name_to_index: HashMap<String, usize>,
}

impl Highlights {
    /// Get a highlight definition by name.
    pub fn get(&self, name: &str) -> Option<&HighlightDef> {
        self.name_to_index.get(name).map(|&i| &self.defs[i])
    }

    /// Resolve the style for a highlight name, following parent chain if needed.
    /// Returns the name of the highlight that should provide the style.
    pub fn resolve_style_source(&self, name: &str, has_style: impl Fn(&str) -> bool) -> Option<String> {
        let mut current = name.to_string();
        loop {
            if has_style(&current) {
                return Some(current);
            }
            let def = self.get(&current)?;
            current = def.parent.as_ref()?.clone();
        }
    }

    /// Get all capture names (including aliases) for tree-sitter configuration.
    pub fn all_capture_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = Vec::new();
        for def in &self.defs {
            names.push(&def.name);
            for alias in &def.aliases {
                names.push(alias);
            }
        }
        names
    }

    /// Get unique tags with their representative definition.
    /// Multiple highlights can share the same tag (e.g., number and float both use "n").
    /// This returns the first definition for each unique non-empty tag.
    pub fn unique_tags(&self) -> Vec<&HighlightDef> {
        let mut seen = std::collections::HashSet::new();
        let mut result = Vec::new();
        for def in &self.defs {
            if !def.tag.is_empty() && seen.insert(&def.tag) {
                result.push(def);
            }
        }
        result
    }
}

/// Parse highlights.toml and return all definitions.
pub fn parse_highlights(crates_dir: &Utf8Path) -> Result<Highlights, String> {
    let toml_path = crates_dir.join("arborium-theme/highlights.toml");
    let content = fs::read_to_string(&toml_path)
        .map_err(|e| format!("Failed to read {}: {}", toml_path, e))?;

    let value: toml::Value = content
        .parse()
        .map_err(|e| format!("Failed to parse {}: {}", toml_path, e))?;

    let table = value
        .as_table()
        .ok_or_else(|| "Expected table at root".to_string())?;

    let mut defs = Vec::new();
    let mut name_to_index = HashMap::new();

    for (name, value) in table {
        let def_table = value
            .as_table()
            .ok_or_else(|| format!("Expected table for '{}'", name))?;

        let tag = def_table
            .get("tag")
            .and_then(|v| v.as_str())
            .ok_or_else(|| format!("Missing 'tag' for '{}'", name))?
            .to_string();

        let parent = def_table
            .get("parent")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let aliases: Vec<String> = def_table
            .get("aliases")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let index = defs.len();
        name_to_index.insert(name.clone(), index);

        // Also map aliases to this index
        for alias in &aliases {
            name_to_index.insert(alias.clone(), index);
        }

        defs.push(HighlightDef {
            name: name.clone(),
            tag,
            parent,
            aliases,
        });
    }

    Ok(Highlights { defs, name_to_index })
}
