//! xtask for arborium - development tasks
//!
//! Usage: cargo xtask <command>
//!
//! Commands:
//!   lint           Validate all grammars
//!   gen [name]     Regenerate crate files from arborium.kdl
//!   serve          Build and serve the WASM demo locally

mod generate;
mod lint_new;
mod plan;
mod serve;
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
#[allow(dead_code)] // variants used by facet_args derive
enum Command {
    /// Validate all grammar configurations
    Lint,

    /// Regenerate crate files (Cargo.toml, build.rs, lib.rs, grammar-src/) from arborium.kdl
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
        Command::Lint => {
            if let Err(e) = lint_new::run_lints(&crates_dir) {
                eprintln!("{:?}", e);
                std::process::exit(1);
            }
        }
        Command::Gen { name, dry_run } => {
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
        Command::Serve { address, port, dev } => {
            let addr = address.as_deref().unwrap_or("127.0.0.1");
            serve::serve(&crates_dir, addr, port, dev);
        }
    }
}
