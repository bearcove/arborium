//! <%= generated_disclaimer %>

#![cfg(test)]

use std::path::Path;

use <%= crate_name_snake %> as grammar;
use libtest_mimic::{run, Arguments, Failed, Trial};

fn main() {
    let args = Arguments::from_args();
    let cases = arborium_test_harness::collect_corpus_cases(env!("CARGO_MANIFEST_DIR"))
        .expect("failed to collect corpus cases");

    let tests: Vec<Trial> = cases
        .into_iter()
        .map(|case| {
            let relative = case
                .file
                .strip_prefix(Path::new(env!("CARGO_MANIFEST_DIR")))
                .unwrap_or(case.file.as_path());
            let name = format!("{}::{}", relative.display(), case.name);
            let display = name.clone();
            Trial::test(name, move || {
                let sexp = arborium_test_harness::run_corpus_case_with_tree(
                    grammar::language(),
                    "<%= grammar_id %>",
                    &case,
                )
                .map_err(|err| Failed::from(err.to_string()))?;
                println!("=== corpus::{display} ===\n{sexp}\n");
                Ok(())
            })
            .with_kind("corpus")
        })
        .collect();

    run(&args, tests).exit();
}
