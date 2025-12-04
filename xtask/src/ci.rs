//! CI workflow generation for GitHub Actions.
//!
//! This module provides typed representations of GitHub Actions workflow files
//! and utilities to generate them from templates.

use facet::Facet;
use indexmap::IndexMap;

use crate::plugins::{PluginGroups, PluginTimings};

// =============================================================================
// GitHub Actions Workflow Schema
// =============================================================================

structstruck::strike! {
    /// A GitHub Actions workflow file.
    #[strikethrough[derive(Debug, Clone, Facet)]]
    #[facet(rename_all = "kebab-case")]
    pub struct Workflow {
        /// The name of the workflow displayed in the GitHub UI.
        pub name: String,

        /// The events that trigger the workflow.
        pub on: On,

        /// Environment variables available to all jobs.
        #[facet(default, skip_serializing_if = Option::is_none)]
        pub env: Option<IndexMap<String, String>>,

        /// The jobs that make up the workflow.
        pub jobs: IndexMap<String, Job>,
    }
}

structstruck::strike! {
    /// Events that trigger a workflow.
    #[strikethrough[derive(Debug, Clone, Facet)]]
    #[facet(rename_all = "snake_case")]
    pub struct On {
        /// Trigger on push events.
        #[facet(default, skip_serializing_if = Option::is_none)]
        pub push: Option<pub struct PushTrigger {
            /// Branches to trigger on.
            #[facet(default, skip_serializing_if = Option::is_none)]
            pub branches: Option<Vec<String>>,
        }>,

        /// Trigger on pull request events.
        #[facet(default, skip_serializing_if = Option::is_none)]
        pub pull_request: Option<pub struct PullRequestTrigger {
            /// Branches to trigger on.
            #[facet(default, skip_serializing_if = Option::is_none)]
            pub branches: Option<Vec<String>>,
        }>,

        /// Trigger on merge group events.
        #[facet(default, skip_serializing_if = Option::is_none)]
        pub merge_group: Option<pub struct MergeGroupTrigger {}>,
    }
}

structstruck::strike! {
    /// A job in a workflow.
    #[strikethrough[derive(Debug, Clone, Facet)]]
    #[facet(rename_all = "kebab-case")]
    pub struct Job {
        /// Display name for the job in the GitHub UI.
        #[facet(default, skip_serializing_if = Option::is_none)]
        pub name: Option<String>,

        /// The runner to use.
        pub runs_on: String,

        /// Container to run the job in.
        #[facet(default, skip_serializing_if = Option::is_none)]
        pub container: Option<String>,

        /// Jobs that must complete before this one.
        #[facet(default, skip_serializing_if = Option::is_none)]
        pub needs: Option<Vec<String>>,

        /// The steps to run.
        pub steps: Vec<Step>,
    }
}

structstruck::strike! {
    /// A step in a job.
    #[strikethrough[derive(Debug, Clone, Facet)]]
    #[facet(rename_all = "kebab-case")]
    pub struct Step {
        /// The name of the step.
        #[facet(default, skip_serializing_if = Option::is_none)]
        pub name: Option<String>,

        /// Use a GitHub Action.
        #[facet(default, skip_serializing_if = Option::is_none)]
        pub uses: Option<String>,

        /// Run a shell command.
        #[facet(default, skip_serializing_if = Option::is_none)]
        pub run: Option<String>,

        /// Inputs for the action.
        #[facet(default, skip_serializing_if = Option::is_none)]
        pub with: Option<IndexMap<String, String>>,

        /// Environment variables for this step.
        #[facet(default, skip_serializing_if = Option::is_none)]
        pub env: Option<IndexMap<String, String>>,

        /// Step ID for referencing outputs.
        #[facet(default, skip_serializing_if = Option::is_none)]
        pub id: Option<String>,
    }
}

// =============================================================================
// Helper constructors
// =============================================================================

impl Step {
    /// Create a step that uses a GitHub Action.
    pub fn uses(name: impl Into<String>, action: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            uses: Some(action.into()),
            run: None,
            with: None,
            env: None,
            id: None,
        }
    }

    /// Create a step that runs a shell command.
    pub fn run(name: impl Into<String>, command: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            uses: None,
            run: Some(command.into()),
            with: None,
            env: None,
            id: None,
        }
    }

    /// Add inputs to this step.
    pub fn with_inputs(
        mut self,
        inputs: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) -> Self {
        let map: IndexMap<String, String> = inputs
            .into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect();
        self.with = Some(map);
        self
    }

    /// Add environment variables to this step.
    pub fn with_env(
        mut self,
        env: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) -> Self {
        let map: IndexMap<String, String> =
            env.into_iter().map(|(k, v)| (k.into(), v.into())).collect();
        self.env = Some(map);
        self
    }
}

impl Job {
    /// Create a new job.
    pub fn new(runs_on: impl Into<String>) -> Self {
        Self {
            name: None,
            runs_on: runs_on.into(),
            container: None,
            needs: None,
            steps: Vec::new(),
        }
    }

    /// Set the display name for this job.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the container image for this job.
    pub fn container(mut self, image: impl Into<String>) -> Self {
        self.container = Some(image.into());
        self
    }

    /// Add dependencies to this job.
    pub fn needs(mut self, deps: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.needs = Some(deps.into_iter().map(Into::into).collect());
        self
    }

    /// Add steps to this job.
    pub fn steps(mut self, steps: impl IntoIterator<Item = Step>) -> Self {
        self.steps = steps.into_iter().collect();
        self
    }
}

// =============================================================================
// Common step patterns
// =============================================================================

/// Common steps used across jobs.
pub mod common {
    use super::*;

    /// Checkout the repository.
    pub fn checkout() -> Step {
        Step::uses("Checkout", "actions/checkout@v4")
    }

    /// Install Rust toolchain.
    pub fn install_rust() -> Step {
        Step::uses("Install Rust", "dtolnay/rust-toolchain@stable")
    }

    /// Install Rust with specific components.
    #[allow(dead_code)]
    pub fn install_rust_with(components: &str) -> Step {
        Step::uses("Install Rust", "dtolnay/rust-toolchain@stable")
            .with_inputs([("components", components)])
    }

    /// Install Rust with WASM target.
    #[allow(dead_code)]
    pub fn install_rust_wasm() -> Step {
        Step::uses("Install Rust", "dtolnay/rust-toolchain@stable")
            .with_inputs([("targets", "wasm32-unknown-unknown")])
    }

    /// Setup Rust cache.
    pub fn rust_cache() -> Step {
        Step::uses("Rust cache", "Swatinem/rust-cache@v2")
    }

    /// Install cargo-nextest.
    pub fn install_nextest() -> Step {
        Step::uses("Install nextest", "taiki-e/install-action@v2")
            .with_inputs([("tool", "cargo-nextest")])
    }

    /// Download grammar sources artifact.
    pub fn download_grammar_sources() -> Step {
        Step::uses("Download grammar sources", "actions/download-artifact@v4")
            .with_inputs([("name", "grammar-sources")])
    }

    /// Extract grammar sources tarball.
    pub fn extract_grammar_sources() -> Step {
        Step::run("Extract grammar sources", "tar -xvf grammar-sources.tar")
    }
}

// =============================================================================
// Workflow builders
// =============================================================================

/// Depot runner sizes.
#[allow(dead_code)]
pub mod runners {
    pub const UBUNTU_4: &str = "depot-ubuntu-24.04-4";
    pub const UBUNTU_32: &str = "depot-ubuntu-24.04-32";
    pub const UBUNTU_64: &str = "depot-ubuntu-24.04-64";
    pub const MACOS: &str = "depot-macos-latest";
    pub const WINDOWS_32: &str = "depot-windows-2022-32";
}

/// Configuration for CI workflow generation.
#[derive(Default)]
pub struct CiConfig {
    /// Plugin build groups (if available)
    pub plugin_groups: Option<PluginGroups>,
}

/// Build the CI workflow.
pub fn build_ci_workflow(config: &CiConfig) -> Workflow {
    use common::*;

    let mut jobs = IndexMap::new();

    // Generate job - runs first, produces grammar sources artifact
    jobs.insert(
        "generate".into(),
        Job::new(runners::UBUNTU_32)
            .name("🌱 Generate Grammars")
            .container("ghcr.io/bearcove/arborium-plugin-builder:latest")
            .steps([
            checkout(),
            Step::uses("Restore grammar generation cache", "actions/cache@v4")
                .with_inputs([
                    ("path", ".cache/arborium"),
                    (
                        "key",
                        "grammar-cache-v2-${{ hashFiles('crates/*/grammar/grammar.js', 'crates/*/grammar/package.json', 'crates/*/common/**') }}",
                    ),
                    ("restore-keys", "grammar-cache-v2-"),
                ]),
            Step::run("Generate grammar sources", "arborium-xtask gen"),
            Step::run(
                "Create grammar sources tarball",
                r#"# Collect all generated grammar/src directories
find crates -type d -name 'src' -path '*/grammar/src' > grammar_dirs.txt
tar -cvf grammar-sources.tar -T grammar_dirs.txt"#,
            ),
            Step::uses("Upload grammar sources", "actions/upload-artifact@v4")
                .with_inputs([
                    ("name", "grammar-sources"),
                    ("path", "grammar-sources.tar"),
                    ("retention-days", "1"),
                ]),
        ]),
    );

    // Test Linux job
    jobs.insert(
        "test-linux".into(),
        Job::new(runners::UBUNTU_32)
            .name("🐧 Test (Linux)")
            .container("ghcr.io/bearcove/arborium-plugin-builder:latest")
            .needs(["generate"])
            .steps([
                checkout(),
                download_grammar_sources(),
                extract_grammar_sources(),
                Step::run("Build", "cargo build --locked --verbose"),
                Step::run("Run tests", "cargo nextest run --locked --verbose"),
                Step::run(
                    "Build with all features",
                    "cargo build --locked --all-features --verbose",
                ),
            ]),
    );

    // Test macOS job
    jobs.insert(
        "test-macos".into(),
        Job::new(runners::MACOS)
            .name("🍎 Test (macOS)")
            .needs(["generate"])
            .steps([
                checkout(),
                download_grammar_sources(),
                extract_grammar_sources(),
                install_rust(),
                rust_cache(),
                install_nextest(),
                Step::run("Build", "cargo build --locked --verbose"),
                Step::run("Run tests", "cargo nextest run --locked --verbose"),
            ]),
    );

    // WASM job
    jobs.insert(
        "wasm".into(),
        Job::new(runners::UBUNTU_32)
            .name("🌐 WASM Compatibility")
            .container("ghcr.io/bearcove/arborium-plugin-builder:latest")
            .needs(["generate"])
            .steps([
                checkout(),
                download_grammar_sources(),
                extract_grammar_sources(),
                Step::run(
                    "Build arborium for WASM",
                    "cargo build --locked -p arborium --target wasm32-unknown-unknown",
                ),
                Step::run(
                    "Check for env imports in WASM",
                    r#"# Find all .wasm files and check for env imports
found_env_imports=false
for wasm_file in $(find target/wasm32-unknown-unknown -name "*.wasm" -type f); do
  if wasm-objdump -j Import -x "$wasm_file" 2>/dev/null | grep -q '<- env\.'; then
    echo "ERROR: Found env imports in $wasm_file:"
    wasm-objdump -j Import -x "$wasm_file" | grep '<- env\.'
    found_env_imports=true
  fi
done
if [ "$found_env_imports" = true ]; then
  echo "WASM modules should not have env imports - these won't work in the browser"
  exit 1
fi
echo "No env imports found - WASM modules are browser-compatible""#,
                ),
            ]),
    );

    // Clippy job
    jobs.insert(
        "clippy".into(),
        Job::new(runners::UBUNTU_32)
            .name("📎 Clippy")
            .container("ghcr.io/bearcove/arborium-plugin-builder:latest")
            .needs(["generate"])
            .steps([
                checkout(),
                download_grammar_sources(),
                extract_grammar_sources(),
                Step::run(
                    "Run Clippy",
                    "cargo clippy --locked --all-targets -- -D warnings",
                ),
            ]),
    );

    // Fmt job (no dependency on generate)
    jobs.insert(
        "fmt".into(),
        Job::new(runners::UBUNTU_4)
            .name("📝 Format")
            .container("ghcr.io/bearcove/arborium-plugin-builder:latest")
            .steps([
                checkout(),
                Step::run("Check formatting", "cargo fmt --all -- --check"),
                Step::run(
                    "Check CI workflow is up to date",
                    "arborium-xtask ci generate --check",
                ),
            ]),
    );

    // Docs job
    jobs.insert(
        "docs".into(),
        Job::new(runners::UBUNTU_32)
            .name("📚 Documentation")
            .container("ghcr.io/bearcove/arborium-plugin-builder:latest")
            .needs(["generate"])
            .steps([
                checkout(),
                download_grammar_sources(),
                extract_grammar_sources(),
                Step::run("Build docs", "cargo doc --locked --no-deps")
                    .with_env([("RUSTDOCFLAGS", "-D warnings")]),
            ]),
    );

    // Plugin build jobs (if groups are available)
    if let Some(ref groups) = config.plugin_groups {
        let total_groups = groups.groups.len();
        let mut plugin_job_ids = Vec::new();

        for group in &groups.groups {
            let job_id = format!("build-plugins-{}", group.index);
            let grammars_list = group.grammars.join(" ");
            let display_grammars = group.grammars.join(", ");
            let job_name = format!(
                "🔌 Plugins ({} of {}): {}",
                group.index + 1,
                total_groups,
                display_grammars
            );

            plugin_job_ids.push(job_id.clone());

            jobs.insert(
                job_id,
                Job::new(runners::UBUNTU_32)
                    .name(job_name)
                    .container("ghcr.io/bearcove/arborium-plugin-builder:latest")
                    .needs(["generate"])
                    .steps([
                        checkout(),
                        download_grammar_sources(),
                        extract_grammar_sources(),
                        Step::run(
                            format!("Build {}", display_grammars),
                            format!("arborium-xtask plugins build {}", grammars_list),
                        ),
                        Step::uses("Upload plugins artifact", "actions/upload-artifact@v4")
                            .with_inputs([
                                ("name", format!("plugins-group-{}", group.index)),
                                ("path", "dist/plugins".to_string()),
                                ("retention-days", "7".to_string()),
                            ]),
                    ]),
            );
        }

        // Collect all plugins and package for npm
        let mut collect_steps = vec![
            checkout(),
            Step::run("Create dist directory", "mkdir -p dist/plugins"),
        ];

        // Download all plugin artifacts
        for group in &groups.groups {
            collect_steps.push(
                Step::uses(
                    format!("Download plugins group {}", group.index),
                    "actions/download-artifact@v4",
                )
                .with_inputs([
                    ("name", format!("plugins-group-{}", group.index)),
                    ("path", "dist/plugins".to_string()),
                ]),
            );
        }

        // TODO: Add npm packaging step here
        collect_steps.push(Step::run(
            "List collected plugins",
            "find dist/plugins -type f | sort",
        ));

        jobs.insert(
            "collect-plugins".into(),
            Job::new(runners::UBUNTU_4)
                .name("📦 Collect Plugins")
                .needs(plugin_job_ids)
                .steps(collect_steps),
        );
    }

    Workflow {
        name: "CI".into(),
        on: On {
            push: Some(PushTrigger {
                branches: Some(vec!["main".into()]),
            }),
            pull_request: Some(PullRequestTrigger {
                branches: Some(vec!["main".into()]),
            }),
            merge_group: Some(MergeGroupTrigger {}),
        },
        env: Some(
            [
                ("CARGO_TERM_COLOR", "always"),
                ("CARGO_INCREMENTAL", "0"),
                ("CARGO_PROFILE_TEST_DEBUG", "0"),
            ]
            .into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect(),
        ),
        jobs,
    }
}

// =============================================================================
// Generation
// =============================================================================

use camino::Utf8Path;
use facet_diff::FacetDiff;
use facet_pretty::FacetPretty;
use miette::Result;

const GENERATED_HEADER: &str =
    "# GENERATED BY: cargo xtask ci generate\n# DO NOT EDIT - edit xtask/src/ci.rs instead\n";

/// Default number of plugin build groups for CI.
const DEFAULT_NUM_GROUPS: usize = 2;

/// Generate CI workflow files.
pub fn generate(repo_root: &Utf8Path, check: bool) -> Result<()> {
    // Try to load plugin timings if available
    let timings_path = repo_root.join("plugin-timings.json");
    let plugin_groups = if timings_path.exists() {
        match PluginTimings::load(&timings_path) {
            Ok(timings) => {
                println!(
                    "Loaded plugin timings from {} ({} plugins)",
                    timings_path,
                    timings.timings.len()
                );
                Some(PluginGroups::from_timings(&timings, DEFAULT_NUM_GROUPS))
            }
            Err(e) => {
                eprintln!(
                    "Warning: failed to load plugin timings from {}: {}",
                    timings_path, e
                );
                None
            }
        }
    } else {
        println!(
            "No plugin timings file found at {} - skipping plugin jobs",
            timings_path
        );
        println!("Run `cargo xtask plugins build --profile` to generate timings");
        None
    };

    let config = CiConfig { plugin_groups };
    let workflow = build_ci_workflow(&config);
    let yaml_content = format!(
        "{}{}\n",
        GENERATED_HEADER,
        facet_yaml::to_string(&workflow)
            .map_err(|e| miette::miette!("failed to serialize workflow: {}", e))?
    );

    let ci_path = repo_root.join(".github/workflows/ci.yml");

    if check {
        // Check mode: compare with existing file
        let existing = fs_err::read_to_string(&ci_path)
            .map_err(|e| miette::miette!("failed to read {}: {}", ci_path, e))?;

        if existing != yaml_content {
            // Show diff using facet-diff
            let existing_workflow: Workflow = facet_yaml::from_str(&existing)
                .map_err(|e| miette::miette!("failed to parse existing workflow: {}", e))?;

            println!("Workflow diff:");
            let diff = existing_workflow.diff(&workflow);
            println!("{}", diff);

            return Err(miette::miette!(
                "CI workflow is out of date. Run `cargo xtask ci generate` to update."
            ));
        }
        println!("CI workflow is up to date.");
    } else {
        // Generate mode: write the file
        println!("Generated workflow:");
        println!("{}", workflow.pretty());

        fs_err::write(&ci_path, &yaml_content)
            .map_err(|e| miette::miette!("failed to write {}: {}", ci_path, e))?;

        println!("\nWritten to: {}", ci_path);
    }

    Ok(())
}
