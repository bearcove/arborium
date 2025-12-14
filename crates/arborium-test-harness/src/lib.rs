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
use std::path::{Path, PathBuf};

use arborium_highlight::{CompiledGrammar, GrammarConfig, ParseContext};
use arborium_tree_sitter::Language;
use arborium_tree_sitter::{Node, Parser, Tree};
use tree_sitter_language::LanguageFn;

// Re-export CAPTURE_NAMES from arborium-theme as HIGHLIGHT_NAMES for convenience
pub use arborium_theme::CAPTURE_NAMES as HIGHLIGHT_NAMES_FULL;

#[derive(Debug, Default)]
struct CorpusTest {
    name: String,
    input: String,
    contains: Vec<String>,
    expected_sexp: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CorpusCase {
    pub file: PathBuf,
    pub name: String,
    pub input: String,
    pub contains: Vec<String>,
    pub expected_sexp: Option<String>,
}

#[derive(Debug)]
pub struct HarnessError {
    message: String,
}

impl HarnessError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for HarnessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for HarnessError {}

type HarnessResult<T = ()> = Result<T, HarnessError>;

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
pub fn test_corpus(language: LanguageFn, name: &str, crate_dir: &str) {
    let cases = collect_corpus_cases(crate_dir).unwrap_or_else(|e| {
        panic!(
            "Failed to gather corpus cases for {} (crate dir {}): {}",
            name, crate_dir, e
        )
    });

    for case in &cases {
        if let Err(err) = run_corpus_case(language, name, case) {
            panic!(
                "Corpus failure for {} / {} (file {}): {}",
                name,
                case.name,
                case.file.display(),
                err
            );
        }
    }
}

/// Return all `.txt` corpus files for a grammar crate.
pub fn corpus_files(crate_dir: &str) -> Vec<PathBuf> {
    let crate_path = Path::new(crate_dir);
    let corpus_dir = crate_path.join("corpus");
    if !corpus_dir.exists() {
        return Vec::new();
    }

    let mut entries: Vec<_> = match fs::read_dir(&corpus_dir) {
        Ok(read_dir) => read_dir
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file() && p.extension().is_some_and(|ext| ext == "txt"))
            .collect(),
        Err(_) => Vec::new(),
    };
    entries.sort();
    entries
}

/// Parse every corpus file and yield a case per `=== test`.
pub fn collect_corpus_cases(crate_dir: &str) -> HarnessResult<Vec<CorpusCase>> {
    let files = corpus_files(crate_dir);
    if files.is_empty() {
        return Ok(Vec::new());
    }

    let mut cases = Vec::new();
    for path in files {
        let content = fs::read_to_string(&path).map_err(|e| {
            HarnessError::new(format!(
                "Failed to read corpus file {}: {}",
                path.display(),
                e
            ))
        })?;

        let tests = parse_corpus(&content).map_err(|e| {
            HarnessError::new(format!(
                "Failed to parse corpus file {}: {}",
                path.display(),
                e
            ))
        })?;

        if tests.is_empty() {
            return Err(HarnessError::new(format!(
                "Corpus file {} contains no tests",
                path.display()
            )));
        }

        for test in tests {
            cases.push(CorpusCase {
                file: path.clone(),
                name: test.name,
                input: test.input,
                contains: test.contains,
                expected_sexp: test.expected_sexp,
            });
        }
    }

    Ok(cases)
}

/// Execute all tests defined in a single corpus file.
pub fn run_corpus_file(language: LanguageFn, name: &str, path: &Path) -> HarnessResult<()> {
    let content = fs::read_to_string(path).map_err(|e| {
        HarnessError::new(format!(
            "Failed to read corpus file {} for {}: {}",
            path.display(),
            name,
            e
        ))
    })?;

    let tests = parse_corpus(&content).map_err(|e| {
        HarnessError::new(format!(
            "Failed to parse corpus file {} for {}: {}",
            path.display(),
            name,
            e
        ))
    })?;

    if tests.is_empty() {
        return Err(HarnessError::new(format!(
            "Corpus file {} for {} contains no tests",
            path.display(),
            name
        )));
    }

    for test in tests {
        let case = CorpusCase {
            file: path.to_path_buf(),
            name: test.name,
            input: test.input,
            contains: test.contains,
            expected_sexp: test.expected_sexp,
        };
        run_corpus_case(language, name, &case)?;
    }

    Ok(())
}

/// Execute a single corpus test case.
pub fn run_corpus_case(language: LanguageFn, name: &str, case: &CorpusCase) -> HarnessResult<()> {
    run_corpus_case_with_tree(language, name, case).map(|_| ())
}

/// Run a corpus test case and return the parsed tree's s-expression.
pub fn run_corpus_case_with_tree(
    language: LanguageFn,
    name: &str,
    case: &CorpusCase,
) -> HarnessResult<String> {
    let tree = parse_case(language, name, case)?;
    let root = tree.root_node();

    if let Some(expected) = &case.expected_sexp {
        let actual = root.to_sexp();
        if actual.trim() != expected.trim() {
            return Err(HarnessError::new(format!(
                "S-expression mismatch for {} / {} (file {})\n--- input ---\n{}\n--- expected ---\n{}\n--- actual ---\n{}",
                name,
                case.name,
                case.file.display(),
                case.input,
                expected,
                actual
            )));
        }
    }

    if !case.contains.is_empty() {
        let mut seen: HashSet<&str> = HashSet::new();
        collect_kinds(root, &mut seen);

        for kind in &case.contains {
            if !seen.contains(kind.as_str()) {
                return Err(HarnessError::new(format!(
                    "Expected node kind `{}` not found for {} / {} (file {})\n--- input ---\n{}\n--- seen ---\n{:?}\n--- sexp ---\n{}",
                    kind,
                    name,
                    case.name,
                    case.file.display(),
                    case.input,
                    seen,
                    root.to_sexp()
                )));
            }
        }
    }

    Ok(root.to_sexp())
}

fn parse_case(language: LanguageFn, name: &str, case: &CorpusCase) -> HarnessResult<Tree> {
    if case.input.trim().is_empty() {
        return Err(HarnessError::new(format!(
            "Corpus test {} / {} (file {}) is missing an `--- input` section",
            name,
            case.name,
            case.file.display()
        )));
    }

    let language = Language::from(language);
    let mut parser = Parser::new();
    parser
        .set_language(&language)
        .map_err(|e| HarnessError::new(format!("Failed to set language for {}: {:?}", name, e)))?;

    let tree = parser.parse(&case.input, None).ok_or_else(|| {
        HarnessError::new(format!(
            "Parser returned no tree for {} / {} (file {})",
            name,
            case.name,
            case.file.display()
        ))
    })?;

    let root = tree.root_node();
    if root.has_error() {
        return Err(HarnessError::new(format!(
            "Parse errors for {} / {} (file {})\n--- input ---\n{}\n--- sexp ---\n{}",
            name,
            case.name,
            case.file.display(),
            case.input,
            root.to_sexp()
        )));
    }

    Ok(tree)
}

fn collect_kinds(node: Node, out: &mut HashSet<&str>) {
    out.insert(node.kind());
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_kinds(child, out);
    }
}

fn parse_corpus(content: &str) -> HarnessResult<Vec<CorpusTest>> {
    let mut tests: Vec<CorpusTest> = Vec::new();
    let mut current: Option<CorpusTest> = None;
    let mut section: Option<String> = None;

    for (idx, chunk) in content.split_inclusive('\n').enumerate() {
        let line = chunk
            .strip_suffix('\n')
            .map(|l| l.strip_suffix('\r').unwrap_or(l))
            .unwrap_or(chunk);
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
            return Err(HarnessError::new(format!(
                "Unexpected content before first test at line {}: {}",
                idx + 1,
                trimmed
            )));
        };

        match section.as_deref() {
            Some("input") => test.input.push_str(chunk),
            Some("sexp") => {
                let expected = test.expected_sexp.get_or_insert_with(String::new);
                expected.push_str(chunk);
            }
            Some("contains") => {
                for tok in trimmed.split_whitespace() {
                    test.contains.push(tok.to_string());
                }
            }
            Some(other) => {
                return Err(HarnessError::new(format!(
                    "Unknown section `{}` at line {}",
                    other,
                    idx + 1
                )));
            }
            None => {
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    continue;
                }
                return Err(HarnessError::new(format!(
                    "Content outside a section at line {}: {}",
                    idx + 1,
                    trimmed
                )));
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
