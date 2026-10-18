# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Package Plugin Fixtures From The Existing Workspace Build"""

# === Imports === #

# Standard Library Imports
import json
import subprocess
from pathlib import Path

# Third Party Imports
# Local Imports
from dokime_tools.package.staging import clear_output

# Type Checking Imports

# === Public Exports === #

# === Constants === #

PACKAGES = ("obc-plugin",)

# === Types === #

# === Classes === #

# === Public Functions === #


def prepare(root: Path, messages: Path, profile: str, *, tool: Path, output: Path) -> list[Path]:
    """Collect The Archived Build's Plugin Executables And Package Them Without Another Compiler."""
    root = root.resolve()
    metadata = messages.with_name("metadata.json")
    with metadata.open("w", encoding="utf-8") as written:
        subprocess.run(["cargo", "metadata", "--locked", "--format-version", "1"], cwd=root, stdout=written, check=True)
    clear_output(output)
    output.mkdir(parents=True)
    staged: list[Path] = []
    for package in PACKAGES:
        build_json = output / f"{package}.build.json"
        collector = [str(tool), "dokime", "build", "--from-cargo-json", str(messages), "--metadata-json", str(metadata)]
        collector += ["--profile", profile, "--package", package, "--json"]
        with build_json.open("w", encoding="utf-8") as written:
            subprocess.run(collector, cwd=root, stdout=written, check=True)
        packager = [str(tool), "dokime", "package", "--build-json", str(build_json), "--output", str(output), "--json"]
        packaged = subprocess.run(packager, cwd=root, capture_output=True, text=True, check=True).stdout
        for item in json.loads(packaged):
            path = Path(item["path"])
            staged.append(path if path.is_absolute() else root / path)
    return staged


# === Private Functions === #

# === Entry Point === #
