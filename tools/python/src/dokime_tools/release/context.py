# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Own Release Context Operations."""

# === Imports === #

# Standard Library Imports
import json
import os
import platform
import re
import tomllib
import urllib.parse
from pathlib import Path

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from .. import policy
from ..test import archive

# Type Checking Imports

# === Public Exports === #

# === Constants === #

STABLE = r"(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)"
SUMS = "SHA256SUMS"
INDEX = "release.json"
STATE = "published.json"
ASSETS = "target/release-assets"
SHARED = "any"
RELEASE_WORKFLOW = "release.yml"

# === Types === #

# === Classes === #

# === Public Functions === #


def continuous_integration() -> bool:
    """Whether A CI Runner Executes This Process."""
    return any(os.environ.get(name, "").lower() in {"1", "true"} for name in ("CI", "GITHUB_ACTIONS", "GITLAB_CI"))


def release_tag(directory: Path) -> str:
    """The Stable Tag Of The Prepared Version: `v` Plus `[version] value`."""
    value = policy.policy(directory, "version").get("value")
    if not isinstance(value, str) or re.fullmatch(STABLE, value) is None:
        raise policy.ToolError("Configure Release: version.value must declare a stable MAJOR.MINOR.PATCH version")
    return "v" + value


def native_target(settings: dict) -> str:
    system = {"Linux": "linux", "Windows": "windows", "Darwin": "macos"}.get(platform.system())
    architecture = {"amd64": "x86_64", "arm64": "aarch64"}.get(platform.machine().lower(), platform.machine().lower())
    selected = f"{system}-{architecture}"
    if selected not in settings.get("targets", []):
        raise policy.ToolError(f"Configure Release: native target {selected} is not one of release.targets")
    return selected


def native() -> str:
    system = {"Linux": "linux", "Windows": "windows", "Darwin": "macos"}.get(platform.system(), "")
    architecture = {"amd64": "x86_64", "arm64": "aarch64"}.get(platform.machine().lower(), platform.machine().lower())
    return f"{system}-{architecture}"


def package(settings: dict) -> str:
    """The GitLab Generic Package That Carries The Release Downloads."""
    return settings.get("package", settings["product"])


def quote(value: str) -> str:
    """A Single Path Segment Of A Provider Endpoint."""
    return urllib.parse.quote(value, safe="")


def refuse_overrides(directory: Path) -> None:
    """Published Bytes Never Come From Unpublished Local Dependency Checkouts."""
    if not (directory / "Cargo.toml").is_file():
        return
    metadata = json.loads(policy.execute(directory, ["cargo", "metadata", "--locked", "--format-version", "1"], capture=True))
    outside = []
    for outside_package in metadata["packages"]:
        if outside_package["source"] is not None:
            continue
        if Path(outside_package["manifest_path"]).resolve().is_relative_to(directory.resolve()):
            continue
        outside.append(outside_package["name"])

    if outside:
        raise policy.ToolError(f"Configure Release: unpublished local dependency override: {', '.join(sorted(outside))}")


def revision(directory: Path) -> str:
    sha = archive.revision(directory)
    if not sha:
        raise policy.ToolError("Configure Release: release operations require a committed HEAD")
    return sha


def root(directory: Path) -> Path:
    return policy.inside(directory, ASSETS)


def task_defined(directory: Path, name: str) -> bool:
    document = tomllib.loads((directory / "mise.toml").read_text(encoding="utf-8"))
    return name in document.get("tasks", {})


# === Private Functions === #

# === Entry Point === #
