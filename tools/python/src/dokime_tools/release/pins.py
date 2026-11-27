# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***
"Reject Git Dependencies for Releases and Report Them in Nightly."

# === Imports === #

# Standard Library Imports
import json
import os
import re
import tomllib
from pathlib import Path

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from .. import policy

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def check(directory: Path, *, warn: bool = False) -> bool:
    """Check the Resolved Graph, Including Transitive Git Sources."""
    metadata = policy.execute(directory, ["cargo", "metadata", "--locked", "--format-version", "1"], capture=True)
    packages = json.loads(metadata)["packages"]
    git_packages = {package["name"] for package in packages if (package.get("source") or "").startswith("git+")}
    git_dependencies = {
        dependency["name"]
        for package in packages
        for dependency in package.get("dependencies", [])
        if (dependency.get("source") or "").startswith("git+")
    }
    names = sorted(git_packages | git_dependencies | python_git_dependencies(directory))
    if not names:
        return True
    message = "release requires registry dependencies; replace Git sources with published registry versions: " + ", ".join(names)
    if not warn:
        raise policy.ToolError(message)
    # Report Pending Registry Pins on Nightly.
    pending = "Stable Release Prerequisite Pending: " + message + "; Pin Published Registry Versions Before a Stable Release"
    print("::warning::" + pending + "; skipping the crate publication dry run on this non-stable channel")
    if summary := os.environ.get("GITHUB_STEP_SUMMARY"):
        with Path(summary).open("a", encoding="utf-8") as stream:
            stream.write("\n### Release Prerequisite Pending\n\n" + pending + "\n")
    return False


def python_git_dependencies(directory: Path) -> set[str]:
    """Find Git Sources in the Published SDK Wheel Metadata and the Starters `dokime new` Ships."""
    sdk = directory / "sdk/python/pyproject.toml"
    names: set[str] = set()
    for manifest in [sdk, *sorted((directory / "sdk/python/examples").glob("*/pyproject.toml"))]:
        if not manifest.is_file():
            continue
        document = tomllib.loads(manifest.read_text(encoding="utf-8"))
        location = manifest.relative_to(directory).as_posix()
        project = document.get("project", {})
        requirements = list(project.get("dependencies", []))
        for group in project.get("optional-dependencies", {}).values():
            requirements.extend(group)
        for requirement in requirements:
            if "git+" in requirement:
                name = re.split(r"[\s\[<>=!~@;]", requirement, maxsplit=1)[0]
                names.add(f"{name} ({location})")
        # Starters Ship Their uv Sources; the Wheel Carries Only Its Project Tables.
        sources = {} if manifest == sdk else document.get("tool", {}).get("uv", {}).get("sources", {})
        for name, source in sources.items():
            if any("git" in entry for entry in (source if isinstance(source, list) else [source])):
                names.add(f"{name} ({location})")
    return names


# === Private Functions === #

# === Entry Point === #
