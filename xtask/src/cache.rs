//! Grammar generation cache.
//!
//! This module provides caching for tree-sitter grammar generation.
//! Each grammar's generated files (parser.c, etc.) are cached based on
//! a blake3 hash of all input files (grammar.js, common/, etc.).

use atomicwrites::{AtomicFile, OverwriteBehavior};
use camino::{Utf8Path, Utf8PathBuf};
use fs_err as fs;
use std::io::{Read, Write};

/// The cache directory relative to repo root.
const CACHE_DIR: &str = ".cache/arborium";

/// Represents a grammar generation cache.
pub struct GrammarCache {
    cache_dir: Utf8PathBuf,
}

impl GrammarCache {
    /// Create a new grammar cache.
    pub fn new(repo_root: &Utf8Path) -> Self {
        Self {
            cache_dir: repo_root.join(CACHE_DIR),
        }
    }

    /// Compute the cache key for a grammar.
    ///
    /// The cache key is a blake3 hash of all input files that affect generation:
    /// - grammar/grammar.js
    /// - grammar/package.json (if exists)
    /// - common/* (if exists)
    /// - Any files in grammar/ that aren't in src/ (scanner sources, etc.)
    pub fn compute_cache_key(
        &self,
        crate_path: &Utf8Path,
        crates_dir: &Utf8Path,
        crate_name: &str,
    ) -> std::io::Result<String> {
        let mut hasher = blake3::Hasher::new();

        let grammar_dir = crate_path.join("grammar");

        // Hash grammar.js (the main input)
        self.hash_file(&mut hasher, &grammar_dir.join("grammar.js"))?;

        // Hash package.json if it exists
        let package_json = grammar_dir.join("package.json");
        if package_json.exists() {
            self.hash_file(&mut hasher, &package_json)?;
        }

        // Hash all files in grammar/ except src/ directory
        self.hash_dir_except(&mut hasher, &grammar_dir, &["src", "node_modules"])?;

        // Hash common/ directory if it exists
        let common_dir = crate_path.join("common");
        if common_dir.exists() {
            self.hash_dir_recursive(&mut hasher, &common_dir)?;
        }

        // Hash dependency grammars (for cross-grammar dependencies)
        let deps = get_grammar_dependencies(crate_name);
        for (_npm_name, arborium_name) in deps {
            let dep_grammar_dir = crates_dir.join(arborium_name).join("grammar");
            if dep_grammar_dir.exists() {
                self.hash_dir_except(&mut hasher, &dep_grammar_dir, &["src", "node_modules"])?;
            }
        }

        Ok(hasher.finalize().to_hex().to_string())
    }

    /// Check if we have a cached result for the given key.
    pub fn get(&self, crate_name: &str, cache_key: &str) -> Option<CachedGrammar> {
        let cache_path = self.cache_path(crate_name, cache_key);
        if cache_path.exists() {
            Some(CachedGrammar { path: cache_path })
        } else {
            None
        }
    }

    /// Save generated files to cache.
    pub fn save(
        &self,
        crate_name: &str,
        cache_key: &str,
        generated_src: &Utf8Path,
    ) -> std::io::Result<()> {
        let cache_path = self.cache_path(crate_name, cache_key);

        // Create cache directory
        if let Some(parent) = cache_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Create a tarball atomically (write to temp, then rename)
        let atomic_file = AtomicFile::new(&cache_path, OverwriteBehavior::AllowOverwrite);
        atomic_file.write(|file| {
            let encoder = flate2::write::GzEncoder::new(file, flate2::Compression::fast());
            let mut tar = tar::Builder::new(encoder);

            // Add all files from generated_src
            Self::add_dir_to_tar(&mut tar, generated_src, Utf8Path::new(""))?;

            tar.finish()?;
            Ok(())
        })?;

        Ok(())
    }

    fn cache_path(&self, crate_name: &str, cache_key: &str) -> Utf8PathBuf {
        // Use first 16 chars of hash for shorter filenames
        let short_key = &cache_key[..16.min(cache_key.len())];
        self.cache_dir
            .join(crate_name)
            .join(format!("{}.tar.gz", short_key))
    }

    fn hash_file(&self, hasher: &mut blake3::Hasher, path: &Utf8Path) -> std::io::Result<()> {
        // Include the filename in the hash (so renames are detected)
        if let Some(name) = path.file_name() {
            hasher.update(name.as_bytes());
            hasher.update(b"\0");
        }

        let mut file = std::fs::File::open(path)?;
        let mut buffer = [0u8; 8192];
        loop {
            let n = file.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }
        Ok(())
    }

    fn hash_dir_recursive(
        &self,
        hasher: &mut blake3::Hasher,
        dir: &Utf8Path,
    ) -> std::io::Result<()> {
        self.hash_dir_except(hasher, dir, &[])
    }

    fn hash_dir_except(
        &self,
        hasher: &mut blake3::Hasher,
        dir: &Utf8Path,
        exclude: &[&str],
    ) -> std::io::Result<()> {
        if !dir.exists() {
            return Ok(());
        }

        // Collect and sort entries for deterministic hashing
        let mut entries: Vec<_> = fs::read_dir(dir)?.filter_map(|e| e.ok()).collect();
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let name = entry.file_name().to_string_lossy().to_string();

            // Skip excluded directories
            if exclude.contains(&name.as_str()) {
                continue;
            }

            let path = Utf8PathBuf::from_path_buf(entry.path())
                .map_err(|_| std::io::Error::other("Non-UTF8 path"))?;

            if path.is_dir() {
                // Recursively hash subdirectories
                self.hash_dir_recursive(hasher, &path)?;
            } else if path.is_file() {
                self.hash_file(hasher, &path)?;
            }
        }

        Ok(())
    }

    fn add_dir_to_tar<W: Write>(
        tar: &mut tar::Builder<W>,
        src_dir: &Utf8Path,
        prefix: &Utf8Path,
    ) -> std::io::Result<()> {
        for entry in fs::read_dir(src_dir)? {
            let entry = entry?;
            let path = Utf8PathBuf::from_path_buf(entry.path())
                .map_err(|_| std::io::Error::other("Non-UTF8 path"))?;
            let name = entry.file_name().to_string_lossy().to_string();
            let tar_path = prefix.join(&name);

            if path.is_dir() {
                Self::add_dir_to_tar(tar, &path, &tar_path)?;
            } else if path.is_file() {
                let mut file = std::fs::File::open(&path)?;
                tar.append_file(tar_path.as_str(), &mut file)?;
            }
        }
        Ok(())
    }
}

/// A cached grammar that can be restored.
pub struct CachedGrammar {
    path: Utf8PathBuf,
}

impl CachedGrammar {
    /// Extract the cached grammar to the destination directory.
    pub fn extract_to(&self, dest_dir: &Utf8Path) -> std::io::Result<()> {
        // Ensure destination exists
        fs::create_dir_all(dest_dir)?;

        // Extract tarball
        let file = std::fs::File::open(&self.path)?;
        let decoder = flate2::read::GzDecoder::new(file);
        let mut archive = tar::Archive::new(decoder);

        archive.unpack(dest_dir)?;
        Ok(())
    }
}

/// Get the cross-grammar dependencies for a grammar.
/// Duplicated from generate.rs to avoid circular dependencies.
fn get_grammar_dependencies(crate_name: &str) -> Vec<(&'static str, &'static str)> {
    match crate_name {
        "arborium-typescript" | "arborium-tsx" => {
            vec![("tree-sitter-javascript", "arborium-javascript")]
        }
        "arborium-cpp" => vec![("tree-sitter-c", "arborium-c")],
        "arborium-objc" => vec![("tree-sitter-c", "arborium-c")],
        "arborium-glsl" => vec![("tree-sitter-c", "arborium-c")],
        "arborium-hlsl" => vec![
            ("tree-sitter-cpp", "arborium-cpp"),
            ("tree-sitter-c", "arborium-c"),
        ],
        "arborium-scss" => vec![("tree-sitter-css", "arborium-css")],
        "arborium-svelte" => vec![("tree-sitter-html", "arborium-html")],
        "arborium-vue" => vec![("tree-sitter-html", "arborium-html")],
        "arborium-commonlisp" => vec![("tree-sitter-clojure", "arborium-clojure")],
        _ => vec![],
    }
}
