#!/usr/bin/env python3
"""
Hard-reset `crates/arborium-tree-sitter` from upstream tree-sitter `lib/`
and re-apply Arborium-specific patches.

Usage examples:

  # Dry-run (default), shows planned actions
  python3 scripts/sync_tree_sitter_fork.py --upstream ~/bearcove/tree-sitter --tag v0.26.6

  # Apply changes in-place
  python3 scripts/sync_tree_sitter_fork.py --upstream ~/bearcove/tree-sitter --tag v0.26.6 --apply

  # Also commit result
  python3 scripts/sync_tree_sitter_fork.py --upstream ~/bearcove/tree-sitter --tag v0.26.6 --apply --commit

Assumptions:
- Run from repo root (expects this script at scripts/sync_tree_sitter_fork.py).
- Upstream repo is already cloned.
- This script will not push.
"""

from __future__ import annotations

import argparse
import dataclasses
import datetime as dt
import shutil
import subprocess
import sys
from pathlib import Path
from typing import List, Tuple

# ----------------------------
# Config
# ----------------------------

TARGET_REL = Path("crates/arborium-tree-sitter")


# Files/directories to preserve from the existing Arborium fork before reset.
PRESERVE_PATHS = [
    Path("Cargo.stpl.toml"),
    Path("Cargo.lock"),
    Path("CMakeLists.txt"),
    Path("tree-sitter.pc.in"),
    Path("README.md"),
    Path("LICENSE"),
]

# A generated marker to place in patched files where useful.
PATCH_HEADER = (
    "# This file is patched by scripts/sync_tree_sitter_fork.py\n"
    "# Do not edit generated sync sections manually unless you also update the script.\n"
)


@dataclasses.dataclass
class CmdResult:
    code: int
    out: str
    err: str


def run(
    args: List[str],
    cwd: Path | None = None,
    check: bool = True,
    capture: bool = True,
) -> CmdResult:
    proc = subprocess.run(
        args,
        cwd=str(cwd) if cwd else None,
        text=True,
        capture_output=capture,
    )
    res = CmdResult(proc.returncode, proc.stdout or "", proc.stderr or "")
    if check and res.code != 0:
        joined = " ".join(args)
        raise RuntimeError(
            f"Command failed ({res.code}): {joined}\nSTDOUT:\n{res.out}\nSTDERR:\n{res.err}"
        )
    return res


def info(msg: str) -> None:
    print(f"[sync-tree-sitter] {msg}")


def warn(msg: str) -> None:
    print(f"[sync-tree-sitter][WARN] {msg}")


def die(msg: str, code: int = 1) -> None:
    print(f"[sync-tree-sitter][ERROR] {msg}", file=sys.stderr)
    raise SystemExit(code)


def ensure_clean_tree(repo_root: Path, allow_dirty: bool) -> None:
    if allow_dirty:
        return
    res = run(["git", "status", "--porcelain"], cwd=repo_root)
    if res.out.strip():
        die(
            "Working tree is not clean. Commit/stash changes or pass --allow-dirty.",
            code=2,
        )


def ensure_paths(repo_root: Path, upstream_root: Path, tag: str) -> Tuple[Path, Path]:
    target = repo_root / TARGET_REL
    if not target.exists():
        die(f"Target directory not found: {target}")

    upstream_root = upstream_root.expanduser().resolve()
    if not (upstream_root / ".git").exists():
        die(f"Upstream path is not a git repository: {upstream_root}")

    upstream_lib = upstream_root / "lib"
    if not upstream_lib.exists():
        die(f"Upstream lib directory not found: {upstream_lib}")

    # Verify tag exists.
    try:
        run(
            ["git", "rev-parse", "-q", "--verify", f"refs/tags/{tag}"],
            cwd=upstream_root,
            check=True,
        )
    except RuntimeError:
        die(f"Tag not found in upstream: {tag}")

    return target, upstream_lib


def checkout_upstream_tag(upstream_root: Path, tag: str, dry_run: bool) -> str:
    # Save current ref so we can put it back.
    res = run(
        ["git", "rev-parse", "--abbrev-ref", "HEAD"], cwd=upstream_root, check=False
    )
    current_ref = res.out.strip()
    if current_ref == "HEAD":
        detached = run(["git", "rev-parse", "HEAD"], cwd=upstream_root).out.strip()
        current_ref = detached

    info(f"Upstream current ref: {current_ref}")
    info(f"Checking out upstream tag: {tag}")
    if not dry_run:
        run(["git", "fetch", "--tags", "--prune"], cwd=upstream_root)
        run(["git", "checkout", tag], cwd=upstream_root)

    new_rev = run(
        ["git", "rev-parse", "--short", "HEAD"], cwd=upstream_root
    ).out.strip()
    info(f"Upstream now at: {new_rev}")
    return current_ref


def restore_upstream_ref(upstream_root: Path, old_ref: str, dry_run: bool) -> None:
    info(f"Restoring upstream ref: {old_ref}")
    if dry_run:
        return
    run(["git", "checkout", old_ref], cwd=upstream_root)


def rm_tree(path: Path) -> None:
    if path.exists():
        shutil.rmtree(path)


def copy_tree(src: Path, dst: Path) -> None:
    if dst.exists():
        rm_tree(dst)
    shutil.copytree(src, dst)


def copy_file(src: Path, dst: Path) -> None:
    dst.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(src, dst)


def backup_preserved(target: Path, backup_dir: Path) -> None:
    backup_dir.mkdir(parents=True, exist_ok=True)
    for rel in PRESERVE_PATHS:
        p = target / rel
        if p.exists():
            copy_file(p, backup_dir / rel)


def restore_preserved(target: Path, backup_dir: Path) -> None:
    for rel in PRESERVE_PATHS:
        b = backup_dir / rel
        if b.exists():
            copy_file(b, target / rel)


def patch_binding_rust_build_rs(target: Path) -> None:
    """
    Apply Arborium-specific WASM sysroot logic to build.rs.

    We patch the existing wasm gate inline so scope/order are always correct.
    """
    path = target / "binding_rust" / "build.rs"
    if not path.exists():
        warn(f"Missing {path}, skipping patch.")
        return

    src = path.read_text()

    if "DEP_ARBORIUM_SYSROOT_PATH" in src:
        info("build.rs already has arborium sysroot patch.")
        return

    old_gate = """if target.starts_with("wasm32-unknown") {
        configure_wasm_build(&mut config);
    }"""

    new_gate = """if target.starts_with("wasm32-unknown") {
        let mut arborium_has_sysroot = false;

        // Arborium patch: prefer arborium-sysroot and disable upstream wasm stdlib
        // sources to avoid duplicate symbols (malloc/free/...).
        if let Ok(sysroot) = env::var("DEP_ARBORIUM_SYSROOT_PATH") {
            let wasm_sysroot = PathBuf::from(&sysroot);
            config.include(&wasm_sysroot);
            println!("cargo:rerun-if-changed={}", wasm_sysroot.display());
            arborium_has_sysroot = true;
        }

        if !arborium_has_sysroot {
            configure_wasm_build(&mut config);
        }
    }"""

    if old_gate not in src:
        warn(
            "Could not find wasm32 configure gate in build.rs; upstream layout may have changed."
        )
        return

    patched = src.replace(old_gate, new_gate, 1)

    # Keep existing Arborium warning suppression behavior for wasm targets.
    if (
        'if target.contains("wasm") {' in patched
        and 'config.flag_if_supported("-Wno-format");' not in patched
    ):
        patched = patched.replace(
            'if target.contains("wasm") {',
            'if target.contains("wasm") {\n'
            "        // Arborium patch: suppress format warnings on wasm32 where\n"
            "        // uint32_t may be unsigned long.\n"
            '        config.flag_if_supported("-Wno-format");',
            1,
        )

    path.write_text(patched)


def patch_binding_rust_lib_rs_languagefn_reexport(target: Path) -> None:
    """
    Ensure `binding_rust/lib.rs` publicly re-exports `LanguageFn` so downstream
    crates can import it via `arborium_tree_sitter::LanguageFn`.
    """
    path = target / "binding_rust" / "lib.rs"
    if not path.exists():
        warn(f"Missing {path}, skipping LanguageFn re-export patch.")
        return

    src = path.read_text()

    if "pub use tree_sitter_language::LanguageFn;" in src:
        info("binding_rust/lib.rs already has LanguageFn public re-export.")
        return

    old = "use tree_sitter_language::LanguageFn;"
    new = "pub use tree_sitter_language::LanguageFn;"

    if old in src:
        path.write_text(src.replace(old, new, 1))
        info("Patched binding_rust/lib.rs to publicly re-export LanguageFn.")
        return

    warn(
        "Could not find `use tree_sitter_language::LanguageFn;` in binding_rust/lib.rs; "
        "skipping LanguageFn re-export patch."
    )


def patch_clock_h_if_needed(target: Path) -> None:
    """
    Ensure src/clock.h contains Arborium wasm stub branch if the file exists.
    Newer upstream may remove this file; if absent, skip.
    """
    path = target / "src" / "clock.h"
    if not path.exists():
        info("src/clock.h not present in upstream layout; skipping clock patch.")
        return

    src = path.read_text()
    if "defined(__wasm__) && !defined(__EMSCRIPTEN__)" in src:
        info("clock.h already has wasm stub branch.")
        return

    needle = "#if defined(_WIN32)"
    if needle not in src:
        warn("clock.h has unexpected format; skipping clock patch.")
        return

    wasm_block = """#if defined(__wasm__) && !defined(__EMSCRIPTEN__)

// WASM (non-Emscripten): stub out clock functions
// In WASM mode, we don't have access to clock() or clock_gettime(),
// so we provide stub implementations that disable timeout functionality.
typedef uint64_t TSClock;

static inline TSDuration duration_from_micros(uint64_t micros) {
  (void)micros;
  return 0;
}

static inline uint64_t duration_to_micros(TSDuration self) {
  (void)self;
  return 0;
}

static inline TSClock clock_null(void) {
  return 0;
}

static inline TSClock clock_now(void) {
  return 0;
}

static inline TSClock clock_after(TSClock base, TSDuration duration) {
  (void)base;
  (void)duration;
  return 0;
}

static inline bool clock_is_null(TSClock self) {
  return !self;
}

static inline bool clock_is_gt(TSClock self, TSClock other) {
  (void)self;
  (void)other;
  return false;
}

#elif defined(_WIN32)
"""
    src = src.replace(needle, wasm_block, 1)
    path.write_text(src)


def write_sync_metadata(
    target: Path, upstream_tag: str, upstream_rev_short: str
) -> None:
    meta = target / ".sync-meta.txt"
    now = dt.datetime.now(dt.UTC).isoformat(timespec="seconds")
    meta.write_text(
        f"{PATCH_HEADER}"
        f"upstream_tag={upstream_tag}\n"
        f"upstream_rev={upstream_rev_short}\n"
        f"synced_at_utc={now}\n"
    )


def maybe_commit(repo_root: Path, tag: str, rev: str, do_commit: bool) -> None:
    if not do_commit:
        return
    msg = f"tree-sitter: hard-reset fork from upstream {tag} ({rev}) and reapply arborium patches"
    run(["git", "add", str(TARGET_REL)], cwd=repo_root)
    run(["git", "commit", "-m", msg], cwd=repo_root)
    info("Committed changes.")


def summarize_diff(repo_root: Path) -> None:
    info("Changed files under target:")
    res = run(
        ["git", "status", "--short", "--", str(TARGET_REL)], cwd=repo_root, check=False
    )
    out = res.out.strip()
    if out:
        print(out)
    else:
        print("(none)")


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Hard-reset arborium-tree-sitter from upstream lib and reapply Arborium patches."
    )
    parser.add_argument(
        "--upstream", required=True, help="Path to upstream tree-sitter repo"
    )
    parser.add_argument(
        "--tag", required=True, help="Upstream tag to sync from (e.g. v0.26.6)"
    )
    parser.add_argument(
        "--apply", action="store_true", help="Apply changes (default is dry-run)"
    )
    parser.add_argument(
        "--commit", action="store_true", help="Create a commit after applying"
    )
    parser.add_argument(
        "--allow-dirty",
        action="store_true",
        help="Allow running with dirty working tree",
    )
    args = parser.parse_args()

    dry_run = not args.apply
    repo_root = Path(__file__).resolve().parents[1]
    upstream_root = Path(args.upstream).expanduser().resolve()

    ensure_clean_tree(repo_root, args.allow_dirty)
    target, upstream_lib = ensure_paths(repo_root, upstream_root, args.tag)

    info(f"Repo root: {repo_root}")
    info(f"Target: {target}")
    info(f"Upstream: {upstream_root}")
    info(f"Mode: {'DRY-RUN' if dry_run else 'APPLY'}")

    current_ref = checkout_upstream_tag(upstream_root, args.tag, dry_run=dry_run)
    try:
        upstream_rev_short = run(
            ["git", "rev-parse", "--short", f"refs/tags/{args.tag}"], cwd=upstream_root
        ).out.strip()

        backup_dir = repo_root / ".cache" / "sync-tree-sitter-backup"
        if dry_run:
            info("Would backup preserved files:")
            for p in PRESERVE_PATHS:
                print(f"  - {TARGET_REL / p}")
        else:
            rm_tree(backup_dir)
            backup_preserved(target, backup_dir)

        if dry_run:
            info(f"Would replace {TARGET_REL} with upstream lib/ at {args.tag}")
        else:
            rm_tree(target)
            copy_tree(upstream_lib, target)

        if dry_run:
            info("Would restore preserved files and apply Arborium patches.")
        else:
            restore_preserved(target, backup_dir)
            patch_binding_rust_build_rs(target)
            patch_binding_rust_lib_rs_languagefn_reexport(target)
            patch_clock_h_if_needed(target)
            write_sync_metadata(target, args.tag, upstream_rev_short)

        summarize_diff(repo_root)

        if not dry_run:
            maybe_commit(repo_root, args.tag, upstream_rev_short, args.commit)

        info("Done.")
    finally:
        restore_upstream_ref(upstream_root, current_ref, dry_run=dry_run)


if __name__ == "__main__":
    main()
