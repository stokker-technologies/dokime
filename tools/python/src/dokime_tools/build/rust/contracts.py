# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Run Compile Contracts From The Prepared Archive"""

# === Imports === #

# Standard Library Imports
import os
import subprocess
import sys

# Third Party Imports
# Local Imports
from dokime_tools.build import configuration
from dokime_tools.test import archive

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def build(profile: str) -> None:
    """Run The Prepared Contract Harness Against Its Original SDK Compiler Inputs."""
    prepared = archive.require(configuration.ROOT, profile)
    environment = dict(os.environ, DOKIME_COMPILE_CONTRACTS_DIRECTORY=str(prepared.parent), TEST_PROFILE=profile)
    subprocess.run(
        [
            "cargo",
            "nextest",
            "run",
            "--archive-file",
            str(prepared),
            "--workspace-remap",
            str(configuration.ROOT),
            "-E",
            "binary_id(dokime-macros::compile)",
        ],
        cwd=configuration.ROOT,
        env=environment,
        stdin=sys.stdin,
        stdout=sys.stdout,
        stderr=sys.stderr,
        creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
        check=True,
    )


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    raise SystemExit(configuration.invoke(build))
