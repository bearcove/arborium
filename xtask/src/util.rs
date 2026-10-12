//! Shared utilities for xtask commands

use camino::Utf8Path;
use std::env;
use std::path::PathBuf;

/// Render a relative path with `/` separators, for paths written into
/// generated manifests (Cargo.toml `path = "..."`) and URLs. On Windows,
/// `Display` uses `\`, which TOML reads as an escape sequence.
pub fn slash_path(path: &Utf8Path) -> String {
    if cfg!(windows) {
        path.as_str().replace('\\', "/")
    } else {
        path.as_str().to_owned()
    }
}

/// Find repository root by looking for .git directory
pub fn find_repo_root() -> Option<PathBuf> {
    let cwd = env::current_dir().ok()?;
    let mut current = cwd.clone();

    loop {
        let git_dir = current.join(".git");
        if git_dir.exists() {
            return Some(current);
        }
        if !current.pop() {
            return None;
        }
    }
}
