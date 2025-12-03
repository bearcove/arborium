//! New linting system based on CrateRegistry.
//!
//! This module provides linting that uses the arborium.kdl files as the source
//! of truth, with Miette diagnostics for precise error reporting.

use camino::Utf8Path;
use miette::{Diagnostic, NamedSource, SourceSpan};
use owo_colors::OwoColorize;
use thiserror::Error;

use crate::types::{CrateRegistry, CrateState, MIN_SAMPLE_LINES, SampleFileState};

/// Options for running lints.
#[derive(Debug, Clone, Copy, Default)]
pub struct LintOptions {
    /// When true, missing generated files (parser.c) are errors.
    /// When false, they're warnings (useful before running `cargo xtask gen`).
    pub strict: bool,
}

/// Run all lints on the registry.
pub fn run_lints(crates_dir: &Utf8Path, options: LintOptions) -> miette::Result<()> {
    println!("{}", "Loading crate registry...".cyan().bold());
    if !options.strict {
        println!(
            "{}",
            "(non-strict mode: missing generated files are warnings)".dimmed()
        );
    }
    println!();

    let registry = CrateRegistry::load(crates_dir).map_err(|e| miette::miette!("{e}"))?;

    let mut errors = 0;
    let mut warnings = 0;

    // First pass: check for crates without arborium.kdl
    println!("{}", "Checking for missing arborium.kdl files...".cyan());
    for (name, state) in registry.iter() {
        if state.config.is_none() {
            println!(
                "  {} {} - {}",
                "!".yellow(),
                name.bold(),
                "missing arborium.kdl".yellow()
            );
            warnings += 1;
        }
    }
    println!();

    // Second pass: lint each configured crate
    println!("{}", "Linting configured crates...".cyan());
    for (name, state, config) in registry.configured_crates() {
        let crate_errors = lint_crate(name, state, config, options);

        if !crate_errors.is_empty() {
            println!("{} {}", "●".yellow(), name.bold());

            for diagnostic in &crate_errors {
                match diagnostic {
                    LintDiagnostic::Error(msg) => {
                        println!("  {} {}", "error:".red().bold(), msg);
                        errors += 1;
                    }
                    LintDiagnostic::Warning(msg) => {
                        println!("  {} {}", "warning:".yellow(), msg);
                        warnings += 1;
                    }
                    LintDiagnostic::Spanned {
                        source_name,
                        source,
                        span,
                        message,
                        is_error,
                    } => {
                        // For spanned diagnostics, print with Miette
                        let report = SpannedLint {
                            message: message.clone(),
                            src: NamedSource::new(source_name, source.clone()),
                            span: *span,
                        };
                        if *is_error {
                            println!("  {} {}", "error:".red().bold(), message);
                            errors += 1;
                        } else {
                            println!("  {} {}", "warning:".yellow(), message);
                            warnings += 1;
                        }
                        // Could print full miette report here if desired
                        let _ = report;
                    }
                }
            }
            println!();
        }
    }

    // Third pass: check for legacy files
    println!("{}", "Checking for legacy files...".cyan());
    for (name, state) in registry.iter() {
        if !state.files.legacy_files.is_empty() {
            println!("{} {}", "●".yellow(), name.bold());
            for legacy in &state.files.legacy_files {
                println!(
                    "  {} legacy file should be deleted: {}",
                    "warning:".yellow(),
                    legacy.file_name().unwrap_or("?")
                );
                warnings += 1;
            }
            println!();
        }
    }

    // Summary
    println!("{}", "─".repeat(60));
    println!();
    println!("Checked {} crate(s)", registry.crates.len());

    if errors > 0 {
        println!("{} {} error(s)", "✗".red(), errors);
    }
    if warnings > 0 {
        println!("{} {} warning(s)", "⚠".yellow(), warnings);
    }
    if errors == 0 && warnings == 0 {
        println!("{} All crates are valid!", "✓".green());
    }

    if errors > 0 {
        std::process::exit(1);
    }

    Ok(())
}

/// A lint diagnostic.
enum LintDiagnostic {
    Error(String),
    Warning(String),
    #[allow(dead_code)]
    Spanned {
        source_name: String,
        source: String,
        span: SourceSpan,
        message: String,
        is_error: bool,
    },
}

/// A Miette-compatible spanned lint error.
#[derive(Debug, Error, Diagnostic)]
#[error("{message}")]
struct SpannedLint {
    message: String,
    #[source_code]
    src: NamedSource<String>,
    #[label("here")]
    span: SourceSpan,
}

/// Lint a single crate and return diagnostics.
fn lint_crate(
    _name: &str,
    state: &CrateState,
    config: &crate::types::CrateConfig,
    options: LintOptions,
) -> Vec<LintDiagnostic> {
    let mut diagnostics = Vec::new();

    // Check that we have at least one grammar
    if config.grammars.is_empty() {
        diagnostics.push(LintDiagnostic::Error(
            "no grammars defined in arborium.kdl".to_string(),
        ));
        return diagnostics;
    }

    // Lint each grammar
    for grammar in &config.grammars {
        let gid = grammar.id();

        // Check required grammar/src files (generated by `cargo xtask gen`)
        // In non-strict mode, missing parser.c is a warning (gen hasn't run yet)
        if !state.files.grammar_src.parser_c.is_present() {
            if options.strict {
                diagnostics.push(LintDiagnostic::Error(format!(
                    "grammar '{gid}': missing grammar/src/parser.c",
                )));
            } else {
                diagnostics.push(LintDiagnostic::Warning(format!(
                    "grammar '{gid}': missing grammar/src/parser.c (run `cargo xtask gen` to generate)",
                )));
            }
        }

        // Check scanner if declared
        // scanner.c is in grammar/ (handwritten, not generated)
        if grammar.has_scanner() && !state.files.grammar_src.scanner_c.is_present() {
            diagnostics.push(LintDiagnostic::Error(format!(
                "grammar '{gid}': has-scanner is true but grammar/scanner.c is missing",
            )));
        }

        // Check for scanner file without has-scanner declaration
        if !grammar.has_scanner() && state.files.grammar_src.scanner_c.is_present() {
            diagnostics.push(LintDiagnostic::Warning(format!(
                "grammar '{gid}': grammar/scanner.c exists but has-scanner is not set",
            )));
        }

        // Check highlights.scm exists
        if !state.files.queries.highlights.is_present() {
            diagnostics.push(LintDiagnostic::Warning(format!(
                "grammar '{gid}': missing queries/highlights.scm",
            )));
        }

        // Skip user-facing metadata checks for internal grammars
        if grammar.is_internal() {
            continue;
        }

        // Check samples
        if grammar.samples.is_empty() {
            diagnostics.push(LintDiagnostic::Warning(format!(
                "grammar '{gid}': no samples defined",
            )));
        }

        // Validate tier
        if let Some(ref tier) = grammar.tier {
            let tier_val = *tier.value;
            if !(1..=5).contains(&tier_val) {
                diagnostics.push(LintDiagnostic::Error(format!(
                    "grammar '{gid}': tier must be between 1 and 5, got {tier_val}",
                )));
            }
        }

        // Check recommended metadata
        if grammar.inventor.is_none() {
            diagnostics.push(LintDiagnostic::Warning(format!(
                "grammar '{gid}': missing recommended field 'inventor'",
            )));
        }
        if grammar.year.is_none() {
            diagnostics.push(LintDiagnostic::Warning(format!(
                "grammar '{gid}': missing recommended field 'year'",
            )));
        }
        if grammar.description.is_none() {
            diagnostics.push(LintDiagnostic::Warning(format!(
                "grammar '{gid}': missing recommended field 'description'",
            )));
        }
        if grammar.link.is_none() {
            diagnostics.push(LintDiagnostic::Warning(format!(
                "grammar '{gid}': missing recommended field 'link'",
            )));
        }
    }

    // Check sample file states
    for sample in &state.files.samples {
        match &sample.state {
            SampleFileState::Missing => {
                diagnostics.push(LintDiagnostic::Error(format!(
                    "sample '{}' does not exist",
                    sample.path
                )));
            }
            SampleFileState::Empty => {
                diagnostics.push(LintDiagnostic::Error(format!(
                    "sample '{}' is empty",
                    sample.path
                )));
            }
            SampleFileState::HttpError => {
                diagnostics.push(LintDiagnostic::Error(format!(
                    "sample '{}' contains HTTP error (failed download?)",
                    sample.path
                )));
            }
            SampleFileState::TooShort { lines } => {
                diagnostics.push(LintDiagnostic::Warning(format!(
                    "sample '{}' has only {} lines (minimum {} recommended)",
                    sample.path, lines, MIN_SAMPLE_LINES
                )));
            }
            SampleFileState::Ok { .. } => {}
        }
    }

    diagnostics
}
