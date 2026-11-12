# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Run the already-built SDK journeys with explicit artifact paths."""
# === Imports === #

# Standard Library Imports
import os
import subprocess
import sys
from pathlib import Path

# Third Party Imports
# Local Imports
# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #

# === Private Functions === #

# === Entry Point === #

root = Path(__file__).resolve().parents[1]
suffix = ".exe" if os.name == "nt" else ""
environment = os.environ.copy()
environment["HARMOS_CONFORMANCE_HOST"] = str(root / f"../../target/csharp-reference/target/debug/harmos-conformance{suffix}")
environment["DOKIME_CONFORMANCE_PLUGIN"] = str(root / f"tests/Conformance/bin/Release/net10.0/dokime-conformance-plugin{suffix}")
environment["DOKIME_STARTER"] = str(root / f"starter/bin/Release/net10.0/dokime-starter{suffix}")
subprocess.run(
    [
        sys.argv[1],
        "test",
        str(root / "tests/Stories/Stories.csproj"),
        "--configuration",
        "Release",
        "--no-build",
        "--no-restore",
        "--filter",
        sys.argv[2],
        "--logger",
        "console;verbosity=normal",
    ],
    env=environment,
    check=True,
)
