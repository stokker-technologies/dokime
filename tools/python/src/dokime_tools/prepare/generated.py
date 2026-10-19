# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Carry The Generated GUI Message Modules Between The Build And The Lanes

    python -m dokime_tools.prepare.generated

The build regenerates `target/generated/typescript` from the server's contract export
and stages a copy under `target/build/<profile>/<target>/generated/typescript`, a path the build
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
MODULES = ROOT / "target/generated/typescript"

# === Types === #

# === Classes === #

# === Public Functions === #


def stage(profile: str) -> Path:
    """Copy The Freshly Generated Modules Into The Build Output."""
    staged = paths.output(ROOT, "build", profile) / "generated/typescript"
    sync_output(MODULES, staged)
    return staged


def restore(profile: str) -> None:
    """Replace The GUI's Generated Modules With The Staged Build Output."""
    staged = paths.output(ROOT, "build", profile) / "generated/typescript"
    if not staged.is_dir():
        raise SystemExit(f"missing staged generated modules {staged}; run `mise run build`")
    sync_output(staged, MODULES)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", choices=paths.PROFILES, default="dev")
    restore(parser.parse_args().profile)
    return 0


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    raise SystemExit(main())
