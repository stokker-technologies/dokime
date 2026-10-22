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
import subprocess
import sys
from pathlib import Path

# Third Party Imports
# Local Imports
from dokime_tools.build import paths

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def build(root: Path, profile: str) -> Path:
    """Build The Client Wheel Offline With The Prepared Backend Into The Lanes' Wheel Directory."""
    root = root.resolve()
    environment = root / ".venv"
    python = environment / ("Scripts/python.exe" if os.name == "nt" else "bin/python")
    if not python.is_file():
        raise ValueError("prepare the locked repository-root Python environment before building the wheel")
    output = paths.output(root, "build", profile) / "python"
    output.mkdir(parents=True, exist_ok=True)
    for stale in output.glob("dokime-*"):
        stale.unlink()
    command = ["uv", "build", "--no-build-isolation", "--offline", "--python", str(python), "--package", "dokime", "--out-dir", str(output)]
    if profile != "release":
        command += ["--wheel"]
    subprocess.run(
        command,
        cwd=root,
        check=True,
        env={**os.environ, "PYTHONDONTWRITEBYTECODE": "1"},
        creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
        stdin=sys.stdin,
        stdout=sys.stdout,
        stderr=sys.stderr,
    )
    wheels = sorted(output.glob("dokime-*.whl"))
    if len(wheels) != 1:
        raise ValueError(f"expected one dokime wheel in {output}, found {len(wheels)}")
    return wheels[0]


# === Private Functions === #

# === Entry Point === #
