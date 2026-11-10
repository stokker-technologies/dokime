# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Receive And Export Task Artifacts Through The Single Archive Implementation."""

# === Imports === #

# Standard Library Imports
import json
import os
from pathlib import Path

# Third Party Imports
# Local Imports
from ..build.paths import target
from . import artifacts

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def receive() -> None:
    """Extract Only Explicitly Downloaded Transport Archives."""
    if os.environ.get("CI_TRANSPORT") != "true":
        return
    for archive in sorted(Path("target/ci").glob("*.tar.zst")):
        artifacts.extract(Path.cwd(), archive)


def send(profile: str) -> None:
    """Export The Producer's Declared Outputs Once For All Consumers."""
    for output in json.loads(os.environ.get("CI_OUTPUTS", "[]")):
        artifacts.pack(
            Path.cwd(),
            Path(
                output["archive"]
                .replace("{profile}", profile)
                .replace("{platform}", os.environ.get("CI_PLATFORM", "windows" if os.name == "nt" else "unix"))
            ),
            output["paths"],
            profile,
            target(),
        )


# === Private Functions === #

# === Entry Point === #
