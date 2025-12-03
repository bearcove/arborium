//! External tool management with nice error messages.
//!
//! This module provides a way to look up external executables with helpful
//! diagnostics when they're not found.

use std::path::PathBuf;
use std::process::Command;

use owo_colors::OwoColorize;
use thiserror::Error;

/// External tools that xtask depends on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    /// tree-sitter CLI for generating parsers
    TreeSitter,
    /// git for cloning repositories
    Git,
    /// wasm-pack for building WASM packages
    WasmPack,
}

/// All tools that xtask may need.
pub const ALL_TOOLS: &[Tool] = &[Tool::TreeSitter, Tool::Git, Tool::WasmPack];

impl Tool {
    /// The executable name to search for in PATH.
    pub fn executable_name(self) -> &'static str {
        match self {
            Tool::TreeSitter => "tree-sitter",
            Tool::Git => "git",
            Tool::WasmPack => "wasm-pack",
        }
    }

    /// Human-readable name for error messages.
    pub fn display_name(self) -> &'static str {
        match self {
            Tool::TreeSitter => "tree-sitter",
            Tool::Git => "Git",
            Tool::WasmPack => "wasm-pack",
        }
    }

    /// Homebrew package name (if available).
    pub fn brew_package(self) -> Option<&'static str> {
        match self {
            Tool::TreeSitter => Some("tree-sitter"),
            Tool::Git => Some("git"),
            Tool::WasmPack => Some("wasm-pack"),
        }
    }

    /// Installation instructions for this tool (platform-aware).
    pub fn install_hint(self) -> &'static str {
        match self {
            Tool::TreeSitter => {
                if cfg!(target_os = "macos") {
                    "brew install tree-sitter  (or: cargo install tree-sitter-cli)"
                } else {
                    "cargo binstall tree-sitter-cli  (or: cargo install tree-sitter-cli)"
                }
            }
            Tool::Git => {
                if cfg!(target_os = "macos") {
                    "xcode-select --install  (or: brew install git)"
                } else if cfg!(target_os = "linux") {
                    "apt install git  (or: dnf install git)"
                } else {
                    "Install Git from https://git-scm.com/"
                }
            }
            Tool::WasmPack => {
                if cfg!(target_os = "macos") {
                    "brew install wasm-pack  (or: cargo install wasm-pack)"
                } else {
                    "cargo binstall wasm-pack  (or: cargo install wasm-pack)"
                }
            }
        }
    }

    /// Cargo package name for binstall (if available).
    pub fn cargo_package(self) -> Option<&'static str> {
        match self {
            Tool::TreeSitter => Some("tree-sitter-cli"),
            Tool::Git => None,
            Tool::WasmPack => Some("wasm-pack"),
        }
    }

    /// Look up the tool in PATH and return its absolute path.
    pub fn find(self) -> Result<ToolPath, ToolNotFound> {
        match which::which(self.executable_name()) {
            Ok(path) => Ok(ToolPath { tool: self, path }),
            Err(_) => Err(ToolNotFound { tool: self }),
        }
    }
}

/// Print a comprehensive tools report showing installed and missing tools.
pub fn print_tools_report() {
    let mut installed = Vec::new();
    let mut missing = Vec::new();

    for &tool in ALL_TOOLS {
        match tool.find() {
            Ok(path) => installed.push((tool, path)),
            Err(_) => missing.push(tool),
        }
    }

    // Print installed tools
    println!("{}", "Installed tools:".cyan().bold());
    if installed.is_empty() {
        println!("  {}", "(none)".dimmed());
    } else {
        for (tool, path) in &installed {
            println!(
                "  {} {} {}",
                "[x]".green(),
                tool.display_name().bold(),
                format!("({})", path.path().display()).dimmed()
            );
        }
    }

    // Print missing tools
    println!("\n{}", "Missing tools:".cyan().bold());
    if missing.is_empty() {
        println!("  {}", "(none - all tools available!)".green());
    } else {
        for tool in &missing {
            println!("  {} {}", "[ ]".red(), tool.display_name().bold());
            println!("       {}", tool.install_hint().yellow());
        }

        // Provide combined install commands
        if cfg!(target_os = "macos") {
            let brew_packages: Vec<_> = missing
                .iter()
                .filter_map(|t| t.brew_package())
                .collect();

            if !brew_packages.is_empty() {
                println!("\n{}", "Quick install (macOS):".green().bold());
                println!("  {}", format!("brew install {}", brew_packages.join(" ")).yellow());
            }
        } else {
            let cargo_packages: Vec<_> = missing
                .iter()
                .filter_map(|t| t.cargo_package())
                .collect();

            if !cargo_packages.is_empty() {
                println!("\n{}", "Quick install (with cargo-binstall):".green().bold());
                println!("  {}", format!("cargo binstall {}", cargo_packages.join(" ")).yellow());
            }
        }
    }
}

/// Check required tools and print a report. Returns true if all are available.
pub fn check_tools_or_report(required: &[Tool]) -> bool {
    let mut installed = Vec::new();
    let mut missing = Vec::new();

    for &tool in ALL_TOOLS {
        let is_required = required.contains(&tool);
        match tool.find() {
            Ok(path) => installed.push((tool, path, is_required)),
            Err(_) => missing.push((tool, is_required)),
        }
    }

    // Check if any required tools are missing
    let missing_required: Vec<_> = missing.iter().filter(|(_, req)| *req).collect();
    if missing_required.is_empty() {
        return true;
    }

    // Print report
    eprintln!("{}", "Installed tools:".cyan().bold());
    if installed.is_empty() {
        eprintln!("  {}", "(none)".dimmed());
    } else {
        for (tool, path, _required) in &installed {
            eprintln!(
                "  {} {} {}",
                "[x]".green(),
                tool.display_name().bold(),
                format!("({})", path.path().display()).dimmed()
            );
        }
    }

    eprintln!("\n{}", "Missing tools:".cyan().bold());
    for (tool, required) in &missing {
        if *required {
            eprintln!(
                "  {} {} {}",
                "[ ]".red(),
                tool.display_name().bold(),
                "(required)".red()
            );
            eprintln!("       {}", tool.install_hint().yellow());
        } else {
            eprintln!(
                "  {} {}",
                "[-]".dimmed(),
                tool.display_name().dimmed()
            );
        }
    }

    // Provide combined install commands for missing required tools
    let missing_required_tools: Vec<_> = missing.iter().filter(|(_, req)| *req).map(|(t, _)| *t).collect();

    if cfg!(target_os = "macos") {
        let brew_packages: Vec<_> = missing_required_tools
            .iter()
            .filter_map(|t| t.brew_package())
            .collect();

        if !brew_packages.is_empty() {
            eprintln!("\n{}", "Quick install (macOS):".green().bold());
            eprintln!("  {}", format!("brew install {}", brew_packages.join(" ")).yellow());
        }
    } else {
        let cargo_packages: Vec<_> = missing_required_tools
            .iter()
            .filter_map(|t| t.cargo_package())
            .collect();

        if !cargo_packages.is_empty() {
            eprintln!("\n{}", "Quick install (with cargo-binstall):".green().bold());
            eprintln!("  {}", format!("cargo binstall {}", cargo_packages.join(" ")).yellow());
        }
    }
    eprintln!();

    false
}

/// A resolved tool with its absolute path.
#[derive(Debug, Clone)]
pub struct ToolPath {
    #[allow(dead_code)]
    tool: Tool,
    path: PathBuf,
}

impl ToolPath {
    /// Create a new Command for this tool.
    pub fn command(&self) -> Command {
        Command::new(&self.path)
    }

    /// Get the absolute path to the tool.
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}

/// Error when a required tool is not found in PATH.
#[derive(Debug, Error)]
#[error("{} not found in PATH\n\n  {}", .tool.display_name(), .tool.install_hint())]
pub struct ToolNotFound {
    pub tool: Tool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_names() {
        assert_eq!(Tool::TreeSitter.executable_name(), "tree-sitter");
        assert_eq!(Tool::Git.executable_name(), "git");
        assert_eq!(Tool::WasmPack.executable_name(), "wasm-pack");
    }
}
