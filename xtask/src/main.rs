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
mod theme_gen;

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
use std::process::Command as StdCommand;

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

    /// Print global cache key for CI (combines all grammar cache keys)
    CacheKey,

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
        #[facet(args::named, default)]
        version: Option<String>,

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

    /// Run grammar tests for a specific language crate
    GrammarTest {
        /// Grammar ID (e.g., "kdl")
        #[facet(args::positional)]
        grammar: String,

        /// Forward -- --nocapture to cargo test
        #[facet(args::named, default)]
        no_capture: bool,
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

    /// Show the topological levels for grammar crate publication order
    ///
    /// Useful for debugging and understanding the dependency structure.
    ShowLevels,
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

    let repo_root_path = util::find_repo_root().expect("Could not find repo root");
    let repo_root = camino::Utf8PathBuf::from_path_buf(repo_root_path).expect("non-UTF8 path");
    let crates_dir = repo_root.join("crates");

    match args.command {
        Command::Version => unreachable!(),
        Command::CacheKey => {
            match cache::compute_global_cache_key(&repo_root) {
                Ok(key) => println!("{}", key),
                Err(e) => {
                    eprintln!("Error computing cache key: {}", e);
                    std::process::exit(1);
                }
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
            let options = lint_new::LintOptions { strict, only: None };
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

            let resolved_version = resolve_workspace_version(version, &repo_root);
            run_generation_pipeline(
                &crates_dir,
                name.as_deref(),
                dry_run,
                quiet,
                resolved_version.as_str(),
                no_fail_fast,
                jobs.unwrap_or(16),
                !dry_run,
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
        Command::GrammarTest {
            grammar,
            no_capture,
        } => {
            if !tool::check_tools_or_report(tool::GEN_TOOLS) {
                std::process::exit(1);
            }

            let resolved_version = resolve_workspace_version(None, &repo_root);
            run_generation_pipeline(
                &crates_dir,
                Some(grammar.as_str()),
                false,
                true,
                resolved_version.as_str(),
                false,
                16,
                false,
            );

            let registry = crate::types::CrateRegistry::load(&crates_dir)
                .expect("Failed to load crate registry");
            let Some((crate_state, _)) = registry.find_grammar(&grammar) else {
                eprintln!("Unknown grammar `{}`", grammar);
                std::process::exit(1);
            };

            let manifest = crate_state.crate_path.join("Cargo.toml");
            println!(
                "{} Running grammar tests for {} ({})",
                "→".blue(),
                grammar,
                manifest.as_str()
            );

            let mut cmd = StdCommand::new("cargo");
            cmd.arg("test")
                .arg("--manifest-path")
                .arg(manifest.as_str());
            if no_capture {
                cmd.arg("--").arg("--nocapture");
            }
            let status = cmd.status().expect("Failed to run cargo test");

            if !status.success() {
                std::process::exit(status.code().unwrap_or(1));
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
                PublishAction::ShowLevels => {
                    let langs_dir = repo_root.join("langs");
                    if let Err(e) = publish::show_levels(&repo_root, &langs_dir) {
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

fn resolve_workspace_version(provided: Option<String>, repo_root: &camino::Utf8Path) -> String {
    if let Some(version) = provided {
        version
    } else {
        // Returns DEV_VERSION (0.0.0) if version.json doesn't exist
        version_store::read_version(repo_root).unwrap_or_else(|err| {
            eprintln!("Failed to parse version.json: {err}");
            std::process::exit(1);
        })
    }
}

#[allow(clippy::too_many_arguments)]
fn run_generation_pipeline(
    crates_dir: &camino::Utf8Path,
    name: Option<&str>,
    dry_run: bool,
    quiet: bool,
    version: &str,
    no_fail_fast: bool,
    jobs: usize,
    show_next_steps: bool,
) {
    let options = generate::GenerateOptions {
        name,
        mode: if dry_run {
            plan::PlanMode::DryRun
        } else {
            plan::PlanMode::Execute
        },
        version,
        no_fail_fast,
        jobs,
    };

    let plans = match generate::plan_generate(crates_dir, options) {
        Ok(plans) => plans,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    let effective_quiet = !dry_run || quiet;
    if let Err(e) = plans.run_with_options(dry_run, effective_quiet) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    if dry_run {
        return;
    }

    use std::time::Instant;
    let lint_start = Instant::now();
    let lint_filter = name.map(|n| vec![n.to_string()]);
    let lint_options = lint_new::LintOptions {
        strict: true,
        only: lint_filter.clone(),
    };
    if let Err(e) = lint_new::run_lints(crates_dir, lint_options) {
        eprintln!("{:?}", e);
        std::process::exit(1);
    }
    let lint_elapsed = lint_start.elapsed();

    let registry = crate::types::CrateRegistry::load(crates_dir).expect("Failed to load registry");
    let crate_total = lint_filter
        .as_ref()
        .map(|targets| targets.len())
        .unwrap_or_else(|| registry.crates.len());
    println!(
        "  {} Generated {} Rust crate{} ({:.2}s)",
        "✓".green(),
        crate_total,
        if crate_total == 1 { "" } else { "s" },
        lint_elapsed.as_secs_f64()
    );

    if name.is_none() {
        if let Err(e) = theme_gen::generate_theme_code(crates_dir) {
            eprintln!("{:?}", e);
            std::process::exit(1);
        }

        if let Err(e) = serve::generate_npm_theme_css(crates_dir) {
            eprintln!("{:?}", e);
            std::process::exit(1);
        }
    }

    if show_next_steps {
        println!();
        println!("{}", "Next steps:".bold());
        println!(
            "  {} {} to build WASM plugins",
            "→".blue(),
            "cargo xtask build".cyan()
        );
    }
}
