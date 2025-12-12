use std::io::{BufRead, BufReader, Write};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

use owo_colors::OwoColorize;
use rand::seq::SliceRandom;

use camino::{Utf8Path, Utf8PathBuf};
use chrono::Utc;
use miette::{Context, IntoDiagnostic, Result};
use rayon::prelude::*;
use sailfish::TemplateSimple;

use crate::tool::Tool;
use crate::types::CrateRegistry;
use crate::version_store;

/// Ensure nightly toolchain and wasm32-unknown-unknown target are installed
fn ensure_rust_nightly_with_wasm_target() -> Result<()> {
    // Check if nightly toolchain is installed
    let output = Command::new("rustup")
        .args(["toolchain", "list"])
        .output()
        .into_diagnostic()
        .context("failed to run rustup toolchain list")?;

    let toolchains = String::from_utf8_lossy(&output.stdout);
    let has_nightly = toolchains.lines().any(|line| line.contains("nightly"));

    if !has_nightly {
        println!("{} Installing nightly toolchain...", "●".cyan());
        let status = Command::new("rustup")
            .args(["toolchain", "install", "nightly"])
            .status()
            .into_diagnostic()
            .context("failed to install nightly toolchain")?;

        if !status.success() {
            miette::bail!("failed to install nightly toolchain");
        }
        println!("{} Nightly toolchain installed", "✓".green());
    }

    // Check if wasm32-unknown-unknown target is installed for nightly
    let output = Command::new("rustup")
        .args(["+nightly", "target", "list", "--installed"])
        .output()
        .into_diagnostic()
        .context("failed to check installed targets")?;

    let targets = String::from_utf8_lossy(&output.stdout);
    let has_wasm_target = targets
        .lines()
        .any(|line| line.trim() == "wasm32-unknown-unknown");

    if !has_wasm_target {
        println!(
            "{} Installing wasm32-unknown-unknown target for nightly...",
            "●".cyan()
        );
        let status = Command::new("rustup")
            .args(["+nightly", "target", "add", "wasm32-unknown-unknown"])
            .status()
            .into_diagnostic()
            .context("failed to add wasm32-unknown-unknown target")?;

        if !status.success() {
            miette::bail!("failed to add wasm32-unknown-unknown target");
        }
        println!("{} wasm32-unknown-unknown target installed", "✓".green());
    }

    Ok(())
}

/// Thread-safe output printer for parallel builds.
#[derive(Clone)]
struct OutputPrinter {
    mutex: Arc<Mutex<()>>,
}

impl OutputPrinter {
    fn new() -> Self {
        Self {
            mutex: Arc::new(Mutex::new(())),
        }
    }

    fn print_line(&self, grammar: &str, line: &str, is_stderr: bool) {
        let _lock = self.mutex.lock().unwrap();
        let prefix = format!("[{:^18}]", grammar);
        let colored_prefix = if is_stderr {
            prefix.red().to_string()
        } else {
            prefix.blue().to_string()
        };
        if is_stderr {
            eprintln!("{} {}", colored_prefix, line);
            let _ = std::io::stderr().flush();
        } else {
            println!("{} {}", colored_prefix, line);
            let _ = std::io::stdout().flush();
        }
    }
}

/// Run a command and stream its output with prefixed lines.
fn run_streaming(
    mut cmd: Command,
    grammar: &str,
    printer: &OutputPrinter,
) -> std::io::Result<ExitStatus> {
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

    let mut child = cmd.spawn()?;

    let stdout = child.stdout.take().expect("stdout piped");
    let stderr = child.stderr.take().expect("stderr piped");

    let grammar_out = grammar.to_string();
    let grammar_err = grammar.to_string();
    let printer_out = printer.clone();
    let printer_err = printer.clone();

    let stdout_thread = thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            let Ok(line) = line else { break };
            printer_out.print_line(&grammar_out, &line, false);
        }
    });

    let stderr_thread = thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            let Ok(line) = line else { break };
            printer_err.print_line(&grammar_err, &line, true);
        }
    });

    let status = child.wait()?;

    stdout_thread.join().expect("stdout thread panicked");
    stderr_thread.join().expect("stderr thread panicked");

    Ok(status)
}

pub struct BuildOptions {
    pub grammars: Vec<String>,
    pub group: Option<String>,
    pub output_dir: Option<Utf8PathBuf>,
    pub jobs: usize,
}

impl Default for BuildOptions {
    fn default() -> Self {
        Self {
            grammars: Vec::new(),
            group: None,
            output_dir: None,
            jobs: 16,
        }
    }
}

/// A group of plugins to build together (maps to langs/group-* folders).
#[derive(Debug, Clone)]
pub struct PluginGroup {
    /// The group name (e.g., "acorn", "birch")
    pub name: String,
    /// Grammars in this group
    pub grammars: Vec<String>,
}

/// All plugin groups discovered from the filesystem.
#[derive(Debug, Clone)]
pub struct PluginGroups {
    pub groups: Vec<PluginGroup>,
}

impl PluginGroups {
    /// Discover plugin groups from langs/group-* directories.
    pub fn discover(langs_dir: &Utf8Path) -> miette::Result<Self> {
        let mut groups = Vec::new();

        // Read all group-* directories
        let mut group_dirs: Vec<_> = std::fs::read_dir(langs_dir)
            .map_err(|e| miette::miette!("failed to read {}: {}", langs_dir, e))?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                let name = entry.file_name();
                let name = name.to_string_lossy();
                name.starts_with("group-") && entry.path().is_dir()
            })
            .collect();

        // Sort by name for consistent ordering
        group_dirs.sort_by_key(|e| e.file_name());

        for group_entry in group_dirs {
            let group_name = group_entry
                .file_name()
                .to_string_lossy()
                .strip_prefix("group-")
                .unwrap_or_default()
                .to_string();

            let group_path = group_entry.path();
            let mut grammars = Vec::new();

            // Read all grammar directories within this group
            for lang_entry in std::fs::read_dir(&group_path)
                .map_err(|e| miette::miette!("failed to read {:?}: {}", group_path, e))?
            {
                let lang_entry =
                    lang_entry.map_err(|e| miette::miette!("failed to read entry: {}", e))?;
                if lang_entry.path().is_dir() {
                    grammars.push(lang_entry.file_name().to_string_lossy().to_string());
                }
            }

            // Sort grammars for consistent ordering
            grammars.sort();

            if !grammars.is_empty() {
                groups.push(PluginGroup {
                    name: group_name,
                    grammars,
                });
            }
        }

        Ok(Self { groups })
    }
}

#[derive(Debug, Clone, facet::Facet)]
#[facet(rename_all = "snake_case")]
pub struct PluginManifestEntry {
    pub language: String,
    pub package: String,
    pub version: String,
    pub cdn_js: String,
    pub cdn_wasm: String,
    pub local_js: String,
    pub local_wasm: String,
}

#[derive(Debug, Clone, facet::Facet)]
#[facet(rename_all = "snake_case")]
pub struct PluginManifest {
    pub generated_at: String,
    pub entries: Vec<PluginManifestEntry>,
}

pub fn build_plugins(repo_root: &Utf8Path, options: &BuildOptions) -> Result<()> {
    let crates_dir = repo_root.join("crates");
    let version = version_store::read_version(repo_root)?;

    let registry = CrateRegistry::load(&crates_dir)
        .map_err(|e| miette::miette!("failed to load crate registry: {}", e))?;

    let mut grammars: Vec<String> = if !options.grammars.is_empty() {
        options.grammars.clone()
    } else if let Some(ref group) = options.group {
        // Filter by group name (e.g., "birch" matches "group-birch")
        let group_prefix = format!("group-{}", group);
        registry
            .all_grammars()
            .filter(|(state, _, grammar)| {
                grammar.generate_component() && state.crate_path.as_str().contains(&group_prefix)
            })
            .map(|(_, _, grammar)| grammar.id().to_string())
            .collect()
    } else {
        registry
            .all_grammars()
            .filter(|(_, _, grammar)| grammar.generate_component())
            .map(|(_, _, grammar)| grammar.id().to_string())
            .collect()
    };

    // Randomize build order to reduce Cargo.lock contention between plugins in the same group
    grammars.shuffle(&mut rand::rng());

    if grammars.is_empty() {
        println!(
            "{} No grammars have generate-component enabled",
            "○".dimmed()
        );
        return Ok(());
    }

    println!(
        "{} Building {} plugin(s) with {} job(s)",
        "●".cyan(),
        grammars.len(),
        options.jobs
    );

    // Ensure nightly toolchain and wasm32-unknown-unknown target are installed
    println!(
        "{} Checking nightly toolchain and wasm target...",
        "●".cyan()
    );
    ensure_rust_nightly_with_wasm_target()?;

    let wasm_bindgen = Tool::WasmBindgen
        .find()
        .into_diagnostic()
        .context("wasm-bindgen not found")?;

    let wasm_opt = Tool::WasmOpt
        .find()
        .into_diagnostic()
        .context("wasm-opt not found")?;

    let printer = OutputPrinter::new();

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(options.jobs)
        .build()
        .expect("failed to create thread pool");

    pool.install(|| {
        grammars.par_iter().for_each(|grammar| {
            let result = build_single_plugin(
                repo_root,
                &registry,
                grammar,
                options.output_dir.as_deref(),
                &version,
                &wasm_bindgen,
                &wasm_opt,
                &printer,
            );

            match result {
                Ok(()) => {
                    println!(
                        "{} {}",
                        format!("[{:^18}]", grammar).green(),
                        "done".green()
                    );
                }
                Err(e) => {
                    eprintln!(
                        "{} {}",
                        format!("[{:^18}]", grammar).red(),
                        format!("{}", e).red()
                    );
                    std::process::exit(1);
                }
            }
        })
    });

    let manifest = build_manifest(
        repo_root,
        &registry,
        &grammars,
        options.output_dir.as_deref(),
        &version,
    )?;

    // Write JSON manifest to langs/plugins.json (for dev server)
    let manifest_path = repo_root.join("langs").join("plugins.json");
    fs_err::create_dir_all(manifest_path.parent().unwrap())
        .into_diagnostic()
        .context("failed to create manifest dir")?;
    fs_err::write(&manifest_path, facet_json::to_string_pretty(&manifest))
        .into_diagnostic()
        .context("failed to write manifest")?;
    println!(
        "{} Wrote plugin manifest {}",
        "✓".green(),
        manifest_path.cyan()
    );

    // Write TypeScript manifest to packages/arborium/src/plugins-manifest.ts (bundled)
    // This is a simplified manifest - just a list of language names
    let mut sorted_grammars = grammars.clone();
    sorted_grammars.sort();
    let ts_manifest_path = repo_root
        .join("packages/arborium/src")
        .join("plugins-manifest.ts");
    let ts_template = PluginsManifestTsTemplate {
        languages: &sorted_grammars,
    };
    let ts_content = ts_template
        .render_once()
        .expect("PluginsManifestTsTemplate render failed");
    fs_err::write(&ts_manifest_path, ts_content)
        .into_diagnostic()
        .context("failed to write TypeScript manifest")?;
    println!(
        "{} Wrote TypeScript manifest {}",
        "✓".green(),
        ts_manifest_path.cyan()
    );

    // Print next steps hint
    println!();
    println!("{}", "Next steps:".bold());
    println!(
        "  {} {} to publish crates (start with {} then language groups, then {})",
        "→".blue(),
        "cargo xtask publish crates".cyan(),
        "--group pre".yellow(),
        "--group post".yellow()
    );
    println!(
        "  {} {} to publish npm packages",
        "→".blue(),
        "cargo xtask publish npm".cyan()
    );

    Ok(())
}

/// Build the arborium-host WASM module using wasm-pack for the browser.
pub fn build_host(repo_root: &Utf8Path) -> Result<()> {
    println!(
        "{} {}",
        "==>".cyan().bold(),
        "Building arborium-host (wasm-bindgen)".bold()
    );

    let wasm_pack = Tool::WasmPack
        .find()
        .into_diagnostic()
        .context("wasm-pack not found")?;

    let host_crate = repo_root.join("crates/arborium-host");
    let demo_pkg = repo_root.join("demo/pkg");

    // Build with wasm-pack for web target
    println!("  {} Building with wasm-pack...", "●".cyan());
    let mut cmd = wasm_pack.command();
    cmd.args([
        "build",
        "--release",
        "--target",
        "web",
        "--out-dir",
        demo_pkg.as_str(),
        "--out-name",
        "arborium_host",
    ])
    .current_dir(&host_crate);

    let output = cmd
        .output()
        .into_diagnostic()
        .context("failed to run wasm-pack")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        miette::bail!("wasm-pack build failed:\n{}\n{}", stdout, stderr);
    }

    // wasm-pack generates files with _bg suffix for the wasm file
    // The generated files are:
    // - arborium_host.js (the entry point)
    // - arborium_host_bg.wasm (the wasm binary)
    // - arborium_host.d.ts (type declarations)

    println!("  {} Host built successfully", "✓".green());
    Ok(())
}

pub fn clean_plugins(repo_root: &Utf8Path, _output_dir: &str) -> Result<()> {
    // Clean all individual plugin crate target directories
    let langs_dir = repo_root.join("langs");

    let mut cleaned_count = 0;

    // Find all plugin npm/ directories and clean their target and artifact directories
    for group_entry in std::fs::read_dir(&langs_dir)
        .into_diagnostic()
        .context("failed to read langs dir")?
    {
        let group_entry = group_entry.into_diagnostic()?;
        let group_path = group_entry.path();

        if !group_path.is_dir()
            || !group_entry
                .file_name()
                .to_string_lossy()
                .starts_with("group-")
        {
            continue;
        }

        for lang_entry in std::fs::read_dir(&group_path)
            .into_diagnostic()
            .context(format!("failed to read {:?}", group_path))?
        {
            let lang_entry = lang_entry.into_diagnostic()?;
            let npm_dir = lang_entry.path().join("npm");
            let target_dir = npm_dir.join("target");
            let artifact_dir = npm_dir.join("artifact-out");

            if target_dir.exists() {
                std::fs::remove_dir_all(&target_dir)
                    .into_diagnostic()
                    .context(format!("failed to remove {:?}", target_dir))?;
                cleaned_count += 1;
            }

            if artifact_dir.exists() {
                std::fs::remove_dir_all(&artifact_dir)
                    .into_diagnostic()
                    .context(format!("failed to remove {:?}", artifact_dir))?;
            }
        }
    }

    if cleaned_count > 0 {
        println!(
            "{} Cleaned {} plugin target directories",
            "✓".green(),
            cleaned_count
        );
    } else {
        println!("{} Nothing to clean", "○".dimmed());
    }
    Ok(())
}

/// Generate demo assets (registry.json, samples, HTML, JS).
///
/// The demo loads grammar WASM components on demand - it doesn't need
/// a monolithic WASM build. This just generates the static assets.
pub fn build_demo(repo_root: &Utf8Path, crates_dir: &Utf8Path, dev: bool) -> Result<()> {
    let demo_dir = repo_root.join("demo");

    println!(
        "{} {}",
        "==>".cyan().bold(),
        "Generating demo assets".bold()
    );
    if dev {
        println!("    {}", "(dev mode - using local plugin paths)".dimmed());
    }
    println!();

    // Generate registry.json and assets
    crate::serve::generate_registry_and_assets(crates_dir, &demo_dir, dev)
        .map_err(|e| miette::miette!("Failed to generate assets: {}", e))?;

    // Print next steps
    println!();
    println!("{}", "Next steps:".bold());
    println!(
        "  {} {} to serve the demo locally",
        "→".blue(),
        "cargo xtask serve".cyan()
    );

    Ok(())
}

fn build_single_plugin(
    repo_root: &Utf8Path,
    registry: &CrateRegistry,
    grammar: &str,
    output_override: Option<&Utf8Path>,
    _version: &str,
    wasm_bindgen: &crate::tool::ToolPath,
    wasm_opt: &crate::tool::ToolPath,
    printer: &OutputPrinter,
) -> Result<()> {
    printer.print_line(grammar, "Building...", false);

    let (crate_state, _) = locate_grammar(registry, grammar).ok_or_else(|| {
        miette::miette!(
            "grammar `{}` not found in registry (generate components must be enabled)",
            grammar
        )
    })?;

    let grammar_crate_path = &crate_state.crate_path;

    // Plugin source is always at langs/group-*/*/npm/ (generated by `cargo xtask gen`)
    let plugin_source = grammar_crate_path
        .parent()
        .expect("lang directory")
        .join("npm");

    // Output directory can be overridden with -o flag
    let plugin_output = if let Some(base) = output_override {
        let base = if base.is_absolute() {
            base.to_owned()
        } else {
            repo_root.join(base)
        };
        base.join(grammar)
    } else {
        plugin_source.clone()
    };

    // Plugin crate files (Cargo.toml, src/lib.rs, package.json) are now generated
    // by `cargo xtask gen`. Verify they exist before building.
    let cargo_toml = plugin_source.join("Cargo.toml");
    let lib_rs = plugin_source.join("src/lib.rs");
    if !cargo_toml.exists() || !lib_rs.exists() {
        miette::bail!(
            "Plugin crate files not found at {}. Run `cargo xtask gen --version <version>` first.",
            plugin_source
        );
    }

    // Step 1: Build with cargo +nightly using unstable features
    // We use -Zbuild-std to rebuild std with optimizations for smaller WASM size

    // Create a unique artifact directory for this plugin to avoid locking
    let artifact_dir = plugin_source.join("artifact-out");
    std::fs::create_dir_all(&artifact_dir)
        .into_diagnostic()
        .context("failed to create artifact directory")?;

    let mut cargo_cmd = Command::new("cargo");
    cargo_cmd
        .args([
            "+nightly",
            "build",
            "--lib",
            "--release",
            "--target",
            "wasm32-unknown-unknown",
            "-Zbuild-std=std,panic_abort",
            "-Zunstable-options",
            "-Zbuild-dir-new-layout",
            "-Zbinary-dep-depinfo",
            "-Zchecksum-freshness",
            "--artifact-dir",
            artifact_dir.as_str(),
        ])
        .env(
            "RUSTFLAGS",
            "-Zunstable-options -Cpanic=immediate-abort -Copt-level=s",
        )
        .current_dir(&plugin_source);

    let status = run_streaming(cargo_cmd, grammar, printer)
        .into_diagnostic()
        .context("failed to run cargo build")?;

    if !status.success() {
        miette::bail!("cargo build failed (see output above)");
    }

    // Step 2: Locate the .wasm file in the artifact directory
    // With -Zartifact-dir, the final artifact is placed directly in artifact-out/
    let wasm_name = format!("arborium_{}_plugin", grammar.replace('-', "_"));
    let wasm_file = artifact_dir.join(format!("{}.wasm", wasm_name));

    if !wasm_file.exists() {
        miette::bail!(
            "WASM file not found at {}. Build may have failed.",
            wasm_file
        );
    }

    // Step 3: Run wasm-bindgen to generate JS bindings
    // Create a temporary output directory for wasm-bindgen
    let bindgen_out = plugin_source.join("pkg");
    fs_err::create_dir_all(&bindgen_out)
        .into_diagnostic()
        .context("failed to create bindgen output directory")?;

    let mut bindgen_cmd = wasm_bindgen.command();
    bindgen_cmd
        .args([
            "--target",
            "web",
            "--out-dir",
            bindgen_out.as_str(),
            "--out-name",
            &wasm_name,
            wasm_file.as_str(),
        ])
        .current_dir(&plugin_source);

    let status = run_streaming(bindgen_cmd, grammar, printer)
        .into_diagnostic()
        .context("failed to run wasm-bindgen")?;

    if !status.success() {
        miette::bail!("wasm-bindgen failed (see output above)");
    }

    // Step 4: Optimize WASM with wasm-opt
    let src_wasm = bindgen_out.join(format!("{}_bg.wasm", wasm_name));
    let optimized_wasm = bindgen_out.join(format!("{}_bg.opt.wasm", wasm_name));

    let mut opt_cmd = wasm_opt.command();
    opt_cmd
        .args([
            "-O3", // Aggressive optimization
            "--enable-bulk-memory",
            "--enable-mutable-globals",
            "--enable-nontrapping-float-to-int",
            "--enable-sign-ext",
            "--enable-simd",
            "-o",
            optimized_wasm.as_str(),
            src_wasm.as_str(),
        ])
        .current_dir(&plugin_source);

    let status = run_streaming(opt_cmd, grammar, printer)
        .into_diagnostic()
        .context("failed to run wasm-opt")?;

    if !status.success() {
        miette::bail!("wasm-opt failed (see output above)");
    }

    // Step 5: Copy and rename output files
    fs_err::create_dir_all(&plugin_output)
        .into_diagnostic()
        .context("failed to create output directory")?;

    // Use optimized WASM and generated JS
    let src_js = bindgen_out.join(format!("{}.js", wasm_name));

    let dest_wasm = plugin_output.join("grammar_bg.wasm");
    let dest_js = plugin_output.join("grammar.js");

    // Copy and rename files (use optimized WASM)
    std::fs::copy(&optimized_wasm, &dest_wasm)
        .into_diagnostic()
        .with_context(|| {
            format!(
                "failed to copy optimized wasm file from {} to {}",
                optimized_wasm, dest_wasm
            )
        })?;

    std::fs::copy(&src_js, &dest_js)
        .into_diagnostic()
        .with_context(|| format!("failed to copy js file from {} to {}", src_js, dest_js))?;

    // Generate package.json
    let package_json_content = serde_json::json!({
        "name": format!("@arborium/{}", grammar),
        "version": _version,
        "type": "module",
        "files": ["grammar.js", "grammar_bg.wasm"]
    });
    let dest_package_json = plugin_output.join("package.json");
    std::fs::write(
        &dest_package_json,
        serde_json::to_string_pretty(&package_json_content).unwrap(),
    )
    .into_diagnostic()
    .context("failed to write package.json")?;

    Ok(())
}

pub fn locate_grammar<'a>(
    registry: &'a CrateRegistry,
    grammar: &str,
) -> Option<(
    &'a crate::types::CrateState,
    &'a crate::types::GrammarConfig,
)> {
    registry.configured_crates().find_map(|(_, state, cfg)| {
        cfg.grammars
            .iter()
            .find(|g| <String as AsRef<str>>::as_ref(&g.id.value) == grammar)
            .map(|g| (state, g))
    })
}

/// Sailfish template for TypeScript manifest (simplified - just language names).
#[derive(sailfish::TemplateSimple)]
#[template(path = "plugins_manifest.stpl.ts")]
struct PluginsManifestTsTemplate<'a> {
    languages: &'a [String],
}

/// Generate the plugins-manifest.ts file for the npm package.
/// This uses ALL grammars with generate_component enabled, not just locally built ones.
/// The manifest is simplified: just a list of language names.
/// CDN URLs are derived at runtime: `https://cdn.jsdelivr.net/npm/@arborium/{lang}@1/grammar.js`
pub fn generate_plugins_manifest(repo_root: &Utf8Path, crates_dir: &Utf8Path) -> Result<()> {
    let registry = CrateRegistry::load(crates_dir)
        .map_err(|e| miette::miette!("failed to load crate registry: {}", e))?;

    // Get ALL grammars that have generate_component enabled
    let mut languages: Vec<String> = registry
        .all_grammars()
        .filter(|(_, _, grammar)| grammar.generate_component())
        .map(|(_, _, grammar)| grammar.id().to_string())
        .collect();

    // Sort for consistent output
    languages.sort();

    if languages.is_empty() {
        miette::bail!("No grammars have generate-component enabled");
    }

    println!(
        "{} Generating manifest for {} language(s)",
        "●".cyan(),
        languages.len()
    );

    // Write TypeScript manifest
    let ts_manifest_path = repo_root
        .join("packages/arborium/src")
        .join("plugins-manifest.ts");
    let ts_template = PluginsManifestTsTemplate {
        languages: &languages,
    };
    let ts_content = ts_template
        .render_once()
        .expect("PluginsManifestTsTemplate render failed");
    fs_err::write(&ts_manifest_path, ts_content)
        .into_diagnostic()
        .context("failed to write TypeScript manifest")?;

    println!(
        "{} Wrote {} with {} languages",
        "✓".green(),
        ts_manifest_path.cyan(),
        languages.len()
    );

    Ok(())
}

fn build_manifest(
    repo_root: &Utf8Path,
    registry: &CrateRegistry,
    grammars: &[String],
    output_override: Option<&Utf8Path>,
    version: &str,
) -> Result<PluginManifest> {
    let mut entries = Vec::new();

    for grammar in grammars {
        let (state, _) = locate_grammar(registry, grammar)
            .ok_or_else(|| miette::miette!("grammar `{}` not found for manifest", grammar))?;

        let local_root = if let Some(base) = output_override {
            if base.is_absolute() {
                base.to_owned()
            } else {
                repo_root.join(base)
            }
        } else {
            state
                .crate_path
                .parent()
                .expect("lang directory")
                .join("npm")
        };
        let local_js = local_root.join("grammar.js");
        let local_wasm = local_root.join("grammar_bg.wasm");

        // Make local paths relative to repo root for serving
        let rel_js = local_js.strip_prefix(repo_root).unwrap_or(&local_js);
        let rel_wasm = local_wasm.strip_prefix(repo_root).unwrap_or(&local_wasm);

        let package = format!("@arborium/{}", grammar);
        let cdn_base = format!(
            "https://cdn.jsdelivr.net/npm/@arborium/{}@{}",
            grammar, version
        );

        entries.push(PluginManifestEntry {
            language: grammar.clone(),
            package: package.clone(),
            version: version.to_string(),
            cdn_js: format!("{}/grammar.js", cdn_base),
            cdn_wasm: format!("{}/grammar_bg.wasm", cdn_base),
            local_js: format!("/{}", rel_js),
            local_wasm: format!("/{}", rel_wasm),
        });
    }

    Ok(PluginManifest {
        generated_at: Utc::now().to_rfc3339(),
        entries,
    })
}
