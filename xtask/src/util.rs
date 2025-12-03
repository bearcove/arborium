//! Shared utilities for xtask commands

use std::env;
use std::fs;
use std::path::PathBuf;

/// Find the repository root by looking for Cargo.toml with [workspace] and GRAMMARS.toml
pub fn find_repo_root() -> Option<PathBuf> {
    let cwd = env::current_dir().ok()?;
    let mut current = cwd.clone();

    loop {
        // Look for Cargo.toml with [workspace] and GRAMMARS.toml
        let cargo_toml = current.join("Cargo.toml");
        let grammars_toml = current.join("GRAMMARS.toml");
        if cargo_toml.exists() && grammars_toml.exists() {
            if let Ok(contents) = fs::read_to_string(&cargo_toml) {
                if contents.contains("[workspace]") {
                    return Some(current);
                }
            }
        }
        if !current.pop() {
            return None;
        }
    }
}
