# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Build The Pure Python SDK From Authoritative Protobuf Sources."""

# === Imports === #

# Standard Library Imports
import os
import subprocess
import sys
from pathlib import Path

# Third Party Imports
# Local Imports
from dokime_tools.build import paths
from dokime_tools.package.staging import clear_output
from dokime_tools.process import spawn_flags

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def build(root: Path, profile: str) -> list[Path]:
    """Build One Platform-Independent Wheel In The Selected Product Output."""
    root = root.resolve()
    output = paths.output(root, "build", profile) / "python"
    clear_output(output)
    output.mkdir(parents=True, exist_ok=True)
    environment = dict(os.environ, PYTHONDONTWRITEBYTECODE="1")
    environment.pop("UV_PROJECT_ENVIRONMENT", None)
    command = [
        "uv",
        "build",
        "--wheel",
        "--out-dir",
        str(output),
        "sdk/python",
    ]
    subprocess.run(
        command,
        cwd=root,
        check=True,
        env=environment,
        creationflags=spawn_flags(),
        stdin=sys.stdin,
        stdout=sys.stdout,
        stderr=sys.stderr,
    )
    wheels = sorted(output.glob("*.whl"))
    if len(wheels) != 1 or not wheels[0].name.endswith("-py3-none-any.whl"):
        raise ValueError(f"expected one pure Python wheel in {output}")
    print(f"Python Wheel: {wheels[0].name} ({wheels[0].stat().st_size} bytes)", flush=True)
    return wheels


# === Private Functions === #

# === Entry Point === #
