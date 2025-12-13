//! arborium-rustdoc CLI - Post-process rustdoc output with syntax highlighting.

use arborium_rustdoc::{ProcessOptions, Processor};
use color_eyre::eyre::{Result, eyre};
use facet::Facet;
use facet_args as args;
use owo_colors::OwoColorize;
use std::path::PathBuf;
use std::time::Instant;

/// Post-process rustdoc HTML output to add syntax highlighting for non-Rust code blocks.
///
/// This tool transforms rustdoc-generated documentation by adding tree-sitter based
/// syntax highlighting for code blocks in languages other than Rust.
#[derive(Debug, Facet)]
struct Args {
    /// Input directory containing rustdoc output (e.g., target/doc)
    #[facet(args::positional)]
    input: PathBuf,

    /// Output directory (defaults to modifying input in place)
    #[facet(args::positional, default)]
    output: Option<PathBuf>,

    /// Show verbose output
    #[facet(args::named, args::short = 'v', default)]
    verbose: bool,
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let args: Args = facet_args::from_std_args()?;

    // Validate input directory
    if !args.input.exists() {
        return Err(eyre!(
            "Input directory does not exist: {}",
            args.input.display()
        ));
    }

    if !args.input.is_dir() {
        return Err(eyre!(
            "Input path is not a directory: {}",
            args.input.display()
        ));
    }

    // Create processor
    let options = ProcessOptions {
        input_dir: args.input.clone(),
        output_dir: args.output.clone(),
        verbose: args.verbose,
    };

    let mut processor = Processor::new(options);

    // Print header
    eprintln!(
        "{} Processing rustdoc output: {}",
        "arborium-rustdoc".green().bold(),
        args.input.display()
    );

    if let Some(out) = &args.output {
        eprintln!("  Output: {}", out.display());
    } else {
        eprintln!("  {} Modifying in place", "Note:".yellow());
    }

    eprintln!();

    // Process
    let start = Instant::now();
    let stats = processor.process()?;
    let elapsed = start.elapsed();

    // Print results
    eprintln!("{}", "Results:".bold());
    eprintln!(
        "  {} HTML files processed",
        stats.files_processed.to_string().cyan()
    );
    eprintln!(
        "  {} code blocks highlighted",
        stats.blocks_highlighted.to_string().green()
    );
    eprintln!(
        "  {} code blocks skipped (Rust or unsupported)",
        stats.blocks_skipped.to_string().yellow()
    );

    if let Some(css_path) = stats.css_file_modified {
        eprintln!("  {} CSS patched: {}", "✓".green(), css_path.display());
    }

    if !stats.unsupported_languages.is_empty() {
        eprintln!(
            "\n  {} Unsupported languages: {}",
            "Note:".yellow(),
            stats.unsupported_languages.join(", ")
        );
    }

    eprintln!("\n  Completed in {:.2}s", elapsed.as_secs_f64());

    Ok(())
}
