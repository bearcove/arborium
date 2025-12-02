//! Core types for the arborium xtask system.
//!
//! This module defines the data structures used throughout xtask, primarily
//! for representing grammar/language metadata stored in `arborium.kdl` files.
//!
//! # File Format
//!
//! Each crate in `crates/arborium-*` contains an `arborium.kdl` file that
//! describes the language grammar. This is the single source of truth for:
//!
//! - Upstream repository and commit information
//! - Language metadata (name, icon, description, etc.)
//! - Sample files for testing and demos
//! - Build configuration for special cases
//!
//! # Example `arborium.kdl`
//!
//! ```kdl
//! id "rust"
//! name "Rust"
//! tag "code"
//! tier 1
//! icon "devicon-plain:rust"
//! aliases "rs"
//!
//! repo "https://codeberg.org/grammar-orchard/tree-sitter-rust-orchard"
//! commit "261b20226c04ef601adbdf185a800512a5f66291"
//! license "MIT"
//!
//! inventor "Graydon Hoare"
//! year 2010
//! description "Systems language focused on safety and performance without GC"
//! link "https://en.wikipedia.org/wiki/Rust_(programming_language)"
//! trivia "Hoare began Rust as a side project at Mozilla in 2006"
//!
//! sample {
//!     path "samples/example.rs"
//!     description "Clippy lint implementation"
//!     link "https://github.com/rust-lang/rust/blob/main/..."
//!     license "MIT OR Apache-2.0"
//! }
//! ```

use std::collections::BTreeMap;

use camino::{Utf8Path, Utf8PathBuf};
use facet::Facet;
pub use rootcause::Report;

/// Complete metadata for a language grammar crate.
///
/// This struct represents the contents of an `arborium.kdl` file and serves
/// as the single source of truth for all grammar-related information.
///
/// The struct is used by xtask commands for:
/// - `grammars vendor` - Creating new grammar crates
/// - `grammars update` - Checking for upstream updates
/// - `grammars generate` - Regenerating parser sources and crate code
/// - `serve-demo` - Building the demo with language metadata
#[derive(Debug, Clone, Facet)]
pub struct GrammarInfo {
    // =========================================================================
    // Identity
    // =========================================================================
    /// Unique identifier for this grammar, used in crate names and feature flags.
    ///
    /// This should be a lowercase, hyphen-separated string (e.g., "rust", "c-sharp").
    /// The crate will be named `arborium-{id}` and the feature flag `lang-{id}`.
    pub id: String,

    /// Human-readable display name for the language (e.g., "Rust", "C#", "TypeScript").
    pub name: String,

    /// Category tag for grouping languages in the UI.
    ///
    /// Common values: "code", "markup", "config", "data", "shell", "query", "build"
    pub tag: String,

    /// Quality/completeness tier (1 = best, 3 = experimental).
    ///
    /// - Tier 1: Well-tested, complete highlighting, curated samples
    /// - Tier 2: Working but may have gaps in highlighting
    /// - Tier 3: Experimental or incomplete
    #[facet(default)]
    pub tier: Option<u8>,

    /// Iconify icon identifier (e.g., "devicon-plain:rust", "mdi:language-python").
    #[facet(default)]
    pub icon: Option<String>,

    /// Alternative names or file extensions for this language.
    ///
    /// Used for language detection (e.g., ["rs"] for Rust, ["ts", "mts", "cts"] for TypeScript).
    #[facet(default)]
    pub aliases: Vec<String>,

    // =========================================================================
    // Upstream Source
    // =========================================================================
    /// Git repository URL for the upstream tree-sitter grammar.
    ///
    /// Use "local" for grammars that are maintained in this repository.
    pub repo: String,

    /// Git commit hash of the vendored version.
    ///
    /// This is updated by `cargo xtask grammars vendor` and checked by
    /// `cargo xtask grammars update`.
    pub commit: String,

    /// SPDX license identifier for the grammar (e.g., "MIT", "Apache-2.0", "GPL-3.0").
    pub license: String,

    /// Subdirectory within the repo containing the grammar (for multi-grammar repos).
    ///
    /// For example, tree-sitter-typescript has `typescript/` and `tsx/` subdirectories.
    #[facet(default)]
    pub subdir: Option<String>,

    // =========================================================================
    // Language Metadata (for demos and documentation)
    // =========================================================================
    /// Creator(s) of the programming language.
    #[facet(default)]
    pub inventor: Option<String>,

    /// Year the language was first released.
    #[facet(default)]
    pub year: Option<u16>,

    /// Brief description of the language and its primary use cases.
    ///
    /// May contain Markdown links for references.
    #[facet(default)]
    pub description: Option<String>,

    /// URL to more information (typically Wikipedia or official docs).
    #[facet(default)]
    pub link: Option<String>,

    /// Fun facts or interesting history about the language.
    ///
    /// Shown in the demo UI to make learning about languages more engaging.
    #[facet(default)]
    pub trivia: Option<String>,

    /// Whether this sample was hand-picked for quality.
    #[facet(default)]
    pub handpicked: Option<bool>,

    // =========================================================================
    // Samples
    // =========================================================================
    /// Sample files for testing highlighting and displaying in demos.
    #[facet(default)]
    pub samples: Vec<SampleInfo>,

    // =========================================================================
    // Build Configuration
    // =========================================================================
    /// Build-time configuration for special cases.
    ///
    /// Most grammars don't need this - it's auto-detected from the grammar sources.
    #[facet(default)]
    pub build: Option<BuildConfig>,
}

/// Metadata for a sample source file.
///
/// Samples are used for:
/// - Testing that highlighting works correctly
/// - Displaying in the demo UI
/// - Validating grammar completeness
#[derive(Debug, Clone, Facet)]
pub struct SampleInfo {
    /// Path to the sample file, relative to the crate root.
    ///
    /// Typically something like "samples/example.rs".
    pub path: String,

    /// Brief description of what the sample demonstrates.
    pub description: Option<String>,

    /// URL to the original source of this sample (for attribution).
    #[facet(default)]
    pub link: Option<String>,

    /// License of the sample file (may differ from the grammar license).
    #[facet(default)]
    pub license: Option<String>,
}

/// Build configuration for grammars that need special handling.
///
/// Most grammars are auto-detected and don't need explicit configuration.
/// This is only needed for edge cases like:
/// - Grammars with non-standard C symbol names
/// - Grammars that export multiple languages (e.g., TypeScript + TSX)
/// - Grammars with unusual query directory structures
#[derive(Debug, Clone, Default, Facet)]
pub struct BuildConfig {
    /// Override the C symbol name (defaults to id with hyphens replaced by underscores).
    ///
    /// The generated code calls `tree_sitter_{c_symbol}()` to get the language.
    #[facet(default)]
    pub c_symbol: Option<String>,

    /// Path prefix for query files within the queries/ directory.
    ///
    /// Some grammars have nested query directories (e.g., "just/" for tree-sitter-just).
    #[facet(default)]
    pub query_path: Option<String>,

    /// For sub-grammars: the parent repository name.
    ///
    /// Used to find shared query files (e.g., "typescript" for the tsx sub-grammar).
    #[facet(default)]
    pub parent_repo: Option<String>,

    /// Languages whose queries should be inherited (prepended to this grammar's queries).
    ///
    /// For example, TypeScript inherits from JavaScript.
    #[facet(default)]
    pub inherits_queries_from: Vec<String>,

    /// Additional languages exported by this grammar crate.
    ///
    /// For grammars like tree-sitter-typescript that export both TypeScript and TSX.
    #[facet(default)]
    pub extra_languages: Vec<ExtraLanguage>,
}

/// An additional language exported by a grammar crate.
///
/// Used for multi-language grammars like tree-sitter-typescript.
#[derive(Debug, Clone, Facet)]
pub struct ExtraLanguage {
    /// C symbol name for this language (e.g., "tsx").
    pub c_symbol: String,

    /// Export name in the generated Rust code (e.g., "tsx").
    pub export_name: String,
}

/// Registry of all grammar crates in the workspace.
///
/// Built by scanning `crates/arborium-*/arborium.kdl` files at startup.
#[derive(Debug, Default)]
pub struct GrammarRegistry {
    /// All grammars, keyed by their id.
    pub grammars: BTreeMap<String, GrammarInfo>,
}

impl GrammarRegistry {
    /// Load the grammar registry by scanning all arborium-* crates.
    ///
    /// This reads `arborium.kdl` from each crate directory and builds
    /// a complete registry of all available grammars.
    pub fn load(crates_dir: &Utf8Path) -> Result<Self, Report> {
        let mut grammars = BTreeMap::new();

        for entry in std::fs::read_dir(crates_dir)? {
            let entry = entry?;
            let path = entry.path();

            if !path.is_dir() {
                continue;
            }

            let dir_name = path.file_name().unwrap().to_string_lossy();
            if !dir_name.starts_with("arborium-") {
                continue;
            }

            // Skip utility crates
            let crate_suffix = dir_name.strip_prefix("arborium-").unwrap();
            if matches!(crate_suffix, "sysroot" | "test-harness") {
                continue;
            }

            let kdl_path = path.join("arborium.kdl");
            if !kdl_path.exists() {
                // TODO: warn about missing arborium.kdl?
                continue;
            }

            let content = std::fs::read_to_string(&kdl_path)?;
            let info: GrammarInfo = facet_kdl::from_str(&content)?;

            grammars.insert(info.id.clone(), info);
        }

        Ok(Self { grammars })
    }

    /// Get a grammar by its id.
    pub fn get(&self, id: &str) -> Option<&GrammarInfo> {
        self.grammars.get(id)
    }

    /// Iterate over all grammars.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &GrammarInfo)> {
        self.grammars.iter()
    }

    /// Number of grammars in the registry.
    pub fn len(&self) -> usize {
        self.grammars.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.grammars.is_empty()
    }
}
