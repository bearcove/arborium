use std::io::{BufRead, BufReader};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use rand::seq::SliceRandom;

use camino::{Utf8Path, Utf8PathBuf};
use chrono::Utc;
use rootcause::Report;

type Result<T> = std::result::Result<T, Report>;

fn report(msg: impl Into<String>) -> Report {
    std::io::Error::other(msg.into()).into()
}
use rayon::prelude::*;
use sailfish::TemplateSimple;
use walrus::Module;

use crate::highlight_gen::{self, NamedHighlight};
use crate::tool::Tool;
use crate::types::CrateRegistry;
use crate::version_store;

/// Print command before execution (for non-streaming commands)
fn print_cmd(cmd: &Command) {
    let program = cmd.get_program().to_string_lossy();
    let args: Vec<_> = cmd.get_args().map(|a| a.to_string_lossy()).collect();
    let full_cmd = if args.is_empty() {
        program.to_string()
    } else {
        format!("{} {}", program, args.join(" "))
    };
    println!("  {} {}", "$".dimmed(), full_cmd.dimmed());
}

/// Run a command, print it first, and return output
fn run_cmd_output(mut cmd: Command) -> std::io::Result<std::process::Output> {
    print_cmd(&cmd);
    cmd.output()
}

/// Verify nightly toolchain with wasm32-unknown-unknown target and rust-src are available.
/// These should be pre-installed in CI (via Dockerfile.ci) or locally by the developer.
fn ensure_rust_nightly_with_wasm_target() -> Result<()> {
    // Check if nightly toolchain is installed
    let mut cmd = Command::new("rustup");
    cmd.args(["toolchain", "list"]);
    let output = run_cmd_output(cmd)
        ?;

    let toolchains = String::from_utf8_lossy(&output.stdout);
    let has_nightly = toolchains.lines().any(|line| line.contains("nightly"));

    if !has_nightly {
        return Err(report(
            "nightly toolchain not found. Install with: rustup toolchain install nightly"
        ));
    }

    // Check if wasm32-unknown-unknown target is installed for nightly
    let mut cmd = Command::new("rustup");
    cmd.args(["+nightly", "target", "list", "--installed"]);
    let output = run_cmd_output(cmd)?;

    let targets = String::from_utf8_lossy(&output.stdout);
    let has_wasm_target = targets
        .lines()
        .any(|line| line.trim() == "wasm32-unknown-unknown");

    if !has_wasm_target {
        return Err(report(
            "wasm32-unknown-unknown target not found for nightly. Install with: rustup target add wasm32-unknown-unknown --toolchain nightly"
        ));
    }

    // Check if rust-src component is installed for nightly (needed for -Zbuild-std)
    let mut cmd = Command::new("rustup");
    cmd.args(["+nightly", "component", "list", "--installed"]);
    let output = run_cmd_output(cmd)
        ?;

    let components = String::from_utf8_lossy(&output.stdout);
    let has_rust_src = components.lines().any(|line| line.starts_with("rust-src"));

    if !has_rust_src {
        return Err(report(
            "rust-src component not found for nightly. Install with: rustup component add rust-src --toolchain nightly"
        ));
    }

    Ok(())
}

/// Build phase icons (nerd font)
const ICON_RUST: &str = "\u{e7a8}"; //
const ICON_WASM: &str = "\u{e6a1}"; //
const ICON_JS: &str = "\u{e74e}"; //
#[allow(dead_code)]
const ICON_PACKAGE: &str = "\u{f487}"; //
const ICON_CHECK: &str = "\u{f00c}"; //
const ICON_CROSS: &str = "\u{f00d}"; //
const ICON_GEAR: &str = "\u{f013}"; //

/// Separator between prefix and log
const SEP: &str = "│"; // U+2502 box drawings light vertical

/// Get the nerd font icon for a language based on its grammar name
fn language_icon(grammar: &str) -> &'static str {
    // Extract the base language name from arborium-{lang}-plugin or arborium-{lang}
    let lang = grammar
        .strip_prefix("arborium-")
        .and_then(|s| s.strip_suffix("-plugin"))
        .or_else(|| grammar.strip_prefix("arborium-"))
        .unwrap_or(grammar);

    match lang {
        // Programming languages with official icons
        "rust" => "\u{e7a8}",            //
        "python" => "\u{e73c}",          //
        "javascript" => "\u{e74e}",      //
        "typescript" => "\u{e628}",      //
        "java" => "\u{e738}",            //
        "c" => "\u{e61e}",               //
        "cpp" | "c-sharp" => "\u{e61d}", //
        "go" => "\u{e626}",              //
        "ruby" => "\u{e21e}",            //
        "php" => "\u{e73d}",             //
        "swift" => "\u{e755}",           //
        "kotlin" => "\u{e634}",          //
        "scala" => "\u{e737}",           //
        "clojure" => "\u{e768}",         //
        "elixir" => "\u{e62d}",          //
        "erlang" => "\u{e7b1}",          //
        "haskell" => "\u{e777}",         //
        "ocaml" => "\u{e91a}",           //
        "lua" => "\u{e620}",             //
        "r" => "\u{f25d}",               //
        "julia" => "\u{e624}",           //
        "dart" => "\u{e798}",            //
        "elm" => "\u{e62c}",             //
        "fsharp" => "\u{e7a7}",          //
        "nim" => "\u{e677}",             //
        "zig" => "\u{e6a9}",             //

        // Markup & data
        "html" => "\u{e736}",                  //
        "css" | "scss" | "sass" => "\u{e749}", //
        "json" => "\u{e60b}",                  //
        "yaml" | "yml" => "\u{f481}",          //
        "toml" => "\u{e6b2}",                  //
        "xml" => "\u{e619}",                   //
        "markdown" => "\u{e609}",              //
        "latex" | "tex" => "\u{e6a4}",         //

        // Shell & config
        "bash" | "sh" => "\u{e795}", //
        "fish" => "\u{f739}",        //
        "powershell" => "\u{e683}",  //
        "vim" => "\u{e62b}",         //
        "dockerfile" => "\u{f308}",  //
        "nginx" => "\u{f308}",       //

        // Build & tools
        "makefile" | "make" => "\u{e779}", //
        "cmake" => "\u{e615}",             //
        "meson" => "\u{e615}",             //

        // Databases
        "sql" | "mysql" | "postgresql" => "\u{e706}", //

        // Other
        "git" => "\u{e702}",     //
        "graphql" => "\u{e602}", //

        // Default fallback
        _ => "\u{f15c}", // generic file icon
    }
}

/// Build phase for display
#[derive(Clone, Copy)]
#[allow(dead_code)]
enum BuildPhase {
    Cargo,
    WasmBindgen,
    WasmOpt,
    Package,
}

impl BuildPhase {
    fn icon(&self) -> &'static str {
        match self {
            BuildPhase::Cargo => ICON_RUST,
            BuildPhase::WasmBindgen => ICON_WASM,
            BuildPhase::WasmOpt => ICON_GEAR,
            BuildPhase::Package => ICON_JS,
        }
    }

    fn name(&self) -> &'static str {
        match self {
            BuildPhase::Cargo => "cargo",
            BuildPhase::WasmBindgen => "wasm-bindgen",
            BuildPhase::WasmOpt => "wasm-opt",
            BuildPhase::Package => "package",
        }
    }
}

/// Thread-safe output printer for parallel builds with progress tracking.
#[derive(Clone)]
struct OutputPrinter {
    multi: MultiProgress,
    progress: ProgressBar,
    completed: Arc<AtomicUsize>,
    #[allow(dead_code)]
    total: usize,
}

impl OutputPrinter {
    fn new(total: usize) -> Self {
        let multi = MultiProgress::new();

        // Progress bar at top with fixed width (40 chars max)
        let style = ProgressStyle::default_bar()
            .template("{spinner:.cyan} [{bar:40.cyan/dim}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("━━╸");

        let progress = multi.insert(0, ProgressBar::new(total as u64));
        progress.set_style(style);
        progress.set_message("Building plugins");

        Self {
            multi,
            progress,
            completed: Arc::new(AtomicUsize::new(0)),
            total,
        }
    }

    fn inc_completed(&self) {
        let completed = self.completed.fetch_add(1, Ordering::SeqCst) + 1;
        self.progress.set_position(completed as u64);
    }

    fn finish(&self) {
        self.progress.finish_with_message("Build complete");
    }

    fn format_prefix(grammar: &str, phase: Option<BuildPhase>) -> String {
        let icon = language_icon(grammar);
        match phase {
            Some(p) => format!("{:>14} {} {}", grammar, p.icon(), SEP),
            None => format!("{:>14} {} {}", grammar, icon, SEP),
        }
    }

    fn print_line(&self, grammar: &str, line: &str, is_stderr: bool) {
        self.print_line_with_phase(grammar, line, is_stderr, None);
    }

    fn print_line_with_phase(
        &self,
        grammar: &str,
        line: &str,
        is_stderr: bool,
        phase: Option<BuildPhase>,
    ) {
        let prefix = Self::format_prefix(grammar, phase);
        let colored_prefix = if is_stderr {
            prefix.red().to_string()
        } else {
            prefix.blue().to_string()
        };
        let msg = format!("{} {}", colored_prefix, line);
        let _ = self.multi.println(&msg);
    }

    #[allow(dead_code)]
    fn print_phase_start(&self, grammar: &str, phase: BuildPhase) {
        let prefix = Self::format_prefix(grammar, Some(phase));
        let msg = format!("{} {} ...", prefix.cyan(), phase.name().dimmed());
        let _ = self.multi.println(&msg);
    }

    fn print_success(&self, grammar: &str) {
        self.inc_completed();
        let msg = format!(
            "{:>14} {} {} {}",
            grammar.green(),
            ICON_CHECK.green(),
            SEP.green(),
            "done".green().bold()
        );
        let _ = self.multi.println(&msg);
    }

    fn print_error(&self, grammar: &str, error: &str) {
        let msg = format!(
            "{:>14} {} {} {} {}",
            grammar.red(),
            ICON_CROSS.red(),
            SEP.red(),
            "failed:".red().bold(),
            error.red()
        );
        let _ = self.multi.println(&msg);
    }
}

/// Format a command for display (program path + all arguments).
fn format_command(cmd: &Command) -> String {
    let program = cmd.get_program().to_string_lossy();
    let args: Vec<_> = cmd.get_args().map(|a| a.to_string_lossy()).collect();
    if args.is_empty() {
        program.to_string()
    } else {
        format!("{} {}", program, args.join(" "))
    }
}

/// Result of running a streaming command - includes captured stderr for error reporting.
struct StreamingResult {
    status: ExitStatus,
    stderr: String,
}

/// Run a command and stream its output with prefixed lines.
fn run_streaming(
    cmd: Command,
    grammar: &str,
    printer: &OutputPrinter,
) -> std::io::Result<StreamingResult> {
    run_streaming_with_phase(cmd, grammar, printer, None)
}

fn run_streaming_with_phase(
    mut cmd: Command,
    grammar: &str,
    printer: &OutputPrinter,
    phase: Option<BuildPhase>,
) -> std::io::Result<StreamingResult> {
    // Print the full command being executed
    let cmd_str = format_command(&cmd);
    printer.print_line_with_phase(grammar, &format!("$ {}", cmd_str), false, phase);

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
            printer_out.print_line_with_phase(&grammar_out, &line, false, phase);
        }
    });

    // Capture stderr while also printing it
    let stderr_thread = thread::spawn(move || {
        let reader = BufReader::new(stderr);
        let mut captured = Vec::new();
        for line in reader.lines() {
            let Ok(line) = line else { break };
            printer_err.print_line_with_phase(&grammar_err, &line, true, phase);
            captured.push(line);
        }
        captured
    });

    let status = child.wait()?;

    stdout_thread.join().expect("stdout thread panicked");
    let stderr_lines = stderr_thread.join().expect("stderr thread panicked");

    Ok(StreamingResult {
        status,
        stderr: stderr_lines.join("\n"),
    })
}

pub struct BuildOptions {
    pub grammars: Vec<String>,
    pub group: Option<String>,
    pub output_dir: Option<Utf8PathBuf>,
    pub jobs: usize,
    pub no_fail_fast: bool,
}

impl Default for BuildOptions {
    fn default() -> Self {
        Self {
            grammars: Vec::new(),
            group: None,
            output_dir: None,
            jobs: 16,
            no_fail_fast: false,
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
    pub fn discover(langs_dir: &Utf8Path) -> Result<Self> {
        let mut groups = Vec::new();

        // Read all group-* directories
        let mut group_dirs: Vec<_> = std::fs::read_dir(langs_dir)
            .map_err(|e| report(format!("failed to read {}: {}", langs_dir, e)))?
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
                .map_err(|e| report(format!("failed to read {:?}: {}", group_path, e)))?
            {
                let lang_entry =
                    lang_entry.map_err(|e| report(format!("failed to read entry: {}", e)))?;
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
    pub size_bytes: u64,
    pub size_gzip: u64,
    pub size_brotli: u64,
    pub c_lines: u64,
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
        .map_err(|e| report(format!("failed to load crate registry: {}", e)))?;

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

    // Verify nightly toolchain and wasm32-unknown-unknown target are available
    ensure_rust_nightly_with_wasm_target()?;

    let wasm_bindgen = Tool::WasmBindgen.find()?;

    let wasm_opt = Tool::WasmOpt.find()?;

    let printer = OutputPrinter::new(grammars.len());
    let errors: Arc<Mutex<Vec<(String, String)>>> = Arc::new(Mutex::new(Vec::new()));

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
                Ok(_) => {
                    printer.print_success(grammar);
                }
                Err(e) => {
                    printer.print_error(grammar, &format!("{}", e));
                    if options.no_fail_fast {
                        let mut errors = errors.lock().expect("errors mutex poisoned");
                        errors.push((grammar.clone(), format!("{}", e)));
                    } else {
                        eprintln!("Build failed for grammar `{}`: {:?}", grammar, e);
                        std::process::exit(1);
                    }
                }
            }
        })
    });

    printer.finish();

    if options.no_fail_fast {
        let errors = errors.lock().expect("errors mutex poisoned");
        if !errors.is_empty() {
            eprintln!();
            eprintln!("Build failures ({}):", errors.len());
            for (grammar, err) in errors.iter() {
                eprintln!();
                eprintln!("== {} ==", grammar);
                eprintln!("{}", err);
            }

            let summary = errors
                .iter()
                .map(|(g, e)| format!("  - {}: {}", g, e.lines().next().unwrap_or("")))
                .collect::<Vec<_>>()
                .join("\n");

            return Err(report(format!(
                "Build completed with {} failure(s):\n{}",
                errors.len(),
                summary
            )));
        }
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
    fs_err::create_dir_all(manifest_path.parent().unwrap())?;
    fs_err::write(&manifest_path, facet_json::to_string_pretty(&manifest).expect("manifest serialization failed"))?;
    println!(
        "{} Wrote plugin manifest {}",
        "✓".green(),
        manifest_path.cyan()
    );

    // Write TypeScript manifest to packages/arborium/src/plugins-manifest.ts (bundled)
    // This is a simplified manifest - just a list of language names plus version
    let mut sorted_grammars = grammars.clone();
    sorted_grammars.sort();
    let ts_manifest_path = repo_root
        .join("packages/arborium/src")
        .join("plugins-manifest.ts");
    let highlights = highlight_gen::parse_highlights(&crates_dir)
        .map_err(|e| report(format!("failed to parse highlights: {}", e)))?;
    let ts_template = PluginsManifestTsTemplate {
        version: &version,
        languages: &sorted_grammars,
        highlights: &highlights.defs,
    };
    let ts_content = ts_template
        .render_once()
        .expect("PluginsManifestTsTemplate render failed");
    fs_err::write(&ts_manifest_path, ts_content)?;
    println!(
        "{} Wrote TypeScript manifest {} (version {})",
        "✓".green(),
        ts_manifest_path.cyan(),
        version.cyan()
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

    let wasm_pack = Tool::WasmPack.find()?;

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

    let output = run_cmd_output(cmd)?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(report(format!("wasm-pack build failed:\n{}\n{}", stdout, stderr)));
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
    for group_entry in std::fs::read_dir(&langs_dir)? {
        let group_entry = group_entry?;
        let group_path = group_entry.path();

        if !group_path.is_dir()
            || !group_entry
                .file_name()
                .to_string_lossy()
                .starts_with("group-")
        {
            continue;
        }

        for lang_entry in std::fs::read_dir(&group_path)? {
            let lang_entry = lang_entry?;
            let npm_dir = lang_entry.path().join("npm");
            let target_dir = npm_dir.join("target");
            let artifact_dir = npm_dir.join("artifact-out");

            if target_dir.exists() {
                std::fs::remove_dir_all(&target_dir)?;
                cleaned_count += 1;
            }

            if artifact_dir.exists() {
                std::fs::remove_dir_all(&artifact_dir)?;
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
        .map_err(|e| report(format!("Failed to generate assets: {}", e)))?;

    // Print next steps
    println!();
    println!("{}", "Next steps:".bold());
    println!(
        "  {} {} to serve the demo locally",
        "→".blue(),
        "cargo xtask serve --dev".cyan()
    );

    Ok(())
}

#[allow(clippy::complexity)]
fn build_single_plugin(
    repo_root: &Utf8Path,
    registry: &CrateRegistry,
    grammar: &str,
    output_override: Option<&Utf8Path>,
    _version: &str,
    wasm_bindgen: &crate::tool::ToolPath,
    wasm_opt: &crate::tool::ToolPath,
    printer: &OutputPrinter,
) -> Result<(u64, u64, u64)> {
    printer.print_line(grammar, "Building...", false);

    let (crate_state, _) = locate_grammar(registry, grammar).ok_or_else(|| {
        report(format!(
            "grammar `{}` not found in registry (generate components must be enabled)",
            grammar
        ))
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
        return Err(report(format!(
            "Plugin crate files not found at {}. Run `cargo xtask gen --version <version>` first.",
            plugin_source
        )));
    }

    // Step 1: Build with cargo +nightly using unstable features
    // We use -Zbuild-std to rebuild std with optimizations for smaller WASM size

    // Create a unique artifact directory for this plugin to avoid locking
    let artifact_dir = plugin_source.join("artifact-out");
    std::fs::create_dir_all(&artifact_dir)?;

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
        // For wasm32 targets, clear environment flags that don't apply:
        // Some environments set global CFLAGS (e.g. `-fembed-bitcode=all` on macOS)
        // which cause warnings or failures in cc-rs-based build scripts when building WASM.
        // Also clear Apple SDK-related variables that inject iOS/macOS-specific flags.
        .env("CFLAGS", "")
        .env("CXXFLAGS", "")
        .env("CFLAGS_wasm32_unknown_unknown", "")
        .env("CFLAGS_wasm32-unknown-unknown", "")
        .env("CXXFLAGS_wasm32_unknown_unknown", "")
        .env("CXXFLAGS_wasm32-unknown-unknown", "")
        // Prevent cc-rs from inheriting Apple SDK flags (like -fembed-bitcode=all for iOS)
        .env("SDKROOT", "")
        .env("IPHONEOS_DEPLOYMENT_TARGET", "")
        .env("TVOS_DEPLOYMENT_TARGET", "")
        .env("WATCHOS_DEPLOYMENT_TARGET", "")
        .env("XROS_DEPLOYMENT_TARGET", "")
        .env(
            "RUSTFLAGS",
            "-Zunstable-options -Cpanic=immediate-abort -Copt-level=s -Cembed-bitcode=yes -Clto=fat -Ccodegen-units=1 -Cstrip=symbols",
        )
        .current_dir(&plugin_source);

    let result = run_streaming(cargo_cmd, grammar, printer)?;

    if !result.status.success() {
        return Err(report(format!("cargo build failed:\n{}", result.stderr)));
    }

    // Step 2: Locate the .wasm file in the artifact directory
    // With -Zartifact-dir, the final artifact is placed directly in artifact-out/
    let wasm_name = format!("arborium_{}_plugin", grammar.replace('-', "_"));
    let wasm_file = artifact_dir.join(format!("{}.wasm", wasm_name));

    if !wasm_file.exists() {
        return Err(report(format!(
            "WASM file not found at {}. Build may have failed.",
            wasm_file
        )));
    }

    // Step 3: Run wasm-bindgen to generate JS bindings
    // Create a temporary output directory for wasm-bindgen
    let bindgen_out = plugin_source.join("pkg");
    fs_err::create_dir_all(&bindgen_out)?;

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

    let result = run_streaming(bindgen_cmd, grammar, printer)?;

    if !result.status.success() {
        return Err(report(format!("wasm-bindgen failed:\n{}", result.stderr)));
    }

    // Step 4: Optimize WASM with wasm-opt
    let src_wasm = bindgen_out.join(format!("{}_bg.wasm", wasm_name));
    let optimized_wasm = bindgen_out.join(format!("{}_bg.opt.wasm", wasm_name));

    let mut opt_cmd = wasm_opt.command();
    opt_cmd
        .args([
            "-Oz", // Optimize for size
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

    let result = run_streaming(opt_cmd, grammar, printer)?;

    if !result.status.success() {
        return Err(report(format!("wasm-opt failed:\n{}", result.stderr)));
    }

    // Step 5: Copy and rename output files
    fs_err::create_dir_all(&plugin_output)?;

    // Use optimized WASM and generated JS
    let src_js = bindgen_out.join(format!("{}.js", wasm_name));

    let dest_wasm = plugin_output.join("grammar_bg.wasm");
    let dest_js = plugin_output.join("grammar.js");

    // Copy and rename files (use optimized WASM)
    std::fs::copy(&optimized_wasm, &dest_wasm)?;

    std::fs::copy(&src_js, &dest_js)?;

    // Check WASM browser compatibility on final WASM file
    println!("  {} Checking WASM browser compatibility...", "●".yellow());
    check_wasm_browser_compatibility(&dest_wasm)?;
    println!("  {} No browser-incompatible imports found", "✓".green());

    // Copy and update package.json (preserve generated metadata like repository/homepage)
    let source_package_json = plugin_source.join("package.json");
    let dest_package_json = plugin_output.join("package.json");
    let package_json_str = fs_err::read_to_string(&source_package_json)?;
    let mut package_json_value: serde_json::Value = serde_json::from_str(&package_json_str)?;
    if let Some(obj) = package_json_value.as_object_mut() {
        obj.insert(
            "version".to_string(),
            serde_json::Value::String(_version.to_string()),
        );
    }
    std::fs::write(
        &dest_package_json,
        serde_json::to_string_pretty(&package_json_value).unwrap(),
    )?;

    // Calculate WASM sizes for the final optimized file
    let (size_bytes, size_gzip, size_brotli) = calculate_wasm_sizes(&dest_wasm)?;

    Ok((size_bytes, size_gzip, size_brotli))
}

/// Count lines of C code in parser.c (and scanner.c if present)
pub fn count_c_lines(crate_path: &Utf8Path) -> u64 {
    let mut total = 0;

    // Count parser.c (in src/)
    let parser_c = crate_path.join("grammar/src/parser.c");
    if let Ok(content) = fs_err::read_to_string(&parser_c) {
        total += content.lines().count() as u64;
    }

    // Count scanner.c if it exists (try src/ first, then grammar/)
    let scanner_c_src = crate_path.join("grammar/src/scanner.c");
    let scanner_c_root = crate_path.join("grammar/scanner.c");

    if let Ok(content) = fs_err::read_to_string(&scanner_c_src) {
        total += content.lines().count() as u64;
    } else if let Ok(content) = fs_err::read_to_string(&scanner_c_root) {
        total += content.lines().count() as u64;
    }

    total
}

pub fn calculate_wasm_sizes(wasm_path: &Utf8Path) -> Result<(u64, u64, u64)> {
    use flate2::Compression;
    use flate2::write::GzEncoder;
    use std::io::Write;

    // Uncompressed size
    let metadata = fs_err::metadata(wasm_path)?;
    let size_bytes = metadata.len();

    // Read file into memory
    let wasm_data = fs_err::read(wasm_path)?;

    // Gzip compression
    let mut gz_encoder = GzEncoder::new(Vec::new(), Compression::best());
    gz_encoder.write_all(&wasm_data)?;
    let size_gzip = gz_encoder.finish()?.len() as u64;

    // Brotli compression (quality 11 = max)
    let mut brotli_output = Vec::new();
    let mut brotli_encoder = brotli::CompressorWriter::new(
        &mut brotli_output,
        4096, // buffer size
        11,   // quality (max)
        22,   // lg_window_size
    );
    brotli_encoder.write_all(&wasm_data)?;
    drop(brotli_encoder);
    let size_brotli = brotli_output.len() as u64;

    Ok((size_bytes, size_gzip, size_brotli))
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
            .find(|g| g.id.as_str() == grammar)
            .map(|g| (state, g))
    })
}

/// Sailfish template for TypeScript manifest (simplified - just language names and highlight defs).
#[derive(sailfish::TemplateSimple)]
#[template(path = "plugins_manifest.stpl.ts")]
struct PluginsManifestTsTemplate<'a> {
    version: &'a str,
    languages: &'a [String],
    highlights: &'a [NamedHighlight],
}

/// Generate the plugins-manifest.ts file for the npm package.
/// This uses ALL grammars with generate_component enabled, not just locally built ones.
/// The manifest is simplified: just a list of language names plus the version.
/// CDN URLs are derived at runtime: `https://cdn.jsdelivr.net/npm/@arborium/{lang}@{version}/grammar.js`
pub fn generate_plugins_manifest(repo_root: &Utf8Path, crates_dir: &Utf8Path) -> Result<()> {
    // Read the canonical version from version.json
    let version = version_store::read_version(repo_root)?;

    let registry = CrateRegistry::load(crates_dir)
        .map_err(|e| report(format!("failed to load crate registry: {}", e)))?;

    // Get ALL grammars that have generate_component enabled
    let mut languages: Vec<String> = registry
        .all_grammars()
        .filter(|(_, _, grammar)| grammar.generate_component())
        .map(|(_, _, grammar)| grammar.id().to_string())
        .collect();

    // Sort for consistent output
    languages.sort();

    if languages.is_empty() {
        return Err(report("No grammars have generate-component enabled"));
    }

    println!(
        "{} Generating manifest for {} language(s) at version {}",
        "●".cyan(),
        languages.len(),
        version.cyan()
    );

    // Write TypeScript manifest
    let ts_manifest_path = repo_root
        .join("packages/arborium/src")
        .join("plugins-manifest.ts");
    let highlights = highlight_gen::parse_highlights(crates_dir)
        .map_err(|e| report(format!("failed to parse highlights: {}", e)))?;
    let ts_template = PluginsManifestTsTemplate {
        version: &version,
        languages: &languages,
        highlights: &highlights.defs,
    };
    let ts_content = ts_template
        .render_once()
        .expect("PluginsManifestTsTemplate render failed");
    fs_err::write(&ts_manifest_path, ts_content)?;

    println!(
        "{} Wrote {} with {} languages at version {}",
        "✓".green(),
        ts_manifest_path.cyan(),
        languages.len(),
        version.cyan()
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
            .ok_or_else(|| report(format!("grammar `{}` not found for manifest", grammar)))?;

        let local_root = if let Some(base) = output_override {
            let base = if base.is_absolute() {
                base.to_owned()
            } else {
                repo_root.join(base)
            };
            base.join(grammar)
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

        // Calculate WASM sizes
        let (size_bytes, size_gzip, size_brotli) = calculate_wasm_sizes(&local_wasm)?;

        // Count C lines in parser
        let c_lines = count_c_lines(&state.crate_path);

        entries.push(PluginManifestEntry {
            language: grammar.clone(),
            package: package.clone(),
            version: version.to_string(),
            cdn_js: format!("{}/grammar.js", cdn_base),
            cdn_wasm: format!("{}/grammar_bg.wasm", cdn_base),
            local_js: format!("/{}", rel_js),
            local_wasm: format!("/{}", rel_wasm),
            size_bytes,
            size_gzip,
            size_brotli,
            c_lines,
        });
    }

    Ok(PluginManifest {
        generated_at: Utc::now().to_rfc3339(),
        entries,
    })
}

/// Analyzes a WASM file for browser compatibility issues
/// Returns an error if any "env" imports are found (which won't work in browsers)
fn check_wasm_browser_compatibility(wasm_file: &camino::Utf8Path) -> Result<()> {
    let wasm_bytes = std::fs::read(wasm_file)
        .map_err(|e| report(format!("Failed to read WASM file {}: {}", wasm_file, e)))?;

    let module = Module::from_buffer(&wasm_bytes)
        .map_err(|e| report(format!("Failed to parse WASM file {}: {}", wasm_file, e)))?;

    let mut env_imports = Vec::new();

    for import in module.imports.iter() {
        if import.module == "env" {
            env_imports.push(import.name.clone());
        }
    }

    if !env_imports.is_empty() {
        return Err(report(format!(
            "WASM module {} has {} browser-incompatible imports from 'env':\n{}\n\n\
             These imports won't work in web browsers. Provide these symbols in the WASM sysroot or \
             remove the dependencies that require them.",
            wasm_file,
            env_imports.len(),
            env_imports
                .iter()
                .map(|name| format!("  - env.{}", name))
                .collect::<Vec<_>>()
                .join("\n")
        )));
    }

    Ok(())
}
