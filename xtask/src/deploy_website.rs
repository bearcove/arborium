//! Deploy website to GitHub Pages
//!
//! This module handles deploying the demo website to the `gh-pages` branch.
//! It generates the plugins.json with CDN URLs pointing to the specified version,
//! then pushes everything to the gh-pages branch.

use camino::{Utf8Path, Utf8PathBuf};
use chrono::Utc;
use owo_colors::OwoColorize;
use rootcause::Report;
use std::process::Command;

use crate::build::{PluginManifest, PluginManifestEntry};
use crate::serve::build_static_site;
use crate::types::CrateRegistry;

type Result<T> = std::result::Result<T, Report>;

/// Deploy the website to GitHub Pages
pub fn deploy_website(repo_root: &Utf8Path, version: &str, dry_run: bool) -> Result<()> {
    println!(
        "{} Deploying website with version {}",
        "→".blue(),
        version.cyan()
    );

    // Load the registry to get all grammar info
    let crates_dir = repo_root.join("crates");
    let registry = CrateRegistry::load(&crates_dir)
        .map_err(|e| std::io::Error::other(format!("failed to load crate registry: {}", e)))?;

    // Build the demo site first (generates index.html, app.generated.js, etc.)
    println!("  {} Building demo site...", "•".dimmed());
    build_static_site(&crates_dir, false)
        .map_err(|e| std::io::Error::other(format!("failed to build static site: {}", e)))?;

    // Create a temporary directory for the site
    let temp_dir = tempfile::tempdir()?;
    let site_dir = Utf8PathBuf::from_path_buf(temp_dir.path().to_path_buf())
        .map_err(|p| std::io::Error::other(format!("non-UTF8 temp path: {}", p.display())))?;

    println!("  {} Building site in {}", "•".dimmed(), site_dir);

    // Copy static files from demo/
    let demo_dir = repo_root.join("demo");
    copy_static_files(&demo_dir, &site_dir, version)?;

    // Generate plugins.json with CDN URLs for the specified version
    generate_plugins_json(&registry, &site_dir, version)?;

    // Generate registry.json (copy from demo or generate fresh)
    copy_registry_json(&demo_dir, &site_dir)?;

    // Write CNAME file for custom domain
    fs_err::write(site_dir.join("CNAME"), "arborium.bearcove.eu\n")?;

    if dry_run {
        println!();
        println!("{} Dry run - would deploy:", "ℹ".blue());
        println!("  Site directory: {}", site_dir);
        list_site_contents(&site_dir)?;
        println!();
        println!(
            "  {} To actually deploy, run without {}",
            "→".blue(),
            "--dry-run".yellow()
        );
        return Ok(());
    }

    // Deploy to gh-pages branch
    deploy_to_gh_pages(repo_root, &site_dir)?;

    println!();
    println!("{} Website deployed to gh-pages branch", "✓".green());
    println!(
        "  {} View at: {}",
        "→".blue(),
        "https://arborium.bearcove.eu/".cyan()
    );

    Ok(())
}

fn copy_static_files(demo_dir: &Utf8Path, site_dir: &Utf8Path, version: &str) -> Result<()> {
    println!("  {} Copying static files...", "•".dimmed());

    // Copy index.html with version replacement
    let index_src = demo_dir.join("index.html");
    let index_dst = site_dir.join("index.html");
    if index_src.exists() {
        let content = fs_err::read_to_string(&index_src)?;
        let content = content.replace("{{VERSION}}", version);
        fs_err::write(&index_dst, content)?;
    }

    // Copy iife-demo.html with version replacement
    let iife_demo_src = demo_dir.join("iife-demo.html");
    let iife_demo_dst = site_dir.join("iife-demo.html");
    if iife_demo_src.exists() {
        let content = fs_err::read_to_string(&iife_demo_src)?;
        let content = content.replace("{{VERSION}}", version);
        fs_err::write(&iife_demo_dst, content)?;
    }

    // Copy rustdoc-comparison.html
    let rustdoc_comparison_src = demo_dir.join("rustdoc-comparison.html");
    let rustdoc_comparison_dst = site_dir.join("rustdoc-comparison.html");
    if rustdoc_comparison_src.exists() {
        fs_err::copy(&rustdoc_comparison_src, &rustdoc_comparison_dst)?;
    }

    // Copy styles.css directly
    let styles_src = demo_dir.join("styles.css");
    let styles_dst = site_dir.join("styles.css");
    if styles_src.exists() {
        fs_err::copy(&styles_src, &styles_dst)?;
    }

    // Copy dodeca-logo.svg directly
    let logo_src = demo_dir.join("dodeca-logo.svg");
    let logo_dst = site_dir.join("dodeca-logo.svg");
    if logo_src.exists() {
        fs_err::copy(&logo_src, &logo_dst)?;
    }

    // Copy font files (*.woff2) and SVG files (*.svg)
    for entry in fs_err::read_dir(demo_dir)? {
        let entry = entry?;
        let path = entry.path();
        let should_copy = path
            .extension()
            .map(|e| e == "woff2" || e == "svg")
            .unwrap_or(false);
        if should_copy {
            let filename = path.file_name().unwrap();
            let dst = site_dir.join(filename.to_string_lossy().as_ref());
            fs_err::copy(&path, &dst)?;
        }
    }

    // Copy directories
    let dirs = ["pkg", "samples", "rustdoc-comparison"];
    for dir in dirs {
        let src = demo_dir.join(dir);
        let dst = site_dir.join(dir);
        if src.exists() {
            copy_dir_recursive(&src, &dst)?;
        }
    }

    Ok(())
}

fn copy_dir_recursive(src: &Utf8Path, dst: &Utf8Path) -> Result<()> {
    fs_err::create_dir_all(dst)?;

    for entry in fs_err::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = path.file_name().unwrap().to_string_lossy();

        // Skip .gitignore files - they would prevent git from adding files in the deploy dir
        if file_name == ".gitignore" {
            continue;
        }

        let dst_path = dst.join(file_name.as_ref());

        if path.is_dir() {
            let src_utf8 = Utf8PathBuf::from_path_buf(path.clone())
                .map_err(|p| std::io::Error::other(format!("non-UTF8 path: {}", p.display())))?;
            copy_dir_recursive(&src_utf8, &dst_path)?;
        } else {
            fs_err::copy(&path, &dst_path)?;
        }
    }

    Ok(())
}

fn generate_plugins_json(
    registry: &CrateRegistry,
    site_dir: &Utf8Path,
    version: &str,
) -> Result<()> {
    println!(
        "  {} Generating plugins.json for version {}...",
        "•".dimmed(),
        version
    );

    let mut entries = Vec::new();

    // Get all grammars from registry that have component generation enabled
    for (state, _config, grammar) in registry.all_grammars() {
        if !grammar.generate_component() {
            continue;
        }

        let lang_name = grammar.id();
        let package = format!("@arborium/{}", lang_name);
        let cdn_base = format!(
            "https://cdn.jsdelivr.net/npm/@arborium/{}@{}",
            lang_name, version
        );

        // Extract group name from crate_path (e.g., "langs/group-birch/rust/crate" -> "birch")
        let group_name = state
            .crate_path
            .as_str()
            .split('/')
            .find(|s| s.starts_with("group-"))
            .map(|s| s.strip_prefix("group-").unwrap_or(s))
            .unwrap_or("unknown");

        // Build path to WASM file in npm output
        let wasm_path = state
            .crate_path
            .parent()
            .expect("lang directory")
            .join("npm")
            .join("grammar_bg.wasm");

        // Calculate WASM sizes (use zeros if file doesn't exist yet)
        let (size_bytes, size_gzip, size_brotli) =
            crate::build::calculate_wasm_sizes(&wasm_path).unwrap_or((0, 0, 0));

        // Count C lines in parser
        let c_lines = crate::build::count_c_lines(&state.crate_path);

        entries.push(PluginManifestEntry {
            language: lang_name.to_string(),
            package,
            version: version.to_string(),
            cdn_js: format!("{}/grammar.js", cdn_base),
            cdn_wasm: format!("{}/grammar_bg.wasm", cdn_base),
            // Local paths for reference (used in dev mode)
            local_js: format!("/langs/group-{}/{}/npm/grammar.js", group_name, lang_name),
            local_wasm: format!(
                "/langs/group-{}/{}/npm/grammar_bg.wasm",
                group_name, lang_name
            ),
            size_bytes,
            size_gzip,
            size_brotli,
            c_lines,
        });
    }

    let manifest = PluginManifest {
        generated_at: Utc::now().to_rfc3339(),
        entries,
    };

    // Write with dev_mode: false for production
    let json = facet_json::to_string_pretty(&manifest).expect("manifest serialization failed");
    // Insert dev_mode field at the start
    let json_with_dev_mode = json.replacen("{", "{\n  \"dev_mode\": false,", 1);

    let output_path = site_dir.join("plugins.json");
    fs_err::write(&output_path, json_with_dev_mode)?;

    Ok(())
}

fn copy_registry_json(demo_dir: &Utf8Path, site_dir: &Utf8Path) -> Result<()> {
    println!("  {} Copying registry.json...", "•".dimmed());

    let src = demo_dir.join("registry.json");
    let dst = site_dir.join("registry.json");

    if src.exists() {
        fs_err::copy(&src, &dst)?;
    } else {
        return Err(
            std::io::Error::other("registry.json not found in demo/. Run `cargo xtask build` first.")
                .into(),
        );
    }

    Ok(())
}

fn list_site_contents(site_dir: &Utf8Path) -> Result<()> {
    println!("  Site contents:");
    for entry in fs_err::read_dir(site_dir)? {
        let entry = entry?;
        let path = entry.path();
        let name = path.file_name().unwrap().to_string_lossy();
        let meta = entry.metadata()?;
        if meta.is_dir() {
            println!("    {}/", name.blue());
        } else {
            println!("    {} ({})", name, format_size(meta.len()));
        }
    }
    Ok(())
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / 1024.0 / 1024.0)
    }
}

fn deploy_to_gh_pages(repo_root: &Utf8Path, site_dir: &Utf8Path) -> Result<()> {
    println!("  {} Deploying to gh-pages branch...", "•".dimmed());

    // Initialize a new git repo in the site directory
    run_git(site_dir, &["init"])?;
    run_git(site_dir, &["checkout", "-b", "gh-pages"])?;

    // Add all files
    run_git(site_dir, &["add", "-A"])?;

    // Commit
    let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();
    let commit_msg = format!("Deploy website ({})", timestamp);
    run_git(site_dir, &["commit", "-m", &commit_msg])?;

    // Get the remote URL from the main repo
    let remote_url = get_remote_url(repo_root)?;

    // Add remote and force push
    run_git(site_dir, &["remote", "add", "origin", &remote_url])?;
    run_git(site_dir, &["push", "-f", "origin", "gh-pages"])?;

    Ok(())
}

fn run_git(cwd: &Utf8Path, args: &[&str]) -> Result<()> {
    let output = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(
            std::io::Error::other(format!("git {} failed: {}", args.join(" "), stderr)).into(),
        );
    }

    Ok(())
}

fn get_remote_url(repo_root: &Utf8Path) -> Result<String> {
    let output = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(repo_root)
        .output()?;

    if !output.status.success() {
        return Err(std::io::Error::other("failed to get remote URL").into());
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
