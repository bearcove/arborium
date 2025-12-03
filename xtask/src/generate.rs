//! Generate command - regenerates crate files from arborium.kdl.
//!
//! This command reads arborium.kdl files and generates:
//! - Cargo.toml
//! - build.rs
//! - src/lib.rs
//! - grammar-src/ (by running tree-sitter generate)

use crate::plan::{Operation, Plan, PlanSet};
use crate::types::{CrateRegistry, CrateState};
use camino::{Utf8Path, Utf8PathBuf};
use std::process::{Command, Stdio};

/// Generate crate files for all or a specific grammar.
pub fn plan_generate(crates_dir: &Utf8Path, name: Option<&str>) -> Result<PlanSet, String> {
    let registry = CrateRegistry::load(crates_dir).map_err(|e| e.to_string())?;
    let mut plans = PlanSet::new();

    for (_name, crate_state) in &registry.crates {
        // Skip if a specific name was requested and this isn't it
        if let Some(filter) = name {
            // Match either full name (arborium-rust) or suffix (rust)
            let matches = crate_state.name == filter
                || crate_state
                    .name
                    .strip_prefix("arborium-")
                    .map_or(false, |suffix| suffix == filter);
            if !matches {
                continue;
            }
        }

        // Skip crates without arborium.kdl
        let Some(ref config) = crate_state.config else {
            continue;
        };

        let plan = plan_crate_generation(crate_state, config).map_err(|e| e.to_string())?;
        plans.add(plan);
    }

    Ok(plans)
}

fn plan_crate_generation(
    crate_state: &CrateState,
    config: &crate::types::CrateConfig,
) -> Result<Plan, Box<dyn std::error::Error>> {
    let mut plan = Plan::for_crate(&crate_state.name);
    let crate_path = &crate_state.path;

    // Generate Cargo.toml
    let cargo_toml_path = crate_path.join("Cargo.toml");
    let new_cargo_toml = generate_cargo_toml(&crate_state.name, config);

    if cargo_toml_path.exists() {
        let old_content = std::fs::read_to_string(&cargo_toml_path)?;
        if old_content != new_cargo_toml {
            plan.add(Operation::UpdateFile {
                path: cargo_toml_path,
                old_content,
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
        let old_content = std::fs::read_to_string(&build_rs_path)?;
        if old_content != new_build_rs {
            plan.add(Operation::UpdateFile {
                path: build_rs_path,
                old_content,
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
        let old_content = std::fs::read_to_string(&lib_rs_path)?;
        if old_content != new_lib_rs {
            plan.add(Operation::UpdateFile {
                path: lib_rs_path,
                old_content,
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

    // Generate grammar-src/ from vendored grammar sources
    // Only regenerate if grammar-src/parser.c doesn't exist yet
    let grammar_dir = crate_path.join("grammar");
    let grammar_src_dir = crate_path.join("grammar-src");
    let parser_c = grammar_src_dir.join("parser.c");

    if grammar_dir.exists() && grammar_dir.join("grammar.js").exists() && !parser_c.exists() {
        plan_grammar_src_generation(&mut plan, crate_path, config)?;
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
) -> Result<(), Box<dyn std::error::Error>> {
    let deps = get_grammar_dependencies(crate_name);
    if deps.is_empty() {
        return Ok(());
    }

    let node_modules = temp_path.join("node_modules");
    std::fs::create_dir_all(&node_modules)?;

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

/// Plan the generation of grammar-src/ by running tree-sitter generate in a temp directory.
fn plan_grammar_src_generation(
    plan: &mut Plan,
    crate_path: &Utf8Path,
    config: &crate::types::CrateConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let grammar_dir = crate_path.join("grammar");
    let grammar_src_dir = crate_path.join("grammar-src");
    let crate_name = crate_path.file_name().unwrap_or("unknown");

    // Get the crates directory (parent of crate_path)
    let crates_dir = crate_path.parent().ok_or("Could not get crates directory")?;

    // Create a temp directory
    let temp_dir = tempfile::tempdir()?;
    let temp_path = Utf8PathBuf::from_path_buf(temp_dir.path().to_path_buf())
        .map_err(|_| "Non-UTF8 temp path")?;

    // Copy vendored grammar files to temp dir
    copy_dir_contents(&grammar_dir, &temp_path)?;

    // Set up cross-grammar dependencies if needed
    setup_grammar_dependencies(&temp_path, crates_dir, crate_name)?;

    // Run tree-sitter generate in the temp directory
    let output = Command::new("tree-sitter")
        .args(["generate"])
        .current_dir(&temp_path)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Show more context for debugging
        let error_lines: Vec<&str> = stderr.lines().take(20).collect();
        return Err(format!(
            "tree-sitter generate failed for {}:\n{}",
            crate_name,
            error_lines.join("\n")
        )
        .into());
    }

    // The generated files are in temp_path/src/
    let generated_src = temp_path.join("src");

    // Ensure grammar-src/ directory exists in plan
    if !grammar_src_dir.exists() {
        plan.add(Operation::CreateDir {
            path: grammar_src_dir.clone(),
            description: "Create grammar-src directory".to_string(),
        });
    }

    // Check for scanner.c - copy from vendored grammar/ if it exists there
    // (scanner.c is handwritten, not generated)
    let has_scanner = config
        .grammars
        .first()
        .map(|g| g.has_scanner())
        .unwrap_or(false);

    if has_scanner {
        let vendored_scanner = grammar_dir.join("scanner.c");
        let dest_scanner = grammar_src_dir.join("scanner.c");
        if vendored_scanner.exists() {
            let new_content = std::fs::read_to_string(&vendored_scanner)?;
            plan_file_update(plan, &dest_scanner, new_content, "scanner.c")?;
        }
    }

    // Compare generated files with existing grammar-src/
    // Files to copy: parser.c, grammar.json, node-types.json, tree_sitter/
    for file_name in ["parser.c", "grammar.json", "node-types.json"] {
        let generated_file = generated_src.join(file_name);
        let dest_file = grammar_src_dir.join(file_name);

        if generated_file.exists() {
            let new_content = std::fs::read_to_string(&generated_file)?;
            plan_file_update(plan, &dest_file, new_content, file_name)?;
        }
    }

    // Copy tree_sitter/ directory
    let generated_tree_sitter = generated_src.join("tree_sitter");
    let dest_tree_sitter = grammar_src_dir.join("tree_sitter");
    if generated_tree_sitter.exists() {
        // Ensure tree_sitter/ directory exists
        if !dest_tree_sitter.exists() {
            plan.add(Operation::CreateDir {
                path: dest_tree_sitter.clone(),
                description: "Create tree_sitter directory".to_string(),
            });
        }

        // Copy each file in tree_sitter/
        for entry in std::fs::read_dir(&generated_tree_sitter)? {
            let entry = entry?;
            let file_name = entry.file_name().to_string_lossy().to_string();
            let generated_file =
                Utf8PathBuf::from_path_buf(entry.path()).map_err(|_| "Non-UTF8 path")?;
            let dest_file = dest_tree_sitter.join(&file_name);

            if generated_file.is_file() {
                let new_content = std::fs::read_to_string(&generated_file)?;
                plan_file_update(plan, &dest_file, new_content, &format!("tree_sitter/{}", file_name))?;
            }
        }
    }

    Ok(())
}

/// Helper to plan a file update (create or update based on whether content changed).
fn plan_file_update(
    plan: &mut Plan,
    dest_path: &Utf8Path,
    new_content: String,
    description: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if dest_path.exists() {
        let old_content = std::fs::read_to_string(dest_path)?;
        if old_content != new_content {
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
fn copy_dir_contents(src: &Utf8Path, dest: &Utf8Path) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(dest)?;

    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = Utf8PathBuf::from_path_buf(entry.path()).map_err(|_| "Non-UTF8 path")?;
        let dest_path = dest.join(entry.file_name().to_string_lossy().as_ref());

        if src_path.is_dir() {
            copy_dir_contents(&src_path, &dest_path)?;
        } else {
            std::fs::copy(&src_path, &dest_path)?;
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
        r#"    println!("cargo:rerun-if-changed={}/scanner.c", src_dir);
"#
    } else {
        ""
    };

    let scanner_compile = if has_scanner {
        r#"
    build.file(format!("{}/scanner.c", src_dir));"#
    } else {
        ""
    };

    format!(
        r#"fn main() {{
    let src_dir = "grammar-src";

    println!("cargo:rerun-if-changed={{}}/parser.c", src_dir);
{scanner_section}
    let mut build = cc::Build::new();

    build
        .include(src_dir)
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
fn generate_lib_rs(crate_name: &str, crate_path: &Utf8Path, config: &crate::types::CrateConfig) -> String {
    let grammar = config.grammars.first();

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
}
