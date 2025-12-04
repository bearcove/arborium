#!/usr/bin/env python3
"""Deduplicate identical WASM modules across plugins.

These are typically the WASI shim modules that jco generates.
"""

import hashlib
import os
import re
import sys
from collections import defaultdict
from pathlib import Path

def main():
    dist_dir = Path(sys.argv[1]) if len(sys.argv) > 1 else Path("dist/plugins")
    shared_dir = dist_dir.parent / "shared"

    print(f"Deduplicating WASM modules in {dist_dir}...")

    shared_dir.mkdir(parents=True, exist_ok=True)

    # Find all .wasm files and group by hash
    hash_to_files = defaultdict(list)

    for wasm in dist_dir.glob("*/*.wasm"):
        # Skip the main grammar.core.wasm - those are unique per language
        if wasm.name.endswith(".core.wasm"):
            continue

        file_hash = hashlib.sha256(wasm.read_bytes()).hexdigest()[:16]
        hash_to_files[file_hash].append(wasm)

    # Process duplicates
    saved_bytes = 0
    deduped_count = 0

    for file_hash, files in hash_to_files.items():
        # Only dedupe if there are multiple copies
        if len(files) < 2:
            continue

        # Get a canonical name (e.g., "shim.core2.wasm" from "grammar.core2.wasm")
        match = re.search(r"\.core(\d+)\.wasm$", files[0].name)
        if match:
            shared_name = f"shim.core{match.group(1)}.wasm"
        else:
            shared_name = f"shim.{file_hash}.wasm"

        shared_path = shared_dir / shared_name
        file_size = files[0].stat().st_size

        # Copy one to shared location
        shared_path.write_bytes(files[0].read_bytes())
        print(f"  Created shared: {shared_name} ({file_size} bytes, {len(files)} copies)")

        # Calculate savings
        saved_bytes += (len(files) - 1) * file_size
        deduped_count += len(files) - 1

        # Update each plugin's JS to reference shared path and remove duplicate
        for wasm in files:
            plugin_dir = wasm.parent
            js_file = plugin_dir / "grammar.js"
            wasm_basename = wasm.name

            if js_file.exists():
                content = js_file.read_text()
                content = content.replace(
                    f"getCoreModule('{wasm_basename}')",
                    f"getCoreModule('../shared/{shared_name}')"
                )
                js_file.write_text(content)

            # Remove the duplicate
            wasm.unlink()

    print()
    print("Deduplication complete!")
    print(f"  Removed {deduped_count} duplicate files")
    print(f"  Saved {saved_bytes} bytes ({saved_bytes // 1024} KiB)")


if __name__ == "__main__":
    main()
