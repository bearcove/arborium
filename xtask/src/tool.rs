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
    /// cargo-component for building WASM components
    CargoComponent,
    /// jco for transpiling WASM components to JS
    Jco,
}

/// All tools that xtask may need.
pub const ALL_TOOLS: &[Tool] = &[
    Tool::TreeSitter,
    Tool::Git,
    Tool::WasmPack,
    Tool::CargoComponent,
    Tool::Jco,
];

/// Tools needed for `cargo xtask gen` (generation).
pub const GEN_TOOLS: &[Tool] = &[Tool::TreeSitter, Tool::Git];

/// Tools needed for `cargo xtask serve` (WASM demo).
pub const SERVE_TOOLS: &[Tool] = &[Tool::WasmPack];

/// Tools needed for `cargo xtask plugins` (WASM component plugins).
pub const PLUGIN_TOOLS: &[Tool] = &[Tool::CargoComponent, Tool::Jco];

impl Tool {
    /// The executable name to search for in PATH.
    pub fn executable_name(self) -> &'static str {
        match self {
            Tool::TreeSitter => "tree-sitter",
            Tool::Git => "git",
            Tool::WasmPack => "wasm-pack",
            Tool::CargoComponent => "cargo-component",
            Tool::Jco => "jco",
        }
    }

    /// Human-readable name for error messages.
    pub fn display_name(self) -> &'static str {
        match self {
            Tool::TreeSitter => "tree-sitter",
            Tool::Git => "Git",
            Tool::WasmPack => "wasm-pack",
            Tool::CargoComponent => "cargo-component",
            Tool::Jco => "jco",
        }
    }

    /// Homebrew package name (if available).
    pub fn brew_package(self) -> Option<&'static str> {
        match self {
            Tool::TreeSitter => Some("tree-sitter"),
            Tool::Git => Some("git"),
            Tool::WasmPack => Some("wasm-pack"),
            Tool::CargoComponent => None,
            Tool::Jco => None,
        }
    }

    /// Installation instructions for this tool (platform-aware).
    pub fn install_hint(self) -> &'static str {
        match self {
            Tool::TreeSitter => {
                if cfg!(target_os = "macos") {
                    "brew install tree-sitter"
                } else {
                    "cargo install tree-sitter-cli --locked"
                }
            }
            Tool::Git => {
                if cfg!(target_os = "macos") {
                    "xcode-select --install"
                } else if cfg!(target_os = "linux") {
                    "apt install git"
                } else {
                    "https://git-scm.com/"
                }
            }
            Tool::WasmPack => {
                if cfg!(target_os = "macos") {
                    "brew install wasm-pack"
                } else {
                    "cargo binstall -y wasm-pack"
                }
            }
            Tool::CargoComponent => "cargo binstall -y cargo-component",
            Tool::Jco => "pnpm add -g @bytecodealliance/jco",
        }
    }

    /// Cargo package name for binstall (if available).
    pub fn cargo_package(self) -> Option<&'static str> {
        match self {
            Tool::TreeSitter => None, // not available via binstall
            Tool::Git => None,
            Tool::WasmPack => Some("wasm-pack"),
            Tool::CargoComponent => Some("cargo-component"),
            Tool::Jco => None, // npm package, not cargo
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

    // Build content lines
    let mut lines = Vec::new();

    if installed.is_empty() && missing.is_empty() {
        lines.push("(no tools configured)".dimmed().to_string());
    } else {
        for (tool, path) in &installed {
            lines.push(format!(
                "{} {} {}",
                "✓".green().bold(),
                tool.display_name().bold(),
                format!("({})", path.path().display()).dimmed()
            ));
        }
        for tool in &missing {
            lines.push(format!(
                "{} {}",
                "✗".red().bold(),
                tool.display_name().bold()
            ));
            lines.push(format!("    {}", tool.install_hint().yellow()));
        }
    }

    // Add quick install section only if more than one tool is missing
    if missing.len() > 1 {
        lines.push(String::new());
        if cfg!(target_os = "macos") {
            let brew_packages: Vec<_> = missing.iter().filter_map(|t| t.brew_package()).collect();

            if brew_packages.len() > 1 {
                lines.push(format!("{}", "Quick install:".green().bold()));
                lines.push(format!(
                    "  {}",
                    format!("brew install {}", brew_packages.join(" ")).yellow()
                ));
            }
        } else {
            let cargo_packages: Vec<_> = missing.iter().filter_map(|t| t.cargo_package()).collect();

            if cargo_packages.len() > 1 {
                lines.push(format!("{}", "Quick install:".green().bold()));
                lines.push(format!(
                    "  {}",
                    format!("cargo binstall -y {}", cargo_packages.join(" ")).yellow()
                ));
            }
        }
    }

    // Render with boxen
    let content = lines.join("\n");
    let boxed = boxen::builder()
        .border_style(boxen::BorderStyle::Round)
        .padding(1)
        .title("Tools Status")
        .border_color("cyan")
        .render(&content)
        .unwrap_or(content);

    println!("{}", boxed);
}

/// Check specified tools and print a report. Returns true if all are available.
pub fn check_tools_or_report(tools: &[Tool]) -> bool {
    let mut installed = Vec::new();
    let mut missing = Vec::new();

    for &tool in tools {
        match tool.find() {
            Ok(path) => installed.push((tool, path)),
            Err(_) => missing.push(tool),
        }
    }

    if missing.is_empty() {
        return true;
    }

    // Build content lines
    let mut lines = Vec::new();

    for (tool, path) in &installed {
        lines.push(format!(
            "{} {} {}",
            "✓".green().bold(),
            tool.display_name().bold(),
            format!("({})", path.path().display()).dimmed()
        ));
    }

    for tool in &missing {
        lines.push(format!(
            "{} {}",
            "✗".red().bold(),
            tool.display_name().bold(),
        ));
        lines.push(format!("    {}", tool.install_hint().yellow()));
    }

    // Provide combined install command if more than one tool is missing
    if missing.len() > 1 {
        lines.push(String::new());
        if cfg!(target_os = "macos") {
            let brew_packages: Vec<_> = missing.iter().filter_map(|t| t.brew_package()).collect();

            if brew_packages.len() > 1 {
                lines.push(format!("{}", "Quick install:".green().bold()));
                lines.push(format!(
                    "  {}",
                    format!("brew install {}", brew_packages.join(" ")).yellow()
                ));
            }
        } else {
            let cargo_packages: Vec<_> = missing.iter().filter_map(|t| t.cargo_package()).collect();

            if cargo_packages.len() > 1 {
                lines.push(format!("{}", "Quick install:".green().bold()));
                lines.push(format!(
                    "  {}",
                    format!("cargo binstall -y {}", cargo_packages.join(" ")).yellow()
                ));
            }
        }
    }

    // Render with boxen
    let content = lines.join("\n");
    let boxed = boxen::builder()
        .border_style(boxen::BorderStyle::Round)
        .padding(1)
        .title("Missing Tools")
        .border_color("red")
        .render(&content)
        .unwrap_or(content);

    eprintln!("{}", boxed);

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
        assert_eq!(Tool::CargoComponent.executable_name(), "cargo-component");
        assert_eq!(Tool::Jco.executable_name(), "jco");
    }
}
