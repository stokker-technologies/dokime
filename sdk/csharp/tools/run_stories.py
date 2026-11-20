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

root: Path = Path(__file__).resolve().parents[1]
configuration: str = os.environ.get("DOKIME_CSHARP_CONFIGURATION", "Release")
suffix: str = ".exe" if os.name == "nt" else ""
environment: dict[str, str] = os.environ.copy()
environment.setdefault("HARMOS_CONFORMANCE_HOST", str(root / f"../../target/reference-host/bin/harmos-conformance{suffix}"))
environment["DOKIME_CONFORMANCE_PLUGIN"] = str(
    root / f"../../target/cs/bin/Conformance/{configuration.lower()}/dokime-conformance-plugin{suffix}"
)
environment.setdefault("DOKIME_TEST_HOST", str(root / f"../../target/debug/dokime-host{suffix}"))
environment.setdefault("DOKIME_STARTER", str(root / f"examples/Starter/assets/plugin/Example.Plugin{suffix}"))
subprocess.run(
    [
        sys.argv[1],
        "test",
        str(root / "tests/Stories/Stories.csproj"),
        "--configuration",
        configuration,
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
