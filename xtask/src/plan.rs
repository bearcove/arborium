//! Plan-execute pattern for xtask operations.
//!
//! This module provides a way to plan operations before executing them,
//! with support for `--dry-run` to preview changes without making them.

#![allow(dead_code)]

use crate::tool::Tool;
use camino::{Utf8Path, Utf8PathBuf};
use fs_err as fs;
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use std::fmt;

/// Execution mode for planning operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanMode {
    /// Preview mode - collect old content for diffs, don't write anything.
    DryRun,
    /// Execute mode - hash-compare files, write changes to disk.
    Execute,
}

impl PlanMode {
    pub fn is_dry_run(self) -> bool {
        matches!(self, PlanMode::DryRun)
    }
}

/// A single operation that can be planned and executed.
#[derive(Debug, Clone)]
pub enum Operation {
    /// Create a new file with the given content.
    CreateFile {
        path: Utf8PathBuf,
        content: String,
        description: String,
    },

    /// Update an existing file with new content.
    /// Note: old_content is only populated in dry-run mode for diffing.
    UpdateFile {
        path: Utf8PathBuf,
        old_content: Option<String>,
        new_content: String,
        description: String,
    },

    /// Delete a file.
    DeleteFile {
        path: Utf8PathBuf,
        description: String,
    },

    /// Create a directory (and parents if needed).
    CreateDir {
        path: Utf8PathBuf,
        description: String,
    },

    /// Run a shell command.
    RunCommand {
        command: String,
        args: Vec<String>,
        working_dir: Option<Utf8PathBuf>,
        description: String,
    },

    /// Copy a file from source to destination.
    CopyFile {
        src: Utf8PathBuf,
        dest: Utf8PathBuf,
        description: String,
    },

    /// Git clone a repository.
    GitClone {
        url: String,
        dest: Utf8PathBuf,
        commit: Option<String>,
        description: String,
    },

    /// Compute grammar hash and write to .arborium-hash file.
    ComputeGrammarHash {
        crate_path: Utf8PathBuf,
        description: String,
    },
}

impl Operation {
    /// Returns a human-readable description of this operation.
    pub fn description(&self) -> &str {
        match self {
            Operation::CreateFile { description, .. } => description,
            Operation::UpdateFile { description, .. } => description,
            Operation::DeleteFile { description, .. } => description,
            Operation::CreateDir { description, .. } => description,
            Operation::RunCommand { description, .. } => description,
            Operation::CopyFile { description, .. } => description,
            Operation::GitClone { description, .. } => description,
            Operation::ComputeGrammarHash { description, .. } => description,
        }
    }

    /// Returns the primary path affected by this operation, if any.
    pub fn path(&self) -> Option<&Utf8Path> {
        match self {
            Operation::CreateFile { path, .. } => Some(path),
            Operation::UpdateFile { path, .. } => Some(path),
            Operation::DeleteFile { path, .. } => Some(path),
            Operation::CreateDir { path, .. } => Some(path),
            Operation::RunCommand { .. } => None,
            Operation::CopyFile { dest, .. } => Some(dest),
            Operation::GitClone { dest, .. } => Some(dest),
            Operation::ComputeGrammarHash { .. } => None,
        }
    }

    /// Returns a short verb describing the operation type.
    pub fn verb(&self) -> &'static str {
        match self {
            Operation::CreateFile { .. } => "create",
            Operation::UpdateFile { .. } => "update",
            Operation::DeleteFile { .. } => "delete",
            Operation::CreateDir { .. } => "mkdir",
            Operation::RunCommand { .. } => "run",
            Operation::CopyFile { .. } => "copy",
            Operation::GitClone { .. } => "clone",
            Operation::ComputeGrammarHash { .. } => "hash",
        }
    }

    /// Execute this operation.
    pub fn execute(&self) -> Result<(), ExecuteError> {
        match self {
            Operation::CreateFile { path, content, .. } => {
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(path, content)?;
                Ok(())
            }
            Operation::UpdateFile {
                path, new_content, ..
            } => {
                fs::write(path, new_content)?;
                Ok(())
            }
            Operation::DeleteFile { path, .. } => {
                fs::remove_file(path)?;
                Ok(())
            }
            Operation::CreateDir { path, .. } => {
                fs::create_dir_all(path)?;
                Ok(())
            }
            Operation::RunCommand {
                command,
                args,
                working_dir,
                ..
            } => {
                let mut cmd = std::process::Command::new(command);
                cmd.args(args);
                if let Some(dir) = working_dir {
                    cmd.current_dir(dir);
                }
                let status = cmd.status()?;
                if !status.success() {
                    return Err(ExecuteError::CommandFailed {
                        command: command.clone(),
                        status,
                    });
                }
                Ok(())
            }
            Operation::CopyFile { src, dest, .. } => {
                if let Some(parent) = dest.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::copy(src, dest)?;
                Ok(())
            }
            Operation::GitClone {
                url, dest, commit, ..
            } => {
                let git = Tool::Git.find()?;
                // Clone the repo
                let status = git
                    .command()
                    .args(["clone", "--depth", "1", url, dest.as_str()])
                    .status()?;
                if !status.success() {
                    return Err(ExecuteError::CommandFailed {
                        command: "git clone".to_string(),
                        status,
                    });
                }
                // Checkout specific commit if specified
                if let Some(commit) = commit {
                    let status = git
                        .command()
                        .args(["checkout", commit])
                        .current_dir(dest)
                        .status()?;
                    if !status.success() {
                        return Err(ExecuteError::CommandFailed {
                            command: "git checkout".to_string(),
                            status,
                        });
                    }
                }
                Ok(())
            }
            Operation::ComputeGrammarHash { crate_path, .. } => {
                // Compute hash from entire crate directory
                let hash = crate::publish::compute_grammar_hash(crate_path).map_err(|e| {
                    ExecuteError::Other(format!("Failed to compute grammar hash: {}", e))
                })?;

                // Write to .arborium-hash file
                let hash_file = crate_path.join(".arborium-hash");
                if let Some(parent) = hash_file.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(&hash_file, hash)?;
                Ok(())
            }
        }
    }
}

impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operation::CreateFile { path, .. } => {
                write!(f, "create {}", path)
            }
            Operation::UpdateFile { path, .. } => {
                write!(f, "update {}", path)
            }
            Operation::DeleteFile { path, .. } => {
                write!(f, "delete {}", path)
            }
            Operation::CreateDir { path, .. } => {
                write!(f, "mkdir  {}", path)
            }
            Operation::RunCommand { command, args, .. } => {
                write!(f, "run    {} {}", command, args.join(" "))
            }
            Operation::CopyFile { src, dest, .. } => {
                write!(f, "copy   {} -> {}", src, dest)
            }
            Operation::GitClone { url, dest, .. } => {
                write!(f, "clone  {} -> {}", url, dest)
            }
            Operation::ComputeGrammarHash { crate_path, .. } => {
                write!(f, "hash   {}", crate_path.join(".arborium-hash"))
            }
        }
    }
}

/// Error during operation execution.
#[derive(Debug)]
pub enum ExecuteError {
    Io(std::io::Error),
    CommandFailed {
        command: String,
        status: std::process::ExitStatus,
    },
    ToolNotFound(crate::tool::ToolNotFound),
    Other(String),
}

impl From<std::io::Error> for ExecuteError {
    fn from(e: std::io::Error) -> Self {
        ExecuteError::Io(e)
    }
}

impl From<crate::tool::ToolNotFound> for ExecuteError {
    fn from(e: crate::tool::ToolNotFound) -> Self {
        ExecuteError::ToolNotFound(e)
    }
}

impl fmt::Display for ExecuteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecuteError::Io(e) => write!(f, "IO error: {}", e),
            ExecuteError::CommandFailed { command, status } => {
                write!(f, "Command '{}' failed with {}", command, status)
            }
            ExecuteError::ToolNotFound(e) => write!(f, "{}", e),
            ExecuteError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for ExecuteError {}

/// Display a simple line-by-line diff between old and new content.
fn display_diff(old: &str, new: &str) {
    let old_lines: Vec<&str> = old.lines().collect();
    let new_lines: Vec<&str> = new.lines().collect();

    // Simple diff: find first and last differing lines
    let mut first_diff = None;
    let mut last_diff = None;

    let max_len = old_lines.len().max(new_lines.len());
    for i in 0..max_len {
        let old_line = old_lines.get(i);
        let new_line = new_lines.get(i);
        if old_line != new_line {
            if first_diff.is_none() {
                first_diff = Some(i);
            }
            last_diff = Some(i);
        }
    }

    let Some(first) = first_diff else {
        println!("    {}", "(no changes)".dimmed());
        return;
    };
    let last = last_diff.unwrap_or(first);

    // Show context around changes
    let context = 2;
    let start = first.saturating_sub(context);
    let end = (last + context + 1).min(max_len);

    if start > 0 {
        println!("    {}", format!("@@ line {} @@", start + 1).dimmed());
    }

    for i in start..end {
        let old_line = old_lines.get(i);
        let new_line = new_lines.get(i);

        match (old_line, new_line) {
            (Some(o), Some(n)) if o == n => {
                println!("    {} {}", " ".dimmed(), o.dimmed());
            }
            (Some(o), Some(n)) => {
                println!("    {} {}", "-".red(), o.red());
                println!("    {} {}", "+".green(), n.green());
            }
            (Some(o), None) => {
                println!("    {} {}", "-".red(), o.red());
            }
            (None, Some(n)) => {
                println!("    {} {}", "+".green(), n.green());
            }
            (None, None) => {}
        }
    }

    if end < max_len {
        println!(
            "    {}",
            format!("... {} more lines", max_len - end).dimmed()
        );
    }
}

/// A plan consisting of multiple operations.
#[derive(Debug, Default, Clone)]
pub struct Plan {
    operations: Vec<Operation>,
    pub crate_name: Option<String>,
}

impl Plan {
    /// Create a new empty plan.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new plan for a specific crate.
    pub fn for_crate(name: impl Into<String>) -> Self {
        Self {
            operations: Vec::new(),
            crate_name: Some(name.into()),
        }
    }

    /// Add an operation to the plan.
    pub fn add(&mut self, op: Operation) {
        self.operations.push(op);
    }

    /// Returns true if the plan has no operations.
    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }

    /// Returns the number of operations in the plan.
    pub fn len(&self) -> usize {
        self.operations.len()
    }

    /// Iterate over operations.
    pub fn operations(&self) -> impl Iterator<Item = &Operation> {
        self.operations.iter()
    }

    /// Display the plan to the user.
    pub fn display(&self) {
        if let Some(ref name) = self.crate_name {
            println!("{} {}", "●".cyan(), name.bold());
        }

        if self.operations.is_empty() {
            println!("  {}", "(no changes)".dimmed());
            return;
        }

        for op in &self.operations {
            match op {
                Operation::CreateFile {
                    path,
                    content,
                    description,
                } => {
                    println!("  {} {}", "create".green(), description);
                    println!("    {} {}", "->".dimmed(), path);
                    // Just show line count, no content preview (too verbose for large files)
                    let total_lines = content.lines().count();
                    println!("    {} ({} lines)", "+".green(), total_lines);
                }
                Operation::UpdateFile {
                    path,
                    old_content,
                    new_content,
                    description,
                } => {
                    println!("  {} {}", "update".yellow(), description);
                    println!("    {} {}", "->".dimmed(), path);
                    if let Some(old) = old_content {
                        display_diff(old, new_content);
                    } else {
                        println!("    {}", "(content changed)".dimmed());
                    }
                }
                Operation::DeleteFile { path, description } => {
                    println!("  {} {}", "delete".red(), description);
                    println!("    {} {}", "->".dimmed(), path);
                }
                Operation::CreateDir { path, description } => {
                    println!("  {} {}", "mkdir".blue(), description);
                    println!("    {} {}", "->".dimmed(), path);
                }
                _ => {
                    println!("  {} {}", op.verb(), op.description());
                    if let Some(path) = op.path() {
                        println!("    {} {}", "->".dimmed(), path);
                    }
                }
            }
        }
    }

    /// Execute all operations in the plan.
    pub fn execute(&self) -> Result<(), ExecuteError> {
        for op in &self.operations {
            op.execute()?;
        }
        Ok(())
    }
}

/// A collection of plans, typically one per crate.
#[derive(Debug, Default, Clone)]
pub struct PlanSet {
    plans: Vec<Plan>,
}

impl PlanSet {
    /// Create a new empty plan set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a plan to the set.
    pub fn add(&mut self, plan: Plan) {
        if !plan.is_empty() {
            self.plans.push(plan);
        }
    }

    /// Returns true if there are no plans with operations.
    pub fn is_empty(&self) -> bool {
        self.plans.is_empty()
    }

    /// Total number of operations across all plans.
    pub fn total_operations(&self) -> usize {
        self.plans.iter().map(|p| p.len()).sum()
    }

    /// Display all plans.
    pub fn display(&self, dry_run: bool) {
        if dry_run {
            println!("Dry run - showing what would be done:\n");
        }

        if self.is_empty() {
            println!("Nothing to do.");
            return;
        }

        for plan in &self.plans {
            plan.display();
            println!();
        }

        println!(
            "Total: {} operation(s) across {} crate(s)",
            self.total_operations(),
            self.plans.len()
        );

        if dry_run {
            println!("\nRun without --dry-run to apply changes.");
        }
    }

    /// Execute all plans.
    pub fn execute(&self) -> Result<(), ExecuteError> {
        use std::time::Instant;
        let start = Instant::now();

        for plan in &self.plans {
            if let Some(ref name) = plan.crate_name {
                println!("Processing {}...", name);
            }
            plan.execute()?;
        }

        let elapsed = start.elapsed();
        println!(
            "\n{} {} operation(s) completed in {:.2}s",
            "●".green(),
            self.total_operations(),
            elapsed.as_secs_f64()
        );
        Ok(())
    }

    /// Display and optionally execute the plans.
    pub fn run(&self, dry_run: bool) -> Result<(), ExecuteError> {
        self.run_with_options(dry_run, false)
    }

    /// Display and optionally execute the plans, with quiet mode support.
    pub fn run_with_options(&self, dry_run: bool, quiet: bool) -> Result<(), ExecuteError> {
        if !quiet {
            self.display(dry_run);
        }
        // In quiet mode, skip the summary - caller can print their own

        if !dry_run && !self.is_empty() {
            if !quiet {
                println!();
            }
            self.execute_quiet(quiet)?;
        }
        Ok(())
    }

    /// Execute all plans, optionally in quiet mode.
    fn execute_quiet(&self, quiet: bool) -> Result<(), ExecuteError> {
        use std::time::Instant;
        let start = Instant::now();

        let pb = if quiet {
            let pb = ProgressBar::new(self.plans.len() as u64);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} Writing {msg}")
                    .unwrap()
                    .progress_chars("━━╸"),
            );
            Some(pb)
        } else {
            None
        };

        for plan in &self.plans {
            if let Some(ref pb) = pb {
                if let Some(ref name) = plan.crate_name {
                    pb.set_message(name.strip_prefix("arborium-").unwrap_or(name).to_string());
                }
            } else if let Some(ref name) = plan.crate_name {
                println!("Processing {}...", name);
            }
            plan.execute()?;
            if let Some(ref pb) = pb {
                pb.inc(1);
            }
        }

        if let Some(pb) = pb {
            pb.finish_and_clear();
        }

        if !quiet {
            let elapsed = start.elapsed();
            println!(
                "\n{} {} operation(s) completed in {:.2}s",
                "●".green(),
                self.total_operations(),
                elapsed.as_secs_f64()
            );
        }
        Ok(())
    }
}
