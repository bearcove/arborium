use anyhow::Result;
use std::path::PathBuf;
use walrus::Module;

fn main() -> Result<()> {
    let wasm_path = PathBuf::from("pkg/arborium_wasm_browser_example_bg.wasm");

    println!("🔍 Inspecting WASM file: {}", wasm_path.display());
    println!();

    let wasm_bytes = std::fs::read(&wasm_path)?;
    let module = Module::from_buffer(&wasm_bytes)?;

    println!("📦 Module Information:");
    println!("   Functions: {}", module.funcs.iter().count());
    println!("   Imports: {}", module.imports.iter().count());
    println!("   Exports: {}", module.exports.iter().count());
    println!();

    // Check for env imports (the problematic ones)
    let mut env_imports = Vec::new();
    let mut other_imports = Vec::new();

    for import in module.imports.iter() {
        if import.module == "env" {
            env_imports.push(import.name.clone());
        } else {
            other_imports.push((import.module.clone(), import.name.clone()));
        }
    }

    if !env_imports.is_empty() {
        println!(
            "❌ Found {} problematic 'env' module imports:",
            env_imports.len()
        );
        for name in &env_imports {
            println!("   - {}", name);
        }
        println!();
        println!("⚠️  These imports will cause browser compatibility issues!");
        println!("   The browser will look for a bare module specifier 'env',");
        println!("   which requires an import map to resolve.");
        println!();
    } else {
        println!("✅ No 'env' module imports found!");
        println!();
    }

    if !other_imports.is_empty() {
        println!("ℹ️  Other imports ({}):", other_imports.len());
        for (module, name) in &other_imports {
            println!("   - {}::{}", module, name);
        }
        println!();
    }

    // Summary
    if !env_imports.is_empty() {
        println!("🔧 This confirms issue #76: arborium 2.3.2 generates WASM");
        println!("   with unresolved 'env' imports that break in browsers.");
    }

    Ok(())
}
