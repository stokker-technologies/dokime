# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Build Client Distributions With The Locked Repository Backend"""

# === Imports === #

# Standard Library Imports
import os
from collections.abc import Sequence
from pathlib import Path

# Third Party Imports

# Local Imports
from dokime_tools.package.staging import clear_output
from dokime_tools.test import archive

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def capture(root: Path, producer: dict, backend: Sequence[Path]) -> dict:
    """Use The Prepared Backend Offline And Preserve Original Wheel Exports.

    `backend` names the installed build-system packages the nonisolated build imports.
    """
    root = root.resolve()
    profile = producer["profile"]
    environment = root / ".venv"
    python = environment / ("Scripts/python.exe" if os.name == "nt" else "bin/python")
    if not python.is_file():
        raise ValueError("prepare the locked repository-root Python environment before building the wheel")
    output = root / "target/build" / profile / "python"
    clear_output(output)
    command = [
        "uv",
        "build",
        "--no-build-isolation",
        "--offline",
        "--python",
        str(python),
        "--package",
        "dokime",
        "--out-dir",
        str(output),
    ]
    if profile != "release":
        command += ["--wheel"]
    previous = os.environ.get("PYTHONDONTWRITEBYTECODE")
    os.environ["PYTHONDONTWRITEBYTECODE"] = "1"
    try:
        return archive.native(
            root,
            profile,
            "python-package",
            [command],
            [output],
            locks=[root / "uv.lock", root / "mise.lock"],
            installed=list(backend),
            parents=[producer],
        )
    finally:
        if previous is None:
            os.environ.pop("PYTHONDONTWRITEBYTECODE", None)
        else:
            os.environ["PYTHONDONTWRITEBYTECODE"] = previous



# === Private Functions === #

# === Entry Point === #
