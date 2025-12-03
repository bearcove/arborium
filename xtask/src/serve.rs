//! Serve command - builds and serves the WASM demo.
//!
//! This module generates registry.json from arborium.kdl files and serves
//! the demo with all grammar metadata and inlined sample content.

use crate::types::{CrateRegistry, GrammarConfig, SampleConfig};
use crate::util;
use camino::{Utf8Path, Utf8PathBuf};
use facet::Facet;
use owo_colors::OwoColorize;
use pulldown_cmark::{Options, Parser, html};
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

// =============================================================================
// Registry JSON types (for demo consumption)
// =============================================================================

structstruck::strike! {
    /// The registry.json format consumed by the demo frontend.
    #[strikethrough[derive(Debug, Clone, Facet)]]
    pub struct Registry {
        /// All grammars in the registry, sorted by id.
        pub grammars: Vec<pub struct RegistryGrammar {
            /// Unique identifier (e.g., "rust", "javascript").
            pub id: String,

            /// Crate name (e.g., "arborium-rust").
            #[facet(rename = "crate")]
            pub crate_name: String,

            /// Human-readable name (e.g., "Rust", "JavaScript").
            pub name: String,

            /// Iconify icon identifier (e.g., "devicon-plain:rust").
            pub icon: Option<String>,

            /// Quality tier (1 = best, 5 = experimental).
            pub tier: Option<u8>,

            /// Category tag (e.g., "code", "markup", "data").
            pub tag: String,

            /// Brief description of the language.
            pub description: Option<String>,

            /// Creator(s) of the language.
            pub inventor: Option<String>,

            /// Year the language was first released.
            pub year: Option<u16>,

            /// URL to more information.
            pub link: Option<String>,

            /// Fun facts or interesting history.
            pub trivia: Option<String>,

            /// File extension aliases (e.g., ["rs"] for Rust).
            pub aliases: Vec<String>,

            /// Sample files (content served separately via /samples/{id}.txt).
            pub samples: Vec<pub struct RegistrySample {
                /// Path relative to crate (e.g., "samples/example.rs").
                pub path: String,

                /// Description of what this sample demonstrates.
                pub description: Option<String>,
            }>,
        }>,
    }
}

impl Registry {
    /// Build the registry from a CrateRegistry, inlining sample content.
    pub fn from_crate_registry(registry: &CrateRegistry, crates_dir: &Utf8Path) -> Self {
        let mut grammars: Vec<RegistryGrammar> = registry
            .all_grammars()
            .filter(|(_, _, grammar)| !grammar.is_internal())
            .map(|(crate_name, _config, grammar)| {
                let crate_path = crates_dir.join(crate_name);
                RegistryGrammar::from_grammar_config(crate_name, grammar, &crate_path)
            })
            .collect();

        // Sort by id for consistent output
        grammars.sort_by(|a, b| a.id.cmp(&b.id));

        Self { grammars }
    }

    /// Serialize to pretty JSON using facet-json.
    pub fn to_json_pretty(&self) -> String {
        facet_json::to_string_pretty(self)
    }
}

impl RegistryGrammar {
    /// Build from a GrammarConfig, reading sample content from disk.
    fn from_grammar_config(
        crate_name: &str,
        grammar: &GrammarConfig,
        crate_path: &Utf8Path,
    ) -> Self {
        let samples: Vec<RegistrySample> = grammar
            .samples
            .iter()
            .filter_map(|sample| RegistrySample::from_sample_config(sample, crate_path))
            .collect();

        Self {
            id: grammar.id.value.to_string(),
            crate_name: crate_name.to_string(),
            name: grammar.name.value.to_string(),
            icon: grammar.icon.as_ref().map(|i| i.value.clone()),
            tier: grammar.tier.as_ref().map(|t| *t.value),
            tag: grammar.tag.value.to_string(),
            description: grammar.description.as_ref().map(|d| d.value.clone()),
            inventor: grammar.inventor.as_ref().map(|i| i.value.clone()),
            year: grammar.year.as_ref().map(|y| y.value),
            link: grammar.link.as_ref().map(|l| l.value.clone()),
            trivia: grammar.trivia.as_ref().map(|t| t.value.clone()),
            aliases: grammar
                .aliases
                .as_ref()
                .map(|a| a.values.clone())
                .unwrap_or_default(),
            samples,
        }
    }
}

impl RegistrySample {
    /// Build from a SampleConfig (content is served separately).
    fn from_sample_config(sample: &SampleConfig, crate_path: &Utf8Path) -> Option<Self> {
        // Check the file exists
        let sample_path = crate_path.join(&*sample.path);
        if !sample_path.exists() {
            return None;
        }

        Some(Self {
            path: sample.path.value.to_string(),
            description: sample.description.as_ref().map(|d| d.value.clone()),
        })
    }
}

// =============================================================================
// Serve command implementation
// =============================================================================

/// Build and serve the WASM demo.
pub fn serve(crates_dir: &Utf8Path, addr: &str, port: Option<u16>, dev: bool) {
    let repo_root = util::find_repo_root().expect("Could not find repo root");
    let demo_dir = repo_root.join("demo");

    println!(
        "{} {}",
        "==>".cyan().bold(),
        "Building arborium demo".bold()
    );
    if dev {
        println!("    {}", "(dev mode - skipping optimizations)".dimmed());
    }
    println!();

    // Step 0: Build WASM
    step("Building WASM", || build_wasm(&demo_dir, dev));

    // Step 1: Generate registry.json and get the registry for later use
    let registry = step_with_result("Generating registry.json", || {
        generate_registry_json(crates_dir, &demo_dir)
    });

    // Step 2: Generate individual sample files
    step("Generating sample files", || {
        generate_sample_files(crates_dir, &registry, &demo_dir)
    });

    // Step 2: Fetch icons (from registry + template)
    let icons = step_with_result("Fetching icons", || {
        fetch_icons_from_registry(&registry, &demo_dir)
    });

    // Step 3: Generate index.html from template
    step("Generating index.html", || {
        generate_index_html(&demo_dir, &icons)
    });

    // Step 4: Generate app.generated.js
    step("Generating app.generated.js", || {
        generate_app_js(&demo_dir, &registry, &icons)
    });

    // Step 5: Pre-compress files
    if dev {
        step("Pre-compressing files (fast)", || {
            precompress_files_fast(&demo_dir)
        });
    } else {
        step("Pre-compressing files", || precompress_files(&demo_dir));
    }

    // Step 6: Start HTTP server
    println!(
        "\n{} {}",
        "==>".cyan().bold(),
        "Starting HTTP server".bold()
    );

    let (server, actual_port) = bind_server(addr, port);
    let display_addr = if addr == "0.0.0.0" { "localhost" } else { addr };
    let url = format!("http://{}:{}", display_addr, actual_port);

    println!();
    println!(
        "  {} {}",
        "✓".green().bold(),
        "Demo server ready!".green().bold()
    );
    println!();
    println!("    {} {}", "→".cyan(), url.cyan().bold().underline());
    println!();
    println!("    {}", "Press Ctrl+C to stop".dimmed());
    println!();

    // Serve files
    serve_files(server, &demo_dir);
}

fn step<F, E>(name: &str, f: F)
where
    F: FnOnce() -> Result<(), E>,
    E: std::fmt::Display,
{
    print!("  {} {}... ", "●".cyan(), name);
    std::io::stdout().flush().ok();

    match f() {
        Ok(()) => println!("{}", "done".green()),
        Err(e) => {
            println!("{}", "failed".red());
            eprintln!("    Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn step_with_result<T, F, E>(name: &str, f: F) -> T
where
    F: FnOnce() -> Result<T, E>,
    E: std::fmt::Display,
{
    print!("  {} {}... ", "●".cyan(), name);
    std::io::stdout().flush().ok();

    match f() {
        Ok(result) => {
            println!("{}", "done".green());
            result
        }
        Err(e) => {
            println!("{}", "failed".red());
            eprintln!("    Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn generate_sample_files(
    crates_dir: &Utf8Path,
    registry: &Registry,
    demo_dir: &Path,
) -> Result<(), String> {
    let samples_dir = demo_dir.join("samples");
    if !samples_dir.exists() {
        fs::create_dir_all(&samples_dir).map_err(|e| e.to_string())?;
    }

    for grammar in &registry.grammars {
        if let Some(sample) = grammar.samples.first() {
            // Read sample content from crate
            let crate_path = crates_dir.join(&grammar.crate_name);
            let sample_path = crate_path.join(&sample.path);

            if let Ok(content) = fs::read_to_string(&sample_path) {
                // Get extension from original sample path
                let ext = std::path::Path::new(&sample.path)
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("txt");
                // Write to demo/samples/{id}.{ext}
                let output_path = samples_dir.join(format!("{}.{}", grammar.id, ext));
                fs::write(&output_path, &content).map_err(|e| e.to_string())?;
            }
        }
    }

    Ok(())
}

fn build_wasm(demo_dir: &Path, dev: bool) -> Result<(), String> {
    use crate::tool::Tool;

    // Nuke pkg/ to avoid stale cached files
    let pkg_dir = demo_dir.join("pkg");
    if pkg_dir.exists() {
        fs::remove_dir_all(&pkg_dir).map_err(|e| format!("Failed to remove pkg/: {}", e))?;
    }

    let wasm_pack = Tool::WasmPack.find().map_err(|e| e.to_string())?;
    let mut cmd = wasm_pack.command();
    cmd.arg("build").arg(demo_dir).arg("--target").arg("web");

    if dev {
        cmd.arg("--dev");
    } else {
        cmd.arg("--release");
    }

    let status = cmd
        .status()
        .map_err(|e| format!("Failed to run wasm-pack: {}", e))?;

    if !status.success() {
        return Err("wasm-pack build failed".to_string());
    }

    // Pre-compress the WASM file
    let wasm_path = demo_dir.join("pkg").join("arborium_demo_bg.wasm");
    if wasm_path.exists() {
        let wasm_data = fs::read(&wasm_path).map_err(|e| e.to_string())?;

        // Brotli
        let br_path = PathBuf::from(format!("{}.br", wasm_path.display()));
        let br_data = if dev {
            compress_brotli_fast(&wasm_data).map_err(|e| e.to_string())?
        } else {
            compress_brotli(&wasm_data).map_err(|e| e.to_string())?
        };
        fs::write(&br_path, &br_data).map_err(|e| e.to_string())?;

        // Gzip
        let gz_path = PathBuf::from(format!("{}.gz", wasm_path.display()));
        let gz_data = if dev {
            compress_gzip_fast(&wasm_data).map_err(|e| e.to_string())?
        } else {
            compress_gzip(&wasm_data).map_err(|e| e.to_string())?
        };
        fs::write(&gz_path, &gz_data).map_err(|e| e.to_string())?;
    }

    // Copy static assets to pkg/
    let assets = ["styles.css", "Iosevka-Regular.woff2", "Iosevka-Bold.woff2"];
    for asset in assets {
        let src = demo_dir.join(asset);
        let dest = demo_dir.join("pkg").join(asset);
        if src.exists() {
            fs::copy(&src, &dest).map_err(|e| format!("Failed to copy {}: {}", asset, e))?;
        }
    }

    Ok(())
}

fn generate_registry_json(crates_dir: &Utf8Path, demo_dir: &Path) -> Result<Registry, String> {
    let crates_dir =
        Utf8PathBuf::from_path_buf(crates_dir.to_path_buf().into()).map_err(|_| "non-UTF8 path")?;

    let crate_registry = CrateRegistry::load(&crates_dir).map_err(|e| e.to_string())?;
    let registry = Registry::from_crate_registry(&crate_registry, &crates_dir);

    let json = registry.to_json_pretty();
    let registry_path = demo_dir.join("registry.json");
    fs::write(&registry_path, &json).map_err(|e| e.to_string())?;

    // Also write compressed versions
    let br_path = demo_dir.join("registry.json.br");
    let br_data = compress_brotli(json.as_bytes()).map_err(|e| e.to_string())?;
    fs::write(&br_path, &br_data).map_err(|e| e.to_string())?;

    Ok(registry)
}

fn fetch_icons_from_registry(
    registry: &Registry,
    demo_dir: &Path,
) -> Result<BTreeMap<String, String>, String> {
    let template_path = demo_dir.join("template.html");
    let template = fs::read_to_string(&template_path).map_err(|e| e.to_string())?;

    // Collect all icon names from registry
    let mut icon_names: HashSet<String> = HashSet::new();
    for grammar in &registry.grammars {
        if let Some(ref icon) = grammar.icon {
            icon_names.insert(icon.clone());
        }
    }

    // Also collect icons from template ({{ICON:xxx}} patterns)
    let icon_pattern = regex::Regex::new(r"\{\{ICON:([^}]+)\}\}").unwrap();
    for cap in icon_pattern.captures_iter(&template) {
        if let Some(icon) = cap.get(1) {
            icon_names.insert(icon.as_str().to_string());
        }
    }

    // Add fallback icon
    icon_names.insert("mdi:code-tags".to_string());

    // Load cached icons
    let cache_path = demo_dir.join(".icon-cache.json");
    let mut cached_icons: BTreeMap<String, String> = if cache_path.exists() {
        let content = fs::read_to_string(&cache_path).unwrap_or_default();
        // Parse JSON manually since we don't have serde
        parse_icon_cache(&content)
    } else {
        BTreeMap::new()
    };

    let mut icons: BTreeMap<String, String> = BTreeMap::new();
    let mut fetch_count = 0;

    for icon_name in &icon_names {
        // Check cache first
        if let Some(svg) = cached_icons.get(icon_name) {
            icons.insert(icon_name.clone(), svg.clone());
            continue;
        }

        // Parse icon name (prefix:name)
        let parts: Vec<&str> = icon_name.split(':').collect();
        if parts.len() != 2 {
            continue;
        }
        let (prefix, name) = (parts[0], parts[1]);

        // Fetch from Iconify API
        let url = format!("https://api.iconify.design/{}/{}.svg", prefix, name);
        match ureq::get(&url).call() {
            Ok(resp) => {
                if let Ok(svg) = resp.into_string() {
                    // Clean up the SVG
                    let cleaned = svg
                        .replace("xmlns=\"http://www.w3.org/2000/svg\"", "")
                        .replace("xmlns:xlink=\"http://www.w3.org/1999/xlink\"", "");
                    icons.insert(icon_name.clone(), cleaned.clone());
                    cached_icons.insert(icon_name.clone(), cleaned);
                    fetch_count += 1;
                }
            }
            Err(_) => {
                // Skip failed icons
            }
        }
    }

    // Save cache
    if fetch_count > 0 {
        let cache_json = serialize_icon_cache(&cached_icons);
        let _ = fs::write(&cache_path, cache_json);
    }

    Ok(icons)
}

fn parse_icon_cache(content: &str) -> BTreeMap<String, String> {
    // Simple JSON parsing for icon cache (key-value string pairs)
    let mut result = BTreeMap::new();
    // Use facet-json to parse
    if let Ok(map) = facet_json::from_str::<BTreeMap<String, String>>(content) {
        result = map;
    }
    result
}

fn serialize_icon_cache(icons: &BTreeMap<String, String>) -> String {
    facet_json::to_string_pretty(icons)
}

/// Theme metadata for generating swatches
struct ThemeInfo {
    id: &'static str,
    name: &'static str,
    variant: &'static str,
    bg: &'static str,
}

/// All themes with their metadata (must match CSS and app.js)
const THEMES: &[ThemeInfo] = &[
    // Catppuccin family
    ThemeInfo {
        id: "mocha",
        name: "Catppuccin Mocha",
        variant: "dark",
        bg: "#1e1e2e",
    },
    ThemeInfo {
        id: "macchiato",
        name: "Catppuccin Macchiato",
        variant: "dark",
        bg: "#24273a",
    },
    ThemeInfo {
        id: "frappe",
        name: "Catppuccin Frappe",
        variant: "dark",
        bg: "#303446",
    },
    ThemeInfo {
        id: "latte",
        name: "Catppuccin Latte",
        variant: "light",
        bg: "#eff1f5",
    },
    // Popular dark themes
    ThemeInfo {
        id: "tokyo-night",
        name: "Tokyo Night",
        variant: "dark",
        bg: "#1a1b26",
    },
    ThemeInfo {
        id: "dracula",
        name: "Dracula",
        variant: "dark",
        bg: "#282a36",
    },
    ThemeInfo {
        id: "monokai",
        name: "Monokai Pro",
        variant: "dark",
        bg: "#2d2a2e",
    },
    ThemeInfo {
        id: "one-dark",
        name: "One Dark",
        variant: "dark",
        bg: "#282c34",
    },
    ThemeInfo {
        id: "nord",
        name: "Nord",
        variant: "dark",
        bg: "#2e3440",
    },
    ThemeInfo {
        id: "gruvbox-dark",
        name: "Gruvbox Dark",
        variant: "dark",
        bg: "#282828",
    },
    ThemeInfo {
        id: "rose-pine-moon",
        name: "Rosé Pine Moon",
        variant: "dark",
        bg: "#232136",
    },
    ThemeInfo {
        id: "kanagawa-dragon",
        name: "Kanagawa Dragon",
        variant: "dark",
        bg: "#181616",
    },
    ThemeInfo {
        id: "cobalt2",
        name: "Cobalt2",
        variant: "dark",
        bg: "#193549",
    },
    ThemeInfo {
        id: "zenburn",
        name: "Zenburn",
        variant: "dark",
        bg: "#3f3f3f",
    },
    ThemeInfo {
        id: "melange-dark",
        name: "Melange Dark",
        variant: "dark",
        bg: "#292522",
    },
    ThemeInfo {
        id: "monokai-aqua",
        name: "Monokai Aqua",
        variant: "dark",
        bg: "#222222",
    },
    ThemeInfo {
        id: "desert256",
        name: "Desert256",
        variant: "dark",
        bg: "#000000",
    },
    // GitHub
    ThemeInfo {
        id: "github-dark",
        name: "GitHub Dark",
        variant: "dark",
        bg: "#0d1117",
    },
    ThemeInfo {
        id: "github-light",
        name: "GitHub Light",
        variant: "light",
        bg: "#ffffff",
    },
    // Light themes
    ThemeInfo {
        id: "gruvbox-light",
        name: "Gruvbox Light",
        variant: "light",
        bg: "#fbf1c7",
    },
    ThemeInfo {
        id: "alabaster",
        name: "Alabaster",
        variant: "light",
        bg: "#f7f7f7",
    },
    ThemeInfo {
        id: "dayfox",
        name: "Dayfox",
        variant: "light",
        bg: "#f6f2ee",
    },
    ThemeInfo {
        id: "melange-light",
        name: "Melange Light",
        variant: "light",
        bg: "#f1f1f1",
    },
];

/// Generate HTML for theme swatches in the "Theme support" section
fn generate_theme_swatches() -> String {
    let mut html = String::new();

    // Sample code snippet (Rust) for each theme preview
    // Keep it short to fit in small boxes
    let sample_code = r#"<a-k>fn</a-k> <a-f>main</a-f><a-p>()</a-p> <a-p>{</a-p>
    <a-k>let</a-k> <a-v>x</a-v> <a-o>=</a-o> <a-n>42</a-n><a-p>;</a-p>
    <a-fb>println!</a-fb><a-p>(</a-p><a-s>"Hello"</a-s><a-p>)</a-p><a-p>;</a-p>
<a-p>}</a-p>"#;

    for theme in THEMES {
        html.push_str(&format!(
            r#"<div class="theme-preview" data-variant="{variant}" data-theme="{id}">
    <pre style="background: {bg}; padding: 0.75rem; border-radius: 6px;"><code>{code}</code></pre>
    <span class="theme-name">{name}</span>
</div>
"#,
            id = theme.id,
            variant = theme.variant,
            bg = theme.bg,
            name = theme.name,
            code = sample_code,
        ));
    }

    html
}

fn generate_index_html(demo_dir: &Path, icons: &BTreeMap<String, String>) -> Result<(), String> {
    let template_path = demo_dir.join("template.html");
    let output_path = demo_dir.join("index.html");

    let template = fs::read_to_string(&template_path).map_err(|e| e.to_string())?;

    // Replace {{ICON:xxx}} with inline SVGs
    let icon_pattern = regex::Regex::new(r"\{\{ICON:([^}]+)\}\}").unwrap();
    let mut html = template.clone();

    for cap in icon_pattern.captures_iter(&template) {
        let full_match = cap.get(0).unwrap().as_str();
        let icon_name = cap.get(1).unwrap().as_str();
        if let Some(svg) = icons.get(icon_name) {
            html = html.replace(full_match, svg);
        } else {
            html = html.replace(full_match, "");
        }
    }

    // Generate theme swatches
    let swatches_html = generate_theme_swatches();
    html = html.replace("{{THEME_SWATCHES}}", &swatches_html);

    fs::write(&output_path, &html).map_err(|e| e.to_string())?;
    Ok(())
}

fn generate_app_js(
    demo_dir: &Path,
    registry: &Registry,
    icons: &BTreeMap<String, String>,
) -> Result<(), String> {
    let app_js_path = demo_dir.join("app.js");
    let output_path = demo_dir.join("pkg").join("app.generated.js");

    // Ensure pkg directory exists
    let pkg_dir = demo_dir.join("pkg");
    if !pkg_dir.exists() {
        fs::create_dir_all(&pkg_dir).map_err(|e| e.to_string())?;
    }

    let app_js = fs::read_to_string(&app_js_path).map_err(|e| e.to_string())?;

    // Build languageInfo object from registry
    let lang_info_js = build_language_info_js(registry);

    // Build examples object from registry samples
    let examples_js = build_examples_js(registry);

    // Build icons object
    let icons_js = build_icons_js(icons);

    // Do replacements
    let output = app_js
        .replace(
            "// {{LANGUAGE_INFO}}",
            &format!("const languageInfo = {};", lang_info_js),
        )
        .replace("{{EXAMPLES}}", &examples_js)
        .replace("{{ICONS}}", &icons_js);

    fs::write(&output_path, &output).map_err(|e| e.to_string())?;
    Ok(())
}

fn build_language_info_js(registry: &Registry) -> String {
    // Build a JS object with language info keyed by id
    let mut js = String::from("{\n");
    for (i, grammar) in registry.grammars.iter().enumerate() {
        js.push_str(&format!("    \"{}\": {{\n", grammar.id));
        js.push_str(&format!(
            "        \"name\": \"{}\",\n",
            escape_for_js(&grammar.name)
        ));
        js.push_str(&format!(
            "        \"tag\": \"{}\",\n",
            escape_for_js(&grammar.tag)
        ));
        if let Some(ref icon) = grammar.icon {
            js.push_str(&format!("        \"icon\": \"{}\",\n", escape_for_js(icon)));
        }
        if let Some(tier) = grammar.tier {
            js.push_str(&format!("        \"tier\": {},\n", tier));
        }
        if let Some(ref desc) = grammar.description {
            let html = markdown_to_html(desc);
            js.push_str(&format!(
                "        \"description\": \"{}\",\n",
                escape_for_js(&html)
            ));
        }
        if let Some(ref inventor) = grammar.inventor {
            js.push_str(&format!(
                "        \"inventor\": \"{}\",\n",
                escape_for_js(inventor)
            ));
        }
        if let Some(year) = grammar.year {
            js.push_str(&format!("        \"year\": {},\n", year));
        }
        if let Some(ref link) = grammar.link {
            js.push_str(&format!("        \"url\": \"{}\",\n", escape_for_js(link)));
        }
        if let Some(ref trivia) = grammar.trivia {
            let html = markdown_to_html(trivia);
            js.push_str(&format!(
                "        \"trivia\": \"{}\",\n",
                escape_for_js(&html)
            ));
        }
        if !grammar.aliases.is_empty() {
            let aliases: Vec<String> = grammar
                .aliases
                .iter()
                .map(|a| format!("\"{}\"", escape_for_js(a)))
                .collect();
            js.push_str(&format!("        \"aliases\": [{}],\n", aliases.join(", ")));
        }
        // Remove trailing comma and newline, add just newline
        if js.ends_with(",\n") {
            js.pop();
            js.pop();
            js.push('\n');
        }
        js.push_str("    }");
        if i < registry.grammars.len() - 1 {
            js.push(',');
        }
        js.push('\n');
    }
    js.push('}');
    js
}

fn build_examples_js(registry: &Registry) -> String {
    // Build object mapping language id to file extension
    // Samples are fetched on demand from /samples/{id}.{ext}
    let mut js = String::from("{\n");
    let mut first = true;
    for grammar in &registry.grammars {
        if let Some(sample) = grammar.samples.first() {
            if !first {
                js.push_str(",\n");
            }
            first = false;
            // Get extension from sample path
            let ext = std::path::Path::new(&sample.path)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("txt");
            // Value is the file extension for fetching
            js.push_str(&format!("    \"{}\": \"{}\"", grammar.id, ext));
        }
    }
    js.push_str("\n}");
    js
}

fn build_icons_js(icons: &BTreeMap<String, String>) -> String {
    let mut js = String::from("{\n");
    for (i, (name, svg)) in icons.iter().enumerate() {
        js.push_str(&format!("    \"{}\": \"{}\"", name, escape_for_js(svg)));
        if i < icons.len() - 1 {
            js.push(',');
        }
        js.push('\n');
    }
    js.push('}');
    js
}

fn escape_for_js(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// Render markdown to HTML (inline, stripping outer `<p>` tags)
fn markdown_to_html(markdown: &str) -> String {
    let options = Options::empty();
    let parser = Parser::new_ext(markdown, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    // Strip outer `<p>...</p>` tags for inline use
    let trimmed = html_output.trim();
    if trimmed.starts_with("<p>") && trimmed.ends_with("</p>") {
        trimmed[3..trimmed.len() - 4].to_string()
    } else {
        trimmed.to_string()
    }
}

fn precompress_files(demo_dir: &Path) -> Result<(), String> {
    let files = ["index.html", "registry.json", "styles.css", "app.js"];
    let pkg_files = ["arborium_demo.js", "app.generated.js"];

    for file in files {
        let path = demo_dir.join(file);
        compress_file(&path)?;
    }

    for file in pkg_files {
        let path = demo_dir.join("pkg").join(file);
        compress_file(&path)?;
    }

    Ok(())
}

fn compress_file(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Ok(());
    }

    let data = fs::read(path).map_err(|e| e.to_string())?;

    // Brotli
    let br_path = PathBuf::from(format!("{}.br", path.display()));
    let br_data = compress_brotli(&data).map_err(|e| e.to_string())?;
    fs::write(&br_path, &br_data).map_err(|e| e.to_string())?;

    // Gzip
    let gz_path = PathBuf::from(format!("{}.gz", path.display()));
    let gz_data = compress_gzip(&data).map_err(|e| e.to_string())?;
    fs::write(&gz_path, &gz_data).map_err(|e| e.to_string())?;

    Ok(())
}

fn precompress_files_fast(demo_dir: &Path) -> Result<(), String> {
    let files = ["index.html", "registry.json", "styles.css", "app.js"];
    let pkg_files = ["arborium_demo.js", "app.generated.js"];

    for file in files {
        let path = demo_dir.join(file);
        compress_file_fast(&path)?;
    }

    for file in pkg_files {
        let path = demo_dir.join("pkg").join(file);
        compress_file_fast(&path)?;
    }

    Ok(())
}

fn compress_file_fast(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Ok(());
    }

    let data = fs::read(path).map_err(|e| e.to_string())?;

    // Brotli (fast)
    let br_path = PathBuf::from(format!("{}.br", path.display()));
    let br_data = compress_brotli_fast(&data).map_err(|e| e.to_string())?;
    fs::write(&br_path, &br_data).map_err(|e| e.to_string())?;

    // Gzip (fast)
    let gz_path = PathBuf::from(format!("{}.gz", path.display()));
    let gz_data = compress_gzip_fast(&data).map_err(|e| e.to_string())?;
    fs::write(&gz_path, &gz_data).map_err(|e| e.to_string())?;

    Ok(())
}

fn compress_brotli(data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    let mut output = Vec::new();
    {
        let mut encoder = brotli::CompressorWriter::new(&mut output, 4096, 5, 22);
        encoder.write_all(data)?;
    }
    Ok(output)
}

fn compress_brotli_fast(data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    let mut output = Vec::new();
    {
        let mut encoder = brotli::CompressorWriter::new(&mut output, 4096, 1, 22);
        encoder.write_all(data)?;
    }
    Ok(output)
}

fn compress_gzip(data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    use flate2::Compression;
    use flate2::write::GzEncoder;

    let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(data)?;
    encoder.finish()
}

fn compress_gzip_fast(data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    use flate2::Compression;
    use flate2::write::GzEncoder;

    let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
    encoder.write_all(data)?;
    encoder.finish()
}

fn bind_server(addr: &str, port: Option<u16>) -> (tiny_http::Server, u16) {
    if let Some(p) = port {
        match tiny_http::Server::http(format!("{}:{}", addr, p)) {
            Ok(s) => (s, p),
            Err(e) => {
                eprintln!("Error: Could not bind to {}:{}: {}", addr, p, e);
                std::process::exit(1);
            }
        }
    } else {
        // Try ports 8000-8010
        for p in 8000..=8010 {
            if let Ok(s) = tiny_http::Server::http(format!("{}:{}", addr, p)) {
                return (s, p);
            }
        }
        eprintln!("Error: Could not bind to any port between 8000-8010");
        std::process::exit(1);
    }
}

fn serve_files(server: tiny_http::Server, demo_dir: &Path) {
    for request in server.incoming_requests() {
        let url_path = request.url().trim_start_matches('/');

        let file_path = if url_path.is_empty() || url_path == "/" {
            demo_dir.join("index.html")
        } else {
            demo_dir.join(url_path)
        };

        // Security: ensure path is within demo_dir
        let file_path = match file_path.canonicalize() {
            Ok(p) => p,
            Err(_) => {
                let response = tiny_http::Response::from_string("Not Found").with_status_code(404);
                let _ = request.respond(response);
                continue;
            }
        };

        if !file_path.starts_with(demo_dir) {
            let response = tiny_http::Response::from_string("Forbidden").with_status_code(403);
            let _ = request.respond(response);
            continue;
        }

        // Check what encodings the client accepts
        let accept_encoding = request
            .headers()
            .iter()
            .find(|h| h.field.as_str().to_ascii_lowercase() == "accept-encoding")
            .map(|h| h.value.as_str().to_string())
            .unwrap_or_default();

        let accepts_br = accept_encoding.contains("br");
        let accepts_gzip = accept_encoding.contains("gzip");

        // Try to serve pre-compressed files
        let br_path = PathBuf::from(format!("{}.br", file_path.display()));
        let gz_path = PathBuf::from(format!("{}.gz", file_path.display()));
        let (serve_path, encoding) = if accepts_br && br_path.exists() {
            (br_path, Some("br"))
        } else if accepts_gzip && gz_path.exists() {
            (gz_path, Some("gzip"))
        } else {
            (file_path.clone(), None)
        };

        // Read and serve the file
        match fs::read(&serve_path) {
            Ok(content) => {
                let content_type = guess_content_type(&file_path);

                let mut response = tiny_http::Response::from_data(content).with_header(
                    tiny_http::Header::from_bytes(&b"Content-Type"[..], content_type.as_bytes())
                        .unwrap(),
                );

                if let Some(enc) = encoding {
                    response = response.with_header(
                        tiny_http::Header::from_bytes(&b"Content-Encoding"[..], enc.as_bytes())
                            .unwrap(),
                    );
                }

                let _ = request.respond(response);
            }
            Err(_) => {
                let response = tiny_http::Response::from_string("Not Found").with_status_code(404);
                let _ = request.respond(response);
            }
        }
    }
}

fn guess_content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") | Some("htm") => "text/html; charset=utf-8",
        Some("js") | Some("mjs") | Some("cjs") => "application/javascript; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        Some("wasm") => "application/wasm",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("ico") => "image/x-icon",
        Some("woff2") => "font/woff2",
        Some("woff") => "font/woff",
        // All source code files served as plain text
        _ => "text/plain; charset=utf-8",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util;

    #[test]
    fn test_registry_to_json() {
        let registry = Registry {
            grammars: vec![RegistryGrammar {
                id: "rust".to_string(),
                crate_name: "arborium-rust".to_string(),
                name: "Rust".to_string(),
                icon: Some("devicon-plain:rust".to_string()),
                tier: Some(1),
                tag: "code".to_string(),
                description: Some("Systems programming language".to_string()),
                inventor: Some("Graydon Hoare".to_string()),
                year: Some(2010),
                link: Some("https://www.rust-lang.org/".to_string()),
                trivia: Some("Originally a personal project".to_string()),
                aliases: vec!["rs".to_string()],
                samples: vec![RegistrySample {
                    path: "samples/example.rs".to_string(),
                    description: Some("Example code".to_string()),
                }],
            }],
        };

        let json = registry.to_json_pretty();
        assert!(json.contains("\"rust\""));
        assert!(json.contains("\"arborium-rust\""));
    }

    #[test]
    fn test_registry_from_crate_registry() {
        let repo_root = util::find_repo_root().expect("Could not find repo root");
        let crates_dir = repo_root.join("crates");
        let crates_dir = camino::Utf8PathBuf::from_path_buf(crates_dir).expect("non-UTF8 path");

        let crate_registry = CrateRegistry::load(&crates_dir).expect("Failed to load registry");
        let registry = Registry::from_crate_registry(&crate_registry, &crates_dir);

        // Should have many grammars
        assert!(
            registry.grammars.len() > 50,
            "Expected 50+ grammars, got {}",
            registry.grammars.len()
        );

        // Check a known grammar exists
        let rust = registry.grammars.iter().find(|g| g.id == "rust");
        assert!(rust.is_some(), "Rust grammar should exist");

        let rust = rust.unwrap();
        assert_eq!(rust.crate_name, "arborium-rust");
        assert_eq!(rust.name, "Rust");
        assert!(!rust.samples.is_empty(), "Rust should have samples");

        // Print stats
        let total_samples: usize = registry.grammars.iter().map(|g| g.samples.len()).sum();
        println!("Registry stats:");
        println!("  Grammars: {}", registry.grammars.len());
        println!("  Total samples: {}", total_samples);

        // Print JSON size
        let json = registry.to_json_pretty();
        println!("  JSON size: {} bytes", json.len());
    }
}
