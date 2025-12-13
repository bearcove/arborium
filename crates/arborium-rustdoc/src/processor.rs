//! Main processor that transforms rustdoc output directories.

use crate::css::generate_rustdoc_theme_css;
use crate::html::{TransformError, TransformResult, transform_html};
use arborium::Highlighter;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Options for the processor.
#[derive(Debug, Clone)]
pub struct ProcessOptions {
    /// Input directory containing rustdoc output.
    pub input_dir: PathBuf,
    /// Output directory (if None, modifies in place).
    pub output_dir: Option<PathBuf>,
    /// Whether to show verbose output.
    pub verbose: bool,
}

/// Statistics from processing.
#[derive(Debug, Default)]
pub struct ProcessorStats {
    /// Number of HTML files processed.
    pub files_processed: usize,
    /// Number of code blocks highlighted.
    pub blocks_highlighted: usize,
    /// Number of code blocks skipped.
    pub blocks_skipped: usize,
    /// CSS file that was modified.
    pub css_file_modified: Option<PathBuf>,
    /// Languages that were not supported.
    pub unsupported_languages: Vec<String>,
}

/// Processor for rustdoc output.
pub struct Processor {
    options: ProcessOptions,
}

impl Processor {
    /// Create a new processor with the given options.
    pub fn new(options: ProcessOptions) -> Self {
        Self { options }
    }

    /// Process the rustdoc output directory.
    pub fn process(&mut self) -> Result<ProcessorStats, ProcessError> {
        let mut stats = ProcessorStats::default();

        // Determine the actual output directory
        let output_dir = self
            .options
            .output_dir
            .as_ref()
            .unwrap_or(&self.options.input_dir);

        // If output_dir is different from input_dir, copy everything first
        if let Some(ref out) = self.options.output_dir {
            if out != &self.options.input_dir {
                copy_dir_recursive(&self.options.input_dir, out)?;
            }
        }

        // Step 1: Find and patch the rustdoc CSS file
        if let Some(css_path) = self.find_and_patch_css(output_dir)? {
            stats.css_file_modified = Some(css_path);
        }

        // Step 2: Process all HTML files
        for entry in WalkDir::new(output_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "html"))
        {
            let path = entry.path();

            if self.options.verbose {
                eprintln!("Processing: {}", path.display());
            }

            match self.process_html_file(path) {
                Ok(result) => {
                    stats.files_processed += 1;
                    stats.blocks_highlighted += result.blocks_highlighted;
                    stats.blocks_skipped += result.blocks_skipped;

                    for lang in result.unsupported_languages {
                        if !stats.unsupported_languages.contains(&lang) {
                            stats.unsupported_languages.push(lang);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Failed to process {}: {}", path.display(), e);
                }
            }
        }

        Ok(stats)
    }

    /// Find the rustdoc CSS file and append arborium theme CSS.
    fn find_and_patch_css(&self, output_dir: &Path) -> Result<Option<PathBuf>, ProcessError> {
        let static_files = output_dir.join("static.files");

        if !static_files.exists() {
            // Not a rustdoc output directory, or CSS is embedded
            return Ok(None);
        }

        // Find rustdoc-*.css file
        let css_file = fs::read_dir(&static_files)?
            .filter_map(|e| e.ok())
            .find(|e| {
                e.file_name()
                    .to_str()
                    .is_some_and(|n| n.starts_with("rustdoc-") && n.ends_with(".css"))
            })
            .map(|e| e.path());

        let Some(css_path) = css_file else {
            return Ok(None);
        };

        // Read existing CSS
        let mut css_content = fs::read_to_string(&css_path)?;

        // Check if we've already patched it
        if css_content.contains("/* arborium syntax highlighting") {
            return Ok(Some(css_path));
        }

        // Generate and append arborium theme CSS
        let arborium_css = generate_rustdoc_theme_css();
        css_content.push_str(&arborium_css);

        // Write back
        fs::write(&css_path, css_content)?;

        Ok(Some(css_path))
    }

    /// Process a single HTML file.
    fn process_html_file(&self, path: &Path) -> Result<TransformResult, ProcessError> {
        let html = fs::read_to_string(path)?;

        // Create a fresh highlighter for each file (consumed by transform_html)
        let highlighter = Highlighter::new();
        let (transformed, result) = transform_html(&html, highlighter)?;

        // Only write if we actually changed something
        if result.blocks_highlighted > 0 {
            fs::write(path, transformed)?;
        }

        Ok(result)
    }
}

/// Copy a directory recursively, handling symlinks properly.
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), ProcessError> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        let file_type = entry.file_type()?;

        if file_type.is_symlink() {
            // Preserve symlinks by reading the target and creating a new symlink
            let target = fs::read_link(&src_path)?;
            symlink(&target, &dst_path)?;
        } else if file_type.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

/// Create a symlink (cross-platform).
#[cfg(unix)]
fn symlink(target: &Path, link: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(target, link)
}

#[cfg(windows)]
fn symlink(target: &Path, link: &Path) -> std::io::Result<()> {
    // On Windows, we need to know if the target is a directory
    // Since we're copying from an existing structure, check if target exists
    if target.is_dir() {
        std::os::windows::fs::symlink_dir(target, link)
    } else {
        std::os::windows::fs::symlink_file(target, link)
    }
}

/// Errors that can occur during processing.
#[derive(Debug)]
pub enum ProcessError {
    /// IO error.
    Io(std::io::Error),
    /// HTML transformation error.
    Transform(TransformError),
}

impl From<std::io::Error> for ProcessError {
    fn from(e: std::io::Error) -> Self {
        ProcessError::Io(e)
    }
}

impl From<TransformError> for ProcessError {
    fn from(e: TransformError) -> Self {
        ProcessError::Transform(e)
    }
}

impl std::fmt::Display for ProcessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessError::Io(e) => write!(f, "IO error: {}", e),
            ProcessError::Transform(e) => write!(f, "Transform error: {}", e),
        }
    }
}

impl std::error::Error for ProcessError {}
