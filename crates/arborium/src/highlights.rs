//! Highlight category definitions - single source of truth.
//!
//! This module defines all highlight categories used for syntax highlighting.
//! It generates both the canonical names (for tree-sitter) and the short HTML tags.

/// A highlight category definition.
pub struct HighlightDef {
    /// The canonical name (e.g., "keyword.function")
    pub name: &'static str,
    /// Short tag suffix for HTML elements (e.g., "kf" -> `<a-kf>`)
    /// Empty string means no styling should be applied.
    pub tag: &'static str,
    /// Parent category tag for CSS inheritance (e.g., "k" for keyword.*)
    /// Empty string means no parent.
    pub parent_tag: &'static str,
    /// Alternative names from Helix/other editors that map to this category
    pub aliases: &'static [&'static str],
}

/// All highlight categories, in order.
/// The index in this array is the highlight index used throughout the codebase.
pub const HIGHLIGHTS: &[HighlightDef] = &[
    // Core categories
    HighlightDef {
        name: "attribute",
        tag: "at",
        parent_tag: "",
        aliases: &[],
    },
    HighlightDef {
        name: "constant",
        tag: "co",
        parent_tag: "",
        aliases: &[],
    },
    HighlightDef {
        name: "constant.builtin",
        tag: "cb",
        parent_tag: "co",
        aliases: &["constant.builtin.boolean"],
    },
    HighlightDef {
        name: "constructor",
        tag: "cr",
        parent_tag: "",
        aliases: &[],
    },
    HighlightDef {
        name: "function.builtin",
        tag: "fb",
        parent_tag: "f",
        aliases: &[],
    },
    HighlightDef {
        name: "function",
        tag: "f",
        parent_tag: "",
        aliases: &[],
    },
    HighlightDef {
        name: "function.method",
        tag: "fm",
        parent_tag: "f",
        aliases: &[],
    },
    HighlightDef {
        name: "keyword",
        tag: "k",
        parent_tag: "",
        aliases: &[],
    },
    HighlightDef {
        name: "keyword.conditional",
        tag: "kc",
        parent_tag: "k",
        aliases: &[],
    },
    HighlightDef {
        name: "keyword.coroutine",
        tag: "ko",
        parent_tag: "k",
        aliases: &[],
    },
    HighlightDef {
        name: "keyword.debug",
        tag: "kd",
        parent_tag: "k",
        aliases: &[],
    },
    HighlightDef {
        name: "keyword.exception",
        tag: "ke",
        parent_tag: "k",
        aliases: &[],
    },
    HighlightDef {
        name: "keyword.function",
        tag: "kf",
        parent_tag: "k",
        aliases: &[],
    },
    HighlightDef {
        name: "keyword.import",
        tag: "ki",
        parent_tag: "k",
        aliases: &[],
    },
    HighlightDef {
        name: "keyword.operator",
        tag: "kp",
        parent_tag: "k",
        aliases: &[],
    },
    HighlightDef {
        name: "keyword.repeat",
        tag: "kr",
        parent_tag: "k",
        aliases: &[],
    },
    HighlightDef {
        name: "keyword.return",
        tag: "kt",
        parent_tag: "k",
        aliases: &[],
    },
    HighlightDef {
        name: "keyword.type",
        tag: "ky",
        parent_tag: "k",
        aliases: &[],
    },
    HighlightDef {
        name: "operator",
        tag: "o",
        parent_tag: "",
        aliases: &[],
    },
    HighlightDef {
        name: "property",
        tag: "pr",
        parent_tag: "",
        aliases: &[],
    },
    HighlightDef {
        name: "punctuation",
        tag: "p",
        parent_tag: "",
        aliases: &[],
    },
    HighlightDef {
        name: "punctuation.bracket",
        tag: "pb",
        parent_tag: "p",
        aliases: &[],
    },
    HighlightDef {
        name: "punctuation.delimiter",
        tag: "pd",
        parent_tag: "p",
        aliases: &[],
    },
    HighlightDef {
        name: "punctuation.special",
        tag: "ps",
        parent_tag: "p",
        aliases: &[],
    },
    HighlightDef {
        name: "string",
        tag: "s",
        parent_tag: "",
        aliases: &[],
    },
    HighlightDef {
        name: "string.special",
        tag: "ss",
        parent_tag: "s",
        aliases: &["string.special.symbol", "string.special.path"],
    },
    HighlightDef {
        name: "tag",
        tag: "tg",
        parent_tag: "",
        aliases: &[],
    },
    HighlightDef {
        name: "tag.delimiter",
        tag: "td",
        parent_tag: "tg",
        aliases: &[],
    },
    HighlightDef {
        name: "tag.error",
        tag: "te",
        parent_tag: "tg",
        aliases: &[],
    },
    HighlightDef {
        name: "type",
        tag: "t",
        parent_tag: "",
        aliases: &[],
    },
    HighlightDef {
        name: "type.builtin",
        tag: "tb",
        parent_tag: "t",
        aliases: &[],
    },
    HighlightDef {
        name: "type.qualifier",
        tag: "tq",
        parent_tag: "t",
        aliases: &[],
    },
    HighlightDef {
        name: "variable",
        tag: "v",
        parent_tag: "",
        aliases: &[],
    },
    HighlightDef {
        name: "variable.builtin",
        tag: "vb",
        parent_tag: "v",
        aliases: &[],
    },
    HighlightDef {
        name: "variable.parameter",
        tag: "vp",
        parent_tag: "v",
        aliases: &["parameter"],
    },
    HighlightDef {
        name: "comment",
        tag: "c",
        parent_tag: "",
        aliases: &[],
    },
    HighlightDef {
        name: "comment.documentation",
        tag: "cd",
        parent_tag: "c",
        aliases: &[],
    },
    HighlightDef {
        name: "macro",
        tag: "m",
        parent_tag: "",
        aliases: &[],
    },
    HighlightDef {
        name: "label",
        tag: "l",
        parent_tag: "",
        aliases: &[],
    },
    HighlightDef {
        name: "diff.addition",
        tag: "da",
        parent_tag: "",
        aliases: &["diff.plus", "diff.delta"],
    },
    HighlightDef {
        name: "diff.deletion",
        tag: "dd",
        parent_tag: "",
        aliases: &["diff.minus"],
    },
    HighlightDef {
        name: "number",
        tag: "n",
        parent_tag: "",
        aliases: &["constant.numeric"],
    },
    HighlightDef {
        name: "text.literal",
        tag: "tl",
        parent_tag: "",
        aliases: &["markup.raw"],
    },
    HighlightDef {
        name: "text.emphasis",
        tag: "em",
        parent_tag: "",
        aliases: &["markup.italic"],
    },
    HighlightDef {
        name: "text.strong",
        tag: "st",
        parent_tag: "",
        aliases: &["markup.bold"],
    },
    HighlightDef {
        name: "text.uri",
        tag: "tu",
        parent_tag: "",
        aliases: &["markup.link.url"],
    },
    HighlightDef {
        name: "text.reference",
        tag: "tr",
        parent_tag: "",
        aliases: &["markup.link.text"],
    },
    HighlightDef {
        name: "string.escape",
        tag: "se",
        parent_tag: "s",
        aliases: &["escape"],
    },
    HighlightDef {
        name: "text.title",
        tag: "tt",
        parent_tag: "",
        aliases: &["markup.heading"],
    },
    HighlightDef {
        name: "text.strikethrough",
        tag: "tx",
        parent_tag: "",
        aliases: &["markup.strikethrough"],
    },
    HighlightDef {
        name: "spell",
        tag: "sp",
        parent_tag: "",
        aliases: &[],
    },
    HighlightDef {
        name: "embedded",
        tag: "eb",
        parent_tag: "",
        aliases: &[],
    },
    HighlightDef {
        name: "error",
        tag: "er",
        parent_tag: "",
        aliases: &[],
    },
    HighlightDef {
        name: "namespace",
        tag: "ns",
        parent_tag: "",
        aliases: &["module"],
    },
    // Legacy/alternative names used by some grammars
    HighlightDef {
        name: "include",
        tag: "in",
        parent_tag: "k",
        aliases: &[],
    },
    HighlightDef {
        name: "storageclass",
        tag: "sc",
        parent_tag: "k",
        aliases: &[],
    },
    HighlightDef {
        name: "repeat",
        tag: "rp",
        parent_tag: "k",
        aliases: &[],
    },
    HighlightDef {
        name: "conditional",
        tag: "cn",
        parent_tag: "k",
        aliases: &[],
    },
    HighlightDef {
        name: "exception",
        tag: "ex",
        parent_tag: "k",
        aliases: &[],
    },
    HighlightDef {
        name: "preproc",
        tag: "pp",
        parent_tag: "k",
        aliases: &[],
    },
    HighlightDef {
        name: "none",
        tag: "",
        parent_tag: "",
        aliases: &[],
    }, // No styling
    HighlightDef {
        name: "character",
        tag: "ch",
        parent_tag: "s",
        aliases: &[],
    },
    HighlightDef {
        name: "character.special",
        tag: "cs",
        parent_tag: "s",
        aliases: &[],
    },
    HighlightDef {
        name: "variable.member",
        tag: "vm",
        parent_tag: "v",
        aliases: &[],
    },
    HighlightDef {
        name: "function.definition",
        tag: "fd",
        parent_tag: "f",
        aliases: &[],
    },
    HighlightDef {
        name: "type.definition",
        tag: "tf",
        parent_tag: "t",
        aliases: &[],
    },
    HighlightDef {
        name: "function.call",
        tag: "fc",
        parent_tag: "f",
        aliases: &[],
    },
    HighlightDef {
        name: "keyword.modifier",
        tag: "km",
        parent_tag: "k",
        aliases: &[],
    },
    HighlightDef {
        name: "keyword.directive",
        tag: "dr",
        parent_tag: "k",
        aliases: &[],
    },
    HighlightDef {
        name: "string.regexp",
        tag: "rx",
        parent_tag: "s",
        aliases: &["string.regex"],
    },
    HighlightDef {
        name: "nospell",
        tag: "",
        parent_tag: "",
        aliases: &[],
    }, // No styling
    HighlightDef {
        name: "float",
        tag: "n",
        parent_tag: "",
        aliases: &[],
    }, // Same as number
    HighlightDef {
        name: "boolean",
        tag: "cb",
        parent_tag: "",
        aliases: &[],
    }, // Same as constant.builtin
];

/// Get the highlight names array for tree-sitter configuration.
pub const fn names() -> [&'static str; HIGHLIGHTS.len()] {
    let mut names = [""; HIGHLIGHTS.len()];
    let mut i = 0;
    while i < HIGHLIGHTS.len() {
        names[i] = HIGHLIGHTS[i].name;
        i += 1;
    }
    names
}

/// Total number of highlight categories.
pub const COUNT: usize = HIGHLIGHTS.len();

/// Get the HTML tag suffix for a highlight index.
/// Returns None for indices that should produce no styling (like "none" or "nospell").
#[inline]
pub fn tag(index: usize) -> Option<&'static str> {
    HIGHLIGHTS
        .get(index)
        .map(|h| h.tag)
        .filter(|t| !t.is_empty())
}

/// Get the prefixed HTML tag (e.g., "a-kf") for a highlight index.
#[inline]
pub fn prefixed_tag(index: usize) -> Option<String> {
    tag(index).map(|t| format!("a-{t}"))
}

/// Get the parent tag for inheritance, if any.
#[inline]
pub fn parent_tag(index: usize) -> Option<&'static str> {
    HIGHLIGHTS
        .get(index)
        .map(|h| h.parent_tag)
        .filter(|t| !t.is_empty())
}

/// Generate CSS inheritance rules for sub-categories.
/// Returns rules like "a-kc, a-kf, a-ki { color: inherit; }" grouped by parent.
pub fn css_inheritance_rules() -> String {
    use std::collections::HashMap;
    use std::fmt::Write;

    // Group children by parent
    let mut parent_children: HashMap<&str, Vec<&str>> = HashMap::new();
    for def in HIGHLIGHTS {
        if !def.parent_tag.is_empty() && !def.tag.is_empty() {
            parent_children
                .entry(def.parent_tag)
                .or_default()
                .push(def.tag);
        }
    }

    let mut css = String::new();
    for (_parent, children) in parent_children {
        if children.is_empty() {
            continue;
        }
        // Create selector list: a-kc, a-kf, a-ki, ...
        let selectors: Vec<String> = children.iter().map(|c| format!("a-{c}")).collect();
        writeln!(css, "{} {{ color: inherit; }}", selectors.join(", ")).unwrap();
    }
    css
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_names_count() {
        assert_eq!(names().len(), COUNT);
    }

    #[test]
    fn test_none_produces_no_tag() {
        // Find the "none" index
        let none_idx = HIGHLIGHTS.iter().position(|h| h.name == "none").unwrap();
        assert_eq!(tag(none_idx), None);
    }

    #[test]
    fn test_keyword_tag() {
        let kw_idx = HIGHLIGHTS.iter().position(|h| h.name == "keyword").unwrap();
        assert_eq!(tag(kw_idx), Some("k"));
        assert_eq!(prefixed_tag(kw_idx), Some("a-k".to_string()));
    }

    #[test]
    fn test_inheritance() {
        let kc_idx = HIGHLIGHTS
            .iter()
            .position(|h| h.name == "keyword.conditional")
            .unwrap();
        assert_eq!(parent_tag(kc_idx), Some("k"));
    }
}
