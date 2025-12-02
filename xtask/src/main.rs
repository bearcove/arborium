//! xtask for arborium - development tasks
//!
//! Usage: cargo xtask <command>
//!
//! Commands:
//!   grammars vendor <url>   Vendor a new grammar from a git URL
//!   grammars update [name]  Check for upstream updates (dry-run)
//!   grammars generate [name] Regenerate parser sources and crate code
//!   serve-demo              Build and serve the WASM demo locally

mod config;
mod generate;
mod lint;
mod lint_new;
mod plan;
mod types;
mod util;

use facet::Facet;
use facet_args as args;

/// Arborium development tasks
#[derive(Debug, Facet)]
struct Args {
    #[facet(args::subcommand)]
    command: Command,
}

/// Available commands
#[derive(Debug, Facet)]
#[repr(u8)]
enum Command {
    /// Manage tree-sitter grammars
    Grammars {
        #[facet(args::subcommand)]
        action: GrammarsAction,
    },

    /// Build and serve the WASM demo locally
    ServeDemo {
        /// Address to bind to
        #[facet(args::named, args::short = 'a')]
        address: Option<String>,

        /// Port to bind to
        #[facet(args::named, args::short = 'p')]
        port: Option<u16>,

        /// Fast dev build (skip optimizations)
        #[facet(args::named)]
        dev: bool,
    },

    /// Generate demo HTML and assets
    GenerateDemo,

    /// Run all lints
    Lint,
}

/// Grammar management subcommands
#[derive(Debug, Facet)]
#[repr(u8)]
enum GrammarsAction {
    /// Vendor a new grammar from a git URL (fails if grammar already exists)
    Vendor {
        /// Git repository URL for the tree-sitter grammar
        #[facet(args::positional)]
        url: String,

        /// Show what would be done without making changes
        #[facet(args::named, default)]
        dry_run: bool,
    },

    /// Check for upstream updates (dry-run, shows what would change)
    Update {
        /// Optional grammar name to check (checks all if omitted)
        #[facet(args::positional, default)]
        name: Option<String>,

        /// Show what would be done without making changes
        #[facet(args::named, default)]
        dry_run: bool,
    },

    /// Regenerate crate files (Cargo.toml, build.rs, lib.rs) from arborium.kdl
    Generate {
        /// Optional grammar name to regenerate (regenerates all if omitted)
        #[facet(args::positional, default)]
        name: Option<String>,

        /// Show what would be done without making changes
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

    let args: Args = facet_args::from_std_args().unwrap_or_else(|e| {
        eprintln!("{:?}", miette::Report::new(e));
        std::process::exit(1);
    });

    let crates_dir = util::find_repo_root()
        .expect("Could not find repo root")
        .join("crates");
    let crates_dir = camino::Utf8PathBuf::from_path_buf(crates_dir).expect("non-UTF8 path");

    match args.command {
        Command::Grammars { action } => match action {
            GrammarsAction::Vendor { url, dry_run } => {
                println!("Vendoring grammar from: {url}");
                if dry_run {
                    println!("(dry run)");
                }
                todo!("implement grammars vendor")
            }
            GrammarsAction::Update { name, dry_run } => {
                if let Some(ref name) = name {
                    println!("Checking updates for: {name}");
                } else {
                    println!("Checking updates for all grammars");
                }
                if dry_run {
                    println!("(dry run)");
                }
                todo!("implement grammars update")
            }
            GrammarsAction::Generate { name, dry_run } => {
                match generate::plan_generate(&crates_dir, name.as_deref()) {
                    Ok(plans) => {
                        if let Err(e) = plans.run(dry_run) {
                            eprintln!("Error: {}", e);
                            std::process::exit(1);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error planning generation: {}", e);
                        std::process::exit(1);
                    }
                }
            }
        },
        Command::ServeDemo { address, port, dev } => {
            let addr = address.as_deref().unwrap_or("127.0.0.1");
            println!("Serving demo at {addr}:{}", port.unwrap_or(8000));
            if dev {
                println!("(dev mode)");
            }
            todo!("implement serve-demo")
        }
        Command::GenerateDemo => {
            println!("Generating demo...");
            todo!("implement generate-demo")
        }
        Command::Lint => {
            if let Err(e) = lint_new::run_lints(&crates_dir) {
                eprintln!("{:?}", e);
                std::process::exit(1);
            }
        }
    }
}
