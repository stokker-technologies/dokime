# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Format Only SDK-Owned C# Sources and Installed Examples."""

# === Imports === #

# Standard Library Imports
import argparse
import subprocess
from pathlib import Path

# Third Party Imports

# Local Imports

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #

# === Private Functions === #

# === Entry Point === #

parser: argparse.ArgumentParser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("dotnet")
parser.add_argument("--check", action="store_true")
options: argparse.Namespace = parser.parse_args()
root: Path = Path(__file__).resolve().parents[1]
files: list[str] = [
    str(path.resolve())
    for folder in ("src", "tests", "examples")
    for path in (root / folder).rglob("*.cs")
    if not {"Generated", "bin", "obj"}.intersection(path.parts)
]
command: list[str] = [options.dotnet, "format", "whitespace", str(root), "--folder", "--include", *files]
if options.check:
    command.append("--verify-no-changes")
subprocess.run(command, check=True)
fixtures: Path = (root / "../tests/fixtures/csharp").resolve()
command = [options.dotnet, "format", "whitespace", str(fixtures), "--folder"]
if options.check:
    command.append("--verify-no-changes")
subprocess.run(command, check=True)
command = [options.dotnet, "format", str(root / "Dokime.slnx"), "--no-restore", "--include", *files]
if options.check:
    command.append("--verify-no-changes")
subprocess.run(command, check=True)
