//! Generate command - regenerates crate files from arborium.kdl.
//!
//! This command reads arborium.kdl files and generates:
//! - Cargo.toml
//! - build.rs
//! - src/lib.rs
//! - grammar/src/ (by running tree-sitter generate)

use crate::cache::GrammarCache;
use crate::plan::{Operation, Plan, PlanMode, PlanSet};
use crate::tool::Tool;
use crate::types::{CrateRegistry, CrateState};
use crate::util::find_repo_root;
use camino::{Utf8Path, Utf8PathBuf};
use fs_err as fs;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use rayon::prelude::*;
use rootcause::Report;
use std::io::IsTerminal;
use std::process::Stdio;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Generate crate files for all or a specific grammar.
pub fn plan_generate(
    crates_dir: &Utf8Path,
    name: Option<&str>,
    mode: PlanMode,
) -> Result<PlanSet, Report> {
    use std::time::Instant;
    let total_start = Instant::now();

    // Note: lint is run by main.rs before and after calling this function
    let registry_start = Instant::now();
    let registry = CrateRegistry::load(crates_dir)?;
    let registry_elapsed = registry_start.elapsed();

    // Set up grammar cache
    let repo_root =
        find_repo_root().ok_or_else(|| std::io::Error::other("Could not find repo root"))?;
    let repo_root = Utf8PathBuf::from_path_buf(repo_root)
        .map_err(|_| std::io::Error::other("Non-UTF8 repo root"))?;
    let cache = GrammarCache::new(&repo_root);

    // Track cache stats
    let cache_hits = AtomicUsize::new(0);
    let cache_misses = AtomicUsize::new(0);

    // Collect crates to process (respecting filter)
    let crates_to_process: Vec<_> = registry
        .crates
        .iter()
        .filter(|(_name, crate_state)| {
            // Skip if a specific name was requested and this isn't it
            if let Some(filter) = name {
                let matches = crate_state.name == filter
                    || (crate_state.name.strip_prefix("arborium-") == Some(filter));
                if !matches {
                    return false;
                }
            }
            // Skip crates without arborium.kdl
            crate_state.config.is_some()
        })
        .collect();

    if crates_to_process.is_empty() {
        return Ok(PlanSet::new());
    }

    // Check if we're in a terminal (for spinners) or CI (for plain output)
    let is_tty = std::io::stdout().is_terminal();

    // Set up multi-progress for parallel spinners (only used in TTY mode)
    let mp = MultiProgress::new();
    let spinner_style = ProgressStyle::default_spinner()
        .template("{spinner:.green} {msg} ({elapsed})")
        .unwrap();

    // Thread-safe collection for plans and errors
    let plans = Mutex::new(PlanSet::new());
    let errors: Mutex<Vec<(String, Report)>> = Mutex::new(Vec::new());

    // Process crates in parallel
    crates_to_process
        .par_iter()
        .for_each(|(_name, crate_state)| {
            let config = crate_state.config.as_ref().unwrap();
            let crate_name = &crate_state.name;

            // Check if this crate has a grammar to generate
            let grammar_dir = crate_state.path.join("grammar");
            let needs_generation = grammar_dir.exists() && grammar_dir.join("grammar.js").exists();

            // Show progress - spinner in TTY, plain text in CI
            let pb = if needs_generation {
                if is_tty {
                    let pb = mp.add(ProgressBar::new_spinner());
                    pb.set_style(spinner_style.clone());
                    pb.set_message(format!("Generating {}...", crate_name));
                    pb.enable_steady_tick(std::time::Duration::from_millis(80));
                    Some(pb)
                } else {
                    println!("{} Generating {}...", "●".cyan(), crate_name);
                    None
                }
            } else {
                None
            };

            match plan_crate_generation(
                crate_state,
                config,
                &cache,
                crates_dir,
                &cache_hits,
                &cache_misses,
                mode,
            ) {
                Ok(plan) => {
                    if !plan.is_empty() {
                        plans.lock().unwrap().add(plan);
                    }
                    // Success: clear the spinner (no need to keep it around)
                    if let Some(pb) = pb {
                        pb.finish_and_clear();
                    }
                }
                Err(e) => {
                    // Failure: show error marker
                    if let Some(pb) = pb {
                        pb.finish_with_message(format!("{} {}", "✗".red(), crate_name));
                    } else if needs_generation {
                        println!("{} {} {}", "●".red(), "✗".red(), crate_name);
                    }
                    errors.lock().unwrap().push((crate_name.clone(), e));
                }
            }
        });

    let processing_elapsed = total_start.elapsed();

    // Print timing and cache stats
    let hits = cache_hits.load(Ordering::Relaxed);
    let misses = cache_misses.load(Ordering::Relaxed);
    println!(
        "{} Processed {} crate(s) in {:.2}s (registry: {:.2}s, generation: {:.2}s)",
        "●".cyan(),
        crates_to_process.len(),
        processing_elapsed.as_secs_f64(),
        registry_elapsed.as_secs_f64(),
        (processing_elapsed - registry_elapsed).as_secs_f64(),
    );
    if hits > 0 || misses > 0 {
        println!(
            "{} {} cache hits, {} misses",
            "●".green(),
            hits.to_string().green().bold(),
            misses.to_string().yellow()
        );
    }

    // Check for errors - print all of them
    let errors = errors.into_inner().unwrap();
    if !errors.is_empty() {
        eprintln!();
        for (crate_name, error) in &errors {
            eprintln!(
                "{}",
                boxen::builder()
                    .border_style(boxen::BorderStyle::Round)
                    .border_color("red")
                    .padding(1)
                    .render(format!("{}: {}", crate_name.bold(), error))
                    .unwrap_or_else(|_| format!("{}: {}", crate_name, error))
            );
        }
        Err(std::io::Error::other(format!(
            "{} grammar(s) failed to generate",
            errors.len()
        )))?;
    }

    Ok(plans.into_inner().unwrap())
}

fn plan_crate_generation(
    crate_state: &CrateState,
    config: &crate::types::CrateConfig,
    cache: &GrammarCache,
    crates_dir: &Utf8Path,
    cache_hits: &AtomicUsize,
    cache_misses: &AtomicUsize,
    mode: PlanMode,
) -> Result<Plan, Report> {
    let mut plan = Plan::for_crate(&crate_state.name);
    let crate_path = &crate_state.path;

    // Generate Cargo.toml
    let cargo_toml_path = crate_path.join("Cargo.toml");
    let new_cargo_toml = generate_cargo_toml(&crate_state.name, config);

    if cargo_toml_path.exists() {
        let old_content = fs::read_to_string(&cargo_toml_path)?;
        if old_content != new_cargo_toml {
            plan.add(Operation::UpdateFile {
                path: cargo_toml_path,
                old_content: Some(old_content),
                new_content: new_cargo_toml,
                description: "Update Cargo.toml".to_string(),
            });
        }
    } else {
        plan.add(Operation::CreateFile {
            path: cargo_toml_path,
            content: new_cargo_toml,
            description: "Create Cargo.toml".to_string(),
        });
    }

    // Generate build.rs
    let build_rs_path = crate_path.join("build.rs");
    let new_build_rs = generate_build_rs(&crate_state.name, config);

    if build_rs_path.exists() {
        let old_content = fs::read_to_string(&build_rs_path)?;
        if old_content != new_build_rs {
            plan.add(Operation::UpdateFile {
                path: build_rs_path,
                old_content: Some(old_content),
                new_content: new_build_rs,
                description: "Update build.rs".to_string(),
            });
        }
    } else {
        plan.add(Operation::CreateFile {
            path: build_rs_path,
            content: new_build_rs,
            description: "Create build.rs".to_string(),
        });
    }

    // Generate src/lib.rs
    let lib_rs_path = crate_path.join("src/lib.rs");
    let new_lib_rs = generate_lib_rs(&crate_state.name, crate_path, config);

    if lib_rs_path.exists() {
        let old_content = fs::read_to_string(&lib_rs_path)?;
        if old_content != new_lib_rs {
            plan.add(Operation::UpdateFile {
                path: lib_rs_path,
                old_content: Some(old_content),
                new_content: new_lib_rs,
                description: "Update src/lib.rs".to_string(),
            });
        }
    } else {
        // Ensure src/ directory exists
        let src_dir = crate_path.join("src");
        if !src_dir.exists() {
            plan.add(Operation::CreateDir {
                path: src_dir,
                description: "Create src directory".to_string(),
            });
        }
        plan.add(Operation::CreateFile {
            path: lib_rs_path,
            content: new_lib_rs,
            description: "Create src/lib.rs".to_string(),
        });
    }

    // Generate grammar/src/ from vendored grammar sources
    let grammar_dir = crate_path.join("grammar");

    if grammar_dir.exists() && grammar_dir.join("grammar.js").exists() {
        plan_grammar_src_generation(
            &mut plan,
            crate_path,
            config,
            cache,
            crates_dir,
            cache_hits,
            cache_misses,
            mode,
        )?;
    }

    Ok(plan)
}

/// Get the cross-grammar dependencies for a grammar.
/// Returns a list of (npm_package_name, arborium_crate_name) tuples.
fn get_grammar_dependencies(crate_name: &str) -> Vec<(&'static str, &'static str)> {
    match crate_name {
        "arborium-typescript" | "arborium-tsx" => {
            vec![("tree-sitter-javascript", "arborium-javascript")]
        }
        "arborium-cpp" => vec![("tree-sitter-c", "arborium-c")],
        "arborium-objc" => vec![("tree-sitter-c", "arborium-c")],
        "arborium-glsl" => vec![("tree-sitter-c", "arborium-c")],
        "arborium-hlsl" => vec![
            ("tree-sitter-cpp", "arborium-cpp"),
            ("tree-sitter-c", "arborium-c"),
        ],
        "arborium-scss" => vec![("tree-sitter-css", "arborium-css")],
        "arborium-svelte" => vec![("tree-sitter-html", "arborium-html")],
        "arborium-vue" => vec![("tree-sitter-html", "arborium-html")],
        "arborium-commonlisp" => vec![("tree-sitter-clojure", "arborium-clojure")],
        _ => vec![],
    }
}

/// Set up node_modules with copies of dependency grammars for tree-sitter generate.
/// This is only used during generation (dev time), not at crate build time.
fn setup_grammar_dependencies(
    temp_path: &Utf8Path,
    crates_dir: &Utf8Path,
    crate_name: &str,
) -> Result<(), Report> {
    let deps = get_grammar_dependencies(crate_name);
    if deps.is_empty() {
        return Ok(());
    }

    let node_modules = temp_path.join("node_modules");
    fs::create_dir_all(&node_modules)?;

    for (npm_name, arborium_name) in deps {
        let dep_grammar_dir = crates_dir.join(arborium_name).join("grammar");
        let target_dir = node_modules.join(npm_name);

        if dep_grammar_dir.exists() {
            // Copy the dependency's grammar files to node_modules
            copy_dir_contents(&dep_grammar_dir, &target_dir)?;
        }
    }

    Ok(())
}

/// Plan the generation of grammar/src/ by running tree-sitter generate in a temp directory.
#[allow(clippy::too_many_arguments)]
fn plan_grammar_src_generation(
    plan: &mut Plan,
    crate_path: &Utf8Path,
    _config: &crate::types::CrateConfig,
    cache: &GrammarCache,
    crates_dir: &Utf8Path,
    cache_hits: &AtomicUsize,
    cache_misses: &AtomicUsize,
    mode: PlanMode,
) -> Result<(), Report> {
    let grammar_dir = crate_path.join("grammar");
    let dest_src_dir = grammar_dir.join("src");
    let crate_name = crate_path.file_name().unwrap_or("unknown");

    // Compute cache key from input files
    let cache_key = cache.compute_cache_key(crate_path, crates_dir, crate_name)?;

    // Check cache first
    if let Some(cached) = cache.get(crate_name, &cache_key) {
        // Cache hit! Extract to a temp dir first, then plan updates
        cache_hits.fetch_add(1, Ordering::Relaxed);

        let temp_dir = tempfile::tempdir()?;
        let temp_src = Utf8PathBuf::from_path_buf(temp_dir.path().to_path_buf())
            .map_err(|_| std::io::Error::other("Non-UTF8 temp path"))?;

        cached.extract_to(&temp_src)?;

        // Plan updates from cached files
        plan_updates_from_generated(&mut *plan, &temp_src, &dest_src_dir, mode)?;
        return Ok(());
    }

    // Cache miss - need to generate
    cache_misses.fetch_add(1, Ordering::Relaxed);

    // Create a temp directory with same structure as the crate
    // Some grammars have `require('../common/...')` so we need to preserve the relative paths
    let temp_dir = tempfile::tempdir()?;
    let temp_root = Utf8PathBuf::from_path_buf(temp_dir.path().to_path_buf())
        .map_err(|_| std::io::Error::other("Non-UTF8 temp path"))?;

    // Copy grammar/ to temp/grammar/
    let temp_grammar = temp_root.join("grammar");
    copy_dir_contents(&grammar_dir, &temp_grammar)?;

    // Copy common/ to temp/common/ if it exists (some grammars share code via ../common/)
    let common_dir = crate_path.join("common");
    if common_dir.exists() {
        let temp_common = temp_root.join("common");
        copy_dir_contents(&common_dir, &temp_common)?;
    }

    // Set up cross-grammar dependencies if needed (in temp/grammar/node_modules/)
    setup_grammar_dependencies(&temp_grammar, crates_dir, crate_name)?;

    // Create src/ directory for grammars that generate files there (e.g., vim's keywords.h)
    fs::create_dir_all(temp_grammar.join("src"))?;

    // Run tree-sitter generate in the temp/grammar directory
    let tree_sitter = Tool::TreeSitter.find()?;
    let output = tree_sitter
        .command()
        .args(["generate"])
        .current_dir(&temp_grammar)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Show more context for debugging
        let error_lines: Vec<&str> = stderr.lines().take(20).collect();
        Err(std::io::Error::other(format!(
            "tree-sitter generate failed for {}:\n{}",
            crate_name,
            error_lines.join("\n")
        )))?;
    }

    // The generated files are in temp/grammar/src/
    let generated_src = temp_grammar.join("src");

    // Save to cache for next time
    if let Err(e) = cache.save(crate_name, &cache_key, &generated_src) {
        // Cache save failure is not fatal, just log it
        eprintln!("Warning: failed to cache {}: {}", crate_name, e);
    }

    // Plan updates from the generated files
    plan_updates_from_generated(plan, &generated_src, &dest_src_dir, mode)?;

    Ok(())
}

/// Plan file updates from a generated source directory to the destination.
fn plan_updates_from_generated(
    plan: &mut Plan,
    generated_src: &Utf8Path,
    dest_src_dir: &Utf8Path,
    mode: PlanMode,
) -> Result<(), Report> {
    // Ensure grammar/src/ directory exists in plan
    if !dest_src_dir.exists() {
        plan.add(Operation::CreateDir {
            path: dest_src_dir.to_owned(),
            description: "Create grammar/src directory".to_string(),
        });
    }

    // Copy all generated files to grammar/src/
    // This includes parser.c, scanner.c, grammar.json, node-types.json, and any .h files
    for entry in fs::read_dir(generated_src)? {
        let entry = entry?;
        let file_name = entry.file_name().to_string_lossy().to_string();
        let generated_file = Utf8PathBuf::from_path_buf(entry.path())
            .map_err(|_| std::io::Error::other("Non-UTF8 path"))?;

        // Skip directories (tree_sitter/ is handled separately)
        if !generated_file.is_file() {
            continue;
        }

        let dest_file = dest_src_dir.join(&file_name);
        let new_content = fs::read_to_string(&generated_file)?;
        plan_file_update(
            plan,
            &dest_file,
            new_content,
            &format!("src/{}", file_name),
            mode,
        )?;
    }

    // Copy tree_sitter/ directory
    let generated_tree_sitter = generated_src.join("tree_sitter");
    let dest_tree_sitter = dest_src_dir.join("tree_sitter");
    if generated_tree_sitter.exists() {
        // Ensure tree_sitter/ directory exists
        if !dest_tree_sitter.exists() {
            plan.add(Operation::CreateDir {
                path: dest_tree_sitter.clone(),
                description: "Create src/tree_sitter directory".to_string(),
            });
        }

        // Copy each file in tree_sitter/
        for entry in fs::read_dir(&generated_tree_sitter)? {
            let entry = entry?;
            let file_name = entry.file_name().to_string_lossy().to_string();
            let generated_file = Utf8PathBuf::from_path_buf(entry.path())
                .map_err(|_| std::io::Error::other("Non-UTF8 path"))?;
            let dest_file = dest_tree_sitter.join(&file_name);

            if generated_file.is_file() {
                let new_content = fs::read_to_string(&generated_file)?;
                plan_file_update(
                    plan,
                    &dest_file,
                    new_content,
                    &format!("src/tree_sitter/{}", file_name),
                    mode,
                )?;
            }
        }
    }

    Ok(())
}

/// Helper to plan a file update (create or update based on whether content changed).
/// In dry-run mode, reads old content for diffing.
/// In normal mode, uses blake3 hashing to check if update is needed.
fn plan_file_update(
    plan: &mut Plan,
    dest_path: &Utf8Path,
    new_content: String,
    description: &str,
    mode: PlanMode,
) -> Result<(), Report> {
    if dest_path.exists() {
        // Hash the new content
        let new_hash = blake3::hash(new_content.as_bytes());

        // Read and hash existing file
        let old_bytes = fs::read(dest_path)?;
        let old_hash = blake3::hash(&old_bytes);

        // Only update if hashes differ
        if old_hash != new_hash {
            let old_content = if mode.is_dry_run() {
                // In dry-run mode, we need the content for diffing
                Some(String::from_utf8_lossy(&old_bytes).into_owned())
            } else {
                None
            };

            plan.add(Operation::UpdateFile {
                path: dest_path.to_owned(),
                old_content,
                new_content,
                description: format!("Update {}", description),
            });
        }
    } else {
        plan.add(Operation::CreateFile {
            path: dest_path.to_owned(),
            content: new_content,
            description: format!("Create {}", description),
        });
    }
    Ok(())
}

/// Copy directory contents (files and subdirectories) from src to dest.
fn copy_dir_contents(src: &Utf8Path, dest: &Utf8Path) -> Result<(), Report> {
    fs::create_dir_all(dest)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = Utf8PathBuf::from_path_buf(entry.path())
            .map_err(|_| std::io::Error::other("Non-UTF8 path"))?;
        let dest_path = dest.join(entry.file_name().to_string_lossy().as_ref());

        if src_path.is_dir() {
            copy_dir_contents(&src_path, &dest_path)?;
        } else {
            fs::copy(&src_path, &dest_path)?;
        }
    }

    Ok(())
}

/// Generate Cargo.toml content for a grammar crate.
fn generate_cargo_toml(crate_name: &str, config: &crate::types::CrateConfig) -> String {
    let grammar_id = config
        .grammars
        .first()
        .map(|g| g.id.as_ref())
        .unwrap_or(crate_name.strip_prefix("arborium-").unwrap_or(crate_name));

    let _description = config
        .grammars
        .first()
        .and_then(|g| g.description.as_ref())
        .map(|d| d.as_ref())
        .unwrap_or_else(|| "tree-sitter grammar bindings");

    format!(
        r#"[package]
name = "{crate_name}"
version = "0.1.0"
edition = "2024"
description = "{grammar_id} grammar for arborium (tree-sitter bindings)"
license = "MIT"
repository = "https://github.com/bearcove/arborium"
keywords = ["tree-sitter", "{grammar_id}", "syntax-highlighting"]
categories = ["parsing", "text-processing"]

[lib]
path = "src/lib.rs"

[dependencies]
tree-sitter-patched-arborium = {{ version = "0.25.10", path = "../../tree-sitter" }}
arborium-sysroot = {{ version = "0.1.0", path = "../arborium-sysroot" }}

[dev-dependencies]
arborium-test-harness = {{ version = "0.1.0", path = "../arborium-test-harness" }}

[build-dependencies]
cc = {{ version = "1", features = ["parallel"] }}
"#
    )
}

/// Generate build.rs content for a grammar crate.
fn generate_build_rs(crate_name: &str, config: &crate::types::CrateConfig) -> String {
    let grammar = config.grammars.first();
    let has_scanner = grammar.map(|g| g.has_scanner()).unwrap_or(false);

    let c_symbol: String = grammar
        .and_then(|g| g.c_symbol.as_ref())
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            crate_name
                .strip_prefix("arborium-")
                .unwrap_or(crate_name)
                .replace('-', "_")
        });

    let scanner_section = if has_scanner {
        r#"    println!("cargo:rerun-if-changed=grammar/scanner.c");
"#
    } else {
        ""
    };

    let scanner_compile = if has_scanner {
        r#"
    build.file("grammar/scanner.c");"#
    } else {
        ""
    };

    format!(
        r#"fn main() {{
    let src_dir = "grammar/src";

    println!("cargo:rerun-if-changed={{}}/parser.c", src_dir);
{scanner_section}
    let mut build = cc::Build::new();

    build
        .include(src_dir)
        .include("grammar") // for common/ includes like "../common/scanner.h"
        .include(format!("{{}}/tree_sitter", src_dir))
        .warnings(false)
        .flag_if_supported("-Wno-unused-parameter")
        .flag_if_supported("-Wno-unused-but-set-variable")
        .flag_if_supported("-Wno-trigraphs");

    // For WASM builds, use our custom sysroot (provided by arborium crate via links = "arborium")
    let target = std::env::var("TARGET").unwrap_or_default();
    if target.contains("wasm")
        && let Ok(sysroot) = std::env::var("DEP_ARBORIUM_SYSROOT_PATH")
    {{
        build.include(&sysroot);
    }}

    build.file(format!("{{}}/parser.c", src_dir));{scanner_compile}

    build.compile("tree_sitter_{c_symbol}");
}}
"#
    )
}

/// Generate src/lib.rs content for a grammar crate.
fn generate_lib_rs(
    crate_name: &str,
    crate_path: &Utf8Path,
    config: &crate::types::CrateConfig,
) -> String {
    let grammar = config.grammars.first();
    let tests_cursed = grammar.map(|g| g.tests_cursed()).unwrap_or(false);

    let grammar_id = grammar
        .map(|g| g.id.as_ref())
        .unwrap_or_else(|| crate_name.strip_prefix("arborium-").unwrap_or(crate_name));

    let grammar_name = grammar
        .map(|g| g.name.as_ref())
        .unwrap_or(grammar_id)
        .to_uppercase();

    let c_symbol = grammar
        .and_then(|g| g.c_symbol.as_ref())
        .map(|s| s.to_string())
        .unwrap_or_else(|| grammar_id.replace('-', "_"));

    // Check if queries exist
    let highlights_exists = crate_path.join("queries/highlights.scm").exists();
    let injections_exists = crate_path.join("queries/injections.scm").exists();
    let locals_exists = crate_path.join("queries/locals.scm").exists();

    let highlights_query = if highlights_exists {
        format!(
            r#"/// The highlights query for {grammar_id}.
pub const HIGHLIGHTS_QUERY: &str = include_str!("../queries/highlights.scm");"#
        )
    } else {
        format!(
            r#"/// The highlights query for {grammar_id} (empty - no highlights available).
pub const HIGHLIGHTS_QUERY: &str = "";"#
        )
    };

    let injections_query = if injections_exists {
        format!(
            r#"/// The injections query for {grammar_id}.
pub const INJECTIONS_QUERY: &str = include_str!("../queries/injections.scm");"#
        )
    } else {
        format!(
            r#"/// The injections query for {grammar_id} (empty - no injections available).
pub const INJECTIONS_QUERY: &str = "";"#
        )
    };

    let locals_query = if locals_exists {
        format!(
            r#"/// The locals query for {grammar_id}.
pub const LOCALS_QUERY: &str = include_str!("../queries/locals.scm");"#
        )
    } else {
        format!(
            r#"/// The locals query for {grammar_id} (empty - no locals available).
pub const LOCALS_QUERY: &str = "";"#
        )
    };

    let test_module = if tests_cursed {
        String::new()
    } else {
        format!(
            r#"
#[cfg(test)]
mod tests {{
    use super::*;

    #[test]
    fn test_grammar() {{
        arborium_test_harness::test_grammar(
            language(),
            "{grammar_id}",
            HIGHLIGHTS_QUERY,
            INJECTIONS_QUERY,
            LOCALS_QUERY,
            env!("CARGO_MANIFEST_DIR"),
        );
    }}
}}
"#
        )
    };

    format!(
        r#"//! {grammar_name} grammar for tree-sitter
//!
//! This crate provides the {grammar_id} language grammar for use with tree-sitter.

use tree_sitter_patched_arborium::Language;

unsafe extern "C" {{
    fn tree_sitter_{c_symbol}() -> Language;
}}

/// Returns the {grammar_id} tree-sitter language.
pub fn language() -> Language {{
    unsafe {{ tree_sitter_{c_symbol}() }}
}}

{highlights_query}

{injections_query}

{locals_query}
{test_module}"#
    )
}
