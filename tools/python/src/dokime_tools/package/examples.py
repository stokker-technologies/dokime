# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Stage And Package The Canonical Remote Examples"""

# === Imports === #

# Standard Library Imports
import stat
import zipfile
from pathlib import Path

# Third Party Imports
# Local Imports
from dokime_tools.package.staging import clear_output, stage_file

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# Only Public Canonical Inputs Travel; Local Keys, Dotenv Files And Evidence Never Do.
FILES = (
    "LICENSE-APACHE",
    ".env.example",
    "cli/README.md",
    "cli/control-device.sh",
    "cli/control-test.sh",
    "cli/control-prompt.sh",
    "cli/control-device.ps1",
    "cli/control-test.ps1",
    "cli/control-prompt.ps1",
    "cli/control-device.bat",
    "cli/control-test.bat",
    "cli/control-prompt.bat",
    "python/control-device.py",
    "python/control-test.py",
    "python/control-prompt.py",
)

# === Types === #

# === Classes === #

# === Public Functions === #


def stage(root: Path, destination: Path) -> list[Path]:
    """Copy The Canonical Scripts Into The Existing Build Artifact."""
    package_root = root / "packages/dokime"
    clear_output(destination)
    staged = []
    for relative in FILES:
        source = package_root / relative if relative == "LICENSE-APACHE" else package_root / "examples/remote" / relative
        _regular(source)
        output = destination / relative
        stage_file(source, output)
        staged.append(output)
    return staged


def package(root: Path, platform: str) -> None:
    """Archive The Staged Examples Without Reading Source Or Rebuilding The Client."""
    source = root / "dist/build" / platform / "examples/remote"
    if source.is_symlink() or not source.is_dir():
        raise ValueError("packaging requires the built remote examples; run `mise run build` first")
    for relative in FILES:
        _regular(source / relative)

    destination = root / "target/package" / platform / "examples"
    clear_output(destination)
    destination.mkdir()
    output = destination / "dokime-remote-examples.zip"
    with zipfile.ZipFile(output, "x", compression=zipfile.ZIP_STORED) as archive:
        _write(archive, source)
    print(f"Packaged {output.relative_to(root).as_posix()}")


# === Private Functions === #


def _regular(path: Path) -> None:
    if path.parent.is_symlink() or not stat.S_ISREG(path.lstat().st_mode):
        raise ValueError(f"example must be a regular file with an unlinked parent: {path}")


def _write(archive: zipfile.ZipFile, source: Path) -> None:
    for relative in FILES:
        # Fixed ZIP Metadata Keeps Identical Build Inputs Identical Across Platforms.
        entry = zipfile.ZipInfo(f"remote/{relative}")
        entry.create_system = 3
        entry.external_attr = (0o100755 if relative.endswith(".sh") else 0o100644) << 16
        archive.writestr(entry, (source / relative).read_bytes())


# === Entry Point === #
