//! CI workflow generation for GitHub Actions.
//!
//! This module generates a single unified workflow that handles both CI and releases.
//! On regular pushes/PRs, it runs tests. On tag pushes, it also publishes.

use facet::Facet;
use indexmap::IndexMap;

use crate::build::PluginGroups;

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
            /// Tags to trigger on.
            #[facet(default, skip_serializing_if = Option::is_none)]
            pub tags: Option<Vec<String>>,
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

        /// Trigger on workflow_dispatch (manual).
        #[facet(default, skip_serializing_if = Option::is_none)]
        pub workflow_dispatch: Option<pub struct WorkflowDispatchTrigger {
            /// Input parameters.
            #[facet(default, skip_serializing_if = Option::is_none)]
            pub inputs: Option<IndexMap<String, WorkflowInput>>,
        }>,
    }
}

structstruck::strike! {
    /// A workflow input parameter.
    #[strikethrough[derive(Debug, Clone, Facet)]]
    #[facet(rename_all = "snake_case")]
    pub struct WorkflowInput {
        /// Description of the input.
        pub description: String,
        /// Whether the input is required.
        #[facet(default)]
        pub required: bool,
        /// The type of input.
        #[facet(rename = "type")]
        pub input_type: String,
        /// Default value.
        #[facet(default, skip_serializing_if = Option::is_none)]
        pub default: Option<String>,
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

        /// Outputs from this job.
        #[facet(default, skip_serializing_if = Option::is_none)]
        pub outputs: Option<IndexMap<String, String>>,

        /// Condition for running this job.
        #[facet(default, skip_serializing_if = Option::is_none, rename = "if")]
        pub if_condition: Option<String>,

        /// Permissions for this job (for trusted publishing, etc.)
        #[facet(default, skip_serializing_if = Option::is_none)]
        pub permissions: Option<IndexMap<String, String>>,

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

        /// The shell to use for the command.
        #[facet(default, skip_serializing_if = Option::is_none)]
        pub shell: Option<String>,

        /// Inputs for the action.
        #[facet(default, skip_serializing_if = Option::is_none)]
        pub with: Option<IndexMap<String, String>>,

        /// Environment variables for this step.
        #[facet(default, skip_serializing_if = Option::is_none)]
        pub env: Option<IndexMap<String, String>>,

        /// Step ID for referencing outputs.
        #[facet(default, skip_serializing_if = Option::is_none)]
        pub id: Option<String>,

        /// Condition for running this step.
        #[facet(default, skip_serializing_if = Option::is_none, rename = "if")]
        pub if_condition: Option<String>,
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
            shell: None,
            with: None,
            env: None,
            id: None,
            if_condition: None,
        }
    }

    /// Create a step that runs a shell command.
    /// All commands are automatically prefixed with `set -e` for strict error handling.
    pub fn run(name: impl Into<String>, command: impl Into<String>) -> Self {
        let cmd = command.into();
        // Prepend set -e if not already present
        let cmd = if cmd.starts_with("set -e") {
            cmd
        } else {
            format!("set -e\n{}", cmd)
        };
        Self {
            name: Some(name.into()),
            uses: None,
            run: Some(cmd),
            shell: Some("bash".into()),
            with: None,
            env: None,
            id: None,
            if_condition: None,
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

    /// Set step ID.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
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
            outputs: None,
            if_condition: None,
            permissions: None,
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

    /// Add outputs to this job.
    pub fn outputs(
        mut self,
        outputs: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) -> Self {
        self.outputs = Some(
            outputs
                .into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
        );
        self
    }

    /// Add a condition for this job.
    pub fn when(mut self, condition: impl Into<String>) -> Self {
        self.if_condition = Some(condition.into());
        self
    }

    /// Set permissions for this job (for trusted publishing, OIDC, etc.)
    pub fn permissions(
        mut self,
        perms: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) -> Self {
        self.permissions = Some(
            perms
                .into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
        );
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

    /// Setup Rust cache.
    /// Note: workspaces input tells it where to find Cargo.lock since there's no root workspace
    pub fn rust_cache() -> Step {
        Step::uses("Rust cache", "Swatinem/rust-cache@v2")
            .with_inputs([("workspaces", "crates/arborium")])
    }

    /// Install cargo-nextest.
    pub fn install_nextest() -> Step {
        Step::uses("Install nextest", "taiki-e/install-action@v2")
            .with_inputs([("tool", "cargo-nextest")])
    }

    /// Download generate output from artifact and extract tar.
    /// We use tar instead of raw artifact upload because GitHub Actions artifacts
    /// use zip which doesn't preserve Unix file permissions.
    pub fn download_generate_output() -> Vec<Step> {
        vec![
            Step::uses("Download generate output", "actions/download-artifact@v4")
                .with_inputs([("name", "generate-output"), ("path", ".")]),
            Step::run(
                "Extract generate output",
                "tar -xf generate-output.tar && rm generate-output.tar",
            ),
        ]
    }
}

// =============================================================================
// Workflow builders
// =============================================================================

/// Depot runner sizes.
pub mod runners {
    pub const UBUNTU_32: &str = "depot-ubuntu-24.04-32";
    pub const MACOS: &str = "depot-macos-latest";
    /// GitHub-hosted runner (required for OIDC trusted publishing)
    pub const UBUNTU_GITHUB: &str = "ubuntu-latest";
}

const CONTAINER: &str = "ghcr.io/bearcove/arborium-plugin-builder:latest";

/// Condition: this is a release (tag push with a valid version).
///
/// We intentionally gate publishing on the parsed version output from the
/// `generate` job so we can support both `vX.Y.Z` and `X.Y.Z` tags while
/// avoiding accidental publishes on arbitrary tags.
const IS_RELEASE: &str = "needs.generate.outputs.is_release == 'true'";

/// Configuration for workflow generation.
#[derive(Default)]
pub struct CiConfig {
    /// Plugin build groups (if available)
    pub plugin_groups: Option<PluginGroups>,
}

/// Build the unified CI + Release workflow.
pub fn build_workflow(config: &CiConfig) -> Workflow {
    use common::*;

    let mut jobs = IndexMap::new();

    // =========================================================================
    // STAGE 1: Generate grammar sources
    // =========================================================================
    jobs.insert(
        "generate".into(),
        Job::new(runners::UBUNTU_32)
            .name("Generate")
            .container(CONTAINER)
            .outputs([
                ("version".to_string(), "${{ steps.version.outputs.version }}".to_string()),
                ("is_release".to_string(), "${{ steps.version.outputs.is_release }}".to_string()),
            ])
            .steps([
                checkout(),
                // Parse version from tag (if this is a release)
                Step::run(
                    "Parse version",
                    r#"set -e
# Default to dev version (matches xtask default)
VERSION="0.0.0"
IS_RELEASE="false"

# On tag push, use the tag as version
if [[ "$GITHUB_REF" == refs/tags/* ]]; then
  TAG="${GITHUB_REF#refs/tags/}"
  TAG="${TAG#v}"

  # Only publish on semver-ish tags (optionally with prerelease/build suffix).
  if [[ "$TAG" =~ ^[0-9]+\.[0-9]+\.[0-9]+([-.].*)?$ ]]; then
    VERSION="$TAG"
    IS_RELEASE="true"
  fi
fi

echo "version=$VERSION" >> $GITHUB_OUTPUT
echo "is_release=$IS_RELEASE" >> $GITHUB_OUTPUT
echo "Version: $VERSION (release: $IS_RELEASE)""#,
                )
                .with_id("version"),
                // Layer 1: Cache xtask build (keyed on source code)
                Step::uses("Restore xtask cache", "actions/cache@v4")
                    .with_inputs([
                        ("path", "xtask/target"),
                        (
                            "key",
                            "xtask-${{ hashFiles('xtask/Cargo.lock', 'xtask/src/**/*.rs') }}",
                        ),
                    ]),
                // Build xtask (not pre-baked in container, built fresh from source)
                Step::run(
                    "Build xtask",
                    "cargo build --release --manifest-path xtask/Cargo.toml",
                ),
                // Layer 2: Cache grammars using canonical key from xtask
                // This key includes tree-sitter version and all grammar inputs
                Step::run(
                    "Compute grammar cache key",
                    "echo \"key=$(./xtask/target/release/xtask cache-key)\" >> $GITHUB_OUTPUT",
                )
                .with_id("grammar-cache-key"),
                Step::uses("Restore grammar generation cache", "actions/cache@v4")
                    .with_inputs([
                        ("path", ".cache/arborium"),
                        (
                            "key",
                            "grammar-v1-${{ hashFiles('xtask/Cargo.lock', 'xtask/src/**/*.rs') }}-${{ steps.grammar-cache-key.outputs.key }}",
                        ),
                        (
                            "restore-keys",
                            "grammar-v1-${{ hashFiles('xtask/Cargo.lock', 'xtask/src/**/*.rs') }}-",
                        ),
                    ]),
                // Generate with version (from tag or 0.0.0 for non-release)
                Step::run(
                    "Generate grammar sources",
                    "./xtask/target/release/xtask gen --version ${{ steps.version.outputs.version }} --quiet",
                ),
                // Upload generated files for downstream jobs
                // Note: no root Cargo.toml/lock - each crate is standalone
                // Includes xtask binary so downstream jobs don't need to rebuild it.
                // Includes version.json so publish jobs know the release version.
                // We tar first because GitHub Actions artifacts use zip which doesn't
                // preserve Unix file permissions (causing lib.rs to be excluded from
                // cargo publish).
                Step::run(
                    "Create generate output tarball",
                    // Exclude node_modules and dist to avoid bloat - only generated source files needed
                    "tar --exclude='node_modules' --exclude='dist' -cf generate-output.tar crates/ langs/ packages/ xtask/target/release/xtask version.json",
                ),
                Step::uses("Upload generate output", "actions/upload-artifact@v4")
                    .with_inputs([
                        ("name", "generate-output"),
                        ("path", "generate-output.tar"),
                        ("retention-days", "1"),
                    ]),
            ]),
    );

    // =========================================================================
    // STAGE 2: CI Jobs (always run)
    // =========================================================================

    // Test Linux
    // Note: no root workspace, so we target crates/arborium directly
    jobs.insert(
        "test-linux".into(),
        Job::new(runners::UBUNTU_32)
            .name("Test (Linux)")
            .container(CONTAINER)
            .needs(["generate"])
            .steps(
                [checkout()].into_iter()
                    .chain(download_generate_output())
                    .chain([
                        rust_cache(),
                        Step::run("Build", "cargo build --manifest-path crates/arborium/Cargo.toml --verbose"),
                        Step::run("Run tests", "cargo nextest run --manifest-path crates/arborium/Cargo.toml --verbose --no-tests=pass"),
                        Step::run(
                            "Build with all features",
                            "cargo build --manifest-path crates/arborium/Cargo.toml --all-features --verbose",
                        ),
                        Step::run("Build arborium-rustdoc", "cargo build --manifest-path crates/arborium-rustdoc/Cargo.toml --verbose"),
                        Step::run("Test arborium-rustdoc", "cargo test --manifest-path crates/arborium-rustdoc/Cargo.toml --verbose"),
                    ])
            ),
    );

    // Test macOS (no container, needs to install tools)
    // Note: no root workspace, so we target crates/arborium directly
    jobs.insert(
        "test-macos".into(),
        Job::new(runners::MACOS)
            .name("Test (macOS)")
            .needs(["generate"])
            .steps(
                [checkout()].into_iter()
                    .chain(download_generate_output())
                    .chain([
                        install_rust(),
                        rust_cache(),
                        install_nextest(),
                        Step::run("Build", "cargo build --manifest-path crates/arborium/Cargo.toml --verbose"),
                        Step::run("Run tests", "cargo nextest run --manifest-path crates/arborium/Cargo.toml --verbose --no-tests=pass"),
                        Step::run("Build arborium-rustdoc", "cargo build --manifest-path crates/arborium-rustdoc/Cargo.toml --verbose"),
                        Step::run("Test arborium-rustdoc", "cargo test --manifest-path crates/arborium-rustdoc/Cargo.toml --verbose"),
                    ])
            ),
    );

    // Clippy
    // Note: no root workspace, so we target crates/arborium directly
    jobs.insert(
        "clippy".into(),
        Job::new(runners::UBUNTU_32)
            .name("Clippy")
            .container(CONTAINER)
            .needs(["generate"])
            .steps(
                [checkout()].into_iter()
                    .chain(download_generate_output())
                    .chain([
                        Step::run("Run Clippy", "cargo clippy --manifest-path crates/arborium/Cargo.toml --all-targets -- -D warnings"),
                        Step::run("Run Clippy on arborium-rustdoc", "cargo clippy --manifest-path crates/arborium-rustdoc/Cargo.toml --all-targets -- -D warnings"),
                    ])
            ),
    );

    // Documentation
    // Note: no root workspace, so we target crates/arborium directly
    jobs.insert(
        "docs".into(),
        Job::new(runners::UBUNTU_32)
            .name("Documentation")
            .container(CONTAINER)
            .needs(["generate"])
            .steps(
                [checkout()]
                    .into_iter()
                    .chain(download_generate_output())
                    .chain([Step::run(
                        "Build docs",
                        "cargo doc --manifest-path crates/arborium/Cargo.toml --no-deps",
                    )
                    .with_env([("RUSTDOCFLAGS", "-D warnings")])]),
            ),
    );

    // =========================================================================
    // STAGE 2b: Plugin builds (one job per langs/group-* folder)
    // =========================================================================
    let mut plugin_job_ids = Vec::new();
    let mut plugin_group_names = Vec::new();

    if let Some(ref groups) = config.plugin_groups {
        for group in &groups.groups {
            let job_id = format!("build-plugins-{}", group.name);
            let grammars_list = group.grammars.join(" ");
            let display_grammars = group.grammars.join(", ");
            let job_name = format!("Plugins ({}): {}", group.name, display_grammars);

            plugin_job_ids.push(job_id.clone());
            plugin_group_names.push(group.name.clone());

            jobs.insert(
                job_id,
                Job::new(runners::UBUNTU_32)
                    .name(job_name)
                    .container(CONTAINER)
                    .needs(["generate"])
                    .steps(
                        [checkout()]
                            .into_iter()
                            .chain(download_generate_output())
                            .chain([
                                Step::run(
                                    format!("Build {}", display_grammars),
                                    format!(
                                        "./xtask/target/release/xtask build {} -o dist/plugins",
                                        grammars_list
                                    ),
                                ),
                                Step::uses("Upload plugins artifact", "actions/upload-artifact@v4")
                                    .with_inputs([
                                        ("name", format!("plugins-group-{}", group.name)),
                                        ("path", "dist/plugins".to_string()),
                                        ("retention-days", "7".to_string()),
                                    ]),
                            ]),
                    ),
            );
        }
    }

    // =========================================================================
    // STAGE 3: Publish to crates.io (only on release, using trusted publishing)
    // =========================================================================
    jobs.insert(
        "publish-crates".into(),
        Job::new(runners::UBUNTU_32)
            .name("Publish crates.io")
            .container(CONTAINER)
            .needs(["generate", "test-linux", "clippy"])
            .when(IS_RELEASE)
            .permissions([("id-token", "write"), ("contents", "read")])
            .steps(
                [checkout()]
                    .into_iter()
                    .chain(download_generate_output())
                    .chain([
                        // Exchange OIDC token for crates.io access token
                        Step::uses(
                            "Authenticate with crates.io",
                            "rust-lang/crates-io-auth-action@v1",
                        )
                        .with_id("crates-io-auth"),
                        Step::run(
                            "Publish to crates.io",
                            "./xtask/target/release/xtask publish crates",
                        )
                        .with_env([(
                            "CARGO_REGISTRY_TOKEN",
                            "${{ steps.crates-io-auth.outputs.token }}",
                        )]),
                    ]),
            ),
    );

    // =========================================================================
    // STAGE 3b: Publish to npm (only on release, after plugins built)
    // =========================================================================
    if !plugin_job_ids.is_empty() {
        let mut npm_needs = vec!["generate".to_string()];
        npm_needs.extend(plugin_job_ids.iter().cloned());

        let mut npm_steps: Vec<Step> = vec![checkout()];
        npm_steps.extend(download_generate_output());

        // Download all plugin artifacts
        for group_name in &plugin_group_names {
            npm_steps.push(
                Step::uses(
                    format!("Download plugins group {}", group_name),
                    "actions/download-artifact@v4",
                )
                .with_inputs([
                    ("name", format!("plugins-group-{}", group_name)),
                    ("path", "dist/plugins".to_string()),
                ]),
            );
        }

        npm_steps.extend([
            Step::run(
                "List plugins",
                "find dist/plugins -name 'package.json' | head -20",
            ),
            Step::run(
                "Install main package dependencies",
                "cd packages/arborium && npm ci",
            ),
            // No NODE_AUTH_TOKEN needed - OIDC trusted publishing uses id-token permission
            Step::run(
                "Publish to npm",
                "./xtask/target/release/xtask publish npm -o dist/plugins",
            ),
        ]);

        jobs.insert(
            "publish-npm".into(),
            Job::new(runners::UBUNTU_GITHUB)
                .name("Publish npm")
                .container(CONTAINER)
                .needs(npm_needs)
                .when(IS_RELEASE)
                // id-token for npm provenance (trusted publishing)
                .permissions([("id-token", "write"), ("contents", "read")])
                .steps(npm_steps),
        );
    }

    // =========================================================================
    // Build the workflow
    // =========================================================================
    Workflow {
        name: "CI".into(),
        on: On {
            push: Some(PushTrigger {
                branches: Some(vec!["main".into()]),
                tags: Some(vec![
                    // Historical + documented tag format.
                    "v*".into(),
                    // Allow bare semver tags like `2.0.0`.
                    "[0-9]*.[0-9]*.[0-9]*".into(),
                ]),
            }),
            pull_request: Some(PullRequestTrigger {
                branches: Some(vec!["main".into()]),
            }),
            merge_group: Some(MergeGroupTrigger {}),
            workflow_dispatch: None,
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
use miette::Result;

const GENERATED_HEADER: &str =
    "# GENERATED BY: cargo xtask ci generate\n# DO NOT EDIT - edit xtask/src/ci.rs instead\n";

/// Generate CI workflow files.
pub fn generate(repo_root: &Utf8Path, check: bool) -> Result<()> {
    // Discover plugin groups from langs/group-* folders
    let langs_dir = repo_root.join("langs");
    let plugin_groups = match PluginGroups::discover(&langs_dir) {
        Ok(groups) => {
            let total_grammars: usize = groups.groups.iter().map(|g| g.grammars.len()).sum();
            println!(
                "Discovered {} plugin groups with {} total grammars from {}",
                groups.groups.len(),
                total_grammars,
                langs_dir
            );
            for group in &groups.groups {
                println!("  - {}: {}", group.name, group.grammars.join(", "));
            }
            Some(groups)
        }
        Err(e) => {
            eprintln!("Warning: failed to discover plugin groups: {}", e);
            None
        }
    };

    let config = CiConfig { plugin_groups };
    let workflow = build_workflow(&config);
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
            return Err(miette::miette!(
                "CI workflow is out of date. Run `cargo xtask ci generate` to update."
            ));
        }
        println!("CI workflow is up to date.");
    } else {
        // Generate mode: write the file
        fs_err::write(&ci_path, &yaml_content)
            .map_err(|e| miette::miette!("failed to write {}: {}", ci_path, e))?;

        println!("Written to: {}", ci_path);
    }

    // Delete old release.yml if it exists (now unified into ci.yml)
    let release_path = repo_root.join(".github/workflows/release.yml");
    if release_path.exists() {
        fs_err::remove_file(&release_path)
            .map_err(|e| miette::miette!("failed to remove {}: {}", release_path, e))?;
        println!("Removed old release.yml (now unified into ci.yml)");
    }

    Ok(())
}
