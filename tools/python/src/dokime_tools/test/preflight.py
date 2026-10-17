# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Refuse A Test Lane Whose Fixtures, Host Or Archive Are Missing Or Stale"""

# === Imports === #

# Standard Library Imports
import argparse
import sys
from pathlib import Path

# Third Party Imports

# Local Imports
# isort: split
from ..fixtures import STAMP, stamp

# Type Checking Imports

# === Public Exports === #

# === Constants === #

ROOT = Path(__file__).resolve().parents[5]
PLATFORM = "windows" if sys.platform == "win32" else "linux"
SUFFIX = ".exe" if sys.platform == "win32" else ""
HOST = ROOT / "dist/build" / PLATFORM / "bin" / f"dokime-server{SUFFIX}"
FIXTURES = ROOT / "target/fixtures"
ARCHIVES = ROOT / "target/test-archives"

# === Types === #

# === Classes === #

# === Public Functions === #


def findings(layer: str) -> list[str]:
    """Every missing or stale prerequisite of one lane, before any test runs."""
    problems = []
    if not any(ARCHIVES.glob("*.tar.zst")):
        problems.append(f"no nextest archive under {ARCHIVES}; run mise run build")
    if layer in {"acceptance", "system"}:
        if not HOST.is_file():
            problems.append(f"missing host {HOST}; run mise run build")
        recorded = FIXTURES / STAMP
        if not recorded.is_file():
            problems.append(f"missing fixture stamp {recorded}; run mise run build")
        elif recorded.read_text(encoding="ascii") != stamp(ROOT):
            problems.append(f"stale fixtures in {FIXTURES}; run mise run build")
    return problems


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("layer", choices=("integration", "acceptance", "system"))
    problems = findings(parser.parse_args().layer)
    for problem in problems:
        print(f"preflight: {problem}", file=sys.stderr)
    raise SystemExit(1 if problems else 0)
