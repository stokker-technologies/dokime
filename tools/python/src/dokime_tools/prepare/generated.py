# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Carry The Generated GUI Message Modules Between The Build And The Lanes

    python -m dokime_tools.prepare.generated

The build regenerates the ignored `crates/dokime-gui/.generated` modules from the server's contract
export and stages a copy under `target/build/<profile>/<target>/generated/typescript`, a path the build
artifact carries. A lane runner has only the artifact, so the TypeScript lanes restore the staged
modules before they compile or import the GUI.
"""

# === Imports === #

# Standard Library Imports
import argparse
from pathlib import Path

# Third Party Imports
# Local Imports
from dokime_tools.build import paths
from dokime_tools.package.staging import sync_output

# Type Checking Imports


# === Public Exports === #

# === Constants === #

ROOT = Path(__file__).resolve().parents[5]
MODULES = ROOT / "crates/dokime-gui/.generated"

# === Types === #

# === Classes === #

# === Public Functions === #


def stage(profile: str) -> None:
    """Copy The Freshly Generated Modules Into The Build Output."""
    sync_output(MODULES, paths.output(ROOT, "build", profile) / "generated/typescript")


def restore(profile: str, *, fixtures: bool = False) -> None:
    """Replace The GUI's Generated Modules With The Staged Build Output."""
    output = paths.output(ROOT, "build", profile)
    staged = output / ("fixtures/typescript" if fixtures else "generated/typescript")
    if not staged.is_dir():
        raise SystemExit(f"missing staged generated modules {staged}; run the matching build producer")
    sync_output(staged, MODULES)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", choices=("dev", "release"), default="dev")
    parser.add_argument("--fixtures", action="store_true")
    options = parser.parse_args()
    restore(options.profile, fixtures=options.fixtures)
    return 0


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    raise SystemExit(main())
