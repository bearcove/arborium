//! Plan-execute pattern for xtask operations.
//!
//! This module provides a way to plan operations before executing them,
//! with support for `--dry-run` to preview changes without making them.

use camino::{Utf8Path, Utf8PathBuf};
use std::fmt;

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
    UpdateFile {
        path: Utf8PathBuf,
        old_content: String,
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
        }
    }

    /// Execute this operation.
    pub fn execute(&self) -> Result<(), ExecuteError> {
        match self {
            Operation::CreateFile { path, content, .. } => {
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(path, content)?;
                Ok(())
            }
            Operation::UpdateFile {
                path, new_content, ..
            } => {
                std::fs::write(path, new_content)?;
                Ok(())
            }
            Operation::DeleteFile { path, .. } => {
                std::fs::remove_file(path)?;
                Ok(())
            }
            Operation::CreateDir { path, .. } => {
                std::fs::create_dir_all(path)?;
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
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::copy(src, dest)?;
                Ok(())
            }
            Operation::GitClone {
                url, dest, commit, ..
            } => {
                // Clone the repo
                let status = std::process::Command::new("git")
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
                    let status = std::process::Command::new("git")
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
}

impl From<std::io::Error> for ExecuteError {
    fn from(e: std::io::Error) -> Self {
        ExecuteError::Io(e)
    }
}

impl fmt::Display for ExecuteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecuteError::Io(e) => write!(f, "IO error: {}", e),
            ExecuteError::CommandFailed { command, status } => {
                write!(f, "Command '{}' failed with {}", command, status)
            }
        }
    }
}

impl std::error::Error for ExecuteError {}

/// A plan consisting of multiple operations.
#[derive(Debug, Default)]
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
            println!("● {}", name);
        }

        if self.operations.is_empty() {
            println!("  (no changes)");
            return;
        }

        for op in &self.operations {
            println!("  {} {}", op.verb(), op.description());
            if let Some(path) = op.path() {
                println!("    -> {}", path);
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
#[derive(Debug, Default)]
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
        for plan in &self.plans {
            if let Some(ref name) = plan.crate_name {
                println!("Processing {}...", name);
            }
            plan.execute()?;
        }
        println!(
            "\nDone! {} operation(s) completed.",
            self.total_operations()
        );
        Ok(())
    }

    /// Display and optionally execute the plans.
    pub fn run(&self, dry_run: bool) -> Result<(), ExecuteError> {
        self.display(dry_run);
        if !dry_run && !self.is_empty() {
            println!();
            self.execute()?;
        }
        Ok(())
    }
}
