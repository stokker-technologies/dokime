# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Verify The Launcher And Icon In Assembled Linux Packages"""

# === Imports === #

# Standard Library Imports
import configparser
import hashlib
import re
import shutil
import subprocess
from pathlib import Path, PurePosixPath

# Third Party Imports

# Local Imports

# Type Checking Imports

# === Public Exports === #

# === Constants === #

HICOLOR = PurePosixPath("/usr/share/icons/hicolor")

# === Types === #

# === Classes === #

# === Public Functions === #


def verify(package: Path, launcher: Path) -> None:
    """Require The Packaged Launcher To Resolve An Installed Hicolor Icon."""
    contents = _contents(package)
    launcher_path = f"/usr/share/applications/{launcher.name}"
    digest = hashlib.sha256(launcher.read_bytes()).hexdigest()
    if contents.get(launcher_path) != digest:
        raise ValueError("RPM desktop launcher differs from its packaged payload")

    # Resolve The Packaged Launcher, Not A Separate Product Name
    desktop = configparser.ConfigParser(interpolation=None)
    desktop.read_string(launcher.read_text(encoding="utf-8"))
    icon = desktop.get("Desktop Entry", "Icon", fallback="").strip()
    if not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9_.-]*", icon):
        raise ValueError("RPM desktop launcher must name an installed theme icon")
    for name, checksum in contents.items():
        path = PurePosixPath(name)
        matching = path.name == f"{icon}.png" and path.parent.name == "apps" and path.parent.parent.parent == HICOLOR
        if matching and re.fullmatch(r"[0-9a-f]{64}", checksum):
            return
    raise ValueError(f"RPM launcher icon {icon!r} has no installed hicolor PNG payload")


# === Private Functions === #


def _contents(package: Path) -> dict[str, str]:
    executable = shutil.which("rpm")
    if executable is None:
        raise ValueError("Linux installer verification requires rpm; install it before packaging")
    query = "%{FILEDIGESTALGO}\n[%{FILENAMES}\t%{FILEDIGESTS}\n]"
    result = subprocess.run([executable, "-qp", "--qf", query, str(package)], capture_output=True, text=True, check=True)
    lines = result.stdout.splitlines()
    if not lines or lines[0] != "8":
        raise ValueError("RPM icon verification requires SHA-256 file digests")

    contents = {}
    for line in lines[1:]:
        name, separator, checksum = line.partition("\t")
        if not separator or not name.startswith("/") or name in contents:
            raise ValueError("invalid RPM payload file index")
        contents[name] = checksum
    return contents


# === Entry Point === #
