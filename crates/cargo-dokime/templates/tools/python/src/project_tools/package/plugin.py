# ***
#
# {{package}}
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Package The Matching Frozen Plugin Without Compiling"""

# === Imports === #

# Standard Library Imports
import argparse
import shutil
import subprocess
import sys
from pathlib import Path

# Third Party Imports
# Local Imports
from .. import temporary
from ..build import paths

# Type Checking Imports

# === Public Exports === #

# === Constants === #

PACKAGE = "{{package}}"

# === Types === #

# === Classes === #

# === Public Functions === #


def package(root: Path, profile: str) -> None:
    source = paths.output(root, "build", profile) / "plugins" / f"{PACKAGE}.dokime"
    if source.is_symlink() or not source.is_file():
        raise ValueError("packaging requires the matching built plugin; run mise run build first")
    destination = paths.output(root, "package", profile) / "rust/plugins"

    # Package Consumes The Built Bundle Without Cargo Compilation
    with temporary.directory() as scratch:
        shutil.copy2(source, scratch / source.name)
        paths.replace(root, scratch, destination, origin=scratch)
    subprocess.run(
        ["cargo", "dokime", "sidecar", "inspect", str(destination / source.name)],
        cwd=root,
        check=True,
        creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
        stdin=sys.stdin,
        stdout=sys.stdout,
        stderr=sys.stderr,
    )


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", choices=("dev", "release"), default="dev")
    package(Path.cwd(), parser.parse_args().profile)


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    main()
