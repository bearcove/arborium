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

/// Download a published crate and extract its version from Cargo.toml.
/// Returns None if the crate doesn't exist on crates.io.
fn get_published_crate_version(name: &str) -> Result<Option<String>> {
    // Download latest version (without specifying version)
    let output = std::process::Command::new("cargo")
        .args(["download", name])
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

        // Look for Cargo.toml in the archive: {name}-{version}/Cargo.toml
        if path.file_name() == Some(std::ffi::OsStr::new("Cargo.toml")) {
            let mut contents = String::new();
            std::io::Read::read_to_string(&mut entry, &mut contents).into_diagnostic()?;

            // Parse version from [package] section using proper TOML parser
            if let Some(version) = extract_toml_string(&contents, "version") {
                return Ok(Some(version));
            }
        }
    }

    Ok(None) // No Cargo.toml or version found
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
            // Track published versions for POST crate regeneration
            let grammar_versions = publish_grammar_crates(repo_root, &langs_dir, dry_run, verbose)?;
            println!();

            // 3. Regenerate POST crates with actual published versions
            // Always regenerate (even in dry-run) so POST crates validate against correct versions
            println!(
                "  {} Regenerating POST crates with published versions{}...",
                "●".cyan(),
                if dry_run { " [dry-run]" } else { "" }
            );
            regenerate_umbrella_crate(repo_root, &grammar_versions)?;
            println!("    {}", "done".green());
            println!();

            // 4. Post crates
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
            // The main package bundles the browser host artifacts built by wasm-pack.
            // Those outputs are gitignored, so CI must generate them before publishing.
            ensure_main_npm_package_host_artifacts(repo_root)?;
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
            (CratePublishResult::Published, _) => published += 1,
            (CratePublishResult::AlreadyExists, _) => skipped += 1,
            (CratePublishResult::Failed, _) => failed += 1,
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

/// Maximum number of retry attempts for transient publish failures.
const MAX_PUBLISH_RETRIES: u32 = 3;

/// Initial backoff delay in milliseconds.
const INITIAL_BACKOFF_MS: u64 = 1000;

/// Publish a single crate with retry logic for transient failures.
///
/// Retries up to MAX_PUBLISH_RETRIES times with exponential backoff for
/// transient errors (network issues, timeouts). Does not retry for permanent
/// errors (already exists, validation failures).
fn publish_single_crate_with_retry(
    crate_dir: &Utf8Path,
    dry_run: bool,
    verbose: bool,
) -> Result<(CratePublishResult, Option<(String, String)>)> {
    let mut last_error = None;

    for attempt in 0..=MAX_PUBLISH_RETRIES {
        if attempt > 0 {
            let backoff_ms = INITIAL_BACKOFF_MS * 2u64.pow(attempt - 1);
            eprintln!(
                "      {} Retry {} after {}ms...",
                "↻".yellow(),
                attempt,
                backoff_ms
            );
            std::thread::sleep(std::time::Duration::from_millis(backoff_ms));
        }

        match publish_single_crate(crate_dir, dry_run, verbose) {
            Ok(result) => return Ok(result),
            Err(e) => {
                let error_str = format!("{:?}", e);
                // Check if this is a transient error worth retrying
                let is_transient = error_str.contains("network")
                    || error_str.contains("timeout")
                    || error_str.contains("connection")
                    || error_str.contains("503")
                    || error_str.contains("502")
                    || error_str.contains("500")
                    || error_str.contains("temporarily unavailable");

                if is_transient && attempt < MAX_PUBLISH_RETRIES {
                    eprintln!(
                        "      {} Transient error: {}",
                        "⚠".yellow(),
                        error_str.dimmed()
                    );
                    last_error = Some(e);
                    continue;
                }

                // Non-transient error or out of retries
                return Err(e);
            }
        }
    }

    // Should not reach here, but return last error if we do
    Err(last_error.unwrap_or_else(|| miette::miette!("Unknown error after retries")))
}

/// Publish a single crate.
/// Returns the publish result and optionally the (name, version) tuple for tracking.
fn publish_single_crate(
    crate_dir: &Utf8Path,
    dry_run: bool,
    verbose: bool,
) -> Result<(CratePublishResult, Option<(String, String)>)> {
    // Check if Cargo.toml exists
    if !crate_dir.join("Cargo.toml").exists() {
        println!(
            "  {} {} - {}",
            "→".blue(),
            crate_dir,
            "no Cargo.toml, skipping".yellow()
        );
        return Ok((CratePublishResult::AlreadyExists, None));
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
        return Ok((CratePublishResult::AlreadyExists, None));
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
                    // In dry-run, don't return version info
                    return Ok((CratePublishResult::AlreadyExists, None));
                } else {
                    println!(" {}", "unchanged (hash match), skipping".dimmed());
                    // Get the published version from crates.io since we're skipping
                    let published_version =
                        get_published_crate_version(&name)?.unwrap_or_else(|| version.clone());
                    return Ok((
                        CratePublishResult::AlreadyExists,
                        Some((name.clone(), published_version)),
                    ));
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
                // Get the published version (should be the same as local, but query to be sure)
                let published_version =
                    get_published_crate_version(&name)?.unwrap_or_else(|| version.clone());
                return Ok((
                    CratePublishResult::AlreadyExists,
                    Some((name.clone(), published_version)),
                ));
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
                return Ok((CratePublishResult::Published, None));
            } else {
                println!("  {}", "published".green());
                return Ok((CratePublishResult::Published, Some((name, version))));
            }
        } else {
            println!("  {}", "FAILED".red());
            return Ok((CratePublishResult::Failed, None));
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
            return Ok((CratePublishResult::Published, None));
        } else {
            println!(" {}", "published".green());
            return Ok((CratePublishResult::Published, Some((name, version))));
        }
    }

    // Check if it's an "already published" error
    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains("already uploaded")
        || stderr.contains("already exists")
        || stderr.contains("is already published")
    {
        println!(" {}", "already exists, skipping".yellow());
        // Get the published version from crates.io
        let published_version =
            get_published_crate_version(&name)?.unwrap_or_else(|| version.clone());
        return Ok((
            CratePublishResult::AlreadyExists,
            Some((name, published_version)),
        ));
    }

    // Real error
    println!(" {}", "FAILED".red());
    eprintln!("    stderr: {}", stderr);
    Ok((CratePublishResult::Failed, None))
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

/// Show the topological levels for grammar crate publication order.
///
/// This is useful for debugging and understanding the dependency structure.
pub fn show_levels(repo_root: &Utf8Path, langs_dir: &Utf8Path) -> Result<()> {
    println!("{}", "Grammar crate publication levels:".cyan().bold());
    println!();

    let levels = topological_sort_grammar_crates(repo_root, langs_dir)?;

    for (level_idx, level_crates) in levels.iter().enumerate() {
        println!(
            "  {} Level {} ({} crates):",
            "●".cyan(),
            level_idx,
            level_crates.len()
        );
        for crate_path in level_crates {
            if let Ok((name, _)) = read_crate_info(crate_path) {
                // Get dependencies for this crate
                let deps = extract_arborium_deps(crate_path).unwrap_or_default();
                if deps.is_empty() {
                    println!("      {} {}", "→".blue(), name);
                } else {
                    println!(
                        "      {} {} (depends on: {})",
                        "→".blue(),
                        name,
                        deps.join(", ").dimmed()
                    );
                }
            }
        }
        println!();
    }

    let total: usize = levels.iter().map(|l| l.len()).sum();
    println!(
        "{} Total: {} crates across {} levels",
        "✓".green().bold(),
        total,
        levels.len()
    );

    Ok(())
}

/// Extract arborium-* dependencies from a crate's Cargo.toml.
///
/// Parses the actual Cargo.toml file (the source of truth) rather than
/// relying on KDL config metadata. This ensures publication order matches
/// real dependency relationships.
fn extract_arborium_deps(crate_dir: &Utf8Path) -> Result<Vec<String>> {
    let cargo_toml_path = crate_dir.join("Cargo.toml");
    let content = fs_err::read_to_string(&cargo_toml_path)
        .into_diagnostic()
        .wrap_err_with(|| format!("Failed to read {}", cargo_toml_path))?;

    let doc: toml_edit::DocumentMut = content
        .parse()
        .into_diagnostic()
        .wrap_err_with(|| format!("Failed to parse {}", cargo_toml_path))?;

    let mut deps = Vec::new();

    // Check [dependencies] section
    if let Some(deps_table) = doc.get("dependencies").and_then(|d| d.as_table()) {
        for (name, _) in deps_table {
            // Only include arborium-* grammar crates, not infrastructure crates
            if name.starts_with("arborium-")
                && name != "arborium-sysroot"
                && name != "arborium-test-harness"
                && name != "arborium-tree-sitter"
                && name != "arborium-highlight"
                && name != "arborium-theme"
                && name != "tree-sitter-language"
            {
                deps.push(name.to_string());
            }
        }
    }

    Ok(deps)
}

/// Collect ALL grammar crates and sort them topologically by dependencies.
///
/// Returns crates grouped into "levels" for parallel publishing:
/// - Level 0: Crates with no dependencies on other grammar crates
/// - Level 1: Crates that only depend on Level 0 crates
/// - Level N: Crates that only depend on Level 0..N-1 crates
///
/// This ensures crates are published leaves-first: if crate A depends on crate B,
/// then B will be in an earlier level than A. Crates within the same level
/// can be published in parallel since they don't depend on each other.
fn topological_sort_grammar_crates(
    _repo_root: &Utf8Path,
    langs_dir: &Utf8Path,
) -> Result<Vec<Vec<Utf8PathBuf>>> {
    use petgraph::graph::DiGraph;
    use petgraph::graph::NodeIndex;

    // Build a map from crate name -> crate path
    let mut crate_paths: HashMap<String, Utf8PathBuf> = HashMap::new();

    // Collect all grammar crates from all groups
    let groups = find_all_groups(langs_dir)?;
    for group_name in &groups {
        let group_crates = find_group_crates(langs_dir, group_name)?;
        for crate_path in group_crates {
            // Extract crate name from Cargo.toml
            if let Ok((name, _)) = read_crate_info(&crate_path) {
                crate_paths.insert(name.clone(), crate_path);
            }
        }
    }

    // Build dependency graph from actual Cargo.toml files
    let mut graph: DiGraph<String, ()> = DiGraph::new();
    let mut node_indices: HashMap<String, NodeIndex> = HashMap::new();

    // Add all crates as nodes
    for name in crate_paths.keys() {
        let idx = graph.add_node(name.clone());
        node_indices.insert(name.clone(), idx);
    }

    // Add edges based on actual Cargo.toml dependencies
    for (name, path) in &crate_paths {
        let deps = extract_arborium_deps(path)?;
        for dep in deps {
            // Only add edge if the dependency is also a grammar crate we're publishing
            if let (Some(&from), Some(&to)) = (node_indices.get(name), node_indices.get(&dep)) {
                // Edge goes from dependent -> dependency (A depends on B means A -> B)
                graph.add_edge(from, to, ());
            }
        }
    }

    // Topological sort using petgraph
    let sorted = petgraph::algo::toposort(&graph, None).map_err(|cycle| {
        let cycle_node = &graph[cycle.node_id()];
        miette::miette!("Dependency cycle detected involving: {}", cycle_node)
    })?;

    // Group nodes into levels by depth (BFS from leaves)
    // Depth = longest path from this node to any leaf
    let mut depths: HashMap<NodeIndex, usize> = HashMap::new();

    // Process in reverse topological order (leaves first)
    for &node_idx in sorted.iter().rev() {
        // Get max depth of all dependencies, then add 1
        let max_dep_depth = graph
            .neighbors(node_idx)
            .filter_map(|dep| depths.get(&dep))
            .max()
            .copied()
            .unwrap_or(0);

        // If no dependencies, depth is 0; otherwise depth is max_dep_depth + 1
        let has_deps = graph.neighbors(node_idx).next().is_some();
        let depth = if has_deps { max_dep_depth + 1 } else { 0 };
        depths.insert(node_idx, depth);
    }

    // Group by depth
    let max_depth = depths.values().max().copied().unwrap_or(0);
    let mut levels: Vec<Vec<Utf8PathBuf>> = vec![Vec::new(); max_depth + 1];

    for (name, &node_idx) in &node_indices {
        let depth = depths.get(&node_idx).copied().unwrap_or(0);
        if let Some(path) = crate_paths.get(name) {
            levels[depth].push(path.clone());
        }
    }

    // Sort each level for deterministic output
    for level in &mut levels {
        level.sort();
    }

    // Remove empty levels (shouldn't happen, but be safe)
    levels.retain(|level| !level.is_empty());

    Ok(levels)
}

/// Publish grammar crates in topological order and track their published versions.
///
/// Crates are published in "levels" with parallelism within each level:
/// - Level 0: All crates with no grammar dependencies (published in parallel)
/// - Level 1: All crates depending only on Level 0 (published in parallel after Level 0 completes)
/// - etc.
///
/// This provides a barrier between levels (crates in Level N wait for Level N-1 to complete)
/// while maximizing parallelism within each level.
///
/// Returns a HashMap of crate_name -> published_version for use in regenerating POST crates.
/// Maximum number of concurrent crate publishes per level.
/// crates.io has no rate limits (https://github.com/rust-lang/crates.io/issues/11685)
/// but we limit concurrency to keep output readable and avoid potential issues.
const MAX_PUBLISH_CONCURRENCY: usize = 8;

fn publish_grammar_crates(
    repo_root: &Utf8Path,
    langs_dir: &Utf8Path,
    dry_run: bool,
    verbose: bool,
) -> Result<HashMap<String, String>> {
    use rayon::prelude::*;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicUsize, Ordering};

    // Create a thread pool with limited concurrency
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(MAX_PUBLISH_CONCURRENCY)
        .build()
        .into_diagnostic()
        .wrap_err("Failed to create thread pool")?;

    println!(
        "  {} Publishing {} (topologically sorted with parallel levels)...",
        "●".cyan(),
        "grammar crates".bold()
    );

    let levels = topological_sort_grammar_crates(repo_root, langs_dir)?;
    let total_crates: usize = levels.iter().map(|l| l.len()).sum();

    println!("    {} crates across {} levels", total_crates, levels.len());

    let published = AtomicUsize::new(0);
    let skipped = AtomicUsize::new(0);
    let failed = AtomicUsize::new(0);
    let versions = Mutex::new(HashMap::new());

    for (level_idx, level_crates) in levels.iter().enumerate() {
        println!(
            "    {} Level {} ({} crates)...",
            "→".blue(),
            level_idx,
            level_crates.len()
        );

        // Publish this level in parallel using rayon with limited concurrency
        // The collect() at the end acts as a barrier - we wait for all crates in this level
        // to finish before moving to the next level
        let results: Vec<Result<(CratePublishResult, Option<(String, String)>)>> =
            pool.install(|| {
                level_crates
                    .par_iter()
                    .map(|crate_dir| publish_single_crate_with_retry(crate_dir, dry_run, verbose))
                    .collect()
            });

        // Process results
        for result in results {
            match result? {
                (CratePublishResult::Published, Some((name, version))) => {
                    published.fetch_add(1, Ordering::Relaxed);
                    versions.lock().unwrap().insert(name, version);
                }
                (CratePublishResult::AlreadyExists, Some((name, version))) => {
                    skipped.fetch_add(1, Ordering::Relaxed);
                    versions.lock().unwrap().insert(name, version);
                }
                (CratePublishResult::Failed, _) => {
                    failed.fetch_add(1, Ordering::Relaxed);
                }
                (_, None) => {
                    // Dry-run or other cases without version tracking
                }
            }
        }

        // Check for failures after each level - don't continue if any failed
        let failed_count = failed.load(Ordering::Relaxed);
        if failed_count > 0 {
            return Err(miette::miette!(
                "{} grammar crates failed to publish in level {}",
                failed_count,
                level_idx
            ));
        }
    }

    let published_count = published.load(Ordering::Relaxed);
    let skipped_count = skipped.load(Ordering::Relaxed);
    let failed_count = failed.load(Ordering::Relaxed);

    println!(
        "    {} published, {} skipped, {} failed",
        published_count.to_string().green(),
        skipped_count.to_string().yellow(),
        failed_count.to_string().red()
    );

    Ok(versions.into_inner().unwrap())
}

/// Regenerate the umbrella crate (crates/arborium/Cargo.toml) with actual published versions.
/// This ensures POST crates reference the correct versions that are actually on crates.io.
fn regenerate_umbrella_crate(
    repo_root: &Utf8Path,
    grammar_versions: &HashMap<String, String>,
) -> Result<()> {
    // Read workspace version
    let workspace_version = crate::version_store::read_version(repo_root)?;

    // Load crate registry to find all grammar crates
    let crates_dir = repo_root.join("crates");
    let registry = crate::types::CrateRegistry::load(&crates_dir)
        .map_err(|e| miette::miette!("Failed to load crate registry: {}", e))?;

    // Collect all grammar crates with their paths and versions
    let mut grammar_crates: Vec<(String, String, Utf8PathBuf, String)> = Vec::new();

    for (name, state, _) in registry.configured_crates() {
        // This is a grammar crate
        let crate_path = state.crate_path.clone();
        let grammar_id = name.strip_prefix("arborium-").unwrap_or(&name).to_string();

        // Get the actual published version for this crate
        let version = grammar_versions
            .get(name.as_str())
            .cloned()
            .unwrap_or_else(|| workspace_version.clone());

        grammar_crates.push((name.clone(), grammar_id, crate_path, version));
    }

    grammar_crates.sort_by(|a, b| a.0.cmp(&b.0));

    // Build Cargo.toml content
    let mut content = String::new();

    // Package section
    content.push_str(&format!(
        r#"[package]
name = "arborium"
version = "{workspace_version}"
edition = "2024"
license = "MIT OR Apache-2.0"
repository = "https://github.com/bearcove/arborium"
description = "Tree-sitter syntax highlighting with HTML rendering and WASM support"
keywords = ["tree-sitter", "syntax-highlighting", "wasm"]
categories = ["parsing", "text-processing", "wasm"]
links = "arborium"

[features]
default = []

# All languages
all-languages = [
"#
    ));

    // Add all lang-* features
    for (_, grammar_id, _, _) in &grammar_crates {
        if grammar_id.ends_with("_inline") {
            continue;
        }
        content.push_str(&format!("    \"lang-{}\",\n", grammar_id));
    }
    content.push_str("]\n\n");

    // Individual language features
    content.push_str("# Individual language features\n");
    for (name, grammar_id, _, _) in &grammar_crates {
        content.push_str(&format!("lang-{} = [\"dep:{}\"]\n", grammar_id, name));
    }

    // Dependencies section - use actual published versions
    content.push_str(&format!(
        r#"

[dependencies]
arborium-tree-sitter = {{ version = "{workspace_version}", path = "../arborium-tree-sitter" }}
arborium-theme = {{ version = "{workspace_version}", path = "../arborium-theme" }}
arborium-highlight = {{ version = "{workspace_version}", path = "../arborium-highlight", features = ["tree-sitter"] }}

# Optional grammar dependencies
"#
    ));

    for (name, _, crate_path, version) in &grammar_crates {
        let rel_path = crate_path.strip_prefix(repo_root).unwrap_or(crate_path);
        content.push_str(&format!(
            "{} = {{ version = \"{}\", path = \"../../{}\", optional = true }}\n",
            name, version, rel_path
        ));
    }

    content.push_str(
        r#"

[dev-dependencies]
indoc = "2"

# WASM allocator (automatically enabled on wasm targets)
[target.'cfg(target_family = "wasm")'.dependencies]
dlmalloc = "0.2"
"#,
    );

    // Write the file
    let cargo_toml_path = repo_root.join("crates/arborium/Cargo.toml");
    fs_err::write(&cargo_toml_path, content)
        .into_diagnostic()
        .wrap_err_with(|| format!("Failed to write {}", cargo_toml_path))?;

    Ok(())
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
    if !dry_run && should_use_npm_provenance() {
        args.push("--provenance");
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

fn should_use_npm_provenance() -> bool {
    // When running under GitHub Actions with `permissions: id-token: write`, this
    // env var is available and npm can generate provenance / use trusted publishing.
    std::env::var_os("ACTIONS_ID_TOKEN_REQUEST_URL").is_some()
}

fn ensure_main_npm_package_host_artifacts(repo_root: &Utf8Path) -> Result<()> {
    // wasm-pack output (gitignored, so it won't exist on a fresh checkout).
    let demo_pkg = repo_root.join("demo/pkg");
    let host_js = demo_pkg.join("arborium_host.js");
    let host_wasm = demo_pkg.join("arborium_host_bg.wasm");

    if host_js.exists() && host_wasm.exists() {
        return Ok(());
    }

    crate::build::build_host(repo_root)?;

    if !host_js.exists() || !host_wasm.exists() {
        return Err(miette::miette!(
            "Expected arborium-host artifacts missing after build: {}, {}",
            host_js,
            host_wasm
        ));
    }

    Ok(())
}
