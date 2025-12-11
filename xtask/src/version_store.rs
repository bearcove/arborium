use camino::Utf8Path;
use miette::{Context, IntoDiagnostic, Result};

const VERSION_FILE: &str = "version.json";

#[derive(Debug, Clone, facet::Facet)]
#[facet(rename_all = "snake_case")]
struct VersionEntry {
    pub version: String,
}

pub fn write_version(repo_root: &Utf8Path, version: &str) -> Result<()> {
    let path = repo_root.join(VERSION_FILE);
    let entry = VersionEntry {
        version: version.to_string(),
    };
    let content = facet_json::to_string_pretty(&entry);
    fs_err::write(&path, content)
        .into_diagnostic()
        .context("failed to write version.json")?;

    // Also update packages/arborium/package.json
    sync_main_npm_package_version(repo_root, version)?;

    Ok(())
}

/// Ensure packages/arborium/package.json matches the canonical version.
pub fn sync_main_npm_package_version(repo_root: &Utf8Path, version: &str) -> Result<()> {
    update_main_npm_package_version(repo_root, version)
}

/// Update the version in packages/arborium/package.json
fn update_main_npm_package_version(repo_root: &Utf8Path, version: &str) -> Result<()> {
    let package_json_path = repo_root.join("packages/arborium/package.json");

    if !package_json_path.exists() {
        // Package doesn't exist yet, skip
        return Ok(());
    }

    let content = fs_err::read_to_string(&package_json_path)
        .into_diagnostic()
        .context("failed to read packages/arborium/package.json")?;

    // Parse as serde_json::Value to preserve structure
    let mut json: serde_json::Value = serde_json::from_str(&content)
        .into_diagnostic()
        .context("failed to parse packages/arborium/package.json")?;

    // Update version field
    if let Some(obj) = json.as_object_mut() {
        obj.insert(
            "version".to_string(),
            serde_json::Value::String(version.to_string()),
        );
    }

    // Write back with pretty formatting
    let updated = serde_json::to_string_pretty(&json)
        .into_diagnostic()
        .context("failed to serialize packages/arborium/package.json")?;

    fs_err::write(&package_json_path, updated + "\n")
        .into_diagnostic()
        .context("failed to write packages/arborium/package.json")?;

    Ok(())
}

pub fn read_version(repo_root: &Utf8Path) -> Result<String> {
    let path = repo_root.join(VERSION_FILE);
    let content = fs_err::read_to_string(&path)
        .into_diagnostic()
        .context("failed to read version.json; run `cargo xtask gen --version <x.y.z>`")?;
    let entry: VersionEntry = facet_json::from_str(&content)
        .into_diagnostic()
        .context("failed to parse version.json")?;
    Ok(entry.version)
}
