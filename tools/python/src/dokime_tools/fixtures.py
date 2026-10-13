# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Compile Native Test Fixtures With The Packaged Tool"""

# === Imports === #

# Standard Library Imports
import argparse
import os
import subprocess
from pathlib import Path

# Third Party Imports

# Local Imports

# Type Checking Imports


# === Public Exports === #

# === Constants === #
FIXTURES = (
    ("mini-sidecar", "mini-sidecar", None),
    ("dokime-route-plugin", "dokime-route-plugin", "dokime-route-plugin"),
    ("fixture-sidecar-protocol", "dokime-route-plugin", "fixture-sidecar-protocol"),
    ("fixture-suites", "dokime-route-plugin", "dokime-app-suite-sidecar-fixture"),
    (
        "fixture-telemetry",
        "dokime-route-plugin",
        "dokime-app-telemetry-sidecar-fixture",
    ),
    (
        "fixture-dependent",
        "dokime-route-plugin",
        "dokime-app-dependent-sidecar-fixture",
    ),
)


# === Types === #

# === Classes === #

# === Public Functions === #


def prepare(
    *, tool: Path | None = None, output: Path = Path("target/fixtures")
) -> None:
    tool = tool or Path(os.environ["DOKIME_TEST_TOOL"])
    if not tool.is_file():
        raise ValueError("run mise run build:release before preparing fixtures")
    for name, package, binary in FIXTURES:
        bundle = output / name / f"{name}.dokime"
        command = [
            str(tool),
            "dokime",
            "build",
            "--package",
            package,
            "--profile",
            "release",
            "--output",
            str(bundle),
        ]
        if binary:
            command += ["--bin", binary]
        subprocess.run(command, check=True)
        if not bundle.is_file() or bundle.is_symlink():
            raise ValueError(f"missing native test fixture: {bundle}")


# === Private Functions === #

# === Entry Point === #
if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--tool", type=Path)
    parser.add_argument("--output", type=Path, default=Path("target/fixtures"))
    options = parser.parse_args()
    prepare(tool=options.tool, output=options.output)
