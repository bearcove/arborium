use arborium::theme::builtin;
use arborium::{AnsiHighlighter, Highlighter};
use facet::Facet;
use facet_args as args;
use std::io::{self, Read};
use std::path::Path;

/// Arborium syntax highlighter - terminal-friendly code highlighting
#[derive(Debug, Facet)]
struct Args {
    /// Language to highlight (e.g., rust, python, javascript)
    ///
    /// If omitted, language is auto-detected from filename or content
    #[facet(args::named, args::short = 'l', default)]
    lang: Option<String>,

    /// Output HTML instead of ANSI escape sequences
    #[facet(args::named, default)]
    html: bool,

    /// Input: code string, filename, or '-' for stdin
    ///
    /// If a file path is provided, reads from that file.
    /// If '-' is provided, reads from stdin.
    /// Otherwise, treats the argument as raw code to highlight.
    #[facet(args::positional, default)]
    input: Option<String>,

    /// Theme for ANSI output (ignored with --html)
    #[facet(args::named, default)]
    theme: Option<String>,
}

fn main() {
    let args: Args = facet_args::from_std_args().unwrap_or_else(|e| {
        eprintln!("Error: {:?}", e);
        std::process::exit(1);
    });

    if let Err(e) = run(args) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run(args: Args) -> Result<(), String> {
    // Determine input source and read content
    let (content, filename) = match args.input.as_deref() {
        None | Some("-") => {
            // Read from stdin
            let mut buffer = String::new();
            io::stdin()
                .read_to_string(&mut buffer)
                .map_err(|e| format!("Failed to read stdin: {}", e))?;
            (buffer, None)
        }
        Some(input) => {
            // Check if input is a file path
            let path = Path::new(input);
            if path.exists() && path.is_file() {
                let content = std::fs::read_to_string(path)
                    .map_err(|e| format!("Failed to read file '{}': {}", input, e))?;
                (content, Some(input.to_string()))
            } else {
                // Treat as literal code string
                (input.to_string(), None)
            }
        }
    };

    // Detect language
    let detected_lang = if let Some(lang) = &args.lang {
        Some(lang.as_str())
    } else if let Some(filename) = &filename {
        arborium::detect_language(filename)
    } else {
        // Try to detect from content (shebang)
        detect_from_content(&content)
    };

    let lang = detected_lang.ok_or_else(|| {
        if args.lang.is_some() {
            format!("Unknown language: {}", args.lang.as_ref().unwrap())
        } else if let Some(filename) = &filename {
            format!(
                "Could not detect language from filename: {}. Use --lang to specify.",
                filename
            )
        } else {
            "Could not detect language. Use --lang to specify.".to_string()
        }
    })?;

    // Highlight based on output format
    if args.html {
        let mut highlighter = Highlighter::new();
        let html = highlighter
            .highlight(lang, &content)
            .map_err(|e| format!("Highlighting failed: {}", e))?;
        println!("{}", html);
    } else {
        // Determine theme
        let theme = match args.theme.as_deref() {
            Some("mocha") | Some("catppuccin-mocha") => builtin::catppuccin_mocha(),
            Some("latte") | Some("catppuccin-latte") => builtin::catppuccin_latte(),
            Some("macchiato") | Some("catppuccin-macchiato") => builtin::catppuccin_macchiato(),
            Some("frappe") | Some("catppuccin-frappe") => builtin::catppuccin_frappe(),
            Some("dracula") => builtin::dracula(),
            Some("tokyo-night") => builtin::tokyo_night(),
            Some("nord") => builtin::nord(),
            Some("one-dark") => builtin::one_dark(),
            Some("github-dark") => builtin::github_dark(),
            Some("github-light") => builtin::github_light(),
            Some("gruvbox-dark") => builtin::gruvbox_dark(),
            Some("gruvbox-light") => builtin::gruvbox_light(),
            Some(other) => {
                return Err(format!("Unknown theme: {}", other));
            }
            None => builtin::catppuccin_mocha(), // Default theme
        };

        let mut highlighter = AnsiHighlighter::new(theme.clone());
        let ansi = highlighter
            .highlight(lang, &content)
            .map_err(|e| format!("Highlighting failed: {}", e))?;
        println!("{}", ansi);
    }

    Ok(())
}

/// Detect language from content (e.g., shebang lines)
fn detect_from_content(content: &str) -> Option<&'static str> {
    let first_line = content.lines().next()?;

    // Check for shebang
    if let Some(shebang) = first_line.strip_prefix("#!") {
        let shebang = shebang.trim();

        // Common interpreters
        if shebang.contains("python") {
            return Some("python");
        } else if shebang.contains("node") || shebang.contains("nodejs") {
            return Some("javascript");
        } else if shebang.contains("ruby") {
            return Some("ruby");
        } else if shebang.contains("perl") {
            return Some("perl");
        } else if shebang.contains("bash") || shebang.contains("/sh") {
            return Some("bash");
        } else if shebang.contains("zsh") {
            return Some("zsh");
        } else if shebang.contains("fish") {
            return Some("fish");
        } else if shebang.contains("php") {
            return Some("php");
        }
    }

    None
}
