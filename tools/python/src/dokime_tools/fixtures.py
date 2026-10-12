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
import os
from pathlib import Path
import subprocess

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


def prepare() -> None:
    tool = Path(os.environ["DOKIME_TEST_TOOL"])
    if not tool.is_file():
        raise ValueError("run mise run build:release before preparing fixtures")
    for name, package, binary in FIXTURES:
        output = Path("target/fixtures") / name / f"{name}.dokime"
        command = [
            str(tool),
            "dokime",
            "build",
            "--package",
            package,
            "--output",
            str(output),
        ]
        if binary:
            command += ["--bin", binary]
        subprocess.run(command, check=True)
        if not output.is_file():
            raise ValueError(f"missing native test fixture: {output}")


# === Private Functions === #

# === Entry Point === #
if __name__ == "__main__":
    prepare()
