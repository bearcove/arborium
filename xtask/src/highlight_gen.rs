//! Highlight parsing - loads highlights.toml for theme generation.
//!
//! This module parses highlights.toml from crates/arborium-theme/
//! and provides types for CSS generation with fallback resolution.

use camino::Utf8Path;
use facet::Facet;
use fs_err as fs;
use indexmap::IndexMap;
use std::collections::{HashMap, HashSet};

/// A highlight definition parsed from TOML.
#[derive(Debug, Clone, Facet)]
pub struct HighlightDef {
    /// Short tag for HTML elements (e.g., "kf" -> `<a-kf>`)
    /// Empty string means no styling.
    pub tag: String,
    /// Parent name for style fallback (e.g., "keyword")
    pub parent: Option<String>,
    /// Alternative capture names that map to this highlight
    #[facet(default)]
    pub aliases: Vec<String>,
}

/// A highlight with its name (after parsing the TOML map).
#[derive(Debug, Clone)]
pub struct NamedHighlight {
    pub name: String,
    pub def: HighlightDef,
}

/// All parsed highlight definitions.
#[derive(Debug)]
pub struct Highlights {
    /// Definitions in order of appearance in TOML
    pub defs: Vec<NamedHighlight>,
    /// Map from name to index for quick lookup
    name_to_index: HashMap<String, usize>,
}

impl Highlights {
    /// Get a highlight definition by name.
    pub fn get(&self, name: &str) -> Option<&NamedHighlight> {
        self.name_to_index.get(name).map(|&i| &self.defs[i])
    }

    /// Get unique tags with their representative definition.
    /// Multiple highlights can share the same tag (e.g., number and float both use "n").
    /// This returns the first definition for each unique non-empty tag.
    pub fn unique_tags(&self) -> Vec<&NamedHighlight> {
        let mut seen = HashSet::new();
        let mut result = Vec::new();
        for def in &self.defs {
            if !def.def.tag.is_empty() && seen.insert(&def.def.tag) {
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

    let map: IndexMap<String, HighlightDef> = facet_toml::from_str(&content)
        .map_err(|e| format!("Failed to parse {}: {}", toml_path, e))?;

    let mut defs = Vec::new();
    let mut name_to_index = HashMap::new();

    for (name, def) in map {
        let index = defs.len();
        name_to_index.insert(name.clone(), index);

        // Also map aliases to this index
        for alias in &def.aliases {
            name_to_index.insert(alias.clone(), index);
        }

        defs.push(NamedHighlight { name, def });
    }

    Ok(Highlights { defs, name_to_index })
}
