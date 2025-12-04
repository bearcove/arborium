//! WASM plugin build system.
//!
//! This module handles building grammar plugins as WASM components
//! and transpiling them to JavaScript for browser usage.

use std::time::Instant;

use camino::{Utf8Path, Utf8PathBuf};
use chrono::Utc;
use miette::{Context, IntoDiagnostic, Result};
use owo_colors::OwoColorize;

use crate::tool::Tool;
use crate::types::{CompressionConfig, CrateRegistry};

/// Build options for plugins.
pub struct BuildOptions {
    /// Specific grammars to build (empty = all)
    pub grammars: Vec<String>,
    /// Output directory for built plugins
    pub output_dir: Utf8PathBuf,
    /// Whether to run jco transpile after building
    pub transpile: bool,
    /// Whether to profile build times and write to plugin-timings.json
    pub profile: bool,
}

impl Default for BuildOptions {
    fn default() -> Self {
        Self {
            grammars: Vec::new(),
            output_dir: Utf8PathBuf::from("dist/plugins"),
            transpile: true,
            profile: false,
        }
    }
}

/// Timing data for a single grammar plugin build.
#[derive(Debug, Clone, facet::Facet)]
#[facet(rename_all = "snake_case")]
pub struct PluginTiming {
    /// Grammar ID (e.g., "rust", "javascript")
    pub grammar: String,
    /// Total build time in milliseconds
    pub build_ms: u64,
    /// Time for cargo-component build step in milliseconds
    pub cargo_component_ms: u64,
    /// Time for jco transpile step in milliseconds (0 if transpile disabled)
    pub transpile_ms: u64,
}

/// Collection of plugin build timings.
#[derive(Debug, Clone, facet::Facet)]
#[facet(rename_all = "snake_case")]
pub struct PluginTimings {
    /// When these timings were recorded
    pub recorded_at: String,
    /// Individual grammar timings
    pub timings: Vec<PluginTiming>,
}

impl PluginTimings {
    /// Load timings from a JSON file.
    pub fn load(path: &Utf8Path) -> miette::Result<Self> {
        let content = fs_err::read_to_string(path)
            .map_err(|e| miette::miette!("failed to read {}: {}", path, e))?;
        facet_json::from_str(&content)
            .map_err(|e| miette::miette!("failed to parse {}: {}", path, e))
    }

    /// Save timings to a JSON file.
    pub fn save(&self, path: &Utf8Path) -> miette::Result<()> {
        let content = facet_json::to_string_pretty(self);
        fs_err::write(path, content)
            .map_err(|e| miette::miette!("failed to write {}: {}", path, e))?;
        Ok(())
    }
}

/// Build WASM component plugins.
pub fn build_plugins(repo_root: &Utf8Path, options: &BuildOptions) -> Result<()> {
    let crates_dir = repo_root.join("crates");

    // Load registry to find grammars with generate-component: true
    let registry = CrateRegistry::load(&crates_dir)
        .map_err(|e| miette::miette!("failed to load crate registry: {}", e))?;

    // Get grammars to build
    let grammars: Vec<String> = if options.grammars.is_empty() {
        // Find all grammars with generate-component: true
        registry
            .all_grammars()
            .filter(|(_, _, grammar)| grammar.generate_component())
            .map(|(_, _, grammar)| grammar.id().to_string())
            .collect()
    } else {
        options.grammars.clone()
    };

    if grammars.is_empty() {
        println!(
            "{} No grammars have generate-component enabled",
            "○".dimmed()
        );
        println!(
            "  Add {} to a grammar's arborium.kdl to enable",
            "generate-component #true".cyan()
        );
        return Ok(());
    }

    println!(
        "{} Building {} plugin(s): {}",
        "●".cyan(),
        grammars.len(),
        grammars.join(", ")
    );

    // Ensure output directory exists
    let output_dir = repo_root.join(&options.output_dir);
    std::fs::create_dir_all(&output_dir)
        .into_diagnostic()
        .context("failed to create output directory")?;

    let cargo_component = Tool::CargoComponent
        .find()
        .into_diagnostic()
        .context("cargo-component not found")?;

    let jco = if options.transpile {
        Some(
            Tool::Jco
                .find()
                .into_diagnostic()
                .context("jco not found")?,
        )
    } else {
        None
    };

    // Track timings if profiling is enabled
    let mut timings: Vec<PluginTiming> = Vec::new();

    for grammar in &grammars {
        let grammar_start = Instant::now();
        println!("{} {}", "Building plugin:".cyan().bold(), grammar);

        let plugin_crate = format!("arborium-{}-plugin", grammar);
        let plugin_dir = repo_root.join("crates").join(&plugin_crate);

        // Check if plugin crate exists
        if !plugin_dir.exists() {
            println!(
                "  {} Plugin crate {} does not exist, creating...",
                "⚠".yellow(),
                plugin_crate
            );
            create_plugin_crate(repo_root, grammar)?;
        }

        // Build with cargo component from the plugin crate directory
        // (plugin crates are excluded from workspace, so we build from their directory)
        let cargo_start = Instant::now();
        let status = cargo_component
            .command()
            .args(["build", "--release"])
            .current_dir(&plugin_dir)
            .status()
            .into_diagnostic()
            .context("failed to run cargo-component")?;
        let cargo_component_ms = cargo_start.elapsed().as_millis() as u64;

        if !status.success() {
            miette::bail!("cargo-component build failed for {}", grammar);
        }

        // Find the built wasm file (in the plugin crate's own target directory)
        let wasm_file = plugin_dir
            .join("target/wasm32-wasip1/release")
            .join(format!("{}.wasm", plugin_crate.replace('-', "_")));

        if !wasm_file.exists() {
            miette::bail!("expected wasm file not found: {}", wasm_file);
        }

        // Copy to output directory
        let plugin_output = output_dir.join(grammar);
        std::fs::create_dir_all(&plugin_output)
            .into_diagnostic()
            .context("failed to create plugin output directory")?;

        let dest_wasm = plugin_output.join("grammar.wasm");
        std::fs::copy(&wasm_file, &dest_wasm)
            .into_diagnostic()
            .context("failed to copy wasm file")?;

        // Transpile with jco if enabled
        let mut transpile_ms = 0u64;
        if let Some(ref jco) = jco {
            println!("  {} Transpiling with jco...", "→".blue());
            let transpile_start = Instant::now();
            let status = jco
                .command()
                .args([
                    "transpile",
                    dest_wasm.as_str(),
                    "--instantiation",
                    "async",
                    "--quiet",
                    "-o",
                    plugin_output.as_str(),
                ])
                .status()
                .into_diagnostic()
                .context("failed to run jco")?;
            transpile_ms = transpile_start.elapsed().as_millis() as u64;

            if !status.success() {
                miette::bail!("jco transpile failed for {}", grammar);
            }

            // Calculate total wasm bundle size
            let total_wasm_size: u64 = std::fs::read_dir(&plugin_output)
                .into_diagnostic()?
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "wasm"))
                .filter_map(|e| e.metadata().ok())
                .map(|m| m.len())
                .sum();

            println!(
                "  {} Transpiled ({})",
                "✓".green(),
                format_size(total_wasm_size as usize)
            );
        }

        let build_ms = grammar_start.elapsed().as_millis() as u64;

        if options.profile {
            println!(
                "  {} Timing: {}ms total (cargo-component: {}ms, transpile: {}ms)",
                "⏱".dimmed(),
                build_ms,
                cargo_component_ms,
                transpile_ms
            );
        }

        timings.push(PluginTiming {
            grammar: grammar.clone(),
            build_ms,
            cargo_component_ms,
            transpile_ms,
        });

        println!("  {} Built {}", "✓".green(), grammar);
    }

    // Run deduplication if we transpiled
    if options.transpile && grammars.len() > 1 {
        println!("\n{} Deduplicating shared WASM modules...", "→".blue());
        deduplicate_wasm_modules(&output_dir)?;
    }

    // Optimize and compress all wasm files
    if options.transpile {
        println!("\n{} Optimizing and compressing WASM files...", "→".blue());
        let compression_config = CompressionConfig::load(repo_root)
            .map_err(|e| miette::miette!("failed to load compression.kdl: {}", e))?;
        optimize_and_compress_wasm(&output_dir, &compression_config)?;
    }

    // Save timings if profiling is enabled
    if options.profile {
        let timings_path = repo_root.join("plugin-timings.json");
        let plugin_timings = PluginTimings {
            recorded_at: Utc::now().to_rfc3339(),
            timings,
        };
        plugin_timings.save(&timings_path)?;
        println!("\n{} Saved timings to {}", "✓".green(), timings_path.cyan());

        // Print summary
        let total_ms: u64 = plugin_timings.timings.iter().map(|t| t.build_ms).sum();
        println!("\n{} Build time summary:", "●".cyan());
        for timing in &plugin_timings.timings {
            let pct = (timing.build_ms as f64 / total_ms as f64) * 100.0;
            println!(
                "  {} {}: {}ms ({:.1}%)",
                "→".dimmed(),
                timing.grammar,
                timing.build_ms,
                pct
            );
        }
        println!("  {} Total: {}ms", "=".dimmed(), total_ms);
    }

    Ok(())
}

/// Create a new plugin crate for a grammar.
fn create_plugin_crate(repo_root: &Utf8Path, grammar: &str) -> Result<()> {
    let grammar_crate = format!("arborium-{}", grammar);
    let plugin_crate = format!("arborium-{}-plugin", grammar);
    let plugin_dir = repo_root.join("crates").join(&plugin_crate);

    // Create directories
    std::fs::create_dir_all(plugin_dir.join("src"))
        .into_diagnostic()
        .context("failed to create plugin crate directory")?;

    // Create Cargo.toml
    let cargo_toml = format!(
        r#"[package]
name = "{plugin_crate}"
version = "0.1.0"
edition = "2024"
description = "{grammar} grammar plugin for arborium"
license = "MIT"
repository = "https://github.com/bearcove/arborium"

[lib]
crate-type = ["cdylib"]

[dependencies]
arborium-plugin-runtime = {{ path = "../arborium-plugin-runtime" }}
arborium-wire = {{ path = "../arborium-wire" }}
{grammar_crate} = {{ path = "../{grammar_crate}" }}
wit-bindgen = "0.36"

[package.metadata.component]
package = "arborium:grammar"

[package.metadata.component.target]
world = "grammar-plugin"
path = "../../wit/grammar.wit"
"#
    );
    std::fs::write(plugin_dir.join("Cargo.toml"), cargo_toml)
        .into_diagnostic()
        .context("failed to write Cargo.toml")?;

    // Create lib.rs
    let lib_rs = format!(
        r#"//! {grammar} grammar plugin for arborium.
#![allow(unsafe_op_in_unsafe_fn)]

wit_bindgen::generate!({{
    world: "grammar-plugin",
    path: "../../wit/grammar.wit",
}});

use arborium_plugin_runtime::{{HighlightConfig, PluginRuntime}};
use arborium_wire::Edit as WireEdit;
use std::cell::RefCell;

// Import the generated types
use arborium::grammar::types::{{Edit, Injection, ParseError, ParseResult, Span}};

thread_local! {{
    static RUNTIME: RefCell<Option<PluginRuntime>> = const {{ RefCell::new(None) }};
}}

fn get_or_init_runtime() -> &'static RefCell<Option<PluginRuntime>> {{
    RUNTIME.with(|r| {{
        let mut runtime = r.borrow_mut();
        if runtime.is_none() {{
            let config = HighlightConfig::new(
                {grammar_crate}::language(),
                {grammar_crate}::HIGHLIGHTS_QUERY,
                {grammar_crate}::INJECTIONS_QUERY,
                {grammar_crate}::LOCALS_QUERY,
            )
            .expect("failed to create highlight config");
            *runtime = Some(PluginRuntime::new(config));
        }}
        // SAFETY: We're returning a reference to thread-local storage which lives
        // for the duration of the WASM instance.
        unsafe {{ &*(r as *const _) }}
    }})
}}

struct PluginImpl;

impl exports::arborium::grammar::plugin::Guest for PluginImpl {{
    fn language_id() -> String {{
        "{grammar}".to_string()
    }}

    fn injection_languages() -> Vec<String> {{
        // TODO: Parse injection queries to determine which languages are injected
        Vec::new()
    }}

    fn create_session() -> u32 {{
        get_or_init_runtime()
            .borrow_mut()
            .as_mut()
            .expect("runtime not initialized")
            .create_session()
    }}

    fn free_session(session: u32) {{
        get_or_init_runtime()
            .borrow_mut()
            .as_mut()
            .expect("runtime not initialized")
            .free_session(session);
    }}

    fn set_text(session: u32, text: String) {{
        get_or_init_runtime()
            .borrow_mut()
            .as_mut()
            .expect("runtime not initialized")
            .set_text(session, &text);
    }}

    fn apply_edit(session: u32, text: String, edit: Edit) {{
        let wire_edit = WireEdit {{
            start_byte: edit.start_byte,
            old_end_byte: edit.old_end_byte,
            new_end_byte: edit.new_end_byte,
            start_row: edit.start_row,
            start_col: edit.start_col,
            old_end_row: edit.old_end_row,
            old_end_col: edit.old_end_col,
            new_end_row: edit.new_end_row,
            new_end_col: edit.new_end_col,
        }};
        get_or_init_runtime()
            .borrow_mut()
            .as_mut()
            .expect("runtime not initialized")
            .apply_edit(session, &text, &wire_edit);
    }}

    fn parse(session: u32) -> Result<ParseResult, ParseError> {{
        let result = get_or_init_runtime()
            .borrow_mut()
            .as_mut()
            .expect("runtime not initialized")
            .parse(session);

        match result {{
            Ok(r) => Ok(ParseResult {{
                spans: r
                    .spans
                    .into_iter()
                    .map(|s| Span {{
                        start: s.start,
                        end: s.end,
                        capture: s.capture,
                    }})
                    .collect(),
                injections: r
                    .injections
                    .into_iter()
                    .map(|i| Injection {{
                        start: i.start,
                        end: i.end,
                        language: i.language,
                        include_children: i.include_children,
                    }})
                    .collect(),
            }}),
            Err(e) => Err(ParseError {{
                message: e.message,
            }}),
        }}
    }}

    fn cancel(session: u32) {{
        get_or_init_runtime()
            .borrow_mut()
            .as_mut()
            .expect("runtime not initialized")
            .cancel(session);
    }}
}}

export!(PluginImpl);
"#,
        grammar_crate = grammar_crate.replace('-', "_")
    );
    std::fs::write(plugin_dir.join("src/lib.rs"), lib_rs)
        .into_diagnostic()
        .context("failed to write lib.rs")?;

    // Add to workspace Cargo.toml
    add_to_workspace(repo_root, &plugin_crate)?;

    println!(
        "  {} Created plugin crate {}",
        "✓".green(),
        plugin_crate.cyan()
    );
    Ok(())
}

/// Add a crate to the workspace members list.
fn add_to_workspace(repo_root: &Utf8Path, crate_name: &str) -> Result<()> {
    let cargo_toml_path = repo_root.join("Cargo.toml");
    let content = std::fs::read_to_string(&cargo_toml_path)
        .into_diagnostic()
        .context("failed to read workspace Cargo.toml")?;

    let member_entry = format!("\"crates/{}\"", crate_name);
    if content.contains(&member_entry) {
        return Ok(()); // Already in workspace
    }

    // Find the end of the members array and insert before it
    // This is a simple string manipulation - a more robust solution would use toml crate
    if let Some(pos) = content.find("]\n\n[workspace.package]") {
        let mut new_content = content[..pos].to_string();
        new_content.push_str(&format!("    {},\n", member_entry));
        new_content.push_str(&content[pos..]);
        std::fs::write(&cargo_toml_path, new_content)
            .into_diagnostic()
            .context("failed to write workspace Cargo.toml")?;
    }

    Ok(())
}

/// Clean plugin build artifacts.
pub fn clean_plugins(repo_root: &Utf8Path, output_dir: &str) -> Result<()> {
    let output_path = repo_root.join(output_dir);
    if output_path.exists() {
        std::fs::remove_dir_all(&output_path)
            .into_diagnostic()
            .context("failed to remove output directory")?;
        println!("{} Removed {}", "✓".green(), output_path);
    } else {
        println!("{} Nothing to clean", "○".dimmed());
    }
    Ok(())
}

fn format_size(bytes: usize) -> String {
    if bytes >= 1_000_000 {
        format!("{:.2} MB", bytes as f64 / 1_000_000.0)
    } else if bytes >= 1000 {
        format!("{:.2} KB", bytes as f64 / 1000.0)
    } else {
        format!("{} B", bytes)
    }
}

/// Deduplicate identical WASM shim modules across plugins.
///
/// jco generates identical shim modules (core2.wasm, core3.wasm, core4.wasm) for each
/// plugin. This function moves duplicates to a shared directory and updates the JS
/// files to reference the shared location.
fn deduplicate_wasm_modules(plugins_dir: &Utf8Path) -> Result<()> {
    use std::collections::HashMap;

    let shared_dir = plugins_dir.parent().unwrap().join("shared");
    std::fs::create_dir_all(&shared_dir)
        .into_diagnostic()
        .context("failed to create shared directory")?;

    // Find all .wasm files and group by hash
    let mut hash_to_files: HashMap<String, Vec<Utf8PathBuf>> = HashMap::new();

    for entry in std::fs::read_dir(plugins_dir).into_diagnostic()? {
        let entry = entry.into_diagnostic()?;
        let plugin_path = Utf8PathBuf::from_path_buf(entry.path()).ok();
        let Some(plugin_path) = plugin_path else {
            continue;
        };
        if !plugin_path.is_dir() {
            continue;
        }

        for wasm_entry in std::fs::read_dir(&plugin_path).into_diagnostic()? {
            let wasm_entry = wasm_entry.into_diagnostic()?;
            let wasm_path = Utf8PathBuf::from_path_buf(wasm_entry.path()).ok();
            let Some(wasm_path) = wasm_path else {
                continue;
            };

            // Skip non-wasm files and the main grammar.core.wasm (unique per language)
            let name = wasm_path.file_name().unwrap_or("");
            if !name.ends_with(".wasm") || name.ends_with(".core.wasm") {
                continue;
            }

            // Hash the file
            let content = std::fs::read(&wasm_path).into_diagnostic()?;
            let hash = blake3::hash(&content).to_hex()[..16].to_string();

            hash_to_files.entry(hash).or_default().push(wasm_path);
        }
    }

    // Process duplicates
    let mut saved_bytes = 0usize;
    let mut deduped_count = 0usize;

    for (hash, files) in hash_to_files {
        // Only dedupe if there are multiple copies
        if files.len() < 2 {
            continue;
        }

        // Get a canonical name (e.g., "shim.core2.wasm" from "grammar.core2.wasm")
        let original_name = files[0].file_name().unwrap();
        let shared_name = if let Some(rest) = original_name.strip_prefix("grammar.") {
            format!("shim.{}", rest)
        } else {
            format!("shim.{}.wasm", &hash[..8])
        };

        let shared_path = shared_dir.join(&shared_name);
        let file_size = std::fs::metadata(&files[0]).into_diagnostic()?.len() as usize;

        // Copy one to shared location
        std::fs::copy(&files[0], &shared_path)
            .into_diagnostic()
            .context("failed to copy shared wasm")?;

        // Calculate savings
        saved_bytes += (files.len() - 1) * file_size;
        deduped_count += files.len() - 1;

        // Update each plugin's JS to reference shared path and remove duplicate
        for wasm_path in &files {
            let plugin_dir = wasm_path.parent().unwrap();
            let js_file = plugin_dir.join("grammar.js");
            let wasm_basename = wasm_path.file_name().unwrap();

            if js_file.exists() {
                let content = std::fs::read_to_string(&js_file).into_diagnostic()?;
                let new_content = content.replace(
                    &format!("getCoreModule('{}')", wasm_basename),
                    &format!("getCoreModule('../shared/{}')", shared_name),
                );
                std::fs::write(&js_file, new_content).into_diagnostic()?;
            }

            // Remove the duplicate
            std::fs::remove_file(wasm_path).into_diagnostic()?;
        }

        println!(
            "  {} {} ({} bytes, {} copies)",
            "→".dimmed(),
            shared_name,
            file_size,
            files.len()
        );
    }

    if deduped_count > 0 {
        println!(
            "  {} Removed {} duplicates, saved {}",
            "✓".green(),
            deduped_count,
            format_size(saved_bytes)
        );
    } else {
        println!("  {} No duplicates found", "○".dimmed());
    }

    Ok(())
}

/// Optimize WASM files with wasm-opt and create compressed versions.
///
/// For each .wasm file:
/// 1. Run wasm-opt -Oz to optimize for size
/// 2. Create .wasm.br (brotli), .wasm.gz (gzip), and .wasm.zst (zstd) versions
fn optimize_and_compress_wasm(plugins_dir: &Utf8Path, config: &CompressionConfig) -> Result<()> {
    use std::io::Write;

    // Find all wasm files (in plugins and shared directories)
    let mut wasm_files = Vec::new();

    // Check plugins directory
    for entry in std::fs::read_dir(plugins_dir).into_diagnostic()? {
        let entry = entry.into_diagnostic()?;
        let path = Utf8PathBuf::try_from(entry.path()).into_diagnostic()?;
        if path.is_dir() {
            for file in std::fs::read_dir(&path).into_diagnostic()? {
                let file = file.into_diagnostic()?;
                let file_path = Utf8PathBuf::try_from(file.path()).into_diagnostic()?;
                // Only optimize .core*.wasm files (core wasm from jco transpile)
                // Skip grammar.wasm which is the component model wasm
                let file_name = file_path.file_name().unwrap_or("");
                if file_path.extension() == Some("wasm") && file_name.contains(".core") {
                    wasm_files.push(file_path);
                }
            }
        }
    }

    // Check shared directory
    let shared_dir = plugins_dir.parent().unwrap().join("shared");
    if shared_dir.exists() {
        for entry in std::fs::read_dir(&shared_dir).into_diagnostic()? {
            let entry = entry.into_diagnostic()?;
            let path = Utf8PathBuf::try_from(entry.path()).into_diagnostic()?;
            if path.extension() == Some("wasm") {
                wasm_files.push(path);
            }
        }
    }

    if wasm_files.is_empty() {
        println!("  {} No WASM files found", "○".dimmed());
        return Ok(());
    }

    use rayon::prelude::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    let total_files = wasm_files.len();
    let processed = AtomicUsize::new(0);
    let total_original = AtomicUsize::new(0);
    let total_optimized = AtomicUsize::new(0);
    let total_br = AtomicUsize::new(0);
    let total_gz = AtomicUsize::new(0);
    let total_zst = AtomicUsize::new(0);

    // Process files in parallel (up to 4 at a time)
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(4)
        .build()
        .into_diagnostic()?;

    let results: Vec<Result<()>> = pool.install(|| {
        wasm_files
            .par_iter()
            .map(|wasm_path| {
                let original_size = std::fs::metadata(wasm_path)
                    .map_err(|e| miette::miette!("failed to read {}: {}", wasm_path, e))?
                    .len() as usize;
                total_original.fetch_add(original_size, Ordering::Relaxed);

                // Optimize with wasm-opt
                let optimized_path = wasm_path.with_extension("wasm.opt");
                wasm_opt::OptimizationOptions::new_optimize_for_size()
                    .run(wasm_path.as_std_path(), optimized_path.as_std_path())
                    .map_err(|e| miette::miette!("wasm-opt failed for {}: {}", wasm_path, e))?;

                // Replace original with optimized
                std::fs::rename(&optimized_path, wasm_path)
                    .map_err(|e| miette::miette!("failed to rename {}: {}", wasm_path, e))?;
                let optimized_size = std::fs::metadata(wasm_path)
                    .map_err(|e| miette::miette!("failed to read {}: {}", wasm_path, e))?
                    .len() as usize;
                total_optimized.fetch_add(optimized_size, Ordering::Relaxed);

                // Read optimized wasm
                let wasm_data = std::fs::read(wasm_path)
                    .map_err(|e| miette::miette!("failed to read {}: {}", wasm_path, e))?;

                // Create brotli compressed version
                let br_path = format!("{}.br", wasm_path);
                let mut br_encoder = brotli::CompressorWriter::new(
                    std::fs::File::create(&br_path)
                        .map_err(|e| miette::miette!("failed to create {}: {}", br_path, e))?,
                    4096,
                    config.brotli_quality(),
                    config.brotli_window(),
                );
                br_encoder
                    .write_all(&wasm_data)
                    .map_err(|e| miette::miette!("failed to write {}: {}", br_path, e))?;
                drop(br_encoder);
                total_br.fetch_add(
                    std::fs::metadata(&br_path)
                        .map_err(|e| miette::miette!("failed to read {}: {}", br_path, e))?
                        .len() as usize,
                    Ordering::Relaxed,
                );

                // Create gzip compressed version
                let gz_path = format!("{}.gz", wasm_path);
                if config.gzip_use_zopfli() {
                    // Use zopfli for best compression
                    let options = zopfli::Options {
                        iteration_count: std::num::NonZeroU64::new(config.gzip_iterations() as u64)
                            .unwrap_or(std::num::NonZeroU64::new(15).unwrap()),
                        ..Default::default()
                    };
                    let mut gz_data = Vec::new();
                    zopfli::compress(options, zopfli::Format::Gzip, &wasm_data[..], &mut gz_data)
                        .map_err(|e| miette::miette!("zopfli failed for {}: {}", wasm_path, e))?;
                    std::fs::write(&gz_path, gz_data)
                        .map_err(|e| miette::miette!("failed to write {}: {}", gz_path, e))?;
                } else {
                    // Use flate2 for fast compression
                    let gz_file = std::fs::File::create(&gz_path)
                        .map_err(|e| miette::miette!("failed to create {}: {}", gz_path, e))?;
                    let mut gz_encoder = flate2::write::GzEncoder::new(
                        gz_file,
                        flate2::Compression::new(config.gzip_level()),
                    );
                    gz_encoder
                        .write_all(&wasm_data)
                        .map_err(|e| miette::miette!("failed to write {}: {}", gz_path, e))?;
                    gz_encoder
                        .finish()
                        .map_err(|e| miette::miette!("failed to finish {}: {}", gz_path, e))?;
                }
                total_gz.fetch_add(
                    std::fs::metadata(&gz_path)
                        .map_err(|e| miette::miette!("failed to read {}: {}", gz_path, e))?
                        .len() as usize,
                    Ordering::Relaxed,
                );

                // Create zstd compressed version
                let zst_path = format!("{}.zst", wasm_path);
                let zst_data = zstd::encode_all(&wasm_data[..], config.zstd_level())
                    .map_err(|e| miette::miette!("zstd failed for {}: {}", wasm_path, e))?;
                std::fs::write(&zst_path, zst_data)
                    .map_err(|e| miette::miette!("failed to write {}: {}", zst_path, e))?;
                total_zst.fetch_add(
                    std::fs::metadata(&zst_path)
                        .map_err(|e| miette::miette!("failed to read {}: {}", zst_path, e))?
                        .len() as usize,
                    Ordering::Relaxed,
                );

                let done = processed.fetch_add(1, Ordering::Relaxed) + 1;
                println!(
                    "  {} [{}/{}] {}",
                    "→".dimmed(),
                    done,
                    total_files,
                    wasm_path.file_name().unwrap_or("?")
                );

                Ok(())
            })
            .collect()
    });

    // Check for errors
    for result in results {
        result?;
    }

    let total_original = total_original.load(Ordering::Relaxed);
    let total_optimized = total_optimized.load(Ordering::Relaxed);
    let total_br = total_br.load(Ordering::Relaxed);
    let total_gz = total_gz.load(Ordering::Relaxed);
    let total_zst = total_zst.load(Ordering::Relaxed);

    println!("  {} Processed {} files:", "✓".green(), total_files);
    println!(
        "      Original:  {} → Optimized: {} ({:.1}% reduction)",
        format_size(total_original),
        format_size(total_optimized),
        (1.0 - total_optimized as f64 / total_original as f64) * 100.0
    );
    println!(
        "      Brotli:    {} ({:.1}% of optimized)",
        format_size(total_br),
        total_br as f64 / total_optimized as f64 * 100.0
    );
    println!(
        "      Gzip:      {} ({:.1}% of optimized)",
        format_size(total_gz),
        total_gz as f64 / total_optimized as f64 * 100.0
    );
    println!(
        "      Zstd:      {} ({:.1}% of optimized)",
        format_size(total_zst),
        total_zst as f64 / total_optimized as f64 * 100.0
    );

    Ok(())
}

// =============================================================================
// Plugin grouping for CI parallelization
// =============================================================================

/// A group of plugins to build together.
#[derive(Debug, Clone, facet::Facet)]
#[facet(rename_all = "snake_case")]
pub struct PluginGroup {
    /// Group index (0-based)
    pub index: usize,
    /// Grammars in this group
    pub grammars: Vec<String>,
    /// Total estimated build time for this group in milliseconds
    pub total_ms: u64,
}

/// Result of grouping plugins for parallel builds.
#[derive(Debug, Clone, facet::Facet)]
#[facet(rename_all = "snake_case")]
pub struct PluginGroups {
    /// The groups
    pub groups: Vec<PluginGroup>,
    /// Maximum group time (determines total CI time)
    pub max_group_ms: u64,
    /// Theoretical minimum time if perfectly balanced
    pub ideal_per_group_ms: u64,
    /// Efficiency: ideal_per_group_ms / max_group_ms (1.0 = perfect)
    pub efficiency: f64,
}

impl PluginGroups {
    /// Compute balanced groups using a greedy bin-packing algorithm.
    ///
    /// Uses "Longest Processing Time First" (LPT) algorithm:
    /// 1. Sort plugins by build time (longest first)
    /// 2. For each plugin, assign to the group with the smallest total time
    ///
    /// This is a simple 4/3-approximation algorithm that works well in practice.
    pub fn from_timings(timings: &PluginTimings, num_groups: usize) -> Self {
        let num_groups = num_groups.max(1);

        // Sort by build time, descending
        let mut sorted_timings: Vec<_> = timings.timings.iter().collect();
        sorted_timings.sort_by(|a, b| b.build_ms.cmp(&a.build_ms));

        // Initialize groups
        let mut groups: Vec<PluginGroup> = (0..num_groups)
            .map(|i| PluginGroup {
                index: i,
                grammars: Vec::new(),
                total_ms: 0,
            })
            .collect();

        // Greedy assignment: always add to the group with smallest total time
        for timing in sorted_timings {
            // Find group with minimum total time
            let min_group = groups
                .iter_mut()
                .min_by_key(|g| g.total_ms)
                .expect("at least one group exists");

            min_group.grammars.push(timing.grammar.clone());
            min_group.total_ms += timing.build_ms;
        }

        // Remove empty groups (if num_groups > num_plugins)
        groups.retain(|g| !g.grammars.is_empty());

        // Renumber groups after filtering
        for (i, group) in groups.iter_mut().enumerate() {
            group.index = i;
        }

        // Calculate statistics
        let total_ms: u64 = timings.timings.iter().map(|t| t.build_ms).sum();
        let max_group_ms = groups.iter().map(|g| g.total_ms).max().unwrap_or(0);
        let ideal_per_group_ms = total_ms / groups.len().max(1) as u64;
        let efficiency = if max_group_ms > 0 {
            ideal_per_group_ms as f64 / max_group_ms as f64
        } else {
            1.0
        };

        Self {
            groups,
            max_group_ms,
            ideal_per_group_ms,
            efficiency,
        }
    }
}

/// Show plugin build groups based on timings.
pub fn show_groups(timings_path: &Utf8Path, num_groups: usize) -> Result<()> {
    let timings = PluginTimings::load(timings_path)?;

    println!(
        "{} Loaded timings from {} (recorded: {})",
        "●".cyan(),
        timings_path.cyan(),
        timings.recorded_at.dimmed()
    );

    let groups = PluginGroups::from_timings(&timings, num_groups);

    println!(
        "\n{} Plugin groups ({} groups, {:.1}% efficiency):",
        "●".cyan(),
        groups.groups.len(),
        groups.efficiency * 100.0
    );

    for group in &groups.groups {
        let time_str = format_duration_ms(group.total_ms);
        println!(
            "\n  {} Group {} ({}):",
            "→".blue(),
            group.index,
            time_str.yellow()
        );
        for grammar in &group.grammars {
            // Find the timing for this grammar
            let timing = timings.timings.iter().find(|t| &t.grammar == grammar);
            if let Some(t) = timing {
                println!(
                    "      {} {} ({})",
                    "•".dimmed(),
                    grammar,
                    format_duration_ms(t.build_ms)
                );
            } else {
                println!("      {} {}", "•".dimmed(), grammar);
            }
        }
    }

    println!("\n{} Summary:", "●".cyan());
    println!(
        "  Max group time: {} (determines CI time)",
        format_duration_ms(groups.max_group_ms).yellow()
    );
    println!(
        "  Ideal per group: {}",
        format_duration_ms(groups.ideal_per_group_ms)
    );

    // Output as JSON for machine consumption
    println!("\n{} JSON output:", "●".cyan());
    let json = facet_json::to_string_pretty(&groups);
    println!("{}", json);

    Ok(())
}

/// Format milliseconds as human-readable duration.
fn format_duration_ms(ms: u64) -> String {
    if ms >= 60_000 {
        let minutes = ms / 60_000;
        let seconds = (ms % 60_000) / 1000;
        format!("{}m {}s", minutes, seconds)
    } else if ms >= 1000 {
        format!("{:.1}s", ms as f64 / 1000.0)
    } else {
        format!("{}ms", ms)
    }
}

// =============================================================================
// npm package generation
// =============================================================================

/// Generate npm package.json files for all grammar plugins.
///
/// This creates a package.json in each plugin directory under output_dir,
/// making them publishable to npm as `@arborium/<language>` packages.
pub fn generate_npm_packages(
    repo_root: &Utf8Path,
    output_dir: &Utf8Path,
    version: &str,
) -> Result<()> {
    let output_path = if output_dir.is_absolute() {
        output_dir.to_owned()
    } else {
        repo_root.join(output_dir)
    };

    if !output_path.exists() {
        miette::bail!(
            "Output directory does not exist: {}\nRun `arborium-xtask plugins build` first.",
            output_path
        );
    }

    println!(
        "{} Generating npm packages (version {})...",
        "→".blue(),
        version.yellow()
    );

    let mut count = 0;

    // Iterate over plugin directories
    for entry in std::fs::read_dir(&output_path).into_diagnostic()? {
        let entry = entry.into_diagnostic()?;
        let plugin_path = Utf8PathBuf::from_path_buf(entry.path()).ok();
        let Some(plugin_path) = plugin_path else {
            continue;
        };

        if !plugin_path.is_dir() {
            continue;
        }

        let language = plugin_path.file_name().unwrap_or("");

        // Check that grammar.js exists (indicates a transpiled plugin)
        let grammar_js = plugin_path.join("grammar.js");
        if !grammar_js.exists() {
            continue;
        }

        // Generate package.json
        let package_json = generate_package_json(language, version);
        let package_json_path = plugin_path.join("package.json");

        std::fs::write(&package_json_path, package_json)
            .into_diagnostic()
            .context(format!("failed to write {}", package_json_path))?;

        println!("  {} @arborium/{}", "✓".green(), language);
        count += 1;
    }

    println!(
        "\n{} Generated {} package.json files",
        "✓".green(),
        count.to_string().yellow()
    );

    Ok(())
}

/// Generate a package.json for a grammar plugin.
fn generate_package_json(language: &str, version: &str) -> String {
    // Escape any special characters in the language name for JSON
    let safe_language = language.replace('\\', "\\\\").replace('"', "\\\"");

    format!(
        r#"{{
  "name": "@arborium/{language}",
  "version": "{version}",
  "description": "Arborium syntax highlighting grammar for {language}",
  "type": "module",
  "main": "./grammar.js",
  "module": "./grammar.js",
  "types": "./grammar.d.ts",
  "exports": {{
    ".": {{
      "import": "./grammar.js",
      "types": "./grammar.d.ts"
    }},
    "./grammar.js": "./grammar.js",
    "./grammar.core.wasm": "./grammar.core.wasm"
  }},
  "files": [
    "grammar.js",
    "grammar.d.ts",
    "grammar.core.wasm",
    "interfaces"
  ],
  "keywords": [
    "arborium",
    "syntax-highlighting",
    "tree-sitter",
    "{language}",
    "wasm"
  ],
  "author": "Amos Wenger <amos@bearcove.net>",
  "license": "MIT OR Apache-2.0",
  "repository": {{
    "type": "git",
    "url": "https://github.com/bearcove/arborium.git"
  }},
  "homepage": "https://github.com/bearcove/arborium",
  "bugs": {{
    "url": "https://github.com/bearcove/arborium/issues"
  }},
  "publishConfig": {{
    "access": "public"
  }}
}}
"#,
        language = safe_language,
        version = version
    )
}
