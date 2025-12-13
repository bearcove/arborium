//! Highlight category definitions - single source of truth.
//!
//! This module defines all highlight categories used for syntax highlighting.
//! It maps the large vocabulary of capture names from various sources (nvim-treesitter,
//! helix, etc.) to a small set of theme slots.
//!
//! # Architecture
//!
//! The highlighting system has three layers:
//!
//! 1. **Capture names** - The broad vocabulary used in highlight queries
//!    (e.g., `@keyword.function`, `@include`, `@conditional`, `@repeat`)
//!
//! 2. **Theme slots** - A fixed set of ~15-20 color slots that themes define
//!    (e.g., `keyword`, `function`, `string`, `comment`, `type`)
//!
//! 3. **HTML tags** - Short tags for rendering (e.g., `<a-k>`, `<a-f>`, `<a-s>`)
//!
//! Multiple capture names map to the same theme slot. For example:
//! - `include`, `keyword.import`, `keyword.require` → all use the `keyword` slot
//! - `conditional`, `keyword.conditional`, `repeat` → all use the `keyword` slot
//!
//! Adjacent spans that map to the same slot are coalesced into a single HTML element.

/// The theme slots - the fixed set of color categories that themes define.
/// This is the final destination for all capture names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ThemeSlot {
    Keyword,
    Function,
    String,
    Comment,
    Type,
    Variable,
    Constant,
    Number,
    Operator,
    Punctuation,
    Property,
    Attribute,
    Tag,
    Macro,
    Label,
    Namespace,
    Constructor,
    /// Markup: headings, titles
    Title,
    /// Markup: bold text
    Strong,
    /// Markup: italic text
    Emphasis,
    /// Markup: links/URLs
    Link,
    /// Markup: raw/literal/code blocks
    Literal,
    /// Markup: strikethrough
    Strikethrough,
    /// Diff additions
    DiffAdd,
    /// Diff deletions
    DiffDelete,
    /// Embedded content
    Embedded,
    /// Errors
    Error,
    /// No styling (invisible captures like spell, nospell)
    None,
}

impl ThemeSlot {
    /// Get the HTML tag suffix for this slot.
    /// Returns None for slots that produce no styling.
    pub fn tag(self) -> Option<&'static str> {
        match self {
            ThemeSlot::Keyword => Some("k"),
            ThemeSlot::Function => Some("f"),
            ThemeSlot::String => Some("s"),
            ThemeSlot::Comment => Some("c"),
            ThemeSlot::Type => Some("t"),
            ThemeSlot::Variable => Some("v"),
            ThemeSlot::Constant => Some("co"),
            ThemeSlot::Number => Some("n"),
            ThemeSlot::Operator => Some("o"),
            ThemeSlot::Punctuation => Some("p"),
            ThemeSlot::Property => Some("pr"),
            ThemeSlot::Attribute => Some("at"),
            ThemeSlot::Tag => Some("tg"),
            ThemeSlot::Macro => Some("m"),
            ThemeSlot::Label => Some("l"),
            ThemeSlot::Namespace => Some("ns"),
            ThemeSlot::Constructor => Some("cr"),
            // Markup: headings, titles
            ThemeSlot::Title => Some("tt"),
            // Markup: bold text
            ThemeSlot::Strong => Some("st"),
            // Markup: italic text
            ThemeSlot::Emphasis => Some("em"),
            // Markup: links/URLs
            ThemeSlot::Link => Some("tu"),
            // Markup: raw/literal/code blocks
            ThemeSlot::Literal => Some("tl"),
            // Markup: strikethrough
            ThemeSlot::Strikethrough => Some("tx"),
            // Diff additions
            ThemeSlot::DiffAdd => Some("da"),
            // Diff deletions
            ThemeSlot::DiffDelete => Some("dd"),
            // Embedded content
            ThemeSlot::Embedded => Some("eb"),
            // Errors
            ThemeSlot::Error => Some("er"),
            // No styling (invisible captures like spell, nospell)
            ThemeSlot::None => None,
        }
    }

    /// Get the full name for this slot (for CSS class names).
    /// Returns None for slots that produce no styling.
    pub fn name(self) -> Option<&'static str> {
        match self {
            ThemeSlot::Keyword => Some("keyword"),
            ThemeSlot::Function => Some("function"),
            ThemeSlot::String => Some("string"),
            ThemeSlot::Comment => Some("comment"),
            ThemeSlot::Type => Some("type"),
            ThemeSlot::Variable => Some("variable"),
            ThemeSlot::Constant => Some("constant"),
            ThemeSlot::Number => Some("number"),
            ThemeSlot::Operator => Some("operator"),
            ThemeSlot::Punctuation => Some("punctuation"),
            ThemeSlot::Property => Some("property"),
            ThemeSlot::Attribute => Some("attribute"),
            ThemeSlot::Tag => Some("tag"),
            ThemeSlot::Macro => Some("macro"),
            ThemeSlot::Label => Some("label"),
            ThemeSlot::Namespace => Some("namespace"),
            ThemeSlot::Constructor => Some("constructor"),
            ThemeSlot::Title => Some("title"),
            ThemeSlot::Strong => Some("strong"),
            ThemeSlot::Emphasis => Some("emphasis"),
            ThemeSlot::Link => Some("link"),
            ThemeSlot::Literal => Some("literal"),
            ThemeSlot::Strikethrough => Some("strikethrough"),
            ThemeSlot::DiffAdd => Some("diff-add"),
            ThemeSlot::DiffDelete => Some("diff-delete"),
            ThemeSlot::Embedded => Some("embedded"),
            ThemeSlot::Error => Some("error"),
            ThemeSlot::None => None,
        }
    }
}

/// Map a theme slot to a canonical highlight index.
///
/// This is useful for ANSI rendering, where we want to
/// look up a single representative style for each slot.
pub fn slot_to_highlight_index(slot: ThemeSlot) -> Option<usize> {
    match slot {
        ThemeSlot::Keyword => HIGHLIGHTS.iter().position(|h| h.name == "keyword"),
        ThemeSlot::Function => HIGHLIGHTS.iter().position(|h| h.name == "function"),
        ThemeSlot::String => HIGHLIGHTS.iter().position(|h| h.name == "string"),
        ThemeSlot::Comment => HIGHLIGHTS.iter().position(|h| h.name == "comment"),
        ThemeSlot::Type => HIGHLIGHTS.iter().position(|h| h.name == "type"),
        ThemeSlot::Variable => HIGHLIGHTS.iter().position(|h| h.name == "variable"),
        ThemeSlot::Constant => HIGHLIGHTS.iter().position(|h| h.name == "constant"),
        ThemeSlot::Number => HIGHLIGHTS.iter().position(|h| h.name == "number"),
        ThemeSlot::Operator => HIGHLIGHTS.iter().position(|h| h.name == "operator"),
        ThemeSlot::Punctuation => HIGHLIGHTS.iter().position(|h| h.name == "punctuation"),
        ThemeSlot::Property => HIGHLIGHTS.iter().position(|h| h.name == "property"),
        ThemeSlot::Attribute => HIGHLIGHTS.iter().position(|h| h.name == "attribute"),
        ThemeSlot::Tag => HIGHLIGHTS.iter().position(|h| h.name == "tag"),
        ThemeSlot::Macro => HIGHLIGHTS.iter().position(|h| h.name == "macro"),
        ThemeSlot::Label => HIGHLIGHTS.iter().position(|h| h.name == "label"),
        ThemeSlot::Namespace => HIGHLIGHTS.iter().position(|h| h.name == "namespace"),
        ThemeSlot::Constructor => HIGHLIGHTS.iter().position(|h| h.name == "constructor"),
        ThemeSlot::Title => HIGHLIGHTS
            .iter()
            .position(|h| h.name == "text.title" || h.name == "markup.heading"),
        ThemeSlot::Strong => HIGHLIGHTS
            .iter()
            .position(|h| h.name == "text.strong" || h.name == "markup.bold"),
        ThemeSlot::Emphasis => HIGHLIGHTS
            .iter()
            .position(|h| h.name == "text.emphasis" || h.name == "markup.italic"),
        ThemeSlot::Link => HIGHLIGHTS
            .iter()
            .position(|h| h.name == "text.uri" || h.name == "text.reference"),
        ThemeSlot::Literal => HIGHLIGHTS.iter().position(|h| h.name == "text.literal"),
        ThemeSlot::Strikethrough => HIGHLIGHTS
            .iter()
            .position(|h| h.name == "text.strikethrough"),
        ThemeSlot::DiffAdd => HIGHLIGHTS.iter().position(|h| h.name == "diff.addition"),
        ThemeSlot::DiffDelete => HIGHLIGHTS.iter().position(|h| h.name == "diff.deletion"),
        ThemeSlot::Embedded => HIGHLIGHTS.iter().position(|h| h.name == "embedded"),
        ThemeSlot::Error => HIGHLIGHTS.iter().position(|h| h.name == "error"),
        ThemeSlot::None => None,
    }
}

/// Map any capture name to its theme slot.
///
/// This handles the full vocabulary of capture names from various sources:
/// - Standard tree-sitter names (keyword, function, string, etc.)
/// - nvim-treesitter names (include, conditional, repeat, storageclass, etc.)
/// - Helix names
/// - Sub-categories (keyword.function, keyword.import, etc.)
///
/// All are mapped to a fixed set of ~20 theme slots.
pub fn capture_to_slot(capture: &str) -> ThemeSlot {
    // First, strip any leading @ (some queries include it)
    let capture = capture.strip_prefix('@').unwrap_or(capture);

    match capture {
        // Keywords - base and all variants
        "keyword" | "keyword.conditional" | "keyword.coroutine" | "keyword.debug"
        | "keyword.exception" | "keyword.function" | "keyword.import" | "keyword.operator"
        | "keyword.repeat" | "keyword.return" | "keyword.type" | "keyword.modifier"
        | "keyword.directive" | "keyword.storage" | "keyword.control"
        | "keyword.control.conditional" | "keyword.control.repeat" | "keyword.control.import"
        | "keyword.control.return" | "keyword.control.exception"
        // nvim-treesitter legacy names that are really keywords
        | "include" | "conditional" | "repeat" | "exception" | "storageclass" | "preproc"
        | "define" | "structure" => ThemeSlot::Keyword,

        // Functions
        "function" | "function.builtin" | "function.method" | "function.definition"
        | "function.call" | "function.special" | "method" | "method.call" => ThemeSlot::Function,

        // Strings
        "string" | "string.special" | "string.special.symbol" | "string.special.path"
        | "string.special.url" | "string.escape" | "string.regexp" | "string.regex"
        | "character" | "character.special" | "escape" => ThemeSlot::String,

        // Comments
        "comment" | "comment.documentation" | "comment.line" | "comment.block"
        | "comment.error" | "comment.warning" | "comment.note" | "comment.todo" => {
            ThemeSlot::Comment
        }

        // Types
        "type" | "type.builtin" | "type.qualifier" | "type.definition" | "type.enum"
        | "type.enum.variant" | "type.parameter" => ThemeSlot::Type,

        // Variables
        "variable" | "variable.builtin" | "variable.parameter" | "variable.member"
        | "variable.other" | "variable.other.member" | "parameter" | "field" => {
            ThemeSlot::Variable
        }

        // Constants
        "constant" | "constant.builtin" | "constant.builtin.boolean" | "boolean" => {
            ThemeSlot::Constant
        }

        // Numbers
        "number" | "constant.numeric" | "float" | "number.float" => ThemeSlot::Number,

        // Operators
        "operator" => ThemeSlot::Operator,

        // Punctuation
        "punctuation" | "punctuation.bracket" | "punctuation.delimiter" | "punctuation.special" => {
            ThemeSlot::Punctuation
        }

        // Properties
        "property" | "property.builtin" => ThemeSlot::Property,

        // Attributes
        "attribute" | "attribute.builtin" => ThemeSlot::Attribute,

        // Tags (HTML/XML)
        "tag" | "tag.delimiter" | "tag.error" | "tag.attribute" | "tag.builtin" => ThemeSlot::Tag,

        // Macros
        "macro" | "function.macro" | "preproc.macro" => ThemeSlot::Macro,

        // Labels
        "label" => ThemeSlot::Label,

        // Namespaces/Modules
        "namespace" | "module" => ThemeSlot::Namespace,

        // Constructors
        "constructor" | "constructor.builtin" => ThemeSlot::Constructor,

        // Markup - titles/headings
        "text.title" | "markup.heading" | "markup.heading.1" | "markup.heading.2"
        | "markup.heading.3" | "markup.heading.4" | "markup.heading.5" | "markup.heading.6" => {
            ThemeSlot::Title
        }

        // Markup - bold
        "text.strong" | "markup.bold" => ThemeSlot::Strong,

        // Markup - italic
        "text.emphasis" | "markup.italic" => ThemeSlot::Emphasis,

        // Markup - links
        "text.uri" | "text.reference" | "markup.link" | "markup.link.url" | "markup.link.text"
        | "markup.link.label" => ThemeSlot::Link,

        // Markup - literal/raw/code
        "text.literal" | "markup.raw" | "markup.raw.block" | "markup.raw.inline"
        | "markup.inline" => ThemeSlot::Literal,

        // Markup - strikethrough
        "text.strikethrough" | "markup.strikethrough" => ThemeSlot::Strikethrough,

        // Markup - lists (treat as punctuation)
        "markup.list" | "markup.list.checked" | "markup.list.unchecked"
        | "markup.list.numbered" | "markup.list.unnumbered" | "markup.quote" => {
            ThemeSlot::Punctuation
        }

        // Diff
        "diff.addition" | "diff.plus" | "diff.delta" => ThemeSlot::DiffAdd,
        "diff.deletion" | "diff.minus" => ThemeSlot::DiffDelete,

        // Embedded
        "embedded" => ThemeSlot::Embedded,

        // Error
        "error" => ThemeSlot::Error,

        // No styling
        "none" | "nospell" | "spell" | "text" | "markup" => ThemeSlot::None,

        // Fallback: try to match by prefix
        other => {
            if other.starts_with("keyword") {
                ThemeSlot::Keyword
            } else if other.starts_with("function") || other.starts_with("method") {
                ThemeSlot::Function
            } else if other.starts_with("string") || other.starts_with("character") {
                ThemeSlot::String
            } else if other.starts_with("comment") {
                ThemeSlot::Comment
            } else if other.starts_with("type") {
                ThemeSlot::Type
            } else if other.starts_with("variable") || other.starts_with("parameter") {
                ThemeSlot::Variable
            } else if other.starts_with("constant") {
                ThemeSlot::Constant
            } else if other.starts_with("punctuation") {
                ThemeSlot::Punctuation
            } else if other.starts_with("tag") {
                ThemeSlot::Tag
            } else if other.starts_with("markup.heading") || other.starts_with("text.title") {
                ThemeSlot::Title
            } else if other.starts_with("markup") || other.starts_with("text") {
                // Generic markup/text - no styling
                ThemeSlot::None
            } else {
                // Unknown capture - no styling
                ThemeSlot::None
            }
        }
    }
}

/// A highlight category definition.
///
/// NOTE: This is the legacy structure used for tree-sitter highlight configuration.
/// For mapping captures to theme slots, use [`capture_to_slot`] instead.
pub struct HighlightDef {
    /// The canonical name (e.g., "keyword.function")
    pub name: &'static str,
    /// Short tag suffix for HTML elements (e.g., "kf" -> `<a-kf>`)
    /// Empty string means no styling should be applied.
    pub tag: &'static str,
    /// Parent category tag for CSS inheritance (e.g., "k" for keyword.*)
    /// Empty string means no parent.
    pub parent_tag: &'static str,
    /// Alternative names from nvim-treesitter/helix/other editors that map to this category
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

/// Get the HTML tag for a capture name directly.
///
/// This is the main function to use when rendering HTML from captures.
/// It maps any capture name to its theme slot and returns the tag.
///
/// Returns None for captures that should produce no styling.
///
/// # Example
/// ```
/// use arborium_theme::highlights::tag_for_capture;
///
/// // All these map to the keyword slot ("k")
/// assert_eq!(tag_for_capture("keyword"), Some("k"));
/// assert_eq!(tag_for_capture("keyword.function"), Some("k"));
/// assert_eq!(tag_for_capture("include"), Some("k"));
/// assert_eq!(tag_for_capture("conditional"), Some("k"));
///
/// // No styling for these
/// assert_eq!(tag_for_capture("spell"), None);
/// assert_eq!(tag_for_capture("nospell"), None);
/// ```
pub fn tag_for_capture(capture: &str) -> Option<&'static str> {
    capture_to_slot(capture).tag()
}

/// Map a short tag to its full name.
///
/// This is useful for class-based HTML output where you need
/// `<span class="keyword">` instead of `<a-k>`.
///
/// # Example
/// ```
/// use arborium_theme::highlights::tag_to_name;
///
/// assert_eq!(tag_to_name("k"), Some("keyword"));
/// assert_eq!(tag_to_name("f"), Some("function"));
/// assert_eq!(tag_to_name("s"), Some("string"));
/// ```
pub fn tag_to_name(tag: &str) -> Option<&'static str> {
    match tag {
        "k" => Some("keyword"),
        "f" => Some("function"),
        "s" => Some("string"),
        "c" => Some("comment"),
        "t" => Some("type"),
        "v" => Some("variable"),
        "co" => Some("constant"),
        "n" => Some("number"),
        "o" => Some("operator"),
        "p" => Some("punctuation"),
        "pr" => Some("property"),
        "at" => Some("attribute"),
        "tg" => Some("tag"),
        "m" => Some("macro"),
        "l" => Some("label"),
        "ns" => Some("namespace"),
        "cr" => Some("constructor"),
        "tt" => Some("title"),
        "st" => Some("strong"),
        "em" => Some("emphasis"),
        "tu" => Some("link"),
        "tl" => Some("literal"),
        "tx" => Some("strikethrough"),
        "da" => Some("diff-add"),
        "dd" => Some("diff-delete"),
        "eb" => Some("embedded"),
        "er" => Some("error"),
        _ => None,
    }
}

/// The complete list of capture names that arborium recognizes.
///
/// This list is used to configure tree-sitter's highlight query processor.
/// It includes all standard names plus common alternatives from nvim-treesitter,
/// helix, and other editors.
pub const CAPTURE_NAMES: &[&str] = &[
    // Keywords
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
    "keyword.modifier",
    "keyword.directive",
    "keyword.storage",
    "keyword.control",
    "keyword.control.conditional",
    "keyword.control.repeat",
    "keyword.control.import",
    "keyword.control.return",
    "keyword.control.exception",
    // nvim-treesitter legacy keyword names
    "include",
    "conditional",
    "repeat",
    "exception",
    "storageclass",
    "preproc",
    "define",
    "structure",
    // Functions
    "function",
    "function.builtin",
    "function.method",
    "function.definition",
    "function.call",
    "function.macro",
    "function.special",
    "method",
    "method.call",
    // Strings
    "string",
    "string.special",
    "string.special.symbol",
    "string.special.path",
    "string.special.url",
    "string.escape",
    "string.regexp",
    "string.regex",
    "character",
    "character.special",
    "escape",
    // Comments
    "comment",
    "comment.documentation",
    "comment.line",
    "comment.block",
    "comment.error",
    "comment.warning",
    "comment.note",
    "comment.todo",
    // Types
    "type",
    "type.builtin",
    "type.qualifier",
    "type.definition",
    "type.enum",
    "type.enum.variant",
    "type.parameter",
    // Variables
    "variable",
    "variable.builtin",
    "variable.parameter",
    "variable.member",
    "variable.other",
    "variable.other.member",
    "parameter",
    "field",
    // Constants
    "constant",
    "constant.builtin",
    "constant.builtin.boolean",
    "constant.numeric",
    "boolean",
    // Numbers
    "number",
    "float",
    "number.float",
    // Operators
    "operator",
    // Punctuation
    "punctuation",
    "punctuation.bracket",
    "punctuation.delimiter",
    "punctuation.special",
    // Properties
    "property",
    "property.builtin",
    // Attributes
    "attribute",
    "attribute.builtin",
    // Tags
    "tag",
    "tag.delimiter",
    "tag.error",
    "tag.attribute",
    "tag.builtin",
    // Macros
    "macro",
    // Labels
    "label",
    // Namespaces
    "namespace",
    "module",
    // Constructors
    "constructor",
    "constructor.builtin",
    // Markup - titles
    "text.title",
    "markup.heading",
    "markup.heading.1",
    "markup.heading.2",
    "markup.heading.3",
    "markup.heading.4",
    "markup.heading.5",
    "markup.heading.6",
    // Markup - emphasis
    "text.strong",
    "markup.bold",
    "text.emphasis",
    "markup.italic",
    // Markup - links
    "text.uri",
    "text.reference",
    "markup.link",
    "markup.link.url",
    "markup.link.text",
    "markup.link.label",
    // Markup - code/raw
    "text.literal",
    "markup.raw",
    "markup.raw.block",
    "markup.raw.inline",
    "markup.inline",
    // Markup - strikethrough
    "text.strikethrough",
    "markup.strikethrough",
    // Markup - lists
    "markup.list",
    "markup.list.checked",
    "markup.list.unchecked",
    "markup.list.numbered",
    "markup.list.unnumbered",
    "markup.quote",
    // Markup - generic
    "text",
    "markup",
    // Diff
    "diff.addition",
    "diff.plus",
    "diff.delta",
    "diff.deletion",
    "diff.minus",
    // Special
    "embedded",
    "error",
    "none",
    "nospell",
    "spell",
];

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

    #[test]
    fn test_capture_to_slot_keywords() {
        // All keyword variants map to Keyword slot
        assert_eq!(capture_to_slot("keyword"), ThemeSlot::Keyword);
        assert_eq!(capture_to_slot("keyword.function"), ThemeSlot::Keyword);
        assert_eq!(capture_to_slot("keyword.import"), ThemeSlot::Keyword);
        assert_eq!(capture_to_slot("include"), ThemeSlot::Keyword);
        assert_eq!(capture_to_slot("conditional"), ThemeSlot::Keyword);
        assert_eq!(capture_to_slot("repeat"), ThemeSlot::Keyword);
        assert_eq!(capture_to_slot("storageclass"), ThemeSlot::Keyword);
    }

    #[test]
    fn test_capture_to_slot_functions() {
        assert_eq!(capture_to_slot("function"), ThemeSlot::Function);
        assert_eq!(capture_to_slot("function.builtin"), ThemeSlot::Function);
        assert_eq!(capture_to_slot("function.method"), ThemeSlot::Function);
        assert_eq!(capture_to_slot("method"), ThemeSlot::Function);
    }

    #[test]
    fn test_capture_to_slot_markup() {
        assert_eq!(capture_to_slot("markup.heading"), ThemeSlot::Title);
        assert_eq!(capture_to_slot("markup.heading.1"), ThemeSlot::Title);
        assert_eq!(capture_to_slot("text.title"), ThemeSlot::Title);
        assert_eq!(capture_to_slot("markup.bold"), ThemeSlot::Strong);
        assert_eq!(capture_to_slot("markup.italic"), ThemeSlot::Emphasis);
    }

    #[test]
    fn test_capture_to_slot_none() {
        assert_eq!(capture_to_slot("none"), ThemeSlot::None);
        assert_eq!(capture_to_slot("spell"), ThemeSlot::None);
        assert_eq!(capture_to_slot("nospell"), ThemeSlot::None);
    }

    #[test]
    fn test_tag_for_capture() {
        // Keywords all get "k"
        assert_eq!(tag_for_capture("keyword"), Some("k"));
        assert_eq!(tag_for_capture("keyword.function"), Some("k"));
        assert_eq!(tag_for_capture("include"), Some("k"));
        assert_eq!(tag_for_capture("conditional"), Some("k"));

        // Functions get "f"
        assert_eq!(tag_for_capture("function"), Some("f"));
        assert_eq!(tag_for_capture("function.builtin"), Some("f"));

        // Comments get "c"
        assert_eq!(tag_for_capture("comment"), Some("c"));
        assert_eq!(tag_for_capture("comment.documentation"), Some("c"));

        // No tag for special captures
        assert_eq!(tag_for_capture("spell"), None);
        assert_eq!(tag_for_capture("none"), None);
    }

    #[test]
    fn test_theme_slot_tag() {
        assert_eq!(ThemeSlot::Keyword.tag(), Some("k"));
        assert_eq!(ThemeSlot::Function.tag(), Some("f"));
        assert_eq!(ThemeSlot::String.tag(), Some("s"));
        assert_eq!(ThemeSlot::Comment.tag(), Some("c"));
        assert_eq!(ThemeSlot::None.tag(), None);
    }

    #[test]
    fn test_capture_names_all_map_to_slot() {
        // Every name in CAPTURE_NAMES should produce a valid mapping
        for name in CAPTURE_NAMES {
            let slot = capture_to_slot(name);
            // Just verify it doesn't panic and produces some slot
            let _ = slot.tag();
        }
    }
}
