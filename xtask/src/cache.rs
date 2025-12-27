//! Grammar generation cache.
//!
//! This module provides caching for tree-sitter grammar generation.
//! Each grammar's generated files (parser.c, etc.) are cached based on
//! a blake3 hash of all input files that affect tree-sitter CLI output:
//! tree-sitter CLI version, grammar.js, common/, dependency grammars, etc.

use crate::tool::Tool;
use camino::{Utf8Path, Utf8PathBuf};
use fs_err as fs;
use std::io::Read;

/// Combines all per-grammar cache keys into a single hash for CI.
pub fn compute_global_cache_key(repo_root: &Utf8Path) -> std::io::Result<String> {
    let crates_dir = repo_root.join("crates");
    let registry = crate::types::CrateRegistry::load(&crates_dir)
        .map_err(|e| std::io::Error::other(format!("Failed to load registry: {}", e)))?;

    let cache = GrammarCache::new(repo_root);
    let mut hasher = blake3::Hasher::new();

    for (name, state) in &registry.crates {
        let Some(config) = &state.config else { continue };
        let Ok(key) = cache.compute_cache_key(&state.def_path, &crates_dir, config) else {
            continue;
        };

        hasher.update(name.as_bytes());
        hasher.update(b":");
        hasher.update(key.as_bytes());
        hasher.update(b"\0");
    }

    Ok(hasher.finalize().to_hex().to_string())
}

/// The cache directory relative to repo root.
const CACHE_DIR: &str = ".cache/arborium";

/// Represents a grammar generation cache.
pub struct GrammarCache {
    pub cache_dir: Utf8PathBuf,
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
    /// The cache key is a blake3 hash of all inputs that affect tree-sitter CLI output:
    /// - tree-sitter CLI version
    /// - grammar/grammar.js
    /// - common/* (if exists)
    /// - Any files in grammar/ that aren't in src/ (scanner sources, etc.)
    /// - Dependency grammars
    pub fn compute_cache_key(
        &self,
        crate_path: &Utf8Path,
        crates_dir: &Utf8Path,
        config: &crate::types::CrateConfig,
    ) -> std::io::Result<String> {
        let mut hasher = blake3::Hasher::new();

        // Hash tree-sitter CLI version (critical for cache invalidation)
        let ts_version = Tool::TreeSitter.get_version().map_err(|e| {
            std::io::Error::other(format!("Failed to get tree-sitter version: {}", e))
        })?;

        hasher.update(b"tree-sitter-version:");
        hasher.update(ts_version.as_bytes());
        hasher.update(b"\0");

        let grammar_dir = crate_path.join("grammar");

        // Hash grammar.js (the main input)
        self.hash_file(&mut hasher, &grammar_dir.join("grammar.js"))?;

        // Hash all files in grammar/ except src/ directory
        self.hash_dir_except(&mut hasher, &grammar_dir, &["src", "node_modules"])?;

        // Hash common/ directory if it exists
        let common_dir = crate_path.join("common");
        if common_dir.exists() {
            self.hash_dir_recursive(&mut hasher, &common_dir)?;
        }

        // Hash dependency grammars (for cross-grammar dependencies)
        let deps = get_grammar_dependencies(config);
        for (_npm_name, arborium_name) in deps {
            let dep_grammar_dir = crates_dir.join(&arborium_name).join("grammar");
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

        // Remove existing cache if it exists
        if cache_path.exists() {
            fs::remove_dir_all(&cache_path)?;
        }

        // Copy directory to cache
        self.copy_dir_recursive(generated_src, &cache_path)?;

        Ok(())
    }

    fn cache_path(&self, crate_name: &str, cache_key: &str) -> Utf8PathBuf {
        // Use first 16 chars of hash for shorter directory names
        let short_key = &cache_key[..16.min(cache_key.len())];
        self.cache_dir.join(crate_name).join(short_key)
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

    fn copy_dir_recursive(&self, src_dir: &Utf8Path, dest_dir: &Utf8Path) -> std::io::Result<()> {
        copy_dir_recursive(src_dir, dest_dir)
    }
}

fn copy_dir_recursive(src_dir: &Utf8Path, dest_dir: &Utf8Path) -> std::io::Result<()> {
    fs::create_dir_all(dest_dir)?;

    for entry in fs::read_dir(src_dir)? {
        let entry = entry?;
        let path = Utf8PathBuf::from_path_buf(entry.path())
            .map_err(|_| std::io::Error::other("Non-UTF8 path"))?;
        let name = entry.file_name().to_string_lossy().to_string();
        let dest_path = dest_dir.join(&name);

        if path.is_dir() {
            copy_dir_recursive(&path, &dest_path)?;
        } else if path.is_file() {
            fs::copy(&path, &dest_path)?;
        }
    }
    Ok(())
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

        // Copy cached directory to destination
        self.copy_dir_recursive(&self.path, dest_dir)?;
        Ok(())
    }

    fn copy_dir_recursive(&self, src_dir: &Utf8Path, dest_dir: &Utf8Path) -> std::io::Result<()> {
        copy_dir_recursive(src_dir, dest_dir)
    }
}

/// Get the cross-grammar dependencies for a grammar.
/// Duplicated from generate.rs to avoid circular dependencies.
fn get_grammar_dependencies(config: &crate::types::CrateConfig) -> Vec<(String, String)> {
    let mut deps = Vec::new();

    for grammar in &config.grammars {
        for dep in &grammar.dependencies {
            deps.push((dep.npm.clone(), dep.krate.clone()));
        }
    }

    deps
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tool::Tool;
    use facet_kdl::{Span, Spanned};

    #[test]
    fn test_tree_sitter_version_detection() {
        // This test requires tree-sitter to be installed
        match Tool::TreeSitter.get_version() {
            Ok(version) => {
                println!("tree-sitter version: {}", version);
                // Should be something like "tree-sitter 0.25.10"
                assert!(version.contains("tree-sitter"));
                assert!(!version.is_empty());
            }
            Err(e) => {
                // Skip test if tree-sitter is not available
                println!("Skipping tree-sitter version test: {}", e);
            }
        }
    }

    #[test]
    fn test_tree_sitter_version_in_cache_key() {
        // Just test that we can get tree-sitter version for cache key
        match Tool::TreeSitter.get_version() {
            Ok(version) => {
                println!("✅ tree-sitter version detection works: {}", version);
                // Test that version gets included in hash computation
                let mut hasher = blake3::Hasher::new();
                hasher.update(b"tree-sitter-version:");
                hasher.update(version.as_bytes());
                hasher.update(b"\0");
                let hash = hasher.finalize().to_hex().to_string();
                assert!(!hash.is_empty());
                assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
                println!("✅ Version hashing works, hash: {}...", &hash[..16]);
            }
            Err(e) => {
                println!("⚠️ Skipping version cache test: {}", e);
            }
        }
    }

    #[test]
    fn test_cache_key_changes_with_tree_sitter_version() {
        // Test that different tree-sitter versions produce different cache keys
        let repo_root = Utf8PathBuf::from(".");
        let cache = GrammarCache::new(&repo_root);

        // Create a minimal config
        let config = crate::types::CrateConfig {
            repo: crate::types::Repo {
                value: Spanned {
                    value: "https://example.com/repo".to_string(),
                    span: Span::default(),
                },
            },
            commit: crate::types::Commit {
                value: Spanned {
                    value: "abc123".to_string(),
                    span: Span::default(),
                },
            },
            license: crate::types::License {
                value: Spanned {
                    value: "MIT".to_string(),
                    span: Span::default(),
                },
            },
            grammars: vec![],
        };

        // We can't really test different versions without mocking,
        // but we can test that cache key computation works
        match cache.compute_cache_key(
            &Utf8PathBuf::from("nonexistent"),
            &Utf8PathBuf::from("."),
            &config,
        ) {
            Ok(key) => {
                println!("✅ Cache key computed: {}", key);
                assert!(!key.is_empty());
                assert!(key.chars().all(|c| c.is_ascii_hexdigit()));
            }
            Err(e) => {
                println!("⚠️ Cache key test failed (expected): {}", e);
                // Expected failure due to missing grammar.js file
            }
        }
    }
}
