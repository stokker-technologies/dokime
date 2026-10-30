# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Carry The Generated GUI Message Modules Between The Build And The Lanes

    python -m dokime_tools.prepare.generated

The build regenerates `target/generated/typescript` through `cargo dokime bindings proto`
and stages a copy under `target/build/<profile>/<target>/generated/typescript`, a path the build
artifact carries. Lanes restore this Product export before importing the GUI.
"""

# === Imports === #

# Standard Library Imports
import argparse
import subprocess
import sys
import tempfile
from pathlib import Path

# Third Party Imports
# Local Imports
from dokime_tools.build import paths
from dokime_tools.build.outputs import executables
from dokime_tools.package.contracts import locate
from dokime_tools.package.staging import sync_output

# Type Checking Imports


# === Public Exports === #

# === Constants === #

ROOT = Path(__file__).resolve().parents[5]
MODULES = ROOT / "target/generated/typescript"

# === Types === #

# === Classes === #

# === Public Functions === #


def generate(messages: Path, *, check: bool = False) -> None:
    """Generate GUI Modules From The Selected Cargo Tool's Protobuf Export."""
    tool = executables(messages, ("cargo-dokime",))["cargo-dokime"]
    settings = locate(messages)
    with tempfile.TemporaryDirectory(prefix="dokime-proto-") as directory:
        output = Path(directory) / "proto"
        commands = [
            [str(tool), "bindings", "proto", "--output", str(output)],
            ["node", str(ROOT / "crates/dokime-gui/tools/bindings.mjs"), str(output), str(settings), *(["--check"] if check else [])],
        ]
        for command in commands:
            subprocess.run(
                command,
                cwd=ROOT,
                check=True,
                creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
                stdin=sys.stdin,
                stdout=sys.stdout,
                stderr=sys.stderr,
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
    parser.add_argument("--stage", action="store_true", help="Stage freshly generated modules instead of restoring them")
    parser.add_argument("--build-json", type=Path, help="Generate from the cargo-dokime executable in these Cargo messages")
    parser.add_argument("--check", action="store_true", help="Check tracked generated clients while regenerating modules")
    options = parser.parse_args()
    if options.check and options.build_json is None:
        parser.error("--check requires --build-json")
    if options.build_json is not None:
        generate(options.build_json, check=options.check)
    if options.stage:
        stage(options.profile)
    elif options.build_json is None:
        restore(options.profile)
    return 0


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    raise SystemExit(main())
