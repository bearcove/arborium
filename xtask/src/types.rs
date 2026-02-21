//! Core types for the arborium xtask system.
//!
//! This module defines the data structures used throughout xtask, primarily
//! for representing grammar/language metadata stored in `arborium.yaml` files.
//!
//! # File Format
//!
//! Each language definition in `langs/group-*/*/def/` contains an `arborium.yaml` file that
//! describes one or more language grammars. This is the single source of truth for:
//!
//! - Upstream repository and commit information (crate-level)
//! - Language metadata (name, icon, description, etc.) per grammar
//! - Sample files for testing and demos
//! - Build configuration for special cases
//!
//! The new structure organizes languages into thematic groups:
//!
//! ```text
//! langs/
//! ├── group-birch/              # Systems languages
//! │   ├── rust/
//! │   │   ├── def/              # Source of truth (committed)
//! │   │   │   ├── arborium.yaml
//! │   │   │   ├── grammar/
//! │   │   │   ├── queries/
//! │   │   │   └── samples/
//! │   │   ├── crate/            # Generated Rust crate
//! │   │   └── npm/              # Generated WASM package
//! │   ├── c/
//! │   └── cpp/
//! ├── group-acorn/              # Web languages
//! │   ├── javascript/
//! │   ├── html/
//! │   └── css/
//! └── ...
//! ```
//!
//! # Example `arborium.yaml` (single grammar, most common)
//!
//! ```yaml
//! repo: https://github.com/tree-sitter/tree-sitter-rust
//! commit: 261b20226c04ef601adbdf185a800512a5f66291
//! license: MIT
//! authors: Maxim Sokolov
//!
//! grammars:
//!   - id: rust
//!     name: Rust
//!     tag: code
//!     tier: 1
//!     icon: devicon-plain:rust
//!     aliases:
//!       - rs
//!     has_scanner: true
//!     c_symbol: rust_orchard
//!
//!     inventor: Graydon Hoare
//!     year: 2010
//!     description: Systems language focused on safety and performance without GC
//!     link: https://en.wikipedia.org/wiki/Rust_(programming_language)
//!     trivia: Hoare began Rust as a side project at Mozilla in 2006
//!
//!     samples:
//!       - path: samples/example.rs
//!         description: Clippy lint implementation
//!         link: https://github.com/rust-lang/rust/blob/main/...
//!         license: MIT OR Apache-2.0
//! ```
//!
//! # Example `arborium.yaml` (multi-grammar crate)
//!
//! ```yaml
//! repo: https://github.com/tree-sitter-grammars/tree-sitter-xml
//! commit: 863dbc381f44f6c136a399e684383b977bb2beaa
//! license: MIT
//! authors: ObserverOfTime
//!
//! grammars:
//!   - id: xml
//!     name: XML
//!     tag: markup
//!     tier: 3
//!     has_scanner: true
//!     grammar_path: xml
//!     # ...metadata, samples...
//!
//!   - id: dtd
//!     name: DTD
//!     tag: markup
//!     tier: 3
//!     has_scanner: true
//!     grammar_path: dtd
//!     # ...metadata, samples...
//! ```

#![allow(dead_code)]

use std::collections::BTreeMap;

use camino::{Utf8Path, Utf8PathBuf};
use facet::Facet;
use fs_err as fs;
pub use rootcause::Report;

// =============================================================================
// Crate-level configuration (parsed from arborium.yaml)
// =============================================================================

/// Configuration for an entire arborium-* crate.
///
/// This represents the contents of an `arborium.yaml` file. A crate can
/// contain one or more grammars that share the same upstream source.
#[derive(Debug, Clone, Facet)]
pub struct CrateConfig {
    /// Git repository URL for the upstream tree-sitter grammar.
    ///
    /// Use "local" for grammars that are maintained in this repository.
    pub repo: String,

    /// Git commit hash of the vendored version.
    pub commit: String,

    /// SPDX license identifier for the grammar (e.g., "MIT", "Apache-2.0").
    pub license: String,

    /// Authors of the tree-sitter grammar.
    #[facet(default)]
    pub authors: Option<String>,

    /// One or more grammars exported by this crate.
    pub grammars: Vec<GrammarConfig>,
}

// =============================================================================
// Per-grammar configuration
// =============================================================================

/// Configuration for a single grammar within a crate.
///
/// This contains all the metadata and build configuration for one language.
#[derive(Debug, Clone, Facet)]
pub struct GrammarConfig {
    // =========================================================================
    // Identity
    // =========================================================================
    /// Unique identifier for this grammar, used in feature flags and exports.
    pub id: String,

    /// Human-readable display name for the language.
    pub name: String,

    /// Category tag for grouping languages in the UI.
    pub tag: String,

    /// Quality/completeness tier (1 = best, 5 = experimental).
    #[facet(default)]
    pub tier: Option<u8>,

    /// Iconify icon identifier.
    #[facet(default)]
    pub icon: Option<String>,

    /// Alternative names or file extensions for this language.
    #[facet(default)]
    pub aliases: Option<Vec<String>>,

    // =========================================================================
    // Build Configuration
    // =========================================================================
    /// Internal grammar (used by other grammars via injection, not user-facing).
    #[facet(default)]
    pub internal: Option<bool>,

    /// Tests are cursed (skip test generation due to platform issues).
    #[facet(default)]
    pub tests_cursed: Option<bool>,

    /// Generate a WASM plugin for this grammar.
    #[facet(default)]
    pub generate_plugin: Option<bool>,

    /// Whether this grammar has a scanner.c file.
    #[facet(default)]
    pub has_scanner: Option<bool>,

    /// Path to the grammar within the repo (for multi-grammar repos).
    #[facet(default)]
    pub grammar_path: Option<String>,

    /// Override the C symbol name.
    #[facet(default)]
    pub c_symbol: Option<String>,

    /// Query configuration (highlights inheritance).
    #[facet(default)]
    pub queries: Option<QueriesConfig>,

    /// Cross-grammar dependencies for tree-sitter generation.
    #[facet(default)]
    pub dependencies: Option<Vec<Dependency>>,

    /// Languages that can be injected into this grammar (e.g., JS/CSS in HTML).
    /// These become optional dependencies with an "injections" feature.
    #[facet(default)]
    pub injections: Option<Vec<String>>,

    // =========================================================================
    // Language Metadata (for demos and documentation)
    // =========================================================================
    /// Creator(s) of the programming language.
    #[facet(default)]
    pub inventor: Option<String>,

    /// Year the language was first released.
    #[facet(default)]
    pub year: Option<u16>,

    /// Brief description of the language.
    #[facet(default)]
    pub description: Option<String>,

    /// URL to more information.
    #[facet(default)]
    pub link: Option<String>,

    /// Fun facts or interesting history.
    #[facet(default)]
    pub trivia: Option<String>,

    // =========================================================================
    // Samples
    // =========================================================================
    /// Sample files for testing highlighting and displaying in demos.
    #[facet(default)]
    pub samples: Option<Vec<SampleConfig>>,
}

impl GrammarConfig {
    /// Get the grammar ID as a string.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Whether this is an internal grammar (used via injection, not user-facing).
    pub fn is_internal(&self) -> bool {
        self.internal.unwrap_or(false)
    }

    /// Whether this grammar has a scanner.
    pub fn has_scanner(&self) -> bool {
        self.has_scanner.unwrap_or(false)
    }

    /// Whether tests are cursed (skip test generation).
    pub fn tests_cursed(&self) -> bool {
        self.tests_cursed.unwrap_or(false)
    }

    /// Whether to generate a WASM plugin for this grammar.
    /// Defaults to true.
    pub fn generate_plugin(&self) -> bool {
        self.generate_plugin.unwrap_or(true)
    }
}

/// Cross-grammar dependency for tree-sitter generation.
#[derive(Debug, Clone, Facet)]
pub struct Dependency {
    /// NPM package name.
    pub npm: String,

    /// Arborium crate name.
    #[facet(rename = "crate")]
    pub krate: String,
}

/// Query configuration for a grammar.
#[derive(Debug, Clone, Facet)]
pub struct QueriesConfig {
    /// Highlights query configuration.
    #[facet(default)]
    pub highlights: Option<HighlightsConfig>,
}

/// Highlights query configuration.
#[derive(Debug, Clone, Facet)]
pub struct HighlightsConfig {
    /// Queries to prepend from other grammars.
    #[facet(default)]
    pub prepend: Option<Vec<PrependConfig>>,
}

/// A reference to another grammar's queries to prepend.
#[derive(Debug, Clone, Facet)]
pub struct PrependConfig {
    /// The crate to prepend from (e.g., "arborium-javascript").
    #[facet(rename = "crate")]
    pub crate_name: String,

    /// The grammar within that crate (optional if crate has only one grammar).
    #[facet(default)]
    pub grammar: Option<String>,
}

/// Metadata for a sample source file.
#[derive(Debug, Clone, Facet)]
pub struct SampleConfig {
    /// Path to the sample file, relative to the crate root.
    pub path: String,

    /// Brief description of what the sample demonstrates.
    #[facet(default)]
    pub description: Option<String>,

    /// URL to the original source of this sample (for attribution).
    #[facet(default)]
    pub link: Option<String>,

    /// License of the sample file (may differ from the grammar license).
    #[facet(default)]
    pub license: Option<String>,
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
#[derive(Debug, Clone)]
pub struct CrateState {
    /// The crate name (e.g., "arborium-rust").
    pub name: String,

    /// Path to the crate directory (for backward compatibility).
    /// In new structure, this points to def/. Use def_path and crate_path instead.
    pub path: Utf8PathBuf,

    /// Path to the def/ directory containing source files (arborium.yaml, grammar/, etc.).
    pub def_path: Utf8PathBuf,

    /// Path to the crate/ directory for generated files (Cargo.toml, build.rs, src/).
    pub crate_path: Utf8PathBuf,

    /// Parsed configuration from arborium.yaml (if present).
    pub config: Option<CrateConfig>,

    /// Raw YAML source for error diagnostics.
    pub yaml_source: Option<String>,

    /// State of files on disk.
    pub files: CrateFiles,
}

/// State of a single file.
#[derive(Debug, Default, Clone)]
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
    #[strikethrough[derive(Debug, Default, Clone)]]
    pub struct CrateFiles {
        /// arborium.yaml - the source of truth
        pub yaml: FileState,

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

        /// Sample files declared in yaml
        pub samples: Vec<SampleState>,

        /// Legacy/unexpected files that should be deleted
        pub legacy_files: Vec<Utf8PathBuf>,
    }
}

/// State of a sample file.
#[derive(Debug, Clone)]
pub struct SampleState {
    /// Path relative to crate root (from yaml).
    pub path: String,

    /// What we found on disk.
    pub state: SampleFileState,
}

/// State of a sample file on disk.
#[derive(Debug, Clone)]
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
const LEGACY_FILES: &[&str] = &["info.toml", "grammar-crate-config.toml", "arborium.kdl"];

/// Minimum recommended lines for a sample file.
pub const MIN_SAMPLE_LINES: usize = 25;

impl CrateRegistry {
    /// Load the registry by scanning language definitions.
    ///
    /// This scans both the new structure (langs/group-*/*/def/) and legacy
    /// structure (crates/arborium-*) for language definitions, building a
    /// complete picture of each crate's state.
    pub fn load(crates_dir: &Utf8Path) -> Result<Self, Report> {
        let mut crates = BTreeMap::new();

        // Try to find repo root to look for langs/ directory
        let repo_root = crates_dir.parent().expect("crates_dir should have parent");
        let langs_dir = repo_root.join("langs");

        // Scan new structure: langs/group-*/*/def/
        if langs_dir.exists() {
            for group_entry in fs::read_dir(&langs_dir)? {
                let group_entry = group_entry?;
                let group_path = group_entry.path();

                if !group_path.is_dir() {
                    continue;
                }

                let group_name = group_path.file_name().unwrap().to_string_lossy();
                if !group_name.starts_with("group-") {
                    continue;
                }

                // Scan languages in this group
                for lang_entry in fs::read_dir(&group_path)? {
                    let lang_entry = lang_entry?;
                    let lang_path = lang_entry.path();

                    if !lang_path.is_dir() {
                        continue;
                    }

                    let lang_name = lang_path.file_name().unwrap().to_string_lossy().to_string();

                    // Skip utility crates
                    if SKIP_CRATES.contains(&lang_name.as_str()) {
                        continue;
                    }

                    let def_path = lang_path.join("def");
                    if !def_path.exists() {
                        continue;
                    }

                    let def_path = Utf8PathBuf::from_path_buf(def_path).expect("non-UTF8 path");
                    let crate_name = format!("arborium-{}", lang_name);

                    // Calculate crate path: langs/group-*/lang/crate/
                    let crate_path = lang_path.join("crate");
                    let crate_path = Utf8PathBuf::from_path_buf(crate_path).expect("non-UTF8 path");

                    let state =
                        Self::scan_crate_new_structure(&crate_name, &def_path, &crate_path)?;
                    crates.insert(crate_name, state);
                }
            }
        }

        // Scan legacy structure: crates/arborium-* (for compatibility during migration)
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

            // Skip if we already loaded this from new structure
            if crates.contains_key(&dir_name) {
                continue;
            }

            let crate_path = Utf8PathBuf::from_path_buf(path).expect("non-UTF8 path");
            let crate_name = dir_name;

            let state = Self::scan_crate_legacy(&crate_name, &crate_path)?;
            crates.insert(crate_name, state);
        }

        Ok(Self { crates })
    }

    /// Find a grammar by ID, returning its crate state and grammar config.
    pub fn find_grammar(&self, grammar_id: &str) -> Option<(&CrateState, &GrammarConfig)> {
        self.crates.values().find_map(|state| {
            let config = state.config.as_ref()?;
            config
                .grammars
                .iter()
                .find(|grammar| grammar.id() == grammar_id)
                .map(|grammar| (state, grammar))
        })
    }

    /// Scan a single crate directory.
    fn scan_crate_new_structure(
        name: &str,
        def_path: &Utf8Path,
        crate_path: &Utf8Path,
    ) -> Result<CrateState, Report> {
        let mut files = CrateFiles::default();

        // Check for arborium.yaml in def/
        let yaml_path = def_path.join("arborium.yaml");
        let (config, yaml_source) = if yaml_path.exists() {
            let content = fs::read_to_string(&yaml_path)?;
            let config: CrateConfig = match facet_yaml::from_str(&content) {
                Ok(c) => c,
                Err(e) => {
                    // Print detailed error info
                    eprintln!("Error parsing {}:", yaml_path);
                    eprintln!("  Details: {:?}", e);
                    return Err(
                        std::io::Error::other(format!("Failed to parse {}", yaml_path)).into(),
                    );
                }
            };
            files.yaml = FileState::Present {
                content: content.clone(),
            };
            (Some(config), Some(content))
        } else {
            (None, None)
        };

        // Check for generated files in crate/
        files.cargo_toml = Self::read_file_state(&crate_path.join("Cargo.toml"));
        files.build_rs = Self::read_file_state(&crate_path.join("build.rs"));
        files.lib_rs = Self::read_file_state(&crate_path.join("src/lib.rs"));

        // Check grammar/src/ for generated files in def/ (tree-sitter generate output)
        let grammar_src_path = def_path.join("grammar/src");
        if grammar_src_path.exists() {
            files.grammar_src.parser_c = Self::read_file_state(&grammar_src_path.join("parser.c"));
        }
        // Check grammar/ for scanner.c (handwritten, not in src/) in def/
        let grammar_path = def_path.join("grammar");
        if grammar_path.exists() {
            files.grammar_src.scanner_c = Self::read_file_state(&grammar_path.join("scanner.c"));
        }

        // Check queries/ in def/
        let queries_path = def_path.join("queries");
        if queries_path.exists() {
            files.queries.highlights = Self::read_file_state(&queries_path.join("highlights.scm"));
            files.queries.injections = Self::read_file_state(&queries_path.join("injections.scm"));
            files.queries.locals = Self::read_file_state(&queries_path.join("locals.scm"));
        }

        // Check for samples declared in config (in def/)
        if let Some(ref cfg) = config {
            for grammar in &cfg.grammars {
                if let Some(samples) = &grammar.samples {
                    for sample in samples {
                        let sample_path = def_path.join(&sample.path);
                        let state = Self::check_sample_file(&sample_path);
                        files.samples.push(SampleState {
                            path: sample.path.clone(),
                            state,
                        });
                    }
                }
            }
        }

        // Check for legacy files in def/
        for legacy in LEGACY_FILES {
            let legacy_path = def_path.join(legacy);
            if legacy_path.exists() {
                files.legacy_files.push(legacy_path);
            }
        }

        Ok(CrateState {
            name: name.to_string(),
            path: def_path.to_owned(), // For backward compatibility
            def_path: def_path.to_owned(),
            crate_path: crate_path.to_owned(),
            config,
            yaml_source,
            files,
        })
    }

    fn scan_crate_legacy(name: &str, path: &Utf8Path) -> Result<CrateState, Report> {
        let mut files = CrateFiles::default();

        // Check for arborium.yaml
        let yaml_path = path.join("arborium.yaml");
        let (config, yaml_source) = if yaml_path.exists() {
            let content = fs::read_to_string(&yaml_path)?;
            let config: CrateConfig = match facet_yaml::from_str(&content) {
                Ok(c) => c,
                Err(e) => {
                    // Print detailed error info
                    eprintln!("Error parsing {}:", yaml_path);
                    eprintln!("  Details: {:?}", e);
                    return Err(
                        std::io::Error::other(format!("Failed to parse {}", yaml_path)).into(),
                    );
                }
            };
            files.yaml = FileState::Present {
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
                if let Some(samples) = &grammar.samples {
                    for sample in samples {
                        let sample_path = path.join(&sample.path);
                        let state = Self::check_sample_file(&sample_path);
                        files.samples.push(SampleState {
                            path: sample.path.clone(),
                            state,
                        });
                    }
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
            def_path: path.to_owned(), // In legacy structure, def and crate are the same
            crate_path: path.to_owned(),
            config,
            yaml_source,
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
    pub fn all_grammars(
        &self,
    ) -> impl Iterator<Item = (&CrateState, &CrateConfig, &GrammarConfig)> {
        self.configured_crates()
            .flat_map(|(_, state, config)| config.grammars.iter().map(move |g| (state, config, g)))
    }
}

// =============================================================================
// Compression Configuration (compression.yaml)
// =============================================================================

structstruck::strike! {
    /// Compression settings for WASM plugin builds.
    #[strikethrough[derive(Debug, Clone, Default, facet::Facet)]]
    pub struct CompressionConfig {
        #[facet(default)]
        pub brotli: Option<pub struct BrotliConfig {
            /// Quality level: 0-11 (11 = best compression, slowest)
            pub quality: u32,

            /// Window size: 10-24 (larger = better compression, more memory)
            pub window: u32,
        }>,

        #[facet(default)]
        pub gzip: Option<pub struct GzipConfig {
            /// Backend: "flate2" (fast) or "zopfli" (best compression, slow)
            #[facet(default)]
            pub backend: Option<String>,

            /// Compression level for flate2: 0-9 (9 = best compression, slowest)
            #[facet(default)]
            pub level: Option<u32>,

            /// Number of iterations for zopfli (15 = default, higher = better but slower)
            #[facet(default)]
            pub iterations: Option<u8>,
        }>,

        #[facet(default)]
        pub zstd: Option<pub struct ZstdConfig {
            /// Compression level: 1-22 (19 is a good balance, 22 = max)
            pub level: i32,
        }>,
    }
}

impl CompressionConfig {
    /// Load compression config from the repo root.
    pub fn load(repo_root: &camino::Utf8Path) -> Result<Self, rootcause::Report> {
        let config_path = repo_root.join("compression.yaml");
        if !config_path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(&config_path)?;
        let config: CompressionConfig = facet_yaml::from_str(&content)?;
        Ok(config)
    }

    /// Get brotli quality (default: 11)
    pub fn brotli_quality(&self) -> u32 {
        self.brotli.as_ref().map(|b| b.quality).unwrap_or(11)
    }

    /// Get brotli window size (default: 22)
    pub fn brotli_window(&self) -> u32 {
        self.brotli.as_ref().map(|b| b.window).unwrap_or(22)
    }

    /// Check if using zopfli backend for gzip (default: false/flate2)
    pub fn gzip_use_zopfli(&self) -> bool {
        self.gzip
            .as_ref()
            .and_then(|g| g.backend.as_ref())
            .map(|b| b == "zopfli")
            .unwrap_or(false)
    }

    /// Get gzip level for flate2 (default: 9)
    pub fn gzip_level(&self) -> u32 {
        self.gzip
            .as_ref()
            .and_then(|g| g.level)
            .unwrap_or(9)
    }

    /// Get zopfli iterations (default: 15)
    pub fn gzip_iterations(&self) -> u8 {
        self.gzip
            .as_ref()
            .and_then(|g| g.iterations)
            .unwrap_or(15)
    }

    /// Get zstd level (default: 19)
    pub fn zstd_level(&self) -> i32 {
        self.zstd.as_ref().map(|z| z.level).unwrap_or(19)
    }
}
