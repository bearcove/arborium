# AGENTS Notes

## tree-sitter fork workflow (`crates/arborium-tree-sitter`)

- Do **not** patch `crates/arborium-tree-sitter` by hand.
- Use `scripts/sync_tree_sitter_fork.py` to sync/reset from upstream and re-apply Arborium patches.
- If you need to change Arborium-specific patch behavior for the tree-sitter fork, update the sync script accordingly (instead of editing forked files directly).

## Generated Cargo manifests

- Most `Cargo.toml` files in this repo are generated from `Cargo.stpl.toml` templates.
- Prefer editing the corresponding `Cargo.stpl.toml` source template, then regenerating, rather than hand-editing generated `Cargo.toml` files.
- The same principle applies to generated plugin manifests (for example under language `npm/` directories): update templates/generation logic, then regenerate.