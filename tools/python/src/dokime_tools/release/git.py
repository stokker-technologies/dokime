# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Own Release Git Operations."""

# === Imports === #

# Standard Library Imports
import os
import subprocess
import sys
from pathlib import Path

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from .. import policy
from ..process import spawn_flags

# Type Checking Imports

# === Public Exports === #

# === Constants === #


# === Types === #

# === Classes === #

# === Public Functions === #


def push_remote(settings: dict, token: str) -> str:
    """An Authenticated Push URL For The Release Identity, Which The main Ruleset Admits."""
    if settings["provider"] == "github":
        repository = settings.get("repository", os.environ.get("GITHUB_REPOSITORY"))
        return f"https://x-access-token:{token}@github.com/{repository}.git"
    return f"https://oauth2:{token}@{os.environ['CI_SERVER_HOST']}/{os.environ['CI_PROJECT_PATH']}.git"


def run(directory: Path, command: list[str], secret: str) -> str:
    """Run A Command That Carries A Credential, Never Echoing It; Returns Its Scrubbed Standard Output."""
    result = subprocess.run(
        command,
        creationflags=spawn_flags(),
        cwd=directory,
        capture_output=True,
        text=True,
        encoding="utf-8",
        check=False,
    )
    stderr = result.stderr or ""
    stdout = result.stdout or ""
    if secret:
        stderr = stderr.replace(secret, "***")
        stdout = stdout.replace(secret, "***")
    print(stderr, end="", file=sys.stderr)
    if result.returncode:
        raise policy.ToolError(f"Update Release Git References: {command[0]} {command[1]} failed with exit code {result.returncode}")
    return stdout


# === Private Functions === #

# === Entry Point === #
