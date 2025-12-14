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
#[allow(dead_code)]
pub enum Tool {
    /// tree-sitter CLI for generating parsers
    TreeSitter,
    /// git for cloning repositories
    Git,
    /// cargo-component for building WASM components
    CargoComponent,
    /// jco for transpiling WASM components to JS
    Jco,
    /// wasm-opt for optimizing WASM files (optional, for future use)
    WasmOpt,
    /// curl for HTTP requests
    Curl,
    /// wasm-pack for building wasm-bindgen projects (deprecated, use WasmBindgen)
    WasmPack,
    /// wasm-bindgen CLI for generating JS bindings
    WasmBindgen,
}

/// Tools needed for `cargo xtask gen` (generation).
pub const GEN_TOOLS: &[Tool] = &[Tool::TreeSitter, Tool::Git];

/// Tools needed for `cargo xtask build` (WASM plugins).
pub const PLUGIN_TOOLS: &[Tool] = &[Tool::WasmBindgen, Tool::WasmOpt];

/// Tools needed for `cargo xtask serve` (demo assets fetch).
pub const SERVE_TOOLS: &[Tool] = &[Tool::Curl];

impl Tool {
    /// The executable name to search for in PATH.
    pub fn executable_name(self) -> &'static str {
        match self {
            Tool::TreeSitter => "tree-sitter",
            Tool::Git => "git",
            Tool::CargoComponent => "cargo-component",
            Tool::Jco => "jco",
            Tool::WasmOpt => "wasm-opt",
            Tool::Curl => "curl",
            Tool::WasmPack => "wasm-pack",
            Tool::WasmBindgen => "wasm-bindgen",
        }
    }

    /// Human-readable name for error messages.
    pub fn display_name(self) -> &'static str {
        match self {
            Tool::TreeSitter => "tree-sitter",
            Tool::Git => "Git",
            Tool::CargoComponent => "cargo-component",
            Tool::Jco => "jco",
            Tool::WasmOpt => "wasm-opt",
            Tool::Curl => "curl",
            Tool::WasmPack => "wasm-pack",
            Tool::WasmBindgen => "wasm-bindgen",
        }
    }

    /// Homebrew package name (if available).
    pub fn brew_package(self) -> Option<&'static str> {
        match self {
            Tool::TreeSitter => Some("tree-sitter"),
            Tool::Git => Some("git"),
            Tool::CargoComponent => None,
            Tool::Jco => None,
            Tool::WasmOpt => Some("binaryen"),
            Tool::Curl => Some("curl"),
            Tool::WasmPack => None,    // cargo install
            Tool::WasmBindgen => None, // cargo install
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
            Tool::CargoComponent => "cargo binstall -y cargo-component",
            Tool::Jco => "pnpm add -g @bytecodealliance/jco",
            Tool::WasmOpt => {
                if cfg!(target_os = "macos") {
                    "brew install binaryen"
                } else {
                    "Download from https://github.com/WebAssembly/binaryen/releases"
                }
            }
            Tool::Curl => {
                if cfg!(target_os = "macos") {
                    "curl is pre-installed on macOS"
                } else if cfg!(target_os = "linux") {
                    "apt install curl"
                } else {
                    "https://curl.se/download.html"
                }
            }
            Tool::WasmPack => "cargo install wasm-pack",
            Tool::WasmBindgen => "cargo install wasm-bindgen-cli",
        }
    }

    /// Cargo package name for binstall (if available).
    pub fn cargo_package(self) -> Option<&'static str> {
        match self {
            Tool::TreeSitter => None, // not available via binstall
            Tool::Git => None,
            Tool::CargoComponent => Some("cargo-component"),
            Tool::Jco => None,     // npm package, not cargo
            Tool::WasmOpt => None, // binary release, not cargo
            Tool::Curl => None,    // system tool, not cargo
            Tool::WasmPack => Some("wasm-pack"),
            Tool::WasmBindgen => Some("wasm-bindgen-cli"),
        }
    }

    /// Look up the tool in PATH and return its absolute path.
    pub fn find(self) -> Result<ToolPath, ToolNotFound> {
        match which::which(self.executable_name()) {
            Ok(path) => Ok(ToolPath { tool: self, path }),
            Err(_) => Err(ToolNotFound { tool: self }),
        }
    }

    /// Get the version string for this tool (if supported).
    pub fn get_version(self) -> Result<String, std::io::Error> {
        let tool_path = self.find().map_err(|_| {
            std::io::Error::other(format!("{} not found in PATH", self.display_name()))
        })?;

        let version_arg = match self {
            Tool::TreeSitter => "--version",
            Tool::Git => "--version",
            Tool::CargoComponent => "--version",
            Tool::Jco => "--version",
            Tool::WasmOpt => "--version",
            Tool::Curl => "--version",
            Tool::WasmPack => "--version",
            Tool::WasmBindgen => "--version",
        };

        let output = tool_path.command().arg(version_arg).output()?;

        if !output.status.success() {
            return Err(std::io::Error::other(format!(
                "{} --version failed",
                self.display_name()
            )));
        }

        let version_output = String::from_utf8_lossy(&output.stdout);
        // Take the first line and trim whitespace
        let version = version_output.lines().next().unwrap_or("").trim();

        if version.is_empty() {
            return Err(std::io::Error::other(format!(
                "{} returned empty version",
                self.display_name()
            )));
        }

        Ok(version.to_string())
    }
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

    // Print simple text output
    eprintln!("Missing Tools:");
    eprintln!("==============");
    for line in lines {
        eprintln!("{}", line);
    }

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
        assert_eq!(Tool::CargoComponent.executable_name(), "cargo-component");
        assert_eq!(Tool::Jco.executable_name(), "jco");
    }
}
