# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Build Client Distributions With The Locked Repository Backend"""

import os
import subprocess
import sys
from pathlib import Path

from dokime_tools.build import paths
from dokime_tools.package.staging import clear_output


def build(root: Path, profile: str) -> list[Path]:
    """Build And Retain The Client Wheel And Release Source Distribution With The Prepared Backend."""
    root = root.resolve()
    environment = root / ".venv"
    python = environment / ("Scripts/python.exe" if os.name == "nt" else "bin/python")
    if not python.is_file():
        raise ValueError("prepare the locked repository-root Python environment before building the wheel")

    output = paths.output(root, "build", profile, "any") / "python"
    # The Pure Client Wheel Replaces The Former Native Product Output.
    clear_output(paths.output(root, "build", profile) / "python")
    clear_output(output)
    output.mkdir(parents=True, exist_ok=True)

    command = [
        "uv",
        "build",
        "--no-build-isolation",
        "--offline",
        "--python",
        str(python),
        "--package",
        "dokime-client",
        "--out-dir",
        str(output),
    ]
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

    wheels = sorted(output.glob("dokime_client-*.whl"))
    if len(wheels) != 1:
        raise ValueError(f"expected one dokime-client wheel in {output}, found {len(wheels)}")
    if profile != "release":
        return wheels
    sources = sorted(output.glob("dokime_client-*.tar.gz"))
    if len(sources) != 1:
        raise ValueError(f"expected one dokime-client source distribution in {output}, found {len(sources)}")
    return [*wheels, *sources]
