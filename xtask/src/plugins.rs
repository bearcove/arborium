//! WASM plugin build system.
//!
//! This module handles building grammar plugins as WASM components
//! and transpiling them to JavaScript for browser usage.

use camino::{Utf8Path, Utf8PathBuf};
use miette::{Context, IntoDiagnostic, Result};
use owo_colors::OwoColorize;

use crate::tool::Tool;
use crate::types::CrateRegistry;

/// Build options for plugins.
pub struct BuildOptions {
    /// Specific grammars to build (empty = all)
    pub grammars: Vec<String>,
    /// Output directory for built plugins
    pub output_dir: Utf8PathBuf,
    /// Whether to run jco transpile after building
    pub transpile: bool,
    /// Size budget in bytes (warn if exceeded)
    pub size_budget: usize,
}

impl Default for BuildOptions {
    fn default() -> Self {
        Self {
            grammars: Vec::new(),
            output_dir: Utf8PathBuf::from("dist/plugins"),
            transpile: true,
            size_budget: 1_500_000, // 1.5 MB
        }
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

    for grammar in &grammars {
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
        let status = cargo_component
            .command()
            .args(["build", "--release"])
            .current_dir(&plugin_dir)
            .status()
            .into_diagnostic()
            .context("failed to run cargo-component")?;

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

        // Check size
        let size = std::fs::metadata(&wasm_file).into_diagnostic()?.len() as usize;
        if size > options.size_budget {
            println!(
                "  {} Size {} exceeds budget {} ({:.1}x)",
                "⚠".yellow(),
                format_size(size),
                format_size(options.size_budget),
                size as f64 / options.size_budget as f64
            );
        } else {
            println!("  {} Size: {}", "✓".green(), format_size(size));
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
        if let Some(ref jco) = jco {
            println!("  {} Transpiling with jco...", "→".blue());
            let status = jco
                .command()
                .args([
                    "transpile",
                    dest_wasm.as_str(),
                    "--instantiation",
                    "async",
                    "-o",
                    plugin_output.as_str(),
                ])
                .status()
                .into_diagnostic()
                .context("failed to run jco")?;

            if !status.success() {
                miette::bail!("jco transpile failed for {}", grammar);
            }
            println!("  {} Transpiled successfully", "✓".green());
        }

        println!("  {} Built {}", "✓".green(), grammar);
    }

    // Run deduplication if we transpiled
    if options.transpile && grammars.len() > 1 {
        println!("\n{} Deduplicating shared WASM modules...", "→".blue());
        deduplicate_wasm_modules(&output_dir)?;
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
