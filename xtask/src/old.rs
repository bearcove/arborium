use std::collections::{BTreeMap, HashMap, HashSet};
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use rayon::prelude::*;

/// Files to keep in each grammar directory (note: all .js and .mjs files are also kept)
const KEEP_FILES: &[&str] = &[
    "grammar.js",
    "package.json",
    "LICENSE",
    "LICENSE.md",
    "COPYING.txt",
    "README.md",
    "grammar-crate-config.toml",
];

/// Directories to keep
const KEEP_DIRS: &[&str] = &["src", "queries", "grammar", "common", "rules", "lib"];

/// File extensions to keep in src/ (all .c, .h, .cc files needed for compilation)
const KEEP_SRC_EXTENSIONS: &[&str] = &[".c", ".cc", ".h", ".json"];

/// Directories to keep in src/
const KEEP_SRC_DIRS: &[&str] = &["tree_sitter"];

/// Grammar dependencies: (grammar_name, &[dependencies])
/// These are grammars that require other grammars to be generated first
const GRAMMAR_DEPS: &[(&str, &[&str])] = &[
    ("tree-sitter-cpp", &["tree-sitter-c"]),
    ("tree-sitter-scss", &["tree-sitter-css"]),
    ("tree-sitter-typescript", &["tree-sitter-javascript"]),
    ("tree-sitter-starlark", &["tree-sitter-python"]),
];

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        std::process::exit(1);
    }

    match args[1].as_str() {
        "regenerate" => {
            let filter = args.get(2).map(|s| s.as_str());
            regenerate(filter);
        }
        "check-updates" => check_updates(),
        "vendor" => {
            if args.len() < 3 {
                eprintln!("Usage: cargo xtask vendor <grammar-name>");
                eprintln!("Example: cargo xtask vendor rust");
                std::process::exit(1);
            }
            vendor_grammar(&args[2]);
        }
        "generate-crates" => generate_grammar_crates(),
        "generate-demo" => generate_demo(),
        "generate-readme" => generate_readme(),
        "update-crates" => update_grammar_crates(),
        "serve-demo" => {
            let mut addr = "127.0.0.1".to_string();
            let mut port: Option<u16> = None;
            let mut dev_mode = false;
            let mut i = 2;
            while i < args.len() {
                match args[i].as_str() {
                    "-a" | "--address" => {
                        if i + 1 < args.len() {
                            addr = args[i + 1].clone();
                            i += 2;
                        } else {
                            eprintln!("Error: -a requires an address argument");
                            std::process::exit(1);
                        }
                    }
                    "-p" | "--port" => {
                        if i + 1 < args.len() {
                            port = Some(args[i + 1].parse().unwrap_or_else(|_| {
                                eprintln!("Error: invalid port number");
                                std::process::exit(1);
                            }));
                            i += 2;
                        } else {
                            eprintln!("Error: -p requires a port argument");
                            std::process::exit(1);
                        }
                    }
                    "--dev" => {
                        dev_mode = true;
                        i += 1;
                    }
                    _ => {
                        eprintln!("Unknown option: {}", args[i]);
                        eprintln!(
                            "Usage: cargo xtask serve-demo [-a <address>] [-p <port>] [--dev]"
                        );
                        std::process::exit(1);
                    }
                }
            }
            serve_demo(&addr, port, dev_mode);
        }
        "lint-info-toml" => lint::lint_info_toml(),
        "lint-highlights" => lint::lint_highlights(),
        "lint" => {
            lint::lint_info_toml();
            println!();
            lint::lint_highlights();
        }
        "help" | "--help" | "-h" => print_usage(),
        cmd => {
            eprintln!("Unknown command: {}", cmd);
            print_usage();
            std::process::exit(1);
        }
    }
}

fn print_usage() {
    eprintln!("Usage: cargo xtask <command>");
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  regenerate [name] Regenerate tree-sitter grammars (optionally filter by name)");
    eprintln!("  check-updates    Check if any vendored grammars have newer versions available");
    eprintln!("  vendor <name>    Update a specific grammar to the latest version");
    eprintln!("  generate-crates  Generate arborium-* crates for all vendored grammars");
    eprintln!("  update-crates    Update existing crates to use test harness");
    eprintln!("  generate-demo    Generate demo/index.html from template and example files");
    eprintln!("  generate-readme  Generate README.md from GRAMMARS.toml");
    eprintln!("  serve-demo       Build and serve the WASM demo locally");
    eprintln!("                   Options: -a <address> (default: 127.0.0.1)");
    eprintln!("                            -p <port> (default: auto-select 8000-8010)");
    eprintln!(
        "                            --dev (fast dev build: -O1, no wasm-opt, fast compression)"
    );
    eprintln!("  lint-info-toml   Lint all info.toml files for missing/invalid fields");
    eprintln!("  lint-highlights  Check that all grammars produce highlights");
    eprintln!("  lint             Run all lints (info-toml + highlights)");
    eprintln!("  help             Show this help message");
}

/// Result of processing a single grammar
struct GrammarResult {
    name: String,
    duration: Duration,
    success: bool,
    error: Option<String>,
}

fn regenerate(filter: Option<&str>) {
    // Configure rayon to use 12 threads
    rayon::ThreadPoolBuilder::new()
        .num_threads(12)
        .build_global()
        .ok();

    let repo_root = find_repo_root().expect("Could not find repo root");
    println!("{} {}", "Repo root:".cyan(), repo_root.display());

    // Step 1: Ensure tree-sitter CLI is installed
    println!("\n{}", "==> Checking tree-sitter CLI".cyan().bold());
    if let Err(e) = ensure_tree_sitter_cli() {
        eprintln!("{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    }

    // Step 2: Find all vendored grammars
    println!("\n{}", "==> Finding vendored grammars".cyan().bold());
    let mut grammars = match find_grammars(&repo_root) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("{} {}", "Error:".red().bold(), e);
            std::process::exit(1);
        }
    };

    // Filter grammars if a filter was provided
    if let Some(filter) = filter {
        let filter_with_prefix = if filter.starts_with("tree-sitter-") {
            filter.to_string()
        } else {
            format!("tree-sitter-{}", filter)
        };
        grammars.retain(|p| {
            let name = p.file_name().unwrap().to_string_lossy();
            name == filter_with_prefix || name.contains(filter)
        });
        if grammars.is_empty() {
            eprintln!(
                "{} No grammars matching '{}'",
                "Error:".red().bold(),
                filter
            );
            std::process::exit(1);
        }
        println!(
            "  Filtered to {} grammar(s) matching '{}'",
            grammars.len(),
            filter
        );
    }

    // Step 3: Group grammars by dependency level for parallel processing
    println!(
        "\n{}",
        "==> Grouping grammars by dependency level".cyan().bold()
    );
    let levels = match group_by_dependency_level(&grammars) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("{} {}", "Error:".red().bold(), e);
            std::process::exit(1);
        }
    };

    for (i, level) in levels.iter().enumerate() {
        let names: Vec<_> = level
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy())
            .collect();
        println!(
            "  Level {}: {} grammars ({})",
            i,
            level.len(),
            names.join(", ").dimmed()
        );
    }

    // Step 4: Set up node_modules symlinks for grammar dependencies
    println!(
        "\n{}",
        "==> Setting up node_modules for dependencies".cyan().bold()
    );
    if let Err(e) = setup_node_modules(&repo_root) {
        eprintln!("{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    }

    // Step 5: Process grammars level by level in parallel
    let total_grammars: usize = levels.iter().map(|l| l.len()).sum();
    let results: Arc<Mutex<Vec<GrammarResult>>> = Arc::new(Mutex::new(Vec::new()));
    let in_progress: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let completed: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));

    // Single progress bar for all grammars
    let pb = ProgressBar::new(total_grammars as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] {msg}")
            .unwrap(),
    );
    pb.enable_steady_tick(Duration::from_millis(100));

    let start_time = Instant::now();

    // Helper to update the progress line
    let update_progress = |pb: &ProgressBar,
                           in_progress: &Arc<Mutex<Vec<String>>>,
                           completed: &Arc<Mutex<usize>>,
                           total: usize| {
        let current = in_progress.lock().unwrap();
        let done = *completed.lock().unwrap();

        // Show at most 6 grammar names
        let display_names: Vec<_> = current
            .iter()
            .map(|n| n.strip_prefix("tree-sitter-").unwrap_or(n))
            .take(6)
            .collect();

        let names_str = if display_names.is_empty() {
            String::new()
        } else if current.len() > 6 {
            format!("{} +{}", display_names.join(", "), current.len() - 6)
        } else {
            display_names.join(", ")
        };

        pb.set_message(format!("[{}/{}] {}", done, total, names_str));
    };

    for (_level_idx, level) in levels.iter().enumerate() {
        level.par_iter().for_each(|grammar| {
            let name = grammar.file_name().unwrap().to_string_lossy().to_string();
            let short_name = name
                .strip_prefix("tree-sitter-")
                .unwrap_or(&name)
                .to_string();
            let grammar_start = Instant::now();

            // Add to in-progress list
            {
                in_progress.lock().unwrap().push(name.clone());
                update_progress(&pb, &in_progress, &completed, total_grammars);
            }

            // Initialize
            let init_result = init_grammar(grammar);
            if let Err(e) = init_result {
                // Remove from in-progress, increment completed
                {
                    in_progress.lock().unwrap().retain(|n| n != &name);
                    *completed.lock().unwrap() += 1;
                    update_progress(&pb, &in_progress, &completed, total_grammars);
                }
                results.lock().unwrap().push(GrammarResult {
                    name: short_name,
                    duration: grammar_start.elapsed(),
                    success: false,
                    error: Some(format!("init failed: {}", e)),
                });
                pb.inc(1);
                return;
            }

            // Regenerate
            let regen_result = regenerate_grammar(grammar);
            let duration = grammar_start.elapsed();

            // Remove from in-progress, increment completed
            {
                in_progress.lock().unwrap().retain(|n| n != &name);
                *completed.lock().unwrap() += 1;
                update_progress(&pb, &in_progress, &completed, total_grammars);
            }

            let result = match regen_result {
                Ok(()) => GrammarResult {
                    name: short_name,
                    duration,
                    success: true,
                    error: None,
                },
                Err(e) => GrammarResult {
                    name: short_name,
                    duration,
                    success: false,
                    error: Some(e.to_string()),
                },
            };

            results.lock().unwrap().push(result);
            pb.inc(1);
        });
    }

    pb.finish_with_message(format!("[{}/{}] Done", total_grammars, total_grammars));

    // Step 6: Clean up all grammars in parallel
    print!("\r{}", "Cleaning up...".dimmed());
    std::io::stdout().flush().ok();

    grammars.par_iter().for_each(|grammar| {
        let _ = clean_grammar(grammar);
    });

    println!("\r{}", "Cleanup complete.".dimmed());

    // Step 7: Copy C sources to crates
    print!("\r{}", "Copying C sources to crates...".dimmed());
    std::io::stdout().flush().ok();

    grammars.par_iter().for_each(|grammar| {
        let _ = copy_grammar_sources_to_crate(&repo_root, grammar);
    });

    println!("\r{}", "C sources copied to crates.".dimmed());

    // Step 8: Copy queries to crates
    print!("\r{}", "Copying queries to crates...".dimmed());
    std::io::stdout().flush().ok();

    grammars.par_iter().for_each(|grammar| {
        let _ = copy_queries_to_crate(&repo_root, grammar);
    });

    println!("\r{}", "Queries copied to crates.".dimmed());

    // Summary
    let total_duration = start_time.elapsed();
    let results = results.lock().unwrap();
    let failures: Vec<_> = results.iter().filter(|r| !r.success).collect();
    let successes: Vec<_> = results.iter().filter(|r| r.success).collect();

    println!("\n{}", "=== Summary ===".cyan().bold());
    println!(
        "Total time: {:.2}s ({} grammars)",
        total_duration.as_secs_f64(),
        results.len()
    );

    // Show timing for all grammars, sorted by duration
    let mut sorted_results: Vec<_> = results.iter().collect();
    sorted_results.sort_by(|a, b| b.duration.cmp(&a.duration));

    println!("\n{}", "Timing (slowest first):".dimmed());
    for result in sorted_results.iter().take(10) {
        let status = if result.success {
            "OK".green().to_string()
        } else {
            "FAILED".red().to_string()
        };
        println!(
            "  {:>6.2}s  {}  {}",
            result.duration.as_secs_f64(),
            status,
            result.name
        );
    }
    if sorted_results.len() > 10 {
        println!("  ... and {} more", sorted_results.len() - 10);
    }

    if failures.is_empty() {
        println!(
            "\n{} All {} grammars regenerated successfully!",
            "SUCCESS:".green().bold(),
            successes.len()
        );
    } else {
        println!(
            "\n{} {} succeeded, {} failed:",
            "PARTIAL:".yellow().bold(),
            successes.len(),
            failures.len()
        );
        for f in &failures {
            println!(
                "  {} {} - {}",
                "-".red(),
                f.name,
                f.error.as_deref().unwrap_or("unknown error").dimmed()
            );
        }
        println!(
            "\n{}",
            "Failed grammars will use their existing parser.c files.".dimmed()
        );
    }
}

fn step<F>(name: &str, f: F)
where
    F: FnOnce() -> Result<(), Box<dyn std::error::Error>>,
{
    println!("\n{} {}", "==>".cyan().bold(), name.bold());
    if let Err(e) = f() {
        eprintln!("{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    }
}

fn find_repo_root() -> Option<PathBuf> {
    let cwd = env::current_dir().ok()?;
    let mut current = cwd.clone();

    loop {
        // Look for Cargo.toml with [workspace] and GRAMMARS.toml
        let cargo_toml = current.join("Cargo.toml");
        let grammars_toml = current.join("GRAMMARS.toml");
        if cargo_toml.exists() && grammars_toml.exists() {
            if let Ok(contents) = fs::read_to_string(&cargo_toml) {
                if contents.contains("[workspace]") {
                    return Some(current);
                }
            }
        }
        if !current.pop() {
            return None;
        }
    }
}

/// Recursively copy a directory
/// TODO: replace with a crate
fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dest_path = dst.join(entry.file_name());

        if path.is_dir() {
            copy_dir_recursive(&path, &dest_path)?;
        } else {
            fs::copy(&path, &dest_path)?;
        }
    }

    Ok(())
}

/// Guess content type from file extension
fn guess_content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "application/javascript; charset=utf-8",
        Some("wasm") => "application/wasm",
        Some("json") => "application/json; charset=utf-8",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("svg") => "image/svg+xml",
        Some("ico") => "image/x-icon",
        _ => "application/octet-stream",
    }
}

/// Compute bundle size information for the wasm files
fn compute_bundle_info(demo_dir: &Path) -> String {
    let pkg_dir = demo_dir.join("pkg");
    let wasm_file = pkg_dir.join("arborium_demo_bg.wasm");
    let js_file = pkg_dir.join("arborium_demo.js");

    let mut info = String::from("{");

    // WASM file
    if let Ok(metadata) = fs::metadata(&wasm_file) {
        let raw_size = metadata.len();
        let compressed_size = fs::read(&wasm_file)
            .ok()
            .and_then(|data| compress_gzip(&data).ok())
            .map(|c| c.len() as u64)
            .unwrap_or(0);
        info.push_str(&format!(
            "\"wasm\":{{\"raw\":{},\"compressed\":{}}},",
            raw_size, compressed_size
        ));
    }

    // JS file
    if let Ok(metadata) = fs::metadata(&js_file) {
        let raw_size = metadata.len();
        let compressed_size = fs::read(&js_file)
            .ok()
            .and_then(|data| compress_gzip(&data).ok())
            .map(|c| c.len() as u64)
            .unwrap_or(0);
        info.push_str(&format!(
            "\"js\":{{\"raw\":{},\"compressed\":{}}}",
            raw_size, compressed_size
        ));
    }

    info.push('}');
    info
}

fn format_size(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

// =============================================================================
// Demo generation
// =============================================================================

/// Escape a string for use in a JavaScript string literal
fn escape_for_js(s: &str) -> String {
    let mut result = String::with_capacity(s.len() + 32);
    for c in s.chars() {
        match c {
            '\\' => result.push_str("\\\\"),
            '"' => result.push_str("\\\""),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            // Handle special HTML sequences that can cause issues
            '<' => result.push_str("\\x3c"),
            '>' => result.push_str("\\x3e"),
            '&' => result.push_str("\\x26"),
            _ => result.push(c),
        }
    }
    result
}
