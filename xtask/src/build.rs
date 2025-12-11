use std::io::{BufRead, BufReader, Write};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

use rand::seq::SliceRandom;

use camino::{Utf8Path, Utf8PathBuf};
use chrono::Utc;
use miette::{Context, IntoDiagnostic, Result};
use owo_colors::OwoColorize;
use rayon::prelude::*;
use sailfish::TemplateSimple;

use crate::tool::Tool;
use crate::types::CrateRegistry;
use crate::version_store;

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
        let prefix = format!("[{}]", grammar);
        let colored_prefix = if is_stderr {
            prefix.red().to_string()
        } else {
            prefix.cyan().to_string()
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

    let wasm_pack = Tool::WasmPack
        .find()
        .into_diagnostic()
        .context("wasm-pack not found")?;

    let errors: Mutex<Vec<(String, String)>> = Mutex::new(Vec::new());
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
                &wasm_pack,
                None,
                &printer,
            );

            match result {
                Ok(()) => {
                    println!("{} {}", format!("[{}]", grammar).green(), "done".green());
                }
                Err(e) => {
                    eprintln!(
                        "{} {}",
                        format!("[{}]", grammar).red(),
                        format!("{}", e).red()
                    );
                    errors
                        .lock()
                        .unwrap()
                        .push((grammar.clone(), format!("{}", e)));
                }
            }
        })
    });

    let errors = errors.into_inner().unwrap();
    if !errors.is_empty() {
        eprintln!("\n{} {} plugin(s) failed:", "✗".red(), errors.len());
        for (grammar, err) in &errors {
            eprintln!("  {} {}", format!("[{}]", grammar).red(), err);
        }
        miette::bail!("{} plugin(s) failed to build", errors.len());
    }

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
    // Clean the workspace target directory (langs/target/)
    // With a workspace, all crates share this single target directory
    let langs_dir = repo_root.join("langs");
    let workspace_target = langs_dir.join("target");

    if workspace_target.exists() {
        std::fs::remove_dir_all(&workspace_target)
            .into_diagnostic()
            .context(format!("failed to remove {}", workspace_target))?;
        println!(
            "{} Cleaned workspace target directory: {}",
            "✓".green(),
            workspace_target
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

#[allow(clippy::too_many_arguments)]
fn build_single_plugin(
    repo_root: &Utf8Path,
    registry: &CrateRegistry,
    grammar: &str,
    output_override: Option<&Utf8Path>,
    _version: &str,
    wasm_pack: &crate::tool::ToolPath,
    _unused: Option<&crate::tool::ToolPath>,
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

    // Use wasm-pack to build the plugin
    let mut cmd = wasm_pack.command();
    cmd.args(["build", "--target", "web", "--release", "--out-dir", "pkg"])
        .current_dir(&plugin_source);
    let status = run_streaming(cmd, grammar, printer)
        .into_diagnostic()
        .context("failed to run wasm-pack")?;

    if !status.success() {
        miette::bail!("wasm-pack build failed (see output above)");
    }

    // wasm-pack outputs to pkg/ directory in the plugin source
    let pkg_dir = plugin_source.join("pkg");

    // Create output directory if it doesn't exist
    fs_err::create_dir_all(&plugin_output)
        .into_diagnostic()
        .context("failed to create output directory")?;

    // Copy the wasm-pack output to the target directory
    if plugin_output != plugin_source {
        // Copy .wasm file
        let src_wasm = pkg_dir.join(format!(
            "arborium_{}_plugin_bg.wasm",
            grammar.replace('-', "_")
        ));
        let dest_wasm = plugin_output.join(format!(
            "arborium_{}_plugin_bg.wasm",
            grammar.replace('-', "_")
        ));
        std::fs::copy(&src_wasm, &dest_wasm)
            .into_diagnostic()
            .context("failed to copy wasm file")?;

        // Copy .js file
        let src_js = pkg_dir.join(format!(
            "arborium_{}_plugin.js",
            grammar.replace('-', "_")
        ));
        let dest_js = plugin_output.join(format!(
            "arborium_{}_plugin.js",
            grammar.replace('-', "_")
        ));
        std::fs::copy(&src_js, &dest_js)
            .into_diagnostic()
            .context("failed to copy js file")?;

        // Copy package.json
        let src_package_json = pkg_dir.join("package.json");
        if src_package_json.exists() {
            let dest_package_json = plugin_output.join("package.json");
            std::fs::copy(&src_package_json, &dest_package_json)
                .into_diagnostic()
                .context("failed to copy package.json")?;
        }
    }

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
        let local_wasm = local_root.join("grammar.core.wasm");

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
            cdn_wasm: format!("{}/grammar.core.wasm", cdn_base),
            local_js: format!("/{}", rel_js),
            local_wasm: format!("/{}", rel_wasm),
        });
    }

    Ok(PluginManifest {
        generated_at: Utc::now().to_rfc3339(),
        entries,
    })
}
