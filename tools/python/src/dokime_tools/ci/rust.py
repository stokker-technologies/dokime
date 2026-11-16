# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Check Workspace Dependency Use"""

# === Imports === #

# Standard Library Imports
import json
import subprocess
import sys
from pathlib import Path

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from dokime_tools.process import spawn_flags

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #

# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    root = Path.cwd()
    metadata = subprocess.run(
        ["cargo", "metadata", "--no-deps", "--locked", "--format-version", "1"],
        cwd=root,
        check=True,
        capture_output=True,
        text=True,
        creationflags=spawn_flags(),
    )
    document = json.loads(metadata.stdout)
    members = set(document["workspace_members"])
    directories = sorted({Path(package["manifest_path"]).parent for package in document["packages"] if package["id"] in members})
    roots = []
    for roots_directory in directories:
        if any(parent in directories for parent in roots_directory.parents):
            continue
        roots.append(roots_directory)

    result = subprocess.run(
        ["cargo-machete", "--skip-target-dir", *map(str, roots)],
        check=False,
        cwd=root,
        creationflags=spawn_flags(),
        capture_output=True,
        text=True,
    )
    print(result.stdout, end="")
    print(result.stderr, end="", file=sys.stderr)
    raise SystemExit(result.returncode)
