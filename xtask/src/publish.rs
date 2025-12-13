//! Publishing to crates.io and npm.
//!
//! This module handles publishing arborium packages to both registries,
//! with proper handling for already-published versions and dependency ordering.
//!
//! ## Crate Groups
//!
//! Crates are organized into groups for publishing:
//! - `pre`: Shared crates that grammars depend on (tree-sitter, sysroot, test-harness)
//! - Language groups (cedar, fern, birch, etc.): Grammar crates in `langs/group-*/`
//! - `post`: Umbrella crates that depend on grammars (arborium)
//!
//! ## Dependency Ordering
//!
//! Within all grammar crates, dependencies are resolved via topological sort.
//! Crates are published leaves-first (crates with no dependencies on other
//! grammar crates are published first).

use camino::{Utf8Path, Utf8PathBuf};
use miette::{Context, IntoDiagnostic, Result};
use owo_colors::OwoColorize;
use std::collections::HashMap;
use std::process::{Command, Stdio};

use crate::tool::Tool;
use crate::types::CrateRegistry;
use crate::version_store;

/// Crates in the "pre" group - must be published before grammar crates.
/// These are shared dependencies that grammar crates rely on.
/// ORDER MATTERS - sorted by dependency order (no deps first).
const PRE_CRATES: &[&str] = &[
    // No arborium dependencies
    "crates/arborium-docsrs-demo",
    "crates/arborium-sysroot",
    "crates/arborium-theme",
    "crates/arborium-wire",
    // Depends on sysroot
    "crates/arborium-tree-sitter",
    // Depends on theme, tree-sitter
    "crates/arborium-highlight",
    // Depends on highlight, theme, tree-sitter
    "crates/arborium-test-harness",
];

/// Crates in the "post" group - must be published after grammar crates.
/// These are umbrella crates that optionally depend on grammar crates.
/// ORDER MATTERS - sorted by dependency order.
const POST_CRATES: &[&str] = &[
    // Main arborium crate first
    "crates/arborium",
    // Depends on arborium
    "crates/miette-arborium",
];

/// Name of the file that contains the grammar content hash
const HASH_FILE: &str = ".arborium-hash";

/// Check if a crate directory is a grammar crate (has grammar/ subdirectory).
fn is_grammar_crate(crate_dir: &Utf8Path) -> bool {
    crate_dir.join("grammar").exists()
}

/// Check if we should skip publishing a grammar crate by comparing content hashes.
///
/// Returns:
/// - Ok(true) if published version exists and hash matches (should skip)
/// - Ok(false) if hash differs or no published version (should publish)
/// - Err if couldn't check (e.g., download failed)
fn should_skip_grammar_publish(crate_dir: &Utf8Path, name: &str, version: &str) -> Result<bool> {
    use owo_colors::OwoColorize;

    // Compute current hash
    let current_hash = compute_grammar_hash(crate_dir)?;
    let short_hash = &current_hash[..12];

    // Try to get hash from published version
    match get_published_crate_hash(name, version) {
        Ok(Some(published_hash)) => {
            let short_published = &published_hash[..std::cmp::min(12, published_hash.len())];
            if current_hash == published_hash {
                eprintln!(
                    "    {} hash match: {}...",
                    name.dimmed(),
                    short_hash.dimmed()
                );
                Ok(true)
            } else {
                eprintln!(
                    "    {} hash changed: {} → {}",
                    name.dimmed(),
                    short_published.dimmed(),
                    short_hash.yellow()
                );
                Ok(false)
            }
        }
        Ok(None) => {
            eprintln!(
                "    {} {}",
                name.dimmed(),
                "no published hash (new or first with hash)".dimmed()
            );
            Ok(false)
        }
        Err(e) => {
            eprintln!(
                "    {} {} {}",
                name.dimmed(),
                "hash check failed:".dimmed(),
                format!("{}", e).dimmed()
            );
            Ok(false)
        }
    }
}

/// Download a published crate and extract its .arborium-hash file if present.
fn get_published_crate_hash(name: &str, version: &str) -> Result<Option<String>> {
    // Use cargo-download which handles crates.io auth properly
    // It outputs INFO messages to stderr and binary content to stdout
    let output = std::process::Command::new("cargo")
        .args(["download", &format!("{}=={}", name, version)])
        .stderr(std::process::Stdio::null())
        .output()
        .into_diagnostic()?;

    if !output.status.success() || output.stdout.is_empty() {
        return Ok(None); // Crate doesn't exist yet or download failed
    }

    // Validate gzip magic bytes (0x1f 0x8b) - cargo download returns XML on error
    if output.stdout.len() < 2 || output.stdout[0] != 0x1f || output.stdout[1] != 0x8b {
        return Ok(None); // Not a valid gzip file (probably XML error response)
    }

    // The stdout contains the gzip crate data
    let cursor = std::io::Cursor::new(&output.stdout);
    let tar = flate2::read::GzDecoder::new(cursor);
    let mut archive = tar::Archive::new(tar);

    for entry in archive.entries().into_diagnostic()? {
        let mut entry = entry.into_diagnostic()?;
        let path = entry.path().into_diagnostic()?;

        // The file will be in the archive like: {name}-{version}/.arborium-hash
        if path.file_name() == Some(std::ffi::OsStr::new(HASH_FILE)) {
            let mut contents = String::new();
            std::io::Read::read_to_string(&mut entry, &mut contents).into_diagnostic()?;
            return Ok(Some(contents.trim().to_string()));
        }
    }

    Ok(None) // No hash file found
}

/// Compute hash of grammar crate contents for change detection.
///
/// Hashes all files that would be published in the crate:
/// - grammar/ directory (parser.c, scanner.c, grammar.json, etc.)
/// - src/ directory (lib.rs and other Rust source)
/// - build.rs
/// - Cargo.toml
/// - queries/ directory (if exists)
/// - arborium.kdl (if exists)
/// - sample files (if exist)
/// - tree-sitter CLI version
///
/// Returns hex-encoded blake3 hash.
pub fn compute_grammar_hash(crate_dir: &Utf8Path) -> Result<String> {
    let mut hasher = blake3::Hasher::new();

    // Hash tree-sitter CLI version
    let ts_version = Tool::TreeSitter
        .get_version()
        .unwrap_or_else(|_| "unknown".to_string());
    hasher.update(b"tree-sitter-version:");
    hasher.update(ts_version.as_bytes());
    hasher.update(b"\n");

    // Collect all files to hash, excluding build artifacts
    let mut files = Vec::new();
    for entry in walkdir::WalkDir::new(crate_dir)
        .sort_by_file_name()
        .into_iter()
        .filter_entry(|e| {
            // Skip target/ directory and .arborium-hash itself
            let name = e.file_name().to_string_lossy();
            name != "target" && name != ".arborium-hash"
        })
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            files.push(entry.path().to_path_buf());
        }
    }

    // Hash all files in sorted order
    for file_path in files {
        let relative = file_path.strip_prefix(crate_dir).unwrap();
        let contents = std::fs::read(&file_path)
            .into_diagnostic()
            .with_context(|| format!("Failed to read {}", file_path.display()))?;

        // Hash file path and contents
        hasher.update(b"file:");
        hasher.update(relative.to_string_lossy().as_bytes());
        hasher.update(b"\n");
        hasher.update(&contents);
        hasher.update(b"\n");
    }

    let hash = hasher.finalize();
    Ok(hash.to_hex().to_string())
}

/// Publish crates to crates.io.
///
/// If `group` is None, publishes everything in order: pre, then all groups, then post.
/// If `group` is Some("pre"), publishes only the pre crates.
/// If `group` is Some("post"), publishes only the post crates.
/// If `group` is Some(name), publishes only crates from that language group.
pub fn publish_crates(
    repo_root: &Utf8Path,
    group: Option<&str>,
    dry_run: bool,
    verbose: bool,
) -> Result<()> {
    println!("{}", "Publishing to crates.io...".cyan().bold());

    if dry_run {
        println!("{}", "  (dry run)".yellow());
    }

    let langs_dir = repo_root.join("langs");

    match group {
        Some("pre") => {
            println!("  Publishing {} group", "pre".cyan());
            let crate_paths: Vec<Utf8PathBuf> =
                PRE_CRATES.iter().map(|c| repo_root.join(c)).collect();
            publish_crate_paths(&crate_paths, dry_run, verbose)?;
            print_crates_next_steps(&langs_dir, Some("pre"))?;
            Ok(())
        }
        Some("post") => {
            println!("  Publishing {} group", "post".cyan());
            let crate_paths: Vec<Utf8PathBuf> =
                POST_CRATES.iter().map(|c| repo_root.join(c)).collect();
            publish_crate_paths(&crate_paths, dry_run, verbose)?;
            print_crates_next_steps(&langs_dir, Some("post"))?;
            Ok(())
        }
        Some(group_name) => {
            println!("  Publishing {} group", group_name.cyan());
            let crates = find_group_crates(&langs_dir, group_name)?;
            if crates.is_empty() {
                println!(
                    "{} No crates found for group '{}' in {}",
                    "!".yellow(),
                    group_name,
                    langs_dir
                );
                return Ok(());
            }
            publish_crate_paths(&crates, dry_run, verbose)?;
            print_crates_next_steps(&langs_dir, Some(group_name))?;
            Ok(())
        }
        None => {
            // Publish everything in order
            println!("  Publishing: pre → grammars (topologically sorted) → post");
            println!();

            // 1. Pre crates
            println!("  {} Publishing {} group...", "●".cyan(), "pre".bold());
            let pre_paths: Vec<Utf8PathBuf> =
                PRE_CRATES.iter().map(|c| repo_root.join(c)).collect();
            publish_crate_paths(&pre_paths, dry_run, verbose)?;
            println!();

            // 2. All grammar crates in dependency order (leaves first)
            println!(
                "  {} Publishing {} (topologically sorted)...",
                "●".cyan(),
                "grammar crates".bold()
            );
            let sorted_crates = topological_sort_grammar_crates(repo_root, &langs_dir)?;
            println!(
                "    {} crates to publish in dependency order",
                sorted_crates.len()
            );
            publish_crate_paths(&sorted_crates, dry_run, verbose)?;
            println!();

            // 3. Post crates
            println!("  {} Publishing {} group...", "●".cyan(), "post".bold());
            let post_paths: Vec<Utf8PathBuf> =
                POST_CRATES.iter().map(|c| repo_root.join(c)).collect();
            publish_crate_paths(&post_paths, dry_run, verbose)?;

            println!();
            println!("{} All crates published!", "✓".green().bold());
            Ok(())
        }
    }
}

/// Print next steps hint for crates publishing.
fn print_crates_next_steps(langs_dir: &Utf8Path, current_group: Option<&str>) -> Result<()> {
    let all_groups = find_all_groups(langs_dir)?;

    println!();
    println!("{}", "Next steps:".bold());

    match current_group {
        Some("pre") => {
            // After pre, suggest language groups
            println!("  {} Continue with language groups:", "→".blue());
            for group in &all_groups {
                println!(
                    "      {} --group {}",
                    "cargo xtask publish crates".cyan(),
                    group
                );
            }
            println!(
                "  {} Then finish with: {} --group post",
                "→".blue(),
                "cargo xtask publish crates".cyan()
            );
        }
        Some("post") => {
            // After post, suggest npm
            println!(
                "  {} {} to publish npm packages",
                "→".blue(),
                "cargo xtask publish npm".cyan()
            );
        }
        Some(group_name) => {
            // After a language group, suggest next group or post
            let current_idx = all_groups.iter().position(|g| g == group_name);
            if let Some(idx) = current_idx {
                if idx + 1 < all_groups.len() {
                    println!(
                        "  {} {} --group {} (next language group)",
                        "→".blue(),
                        "cargo xtask publish crates".cyan(),
                        all_groups[idx + 1]
                    );
                } else {
                    println!(
                        "  {} {} --group post (all language groups done)",
                        "→".blue(),
                        "cargo xtask publish crates".cyan()
                    );
                }
            }
        }
        None => {
            // Published everything
            println!(
                "  {} {} to publish npm packages",
                "→".blue(),
                "cargo xtask publish npm".cyan()
            );
        }
    }

    Ok(())
}

/// Publish npm packages.
///
/// If `group` is None, publishes all npm packages.
/// If `group` is Some(name), publishes only packages from that language group.
pub fn publish_npm(
    repo_root: &Utf8Path,
    langs_dir: &Utf8Path,
    group: Option<&str>,
    dry_run: bool,
) -> Result<()> {
    println!("{}", "Publishing to npm...".cyan().bold());

    if dry_run {
        println!("{}", "  (dry run)".yellow());
    }

    let canonical_version = version_store::read_version(repo_root)?;
    ensure_release_version(&canonical_version)?;
    version_store::sync_main_npm_package_version(repo_root, &canonical_version)?;
    println!("  Using release version {}", canonical_version.cyan());

    let packages = match group {
        Some(group_name) => {
            println!("  Publishing {} group", group_name.cyan());
            find_group_npm_packages(langs_dir, group_name)?
        }
        None => find_npm_packages(langs_dir)?,
    };

    if packages.is_empty() {
        let location = group
            .map(|g| format!("group '{}'", g))
            .unwrap_or_else(|| langs_dir.to_string());
        println!("{} No npm packages found in {}", "!".yellow(), location);
        return Ok(());
    }

    println!("  Found {} packages to publish", packages.len());

    let mut published = 0;
    let mut skipped = 0;

    for package_dir in &packages {
        match publish_single_npm_package(package_dir, &canonical_version, dry_run)? {
            NpmPublishResult::Published => published += 1,
            NpmPublishResult::AlreadyExists => skipped += 1,
        }
    }

    // Also publish the main @arborium/arborium package if it exists and no group filter
    if group.is_none() {
        let main_package = repo_root.join("packages/arborium");
        if main_package.exists() && main_package.join("package.json").exists() {
            println!("  Publishing main package @arborium/arborium...");
            match publish_single_npm_package(&main_package, &canonical_version, dry_run)? {
                NpmPublishResult::Published => published += 1,
                NpmPublishResult::AlreadyExists => skipped += 1,
            }
        }
    }

    println!();
    println!(
        "{} npm publish complete: {} published, {} skipped (already exist)",
        "✓".green(),
        published,
        skipped,
    );

    Ok(())
}

fn ensure_release_version(version: &str) -> Result<()> {
    if version.ends_with("-dev") || version.ends_with("-test") {
        return Err(miette::miette!(
            "Refusing to publish development version {} - run `cargo xtask gen --version <x.y.z>` first",
            version
        ));
    }

    Ok(())
}

/// Publish everything (crates.io + npm).
pub fn publish_all(
    repo_root: &Utf8Path,
    langs_dir: &Utf8Path,
    dry_run: bool,
    verbose: bool,
) -> Result<()> {
    // Publish to crates.io first (all groups in order)
    publish_crates(repo_root, None, dry_run, verbose)?;

    println!();

    // Then publish to npm (all packages)
    publish_npm(repo_root, langs_dir, None, dry_run)?;

    println!();
    println!("{} All publishing complete!", "✓".green().bold());
    Ok(())
}

// =============================================================================
// Crate publishing helpers
// =============================================================================

/// Result of attempting to publish a single crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CratePublishResult {
    Published,
    AlreadyExists,
    Failed,
}

/// Publish crates from a list of paths.
fn publish_crate_paths(crates: &[Utf8PathBuf], dry_run: bool, verbose: bool) -> Result<()> {
    let mut published = 0;
    let mut skipped = 0;
    let mut failed = 0;

    for crate_dir in crates {
        match publish_single_crate(crate_dir, dry_run, verbose)? {
            CratePublishResult::Published => published += 1,
            CratePublishResult::AlreadyExists => skipped += 1,
            CratePublishResult::Failed => failed += 1,
        }
    }

    if failed > 0 {
        return Err(miette::miette!("{} crates failed to publish", failed));
    }

    println!(
        "    {} published, {} skipped, {} failed",
        published.to_string().green(),
        skipped.to_string().yellow(),
        failed.to_string().red()
    );

    Ok(())
}

/// Read crate name and version from Cargo.toml.
fn read_crate_info(crate_dir: &Utf8Path) -> Result<(String, String)> {
    let cargo_toml_path = crate_dir.join("Cargo.toml");
    let content = fs_err::read_to_string(&cargo_toml_path)
        .into_diagnostic()
        .wrap_err_with(|| format!("Failed to read {}", cargo_toml_path))?;

    // Simple TOML parsing - extract name and version
    let name = extract_toml_string(&content, "name")
        .ok_or_else(|| miette::miette!("No 'name' field in {}", cargo_toml_path))?;

    // Version might be "X.Y.Z" or { workspace = true }
    let version = if content.contains("version.workspace = true")
        || content.contains("version = { workspace = true }")
    {
        // Read from workspace - for now just use a placeholder that we'll check against crates.io
        "workspace".to_string()
    } else {
        extract_toml_string(&content, "version")
            .ok_or_else(|| miette::miette!("No 'version' field in {}", cargo_toml_path))?
    };

    Ok((name, version))
}

/// Extract a string value from TOML [package] section.
fn extract_toml_string(toml_str: &str, key: &str) -> Option<String> {
    let doc: toml_edit::DocumentMut = toml_str.parse().ok()?;
    doc.get("package")?
        .get(key)?
        .as_str()
        .map(|s| s.to_string())
}

/// Check if a crate version already exists on crates.io.
fn crate_version_exists(crate_name: &str, version: &str) -> Result<bool> {
    // For workspace versions, we can't easily check without parsing the root Cargo.toml
    if version == "workspace" {
        return Ok(false);
    }

    let output = Command::new("cargo")
        .args(["search", crate_name, "--limit", "1"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .into_diagnostic()
        .wrap_err("Failed to run cargo search")?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Output looks like: crate_name = "version" # description
        if stdout.contains(&format!("\"{}\"", version)) {
            return Ok(true);
        }
    }

    Ok(false)
}

/// Publish a single crate.
fn publish_single_crate(
    crate_dir: &Utf8Path,
    dry_run: bool,
    verbose: bool,
) -> Result<CratePublishResult> {
    // Check if Cargo.toml exists
    if !crate_dir.join("Cargo.toml").exists() {
        println!(
            "  {} {} - {}",
            "→".blue(),
            crate_dir,
            "no Cargo.toml, skipping".yellow()
        );
        return Ok(CratePublishResult::AlreadyExists);
    }

    // Check if publish = false
    let cargo_toml = fs_err::read_to_string(crate_dir.join("Cargo.toml"))
        .into_diagnostic()
        .wrap_err_with(|| format!("Failed to read Cargo.toml in {}", crate_dir))?;
    if cargo_toml.contains("publish = false") {
        println!(
            "  {} {} - {}",
            "→".blue(),
            crate_dir.file_name().unwrap_or("?"),
            "publish = false, skipping".dimmed()
        );
        return Ok(CratePublishResult::AlreadyExists);
    }

    let (name, version) = read_crate_info(crate_dir)?;
    let display_version = if version == "workspace" {
        "workspace".dimmed().to_string()
    } else {
        version.clone()
    };

    print!("  {} {}@{}...", "→".blue(), name, display_version);

    // For grammar crates, check if content hash matches published version
    // Skip publishing if nothing changed (grammar source files + tree-sitter version)
    if is_grammar_crate(crate_dir) {
        match should_skip_grammar_publish(crate_dir, &name, &version) {
            Ok(true) => {
                if dry_run {
                    println!(
                        " {} {}",
                        "[dry-run]".yellow(),
                        "unchanged (hash match), would skip".dimmed()
                    );
                } else {
                    println!(" {}", "unchanged (hash match), skipping".dimmed());
                    return Ok(CratePublishResult::AlreadyExists);
                }
            }
            Ok(false) => {
                // Hash different or no published version, continue to publish
            }
            Err(e) => {
                // Couldn't check hash, continue with normal version check
                eprintln!(" {} checking hash: {}", "warning".yellow(), e);
            }
        }
    }

    // Check if version already exists (skip for workspace versions - cargo publish will handle it)
    if !dry_run && version != "workspace" {
        match crate_version_exists(&name, &version) {
            Ok(true) => {
                println!(" {}", "already exists, skipping".yellow());
                return Ok(CratePublishResult::AlreadyExists);
            }
            Ok(false) => {
                // Continue to publish
            }
            Err(e) => {
                // Couldn't check, try to publish anyway
                eprintln!(" {} checking version: {}", "warning".yellow(), e);
            }
        }
    }

    // Publish (or dry-run publish)
    let mut args = vec!["publish", "--allow-dirty"];
    if dry_run {
        args.push("--dry-run");
    }

    // In verbose mode, let output stream directly to terminal
    if verbose {
        println!(); // newline after the "→ crate@version..." prefix
        let status = Command::new("cargo")
            .args(&args)
            .current_dir(crate_dir)
            .status()
            .into_diagnostic()
            .wrap_err("Failed to run cargo publish")?;

        if status.success() {
            if dry_run {
                println!("  {}", "dry-run ok".green());
            } else {
                println!("  {}", "published".green());
            }
            return Ok(CratePublishResult::Published);
        } else {
            println!("  {}", "FAILED".red());
            return Ok(CratePublishResult::Failed);
        }
    }

    let output = Command::new("cargo")
        .args(&args)
        .current_dir(crate_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .into_diagnostic()
        .wrap_err("Failed to run cargo publish")?;

    if output.status.success() {
        if dry_run {
            println!(" {}", "dry-run ok".green());
        } else {
            println!(" {}", "published".green());
        }
        return Ok(CratePublishResult::Published);
    }

    // Check if it's an "already published" error
    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains("already uploaded")
        || stderr.contains("already exists")
        || stderr.contains("is already published")
    {
        println!(" {}", "already exists, skipping".yellow());
        return Ok(CratePublishResult::AlreadyExists);
    }

    // Real error
    println!(" {}", "FAILED".red());
    eprintln!("    stderr: {}", stderr);
    Ok(CratePublishResult::Failed)
}

/// Find all language group names in langs/.
fn find_all_groups(langs_dir: &Utf8Path) -> Result<Vec<String>> {
    let mut groups = Vec::new();

    if !langs_dir.exists() {
        return Ok(groups);
    }

    for entry in langs_dir
        .read_dir_utf8()
        .into_diagnostic()
        .wrap_err_with(|| format!("Failed to read directory: {}", langs_dir))?
    {
        let entry = entry.into_diagnostic()?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let name = path.file_name().unwrap_or("");
        if let Some(group_name) = name.strip_prefix("group-") {
            groups.push(group_name.to_string());
        }
    }

    groups.sort();
    Ok(groups)
}

/// Find all crates in a specific language group.
fn find_group_crates(langs_dir: &Utf8Path, group_name: &str) -> Result<Vec<Utf8PathBuf>> {
    let mut crates = Vec::new();

    let group_dir = langs_dir.join(format!("group-{}", group_name));
    if !group_dir.exists() {
        return Ok(crates);
    }

    for lang_entry in group_dir
        .read_dir_utf8()
        .into_diagnostic()
        .wrap_err_with(|| format!("Failed to read directory: {}", group_dir))?
    {
        let lang_entry = lang_entry.into_diagnostic()?;
        let lang_path = lang_entry.path();

        if !lang_path.is_dir() {
            continue;
        }

        // Check for crate/Cargo.toml
        let crate_dir = lang_path.join("crate");
        if crate_dir.is_dir() && crate_dir.join("Cargo.toml").exists() {
            crates.push(crate_dir);
        }
    }

    crates.sort();
    Ok(crates)
}

/// Collect ALL grammar crates and sort them topologically by dependencies.
///
/// This ensures crates are published leaves-first: if crate A depends on crate B,
/// then B will appear before A in the returned list.
fn topological_sort_grammar_crates(
    repo_root: &Utf8Path,
    langs_dir: &Utf8Path,
) -> Result<Vec<Utf8PathBuf>> {
    let crates_dir = repo_root.join("crates");

    // Load the registry to get dependency information
    let registry = CrateRegistry::load(&crates_dir)
        .map_err(|e| miette::miette!("failed to load crate registry: {}", e))?;

    // Build a map from crate name -> crate path
    // and a map from crate name -> dependencies (other arborium crate names)
    let mut crate_paths: HashMap<String, Utf8PathBuf> = HashMap::new();
    let mut dependencies: HashMap<String, Vec<String>> = HashMap::new();

    // Collect all grammar crates from all groups
    let groups = find_all_groups(langs_dir)?;
    for group_name in &groups {
        let group_crates = find_group_crates(langs_dir, group_name)?;
        for crate_path in group_crates {
            // Extract crate name from Cargo.toml
            if let Ok((name, _)) = read_crate_info(&crate_path) {
                crate_paths.insert(name.clone(), crate_path);
                dependencies.insert(name, Vec::new());
            }
        }
    }

    // Now fill in dependencies from the registry
    for (_, _state, config) in registry.configured_crates() {
        for grammar in &config.grammars {
            let crate_name = format!("arborium-{}", grammar.id());

            // Only process if this crate is in our map
            if !crate_paths.contains_key(&crate_name) {
                continue;
            }

            // Add dependencies from grammar config
            for dep in &grammar.dependencies {
                // dep.krate is like "arborium-c"
                if crate_paths.contains_key(&dep.krate) {
                    dependencies
                        .get_mut(&crate_name)
                        .unwrap()
                        .push(dep.krate.clone());
                }
            }
        }
    }

    // Topological sort using Kahn's algorithm
    // in_degree[X] = number of crates X depends on
    let mut in_degree: HashMap<String, usize> = HashMap::new();
    for (name, deps) in &dependencies {
        in_degree.insert(name.clone(), deps.len());
    }

    // Build reverse dependency map: dependents[X] = crates that depend on X
    let mut dependents: HashMap<String, Vec<String>> = HashMap::new();
    for name in crate_paths.keys() {
        dependents.insert(name.clone(), Vec::new());
    }
    for (name, deps) in &dependencies {
        for dep in deps {
            if let Some(list) = dependents.get_mut(dep) {
                list.push(name.clone());
            }
        }
    }

    // Start with crates that have no dependencies (in_degree == 0)
    let mut queue: Vec<String> = in_degree
        .iter()
        .filter(|(_, deg)| **deg == 0)
        .map(|(name, _)| name.clone())
        .collect();
    queue.sort(); // Deterministic order for crates at same level

    let mut sorted: Vec<String> = Vec::new();

    while let Some(name) = queue.pop() {
        sorted.push(name.clone());

        // For each crate that depends on this one, decrement its in-degree
        if let Some(deps_on_me) = dependents.get(&name) {
            for dependent in deps_on_me {
                if let Some(deg) = in_degree.get_mut(dependent) {
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push(dependent.clone());
                        queue.sort(); // Keep deterministic
                    }
                }
            }
        }
    }

    // Check for cycles
    if sorted.len() != crate_paths.len() {
        let remaining: Vec<_> = crate_paths.keys().filter(|k| !sorted.contains(k)).collect();
        return Err(miette::miette!(
            "Dependency cycle detected involving: {:?}",
            remaining
        ));
    }

    // Convert names back to paths
    let result: Vec<Utf8PathBuf> = sorted
        .into_iter()
        .filter_map(|name| crate_paths.remove(&name))
        .collect();

    Ok(result)
}

// =============================================================================
// NPM publishing helpers
// =============================================================================

/// Result of attempting to publish a single npm package.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NpmPublishResult {
    Published,
    AlreadyExists,
}

/// Find all npm package directories.
///
/// Supports two layouts:
/// 1. Nested group structure: langs/group-*/lang/npm/package.json
/// 2. Flat structure: dir/lang/package.json (e.g., dist/plugins/rust/package.json)
fn find_npm_packages(packages_dir: &Utf8Path) -> Result<Vec<Utf8PathBuf>> {
    let mut packages = Vec::new();

    if !packages_dir.exists() {
        return Ok(packages);
    }

    for entry in packages_dir
        .read_dir_utf8()
        .into_diagnostic()
        .wrap_err_with(|| format!("Failed to read directory: {}", packages_dir))?
    {
        let entry = entry.into_diagnostic()?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let name = path.file_name().unwrap_or("");

        if name.starts_with("group-") {
            // Nested group structure: group-*/lang/npm/package.json
            for lang_entry in path
                .read_dir_utf8()
                .into_diagnostic()
                .wrap_err_with(|| format!("Failed to read directory: {}", path))?
            {
                let lang_entry = lang_entry.into_diagnostic()?;
                let lang_path = lang_entry.path();

                if !lang_path.is_dir() {
                    continue;
                }

                // Check for npm/package.json
                let npm_dir = lang_path.join("npm");
                if npm_dir.is_dir() && npm_dir.join("package.json").exists() {
                    packages.push(npm_dir);
                }
            }
        } else {
            // Flat structure: lang/package.json (e.g., dist/plugins/rust/package.json)
            if path.join("package.json").exists() {
                packages.push(path.to_path_buf());
            }
        }
    }

    packages.sort();
    Ok(packages)
}

/// Find npm packages in a specific language group.
fn find_group_npm_packages(langs_dir: &Utf8Path, group_name: &str) -> Result<Vec<Utf8PathBuf>> {
    let mut packages = Vec::new();

    let group_dir = langs_dir.join(format!("group-{}", group_name));
    if !group_dir.exists() {
        return Ok(packages);
    }

    for lang_entry in group_dir
        .read_dir_utf8()
        .into_diagnostic()
        .wrap_err_with(|| format!("Failed to read directory: {}", group_dir))?
    {
        let lang_entry = lang_entry.into_diagnostic()?;
        let lang_path = lang_entry.path();

        if !lang_path.is_dir() {
            continue;
        }

        // Check for npm/package.json
        let npm_dir = lang_path.join("npm");
        if npm_dir.is_dir() && npm_dir.join("package.json").exists() {
            packages.push(npm_dir);
        }
    }

    packages.sort();
    Ok(packages)
}

/// Read package name and version from package.json.
fn read_package_info(package_dir: &Utf8Path) -> Result<(String, String)> {
    let package_json_path = package_dir.join("package.json");
    let content = fs_err::read_to_string(&package_json_path)
        .into_diagnostic()
        .wrap_err_with(|| format!("Failed to read {}", package_json_path))?;

    // Simple JSON parsing - extract name and version
    let name = extract_json_string(&content, "name")
        .ok_or_else(|| miette::miette!("No 'name' field in {}", package_json_path))?;
    let version = extract_json_string(&content, "version")
        .ok_or_else(|| miette::miette!("No 'version' field in {}", package_json_path))?;

    Ok((name, version))
}

/// Extract a string value from JSON.
fn extract_json_string(json_str: &str, key: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(json_str).ok()?;
    value.get(key)?.as_str().map(|s| s.to_string())
}

/// Check if a package version already exists on npm.
fn npm_version_exists(package_name: &str, version: &str) -> Result<bool> {
    let output = Command::new("npm")
        .args(["view", &format!("{}@{}", package_name, version), "version"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .into_diagnostic()
        .wrap_err("Failed to run npm view")?;

    // If the command succeeds and outputs the version, it exists
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.trim() == version)
    } else {
        // Command failed - version doesn't exist (or other error, but we'll try to publish)
        Ok(false)
    }
}

/// Publish a single npm package.
fn publish_single_npm_package(
    package_dir: &Utf8Path,
    expected_version: &str,
    dry_run: bool,
) -> Result<NpmPublishResult> {
    let (name, version) = read_package_info(package_dir)?;

    if version != expected_version {
        return Err(miette::miette!(
            "{} has version {} but version.json expects {}",
            package_dir.join("package.json"),
            version,
            expected_version
        ));
    }

    print!("  {} {}@{}...", "→".blue(), name, version);

    // Check if version already exists
    if !dry_run {
        match npm_version_exists(&name, &version) {
            Ok(true) => {
                println!(" {}", "already exists, skipping".yellow());
                return Ok(NpmPublishResult::AlreadyExists);
            }
            Ok(false) => {
                // Continue to publish
            }
            Err(e) => {
                // Couldn't check, try to publish anyway
                eprintln!(" {} checking version: {}", "warning".yellow(), e);
            }
        }
    }

    // Publish (or dry-run publish)
    let mut args = vec!["publish", "--access", "public"];
    if dry_run {
        args.push("--dry-run");
    }

    let output = Command::new("npm")
        .args(&args)
        .current_dir(package_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .into_diagnostic()
        .wrap_err("Failed to run npm publish")?;

    if output.status.success() {
        if dry_run {
            println!(" {}", "dry-run ok".green());
        } else {
            println!(" {}", "published".green());
        }
        return Ok(NpmPublishResult::Published);
    }

    // Check if it's EPUBLISHCONFLICT
    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains("EPUBLISHCONFLICT") || stderr.contains("cannot publish over existing") {
        println!(" {}", "already exists, skipping".yellow());
        return Ok(NpmPublishResult::AlreadyExists);
    }

    // Real error - fail immediately
    println!(" {}", "FAILED".red());
    Err(miette::miette!(
        "npm publish failed for {}:\n{}",
        name,
        stderr
    ))
}
