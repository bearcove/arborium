//! Test harness for arborium grammar crates.
//!
//! This crate provides utilities for testing tree-sitter grammars and their queries.
//!
//! # Usage
//!
//! In your grammar crate's lib.rs tests:
//!
//! ```ignore
//! #[cfg(test)]
//! mod tests {
//!     use super::*;
//!
//!     #[test]
//!     fn test_grammar() {
//!         arborium_test_harness::test_grammar(
//!             language(),
//!             "rust",
//!             HIGHLIGHTS_QUERY,
//!             INJECTIONS_QUERY,
//!             LOCALS_QUERY,
//!             env!("CARGO_MANIFEST_DIR"),
//!         );
//!     }
//! }
//! ```

pub use arborium_highlight;
pub use arborium_tree_sitter as tree_sitter;

use std::collections::HashSet;
use std::fs;
use std::path::Path;

use arborium_highlight::{CompiledGrammar, GrammarConfig, ParseContext};
use arborium_tree_sitter::Language;
use arborium_tree_sitter::{Node, Parser};

// Re-export CAPTURE_NAMES from arborium-theme as HIGHLIGHT_NAMES for convenience
pub use arborium_theme::CAPTURE_NAMES as HIGHLIGHT_NAMES_FULL;

#[derive(Debug, Default)]
struct CorpusTest {
    name: String,
    input: String,
    contains: Vec<String>,
    expected_sexp: Option<String>,
}

/// Tests a grammar by validating its queries and highlighting all samples.
///
/// This function:
/// 1. Validates that the queries compile correctly
/// 2. Finds sample files in the samples/ directory
/// 3. Highlights each sample file and verifies we get highlights
///
/// # Arguments
///
/// * `language` - The tree-sitter Language
/// * `name` - The grammar name (e.g., "rust")
/// * `highlights_query` - The highlights.scm content
/// * `injections_query` - The injections.scm content
/// * `locals_query` - The locals.scm content (currently unused by arborium-highlight)
/// * `crate_dir` - Path to the crate directory (use `env!("CARGO_MANIFEST_DIR")`)
///
/// # Panics
///
/// Panics if query validation fails, highlighting produces errors, or no highlights are found.
pub fn test_grammar(
    language: impl Into<Language>,
    name: &str,
    highlights_query: &str,
    injections_query: &str,
    _locals_query: &str,
    crate_dir: &str,
) {
    let language: Language = language.into();
    // Create grammar config
    let config = GrammarConfig {
        language,
        highlights_query,
        injections_query,
        locals_query: "", // Not used by arborium-highlight yet
    };

    // Validate queries compile by creating the grammar
    let grammar = CompiledGrammar::new(config).unwrap_or_else(|e| {
        panic!(
            "Query validation failed for {}: {:?}\n\
             This usually means highlights.scm references a node type that doesn't exist in the grammar.\n\
             Check the grammar's node-types.json to see valid node types.",
            name, e
        );
    });

    // Create a parse context for this grammar
    let mut ctx = ParseContext::for_grammar(&grammar).unwrap_or_else(|e| {
        panic!("Failed to create parse context for {}: {:?}", name, e);
    });

    // Find samples from arborium.kdl
    let crate_path = Path::new(crate_dir);
    let kdl_path = crate_path.join("arborium.kdl");
    let samples: Vec<_> = if kdl_path.exists() {
        parse_samples_from_kdl(&kdl_path)
            .into_iter()
            .map(|p| crate_path.join(p))
            .collect()
    } else {
        vec![]
    };

    if samples.is_empty() {
        // No samples - just verify query compiles (already done above)
        return;
    }

    // Test each sample - must produce at least one highlight
    for sample_path in &samples {
        let sample_code = fs::read_to_string(sample_path).unwrap_or_else(|e| {
            panic!(
                "Failed to read sample file {} for {}: {}",
                sample_path.display(),
                name,
                e
            );
        });

        // Parse with the grammar
        let result = grammar.parse(&mut ctx, &sample_code);

        // Count highlight spans
        let highlight_count = result.spans.len();

        // Verify we got highlights
        if highlight_count == 0 {
            panic!(
                "No highlights produced for {} in {}.\n\
                 Sample has {} bytes.\n\
                 This likely means the highlights.scm query doesn't match anything in the sample.",
                sample_path.display(),
                name,
                sample_code.len()
            );
        }
    }
}

/// Runs corpus-style parsing tests for a grammar.
///
/// The harness looks for a `corpus/` directory at the crate root and reads all
/// `*.txt` files in it. Each file contains one or more test cases in a simple
/// format:
///
/// ```text
/// === test name
/// --- input
/// node 1;
/// --- contains
/// raw_string
/// quoted_string
/// --- sexp
/// (document ...)
/// ```
///
/// Only `input` is required. `contains` and `sexp` are optional:
/// - `contains`: node kinds that must appear at least once in the parse tree.
/// - `sexp`: expected root s-expression (exact match).
///
/// This does **not** use `tree-sitter test`; it's a lightweight Rust runner.
pub fn test_corpus(language: impl Into<Language>, name: &str, crate_dir: &str) {
    let language: Language = language.into();
    let crate_path = Path::new(crate_dir);
    let corpus_dir = crate_path.join("corpus");
    if !corpus_dir.exists() {
        return;
    }

    let mut parser = Parser::new();
    parser
        .set_language(&language)
        .unwrap_or_else(|e| panic!("Failed to set language for {}: {:?}", name, e));

    let mut entries: Vec<_> = fs::read_dir(&corpus_dir)
        .unwrap_or_else(|e| {
            panic!(
                "Failed to read corpus dir for {}: {:?} ({})",
                name,
                e,
                corpus_dir.display()
            )
        })
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_file() && p.extension().is_some_and(|ext| ext == "txt"))
        .collect();
    entries.sort();

    if entries.is_empty() {
        return;
    }

    for path in entries {
        let content = fs::read_to_string(&path).unwrap_or_else(|e| {
            panic!(
                "Failed to read corpus file {} for {}: {}",
                path.display(),
                name,
                e
            );
        });

        let tests = parse_corpus(&content).unwrap_or_else(|e| {
            panic!(
                "Failed to parse corpus file {} for {}: {}",
                path.display(),
                name,
                e
            );
        });

        if tests.is_empty() {
            panic!(
                "Corpus file {} for {} contains no tests",
                path.display(),
                name
            );
        }

        for test in tests {
            let tree = parser.parse(&test.input, None).unwrap_or_else(|| {
                panic!(
                    "Parser returned no tree for {} / {} (file {})",
                    name,
                    test.name,
                    path.display()
                )
            });

            let root = tree.root_node();
            if root.has_error() {
                panic!(
                    "Parse errors for {} / {} (file {})\n--- input ---\n{}\n--- sexp ---\n{}",
                    name,
                    test.name,
                    path.display(),
                    test.input,
                    root.to_sexp(),
                );
            }

            if let Some(expected) = &test.expected_sexp {
                let actual = root.to_sexp();
                if actual.trim() != expected.trim() {
                    panic!(
                        "S-expression mismatch for {} / {} (file {})\n--- input ---\n{}\n--- expected ---\n{}\n--- actual ---\n{}",
                        name,
                        test.name,
                        path.display(),
                        test.input,
                        expected,
                        actual
                    );
                }
            }

            if !test.contains.is_empty() {
                let mut seen: HashSet<&str> = HashSet::new();
                collect_kinds(root, &mut seen);

                for kind in &test.contains {
                    if !seen.contains(kind.as_str()) {
                        panic!(
                            "Expected node kind `{}` not found for {} / {} (file {})\n--- input ---\n{}\n--- seen ---\n{:?}\n--- sexp ---\n{}",
                            kind,
                            name,
                            test.name,
                            path.display(),
                            test.input,
                            seen,
                            root.to_sexp()
                        );
                    }
                }
            }
        }
    }
}

fn collect_kinds(node: Node, out: &mut HashSet<&str>) {
    out.insert(node.kind());
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_kinds(child, out);
    }
}

fn parse_corpus(content: &str) -> Result<Vec<CorpusTest>, String> {
    let mut tests: Vec<CorpusTest> = Vec::new();
    let mut current: Option<CorpusTest> = None;
    let mut section: Option<String> = None;

    for (idx, line) in content.lines().enumerate() {
        let trimmed = line.trim_end();

        if let Some(name) = trimmed.strip_prefix("===") {
            if let Some(t) = current.take() {
                tests.push(t);
            }
            current = Some(CorpusTest {
                name: name.trim().to_string(),
                ..CorpusTest::default()
            });
            section = None;
            continue;
        }

        if let Some(sec) = trimmed.strip_prefix("---") {
            section = Some(sec.trim().to_string());
            continue;
        }

        let Some(test) = current.as_mut() else {
            // Allow blank lines and comments before first test.
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            return Err(format!(
                "Unexpected content before first test at line {}: {}",
                idx + 1,
                trimmed
            ));
        };

        match section.as_deref() {
            Some("input") => {
                test.input.push_str(line);
                test.input.push('\n');
            }
            Some("sexp") => {
                let expected = test.expected_sexp.get_or_insert_with(String::new);
                expected.push_str(line);
                expected.push('\n');
            }
            Some("contains") => {
                for tok in trimmed.split_whitespace() {
                    test.contains.push(tok.to_string());
                }
            }
            Some(other) => {
                return Err(format!("Unknown section `{}` at line {}", other, idx + 1));
            }
            None => {
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    continue;
                }
                return Err(format!(
                    "Content outside a section at line {}: {}",
                    idx + 1,
                    trimmed
                ));
            }
        }
    }

    if let Some(t) = current.take() {
        tests.push(t);
    }

    Ok(tests)
}

/// Parse sample paths from arborium.kdl
///
/// Looks for `sample { path "..." }` blocks and extracts the path values.
fn parse_samples_from_kdl(path: &Path) -> Vec<String> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    let mut samples = Vec::new();
    let mut in_sample_block = false;
    let mut brace_depth = 0;

    for line in content.lines() {
        let trimmed = line.trim();

        // Track sample blocks
        if trimmed.starts_with("sample") && trimmed.contains('{') {
            in_sample_block = true;
            brace_depth = 1;
            continue;
        }

        if in_sample_block {
            // Track brace depth
            brace_depth += trimmed.matches('{').count();
            brace_depth = brace_depth.saturating_sub(trimmed.matches('}').count());

            if brace_depth == 0 {
                in_sample_block = false;
                continue;
            }

            // Look for path "..."
            if trimmed.starts_with("path")
                && let Some(start) = trimmed.find('"')
                && let Some(end) = trimmed[start + 1..].find('"')
            {
                let path_value = &trimmed[start + 1..start + 1 + end];
                if !path_value.is_empty() {
                    samples.push(path_value.to_string());
                }
            }
        }
    }

    samples
}

/// Standard highlight names used by arborium.
///
/// **Deprecated**: Use [`arborium_theme::CAPTURE_NAMES`] instead, which is the
/// canonical source of truth for all capture names.
///
/// This constant is kept for backwards compatibility.
pub const HIGHLIGHT_NAMES: &[&str] = arborium_theme::CAPTURE_NAMES;
