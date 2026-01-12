//! Serve command - builds and serves the WASM demo.
//!
//! This module generates registry.json from arborium.yaml files and serves
//! the demo with all grammar metadata and inlined sample content.

use crate::highlight_gen;
use crate::theme_gen::{self, Theme};
use crate::types::{CrateConfig, CrateRegistry, GrammarConfig, SampleConfig};
use crate::util;
use camino::{Utf8Path, Utf8PathBuf};
use facet::Facet;
use owo_colors::OwoColorize;
use sailfish::TemplateSimple;

use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

// Sailfish template for app.js
#[derive(TemplateSimple)]
#[template(path = "app.stpl.js")]
struct AppJsTemplate<'a> {
    language_info: &'a str,
    examples: &'a str,
    icons: &'a str,
    theme_info: &'a str,
}

// Sailfish template for index.html
#[derive(TemplateSimple)]
#[template(path = "index.stpl.html")]
struct IndexHtmlTemplate<'a> {
    icons: &'a BTreeMap<String, String>,
    theme_swatches: &'a str,
    theme_css_link: &'a str,
    code: &'a CodeBlocks,
    language_count: usize,
    default_theme: &'a str,
}

/// A code block with its language for client-side highlighting
struct CodeBlock {
    lang: &'static str,
    source: &'static str,
}

/// Code blocks for the index page (highlighted client-side by IIFE)
struct CodeBlocks {
    script_tag: CodeBlock,
    code_block_examples: CodeBlock,
    data_attributes: CodeBlock,
    cargo_toml: CodeBlock,
    rust_highlight: CodeBlock,
    js_esm: CodeBlock,
    docsrs_script: CodeBlock,
    docsrs_cargo: CodeBlock,
    rustdoc_postprocess: CodeBlock,
    html_example_traditional: CodeBlock,
    html_example_arborium: CodeBlock,
}

// Sailfish template for iife-demo.html
#[derive(TemplateSimple)]
#[template(path = "iife-demo.stpl.html")]
struct IifeDemoHtmlTemplate<'a> {
    title_suffix: &'a str,
    theme_css_link: &'a str,
    description: &'a str,
    footer_suffix: &'a str,
    script_tag: &'a str,
}

// Sailfish template for rustdoc-comparison.html
#[derive(TemplateSimple)]
#[template(path = "rustdoc-comparison.stpl.html")]
struct RustdocComparisonHtmlTemplate;

// Sailfish template for arborium-theme README
#[derive(TemplateSimple)]
#[template(path = "arborium_theme_readme.stpl.md")]
struct ArboriumThemeReadmeTemplate<'a> {
    theme_count: usize,
    themes: &'a [ThemeInfo],
}

/// Theme info for README generation
struct ThemeInfo {
    name: String,
    variant: &'static str,
    source_url: Option<String>,
    source_display: String,
}

// =============================================================================
// Registry JSON types (for demo consumption)
// =============================================================================

structstruck::strike! {
    /// The registry.json format consumed by the demo frontend.
    #[strikethrough[derive(Debug, Clone, Facet)]]
    pub struct Registry {
        /// All grammars in the registry, sorted by id.
        pub grammars: Vec<pub struct RegistryGrammar {
            /// Unique identifier (e.g., "rust", "javascript").
            pub id: String,

            /// Crate name (e.g., "arborium-rust").
            #[facet(rename = "crate")]
            pub crate_name: String,

            /// Human-readable name (e.g., "Rust", "JavaScript").
            pub name: String,

            /// Iconify icon identifier (e.g., "devicon-plain:rust").
            pub icon: Option<String>,

            /// Quality tier (1 = best, 5 = experimental).
            pub tier: Option<u8>,

            /// Category tag (e.g., "code", "markup", "data").
            pub tag: String,

            /// Brief description of the language.
            pub description: Option<String>,

            /// Creator(s) of the language.
            pub inventor: Option<String>,

            /// Year the language was first released.
            pub year: Option<u16>,

            /// URL to more information.
            pub link: Option<String>,

            /// Fun facts or interesting history.
            pub trivia: Option<String>,

            /// File extension aliases (e.g., ["rs"] for Rust).
            pub aliases: Vec<String>,

            /// URL to the upstream tree-sitter grammar repository.
            pub grammar_repo: Option<String>,

            /// SPDX license identifier for the grammar.
            pub grammar_license: Option<String>,

            /// Sample files (content served separately via /samples/{id}.txt).
            pub samples: Vec<pub struct RegistrySample {
                /// Path relative to def directory (e.g., "samples/example.rs").
                pub path: String,

                /// Description of what this sample demonstrates.
                pub description: Option<String>,
            }>,

            /// Path to def/ directory (internal use only, not serialized).
            #[facet(skip)]
            pub def_path: String,
        }>,
    }
}

impl Registry {
    /// Build the registry from a CrateRegistry, using def_path for samples.
    pub fn from_crate_registry(registry: &CrateRegistry, _crates_dir: &Utf8Path) -> Self {
        let mut grammars: Vec<RegistryGrammar> = registry
            .all_grammars()
            .filter(|(_, _, grammar)| !grammar.is_internal())
            .map(|(state, config, grammar)| {
                // Use def_path where samples live (langs/group-*/*/def/)
                RegistryGrammar::from_grammar_config(&state.name, config, grammar, &state.def_path)
            })
            .collect();

        // Sort by id for consistent output
        grammars.sort_by(|a, b| a.id.cmp(&b.id));

        Self { grammars }
    }

    /// Serialize to pretty JSON using facet-json.
    pub fn to_json_pretty(&self) -> String {
        facet_json::to_string_pretty(self).expect("registry serialization failed")
    }
}

impl RegistryGrammar {
    /// Build from a GrammarConfig, reading sample content from disk.
    fn from_grammar_config(
        crate_name: &str,
        config: &CrateConfig,
        grammar: &GrammarConfig,
        def_path: &Utf8Path,
    ) -> Self {
        let samples: Vec<RegistrySample> = grammar
            .samples
            .as_ref()
            .map(|s| s.iter())
            .into_iter()
            .flatten()
            .filter_map(|sample| RegistrySample::from_sample_config(sample, def_path))
            .collect();

        // Extract repo URL (skip "local" which means maintained in this repo)
        let grammar_repo = {
            let repo = config.repo.as_str();
            if repo == "local" {
                None
            } else {
                Some(repo.to_string())
            }
        };

        Self {
            id: grammar.id.to_string(),
            crate_name: crate_name.to_string(),
            name: grammar.name.to_string(),
            icon: grammar.icon.clone(),
            tier: grammar.tier,
            tag: grammar.tag.to_string(),
            description: grammar.description.clone(),
            inventor: grammar.inventor.clone(),
            year: grammar.year,
            link: grammar.link.clone(),
            trivia: grammar.trivia.clone(),
            aliases: grammar
                .aliases
                .clone()
                .unwrap_or_default(),
            grammar_repo,
            grammar_license: Some(config.license.to_string()),
            samples,
            def_path: def_path.to_string(),
        }
    }
}

impl RegistrySample {
    /// Build from a SampleConfig (content is served separately).
    fn from_sample_config(sample: &SampleConfig, crate_path: &Utf8Path) -> Option<Self> {
        // Check the file exists
        let sample_path = crate_path.join(&sample.path);
        if !sample_path.exists() {
            return None;
        }

        Some(Self {
            path: sample.path.to_string(),
            description: sample.description.clone(),
        })
    }
}

// =============================================================================
// Serve command implementation
// =============================================================================

/// Build and serve the WASM demo.
pub fn serve(crates_dir: &Utf8Path, addr: &str, port: Option<u16>, dev: bool) {
    let repo_root = util::find_repo_root().expect("Could not find repo root");
    let demo_dir = repo_root.join("demo");

    println!(
        "{} {}",
        "==>".cyan().bold(),
        "Building arborium demo".bold()
    );
    if dev {
        println!("    {}", "(dev mode - skipping optimizations)".dimmed());
    }
    println!();

    // Step 0: Generate Cargo.toml files from templates for shared crates
    step("Generating shared crate manifests", || {
        generate_shared_crate_manifests(&repo_root)
    });

    // Step 0b: Generate theme Rust code (builtin_generated.rs)
    step("Generating theme Rust code", || {
        theme_gen::generate_theme_code(crates_dir)
    });

    // Step 1: Generate registry.json and get the registry for later use
    let registry = step_with_result("Generating registry.json", || {
        generate_registry_json(crates_dir, &demo_dir)
    });

    // Step 2: Generate individual sample files
    step("Generating sample files", || {
        generate_sample_files(crates_dir, &registry, &demo_dir)
    });

    // Step 2: Fetch icons (from registry + template)
    let icons = step_with_result("Fetching icons", || {
        fetch_icons_from_registry(&registry, &demo_dir)
    });

    // Step 3: Generate theme CSS from arborium themes
    step("Generating theme CSS", || {
        generate_theme_css(crates_dir, &demo_dir)
    });

    // Step 4: Generate index.html from template
    step("Generating index.html", || {
        generate_index_html(crates_dir, &demo_dir, &icons, &registry)
    });

    // Step 4b: Generate plugins-manifest.ts for IIFE bundle
    step("Generating plugins manifest", || {
        crate::build::generate_plugins_manifest(
            Utf8Path::new(repo_root.to_str().unwrap()),
            crates_dir,
        )
        .map_err(|e| e.to_string())
    });

    // Step 4c: Build IIFE bundle from packages/arborium
    step("Building IIFE bundle", || {
        build_iife_bundle(&repo_root, &demo_dir)
    });

    // Step 4c: Generate IIFE demo HTML files
    step("Generating IIFE demo HTML", || {
        generate_iife_demo_html(&demo_dir)
    });

    // Step 5: Generate app.generated.js
    step("Generating app.generated.js", || {
        generate_app_js(crates_dir, &demo_dir, &registry, &icons)
    });

    // Step 5b: Generate rustdoc comparison (optional - requires all crates to be buildable)
    optional_step("Generating rustdoc comparison", || {
        generate_rustdoc_comparison(&repo_root, &demo_dir)
    });

    // Step 6: Copy plugins.json with dev_mode flag
    step("Copying plugins.json", || {
        copy_plugins_json(crates_dir, &demo_dir, dev)
    });

    // Step 6: Pre-compress files
    if dev {
        step("Pre-compressing files (fast)", || {
            precompress_files_fast(&demo_dir)
        });
    } else {
        step("Pre-compressing files", || precompress_files(&demo_dir));
    }

    // Step 6: Start HTTP server
    println!(
        "\n{} {}",
        "==>".cyan().bold(),
        "Starting HTTP server".bold()
    );

    let (server, actual_port) = bind_server(addr, port);
    let url = format!("http://{}:{}", addr, actual_port);

    println!();
    println!(
        "  {} {}",
        "✓".green().bold(),
        "Demo server ready!".green().bold()
    );
    println!();
    println!("    {} {}", "→".cyan(), url.cyan().bold().underline());
    println!();
    println!("    {}", "Press Ctrl+C to stop".dimmed());
    println!();

    // Serve files
    serve_files(server, &demo_dir);
}

#[allow(dead_code)]
/// Build the demo assets without starting the server (for static site generation).
pub fn build_static_site(crates_dir: &Utf8Path, dev: bool) -> Result<(), String> {
    let repo_root = util::find_repo_root().ok_or("Could not find repo root")?;
    let demo_dir = repo_root.join("demo");

    println!(
        "{} {}",
        "==>".cyan().bold(),
        "Building arborium demo (static)".bold()
    );
    if dev {
        println!("    {}", "(dev mode - skipping optimizations)".dimmed());
    }
    println!();

    let registry = step_with_result("Generating registry.json", || {
        generate_registry_json(crates_dir, &demo_dir)
    });

    step("Generating sample files", || {
        generate_sample_files(crates_dir, &registry, &demo_dir)
    });

    let icons = step_with_result("Fetching icons", || {
        fetch_icons_from_registry(&registry, &demo_dir)
    });

    step("Generating theme CSS", || {
        generate_theme_css(crates_dir, &demo_dir)
    });
    step("Generating index.html", || {
        generate_index_html(crates_dir, &demo_dir, &icons, &registry)
    });
    step("Generating IIFE demo HTML", || {
        generate_iife_demo_html(&demo_dir)
    });
    step("Generating app.generated.js", || {
        generate_app_js(crates_dir, &demo_dir, &registry, &icons)
    });
    step("Generating rustdoc comparison", || {
        generate_rustdoc_comparison(&repo_root, &demo_dir)
    });

    if dev {
        step("Pre-compressing files (fast)", || {
            precompress_files_fast(&demo_dir)
        });
    } else {
        step("Pre-compressing files", || precompress_files(&demo_dir));
    }

    println!(
        "  {} Static demo built at {}",
        "✓".green(),
        demo_dir.join("pkg").display()
    );

    Ok(())
}

/// Generate registry.json and all demo assets (called from build --demo).
pub fn generate_registry_and_assets(
    crates_dir: &Utf8Path,
    demo_dir: &Utf8Path,
    dev: bool,
) -> Result<(), String> {
    let demo_dir_path: &Path = demo_dir.as_std_path();

    let registry = step_with_result("Generating registry.json", || {
        generate_registry_json(crates_dir, demo_dir_path)
    });

    step("Generating sample files", || {
        generate_sample_files(crates_dir, &registry, demo_dir_path)
    });

    let icons = step_with_result("Fetching icons", || {
        fetch_icons_from_registry(&registry, demo_dir_path)
    });

    step("Generating theme CSS", || {
        generate_theme_css(crates_dir, demo_dir_path)
    });

    step("Generating index.html", || {
        generate_index_html(crates_dir, demo_dir_path, &icons, &registry)
    });

    step("Generating IIFE demo HTML", || {
        generate_iife_demo_html(demo_dir_path)
    });

    step("Generating app.generated.js", || {
        generate_app_js(crates_dir, demo_dir_path, &registry, &icons)
    });

    step("Generating rustdoc comparison", || {
        generate_rustdoc_comparison(
            &util::find_repo_root().ok_or("Could not find repo root")?,
            demo_dir_path,
        )
    });

    // Copy plugins.json from langs/ to demo/, adding dev_mode flag
    step("Copying plugins.json", || {
        copy_plugins_json(crates_dir, demo_dir_path, dev)
    });

    if dev {
        step("Pre-compressing files (fast)", || {
            precompress_files_fast(demo_dir_path)
        });
    } else {
        step("Pre-compressing files", || precompress_files(demo_dir_path));
    }

    Ok(())
}

fn step<F, E>(name: &str, f: F)
where
    F: FnOnce() -> Result<(), E>,
    E: std::fmt::Display,
{
    print!("  {} {}... ", "●".cyan(), name);
    std::io::stdout().flush().ok();

    match f() {
        Ok(()) => println!("{}", "done".green()),
        Err(e) => {
            println!("{}", "failed".red());
            eprintln!("    Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn step_with_result<T, F, E>(name: &str, f: F) -> T
where
    F: FnOnce() -> Result<T, E>,
    E: std::fmt::Display,
{
    print!("  {} {}... ", "●".cyan(), name);
    std::io::stdout().flush().ok();

    match f() {
        Ok(result) => {
            println!("{}", "done".green());
            result
        }
        Err(e) => {
            println!("{}", "failed".red());
            eprintln!("    Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn optional_step<F, E>(name: &str, f: F)
where
    F: FnOnce() -> Result<(), E>,
    E: std::fmt::Display,
{
    print!("  {} {}... ", "●".cyan(), name);
    std::io::stdout().flush().ok();

    match f() {
        Ok(()) => println!("{}", "done".green()),
        Err(e) => {
            println!("{}", "skipped".yellow());
            eprintln!("    {}: {}", "Warning".yellow(), e);
        }
    }
}

fn generate_shared_crate_manifests(repo_root: &Path) -> Result<(), String> {
    let version = crate::version_store::read_version(Utf8Path::new(repo_root.to_str().unwrap()))
        .map_err(|e| e.to_string())?;
    let shared_crates = [
        "arborium-theme",
        "arborium-highlight",
        "arborium-sysroot",
        "arborium-test-harness",
        "arborium-tree-sitter",
        "arborium-host",
        "arborium-plugin-runtime",
        "arborium-wire",
        "arborium-query",
        "arborium-rustdoc",
        "arborium-mdbook",
    ];
    for crate_name in shared_crates {
        let crate_dir = repo_root.join("crates").join(crate_name);
        let cargo_toml = crate_dir.join("Cargo.toml");
        let cargo_template = crate_dir.join("Cargo.stpl.toml");
        if !cargo_toml.exists() && cargo_template.exists() {
            let template = fs::read_to_string(&cargo_template).map_err(|e| e.to_string())?;
            let content = template.replace("<%= version %>", &version);
            fs::write(&cargo_toml, content).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn generate_sample_files(
    _crates_dir: &Utf8Path,
    registry: &Registry,
    demo_dir: &Path,
) -> Result<(), String> {
    let samples_dir = demo_dir.join("samples");
    if !samples_dir.exists() {
        fs::create_dir_all(&samples_dir).map_err(|e| e.to_string())?;
    }

    for grammar in &registry.grammars {
        if let Some(sample) = grammar.samples.first() {
            // Read sample content from def/ directory
            let sample_path = std::path::Path::new(&grammar.def_path).join(&sample.path);

            if let Ok(content) = fs::read_to_string(&sample_path) {
                // Get extension from original sample path
                let ext = std::path::Path::new(&sample.path)
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("txt");
                // Write to demo/samples/{id}.{ext}
                let output_path = samples_dir.join(format!("{}.{}", grammar.id, ext));
                fs::write(&output_path, &content).map_err(|e| e.to_string())?;
            }
        }
    }

    Ok(())
}

fn generate_theme_css(crates_dir: &Utf8Path, demo_dir: &Path) -> Result<(), String> {
    use std::fmt::Write;

    let highlights = highlight_gen::parse_highlights(crates_dir)?;
    let themes = theme_gen::parse_all_themes(crates_dir)?;

    let pkg_dir = demo_dir.join("pkg");
    if !pkg_dir.exists() {
        fs::create_dir_all(&pkg_dir).map_err(|e| e.to_string())?;
    }

    let output_path = pkg_dir.join("themes.generated.css");
    let mut css = String::new();

    // Generate CSS for each built-in theme
    for theme in &themes {
        // Convert theme name to a valid CSS identifier (lowercase, hyphens)
        let id = theme_name_to_id(&theme.name);

        // Add header comment with attribution
        let variant = if theme.is_dark { "dark" } else { "light" };
        writeln!(css, "/* {} ({}) */", theme.name, variant).unwrap();
        if let Some(ref source_url) = theme.source_url {
            writeln!(css, "/* Source: {} */", source_url).unwrap();
        }

        // Generate CSS with [data-theme="id"] selector
        let theme_css = theme.to_css(&format!("[data-theme=\"{id}\"]"), &highlights);
        css.push_str(&theme_css);
        css.push('\n');
    }

    fs::write(&output_path, &css).map_err(|e| e.to_string())?;
    Ok(())
}

fn copy_plugins_json(crates_dir: &Utf8Path, demo_dir: &Path, dev: bool) -> Result<(), String> {
    // The plugins.json is generated in langs/ by the build command
    // We need to go up from crates/ to find langs/
    let repo_root = crates_dir.parent().ok_or("crates_dir has no parent")?;
    let plugins_path = repo_root.join("langs").join("plugins.json");
    let output_path = demo_dir.join("plugins.json");

    if plugins_path.exists() {
        // Read and parse the plugins.json
        let content = fs::read_to_string(&plugins_path).map_err(|e| e.to_string())?;

        // Parse as facet_value::Value so we can modify it
        let mut json: facet_value::Value =
            facet_json::from_str(&content).map_err(|e| e.to_string())?;

        // Add dev_mode field
        if let Some(obj) = json.as_object_mut() {
            obj.insert("dev_mode", dev);
        }

        // Write to demo directory
        let output = facet_json::to_string_pretty(&json).map_err(|e| e.to_string())?;
        fs::write(&output_path, output).map_err(|e| e.to_string())?;
    } else {
        // Generate a minimal plugins.json from registry if build hasn't been run
        // This allows the demo UI to work, even if WASM grammars aren't available
        let crate_registry = CrateRegistry::load(crates_dir).map_err(|e| e.to_string())?;
        let version = crate::version_store::read_version(repo_root).map_err(|e| e.to_string())?;

        let entries: Vec<String> = crate_registry
            .all_grammars()
            .filter(|(_, _, g)| g.generate_component())
            .map(|(_, _, g)| {
                format!(
                    r#"    {{"language": "{}", "js": "/pkg/{}.js", "wasm": "/pkg/{}_bg.wasm"}}"#,
                    g.id(),
                    g.id(),
                    g.id()
                )
            })
            .collect();

        let output = format!(
            r#"{{
  "version": "{}",
  "entries": [
{}
  ],
  "dev_mode": {}
}}"#,
            version,
            entries.join(",\n"),
            dev
        );
        fs::write(&output_path, output).map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn generate_registry_json(crates_dir: &Utf8Path, demo_dir: &Path) -> Result<Registry, String> {
    let crates_dir =
        Utf8PathBuf::from_path_buf(crates_dir.to_path_buf().into()).map_err(|_| "non-UTF8 path")?;

    let crate_registry = CrateRegistry::load(&crates_dir).map_err(|e| e.to_string())?;
    let registry = Registry::from_crate_registry(&crate_registry, &crates_dir);

    let json = registry.to_json_pretty();
    let registry_path = demo_dir.join("registry.json");
    fs::write(&registry_path, &json).map_err(|e| e.to_string())?;

    Ok(registry)
}

fn fetch_icons_from_registry(
    registry: &Registry,
    demo_dir: &Path,
) -> Result<BTreeMap<String, String>, String> {
    // Read template from xtask/templates (compile-time location)
    let template = include_str!("../templates/index.stpl.html");

    // Collect all icon names from registry
    let mut icon_names: HashSet<String> = HashSet::new();
    for grammar in &registry.grammars {
        if let Some(ref icon) = grammar.icon {
            icon_names.insert(icon.clone());
        }
    }

    // Also collect icons from template (icons["xxx"] patterns from sailfish syntax)
    let icon_pattern = regex::Regex::new(r#"icons\["([^"]+)"\]"#).unwrap();
    for cap in icon_pattern.captures_iter(template) {
        if let Some(icon) = cap.get(1) {
            icon_names.insert(icon.as_str().to_string());
        }
    }

    // Add fallback icon
    icon_names.insert("mdi:code-tags".to_string());

    // Load cached icons
    let cache_path = demo_dir.join(".icon-cache.json");
    let mut cached_icons: BTreeMap<String, String> = if cache_path.exists() {
        let content = fs::read_to_string(&cache_path).unwrap_or_default();
        // Parse JSON manually since we don't have serde
        parse_icon_cache(&content)
    } else {
        BTreeMap::new()
    };

    let mut icons: BTreeMap<String, String> = BTreeMap::new();
    let mut fetch_count = 0;

    for icon_name in &icon_names {
        // Check cache first
        if let Some(svg) = cached_icons.get(icon_name) {
            icons.insert(icon_name.clone(), svg.clone());
            continue;
        }

        // Parse icon name (prefix:name)
        let parts: Vec<&str> = icon_name.split(':').collect();
        if parts.len() != 2 {
            continue;
        }
        let (prefix, name) = (parts[0], parts[1]);

        // Fetch from Iconify API
        let url = format!("https://api.iconify.design/{}/{}.svg", prefix, name);
        // Fetch from Iconify API using curl
        let output = std::process::Command::new("curl")
            .arg("-s") // silent
            .arg("-f") // fail fast on HTTP errors
            .arg(&url)
            .output();

        match output {
            Ok(output) if output.status.success() => {
                if let Ok(svg) = String::from_utf8(output.stdout) {
                    // Clean up the SVG
                    let cleaned = svg
                        .replace("xmlns=\"http://www.w3.org/2000/svg\"", "")
                        .replace("xmlns:xlink=\"http://www.w3.org/1999/xlink\"", "");
                    icons.insert(icon_name.clone(), cleaned.clone());
                    cached_icons.insert(icon_name.clone(), cleaned);
                    fetch_count += 1;
                }
            }
            _ => {
                // Skip failed icons
            }
        }
    }

    // Save cache
    if fetch_count > 0 {
        let cache_json = serialize_icon_cache(&cached_icons);
        let _ = fs::write(&cache_path, cache_json);
    }

    Ok(icons)
}

fn parse_icon_cache(content: &str) -> BTreeMap<String, String> {
    // Simple JSON parsing for icon cache (key-value string pairs)
    let mut result = BTreeMap::new();
    // Use facet-json to parse
    if let Ok(map) = facet_json::from_str::<BTreeMap<String, String>>(content) {
        result = map;
    }
    result
}

fn serialize_icon_cache(icons: &BTreeMap<String, String>) -> String {
    facet_json::to_string_pretty(icons).expect("icon cache serialization failed")
}

/// Convert a theme name to a CSS-friendly ID (kebab-case, lowercase)
fn theme_name_to_id(name: &str) -> String {
    name.to_lowercase().replace(' ', "-").replace('é', "e") // Handle "Rosé Pine"
}

/// Generate HTML for theme swatches in the "Theme support" section
fn generate_theme_swatches(themes: &[Theme]) -> String {
    let mut html = String::new();

    // Sample code snippet (Rust) for each theme preview
    // Keep it short to fit in small boxes
    let sample_code = r#"<a-k>fn</a-k> <a-f>main</a-f><a-p>()</a-p> <a-p>{</a-p>
    <a-k>let</a-k> <a-v>x</a-v> <a-o>=</a-o> <a-n>42</a-n><a-p>;</a-p>
    <a-fb>println!</a-fb><a-p>(</a-p><a-s>"Hello"</a-s><a-p>)</a-p><a-p>;</a-p>
<a-p>}</a-p>"#;

    for theme in themes {
        let id = theme_name_to_id(&theme.name);
        let variant = if theme.is_dark { "dark" } else { "light" };
        let bg = theme
            .background
            .map(|c| c.to_hex())
            .unwrap_or_else(|| "#1e1e2e".to_string());

        html.push_str(&format!(
            r#"<div class="theme-preview" data-variant="{variant}" data-theme="{id}">
    <pre style="background: {bg}; padding: 0.75rem; border-radius: 6px;"><code>{code}</code></pre>
    <span class="theme-name">{name}</span>
</div>
"#,
            id = id,
            variant = variant,
            bg = bg,
            name = theme.name,
            code = sample_code,
        ));
    }

    html
}

/// Generate code blocks for the index page (highlighted client-side by IIFE)
fn generate_code_blocks() -> CodeBlocks {
    CodeBlocks {
        script_tag: CodeBlock {
            lang: "html",
            source: r#"<script src="https://cdn.jsdelivr.net/npm/@arborium/arborium@1/dist/arborium.iife.js"></script>"#,
        },
        code_block_examples: CodeBlock {
            lang: "html",
            source: r#"<pre><code class="language-rust">fn main() {}</code></pre>
<!-- or -->
<pre><code data-lang="rust">fn main() {}</code></pre>
<!-- or just let it auto-detect -->
<pre><code>fn main() {}</code></pre>"#,
        },
        data_attributes: CodeBlock {
            lang: "html",
            source: r#"<script src="..."
  data-theme="github-light"      <!-- theme name -->
  data-selector="pre code"        <!-- CSS selector -->
  data-manual                     <!-- disable auto-highlight -->
  data-cdn="unpkg"></script>       <!-- jsdelivr | unpkg | custom URL -->"#,
        },
        cargo_toml: CodeBlock {
            lang: "toml",
            source: r#"arborium = { version = "2", features = ["lang-rust"] }"#,
        },
        rust_highlight: CodeBlock {
            lang: "rust",
            source: r#"let html = arborium::highlight("rust", source)?;"#,
        },
        js_esm: CodeBlock {
            lang: "javascript",
            source: r#"import { loadGrammar, highlight } from '@arborium/arborium';

const html = await highlight('rust', sourceCode);"#,
        },
        docsrs_script: CodeBlock {
            lang: "html",
            source: r#"<script defer src="https://cdn.jsdelivr.net/npm/@arborium/arborium@1/dist/arborium.iife.js"></script>"#,
        },
        docsrs_cargo: CodeBlock {
            lang: "toml",
            source: r#"[package.metadata.docs.rs]
rustdoc-args = ["--html-in-header", "arborium-header.html"]"#,
        },
        rustdoc_postprocess: CodeBlock {
            lang: "bash",
            source: r#"# Process rustdoc output in-place
arborium-rustdoc ./target/doc ./target/doc-highlighted"#,
        },
        html_example_traditional: CodeBlock {
            lang: "html",
            source: r#"<span class="keyword">fn</span>"#,
        },
        html_example_arborium: CodeBlock {
            lang: "html",
            source: r#"<a-k>fn</a-k>"#,
        },
    }
}

fn generate_index_html(
    crates_dir: &Utf8Path,
    demo_dir: &Path,
    icons: &BTreeMap<String, String>,
    registry: &Registry,
) -> Result<(), String> {
    let themes = theme_gen::parse_all_themes(crates_dir)?;
    let output_path = demo_dir.join("index.html");

    // Generate theme swatches
    let swatches_html = generate_theme_swatches(&themes);
    let theme_css_link = "\n    <link rel=\"stylesheet\" href=\"/pkg/themes.generated.css\">";
    let code_blocks = generate_code_blocks();
    let language_count = registry.grammars.len();

    let template = IndexHtmlTemplate {
        icons,
        theme_swatches: &swatches_html,
        theme_css_link,
        code: &code_blocks,
        language_count,
        default_theme: "tokyo-night",
    };

    let html = template.render_once().map_err(|e| e.to_string())?;
    fs::write(&output_path, &html).map_err(|e| e.to_string())?;
    Ok(())
}

fn build_iife_bundle(repo_root: &Path, demo_dir: &Path) -> Result<(), String> {
    let packages_dir = repo_root.join("packages/arborium");

    // Ensure dependencies are installed
    let node_modules = packages_dir.join("node_modules");
    if !node_modules.exists() {
        let status = std::process::Command::new("pnpm")
            .arg("install")
            .current_dir(&packages_dir)
            .status()
            .map_err(|e| format!("Failed to run pnpm install: {}", e))?;

        if !status.success() {
            return Err(format!("pnpm install failed with status: {}", status));
        }
    }

    // Run npm/pnpm to build the IIFE
    let status = std::process::Command::new("pnpm")
        .arg("run")
        .arg("build:iife")
        .current_dir(&packages_dir)
        .status()
        .map_err(|e| format!("Failed to run pnpm build:iife: {}", e))?;

    if !status.success() {
        return Err(format!("pnpm build:iife failed with status: {}", status));
    }

    // Copy the built IIFE to demo/pkg
    let src = packages_dir.join("dist/arborium.iife.js");
    let dst = demo_dir.join("pkg/arborium.iife.js");

    std::fs::copy(&src, &dst)
        .map_err(|e| format!("Failed to copy IIFE from {:?} to {:?}: {}", src, dst, e))?;

    // Also copy the source map if it exists
    let src_map = packages_dir.join("dist/arborium.iife.js.map");
    if src_map.exists() {
        let dst_map = demo_dir.join("pkg/arborium.iife.js.map");
        std::fs::copy(&src_map, &dst_map).ok(); // Ignore errors for source map
    }

    Ok(())
}

fn generate_iife_demo_html(demo_dir: &Path) -> Result<(), String> {
    // Generate local version (iife-demo-local.html)
    let local_template = IifeDemoHtmlTemplate {
        title_suffix: " (Local)",
        theme_css_link: "    <!-- Local theme CSS -->\n    <link rel=\"stylesheet\" href=\"/pkg/themes.generated.css\">",
        description: "Testing the script tag integration with local builds. All code blocks below are automatically highlighted using tree-sitter grammars loaded on demand.",
        footer_suffix: " (Local Build)",
        script_tag: "    <!-- Local IIFE with query params for local testing -->\n    <script src=\"/pkg/arborium.iife.js?pluginsUrl=/plugins.json&hostUrl=/pkg\"></script>",
    };
    let local_html = local_template.render_once().map_err(|e| e.to_string())?;
    let local_output = demo_dir.join("iife-demo-local.html");
    fs::write(&local_output, &local_html).map_err(|e| e.to_string())?;

    // Generate CDN version (iife-demo.html)
    let cdn_template = IifeDemoHtmlTemplate {
        title_suffix: "",
        theme_css_link: "",
        description: "Testing the script tag integration via CDN. All code blocks below are automatically highlighted using tree-sitter grammars loaded on demand.",
        footer_suffix: "",
        script_tag: "    <script src=\"https://cdn.jsdelivr.net/npm/@arborium/arborium@1/dist/arborium.iife.js\"></script>",
    };
    let cdn_html = cdn_template.render_once().map_err(|e| e.to_string())?;
    let cdn_output = demo_dir.join("iife-demo.html");
    fs::write(&cdn_output, &cdn_html).map_err(|e| e.to_string())?;

    Ok(())
}

fn generate_rustdoc_comparison(repo_root: &Path, demo_dir: &Path) -> Result<(), String> {
    use std::process::Command;
    use walkdir::WalkDir;

    let demo_crate_dir = repo_root.join("crates").join("arborium-docsrs-demo");
    let rustdoc_dir = repo_root.join("crates").join("arborium-rustdoc");
    let comparison_dir = demo_dir.join("rustdoc-comparison");
    let before_dir = comparison_dir.join("before");
    let after_dir = comparison_dir.join("after");

    // Create comparison directories
    fs::create_dir_all(&before_dir).map_err(|e| e.to_string())?;
    fs::create_dir_all(&after_dir).map_err(|e| e.to_string())?;

    // Generate rustdoc for arborium-docsrs-demo
    let status = Command::new("cargo")
        .arg("doc")
        .arg("--no-deps")
        .current_dir(&demo_crate_dir)
        .status()
        .map_err(|e| format!("Failed to run cargo doc: {}", e))?;

    if !status.success() {
        return Err("cargo doc failed".to_string());
    }

    let doc_output = demo_crate_dir.join("target").join("doc");

    // Copy rustdoc output to before directory using walkdir
    for entry in WalkDir::new(&doc_output).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        let relative_path = path.strip_prefix(&doc_output).map_err(|e| e.to_string())?;
        let dest_path = before_dir.join(relative_path);

        if entry.file_type().is_dir() {
            fs::create_dir_all(&dest_path).map_err(|e| e.to_string())?;
        } else {
            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            fs::copy(path, &dest_path).map_err(|e| e.to_string())?;
        }
    }

    // Run arborium-rustdoc to process into after directory
    let status = Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg(&doc_output)
        .arg(&after_dir)
        .current_dir(&rustdoc_dir)
        .status()
        .map_err(|e| format!("Failed to run arborium-rustdoc: {}", e))?;

    if !status.success() {
        return Err("arborium-rustdoc failed".to_string());
    }

    // Generate the comparison HTML page
    let template = RustdocComparisonHtmlTemplate;
    let html = template.render_once().map_err(|e| e.to_string())?;
    let output = demo_dir.join("rustdoc-comparison.html");
    fs::write(&output, &html).map_err(|e| e.to_string())?;

    Ok(())
}

fn generate_app_js(
    crates_dir: &Utf8Path,
    demo_dir: &Path,
    registry: &Registry,
    icons: &BTreeMap<String, String>,
) -> Result<(), String> {
    let themes = theme_gen::parse_all_themes(crates_dir)?;
    let output_path = demo_dir.join("pkg").join("app.generated.js");

    // Ensure pkg directory exists
    let pkg_dir = demo_dir.join("pkg");
    if !pkg_dir.exists() {
        fs::create_dir_all(&pkg_dir).map_err(|e| e.to_string())?;
    }

    // Build languageInfo object from registry
    let lang_info_js = build_language_info_js(registry);

    // Build examples object from registry samples
    let examples_js = build_examples_js(registry);

    // Build icons object
    let icons_js = build_icons_js(icons);

    // Build themeInfo object from parsed themes
    let theme_info_js = build_theme_info_js(&themes);

    // Render the template
    let template = AppJsTemplate {
        language_info: &lang_info_js,
        examples: &examples_js,
        icons: &icons_js,
        theme_info: &theme_info_js,
    };
    let output = template.render_once().map_err(|e| e.to_string())?;

    fs::write(&output_path, &output).map_err(|e| e.to_string())?;
    Ok(())
}

fn build_language_info_js(registry: &Registry) -> String {
    // Build a JS object with language info keyed by id
    let mut js = String::from("{\n");
    for (i, grammar) in registry.grammars.iter().enumerate() {
        js.push_str(&format!("    \"{}\": {{\n", grammar.id));
        js.push_str(&format!(
            "        \"name\": \"{}\",\n",
            escape_for_js(&grammar.name)
        ));
        js.push_str(&format!(
            "        \"tag\": \"{}\",\n",
            escape_for_js(&grammar.tag)
        ));
        if let Some(ref icon) = grammar.icon {
            js.push_str(&format!("        \"icon\": \"{}\",\n", escape_for_js(icon)));
        }
        if let Some(tier) = grammar.tier {
            js.push_str(&format!("        \"tier\": {},\n", tier));
        }
        if let Some(ref desc) = grammar.description {
            // description is already HTML in arborium.yaml
            js.push_str(&format!(
                "        \"description\": \"{}\",\n",
                escape_for_js(desc)
            ));
        }
        if let Some(ref inventor) = grammar.inventor {
            js.push_str(&format!(
                "        \"inventor\": \"{}\",\n",
                escape_for_js(inventor)
            ));
        }
        if let Some(year) = grammar.year {
            js.push_str(&format!("        \"year\": {},\n", year));
        }
        if let Some(ref link) = grammar.link {
            js.push_str(&format!("        \"url\": \"{}\",\n", escape_for_js(link)));
        }
        if let Some(ref trivia) = grammar.trivia {
            // trivia is already HTML in arborium.yaml
            js.push_str(&format!(
                "        \"trivia\": \"{}\",\n",
                escape_for_js(trivia)
            ));
        }
        if !grammar.aliases.is_empty() {
            let aliases: Vec<String> = grammar
                .aliases
                .iter()
                .map(|a| format!("\"{}\"", escape_for_js(a)))
                .collect();
            js.push_str(&format!("        \"aliases\": [{}],\n", aliases.join(", ")));
        }
        if let Some(ref repo) = grammar.grammar_repo {
            js.push_str(&format!(
                "        \"grammarRepo\": \"{}\",\n",
                escape_for_js(repo)
            ));
        }
        if let Some(ref license) = grammar.grammar_license {
            js.push_str(&format!(
                "        \"grammarLicense\": \"{}\",\n",
                escape_for_js(license)
            ));
        }
        // Remove trailing comma and newline, add just newline
        if js.ends_with(",\n") {
            js.pop();
            js.pop();
            js.push('\n');
        }
        js.push_str("    }");
        if i < registry.grammars.len() - 1 {
            js.push(',');
        }
        js.push('\n');
    }
    js.push('}');
    js
}

fn build_examples_js(registry: &Registry) -> String {
    // Build object mapping language id to file extension
    // Samples are fetched on demand from /samples/{id}.{ext}
    let mut js = String::from("{\n");
    let mut first = true;
    for grammar in &registry.grammars {
        if let Some(sample) = grammar.samples.first() {
            if !first {
                js.push_str(",\n");
            }
            first = false;
            // Get extension from sample path
            let ext = std::path::Path::new(&sample.path)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("txt");
            // Value is the file extension for fetching
            js.push_str(&format!("    \"{}\": \"{}\"", grammar.id, ext));
        }
    }
    js.push_str("\n}");
    js
}

fn build_icons_js(icons: &BTreeMap<String, String>) -> String {
    let mut js = String::from("{\n");
    for (i, (name, svg)) in icons.iter().enumerate() {
        js.push_str(&format!("    \"{}\": \"{}\"", name, escape_for_js(svg)));
        if i < icons.len() - 1 {
            js.push(',');
        }
        js.push('\n');
    }
    js.push('}');
    js
}

fn escape_for_js(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn build_theme_info_js(themes: &[Theme]) -> String {
    let mut js = String::from("{\n");
    for (i, theme) in themes.iter().enumerate() {
        let id = theme_name_to_id(&theme.name);
        let variant = if theme.is_dark { "dark" } else { "light" };
        let source = theme
            .source_url
            .as_ref()
            .map(|s| format!(", source: \"{}\"", escape_for_js(s)))
            .unwrap_or_default();
        js.push_str(&format!(
            "    \"{}\": {{ name: \"{}\", variant: \"{}\"{}}}",
            id,
            escape_for_js(&theme.name),
            variant,
            source
        ));
        if i < themes.len() - 1 {
            js.push(',');
        }
        js.push('\n');
    }
    js.push('}');
    js
}

fn precompress_files(_demo_dir: &Path) -> Result<(), String> {
    let files = ["index.html", "registry.json", "styles.css", "app.js"];
    let pkg_files = ["arborium_demo.js", "app.generated.js"];

    for _file in files {
        // Compression disabled
    }

    for _file in pkg_files {
        // Compression disabled
    }

    Ok(())
}

fn precompress_files_fast(_demo_dir: &Path) -> Result<(), String> {
    let files = ["index.html", "registry.json", "styles.css", "app.js"];
    let pkg_files = ["arborium_demo.js", "app.generated.js"];

    for _file in files {
        // Compression disabled
    }

    for _file in pkg_files {
        // Compression disabled
    }

    Ok(())
}

fn bind_server(addr: &str, port: Option<u16>) -> (tiny_http::Server, u16) {
    if let Some(p) = port {
        match tiny_http::Server::http(format!("{}:{}", addr, p)) {
            Ok(s) => (s, p),
            Err(e) => {
                eprintln!("Error: Could not bind to {}:{}: {}", addr, p, e);
                std::process::exit(1);
            }
        }
    } else {
        // Try ports 8000-8010
        for p in 8000..=8010 {
            if let Ok(s) = tiny_http::Server::http(format!("{}:{}", addr, p)) {
                return (s, p);
            }
        }
        eprintln!("Error: Could not bind to any port between 8000-8010");
        std::process::exit(1);
    }
}

fn serve_files(server: tiny_http::Server, demo_dir: &Path) {
    // Get repo root for serving langs/ files
    let repo_root = util::find_repo_root().expect("Could not find repo root");

    for request in server.incoming_requests() {
        // Strip query string from URL path
        let url = request.url();
        let url_path = url.split('?').next().unwrap_or(url).trim_start_matches('/');

        // Determine base directory and allowed prefix based on path
        let (file_path, allowed_prefix) = if url_path.is_empty() || url_path == "/" {
            (demo_dir.join("index.html"), demo_dir.to_path_buf())
        } else if url_path.starts_with("langs/") {
            // Serve from repo root for langs/ paths
            (repo_root.join(url_path), repo_root.join("langs"))
        } else {
            (demo_dir.join(url_path), demo_dir.to_path_buf())
        };

        // Security: ensure path is within allowed directory
        let file_path = match file_path.canonicalize() {
            Ok(p) => p,
            Err(_) => {
                let response = tiny_http::Response::from_string("Not Found").with_status_code(404);
                let _ = request.respond(response);
                continue;
            }
        };

        let allowed_prefix = allowed_prefix.canonicalize().unwrap_or(allowed_prefix);
        if !file_path.starts_with(&allowed_prefix) {
            let response = tiny_http::Response::from_string("Forbidden").with_status_code(403);
            let _ = request.respond(response);
            continue;
        }

        // Check what encodings the client accepts
        let accept_encoding = request
            .headers()
            .iter()
            .find(|h| h.field.as_str().to_ascii_lowercase() == "accept-encoding")
            .map(|h| h.value.as_str().to_string())
            .unwrap_or_default();

        let accepts_br = accept_encoding.contains("br");
        let accepts_gzip = accept_encoding.contains("gzip");

        // Try to serve pre-compressed files
        let br_path = PathBuf::from(format!("{}.br", file_path.display()));
        let gz_path = PathBuf::from(format!("{}.gz", file_path.display()));
        let (serve_path, encoding) = if accepts_br && br_path.exists() {
            (br_path, Some("br"))
        } else if accepts_gzip && gz_path.exists() {
            (gz_path, Some("gzip"))
        } else {
            (file_path.clone(), None)
        };

        // Read and serve the file
        match fs::read(&serve_path) {
            Ok(content) => {
                let content_type = guess_content_type(&file_path);

                let mut response = tiny_http::Response::from_data(content).with_header(
                    tiny_http::Header::from_bytes(&b"Content-Type"[..], content_type.as_bytes())
                        .unwrap(),
                );

                if let Some(enc) = encoding {
                    response = response.with_header(
                        tiny_http::Header::from_bytes(&b"Content-Encoding"[..], enc.as_bytes())
                            .unwrap(),
                    );
                }

                let _ = request.respond(response);
            }
            Err(_) => {
                let response = tiny_http::Response::from_string("Not Found").with_status_code(404);
                let _ = request.respond(response);
            }
        }
    }
}

fn guess_content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") | Some("htm") => "text/html; charset=utf-8",
        Some("js") | Some("mjs") | Some("cjs") => "application/javascript; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        Some("wasm") => "application/wasm",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("ico") => "image/x-icon",
        Some("woff2") => "font/woff2",
        Some("woff") => "font/woff",
        // All source code files served as plain text
        _ => "text/plain; charset=utf-8",
    }
}

/// Generate theme CSS files for the npm package from TOML theme definitions.
///
/// This generates:
/// 1. Individual theme files that declare CSS variables with -light or -dark suffix
/// 2. base.css - uses media queries and [data-theme] selectors for switching
/// 3. base-rustdoc.css - uses variable fallback for JS-based switching
pub fn generate_npm_theme_css(crates_dir: &Utf8Path) -> Result<(), String> {
    use std::fmt::Write;

    let highlights = highlight_gen::parse_highlights(crates_dir)?;
    let themes = theme_gen::parse_all_themes(crates_dir)?;

    let repo_root = crates_dir.parent().ok_or("crates_dir has no parent")?;
    let themes_dir = repo_root.join("packages/arborium/src/themes");

    // Create directory if it doesn't exist
    fs::create_dir_all(&themes_dir).map_err(|e| e.to_string())?;

    println!(
        "{} Generating theme CSS files in {}",
        "●".cyan(),
        themes_dir.cyan()
    );

    // Collect all unique tags
    let mut all_tags: Vec<&str> = Vec::new();
    for def in &highlights.defs {
        if !def.def.tag.is_empty() && !all_tags.contains(&def.def.tag.as_str()) {
            all_tags.push(&def.def.tag);
        }
    }

    // Generate individual theme files
    let mut generated = 0;
    for theme in &themes {
        let id = theme_name_to_id(&theme.name);
        let variant = if theme.is_dark { "dark" } else { "light" };
        let output_path = themes_dir.join(format!("{}.css", id));

        let mut css = String::new();

        // Header comment
        writeln!(
            css,
            "/* {} theme ({}) - generated from TOML */",
            theme.name, variant
        )
        .unwrap();
        if let Some(ref source_url) = theme.source_url {
            writeln!(css, "/* Source: {} */", source_url).unwrap();
        }
        writeln!(css, "/* Defines --arb-*-{} variables */\n", variant).unwrap();

        writeln!(css, ":root {{").unwrap();

        // Generate CSS variables for each unique tag with fallback resolution
        for def in highlights.unique_tags() {
            // Resolve style with fallback
            let Some(style) = theme.resolve_style(def, &highlights) else {
                continue;
            };

            if style.is_empty() {
                continue;
            }

            if let Some(fg) = &style.fg {
                writeln!(css, "  --arb-{}-{}: {};", def.def.tag, variant, fg.to_hex()).unwrap();
            }

            // Handle modifiers as separate variables
            if style.bold {
                writeln!(css, "  --arb-{}-{}-weight: bold;", def.def.tag, variant).unwrap();
            }
            if style.italic {
                writeln!(css, "  --arb-{}-{}-style: italic;", def.def.tag, variant).unwrap();
            }
            if style.underline || style.strikethrough {
                let mut decorations = Vec::new();
                if style.underline {
                    decorations.push("underline");
                }
                if style.strikethrough {
                    decorations.push("line-through");
                }
                writeln!(
                    css,
                    "  --arb-{}-{}-decoration: {};",
                    def.def.tag,
                    variant,
                    decorations.join(" ")
                )
                .unwrap();
            }
        }

        writeln!(css, "}}").unwrap();

        fs::write(&output_path, &css).map_err(|e| e.to_string())?;
        generated += 1;
    }

    // Helper to emit tag rules
    let emit_tag_rules = |css: &mut String, variant: &str, indent: &str| {
        let mut emitted: std::collections::HashSet<&str> = std::collections::HashSet::new();
        for def in &highlights.defs {
            if def.def.tag.is_empty() || emitted.contains(def.def.tag.as_str()) {
                continue;
            }
            emitted.insert(&def.def.tag);
            writeln!(
                css,
                "{}a-{} {{ color: var(--arb-{}-{}); font-weight: var(--arb-{}-{}-weight, normal); font-style: var(--arb-{}-{}-style, normal); text-decoration: var(--arb-{}-{}-decoration, none); }}",
                indent, def.def.tag, def.def.tag, variant, def.def.tag, variant, def.def.tag, variant, def.def.tag, variant
            ).unwrap();
        }
    };

    // Generate base.css - standard switching with media queries and [data-theme]
    let base_path = themes_dir.join("base.css");
    let mut base_css = String::new();

    writeln!(
        base_css,
        "/* Arborium base CSS - handles light/dark switching */"
    )
    .unwrap();
    writeln!(
        base_css,
        "/* Include this + one light theme + one dark theme */\n"
    )
    .unwrap();

    // Default: use light variables
    writeln!(base_css, "/* Default: light mode */").unwrap();
    emit_tag_rules(&mut base_css, "light", "");

    // Media query for dark preference
    writeln!(base_css, "\n/* System preference: dark */").unwrap();
    writeln!(base_css, "@media (prefers-color-scheme: dark) {{").unwrap();
    emit_tag_rules(&mut base_css, "dark", "  ");
    writeln!(base_css, "}}").unwrap();

    // Explicit data-theme overrides
    writeln!(base_css, "\n/* Explicit light mode */").unwrap();
    writeln!(base_css, ":root[data-theme=\"light\"] {{").unwrap();
    emit_tag_rules(&mut base_css, "light", "  ");
    writeln!(base_css, "}}").unwrap();

    writeln!(base_css, "\n/* Explicit dark mode */").unwrap();
    writeln!(base_css, ":root[data-theme=\"dark\"] {{").unwrap();
    emit_tag_rules(&mut base_css, "dark", "  ");
    writeln!(base_css, "}}").unwrap();

    fs::write(&base_path, &base_css).map_err(|e| e.to_string())?;

    // Generate base-rustdoc.css - uses variable fallback for JS-based switching
    let base_rustdoc_path = themes_dir.join("base-rustdoc.css");
    let mut rustdoc_css = String::new();

    writeln!(
        rustdoc_css,
        "/* Arborium base CSS for rustdoc - uses variable fallback */"
    )
    .unwrap();
    writeln!(
        rustdoc_css,
        "/* JS dynamically loads one theme at a time */\n"
    )
    .unwrap();

    for def in highlights.unique_tags() {
        writeln!(
            rustdoc_css,
            "a-{} {{ color: var(--arb-{}-dark, var(--arb-{}-light)); font-weight: var(--arb-{}-dark-weight, var(--arb-{}-light-weight, normal)); font-style: var(--arb-{}-dark-style, var(--arb-{}-light-style, normal)); text-decoration: var(--arb-{}-dark-decoration, var(--arb-{}-light-decoration, none)); }}",
            def.def.tag, def.def.tag, def.def.tag, def.def.tag, def.def.tag, def.def.tag, def.def.tag, def.def.tag, def.def.tag
        ).unwrap();
    }

    fs::write(&base_rustdoc_path, &rustdoc_css).map_err(|e| e.to_string())?;

    println!(
        "{} Generated {} theme files + base.css + base-rustdoc.css",
        "✓".green(),
        generated.to_string().cyan()
    );

    // Also generate the arborium-theme README
    generate_arborium_theme_readme(crates_dir)?;

    Ok(())
}

/// Generate the arborium-theme crate README from template.
pub fn generate_arborium_theme_readme(crates_dir: &Utf8Path) -> Result<(), String> {
    let parsed_themes = theme_gen::parse_all_themes(crates_dir)?;

    let readme_path = crates_dir.join("arborium-theme/README.md");

    // Collect theme info
    let themes: Vec<ThemeInfo> = parsed_themes
        .iter()
        .map(|theme| {
            let source_display = theme
                .source_url
                .as_ref()
                .map(|url| {
                    // Extract a nice display name from the URL
                    if url.contains("github.com") {
                        // Extract repo path like "catppuccin/catppuccin"
                        url.trim_start_matches("https://github.com/")
                            .trim_end_matches('/')
                            .to_string()
                    } else {
                        // Use domain for non-GitHub URLs
                        url.trim_start_matches("https://")
                            .trim_start_matches("http://")
                            .trim_end_matches('/')
                            .to_string()
                    }
                })
                .unwrap_or_default();

            ThemeInfo {
                name: theme.name.clone(),
                variant: if theme.is_dark { "dark" } else { "light" },
                source_url: theme.source_url.clone(),
                source_display,
            }
        })
        .collect();

    let template = ArboriumThemeReadmeTemplate {
        theme_count: themes.len(),
        themes: &themes,
    };

    let content = template
        .render_once()
        .map_err(|e| format!("Failed to render arborium-theme README: {}", e))?;

    fs::write(&readme_path, content).map_err(|e| e.to_string())?;

    println!(
        "{} Generated arborium-theme README with {} themes",
        "✓".green(),
        themes.len().to_string().cyan()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util;

    #[test]
    fn test_registry_to_json() {
        let registry = Registry {
            grammars: vec![RegistryGrammar {
                id: "rust".to_string(),
                crate_name: "arborium-rust".to_string(),
                name: "Rust".to_string(),
                icon: Some("devicon-plain:rust".to_string()),
                tier: Some(1),
                tag: "code".to_string(),
                description: Some("Systems programming language".to_string()),
                inventor: Some("Graydon Hoare".to_string()),
                year: Some(2010),
                link: Some("https://www.rust-lang.org/".to_string()),
                trivia: Some("Originally a personal project".to_string()),
                aliases: vec!["rs".to_string()],
                grammar_repo: Some("https://github.com/tree-sitter/tree-sitter-rust".to_string()),
                grammar_license: Some("MIT".to_string()),
                samples: vec![RegistrySample {
                    path: "samples/example.rs".to_string(),
                    description: Some("Example code".to_string()),
                }],
                def_path: String::new(),
            }],
        };

        let json = registry.to_json_pretty();
        assert!(json.contains("\"rust\""));
        assert!(json.contains("\"arborium-rust\""));
    }

    #[test]
    fn test_registry_from_crate_registry() {
        let repo_root = util::find_repo_root().expect("Could not find repo root");
        let crates_dir = repo_root.join("crates");
        let crates_dir = camino::Utf8PathBuf::from_path_buf(crates_dir).expect("non-UTF8 path");

        let crate_registry = CrateRegistry::load(&crates_dir).expect("Failed to load registry");
        let registry = Registry::from_crate_registry(&crate_registry, &crates_dir);

        // Should have many grammars
        assert!(
            registry.grammars.len() > 50,
            "Expected 50+ grammars, got {}",
            registry.grammars.len()
        );

        // Check a known grammar exists
        let rust = registry.grammars.iter().find(|g| g.id == "rust");
        assert!(rust.is_some(), "Rust grammar should exist");

        let rust = rust.unwrap();
        assert_eq!(rust.crate_name, "arborium-rust");
        assert_eq!(rust.name, "Rust");
        assert!(!rust.samples.is_empty(), "Rust should have samples");

        // Print stats
        let total_samples: usize = registry.grammars.iter().map(|g| g.samples.len()).sum();
        println!("Registry stats:");
        println!("  Grammars: {}", registry.grammars.len());
        println!("  Total samples: {}", total_samples);

        // Print JSON size
        let json = registry.to_json_pretty();
        println!("  JSON size: {} bytes", json.len());
    }
}
