use std::{env, fs, path::PathBuf};

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    fs::copy(
        "src/wasm/stdlib-symbols.txt",
        out_dir.join("stdlib-symbols.txt"),
    )
    .unwrap();

    let mut config = cc::Build::new();

    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_WASM");
    if env::var("CARGO_FEATURE_WASM").is_ok() {
        config
            .define("TREE_SITTER_FEATURE_WASM", "")
            .define("static_assert(...)", "")
            .include(env::var("DEP_WASMTIME_C_API_INCLUDE").unwrap());
    }

    let manifest_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let include_path = manifest_path.join("include");
    let src_path = manifest_path.join("src");
    let wasm_path = src_path.join("wasm");
    for entry in fs::read_dir(&src_path).unwrap() {
        let entry = entry.unwrap();
        let path = src_path.join(entry.file_name());
        println!("cargo:rerun-if-changed={}", path.to_str().unwrap());
    }

    // For wasm32 targets, add the wasm-sysroot stub headers.
    // Implementations are provided by the `arborium-sysroot` crate, which is linked
    // into final WASM binaries (e.g. plugins) to avoid generating `env.*` imports.
    let target = env::var("TARGET").unwrap_or_default();
    if target.contains("wasm") {
        // wasm-sysroot is provided by arborium-sysroot via links = "arborium_sysroot"
        if let Ok(sysroot) = env::var("DEP_ARBORIUM_SYSROOT_PATH") {
            let wasm_sysroot = PathBuf::from(&sysroot);
            config.include(&wasm_sysroot);
            println!("cargo:rerun-if-changed={}", wasm_sysroot.display());
        }
        // Suppress format warnings on wasm32 where uint32_t is unsigned long
        // but tree-sitter's C code uses %u format specifiers
        config.flag_if_supported("-Wno-format");
    }

    config
        .flag_if_supported("-std=c11")
        .flag_if_supported("-fvisibility=hidden")
        .flag_if_supported("-Wshadow")
        .flag_if_supported("-Wno-unused-parameter")
        .flag_if_supported("-Wno-incompatible-pointer-types")
        .include(&src_path)
        .include(&wasm_path)
        .include(&include_path)
        .define("_POSIX_C_SOURCE", "200112L")
        .define("_DEFAULT_SOURCE", None)
        .define("_DARWIN_C_SOURCE", None)
        .warnings(false)
        .file(src_path.join("lib.c"))
        .compile("tree-sitter");

    println!("cargo:include={}", include_path.display());
}
