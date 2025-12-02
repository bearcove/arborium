//! Generate command - regenerates crate files from arborium.kdl.
//!
//! This command reads arborium.kdl files and generates:
//! - Cargo.toml
//! - build.rs
//! - src/lib.rs

use crate::plan::{Operation, Plan, PlanSet};
use crate::types::{CrateConfig, CrateRegistry, CrateState};
use camino::Utf8Path;

/// Generate crate files for all or a specific grammar.
pub fn plan_generate(crates_dir: &Utf8Path, name: Option<&str>) -> Result<PlanSet, String> {
    let registry = CrateRegistry::load(crates_dir).map_err(|e| e.to_string())?;
    let mut plans = PlanSet::new();

    for (_name, crate_state) in &registry.crates {
        // Skip if a specific name was requested and this isn't it
        if let Some(filter) = name {
            if crate_state.name != filter {
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
    let new_lib_rs = generate_lib_rs(&crate_state.name, config);

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

    Ok(plan)
}

/// Generate Cargo.toml content for a grammar crate.
fn generate_cargo_toml(crate_name: &str, config: &crate::types::CrateConfig) -> String {
    let grammar_id = config
        .grammars
        .first()
        .map(|g| g.id.as_ref())
        .unwrap_or(crate_name.strip_prefix("arborium-").unwrap_or(crate_name));

    let description = config
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
    if target.contains("wasm") {{
        if let Ok(sysroot) = std::env::var("DEP_ARBORIUM_SYSROOT_PATH") {{
            build.include(&sysroot);
        }}
    }}

    build.file(format!("{{}}/parser.c", src_dir));{scanner_compile}

    build.compile("tree_sitter_{c_symbol}");
}}
"#
    )
}

/// Generate src/lib.rs content for a grammar crate.
fn generate_lib_rs(crate_name: &str, config: &crate::types::CrateConfig) -> String {
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
    let crate_path = format!("crates/{}", crate_name);
    let highlights_exists = std::path::Path::new(&crate_path)
        .join("queries/highlights.scm")
        .exists();
    let injections_exists = std::path::Path::new(&crate_path)
        .join("queries/injections.scm")
        .exists();

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
"#
    )
}
