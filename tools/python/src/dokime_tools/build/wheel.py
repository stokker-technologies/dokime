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
from pathlib import Path

# Third Party Imports
# Local Imports

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def build(root: Path, profile: str) -> list[Path]:
    """Build And Retain The Client Wheel And Release Source Distribution With The Prepared Backend."""
    root = root.resolve()
    environment = root / ".venv"
    python = environment / ("Scripts/python.exe" if os.name == "nt" else "bin/python")
    if not python.is_file():
        raise ValueError("prepare the locked repository-root Python environment before building the wheel")

    output = root / "target/wheels"
    output.mkdir(parents=True, exist_ok=True)
    for stale in output.glob("dokime-*"):
        stale.unlink()

    command = ["uv", "build", "--no-build-isolation", "--offline", "--python", str(python), "--package", "dokime", "--out-dir", str(output)]
    if profile != "release":
        command += ["--wheel"]
    subprocess.run(command, cwd=root, check=True, env={**os.environ, "PYTHONDONTWRITEBYTECODE": "1"})

    wheels = sorted(output.glob("dokime-*.whl"))
    if len(wheels) != 1:
        raise ValueError(f"expected one dokime wheel in {output}, found {len(wheels)}")
    if profile != "release":
        return wheels
    sources = sorted(output.glob("dokime-*.tar.gz"))
    if len(sources) != 1:
        raise ValueError(f"expected one dokime source distribution in {output}, found {len(sources)}")
    return [*wheels, *sources]


# === Private Functions === #

# === Entry Point === #
