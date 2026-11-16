# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Own Release Registry Python Operations."""

# === Imports === #

# Standard Library Imports
import hashlib
import json
import os
import tomllib
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from ... import policy
from ...build import paths, wheel
from .. import notices

# Type Checking Imports

# === Public Exports === #

# === Constants === #


# === Types === #

# === Classes === #

# === Public Functions === #


def publish(directory: Path, settings: dict, *, dry_run: bool) -> None:
    """Upload The Release Python Packages: PyPI For GitHub Products, The Project Registry On GitLab."""
    profile = policy.profile()
    root = paths.output(directory, "package", profile, "any") / "python"
    selected = wheel.distributions(directory, root, policy.config(directory)["version"]["value"])
    command = ["uv", "publish"]
    if dry_run:
        command += ["--dry-run", "--trusted-publishing", "never"]
    elif settings["provider"] == "github":
        # Reruns Skip Files PyPI Already Holds With The Same Digest And Refuse A Different Digest.
        published = digests(tomllib.loads((directory / "sdk/python/pyproject.toml").read_text(encoding="utf-8"))["project"]["name"])
        missing = []
        for path in selected:
            if path.name not in published:
                missing.append(path)
                continue
            if hashlib.sha256(path.read_bytes()).hexdigest() != published[path.name]:
                raise policy.ToolError(f"Publish to PyPI: {path.name} already exists with a different SHA-256; release a new version")
            print(f"PyPI already holds {path.name} with a matching SHA-256; skipping it")
        if not missing:
            return
        selected = missing
        command += ["--trusted-publishing", "always", "--check-url", "https://pypi.org/simple/"]
    else:
        project = urllib.parse.quote(str(settings.get("project_id", os.environ.get("CI_PROJECT_ID", ""))), safe="")
        base = os.environ.get("CI_API_V4_URL", settings.get("api_url", "https://gitlab.com/api/v4"))
        command += ["--publish-url", f"{base}/projects/{project}/packages/pypi", "--trusted-publishing", "never"]
    policy.execute(directory, [*command, *(path.relative_to(directory).as_posix() for path in selected)])


def digests(project: str) -> dict[str, str]:
    """SHA-256 Digests Of Every File PyPI Holds For This Project, By Filename."""
    try:
        document = notices.download(f"https://pypi.org/pypi/{urllib.parse.quote(project, safe='')}/json", operation="PyPI project read")
    except policy.ToolError as error:
        if isinstance(error.__cause__, urllib.error.HTTPError) and error.__cause__.code == 404:
            return {}
        raise
    releases = json.loads(document)["releases"]
    selected_items = {}
    for selected_files in releases.values():
        for selected_item in selected_files:
            selected_items[selected_item["filename"]] = selected_item["digests"]["sha256"]
    return selected_items


# === Private Functions === #

# === Entry Point === #
