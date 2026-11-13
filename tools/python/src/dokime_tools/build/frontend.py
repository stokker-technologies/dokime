# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Prepare Frontend And Report Assets"""

# === Imports === #

# Standard Library Imports
import argparse
import subprocess
import sys
import tempfile
from pathlib import Path

# Third Party Imports
# Local Imports
from dokime_tools.package.staging import clear_output, sync_output

from . import organization, paths

# Type Checking Imports

# === Public Exports === #

# === Constants === #

ROOT = Path(__file__).resolve().parents[5]
WEB_ROOT = ROOT / "crates/dokime-gui"


# === Types === #

# === Classes === #

# === Public Functions === #


def build_live(args: list[str]) -> None:
    organization.generate()
    vite = WEB_ROOT / "node_modules/vite/bin/vite.js"
    command = ["node", str(vite), "build"]
    if args:
        subprocess.run(
            command + args,
            cwd=WEB_ROOT,
            check=True,
            creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
            stdin=sys.stdin,
            stdout=sys.stdout,
            stderr=sys.stderr,
        )
        return
    with tempfile.TemporaryDirectory(prefix="dokime-gui-live-") as directory:
        subprocess.run(
            command + ["--outDir", directory],
            cwd=WEB_ROOT,
            check=True,
            creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
            stdin=sys.stdin,
            stdout=sys.stdout,
            stderr=sys.stderr,
        )
        sync_output(Path(directory), WEB_ROOT / "dist")


def build(profile: str) -> Path:
    """Build The Live Frontend Once And Mirror It Where The Desktop Shell Serves It From."""
    organization.generate()
    output = paths.output(ROOT, "build", profile) / "web"
    clear_output(output)
    subprocess.run(
        ["node", str(WEB_ROOT / "node_modules/vite/bin/vite.js"), "build", "--outDir", str(output)],
        cwd=WEB_ROOT,
        check=True,
        creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
        stdin=sys.stdin,
        stdout=sys.stdout,
        stderr=sys.stderr,
    )
    sync_output(output, WEB_ROOT / "dist")
    return output


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=["live"])
    args, extra = parser.parse_known_args()
    if args.command != "live" and extra:
        parser.error(f"unexpected arguments: {extra}")
    build_live(extra)


# === Private Functions === #


# === Entry Point === #

if __name__ == "__main__":
    main()
