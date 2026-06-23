//! <%= generated_disclaimer %>

fn main() {
    // All build-time grammar sources live inside the crate so that
    // crates.io package verification can rebuild the crate in isolation.
    //
    // Layout inside the published crate:
    //   grammar/
    //     scanner.c      (optional, hand-written)
    //     src/
    //       parser.c
    //       grammar.json
    //       node-types.json
    //       tree_sitter/* (generated headers)
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let src_dir = manifest_dir.join("grammar/src");
    let grammar_dir = manifest_dir.join("grammar");

    println!("cargo:rerun-if-changed={}", src_dir.join("parser.c").display());
<% if has_scanner { %>
    println!("cargo:rerun-if-changed={}", grammar_dir.join("scanner.c").display());
<% } %>

    let mut build = cc::Build::new();

    build
        .include(&src_dir)
        .include(&grammar_dir) // for common/ includes like "../common/scanner.h"
        .include(src_dir.join("tree_sitter"))
        .opt_level_str("z") // optimize aggressively for size
        .warnings(false)
        .flag_if_supported("-Wno-unused-parameter")
        .flag_if_supported("-Wno-unused-but-set-variable")
        .flag_if_supported("-Wno-trigraphs");

    let target = std::env::var("TARGET").unwrap_or_default();
    if target.contains("wasm") {
        // For WASM builds, use our custom sysroot (provided by arborium crate
        // via links = "arborium").
        if let Ok(sysroot) = std::env::var("DEP_ARBORIUM_SYSROOT_PATH") {
            build.include(&sysroot);
        }

        // macOS's BSD `ar` can't archive wasm objects — it warns "not a mach-o
        // file" and silently drops the member, leaving an empty archive and
        // undefined symbols at link time. Linux's GNU `ar` handles wasm fine, so
        // cc's default is only wrong on BSD-`ar` hosts. Point cc at an `llvm-ar`
        // unless the user already chose one via `AR` / `AR_<target>`.
        let ar_var = format!("AR_{}", target.replace('-', "_"));
        if std::env::var(&ar_var).is_err() && std::env::var("AR").is_err() {
            if let Some(llvm_ar) = find_llvm_ar() {
                build.archiver(llvm_ar);
            }
        }
    }

    build.file(src_dir.join("parser.c"));
<% if has_scanner { %>
    build.file(grammar_dir.join("scanner.c"));
<% } %>

    build.compile("tree_sitter_<%= c_symbol %>");
}

/// Locate an `llvm-ar` that understands wasm object files. Prefer the one
/// bundled with the active Rust toolchain (`<sysroot>/lib/rustlib/<host>/bin`),
/// then fall back to `llvm-ar` on `PATH`. Returns `None` if neither is usable,
/// leaving cc's default archiver in place.
fn find_llvm_ar() -> Option<std::path::PathBuf> {
    use std::process::Command;

    let exe = if cfg!(windows) { "llvm-ar.exe" } else { "llvm-ar" };

    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".into());
    if let Ok(out) = Command::new(&rustc).args(["--print", "sysroot"]).output() {
        if out.status.success() {
            let sysroot =
                std::path::PathBuf::from(String::from_utf8_lossy(&out.stdout).trim().to_string());
            if let Ok(host) = std::env::var("HOST") {
                let candidate = sysroot
                    .join("lib/rustlib")
                    .join(&host)
                    .join("bin")
                    .join(exe);
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }
    }

    let on_path = Command::new(exe)
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    on_path.then(|| std::path::PathBuf::from(exe))
}
