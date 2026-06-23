use std::{env, fs, path::PathBuf};

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let target = env::var("TARGET").unwrap();

    #[cfg(feature = "bindgen")]
    generate_bindings(&out_dir);

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

    if target.starts_with("wasm32-unknown") {
        // macOS's BSD `ar` can't archive wasm objects — it warns "not a mach-o
        // file" and silently drops the member, leaving an empty libtree-sitter.a
        // and undefined `ts_*` symbols at link time. (This only surfaced once
        // newer rustc stopped dead-stripping those references before the link.)
        // Linux's GNU `ar` handles wasm fine, so cc's default archiver is only
        // wrong on hosts with a BSD `ar`. Point cc at an `llvm-ar`, unless the
        // user already chose an archiver via `AR` / `AR_<target>`.
        let ar_var = format!("AR_{}", target.replace('-', "_"));
        let ar_overridden = env::var(&ar_var).is_ok() || env::var("AR").is_ok();
        if !ar_overridden {
            if let Some(llvm_ar) = find_llvm_ar() {
                config.archiver(llvm_ar);
            }
        }

        let mut arborium_has_sysroot = false;

        // Arborium patch: prefer arborium-sysroot and disable upstream wasm stdlib
        // sources to avoid duplicate symbols (malloc/free/...).
        if let Ok(sysroot) = env::var("DEP_ARBORIUM_SYSROOT_PATH") {
            let wasm_sysroot = PathBuf::from(&sysroot);
            config.include(&wasm_sysroot);
            println!("cargo:rerun-if-changed={}", wasm_sysroot.display());
            arborium_has_sysroot = true;
        }

        if !arborium_has_sysroot {
            configure_wasm_build(&mut config);
        }
    }

    for entry in fs::read_dir(&src_path).unwrap() {
        let entry = entry.unwrap();
        let path = src_path.join(entry.file_name());
        println!("cargo:rerun-if-changed={}", path.to_str().unwrap());
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
        .define("_BSD_SOURCE", None)
        .define("_DARWIN_C_SOURCE", None)
        .warnings(false)
        .file(src_path.join("lib.c"))
        .compile("tree-sitter");

    println!("cargo:include={}", include_path.display());
}

/// Locate an `llvm-ar` that understands wasm object files. Prefer the one
/// bundled with the active Rust toolchain (`<sysroot>/lib/rustlib/<host>/bin`),
/// then fall back to `llvm-ar` on `PATH`. Returns `None` if neither is usable,
/// leaving cc's default archiver in place.
fn find_llvm_ar() -> Option<PathBuf> {
    use std::process::Command;

    let exe = if cfg!(windows) { "llvm-ar.exe" } else { "llvm-ar" };

    // Toolchain-bundled llvm-ar, via `rustc --print sysroot` + the host triple.
    let rustc = env::var("RUSTC").unwrap_or_else(|_| "rustc".into());
    if let Ok(out) = Command::new(&rustc).args(["--print", "sysroot"]).output() {
        if out.status.success() {
            let sysroot = PathBuf::from(String::from_utf8_lossy(&out.stdout).trim().to_string());
            if let Ok(host) = env::var("HOST") {
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

    // Otherwise, a `PATH` llvm-ar, but only if it actually runs.
    let on_path = Command::new(exe)
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    on_path.then(|| PathBuf::from(exe))
}

fn configure_wasm_build(config: &mut cc::Build) {
    let Ok(wasm_headers) = env::var("DEP_TREE_SITTER_LANGUAGE_WASM_HEADERS") else {
        panic!("Environment variable DEP_TREE_SITTER_LANGUAGE_WASM_HEADERS must be set by the language crate");
    };
    let Ok(wasm_src) = env::var("DEP_TREE_SITTER_LANGUAGE_WASM_SRC").map(PathBuf::from) else {
        panic!("Environment variable DEP_TREE_SITTER_LANGUAGE_WASM_SRC must be set by the language crate");
    };

    config.include(&wasm_headers);
    config.files([
        wasm_src.join("stdio.c"),
        wasm_src.join("stdlib.c"),
        wasm_src.join("string.c"),
    ]);
}

#[cfg(feature = "bindgen")]
fn generate_bindings(out_dir: &std::path::Path) {
    use std::str::FromStr;

    use bindgen::RustTarget;

    const HEADER_PATH: &str = "include/tree_sitter/api.h";

    println!("cargo:rerun-if-changed={HEADER_PATH}");

    let no_copy = [
        "TSInput",
        "TSLanguage",
        "TSLogger",
        "TSLookaheadIterator",
        "TSParser",
        "TSTree",
        "TSQuery",
        "TSQueryCursor",
        "TSQueryCapture",
        "TSQueryMatch",
        "TSQueryPredicateStep",
    ];

    let rust_version = env!("CARGO_PKG_RUST_VERSION");

    let bindings = bindgen::Builder::default()
        .header(HEADER_PATH)
        .layout_tests(false)
        .allowlist_type("^TS.*")
        .allowlist_function("^ts_.*")
        .allowlist_var("^TREE_SITTER.*")
        .no_copy(no_copy.join("|"))
        .prepend_enum_name(false)
        .use_core()
        .clang_arg("-D TREE_SITTER_FEATURE_WASM")
        .rust_target(RustTarget::from_str(rust_version).unwrap())
        .generate()
        .expect("Failed to generate bindings");

    let bindings_rs = out_dir.join("bindings.rs");
    bindings.write_to_file(&bindings_rs).unwrap_or_else(|_| {
        panic!(
            "Failed to write bindings into path: {}",
            bindings_rs.display()
        )
    });
}
