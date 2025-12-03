fn main() {
    let src_dir = "grammar/src";

    println!("cargo:rerun-if-changed={}/parser.c", src_dir);
    println!("cargo:rerun-if-changed=grammar/scanner.c");

    let mut build = cc::Build::new();

    build
        .include(src_dir)
        .include("grammar") // for common/ includes like "../common/scanner.h"
        .include(format!("{}/tree_sitter", src_dir))
        .warnings(false)
        .flag_if_supported("-Wno-unused-parameter")
        .flag_if_supported("-Wno-unused-but-set-variable")
        .flag_if_supported("-Wno-trigraphs");

    // For WASM builds, use our custom sysroot (provided by arborium crate via links = "arborium")
    let target = std::env::var("TARGET").unwrap_or_default();
    if target.contains("wasm")
        && let Ok(sysroot) = std::env::var("DEP_ARBORIUM_SYSROOT_PATH")
    {
        build.include(&sysroot);
    }

    build.file(format!("{}/parser.c", src_dir));
    build.file("grammar/scanner.c");

    build.compile("tree_sitter_lean");
}
