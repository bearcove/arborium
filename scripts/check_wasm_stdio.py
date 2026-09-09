#!/usr/bin/env python3

import os
import shutil
import subprocess
import sys
from pathlib import Path


def main() -> int:
    repo_root = Path(__file__).resolve().parents[1]
    tree_sitter = repo_root / "crates" / "arborium-tree-sitter"
    sysroot = repo_root / "crates" / "arborium-sysroot" / "wasm-sysroot"
    compiler = os.environ.get("CC_wasm32_unknown_unknown") or shutil.which("clang")
    if compiler is None:
        print("`clang` is required for the WASM stdio check", file=sys.stderr)
        return 1

    result = subprocess.run(
        [
            compiler,
            "--target=wasm32-unknown-unknown",
            "-E",
            "-P",
            f"-I{sysroot}",
            f"-I{tree_sitter / 'src'}",
            f"-I{tree_sitter / 'src' / 'wasm'}",
            f"-I{tree_sitter / 'include'}",
            "-D_POSIX_C_SOURCE=200112L",
            "-D_DEFAULT_SOURCE",
            "-D_BSD_SOURCE",
            "-D_DARWIN_C_SOURCE",
            str(tree_sitter / "src" / "lib.c"),
        ],
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        sys.stderr.write(result.stderr)
        return result.returncode
    if "stderr" in result.stdout:
        print("WASM preprocessing retained a `stderr` reference", file=sys.stderr)
        return 1

    print("WASM stdio preprocessing has no `stderr` reference")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
