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
export and stages a copy under `dist/build/<platform>/generated/typescript`, a path the build
artifact carries. A lane runner has only the artifact, so the TypeScript lanes restore the staged
modules before they compile or import the GUI.
"""

# === Imports === #

# Standard Library Imports
import sys
from pathlib import Path

# Third Party Imports
# Local Imports
from dokime_tools.package.staging import sync_output

# Type Checking Imports


# === Public Exports === #

# === Constants === #

ROOT = Path(__file__).resolve().parents[5]
MODULES = ROOT / "crates/dokime-gui/.generated"
STAGED = ROOT / "dist/build" / ("windows" if sys.platform == "win32" else "linux") / "generated/typescript"

# === Types === #

# === Classes === #

# === Public Functions === #


def stage() -> None:
    """Copy The Freshly Generated Modules Into The Build Output."""
    sync_output(MODULES, STAGED)


def restore() -> None:
    """Replace The GUI's Generated Modules With The Staged Build Output."""
    if not STAGED.is_dir():
        raise SystemExit(f"missing staged generated modules {STAGED}; run `mise run build`")
    sync_output(STAGED, MODULES)


def main() -> int:
    restore()
    return 0


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    raise SystemExit(main())
