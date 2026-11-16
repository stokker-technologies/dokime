# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***
"Own Package Verification Operations."

# === Imports === #

# Standard Library Imports
import subprocess
import sys
import tarfile
import zipfile
from pathlib import Path

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from .. import policy, temporary
from ..build import paths
from ..process import spawn_flags
from . import definition

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def binary_bytes(source: Path, name: str) -> bytes:
    """Read Only The Declared Root Binary, Without Extracting Archive Paths."""
    if source.name.endswith(".zip"):
        archive = zipfile.ZipFile(source)
        try:
            matches = []
            for matches_entry in archive.infolist():
                if matches_entry.filename != name:
                    continue
                matches.append(matches_entry)

            if len(matches) != 1 or matches[0].is_dir():
                raise policy.ToolError("Package Product: tool archive must contain one declared binary")
            return archive.read(matches[0])
        finally:
            archive.close()
    with tarfile.open(source, "r:gz") as archive:
        matches = []
        for matches_entry in archive.getmembers():
            if matches_entry.name != name:
                continue
            matches.append(matches_entry)

        if len(matches) != 1 or not matches[0].isfile() or not matches[0].mode & 0o111:
            raise policy.ToolError("Package Product: tool archive must contain one executable regular binary")
        stream = archive.extractfile(matches[0])
        if stream is None:
            raise policy.ToolError("Package Product: tool archive binary has no file content")
        try:
            return stream.read()
        finally:
            stream.close()


def verify(directory: Path, profile: str, target: str, root: Path | None = None) -> None:
    """Execute Packaged Native Bytes In An Isolated Installation Directory."""
    for item in definition.definitions(directory, profile, target):
        # Staged release downloads hold only the executables that ship.
        if "triple" not in item:
            continue
        if root is not None and (not item["release"]):
            continue
        source = (root / item["output"]) if root else paths.output(directory, "package", profile, target) / "rust" / item["output"]
        if not source.is_file() or source.is_symlink():
            raise policy.ToolError("Package Product: missing packaged executable")
        install_directory = temporary.Directory()
        install = install_directory.path
        try:
            executable = install / item["binary"]
            executable.write_bytes(binary_bytes(source, item["binary"]))
            executable.chmod(0o755)
            subprocess.run(
                [str(executable), *item["verify"]],
                stdin=sys.stdin,
                stdout=sys.stdout,
                stderr=sys.stderr,
                creationflags=spawn_flags(),
                cwd=install,
                check=True,
            )
        finally:
            install_directory.close()


# === Private Functions === #

# === Entry Point === #
