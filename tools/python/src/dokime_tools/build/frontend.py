# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"Prepare Frontend and Report Assets"

# === Imports === #

# Standard Library Imports
import argparse
import subprocess
import sys
import tempfile
from pathlib import Path

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from dokime_tools.package.staging import clear_output, sync_output

from ..process import spawn_flags
from . import configuration, organization, paths

# Type Checking Imports

# === Public Exports === #

# === Constants === #


WEB_ROOT = configuration.ROOT / "crates/dokime-gui"


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
            creationflags=spawn_flags(),
            stdin=sys.stdin,
            stdout=sys.stdout,
            stderr=sys.stderr,
        )
        return
    directory_directory = tempfile.TemporaryDirectory(prefix="dokime-gui-live-")
    directory = directory_directory.name
    try:
        subprocess.run(
            command + ["--outDir", directory],
            cwd=WEB_ROOT,
            check=True,
            creationflags=spawn_flags(),
            stdin=sys.stdin,
            stdout=sys.stdout,
            stderr=sys.stderr,
        )
        sync_output(Path(directory), WEB_ROOT / "dist")
    finally:
        directory_directory.cleanup()


def build(profile: str) -> Path:
    """Build the Live Frontend Once and Mirror It Where the Desktop Shell Serves It From."""
    organization.generate()
    output = paths.output(configuration.ROOT, "build", profile) / "web"
    clear_output(output)
    subprocess.run(
        ["node", str(WEB_ROOT / "node_modules/vite/bin/vite.js"), "build", "--outDir", str(output)],
        cwd=WEB_ROOT,
        check=True,
        creationflags=spawn_flags(),
        stdin=sys.stdin,
        stdout=sys.stdout,
        stderr=sys.stderr,
    )
    sync_output(output, WEB_ROOT / "dist")
    return output


def build_report(args: list[str]) -> None:
    """Generate the Desktop's Standalone Report Renderer."""
    organization.generate()
    directory_directory = tempfile.TemporaryDirectory(prefix="dokime-gui-report-")
    directory = directory_directory.name
    try:
        subprocess.run(
            ["node", str(WEB_ROOT / "node_modules/vite/bin/vite.js"), "build", "--mode", "report", "--outDir", directory, *args],
            cwd=WEB_ROOT,
            check=True,
            creationflags=spawn_flags(),
            stdin=sys.stdin,
            stdout=sys.stdout,
            stderr=sys.stderr,
        )
        sync_output(Path(directory), WEB_ROOT / "dist-report")
    finally:
        directory_directory.cleanup()


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=["live", "report"])
    args, extra = parser.parse_known_args()
    if args.command == "report":
        build_report(extra)
    else:
        build_live(extra)


# === Private Functions === #


# === Entry Point === #

if __name__ == "__main__":
    main()
