# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Generate And Transfer Frontend Protobuf Modules."""

# === Imports === #

# Standard Library Imports
import argparse
import subprocess
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


def prepare() -> None:
    """Generate Frontend Message Modules From The Checked-In Protocol Descriptor."""
    subprocess.run(
        ["node", str(ROOT / "tools/typescript/generate.mjs")],
        cwd=ROOT,
        check=True,
        creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
    )


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
    parser.add_argument("--generate", action="store_true", help="Generate Frontend Protobuf Modules")
    options = parser.parse_args()
    if options.generate:
        prepare()
    else:
        restore(options.profile)
    return 0


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    raise SystemExit(main())
