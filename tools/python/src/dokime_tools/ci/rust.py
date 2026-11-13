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
from pathlib import Path

# Third Party Imports
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
    roots = [directory for directory in directories if not any(parent in directories for parent in directory.parents)]
    result = subprocess.run(["cargo-machete", "--skip-target-dir", *map(str, roots)], cwd=root, creationflags=spawn_flags())
    raise SystemExit(result.returncode)
