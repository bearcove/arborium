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
mod lint;
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
    },

    /// Check for upstream updates (dry-run, shows what would change)
    Update {
        /// Optional grammar name to check (checks all if omitted)
        #[facet(args::positional, default)]
        name: Option<String>,
    },

    /// Regenerate parser sources, crate code, and run lints
    Generate {
        /// Optional grammar name to regenerate (regenerates all if omitted)
        #[facet(args::positional, default)]
        name: Option<String>,
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

    match args.command {
        Command::Grammars { action } => match action {
            GrammarsAction::Vendor { url } => {
                println!("Vendoring grammar from: {url}");
                todo!("implement grammars vendor")
            }
            GrammarsAction::Update { name } => {
                if let Some(name) = name {
                    println!("Checking updates for: {name}");
                } else {
                    println!("Checking updates for all grammars");
                }
                todo!("implement grammars update")
            }
            GrammarsAction::Generate { name } => {
                if let Some(name) = name {
                    println!("Generating: {name}");
                } else {
                    println!("Generating all grammars");
                }
                todo!("implement grammars generate")
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
            println!("Running lints...");
            lint::lint_info_toml();
            println!();
            lint::lint_highlights();
        }
    }
}
