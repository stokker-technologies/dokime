#!/usr/bin/env python3
# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Pin the SDK and workspace lockfile to the current Harmos dev revision."""

# === Imports === #

# Standard Library Imports
import pathlib
import re
import subprocess

# Third Party Imports

# Local Imports
# isort: split
from ..ci.bootstrap import harmos_pin

# Type Checking Imports

# === Public Exports === #

# === Constants === #

REPOSITORY = "https://github.com/stokker-technologies/harmos.git"
MANIFEST = pathlib.Path(__file__).resolve().parents[5] / "Cargo.toml"

# === Types === #

# === Classes === #

# === Public Functions === #


def main() -> None:
    dependency = harmos_pin(MANIFEST.parent)

    tip = subprocess.run(
        ["git", "ls-remote", REPOSITORY, "refs/heads/dev"],
        check=True,
        capture_output=True,
        text=True,
        encoding="utf-8",
    ).stdout.split()[0]
    if not re.fullmatch(r"[0-9a-f]{40}", tip):
        raise SystemExit("Harmos dev did not resolve to a full Git revision")

    source = MANIFEST.read_text(encoding="utf-8")
    previous = dependency["rev"]
    pattern = rf'(harmos(?:-contract|-signal)? = \{{[^\n]*rev = ")({previous})(")'
    updated, count = re.subn(pattern, rf"\g<1>{tip}\g<3>", source)
    if count != 3:
        raise SystemExit("Could not locate all three pinned Harmos dependencies")

    MANIFEST.write_text(updated, encoding="utf-8")
    subprocess.run(["cargo", "update", "-p", "harmos", "--precise", tip], check=True)
    print(f"harmos manifest and lockfile pinned to {tip}")


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    main()
