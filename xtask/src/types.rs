//! Core types for the arborium xtask system.
//!
//! This module defines the data structures used throughout xtask, primarily
//! for representing grammar/language metadata stored in `arborium.kdl` files.
//!
//! # File Format
//!
//! Each crate in `crates/arborium-*` contains an `arborium.kdl` file that
//! describes one or more language grammars. This is the single source of truth for:
//!
//! - Upstream repository and commit information (crate-level)
//! - Language metadata (name, icon, description, etc.) per grammar
//! - Sample files for testing and demos
//! - Build configuration for special cases
//!
//! # Example `arborium.kdl` (single grammar, most common)
//!
//! ```kdl
//! repo "https://github.com/tree-sitter/tree-sitter-rust"
//! commit "261b20226c04ef601adbdf185a800512a5f66291"
//! license "MIT"
//! authors "Maxim Sokolov"
//!
//! grammar {
//!     id "rust"
//!     name "Rust"
//!     tag "code"
//!     tier 1
//!     icon "devicon-plain:rust"
//!     aliases "rs"
//!     has-scanner true
//!     c-symbol "rust_orchard"
//!
//!     inventor "Graydon Hoare"
//!     year 2010
//!     description "Systems language focused on safety and performance without GC"
//!     link "https://en.wikipedia.org/wiki/Rust_(programming_language)"
//!     trivia "Hoare began Rust as a side project at Mozilla in 2006"
//!
//!     sample {
//!         path "samples/example.rs"
//!         description "Clippy lint implementation"
//!         link "https://github.com/rust-lang/rust/blob/main/..."
//!         license "MIT OR Apache-2.0"
//!     }
//! }
//! ```
//!
//! # Example `arborium.kdl` (multi-grammar crate)
//!
//! ```kdl
//! repo "https://github.com/tree-sitter-grammars/tree-sitter-xml"
//! commit "863dbc381f44f6c136a399e684383b977bb2beaa"
//! license "MIT"
//! authors "ObserverOfTime"
//!
//! grammar {
//!     id "xml"
//!     name "XML"
//!     tag "markup"
//!     tier 3
//!     has-scanner true
//!     grammar-path "xml"
//!
//!     // ...metadata, samples...
//! }
//!
//! grammar {
//!     id "dtd"
//!     name "DTD"
//!     tag "markup"
//!     tier 3
//!     has-scanner true
//!     grammar-path "dtd"
//!
//!     // ...metadata, samples...
//! }
//! ```

#![allow(dead_code)]

use std::collections::BTreeMap;

use camino::{Utf8Path, Utf8PathBuf};
use facet::Facet;
use facet_kdl as kdl;
use facet_kdl::Spanned;
use fs_err as fs;
use miette::NamedSource;
pub use rootcause::Report;

// =============================================================================
// Crate-level configuration (parsed from arborium.kdl)
// =============================================================================

structstruck::strike! {
    /// Configuration for an entire arborium-* crate.
    ///
    /// This represents the contents of an `arborium.kdl` file. A crate can
    /// contain one or more grammars that share the same upstream source.
    #[strikethrough[derive(Debug, Clone, Facet)]]
    pub struct CrateConfig {
        /// Git repository URL for the upstream tree-sitter grammar.
        ///
        /// Use "local" for grammars that are maintained in this repository.
        #[facet(kdl::child)]
        pub repo: pub struct Repo {
            #[facet(kdl::argument)]
            pub value: Spanned<String>,
        },

        /// Git commit hash of the vendored version.
        #[facet(kdl::child)]
        pub commit: pub struct Commit {
            #[facet(kdl::argument)]
            pub value: Spanned<String>,
        },

        /// SPDX license identifier for the grammar (e.g., "MIT", "Apache-2.0").
        #[facet(kdl::child)]
        pub license: pub struct License {
            #[facet(kdl::argument)]
            pub value: Spanned<String>,
        },

        // TODO: Add authors field back once facet-kdl supports Option<T> + kdl::child + default

        /// One or more grammars exported by this crate.
        #[facet(kdl::children)]
        pub grammars: Vec<GrammarConfig>,
    }
}

/// Authors of the tree-sitter grammar.
#[derive(Debug, Clone, Facet)]
pub struct Authors {
    #[facet(kdl::argument)]
    pub value: String,
}

// =============================================================================
// KDL child node wrapper types
// =============================================================================
// In KDL, `name "value"` is a child node with an argument, not a property.
// These wrapper types allow facet-kdl to deserialize them correctly.

macro_rules! kdl_child_string {
    ($name:ident) => {
        #[derive(Debug, Clone, Facet)]
        pub struct $name {
            #[facet(kdl::argument)]
            pub value: Spanned<String>,
        }

        impl std::ops::Deref for $name {
            type Target = str;
            fn deref(&self) -> &Self::Target {
                &self.value
            }
        }
    };
}

macro_rules! kdl_child_string_optional {
    ($name:ident) => {
        #[derive(Debug, Clone, Facet)]
        pub struct $name {
            #[facet(kdl::argument)]
            pub value: String,
        }

        impl std::ops::Deref for $name {
            type Target = str;
            fn deref(&self) -> &Self::Target {
                &self.value
            }
        }
    };
}

// Required string children
kdl_child_string!(Id);
kdl_child_string!(Name);
kdl_child_string!(Tag);
kdl_child_string!(GrammarPath);
kdl_child_string!(CSymbol);
kdl_child_string!(Path);

// Optional string children (no span tracking needed)
kdl_child_string_optional!(Icon);
kdl_child_string_optional!(Inventor);
kdl_child_string_optional!(Description);
kdl_child_string_optional!(Link);
kdl_child_string_optional!(Trivia);
kdl_child_string_optional!(SampleDescription);
kdl_child_string_optional!(SampleLink);
kdl_child_string_optional!(SampleLicense);

/// Tier child node (u8 value).
#[derive(Debug, Clone, Facet)]
pub struct Tier {
    #[facet(kdl::argument)]
    pub value: Spanned<u8>,
}

impl std::ops::Deref for Tier {
    type Target = u8;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

/// Year child node (u16 value).
#[derive(Debug, Clone, Facet)]
pub struct Year {
    #[facet(kdl::argument)]
    pub value: u16,
}

impl std::ops::Deref for Year {
    type Target = u16;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

/// Has-scanner child node (bool value).
#[derive(Debug, Clone, Facet)]
pub struct HasScanner {
    #[facet(kdl::argument)]
    pub value: bool,
}

/// Internal grammar marker (bool value).
#[derive(Debug, Clone, Facet)]
pub struct Internal {
    #[facet(kdl::argument)]
    pub value: bool,
}

/// Tests cursed marker (bool value) - skip test generation for problematic grammars.
#[derive(Debug, Clone, Facet)]
pub struct TestsCursed {
    #[facet(kdl::argument)]
    pub value: bool,
}

/// Aliases child node (multiple string arguments).
#[derive(Debug, Clone, Facet)]
pub struct Aliases {
    #[facet(kdl::arguments)]
    pub values: Vec<String>,
}

// =============================================================================
// Per-grammar configuration
// =============================================================================

/// Configuration for a single grammar within a crate.
///
/// This contains all the metadata and build configuration for one language.
#[derive(Debug, Clone, Facet)]
#[facet(kdl::child, rename = "grammar")]
pub struct GrammarConfig {
    // =========================================================================
    // Identity
    // =========================================================================
    /// Unique identifier for this grammar, used in feature flags and exports.
    #[facet(kdl::child)]
    pub id: Id,

    /// Human-readable display name for the language.
    #[facet(kdl::child)]
    pub name: Name,

    /// Category tag for grouping languages in the UI.
    #[facet(kdl::child)]
    pub tag: Tag,

    /// Quality/completeness tier (1 = best, 5 = experimental).
    #[facet(kdl::child, default)]
    pub tier: Option<Tier>,

    /// Iconify icon identifier.
    #[facet(kdl::child, default)]
    pub icon: Option<Icon>,

    /// Alternative names or file extensions for this language.
    #[facet(kdl::child, default)]
    pub aliases: Option<Aliases>,

    // =========================================================================
    // Build Configuration
    // =========================================================================
    /// Internal grammar (used by other grammars via injection, not user-facing).
    #[facet(kdl::child, default)]
    pub internal: Option<Internal>,

    /// Tests are cursed (skip test generation due to platform issues).
    #[facet(kdl::child, default, rename = "tests-cursed")]
    pub tests_cursed: Option<TestsCursed>,

    /// Whether this grammar has a scanner.c file.
    #[facet(kdl::child, default, rename = "has-scanner")]
    pub has_scanner: Option<HasScanner>,

    /// Path to the grammar within the repo (for multi-grammar repos).
    #[facet(kdl::child, default, rename = "grammar-path")]
    pub grammar_path: Option<GrammarPath>,

    /// Override the C symbol name.
    #[facet(kdl::child, default, rename = "c-symbol")]
    pub c_symbol: Option<CSymbol>,

    /// Query configuration (highlights inheritance).
    #[facet(kdl::child, default)]
    pub queries: Option<QueriesConfig>,

    // =========================================================================
    // Language Metadata (for demos and documentation)
    // =========================================================================
    /// Creator(s) of the programming language.
    #[facet(kdl::child, default)]
    pub inventor: Option<Inventor>,

    /// Year the language was first released.
    #[facet(kdl::child, default)]
    pub year: Option<Year>,

    /// Brief description of the language.
    #[facet(kdl::child, default)]
    pub description: Option<Description>,

    /// URL to more information.
    #[facet(kdl::child, default)]
    pub link: Option<Link>,

    /// Fun facts or interesting history.
    #[facet(kdl::child, default)]
    pub trivia: Option<Trivia>,

    // =========================================================================
    // Samples
    // =========================================================================
    /// Sample files for testing highlighting and displaying in demos.
    #[facet(kdl::children, default)]
    pub samples: Vec<SampleConfig>,
}

impl GrammarConfig {
    /// Get the grammar ID as a string.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Whether this is an internal grammar (used via injection, not user-facing).
    pub fn is_internal(&self) -> bool {
        self.internal.as_ref().map(|i| i.value).unwrap_or(false)
    }

    /// Whether this grammar has a scanner.
    pub fn has_scanner(&self) -> bool {
        self.has_scanner.as_ref().map(|h| h.value).unwrap_or(false)
    }

    /// Whether tests are cursed (skip test generation).
    pub fn tests_cursed(&self) -> bool {
        self.tests_cursed.as_ref().map(|t| t.value).unwrap_or(false)
    }
}

/// Query configuration for a grammar.
#[derive(Debug, Clone, Facet)]
#[facet(kdl::child, rename = "queries")]
pub struct QueriesConfig {
    /// Highlights query configuration.
    #[facet(kdl::child, default)]
    pub highlights: Option<HighlightsConfig>,
}

/// Highlights query configuration.
#[derive(Debug, Clone, Facet)]
#[facet(kdl::child, rename = "highlights")]
pub struct HighlightsConfig {
    /// Queries to prepend from other grammars.
    #[facet(kdl::children, default)]
    pub prepend: Vec<PrependConfig>,
}

/// A reference to another grammar's queries to prepend.
#[derive(Debug, Clone, Facet)]
#[facet(kdl::child, rename = "prepend")]
pub struct PrependConfig {
    /// The crate to prepend from (e.g., "arborium-javascript").
    #[facet(kdl::property, rename = "crate")]
    pub crate_name: Spanned<String>,

    /// The grammar within that crate (optional if crate has only one grammar).
    #[facet(kdl::property, default)]
    pub grammar: Option<Spanned<String>>,
}

/// Metadata for a sample source file.
#[derive(Debug, Clone, Facet)]
#[facet(kdl::child, rename = "sample")]
pub struct SampleConfig {
    /// Path to the sample file, relative to the crate root.
    #[facet(kdl::child)]
    pub path: Path,

    /// Brief description of what the sample demonstrates.
    #[facet(kdl::child, default)]
    pub description: Option<SampleDescription>,

    /// URL to the original source of this sample (for attribution).
    #[facet(kdl::child, default)]
    pub link: Option<SampleLink>,

    /// License of the sample file (may differ from the grammar license).
    #[facet(kdl::child, default)]
    pub license: Option<SampleLicense>,
}

impl SampleConfig {
    /// Get the sample path as a string.
    pub fn path(&self) -> &str {
        &self.path
    }
}

// =============================================================================
// Crate state (what's on disk)
// =============================================================================

/// Complete state of an arborium-* crate, including config and disk state.
#[derive(Debug)]
pub struct CrateState {
    /// The crate name (e.g., "arborium-rust").
    pub name: String,

    /// Path to the crate directory.
    pub path: Utf8PathBuf,

    /// Parsed configuration from arborium.kdl (if present).
    pub config: Option<CrateConfig>,

    /// Raw KDL source for Miette diagnostics.
    pub kdl_source: Option<String>,

    /// State of files on disk.
    pub files: CrateFiles,
}

/// State of a single file.
#[derive(Debug, Default)]
pub enum FileState {
    #[default]
    Missing,
    Present {
        content: String,
    },
}

impl FileState {
    pub fn is_present(&self) -> bool {
        matches!(self, FileState::Present { .. })
    }

    pub fn content(&self) -> Option<&str> {
        match self {
            FileState::Present { content } => Some(content),
            FileState::Missing => None,
        }
    }
}

structstruck::strike! {
    /// State of files within a crate directory.
    #[strikethrough[derive(Debug, Default)]]
    pub struct CrateFiles {
        /// arborium.kdl - the source of truth
        pub kdl: FileState,

        /// Cargo.toml - generated
        pub cargo_toml: FileState,

        /// build.rs - generated
        pub build_rs: FileState,

        /// src/lib.rs - generated
        pub lib_rs: FileState,

        /// grammar/src/ directory state
        pub grammar_src: pub struct GrammarSrcState {
            /// parser.c - required
            pub parser_c: FileState,

            /// scanner.c - optional depending on grammar
            pub scanner_c: FileState,

            /// Other files present
            pub other_files: Vec<Utf8PathBuf>,
        },

        /// queries/ directory state
        pub queries: pub struct QueriesState {
            /// highlights.scm
            pub highlights: FileState,

            /// injections.scm
            pub injections: FileState,

            /// locals.scm
            pub locals: FileState,
        },

        /// Sample files declared in kdl
        pub samples: Vec<SampleState>,

        /// Legacy/unexpected files that should be deleted
        pub legacy_files: Vec<Utf8PathBuf>,
    }
}

/// State of a sample file.
#[derive(Debug)]
pub struct SampleState {
    /// Path relative to crate root (from kdl).
    pub path: String,

    /// What we found on disk.
    pub state: SampleFileState,
}

/// State of a sample file on disk.
#[derive(Debug)]
pub enum SampleFileState {
    /// File doesn't exist.
    Missing,

    /// File exists but is empty.
    Empty,

    /// File exists but contains an HTTP error (failed download).
    HttpError,

    /// File exists but is very short.
    TooShort { lines: usize },

    /// File is good.
    Ok { lines: usize },
}

// =============================================================================
// Registry
// =============================================================================

/// Registry of all grammar crates in the workspace.
///
/// Built by scanning `crates/arborium-*/` directories at startup.
/// Contains both parsed configuration and disk state for each crate.
#[derive(Debug, Default)]
pub struct CrateRegistry {
    /// All crates, keyed by crate name (e.g., "arborium-rust").
    pub crates: BTreeMap<String, CrateState>,
}

/// Crates to skip when scanning (internal/utility crates).
const SKIP_CRATES: &[&str] = &["sysroot", "test-harness"];

/// Legacy files that should be deleted.
const LEGACY_FILES: &[&str] = &["info.toml", "grammar-crate-config.toml"];

/// Minimum recommended lines for a sample file.
pub const MIN_SAMPLE_LINES: usize = 25;

impl CrateRegistry {
    /// Load the registry by scanning all arborium-* crates.
    ///
    /// This scans both the configuration (arborium.kdl) and the actual
    /// files on disk, building a complete picture of each crate's state.
    pub fn load(crates_dir: &Utf8Path) -> Result<Self, Report> {
        let mut crates = BTreeMap::new();

        for entry in fs::read_dir(crates_dir)? {
            let entry = entry?;
            let path = entry.path();

            if !path.is_dir() {
                continue;
            }

            let dir_name = path.file_name().unwrap().to_string_lossy().to_string();
            if !dir_name.starts_with("arborium-") {
                continue;
            }

            // Skip utility crates
            let crate_suffix = dir_name.strip_prefix("arborium-").unwrap();
            if SKIP_CRATES.contains(&crate_suffix) {
                continue;
            }

            let crate_path = Utf8PathBuf::from_path_buf(path).expect("non-UTF8 path");
            let crate_name = dir_name;

            let state = Self::scan_crate(&crate_name, &crate_path)?;
            crates.insert(crate_name, state);
        }

        Ok(Self { crates })
    }

    /// Scan a single crate directory.
    fn scan_crate(name: &str, path: &Utf8Path) -> Result<CrateState, Report> {
        let mut files = CrateFiles::default();

        // Check for arborium.kdl
        let kdl_path = path.join("arborium.kdl");
        let (config, kdl_source) = if kdl_path.exists() {
            let content = fs::read_to_string(&kdl_path)?;
            let config: CrateConfig = match facet_kdl::from_str(&content) {
                Ok(c) => c,
                Err(e) => {
                    // Print detailed error info
                    eprintln!("Error parsing {}:", kdl_path);
                    eprintln!("  Kind: {:?}", e.kind());

                    // Use miette to display the error with source context
                    let report = miette::Report::new(e)
                        .with_source_code(NamedSource::new(kdl_path.as_str(), content.clone()));
                    eprintln!("{:?}", report);
                    return Err(
                        std::io::Error::other(format!("Failed to parse {}", kdl_path)).into(),
                    );
                }
            };
            files.kdl = FileState::Present {
                content: content.clone(),
            };
            (Some(config), Some(content))
        } else {
            (None, None)
        };

        // Check for generated files
        files.cargo_toml = Self::read_file_state(&path.join("Cargo.toml"));
        files.build_rs = Self::read_file_state(&path.join("build.rs"));
        files.lib_rs = Self::read_file_state(&path.join("src/lib.rs"));

        // Check grammar/src/ for generated files
        let grammar_src_path = path.join("grammar/src");
        if grammar_src_path.exists() {
            files.grammar_src.parser_c = Self::read_file_state(&grammar_src_path.join("parser.c"));
        }
        // Check grammar/ for scanner.c (handwritten, not in src/)
        let grammar_path = path.join("grammar");
        if grammar_path.exists() {
            files.grammar_src.scanner_c = Self::read_file_state(&grammar_path.join("scanner.c"));
        }

        // Check queries/
        let queries_path = path.join("queries");
        if queries_path.exists() {
            files.queries.highlights = Self::read_file_state(&queries_path.join("highlights.scm"));
            files.queries.injections = Self::read_file_state(&queries_path.join("injections.scm"));
            files.queries.locals = Self::read_file_state(&queries_path.join("locals.scm"));
        }

        // Check for samples declared in config
        if let Some(ref cfg) = config {
            for grammar in &cfg.grammars {
                for sample in &grammar.samples {
                    let sample_path = path.join(&*sample.path);
                    let state = Self::check_sample_file(&sample_path);
                    files.samples.push(SampleState {
                        path: sample.path.to_string(),
                        state,
                    });
                }
            }
        }

        // Check for legacy files
        for legacy in LEGACY_FILES {
            let legacy_path = path.join(legacy);
            if legacy_path.exists() {
                files.legacy_files.push(legacy_path);
            }
        }

        Ok(CrateState {
            name: name.to_string(),
            path: path.to_owned(),
            config,
            kdl_source,
            files,
        })
    }

    /// Read a file's state.
    fn read_file_state(path: &Utf8Path) -> FileState {
        match fs::read_to_string(path) {
            Ok(content) => FileState::Present { content },
            Err(_) => FileState::Missing,
        }
    }

    /// Check a sample file's state.
    fn check_sample_file(path: &Utf8Path) -> SampleFileState {
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return SampleFileState::Missing,
        };

        let trimmed = content.trim();

        if trimmed.is_empty() {
            return SampleFileState::Empty;
        }

        // Check for HTTP error pages (failed downloads)
        // Note: Don't flag <!DOCTYPE as error - that's valid for HTML samples
        if trimmed.starts_with("404:") || trimmed == "Not Found" || trimmed == "404 Not Found" {
            return SampleFileState::HttpError;
        }

        let lines = content.lines().count();
        if lines < MIN_SAMPLE_LINES {
            return SampleFileState::TooShort { lines };
        }

        SampleFileState::Ok { lines }
    }

    /// Iterate over all crates.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &CrateState)> {
        self.crates.iter()
    }

    /// Iterate over all crates that have valid configuration.
    pub fn configured_crates(&self) -> impl Iterator<Item = (&String, &CrateState, &CrateConfig)> {
        self.crates
            .iter()
            .filter_map(|(name, state)| state.config.as_ref().map(|cfg| (name, state, cfg)))
    }

    /// Iterate over all grammars across all configured crates.
    pub fn all_grammars(&self) -> impl Iterator<Item = (&str, &CrateConfig, &GrammarConfig)> {
        self.configured_crates()
            .flat_map(|(crate_name, _, config)| {
                config
                    .grammars
                    .iter()
                    .map(move |g| (crate_name.as_str(), config, g))
            })
    }
}
