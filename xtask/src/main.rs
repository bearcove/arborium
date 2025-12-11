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

        /// Skip jco transpile step
        #[facet(args::named, default)]
        no_transpile: bool,

        /// Dev mode: use local plugin paths in demo instead of CDN
        #[facet(args::named, default)]
        dev: bool,
    },

    /// End-to-end sanity check for generation & publishing
    ///
    /// This:
    /// - generates grammars with the given version,
    /// - builds WASM plugins,
    /// - runs a dry-run cargo publish for crates,
    /// - and runs a dry-run npm publish for plugins.
    ///
    /// By default, processes ALL grammars. Use --package to test a single one.
    E2e {
        /// Version to use for this e2e run (default: "0.0.0-test")
        #[facet(args::named, default)]
        version: Option<String>,

        /// Specific package to test (default: all packages)
        #[facet(args::named, default)]
        package: Option<String>,
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
                    if let Err(e) = plans.run_with_options(dry_run, quiet) {
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

            // Print next steps hint
            if !dry_run {
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
            no_transpile,
            dev,
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
                transpile: !no_transpile,
                jobs: jobs.unwrap_or(16),
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
        Command::E2e { version, package } => {
            let version = version.unwrap_or_else(|| "0.0.0-test".to_string());

            let repo_root = util::find_repo_root().expect("Could not find repo root");
            let repo_root = camino::Utf8PathBuf::from_path_buf(repo_root).expect("non-UTF8 path");
            let crates_dir = repo_root.join("crates");

            // Load registry to get list of all grammars
            let registry = crate::types::CrateRegistry::load(&crates_dir)
                .expect("failed to load crate registry");

            // Determine which grammars to process (only actual grammars with generate-component)
            let grammar_ids: Vec<String> = if let Some(ref pkg) = package {
                // Single package specified - validate it exists and is a grammar
                let found = registry
                    .all_grammars()
                    .any(|(_, _, g)| g.id() == pkg && g.generate_component());
                if !found {
                    eprintln!("Unknown grammar: {}", pkg);
                    eprintln!("Available grammars:");
                    for (_, _, g) in registry.all_grammars() {
                        if g.generate_component() {
                            eprintln!("  {}", g.id());
                        }
                    }
                    std::process::exit(1);
                }
                vec![pkg.clone()]
            } else {
                // All grammars with generate-component enabled
                registry
                    .all_grammars()
                    .filter(|(_, _, g)| g.generate_component())
                    .map(|(_, _, g)| g.id().to_string())
                    .collect()
            };

            let total = grammar_ids.len();
            println!(
                "{} Running e2e check for {} grammar(s) with version {}",
                "●".cyan(),
                total,
                version.cyan()
            );

            // 1. Generate grammars with the provided version.
            if !tool::check_tools_or_report(tool::GEN_TOOLS) {
                std::process::exit(1);
            }

            let gen_options = generate::GenerateOptions {
                name: package.as_deref(), // None = all grammars
                mode: plan::PlanMode::Execute,
                version: &version,
                no_fail_fast: true,
                jobs: 4,
            };

            let plans = match generate::plan_generate(&crates_dir, gen_options) {
                Ok(plans) => plans,
                Err(e) => {
                    eprintln!("Generation failed: {}", e);
                    std::process::exit(1);
                }
            };

            if let Err(e) = plans.run(false) {
                eprintln!("Error while applying generation plan: {}", e);
                std::process::exit(1);
            }

            // 2. Build the WASM plugins.
            if !tool::check_tools_or_report(tool::PLUGIN_TOOLS) {
                std::process::exit(1);
            }

            let build_opts = build::BuildOptions {
                grammars: grammar_ids.clone(),
                group: None,
                output_dir: Some(camino::Utf8PathBuf::from("dist/plugins-e2e")),
                transpile: true,
                jobs: 4,
            };

            if let Err(e) = build::build_plugins(&repo_root, &build_opts) {
                eprintln!("Plugin build failed: {:?}", e);
                std::process::exit(1);
            }

            // Reload registry after generation to get updated paths
            let registry = crate::types::CrateRegistry::load(&crates_dir)
                .expect("failed to reload crate registry");

            // 3 & 4. Run dry-run publishes for each grammar
            let mut failures = Vec::new();

            for (idx, grammar_id) in grammar_ids.iter().enumerate() {
                println!(
                    "\n{} [{}/{}] Dry-run publish for '{}'",
                    "●".cyan(),
                    idx + 1,
                    total,
                    grammar_id
                );

                let (crate_state, _) = match crate::build::locate_grammar(&registry, grammar_id) {
                    Some(result) => result,
                    None => {
                        eprintln!("  {} grammar not found in registry", "✗".red());
                        failures.push(format!("{}: not found in registry", grammar_id));
                        continue;
                    }
                };

                let crate_dir = &crate_state.crate_path;

                // Run cargo publish --dry-run
                let output = std::process::Command::new("cargo")
                    .args(["publish", "--dry-run", "--allow-dirty"])
                    .current_dir(crate_dir.as_std_path())
                    .output()
                    .expect("failed to run cargo publish --dry-run");

                if !output.status.success() {
                    eprintln!("  {} cargo publish --dry-run failed:", "✗".red());
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    for line in stderr.lines() {
                        eprintln!("    {}", line);
                    }
                    failures.push(format!("{}: cargo publish failed", grammar_id));
                    continue;
                }

                // Run npm publish --dry-run
                let plugin_dir = crate_dir
                    .parent()
                    .expect("crate_path should have parent")
                    .join("npm");

                if !plugin_dir.join("package.json").exists() {
                    eprintln!("  {} npm package.json not found", "✗".red());
                    failures.push(format!("{}: npm package.json not found", grammar_id));
                    continue;
                }

                let output = std::process::Command::new("npm")
                    .args(["publish", "--dry-run"])
                    .current_dir(plugin_dir.as_std_path())
                    .output()
                    .expect("failed to run npm publish --dry-run");

                if !output.status.success() {
                    eprintln!("  {} npm publish --dry-run failed:", "✗".red());
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    for line in stderr.lines() {
                        eprintln!("    {}", line);
                    }
                    failures.push(format!("{}: npm publish failed", grammar_id));
                    continue;
                }

                println!("  {} dry-run publish succeeded", "✓".green());
            }

            // Summary
            println!();
            if failures.is_empty() {
                println!(
                    "{} e2e check completed successfully for {} grammar(s) at version {}",
                    "✓".green(),
                    total,
                    version.cyan()
                );
            } else {
                eprintln!(
                    "{} e2e check failed for {}/{} grammar(s):",
                    "✗".red(),
                    failures.len(),
                    total
                );
                for failure in &failures {
                    eprintln!("  - {}", failure);
                }
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
                PublishAction::Crates { dry_run, group } => {
                    if let Err(e) = publish::publish_crates(&repo_root, group.as_deref(), dry_run) {
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
                PublishAction::All { dry_run } => {
                    let output_dir = repo_root.join("langs");
                    if let Err(e) = publish::publish_all(&repo_root, &output_dir, dry_run) {
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
