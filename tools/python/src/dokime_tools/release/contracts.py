# ***
#
# dokime-rust-workspace
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Verify and Archive Self-Contained Protocol Contracts"""

# === Imports === #

# Standard Library Imports
import re
import zipfile
from pathlib import Path, PurePosixPath

# Third Party Imports
# Local Imports
from .. import policy

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def package(exported: Path, destination: Path) -> None:
    """Verify Every Local Import Before Archiving All Exported Contracts and Notices."""
    files = {}
    sources = {}
    for path in exported.rglob("*"):
        if not path.is_file():
            continue
        name = path.relative_to(exported).as_posix()
        files[name] = path
        if name.endswith(".proto"):
            sources[name] = path
    required = {"README.md", "buf.yaml", "licenses/Apache-2.0.txt", "licenses/Protobuf.txt"}
    missing = required - files.keys()
    if not sources or missing:
        raise policy.ToolError(
            "Package Protocol Contracts: Export Definitions, README, Buf Configuration and Licenses With the Current CLI"
        )
    for name, path in sorted(sources.items()):
        content = path.read_text(encoding="utf-8")
        # Ignore Comments While Preserving Quoted Import Paths.
        tokens = re.finditer(r'"(?:\\.|[^"\\])*"|/\*.*?\*/|//[^\n]*|[^"/]+|/', content, re.DOTALL)
        parts = []
        for token in tokens:
            if token[0].startswith(("//", "/*")):
                parts.append(" ")
            else:
                parts.append(token[0])
        content = "".join(parts)
        for imported in re.findall(r'\bimport\s+(?:(?:public|weak)\s+)?"([^"\n]+)"\s*;', content):
            relative = PurePosixPath(imported)
            if relative.is_absolute() or ".." in relative.parts or imported not in sources:
                raise policy.ToolError(f"Package Protocol Contracts: Re-Export Missing Import {imported} Required by {name}")
    archive = zipfile.ZipFile(destination, "w", compression=zipfile.ZIP_DEFLATED)
    try:
        for name, path in sorted(files.items()):
            archive.write(path, name)
    finally:
        archive.close()


# === Private Functions === #

# === Entry Point === #
