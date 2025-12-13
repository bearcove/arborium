/// Generate command - regenerates crate files from arborium.kdl.
///
/// This command reads arborium.kdl files and generates:
/// - Cargo.toml
/// - build.rs
/// - src/lib.rs
/// - grammar/src/ (by running tree-sitter generate)
use crate::cache::GrammarCache;
use crate::plan::{Operation, Plan, PlanMode, PlanSet};
use crate::tool::Tool;
use crate::types::{CrateRegistry, CrateState};
use crate::util::find_repo_root;
use crate::version_store;
use camino::{Utf8Path, Utf8PathBuf};
use indicatif::{ProgressBar, ProgressStyle};

/// Options for the generate command.
pub struct GenerateOptions<'a> {
    /// Optional grammar name to regenerate (regenerates all if None)
    pub name: Option<&'a str>,
    /// Dry run mode vs execute mode
    pub mode: PlanMode,
    /// Version string for generated Cargo.toml files
    pub version: &'a str,
    /// Continue processing all crates even if some fail
    pub no_fail_fast: bool,
    /// Number of parallel jobs for tree-sitter generation
    pub jobs: usize,
}
use fs_err as fs;
use owo_colors::OwoColorize;
use rayon::prelude::*;
use rootcause::Report;
use sailfish::TemplateSimple;
use std::process::Stdio;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

// Sailfish templates - compiled at build time
#[derive(TemplateSimple)]
#[template(path = "validate_grammar.stpl.js")]
struct ValidateGrammarTemplate<'a> {
    grammar_path: &'a str,
}

#[derive(TemplateSimple)]
#[template(path = "cargo.stpl.toml")]
struct CargoTomlTemplate<'a> {
    crate_name: &'a str,
    workspace_version: &'a str,
    /// Major version for dependencies (e.g., "1" instead of "1.1.5")
    dep_version: &'a str,
    grammar_id: &'a str,
    grammar_name: &'a str,
    license: &'a str,
    tag: &'a str,
    shared_rel: &'a str,
    /// Crates to add as dependencies for highlight query inheritance
    highlights_prepend_deps: &'a [HighlightDep],
}

#[derive(TemplateSimple)]
#[template(path = "build.stpl.rs")]
struct BuildRsTemplate<'a> {
    has_scanner: bool,
    c_symbol: &'a str,
}

#[derive(TemplateSimple)]
#[template(path = "lib.stpl.rs")]
struct LibRsTemplate<'a> {
    grammar_id: &'a str,
    c_symbol: &'a str,
    highlights_exists: bool,
    injections_exists: bool,
    locals_exists: bool,
    tests_cursed: bool,
    /// Crate names to prepend highlights from, in order
    /// e.g. ["arborium_c"] for C++ inheriting from C
    highlights_prepend: Vec<String>,
}

#[derive(TemplateSimple)]
#[template(path = "readme.stpl.md")]
struct ReadmeTemplate<'a> {
    crate_name: &'a str,
    crate_name_snake: &'a str,
    grammar_name: &'a str,
    grammar_id: &'a str,
    upstream_repo: &'a str,
    upstream_url: &'a str,
    commit: &'a str,
    license: &'a str,
    description: &'a str,
    inventor: &'a str,
    year: u16,
    language_link: &'a str,
}

// Plugin crate templates
#[derive(TemplateSimple)]
#[template(path = "plugin_cargo.stpl.toml")]
struct PluginCargoTomlTemplate<'a> {
    grammar_id: &'a str,
    grammar_crate_name: &'a str,
    crate_rel: &'a str,
    shared_rel: &'a str,
}

#[derive(TemplateSimple)]
#[template(path = "plugin_lib.stpl.rs")]
struct PluginLibRsTemplate<'a> {
    grammar_id: &'a str,
    grammar_crate_name_snake: &'a str,
}

#[derive(TemplateSimple)]
#[template(path = "plugin_package.stpl.json")]
struct PluginPackageJsonTemplate<'a> {
    grammar_id: &'a str,
    grammar_name: &'a str,
    version: &'a str,
}

#[derive(TemplateSimple)]
#[template(path = "plugin_readme.stpl.md")]
struct PluginReadmeTemplate<'a> {
    grammar_id: &'a str,
    grammar_name: &'a str,
    description: &'a str,
    language_link: &'a str,
    inventor: &'a str,
    year: u16,
}

// docs.rs demo crate templates
#[derive(TemplateSimple)]
#[template(path = "docsrs_demo_cargo.stpl.toml")]
struct DocsrsDemoCargoTomlTemplate<'a> {
    version: &'a str,
}

#[derive(TemplateSimple)]
#[template(path = "docsrs_demo_header.stpl.html")]
struct DocsrsDemoHeaderTemplate {}

#[derive(TemplateSimple)]
#[template(path = "docsrs_demo_lib.stpl.rs")]
struct DocsrsDemoLibRsTemplate {}

#[derive(TemplateSimple)]
#[template(path = "docsrs_demo_readme.stpl.md")]
struct DocsrsDemoReadmeTemplate<'a> {
    version: &'a str,
}

/// Generate crate files for all or a specific grammar.
///
/// This follows the 5-function generation flow from generate.md:
/// 1. Load registry
/// 2. Prepare temp structures (SHARED by validation & generation)
/// 3. Validate all grammars
/// 4. Generate all grammars (tree-sitter)
/// 5. Generate all crates (Cargo.toml, lib.rs, etc.)
pub fn plan_generate(
    crates_dir: &Utf8Path,
    options: GenerateOptions<'_>,
) -> Result<PlanSet, Report> {
    use std::time::Instant;
    let total_start = Instant::now();

    // 1. Load Registry
    let registry = load_registry(crates_dir)?;

    // 2. Prepare temp structures (SHARED by validation & generation)
    let prepared = prepare_temp_structures(registry, crates_dir, options.name, options.version)?;

    if prepared.prepared_temps.is_empty() {
        println!("No grammars to process");
        return Ok(PlanSet::new());
    }

    // 3. Validate all grammars using prepared structures
    validate_all_grammars(&prepared)?;

    // 4. Generate all grammars using same prepared structures
    let generation_results =
        generate_all_grammars(&prepared, options.mode, options.no_fail_fast, options.jobs)?;

    // 5. Generate all crates using templates
    let plan_set = generate_all_crates(&prepared, &generation_results, options.mode)?;

    let total_elapsed = total_start.elapsed();

    // Print summary as checkmark line
    let regen = generation_results.cache_misses;
    let reused = generation_results.cache_hits;
    println!(
        "{} Generated {}, re-used {} ({:.2}s)",
        "✓".green(),
        regen,
        reused,
        total_elapsed.as_secs_f64()
    );

    Ok(plan_set)
}

/// Get the cross-grammar dependencies for a grammar.
/// Returns a list of (npm_package_name, arborium_crate_name) tuples.
fn get_grammar_dependencies(config: &crate::types::CrateConfig) -> Vec<(String, String)> {
    let mut deps = Vec::new();

    for grammar in &config.grammars {
        for dep in &grammar.dependencies {
            deps.push((dep.npm.clone(), dep.krate.clone()));
        }
    }

    deps
}

/// Set up node_modules with copies of dependency grammars for tree-sitter generate.
/// This is only used during generation (dev time), not at crate build time.
fn setup_grammar_dependencies(
    temp_path: &Utf8Path,
    crates_dir: &Utf8Path,
    config: &crate::types::CrateConfig,
) -> Result<(), Report> {
    let deps = get_grammar_dependencies(config);
    if deps.is_empty() {
        return Ok(());
    }

    let node_modules = temp_path.join("node_modules");
    fs::create_dir_all(&node_modules)?;

    // Try to find repo root to look for langs/ directory
    let repo_root = crates_dir.parent().expect("crates_dir should have parent");
    let langs_dir = repo_root.join("langs");

    for (npm_name, arborium_name) in deps {
        let target_dir = node_modules.join(&npm_name);

        // Extract language name from arborium crate name
        let lang_name = arborium_name
            .strip_prefix("arborium-")
            .unwrap_or(&arborium_name);

        // Try new structure first: langs/group-*/lang/def/grammar
        let mut dep_grammar_dir = None;
        if langs_dir.exists() {
            // Search through all groups for this language
            if let Ok(entries) = fs::read_dir(&langs_dir) {
                for group_entry in entries.flatten() {
                    let group_path = group_entry.path();
                    if group_path.is_dir() {
                        let lang_path = group_path.join(lang_name);
                        let grammar_path = lang_path.join("def").join("grammar");
                        if grammar_path.exists() {
                            dep_grammar_dir = Some(
                                Utf8PathBuf::from_path_buf(grammar_path).expect("non-UTF8 path"),
                            );
                            break;
                        }
                    }
                }
            }
        }

        // Fall back to old structure: crates/arborium-*/grammar
        if dep_grammar_dir.is_none() {
            let old_path = crates_dir.join(&arborium_name).join("grammar");
            if old_path.exists() {
                dep_grammar_dir = Some(old_path);
            }
        }

        if let Some(grammar_dir) = dep_grammar_dir {
            // Copy the dependency's grammar files to node_modules
            copy_dir_contents(&grammar_dir, &target_dir)?;
        }
    }

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

        let new_content = fs::read_to_string(&generated_file)?;

        // Copy to def/grammar/src/
        let dest_file = dest_src_dir.join(&file_name);
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

            if generated_file.is_file() {
                let new_content = fs::read_to_string(&generated_file)?;

                // Copy to def/grammar/src/tree_sitter/
                let dest_file = dest_tree_sitter.join(&file_name);
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

fn copy_dir_contents(src_dir: &Utf8Path, dest_dir: &Utf8Path) -> Result<(), Report> {
    fs::create_dir_all(dest_dir)?;

    for entry in fs::read_dir(src_dir)? {
        let entry = entry?;
        let src_path = Utf8PathBuf::from_path_buf(entry.path())
            .map_err(|_| std::io::Error::other("Non-UTF8 path"))?;
        let dest_path = dest_dir.join(entry.file_name().to_string_lossy().as_ref());

        if src_path.is_dir() {
            copy_dir_contents(&src_path, &dest_path)?;
        } else {
            fs::copy(&src_path, &dest_path)?;
        }
    }

    Ok(())
}

/// Check if a file is a source file that should be copied to the crate.
fn is_grammar_source_file(file_name: &str) -> bool {
    file_name.ends_with(".c")
        || file_name.ends_with(".cc")
        || file_name.ends_with(".cpp")
        || file_name.ends_with(".h")
        || file_name.ends_with(".hpp")
}

/// Recursively plan copying of grammar source files from src_dir to dest_dir.
/// Skips the src/ subdirectory (generated files) and non-source files.
/// If copy_src_headers is true, also copies hand-written .h files from src/.
fn plan_copy_grammar_sources(
    plan: &mut Plan,
    src_dir: &Utf8Path,
    dest_dir: &Utf8Path,
    mode: PlanMode,
) -> Result<(), Report> {
    plan_copy_grammar_sources_inner(plan, src_dir, dest_dir, mode, true)
}

fn plan_copy_grammar_sources_inner(
    plan: &mut Plan,
    src_dir: &Utf8Path,
    dest_dir: &Utf8Path,
    mode: PlanMode,
    is_top_level: bool,
) -> Result<(), Report> {
    if !src_dir.exists() {
        return Ok(());
    }

    let mut needs_dest_dir = false;
    let ensure_dest_dir = |plan: &mut Plan, needs: &mut bool| {
        if !*needs {
            if !dest_dir.exists() {
                plan.add(Operation::CreateDir {
                    path: dest_dir.to_owned(),
                    description: format!(
                        "Create {}",
                        dest_dir.file_name().unwrap_or(dest_dir.as_str())
                    ),
                });
            }
            *needs = true;
        }
    };

    for entry in fs::read_dir(src_dir)? {
        let entry = entry?;
        let src_path = Utf8PathBuf::from_path_buf(entry.path())
            .map_err(|_| std::io::Error::other("Non-UTF8 path"))?;
        let file_name = entry.file_name().to_string_lossy().to_string();

        if src_path.is_dir() {
            // Skip grammar/ subdirectories (these are JS helper files for tree-sitter generation)
            // Skip node_modules/ and other non-source directories
            if file_name == "grammar" || file_name == "node_modules" || file_name == "lib" {
                continue;
            }

            // Special handling for src/ directory at top level:
            // - The generated files (parser.c, etc.) are handled by plan_updates_from_generated
            // - But we need to copy any hand-written .h files that live alongside generated files
            //   (like vim's keywords.h)
            if file_name == "src" && is_top_level {
                let src_src_dir = src_dir.join("src");
                let dest_src_dir = dest_dir.join("src");
                // Only copy .h files from src/ - .c files are generated
                for src_entry in fs::read_dir(&src_src_dir)? {
                    let src_entry = src_entry?;
                    let src_file_path = Utf8PathBuf::from_path_buf(src_entry.path())
                        .map_err(|_| std::io::Error::other("Non-UTF8 path"))?;
                    let src_file_name = src_entry.file_name().to_string_lossy().to_string();

                    // Only copy .h files (hand-written headers like keywords.h)
                    // Skip .c files (generated by tree-sitter)
                    // Skip directories like tree_sitter/ (generated)
                    if src_file_path.is_file() && src_file_name.ends_with(".h") {
                        ensure_dest_dir(plan, &mut needs_dest_dir);
                        if !dest_src_dir.exists() {
                            plan.add(Operation::CreateDir {
                                path: dest_src_dir.clone(),
                                description: "Create grammar/src directory".to_string(),
                            });
                        }
                        let new_content = fs::read_to_string(&src_file_path)?;
                        let dest_path = dest_src_dir.join(&src_file_name);
                        plan_file_update(
                            plan,
                            &dest_path,
                            new_content,
                            &format!("grammar/src/{}", src_file_name),
                            mode,
                        )?;
                    }
                }
                continue;
            }

            // Recursively copy subdirectories like include/, common/, rules/
            let sub_dest = dest_dir.join(&file_name);
            plan_copy_grammar_sources_inner(plan, &src_path, &sub_dest, mode, false)?;
        } else if is_grammar_source_file(&file_name) {
            ensure_dest_dir(plan, &mut needs_dest_dir);

            let new_content = fs::read_to_string(&src_path)?;
            let dest_path = dest_dir.join(&file_name);
            let desc = dest_path
                .strip_prefix(dest_dir.parent().unwrap_or(dest_dir))
                .unwrap_or(&dest_path)
                .to_string();

            plan_file_update(plan, &dest_path, new_content, &desc, mode)?;
        }
    }

    Ok(())
}

/// Recursively copy all files from src_dir to dest_dir.
fn plan_copy_dir_recursive(
    plan: &mut Plan,
    src_dir: &Utf8Path,
    dest_dir: &Utf8Path,
    mode: PlanMode,
) -> Result<(), Report> {
    if !src_dir.exists() {
        return Ok(());
    }

    // Create destination directory
    if !dest_dir.exists() {
        plan.add(Operation::CreateDir {
            path: dest_dir.to_owned(),
            description: format!(
                "Create {}",
                dest_dir.file_name().unwrap_or(dest_dir.as_str())
            ),
        });
    }

    for entry in fs::read_dir(src_dir)? {
        let entry = entry?;
        let src_path = Utf8PathBuf::try_from(entry.path())?;
        let file_name = src_path.file_name().unwrap_or("");

        if src_path.is_dir() {
            // Recursively copy subdirectories
            let sub_dest = dest_dir.join(file_name);
            plan_copy_dir_recursive(plan, &src_path, &sub_dest, mode)?;
        } else {
            // Copy file
            let content = fs::read_to_string(&src_path)?;
            let dest_path = dest_dir.join(file_name);
            plan_file_update(
                plan,
                &dest_path,
                content,
                &format!("samples/{}", file_name),
                mode,
            )?;
        }
    }

    Ok(())
}

/// Generate Cargo.toml content for a grammar crate.
fn generate_cargo_toml(
    crate_name: &str,
    config: &crate::types::CrateConfig,
    workspace_version: &str,
    shared_rel: &str,
    highlights_prepend_deps: &[HighlightDep],
) -> String {
    let grammar = config.grammars.first();

    let grammar_id = grammar
        .map(|g| g.id.as_ref())
        .unwrap_or(crate_name.strip_prefix("arborium-").unwrap_or(crate_name));

    let grammar_name = grammar.map(|g| g.name.as_ref()).unwrap_or(grammar_id);

    let tag = grammar
        .map(|g| g.tag.value.as_str())
        .unwrap_or("programming");

    // Use license from arborium.kdl, fallback to MIT if empty
    let license: &str = {
        let l: &str = config.license.value.as_ref();
        if l.is_empty() { "MIT" } else { l }
    };

    // Use full version for dependencies (e.g., "1.1.5" -> "1.1.5")
    let dep_version = workspace_version;

    let template = CargoTomlTemplate {
        crate_name,
        workspace_version,
        dep_version,
        grammar_id,
        grammar_name,
        license,
        tag,
        shared_rel,
        highlights_prepend_deps,
    };
    template
        .render_once()
        .expect("CargoTomlTemplate render failed")
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

    let template = BuildRsTemplate {
        has_scanner,
        c_symbol: &c_symbol,
    };
    template
        .render_once()
        .expect("BuildRsTemplate render failed")
}

/// Generate src/lib.rs content for a grammar crate.
fn generate_lib_rs(
    crate_name: &str,
    def_path: &Utf8Path,
    config: &crate::types::CrateConfig,
    highlights_prepend: Vec<String>,
) -> String {
    let grammar = config.grammars.first();
    let tests_cursed = grammar.map(|g| g.tests_cursed()).unwrap_or(false);

    let grammar_id = grammar
        .map(|g| g.id.as_ref())
        .unwrap_or_else(|| crate_name.strip_prefix("arborium-").unwrap_or(crate_name));

    let c_symbol = grammar
        .and_then(|g| g.c_symbol.as_ref())
        .map(|s| s.to_string())
        .unwrap_or_else(|| grammar_id.replace('-', "_"));

    // Check if queries exist (in def/queries/)
    let highlights_exists = def_path.join("queries/highlights.scm").exists();
    let injections_exists = def_path.join("queries/injections.scm").exists();
    let locals_exists = def_path.join("queries/locals.scm").exists();

    let template = LibRsTemplate {
        grammar_id,
        c_symbol: &c_symbol,
        highlights_exists,
        injections_exists,
        locals_exists,
        tests_cursed,
        highlights_prepend,
    };
    template.render_once().expect("LibRsTemplate render failed")
}

/// Generate README.md content for a grammar crate.
fn generate_readme(crate_name: &str, config: &crate::types::CrateConfig) -> String {
    let grammar = config.grammars.first();

    let grammar_id = grammar
        .map(|g| g.id.as_ref())
        .unwrap_or_else(|| crate_name.strip_prefix("arborium-").unwrap_or(crate_name));

    let grammar_name = grammar.map(|g| g.name.as_ref()).unwrap_or(grammar_id);

    let upstream_url: &str = config.repo.value.as_ref();
    // Extract repo name from URL for display
    let upstream_repo = upstream_url
        .strip_prefix("https://github.com/")
        .unwrap_or(upstream_url);

    let commit: &str = config.commit.value.as_ref();

    let license: &str = {
        let l: &str = config.license.value.as_ref();
        if l.is_empty() { "MIT" } else { l }
    };

    // Extract optional grammar metadata
    let description = grammar
        .and_then(|g| g.description.as_ref())
        .map(|d| d.value.as_str())
        .unwrap_or("");

    let inventor = grammar
        .and_then(|g| g.inventor.as_ref())
        .map(|i| i.value.as_str())
        .unwrap_or("");

    let year = grammar
        .and_then(|g| g.year.as_ref())
        .map(|y| y.value)
        .unwrap_or(0);

    let language_link = grammar
        .and_then(|g| g.link.as_ref())
        .map(|l| l.value.as_str())
        .unwrap_or_else(|| upstream_url);

    let crate_name_snake = crate_name.replace('-', "_");

    let template = ReadmeTemplate {
        crate_name,
        crate_name_snake: &crate_name_snake,
        grammar_name,
        grammar_id,
        upstream_repo,
        upstream_url,
        commit,
        license,
        description,
        inventor,
        year,
        language_link,
    };
    template
        .render_once()
        .expect("ReadmeTemplate render failed")
}

/// Generate plugin Cargo.toml content.
fn generate_plugin_cargo_toml(grammar_id: &str, grammar_crate_name: &str) -> String {
    // Paths relative to npm/:
    // npm/ is at langs/group-*/lang/npm/
    // crate/ is at langs/group-*/lang/crate/ (sibling)
    // shared crates are at crates/ (repo root)
    // So: npm -> lang -> group-* -> langs -> repo-root -> crates
    let crate_rel = "../crate";
    let shared_rel = "../../../../crates";

    let template = PluginCargoTomlTemplate {
        grammar_id,
        grammar_crate_name,
        crate_rel,
        shared_rel,
    };
    template
        .render_once()
        .expect("PluginCargoTomlTemplate render failed")
}

/// Generate plugin src/lib.rs content.
fn generate_plugin_lib_rs(grammar_id: &str, grammar_crate_name: &str) -> String {
    let grammar_crate_name_snake = grammar_crate_name.replace('-', "_");

    let template = PluginLibRsTemplate {
        grammar_id,
        grammar_crate_name_snake: &grammar_crate_name_snake,
    };
    template
        .render_once()
        .expect("PluginLibRsTemplate render failed")
}

/// Generate plugin package.json content.
fn generate_plugin_package_json(grammar_id: &str, grammar_name: &str, version: &str) -> String {
    let template = PluginPackageJsonTemplate {
        grammar_id,
        grammar_name,
        version,
    };
    template
        .render_once()
        .expect("PluginPackageJsonTemplate render failed")
}

/// Generate plugin README.md content.
fn generate_plugin_readme(
    grammar_id: &str,
    grammar_name: &str,
    description: &str,
    language_link: &str,
    inventor: &str,
    year: u16,
) -> String {
    let template = PluginReadmeTemplate {
        grammar_id,
        grammar_name,
        description,
        language_link,
        inventor,
        year,
    };
    template
        .render_once()
        .expect("PluginReadmeTemplate render failed")
}

// Data structures for temp directory preparation (shared by validation & generation)
struct PreparedTemp {
    crate_state: CrateState,
    temp_dir: tempfile::TempDir,
    temp_root: Utf8PathBuf,
    temp_grammar: Utf8PathBuf,
    config: crate::types::CrateConfig,
}

struct PreparedStructures {
    prepared_temps: Vec<PreparedTemp>,
    repo_root: Utf8PathBuf,
    cache: GrammarCache,
    workspace_version: String,
    /// Full crate registry for path resolution (includes all crates, not just those being generated)
    registry: CrateRegistry,
}

struct GenerationResults {
    cache_hits: usize,
    cache_misses: usize,
    plans: PlanSet,
}

// 1. Load Registry
fn load_registry(crates_dir: &Utf8Path) -> Result<CrateRegistry, Report> {
    CrateRegistry::load(crates_dir)
}

// 2. Prepare Temp Structures (SHARED by validation & generation)
fn prepare_temp_structures(
    registry: CrateRegistry,
    crates_dir: &Utf8Path,
    name: Option<&str>,
    version: &str,
) -> Result<PreparedStructures, Report> {
    // Set up repo root and cache
    let repo_root =
        find_repo_root().ok_or_else(|| std::io::Error::other("Could not find repo root"))?;
    let repo_root = Utf8PathBuf::from_path_buf(repo_root)
        .map_err(|_| std::io::Error::other("Non-UTF8 repo root"))?;
    let cache = GrammarCache::new(&repo_root);

    // Record canonical version
    version_store::write_version(&repo_root, version)
        .map_err(|e| rootcause::Report::new(std::io::Error::other(e.to_string())))?;

    // Prepare temp directories for all crates that have grammar.js files
    let mut prepared_temps = Vec::new();

    for crate_state in registry.crates.values() {
        // Skip if a specific name was requested and this isn't it
        if let Some(filter) = name {
            let matches = crate_state.name == filter
                || (crate_state.name.strip_prefix("arborium-") == Some(filter));
            if !matches {
                continue;
            }
        }
        // Skip crates without arborium.kdl
        let Some(config) = crate_state.config.clone() else {
            continue;
        };
        let grammar_js = crate_state.def_path.join("grammar/grammar.js");

        if !grammar_js.exists() {
            continue;
        }

        // Create temp directory with proper structure (shared by validation and generation)
        let temp_dir = tempfile::tempdir()?;
        let temp_root = Utf8PathBuf::from_path_buf(temp_dir.path().to_path_buf())
            .map_err(|_| std::io::Error::other("Non-UTF8 temp path"))?;
        let temp_grammar = temp_root.join("grammar");

        // Copy grammar files to temp
        let def_path = &crate_state.def_path;
        let grammar_dir = def_path.join("grammar");
        copy_dir_contents(&grammar_dir, &temp_grammar)?;

        // Copy common/ to temp/common/ if it exists (some grammars share code via ../common/)
        // Check both def/common (shared at language level) and def/grammar/common (local to grammar)
        let def_common_dir = def_path.join("common");
        let grammar_common_dir = def_path.join("grammar/common");

        if def_common_dir.exists() {
            let temp_common = temp_root.join("common");
            copy_dir_contents(&def_common_dir, &temp_common)?;
        }

        if grammar_common_dir.exists() {
            let temp_common = temp_root.join("common");
            copy_dir_contents(&grammar_common_dir, &temp_common)?;
        }

        // Set up cross-grammar dependencies
        setup_grammar_dependencies(&temp_grammar, crates_dir, &config)?;

        prepared_temps.push(PreparedTemp {
            crate_state: (*crate_state).clone(),
            temp_dir,
            temp_root,
            temp_grammar,
            config,
        });
    }

    Ok(PreparedStructures {
        prepared_temps,
        repo_root,
        cache,
        workspace_version: version.to_string(),
        registry,
    })
}

// 3. Validate All Grammars using prepared structures
fn validate_all_grammars(prepared: &PreparedStructures) -> Result<(), Report> {
    let total = prepared.prepared_temps.len();
    let pb = ProgressBar::new(total as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} Validating {msg}")
            .unwrap()
            .progress_chars("━━╸"),
    );

    for prepared_temp in &prepared.prepared_temps {
        let short_name = prepared_temp
            .crate_state
            .name
            .strip_prefix("arborium-")
            .unwrap_or(&prepared_temp.crate_state.name);
        pb.set_message(short_name.to_string());
        validate_single_grammar(prepared_temp)?;
        pb.inc(1);
    }

    pb.finish_and_clear();
    println!("{} Validated {} grammars", "✓".green(), total);
    Ok(())
}

fn validate_single_grammar(prepared_temp: &PreparedTemp) -> Result<(), Report> {
    // Create wrapper script
    let wrapper_path = prepared_temp.temp_dir.path().join("validate_grammar.js");
    let temp_grammar_js = prepared_temp.temp_grammar.join("grammar.js");

    let template = ValidateGrammarTemplate {
        grammar_path: &temp_grammar_js.as_str().replace('\\', "\\\\"),
    };
    let wrapper_content = template.render_once()?;

    fs::write(&wrapper_path, wrapper_content)?;

    // Run Node.js on the wrapper
    let output = std::process::Command::new("node")
        .arg(&wrapper_path)
        .current_dir(&prepared_temp.temp_root)
        .output()
        .map_err(|e| std::io::Error::other(format!("Failed to run node: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(std::io::Error::other(format!(
            "Grammar validation failed for {}: {}",
            prepared_temp
                .crate_state
                .name
                .strip_prefix("arborium-")
                .unwrap_or(&prepared_temp.crate_state.name),
            stderr.trim()
        ))
        .into());
    }

    Ok(())
}

// 4. Generate All Grammars using same prepared structures
fn generate_all_grammars(
    prepared: &PreparedStructures,
    mode: PlanMode,
    no_fail_fast: bool,
    jobs: usize,
) -> Result<GenerationResults, Report> {
    let cache_hits = AtomicUsize::new(0);
    let cache_misses = AtomicUsize::new(0);
    let plans = Mutex::new(PlanSet::new());
    let errors: Mutex<Vec<(String, Report)>> = Mutex::new(Vec::new());
    let first_error_seen = AtomicUsize::new(0); // 0 = no error, 1 = error seen

    // Set up progress bar
    let total = prepared.prepared_temps.len() as u64;
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("━━╸"),
    );
    pb.set_message("Generating grammars...");

    // Process grammars in parallel using prepared temp directories
    let process_grammar = |prepared_temp: &PreparedTemp| {
        // In fail-fast mode, skip if we already saw an error
        if !no_fail_fast && first_error_seen.load(Ordering::Relaxed) == 1 {
            pb.inc(1);
            return;
        }

        let crate_name = &prepared_temp.crate_state.name;
        let short_name = crate_name.strip_prefix("arborium-").unwrap_or(crate_name);
        pb.set_message(format!("{}", short_name));

        let result = plan_grammar_generation_with_prepared_temp(
            prepared_temp,
            &prepared.cache,
            &prepared.repo_root,
            mode,
        );

        match result {
            Ok((plan, hit)) => {
                if hit {
                    cache_hits.fetch_add(1, Ordering::Relaxed);
                } else {
                    cache_misses.fetch_add(1, Ordering::Relaxed);
                }
                plans.lock().unwrap().add(plan);
            }
            Err(e) => {
                first_error_seen.store(1, Ordering::Relaxed);
                errors.lock().unwrap().push((crate_name.clone(), e));
            }
        }
        pb.inc(1);
    };

    // Always parallel, with configurable thread pool
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(jobs)
        .build()
        .expect("Failed to build thread pool");

    pool.install(|| {
        prepared.prepared_temps.par_iter().for_each(process_grammar);
    });

    pb.finish_and_clear();

    // Check for errors
    let errors = errors.into_inner().unwrap();
    if !errors.is_empty() {
        for (crate_name, error) in &errors {
            eprintln!("Error: {}: {}", crate_name.bold(), error);
        }
        return Err(std::io::Error::other(format!(
            "{} grammar(s) failed to generate",
            errors.len()
        ))
        .into());
    }

    Ok(GenerationResults {
        cache_hits: cache_hits.load(Ordering::Relaxed),
        cache_misses: cache_misses.load(Ordering::Relaxed),
        plans: plans.into_inner().unwrap(),
    })
}

// Helper function to generate a single grammar using prepared temp directory
fn plan_grammar_generation_with_prepared_temp(
    prepared_temp: &PreparedTemp,
    cache: &GrammarCache,
    repo_root: &Utf8Path,
    mode: PlanMode,
) -> Result<(Plan, bool), Report> {
    let crate_name = &prepared_temp.crate_state.name;
    let def_path = &prepared_temp.crate_state.def_path;
    let crate_path = &prepared_temp.crate_state.crate_path;

    // Use the already prepared temp directory instead of creating a new one
    let temp_root = &prepared_temp.temp_root;
    let temp_grammar = &prepared_temp.temp_grammar;

    // Destination is grammar/src/ under the definition path
    let dest_src_dir = def_path.join("grammar/src");
    // Also copy to crate/grammar/src/ so the published crate is self-contained.
    // We must plan this here (not in plan_crate_files_only) because that function
    // reads from def/grammar/src/ at plan-building time, but those files don't
    // exist yet in CI (they're gitignored and only created when this plan executes).
    let crate_grammar_src_dir = crate_path.join("grammar/src");

    // Compute cache key
    // Note: compute_cache_key takes (def_path, crates_dir, config) but we only need def_path and config
    // The crates_dir is used for dependency resolution which is already handled in prepared temps
    let crates_dir = repo_root.join("crates");
    let cache_key = cache.compute_cache_key(def_path, &crates_dir, &prepared_temp.config)?;

    if let Some(cached_files) = cache.get(crate_name, &cache_key) {
        // Cache hit - skip tree-sitter generate, but still plan grammar/src updates
        let temp_src = temp_root.join("cached_src");
        cached_files.extract_to(&temp_src)?;

        let mut plan = Plan::for_crate(crate_name);
        plan_updates_from_generated(&mut plan, &temp_src, &dest_src_dir, mode)?;
        // Also copy to crate/grammar/src/
        plan_updates_from_generated(&mut plan, &temp_src, &crate_grammar_src_dir, mode)?;

        return Ok((plan, true)); // true = cache hit
    }

    // Cache miss - run tree-sitter generate in the prepared temp directory
    // Create src/ directory for grammars that generate files there
    fs::create_dir_all(temp_grammar.join("src"))?;

    // Run tree-sitter generate
    let tree_sitter = Tool::TreeSitter.find()?;

    let output = tree_sitter
        .command()
        .args(["generate"])
        .current_dir(temp_grammar)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let error_lines: Vec<&str> = stderr.lines().take(20).collect();
        return Err(std::io::Error::other(format!(
            "tree-sitter generate failed for {}:\n{}",
            crate_name,
            error_lines.join("\n")
        ))
        .into());
    }

    // The generated files are in temp/grammar/src/
    let generated_src = temp_grammar.join("src");

    // Save to cache for next time
    if let Err(e) = cache.save(crate_name, &cache_key, &generated_src) {
        eprintln!("Warning: failed to cache {}: {}", crate_name, e);
    }

    // Plan file updates to both def/grammar/src/ and crate/grammar/src/
    let mut plan = Plan::for_crate(crate_name);
    plan_updates_from_generated(&mut plan, &generated_src, &dest_src_dir, mode)?;
    // Also copy to crate/grammar/src/
    plan_updates_from_generated(&mut plan, &generated_src, &crate_grammar_src_dir, mode)?;

    Ok((plan, false)) // false = cache miss
}

/// Resolve a crate name to its path relative to another crate's directory.
/// E.g., from cpp/crate/ to c/crate/ returns "../../c/crate"
fn resolve_crate_relative_path(
    from_crate_path: &Utf8Path,
    target_crate_name: &str,
    prepared: &PreparedStructures,
) -> Option<String> {
    // Find the target crate in the full registry (not just prepared temps)
    let target_state = prepared.registry.crates.get(target_crate_name)?;
    let target_path = &target_state.crate_path;

    // Calculate relative path from from_crate_path to target_path
    // Both paths are absolute, so we need to find common ancestor and build relative path
    let from_components: Vec<_> = from_crate_path.components().collect();
    let to_components: Vec<_> = target_path.components().collect();

    // Find common prefix length
    let common_len = from_components
        .iter()
        .zip(to_components.iter())
        .take_while(|(a, b)| a == b)
        .count();

    // Build relative path: go up (from_components.len() - common_len) levels, then down to target
    let up_count = from_components.len() - common_len;
    let mut rel_parts: Vec<&str> = vec![".."; up_count];
    for comp in &to_components[common_len..] {
        rel_parts.push(comp.as_str());
    }

    Some(rel_parts.join("/"))
}

/// A dependency to add to Cargo.toml for highlight query inheritance.
#[derive(Debug, Clone)]
struct HighlightDep {
    /// Crate name (e.g., "arborium-c")
    crate_name: String,
    /// Relative path from the dependent crate (e.g., "../../c/crate")
    rel_path: String,
}

/// Result of extracting highlight prepend configuration.
#[derive(Debug, Clone, Default)]
struct HighlightPrepends {
    /// Dependencies to add to Cargo.toml
    cargo_deps: Vec<HighlightDep>,
    /// Rust identifiers for lib.rs (e.g., "arborium_c")
    lib_prepends: Vec<String>,
}

/// Extract highlight prepend configuration from a grammar config.
fn extract_highlights_prepend(
    config: &crate::types::CrateConfig,
    from_crate_path: &Utf8Path,
    registry: &PreparedStructures,
) -> HighlightPrepends {
    let mut result = HighlightPrepends::default();

    let grammar = match config.grammars.first() {
        Some(g) => g,
        None => return result,
    };

    let queries = match &grammar.queries {
        Some(q) => q,
        None => return result,
    };

    let highlights = match &queries.highlights {
        Some(h) => h,
        None => return result,
    };

    for prepend in &highlights.prepend {
        let crate_name = &prepend.crate_name.value;

        // Resolve relative path for Cargo.toml
        if let Some(rel_path) = resolve_crate_relative_path(from_crate_path, crate_name, registry) {
            // Prepend deps are other grammar crates, which use the workspace version
            result.cargo_deps.push(HighlightDep {
                crate_name: crate_name.clone(),
                rel_path,
            });
        }

        // Convert crate name to Rust identifier for lib.rs (e.g., "arborium-c" -> "arborium_c")
        let rust_ident = crate_name.replace('-', "_");
        result.lib_prepends.push(rust_ident);
    }

    result
}

// 5. Generate All Crates using templates (Cargo.toml, build.rs, lib.rs, README.md)
// NOTE: This does NOT run tree-sitter - grammar generation is done in step 4
fn generate_all_crates(
    prepared: &PreparedStructures,
    generation_results: &GenerationResults,
    mode: PlanMode,
) -> Result<PlanSet, Report> {
    let mut final_plan = generation_results.plans.clone();

    // Generate crate files (Cargo.toml, build.rs, lib.rs, README.md) for all crates
    for prepared_temp in &prepared.prepared_temps {
        let crate_state = &prepared_temp.crate_state;
        let config = &prepared_temp.config;

        let crate_plan = plan_crate_files_only(
            crate_state,
            config,
            &prepared.workspace_version,
            prepared,
            mode,
        )?;
        final_plan.add(crate_plan);

        // Generate plugin crate files for grammars that have generate-component enabled
        let plugin_plan = plan_plugin_crate_files(
            crate_state,
            config,
            &prepared.repo_root,
            &prepared.workspace_version,
        )?;
        final_plan.add(plugin_plan);
    }

    // Generate umbrella crate (crates/arborium/Cargo.toml)
    let umbrella_plan = plan_umbrella_crate(prepared)?;
    final_plan.add(umbrella_plan);

    // Update shared crates to use the workspace version
    let shared_plan = plan_shared_crates(prepared, mode)?;
    final_plan.add(shared_plan);

    // Generate docs.rs demo crate
    let demo_plan = plan_docsrs_demo_crate(prepared, mode)?;
    final_plan.add(demo_plan);

    Ok(final_plan)
}

/// Generate only the Rust crate files (Cargo.toml, build.rs, lib.rs, README.md)
/// This does NOT run tree-sitter generate - that's handled separately in step 4.
fn plan_crate_files_only(
    crate_state: &CrateState,
    config: &crate::types::CrateConfig,
    workspace_version: &str,
    registry: &PreparedStructures,
    mode: PlanMode,
) -> Result<Plan, Report> {
    let mut plan = Plan::for_crate(&crate_state.name);
    let def_path = &crate_state.def_path;
    let crate_path = &crate_state.crate_path;

    // crate/ is at langs/group-*/lang/crate/
    // shared crates are at crates/ (repo root)
    // So: crate -> lang -> group-* -> langs -> repo-root -> crates
    let shared_rel = "../../../../crates";

    // Extract highlights prepend configuration
    let highlight_prepends = extract_highlights_prepend(config, crate_path, registry);

    // Ensure crate directory exists
    if !crate_path.exists() {
        plan.add(Operation::CreateDir {
            path: crate_path.to_owned(),
            description: "Create crate directory".to_string(),
        });
    }

    // Generate Cargo.toml
    let cargo_toml_path = crate_path.join("Cargo.toml");
    let new_cargo_toml = generate_cargo_toml(
        &crate_state.name,
        config,
        workspace_version,
        shared_rel,
        &highlight_prepends.cargo_deps,
    );

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

    // Generate README.md
    let readme_path = crate_path.join("README.md");
    let new_readme = generate_readme(&crate_state.name, config);

    if readme_path.exists() {
        let old_content = fs::read_to_string(&readme_path)?;
        if old_content != new_readme {
            plan.add(Operation::UpdateFile {
                path: readme_path,
                old_content: Some(old_content),
                new_content: new_readme,
                description: "Update README.md".to_string(),
            });
        }
    } else {
        plan.add(Operation::CreateFile {
            path: readme_path,
            content: new_readme,
            description: "Create README.md".to_string(),
        });
    }

    // Generate src/lib.rs
    let lib_rs_path = crate_path.join("src/lib.rs");
    let new_lib_rs = generate_lib_rs(
        &crate_state.name,
        def_path,
        config,
        highlight_prepends.lib_prepends,
    );

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

    // Mirror hand-written grammar sources into the crate so that the published
    // package is self-contained for crates.io verification builds.
    //
    // Layout we want inside crate root:
    //   grammar/
    //     scanner.c      (optional, hand-written)
    //     common/        (optional, hand-written headers)
    //     src/
    //       parser.c     (generated by tree-sitter, copied in grammar generation step)
    //       grammar.json (generated by tree-sitter, copied in grammar generation step)
    //       node-types.json (generated, copied in grammar generation step)
    //       tree_sitter/  (generated headers, copied in grammar generation step)
    //
    // NOTE: The generated files in grammar/src/ are copied during grammar generation
    // (plan_grammar_generation_with_prepared_temp), not here. This is necessary because
    // in CI, def/grammar/src/ doesn't exist at plan-building time (it's gitignored and
    // only created when the grammar generation plan executes).
    let def_grammar_dir = def_path.join("grammar");
    let crate_grammar_dir = crate_path.join("grammar");

    // Recursively copy all hand-written source files from def/grammar/ into crate/grammar/.
    // This includes scanner.c, header files (tag.h, unicode.h), and subdirectories like
    // include/, common/, etc. We skip:
    // - src/ subdirectory (contains generated files, copied during grammar generation step)
    // - Non-source files (grammar.js, package.json, etc.)
    if def_grammar_dir.exists() {
        plan_copy_grammar_sources(&mut plan, &def_grammar_dir, &crate_grammar_dir, mode)?;
    }

    // Also check for common/ at the language level (def/common/)
    let def_lang_common = def_path.join("common");
    if def_lang_common.exists() {
        let crate_common_dir = crate_grammar_dir.join("common");
        plan_copy_grammar_sources(&mut plan, &def_lang_common, &crate_common_dir, mode)?;
    }

    // Copy query files (highlights.scm, injections.scm, locals.scm) into crate/queries/
    // so that include_str! paths work in the published package.
    let def_queries_dir = def_path.join("queries");
    let crate_queries_dir = crate_path.join("queries");

    if def_queries_dir.exists() {
        let mut queries_found = false;

        for query_name in &["highlights.scm", "injections.scm", "locals.scm"] {
            let src_query = def_queries_dir.join(query_name);
            if src_query.exists() {
                if !queries_found {
                    // Create queries/ directory on first query found
                    if !crate_queries_dir.exists() {
                        plan.add(Operation::CreateDir {
                            path: crate_queries_dir.clone(),
                            description: "Create crate queries directory".to_string(),
                        });
                    }
                    queries_found = true;
                }

                let query_content = fs::read_to_string(&src_query)?;
                let dest_query = crate_queries_dir.join(query_name);
                let desc = format!("crate queries/{}", query_name);

                plan_file_update(&mut plan, &dest_query, query_content, &desc, mode)?;
            }
        }
    }

    // Copy arborium.kdl and samples for tests
    let def_kdl = def_path.join("arborium.kdl");
    if def_kdl.exists() {
        let kdl_content = fs::read_to_string(&def_kdl)?;
        let crate_kdl = crate_path.join("arborium.kdl");
        plan_file_update(
            &mut plan,
            &crate_kdl,
            kdl_content,
            "arborium.kdl for tests",
            mode,
        )?;

        // Copy samples directory if it exists
        let def_samples = def_path.join("samples");
        if def_samples.exists() {
            let crate_samples = crate_path.join("samples");
            plan_copy_dir_recursive(&mut plan, &def_samples, &crate_samples, mode)?;
        }

        // Copy individual sample files (sample.* at def root)
        for entry in fs::read_dir(def_path)? {
            let entry = entry?;
            let path = Utf8PathBuf::try_from(entry.path())?;
            if let Some(name) = path.file_name()
                && name.starts_with("sample.")
                && path.is_file()
            {
                let content = fs::read_to_string(&path)?;
                let dest = crate_path.join(name);
                plan_file_update(
                    &mut plan,
                    &dest,
                    content,
                    &format!("{} for tests", name),
                    mode,
                )?;
            }
        }
    }

    // Generate .arborium-hash file for publish change detection
    // This hash includes all crate files that would be published
    // It's used during publishing to skip re-publishing if nothing changed
    // We add this as an operation that runs AFTER all file copies complete
    plan.add(Operation::ComputeGrammarHash {
        crate_path: crate_path.clone(),
        description: ".arborium-hash (for publish change detection)".to_string(),
    });

    Ok(plan)
}

/// Generate plugin crate files (npm/Cargo.toml, npm/src/lib.rs, npm/package.json)
/// Only generates for grammars that have generate-component enabled (default: true).
fn plan_plugin_crate_files(
    crate_state: &CrateState,
    config: &crate::types::CrateConfig,
    _repo_root: &Utf8Path,
    workspace_version: &str,
) -> Result<Plan, Report> {
    let mut plan = Plan::for_crate(format!("{}-plugin", crate_state.name));

    // Check if any grammar has generate-component enabled
    let grammar = config.grammars.first();
    let should_generate = grammar.map(|g| g.generate_component()).unwrap_or(true);

    if !should_generate {
        return Ok(plan);
    }

    let grammar = match grammar {
        Some(g) => g,
        None => return Ok(plan),
    };

    let grammar_id = grammar.id();
    let crate_name = &crate_state.name;

    // Plugin crate lives in npm/ sibling to crate/
    // Structure: langs/group-*/lang/npm/
    let lang_dir = crate_state
        .crate_path
        .parent()
        .expect("crate_path should have parent");
    let npm_path = lang_dir.join("npm");

    // Ensure npm directory exists
    if !npm_path.exists() {
        plan.add(Operation::CreateDir {
            path: npm_path.clone(),
            description: "Create npm directory".to_string(),
        });
    }

    // Generate npm/Cargo.toml
    let cargo_toml_path = npm_path.join("Cargo.toml");
    let new_cargo_toml = generate_plugin_cargo_toml(grammar_id, crate_name);

    if cargo_toml_path.exists() {
        let old_content = fs::read_to_string(&cargo_toml_path)?;
        if old_content != new_cargo_toml {
            plan.add(Operation::UpdateFile {
                path: cargo_toml_path,
                old_content: Some(old_content),
                new_content: new_cargo_toml,
                description: "Update plugin Cargo.toml".to_string(),
            });
        }
    } else {
        plan.add(Operation::CreateFile {
            path: cargo_toml_path,
            content: new_cargo_toml,
            description: "Create plugin Cargo.toml".to_string(),
        });
    }

    // Generate npm/src/lib.rs
    let lib_rs_path = npm_path.join("src/lib.rs");
    let new_lib_rs = generate_plugin_lib_rs(grammar_id, crate_name);

    if lib_rs_path.exists() {
        let old_content = fs::read_to_string(&lib_rs_path)?;
        if old_content != new_lib_rs {
            plan.add(Operation::UpdateFile {
                path: lib_rs_path,
                old_content: Some(old_content),
                new_content: new_lib_rs,
                description: "Update plugin src/lib.rs".to_string(),
            });
        }
    } else {
        // Ensure src/ directory exists
        let src_dir = npm_path.join("src");
        if !src_dir.exists() {
            plan.add(Operation::CreateDir {
                path: src_dir,
                description: "Create plugin src directory".to_string(),
            });
        }
        plan.add(Operation::CreateFile {
            path: lib_rs_path,
            content: new_lib_rs,
            description: "Create plugin src/lib.rs".to_string(),
        });
    }

    // Extract grammar metadata for package.json and README
    let grammar_name = &*grammar.name;
    let grammar_description = grammar
        .description
        .as_ref()
        .map(|d| d.value.as_str())
        .unwrap_or("");
    let language_link = grammar
        .link
        .as_ref()
        .map(|l| l.value.as_str())
        .unwrap_or("");
    let inventor = grammar
        .inventor
        .as_ref()
        .map(|i| i.value.as_str())
        .unwrap_or("");
    let year = grammar.year.as_ref().map(|y| y.value).unwrap_or(0);

    // Generate npm/package.json
    let package_json_path = npm_path.join("package.json");
    let new_package_json =
        generate_plugin_package_json(grammar_id, grammar_name, workspace_version);

    if package_json_path.exists() {
        let old_content = fs::read_to_string(&package_json_path)?;
        if old_content != new_package_json {
            plan.add(Operation::UpdateFile {
                path: package_json_path,
                old_content: Some(old_content),
                new_content: new_package_json,
                description: "Update plugin package.json".to_string(),
            });
        }
    } else {
        plan.add(Operation::CreateFile {
            path: package_json_path,
            content: new_package_json,
            description: "Create plugin package.json".to_string(),
        });
    }

    // Generate npm/README.md
    let readme_path = npm_path.join("README.md");
    let new_readme = generate_plugin_readme(
        grammar_id,
        grammar_name,
        grammar_description,
        language_link,
        inventor,
        year,
    );

    if readme_path.exists() {
        let old_content = fs::read_to_string(&readme_path)?;
        if old_content != new_readme {
            plan.add(Operation::UpdateFile {
                path: readme_path,
                old_content: Some(old_content),
                new_content: new_readme,
                description: "Update plugin README.md".to_string(),
            });
        }
    } else {
        plan.add(Operation::CreateFile {
            path: readme_path,
            content: new_readme,
            description: "Create plugin README.md".to_string(),
        });
    }

    // Note: The plugin crate depends on the grammar crate (e.g., arborium-yaml) which
    // already compiles parser.c/scanner.c via its own build.rs. We don't need to
    // duplicate that here - the plugin just links to the grammar crate's static lib.

    // Remove any stray build.rs file if it exists (leftover from old WIT setup)
    let build_rs_path = npm_path.join("build.rs");
    if build_rs_path.exists() {
        plan.add(Operation::DeleteFile {
            path: build_rs_path,
            description: "Remove unnecessary build.rs (plugins don't build C code)".to_string(),
        });
    }

    Ok(plan)
}

/// Generate the umbrella crate (crates/arborium/Cargo.toml)
/// This aggregates all grammar crates as optional dependencies with features.
fn plan_umbrella_crate(prepared: &PreparedStructures) -> Result<Plan, Report> {
    let mut plan = Plan::for_crate("arborium");
    let umbrella_path = prepared.repo_root.join("crates/arborium");
    let cargo_toml_path = umbrella_path.join("Cargo.toml");

    // Collect all grammar crates sorted by name
    let mut grammar_crates: Vec<_> = prepared
        .prepared_temps
        .iter()
        .map(|pt| {
            let name = pt.crate_state.name.clone();
            let crate_path = pt.crate_state.crate_path.clone();
            let grammar_id = name.strip_prefix("arborium-").unwrap_or(&name).to_string();
            (name, grammar_id, crate_path)
        })
        .collect();
    grammar_crates.sort_by(|a, b| a.0.cmp(&b.0));

    let version = &prepared.workspace_version;

    // Build Cargo.toml content
    let mut content = String::new();

    // Package section
    content.push_str(&format!(
        r#"[package]
name = "arborium"
version = "{version}"
edition = "2024"
license = "MIT OR Apache-2.0"
repository = "https://github.com/bearcove/arborium"
description = "Tree-sitter syntax highlighting with HTML rendering and WASM support"
keywords = ["tree-sitter", "syntax-highlighting", "wasm"]
categories = ["parsing", "text-processing", "wasm"]
links = "arborium"

[features]
default = []

# All languages
all-languages = [
"#
    ));

    // Add all lang-* features to all-languages
    for (_name, grammar_id, _) in &grammar_crates {
        // Skip internal grammars
        if grammar_id.ends_with("_inline") {
            continue;
        }
        content.push_str(&format!("    \"lang-{}\",\n", grammar_id));
    }
    content.push_str("]\n\n");

    // Individual language features
    content.push_str("# Individual language features\n");
    for (name, grammar_id, _) in &grammar_crates {
        content.push_str(&format!("lang-{} = [\"dep:{}\"]\n", grammar_id, name));
    }

    // Dependencies section (use full version for all dependencies)
    content.push_str(&format!(
        r#"
[dependencies]
arborium-tree-sitter = {{ version = "{version}", path = "../arborium-tree-sitter" }}
arborium-theme = {{ version = "{version}", path = "../arborium-theme" }}
arborium-highlight = {{ version = "{version}", path = "../arborium-highlight", features = ["tree-sitter"] }}

# Optional grammar dependencies
"#
    ));

    for (name, _, crate_path) in &grammar_crates {
        // Calculate relative path from crates/arborium to the grammar crate
        let rel_path = crate_path
            .strip_prefix(&prepared.repo_root)
            .unwrap_or(crate_path);
        content.push_str(&format!(
            "{} = {{ version = \"{}\", path = \"../../{}\", optional = true }}\n",
            name, version, rel_path
        ));
    }

    // Dev dependencies and WASM section
    content.push_str(
        r#"
[dev-dependencies]
indoc = "2"

# WASM allocator (automatically enabled on wasm targets)
[target.'cfg(target_family = "wasm")'.dependencies]
dlmalloc = "0.2"
"#,
    );

    // Write or update the file
    if cargo_toml_path.exists() {
        let old_content = fs::read_to_string(&cargo_toml_path)?;
        if old_content != content {
            plan.add(Operation::UpdateFile {
                path: cargo_toml_path,
                old_content: Some(old_content),
                new_content: content,
                description: "Update umbrella Cargo.toml".to_string(),
            });
        }
    } else {
        // Ensure directory exists
        if !umbrella_path.exists() {
            plan.add(Operation::CreateDir {
                path: umbrella_path,
                description: "Create umbrella crate directory".to_string(),
            });
        }
        plan.add(Operation::CreateFile {
            path: cargo_toml_path,
            content,
            description: "Create umbrella Cargo.toml".to_string(),
        });
    }

    Ok(plan)
}

/// Update shared crates (arborium-theme, arborium-highlight, etc.) to use the workspace version.
/// These crates are not generated from arborium.kdl but need their versions kept in sync.
fn plan_shared_crates(prepared: &PreparedStructures, mode: PlanMode) -> Result<Plan, Report> {
    let mut plan = Plan::for_crate("shared-crates");
    let version = &prepared.workspace_version;
    let repo_root = &prepared.repo_root;

    // Update all shared crates - dependencies are auto-detected
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
        "miette-arborium",
    ];

    for crate_name in shared_crates {
        update_cargo_toml_version(
            &mut plan,
            &repo_root.join(format!("crates/{}/Cargo.toml", crate_name)),
            version,
            mode,
        )?;
    }

    Ok(plan)
}

/// Update a Cargo.toml file's version and all arborium dependency versions.
/// The package version is set to the full version (e.g., "1.1.11").
/// Dependency versions are set to just the major version (e.g., "1") for SemVer compatibility.
fn update_cargo_toml_version(
    plan: &mut Plan,
    cargo_toml_path: &Utf8Path,
    new_version: &str,
    mode: PlanMode,
) -> Result<(), Report> {
    use toml_edit::{DocumentMut, Item, Value};

    if !cargo_toml_path.exists() {
        return Ok(());
    }

    let old_content = fs::read_to_string(cargo_toml_path)?;
    let mut doc: DocumentMut = old_content.parse().map_err(|e| {
        std::io::Error::other(format!("Failed to parse {}: {}", cargo_toml_path, e))
    })?;

    // Update package version (full version)
    if let Some(package) = doc.get_mut("package")
        && let Some(version) = package.get_mut("version")
    {
        *version = Item::Value(Value::from(new_version));
    }

    // Update arborium dependencies in all dependency sections (full version)
    let dep_sections = ["dependencies", "dev-dependencies", "build-dependencies"];
    for section_name in dep_sections {
        if let Some(deps) = doc.get_mut(section_name)
            && let Some(table) = deps.as_table_mut()
        {
            for (name, value) in table.iter_mut() {
                // Match "arborium" or "arborium-*"
                let dep_name = name.get();
                if (dep_name == "arborium" || dep_name.starts_with("arborium-"))
                    && let Some(dep_table) = value.as_table_like_mut()
                    && dep_table.contains_key("version")
                {
                    dep_table.insert("version", Item::Value(Value::from(new_version)));
                }
            }
        }
    }

    let new_content = doc.to_string();

    if old_content != new_content {
        let old_for_diff = if mode.is_dry_run() {
            Some(old_content)
        } else {
            None
        };

        plan.add(Operation::UpdateFile {
            path: cargo_toml_path.to_owned(),
            old_content: old_for_diff,
            new_content,
            description: format!(
                "Update {} to version {}",
                cargo_toml_path.file_name().unwrap_or("Cargo.toml"),
                new_version
            ),
        });
    }

    Ok(())
}

/// Generate the docs.rs demo crate (crates/arborium-docsrs-demo/).
/// This crate showcases arborium syntax highlighting on docs.rs.
fn plan_docsrs_demo_crate(prepared: &PreparedStructures, mode: PlanMode) -> Result<Plan, Report> {
    let mut plan = Plan::for_crate("arborium-docsrs-demo");
    let version = &prepared.workspace_version;
    let demo_path = prepared.repo_root.join("crates/arborium-docsrs-demo");

    // Ensure directory exists
    if !demo_path.exists() {
        plan.add(Operation::CreateDir {
            path: demo_path.clone(),
            description: "Create docs.rs demo crate directory".to_string(),
        });
    }

    // Generate Cargo.toml
    let cargo_toml_path = demo_path.join("Cargo.toml");
    let new_cargo_toml = DocsrsDemoCargoTomlTemplate { version }
        .render_once()
        .expect("DocsrsDemoCargoTomlTemplate render failed");
    plan_file_update(
        &mut plan,
        &cargo_toml_path,
        new_cargo_toml,
        "Cargo.toml",
        mode,
    )?;

    // Generate arborium-header.html
    let header_path = demo_path.join("arborium-header.html");
    let new_header = DocsrsDemoHeaderTemplate {}
        .render_once()
        .expect("DocsrsDemoHeaderTemplate render failed");
    plan_file_update(
        &mut plan,
        &header_path,
        new_header,
        "arborium-header.html",
        mode,
    )?;

    // Generate src/lib.rs
    let src_dir = demo_path.join("src");
    if !src_dir.exists() {
        plan.add(Operation::CreateDir {
            path: src_dir.clone(),
            description: "Create src directory".to_string(),
        });
    }
    let lib_rs_path = src_dir.join("lib.rs");
    let new_lib_rs = DocsrsDemoLibRsTemplate {}
        .render_once()
        .expect("DocsrsDemoLibRsTemplate render failed");
    plan_file_update(&mut plan, &lib_rs_path, new_lib_rs, "src/lib.rs", mode)?;

    // Generate README.md
    let readme_path = demo_path.join("README.md");
    let new_readme = DocsrsDemoReadmeTemplate { version }
        .render_once()
        .expect("DocsrsDemoReadmeTemplate render failed");
    plan_file_update(&mut plan, &readme_path, new_readme, "README.md", mode)?;

    Ok(plan)
}
