# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Build the Local Library Reference Host for Integration Only."""

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

# === Private Functions === #

# === Entry Point === #

root = Path(__file__).resolve().parents[3]
target = root / "target"
# Integration Only: Compile the Current Library Main Checkout Into Product-Owned Outputs.
source = Path(os.environ.get("HARMOS_SOURCE") or root / "../harmos").resolve()
subprocess.run(
    [
        "cargo",
        "install",
        "--path",
        str(source / "tools/conformance"),
        "--locked",
        "--debug",
        "--force",
        "--root",
        str(target / "reference-host"),
        "--bin",
        "harmos-conformance",
        "--jobs",
        os.environ.get("CARGO_BUILD_JOBS", "6"),
        "--target-dir",
        str(target),
    ],
    check=True,
)
