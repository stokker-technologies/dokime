# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Install The One Built Wheel Into The Shared Environment."""

# === Imports === #

# Standard Library Imports
import subprocess
import sys
from pathlib import Path

# Third Party Imports
# Local Imports
from dokime_tools.build.paths import output
from dokime_tools.process import spawn_flags

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def main() -> None:
    root = Path(__file__).resolve().parents[3]
    wheels = list((output(root, "build", "dev", "any") / "python").glob("*.whl"))
    if len(wheels) != 1:
        raise ValueError("Build exactly one SDK wheel first")
    subprocess.run(
        ["uv", "pip", "install", "--python", sys.executable, "--reinstall-package", "dokime-sdk", str(wheels[0])],
        check=True,
        creationflags=spawn_flags(),
    )


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    main()
