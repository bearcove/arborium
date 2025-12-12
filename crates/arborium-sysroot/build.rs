fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let manifest_path = std::path::Path::new(&manifest_dir);

    // wasm-sysroot is bundled inside this crate
    let wasm_sysroot = manifest_path.join("wasm-sysroot");

    // Emit metadata that dependent crates can access via DEP_ARBORIUM_SYSROOT_PATH
    println!("cargo::metadata=PATH={}", wasm_sysroot.display());

    // For WASM targets, compile the allocator C code
    let target = std::env::var("TARGET").unwrap_or_default();
    if target.contains("wasm") {
        // Compile the C source files that provide the missing symbols
        let mut build = cc::Build::new();

        build
            .include(&wasm_sysroot.join("src"))
            .include(&wasm_sysroot)
            .opt_level_str("z") // optimize for size
            .warnings(false)
            .target(&target)
            .host(&target);

        // Compile sysroot C implementations.
        //
        // Note: wctype symbols are implemented in Rust (`src/wasm.rs`) to avoid
        // duplicate symbol errors when linking with LTO.
        build.file(wasm_sysroot.join("src/stdlib.c"));
        build.file(wasm_sysroot.join("src/stdio.c"));
        build.file(wasm_sysroot.join("src/ctype.c"));
        build.file(wasm_sysroot.join("src/string.c"));

        build.compile("arborium_sysroot");

        println!("cargo:rerun-if-changed={}", wasm_sysroot.display());
        println!("cargo:rerun-if-changed=build.rs");
    }
}
