# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Installed native binding and stub compatibility."""

# === Imports === #

# Standard Library Imports
import os
from pathlib import Path
import subprocess
import sys

# Third Party Imports

# Local Imports

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def test_installed_bindings() -> None:
    root = Path(__file__).resolve().parents[4]
    environment = {**os.environ, "PYTHONPATH": str(root / "tools/python/src")}
    subprocess.run(
        [sys.executable, "-m", "dokime_tools.check.bindings", "--runtime"],
        cwd=root,
        env=environment,
        check=True,
    )

    subprocess.run(
        [
            sys.executable,
            "-m",
            "mypy.stubtest",
            "--strict-type-check-only",
            "--concise",
            "dokime._native",
            "dokime.core.suites",
            "dokime.core.authoring.context",
            "dokime.host",
            "dokime.records",
            "dokime.runtime",
            "dokime.suites",
        ],
        check=True,
    )


# === Private Functions === #

# === Entry Point === #
