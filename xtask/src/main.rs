//! xtask for arborium - development tasks
//!
//! Usage: `cargo xtask <command>`
//!
//! Commands:
//! - `doctor` - Check for required external tools
//! - `lint` - Validate all grammars
//! - `gen \[name\]` - Regenerate crate files from arborium.kdl
//! - `serve` - Build and serve the WASM demo locally

mod cache;
mod generate;
mod lint_new;
mod plan;
mod plugins;
mod serve;
mod tool;
mod types;
mod util;

use facet::Facet;
use facet_args as args;
use owo_colors::OwoColorize;

/// Arborium development tasks
#[derive(Debug, Facet)]
struct Args {
    #[facet(args::subcommand)]
    command: Command,
}

/// Available commands
#[derive(Debug, Facet)]
#[repr(u8)]
#[allow(dead_code)] // variants used by facet_args derive
enum Command {
    /// Check for required external tools
    Doctor,

    /// Validate all grammar configurations
    Lint {
        /// Strict mode: missing generated files (parser.c) are errors.
        /// Without this flag, they're warnings (useful before running gen).
        #[facet(args::named, default)]
        strict: bool,
    },

    /// Regenerate crate files (Cargo.toml, build.rs, lib.rs, grammar/src/) from arborium.kdl
    Gen {
        /// Optional grammar name to regenerate (regenerates all if omitted)
        #[facet(args::positional, default)]
        name: Option<String>,

        /// Show what would be done without making changes
        #[facet(args::named, default)]
        dry_run: bool,
    },

    /// Build and serve the WASM demo locally
    Serve {
        /// Address to bind to
        #[facet(args::named, args::short = 'a', default)]
        address: Option<String>,

        /// Port to bind to
        #[facet(args::named, args::short = 'p', default)]
        port: Option<u16>,

        /// Fast dev build (skip optimizations)
        #[facet(args::named, default)]
        dev: bool,
    },

    /// Build WASM component plugins
    Plugins {
        #[facet(args::subcommand)]
        action: PluginsAction,
    },
}

/// Plugin subcommands
#[derive(Debug, Facet)]
#[repr(u8)]
#[allow(dead_code)]
enum PluginsAction {
    /// Build grammar plugins as WASM components
    Build {
        /// Specific grammars to build (build all if omitted)
        #[facet(args::positional, default)]
        grammars: Vec<String>,

        /// Output directory for built plugins
        #[facet(args::named, args::short = 'o', default)]
        output: Option<String>,

        /// Skip jco transpile step
        #[facet(args::named, default)]
        no_transpile: bool,
    },

    /// Clean plugin build artifacts
    Clean {
        /// Output directory to clean
        #[facet(args::named, args::short = 'o', default)]
        output: Option<String>,
    },
}

fn main() {
    // Install Miette's graphical error handler for nice CLI diagnostics
    miette::set_hook(Box::new(|_| {
        Box::new(miette::MietteHandlerOpts::new().build())
    }))
    .ok();

    let args: Args = facet_args::from_std_args().unwrap_or_else(|e| {
        eprintln!("{:?}", miette::Report::new(e));
        std::process::exit(1);
    });

    let crates_dir = util::find_repo_root()
        .expect("Could not find repo root")
        .join("crates");
    let crates_dir = camino::Utf8PathBuf::from_path_buf(crates_dir).expect("non-UTF8 path");

    match args.command {
        Command::Doctor => {
            tool::print_tools_report();
        }
        Command::Lint { strict } => {
            let options = lint_new::LintOptions { strict };
            if let Err(e) = lint_new::run_lints(&crates_dir, options) {
                eprintln!("{:?}", e);
                std::process::exit(1);
            }
        }
        Command::Gen { name, dry_run } => {
            use std::time::Instant;
            let total_start = Instant::now();

            // Check for required tools before starting
            if !tool::check_tools_or_report(tool::GEN_TOOLS) {
                std::process::exit(1);
            }

            let mode = if dry_run {
                plan::PlanMode::DryRun
            } else {
                plan::PlanMode::Execute
            };

            // Plan and execute generation
            match generate::plan_generate(&crates_dir, name.as_deref(), mode) {
                Ok(plans) => {
                    if let Err(e) = plans.run(dry_run) {
                        eprintln!("Error: {}", e);
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("{}", e);
                    std::process::exit(1);
                }
            }

            // Run strict lint after generation (now parser.c should exist)
            if !dry_run {
                println!();
                println!(
                    "{}",
                    "Running post-generation lint (strict)...".cyan().bold()
                );
                let options = lint_new::LintOptions { strict: true };
                if let Err(e) = lint_new::run_lints(&crates_dir, options) {
                    eprintln!("{:?}", e);
                    std::process::exit(1);
                }
            }

            let total_elapsed = total_start.elapsed();
            println!(
                "\n{} Total time: {:.2}s",
                "●".green(),
                total_elapsed.as_secs_f64()
            );
        }
        Command::Serve { address, port, dev } => {
            // Check for required tools before starting
            if !tool::check_tools_or_report(tool::SERVE_TOOLS) {
                std::process::exit(1);
            }

            let addr = address.as_deref().unwrap_or("127.0.0.1");
            serve::serve(&crates_dir, addr, port, dev);
        }
        Command::Plugins { action } => {
            // Check for required tools before starting
            if !tool::check_tools_or_report(tool::PLUGIN_TOOLS) {
                std::process::exit(1);
            }

            let repo_root = util::find_repo_root().expect("Could not find repo root");
            let repo_root = camino::Utf8PathBuf::from_path_buf(repo_root).expect("non-UTF8 path");

            match action {
                PluginsAction::Build {
                    grammars,
                    output,
                    no_transpile,
                } => {
                    let options = plugins::BuildOptions {
                        grammars,
                        output_dir: output
                            .map(camino::Utf8PathBuf::from)
                            .unwrap_or_else(|| camino::Utf8PathBuf::from("dist/plugins")),
                        transpile: !no_transpile,
                        ..Default::default()
                    };

                    if let Err(e) = plugins::build_plugins(&repo_root, &options) {
                        eprintln!("{:?}", e);
                        std::process::exit(1);
                    }
                }
                PluginsAction::Clean { output } => {
                    let output_dir = output.as_deref().unwrap_or("dist/plugins");
                    if let Err(e) = plugins::clean_plugins(&repo_root, output_dir) {
                        eprintln!("{:?}", e);
                        std::process::exit(1);
                    }
                }
            }
        }
    }
}
