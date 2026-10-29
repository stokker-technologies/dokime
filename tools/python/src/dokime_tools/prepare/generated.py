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
artifact carries. Lanes restore this Product export before importing the GUI.
"""

import argparse
from pathlib import Path

from dokime_tools.build import paths
from dokime_tools.package.staging import sync_output

ROOT = Path(__file__).resolve().parents[5]
MODULES = ROOT / "target/generated/typescript"


def stage(profile: str) -> Path:
    """Copy The Freshly Generated Modules Into The Build Output."""
    staged = paths.output(ROOT, "build", profile) / "generated/typescript"
    sync_output(MODULES, staged)
    return staged


def restore(profile: str) -> None:
    """Replace The GUI's Generated Modules With The Staged Build Output."""
    output = paths.output(ROOT, "build", profile)
    staged = output / "generated/typescript"
    if not staged.is_dir():
        raise SystemExit(f"missing Product generated modules {staged}; run `mise run build`")
    sync_output(staged, MODULES)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", choices=paths.PROFILES, default="dev")
    parser.add_argument("--stage", action="store_true", help="Stage freshly generated modules instead of restoring them")
    options = parser.parse_args()
    if options.stage:
        stage(options.profile)
    else:
        restore(options.profile)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
