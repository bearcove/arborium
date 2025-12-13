//! xtask for arborium - development tasks
//!
//! Usage: `cargo xtask <command>`
//!
//! Commands:
//! - `lint` - Validate all grammars
//! - `gen \[name\]` - Regenerate crate files from arborium.kdl and build the static demo
//! - `serve` - Build and serve the WASM demo locally

mod cache;
mod ci;
mod deploy_website;
mod generate;
mod lint_new;

mod build;
mod plan;
mod publish;
mod serve;
mod tool;
mod types;
mod util;
mod version_store;

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
    /// Print version information
    Version,

    /// Generate theme CSS files for the npm package from TOML sources
    GenThemes,

    /// Generate plugins-manifest.ts for the npm package (used by prepublishOnly)
    GenManifest,

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

        /// Version to use for generated Cargo.toml files
        #[facet(args::named)]
        version: String,

        /// Continue processing all crates even if some fail (default: stop on first failure)
        #[facet(args::named, default)]
        no_fail_fast: bool,

        /// Number of parallel jobs for tree-sitter generation (default: 16)
        #[facet(args::named, args::short = 'j', default)]
        jobs: Option<usize>,

        /// Suppress verbose plan output (only show summary)
        #[facet(args::named, args::short = 'q', default)]
        quiet: bool,
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

    /// Build WASM component plugins and demo assets
    Build {
        /// Specific grammars to build (build all if omitted)
        #[facet(args::positional, default)]
        grammars: Vec<String>,

        /// Build only grammars in a specific group (e.g., "birch", "acorn")
        #[facet(args::named, default)]
        group: Option<String>,

        /// Output directory for built plugins
        #[facet(args::named, args::short = 'o', default)]
        output: Option<String>,

        /// Number of parallel jobs (default: 16)
        #[facet(args::named, args::short = 'j', default)]
        jobs: Option<usize>,

        /// Dev mode: use local plugin paths in demo instead of CDN
        #[facet(args::named, default)]
        dev: bool,

        /// Continue building other plugins even if some fail
        #[facet(args::named, default)]
        no_fail_fast: bool,
    },

    /// Clean plugin build artifacts (standard layout)
    Clean,

    /// Generate CI workflow files
    Ci {
        #[facet(args::subcommand)]
        action: CiAction,
    },

    /// Publish to crates.io and npm
    Publish {
        #[facet(args::subcommand)]
        action: PublishAction,
    },

    /// Deploy website to GitHub Pages
    DeployWebsite {
        /// Version to use for CDN URLs (e.g., "0.2.0")
        #[facet(args::named)]
        version: String,

        /// Dry run - show what would be deployed without actually deploying
        #[facet(args::named, default)]
        dry_run: bool,
    },
}

/// CI workflow subcommands
#[derive(Debug, Facet)]
#[repr(u8)]
#[allow(dead_code)]
enum CiAction {
    /// Generate CI workflow files from Rust code
    Generate {
        /// Check if files are up to date instead of generating
        #[facet(args::named, default)]
        check: bool,
    },
}

/// Publish subcommands
#[derive(Debug, Facet)]
#[repr(u8)]
#[allow(dead_code)]
enum PublishAction {
    /// Publish crates to crates.io
    ///
    /// Use --group to publish specific groups:
    /// - pre: shared crates that grammars depend on
    /// - post: umbrella crates that depend on grammars
    /// - `<group-name>`: a specific language group (e.g., cedar, fern, birch)
    ///
    /// Without --group, publishes everything in order: pre, then all groups, then post.
    Crates {
        /// Dry run - don't actually publish
        #[facet(args::named, default)]
        dry_run: bool,

        /// Specific group to publish (pre, post, or a group name like cedar)
        #[facet(args::named, default)]
        group: Option<String>,

        /// Show full output from cargo publish
        #[facet(args::named, default)]
        verbose: bool,
    },

    /// Publish packages to npm
    ///
    /// Use --group to publish a specific language group (e.g., cedar, fern, birch).
    /// Without --group, publishes all npm packages.
    Npm {
        /// Dry run - don't actually publish
        #[facet(args::named, default)]
        dry_run: bool,

        /// Specific group to publish (e.g., cedar, fern, birch)
        #[facet(args::named, default)]
        group: Option<String>,

        /// Directory containing npm packages to publish (e.g., dist/plugins)
        #[facet(args::named, args::short = 'o', default)]
        output: Option<String>,
    },

    /// Publish everything (crates.io + npm)
    All {
        /// Dry run - don't actually publish
        #[facet(args::named, default)]
        dry_run: bool,

        /// Show full output from cargo publish
        #[facet(args::named, default)]
        verbose: bool,
    },
}

fn main() {
    // Install Miette's graphical error handler for nice CLI diagnostics
    miette::set_hook(Box::new(|_| {
        Box::new(miette::MietteHandlerOpts::new().build())
    }))
    .ok();

    // Initialize tracing subscriber for structured logging
    tracing_subscriber::fmt::init();

    let args: Args = facet_args::from_std_args().unwrap_or_else(|e| {
        eprintln!("{:?}", miette::Report::new(e));
        std::process::exit(1);
    });

    // Handle version early - doesn't need repo root
    if matches!(args.command, Command::Version) {
        println!("arborium-xtask {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    let crates_dir = util::find_repo_root()
        .expect("Could not find repo root")
        .join("crates");
    let crates_dir = camino::Utf8PathBuf::from_path_buf(crates_dir).expect("non-UTF8 path");

    match args.command {
        Command::Version => unreachable!(),
        Command::GenThemes => {
            if let Err(e) = serve::generate_npm_theme_css(&crates_dir) {
                eprintln!("{:?}", e);
                std::process::exit(1);
            }
        }
        Command::GenManifest => {
            let repo_root = util::find_repo_root().expect("Could not find repo root");
            let repo_root = camino::Utf8PathBuf::from_path_buf(repo_root).expect("non-UTF8 path");

            if let Err(e) = build::generate_plugins_manifest(&repo_root, &crates_dir) {
                eprintln!("{:?}", e);
                std::process::exit(1);
            }
        }
        Command::Lint { strict } => {
            let options = lint_new::LintOptions { strict };
            if let Err(e) = lint_new::run_lints(&crates_dir, options) {
                eprintln!("{:?}", e);
                std::process::exit(1);
            }
        }
        Command::Gen {
            name,
            dry_run,
            version,
            no_fail_fast,
            jobs,
            quiet,
        } => {
            // Check for required tools before starting
            if !tool::check_tools_or_report(tool::GEN_TOOLS) {
                std::process::exit(1);
            }

            let mode = if dry_run {
                plan::PlanMode::DryRun
            } else {
                plan::PlanMode::Execute
            };

            // Use provided version
            let version = version.as_str();

            let options = generate::GenerateOptions {
                name: name.as_deref(),
                mode,
                version,
                no_fail_fast,
                jobs: jobs.unwrap_or(16),
            };

            // Plan and execute generation
            let result = generate::plan_generate(&crates_dir, options);
            match result {
                Ok(plans) => {
                    // Quiet by default for normal runs (progress bar + summary is enough)
                    // Dry run shows full details so users can review what would change
                    let effective_quiet = !dry_run || quiet;
                    if let Err(e) = plans.run_with_options(dry_run, effective_quiet) {
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
                use std::time::Instant;
                let lint_start = Instant::now();

                let options = lint_new::LintOptions { strict: true };
                if let Err(e) = lint_new::run_lints(&crates_dir, options) {
                    eprintln!("{:?}", e);
                    std::process::exit(1);
                }

                let lint_elapsed = lint_start.elapsed();

                // Count total crates
                let registry = crate::types::CrateRegistry::load(&crates_dir)
                    .expect("Failed to load registry");
                println!(
                    "  {} Generated {} Rust crates ({:.2}s)",
                    "✓".green(),
                    registry.crates.len(),
                    lint_elapsed.as_secs_f64()
                );

                // Print next steps hint
                println!();
                println!("{}", "Next steps:".bold());
                println!(
                    "  {} {} to build WASM plugins",
                    "→".blue(),
                    "cargo xtask build".cyan()
                );
            }
        }
        Command::Serve { address, port, dev } => {
            // Check for required tools before starting
            if !tool::check_tools_or_report(tool::SERVE_TOOLS) {
                std::process::exit(1);
            }

            let addr = address.as_deref().unwrap_or("127.0.0.1");
            serve::serve(&crates_dir, addr, port, dev);
        }
        Command::Build {
            grammars,
            group,
            output,
            jobs,
            dev,
            no_fail_fast,
        } => {
            let repo_root = util::find_repo_root().expect("Could not find repo root");
            let repo_root = camino::Utf8PathBuf::from_path_buf(repo_root).expect("non-UTF8 path");

            // Check for required tools
            if !tool::check_tools_or_report(tool::PLUGIN_TOOLS) {
                std::process::exit(1);
            }

            // Build the host component first
            if let Err(e) = build::build_host(&repo_root) {
                eprintln!("{:?}", e);
                std::process::exit(1);
            }

            // Build plugins
            let options = build::BuildOptions {
                grammars,
                group,
                output_dir: output.map(camino::Utf8PathBuf::from),
                jobs: jobs.unwrap_or(16),
                no_fail_fast,
            };
            if let Err(e) = build::build_plugins(&repo_root, &options) {
                eprintln!("{:?}", e);
                std::process::exit(1);
            }

            // Generate demo assets
            if let Err(e) = build::build_demo(&repo_root, &crates_dir, dev) {
                eprintln!("{:?}", e);
                std::process::exit(1);
            }
        }
        Command::Clean => {
            let repo_root = util::find_repo_root().expect("Could not find repo root");
            let repo_root = camino::Utf8PathBuf::from_path_buf(repo_root).expect("non-UTF8 path");
            if let Err(e) = build::clean_plugins(&repo_root, "langs") {
                eprintln!("{:?}", e);
                std::process::exit(1);
            }
        }
        Command::Ci { action } => {
            let repo_root = util::find_repo_root().expect("Could not find repo root");
            let repo_root = camino::Utf8PathBuf::from_path_buf(repo_root).expect("non-UTF8 path");

            match action {
                CiAction::Generate { check } => {
                    if let Err(e) = ci::generate(&repo_root, check) {
                        eprintln!("{:?}", e);
                        std::process::exit(1);
                    }
                }
            }
        }

        Command::Publish { action } => {
            let repo_root = util::find_repo_root().expect("Could not find repo root");
            let repo_root = camino::Utf8PathBuf::from_path_buf(repo_root).expect("non-UTF8 path");

            match action {
                PublishAction::Crates {
                    dry_run,
                    group,
                    verbose,
                } => {
                    if let Err(e) =
                        publish::publish_crates(&repo_root, group.as_deref(), dry_run, verbose)
                    {
                        eprintln!("{:?}", e);
                        std::process::exit(1);
                    }
                }
                PublishAction::Npm {
                    dry_run,
                    group,
                    output,
                } => {
                    let packages_dir = output
                        .map(|o| repo_root.join(o))
                        .unwrap_or_else(|| repo_root.join("langs"));
                    if let Err(e) =
                        publish::publish_npm(&repo_root, &packages_dir, group.as_deref(), dry_run)
                    {
                        eprintln!("{:?}", e);
                        std::process::exit(1);
                    }
                }
                PublishAction::All { dry_run, verbose } => {
                    let output_dir = repo_root.join("langs");
                    if let Err(e) = publish::publish_all(&repo_root, &output_dir, dry_run, verbose)
                    {
                        eprintln!("{:?}", e);
                        std::process::exit(1);
                    }
                }
            }
        }

        Command::DeployWebsite { version, dry_run } => {
            let repo_root = util::find_repo_root().expect("Could not find repo root");
            let repo_root = camino::Utf8PathBuf::from_path_buf(repo_root).expect("non-UTF8 path");

            if let Err(e) = deploy_website::deploy_website(&repo_root, &version, dry_run) {
                eprintln!("{:?}", e);
                std::process::exit(1);
            }
        }
    }
}
