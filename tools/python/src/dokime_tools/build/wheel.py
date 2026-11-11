# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Build The Native SDK With The Locked Repository Backend"""

# === Imports === #

# Standard Library Imports
import os
import subprocess
import sys
from pathlib import Path

# Third Party Imports
from harmos_tools.build import paths
from harmos_tools.process import spawn_flags

# Local Imports
from dokime_tools.build import storage
from dokime_tools.package.staging import clear_output

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def build(root: Path, profile: str) -> list[Path]:
    """Build One Native abi3 Wheel In The Product Cargo Tree And Profile."""
    root = root.resolve()
    output = paths.output(root, "build", profile) / "python"
    clear_output(output)
    output.mkdir(parents=True, exist_ok=True)
    environment = dict(os.environ, PYTHONDONTWRITEBYTECODE="1", PYO3_PYTHON=sys.executable)
    environment.pop("UV_PROJECT_ENVIRONMENT", None)
    selectors = ["--locked", "--manifest-path", "sdk/python/Cargo.toml", "--profile", profile]
    features = ["--features", "media-native"] if profile == "release" else []
    storage.preflight(root)
    command = [
        sys.executable,
        "-m",
        "maturin",
        "build",
        *selectors,
        *features,
        "--interpreter",
        sys.executable,
        "--out",
        str(output),
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
    if len(wheels) != 1 or "-cp313-abi3-" not in wheels[0].name:
        raise ValueError(f"expected one Python 3.13+ abi3 wheel in {output}")
    print(f"Native Wheel: {wheels[0].name} ({wheels[0].stat().st_size} bytes)", flush=True)
    return wheels


# === Private Functions === #

# === Entry Point === #
