# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Select Packageable Native Outputs From The Original Compiler Record"""

# === Imports === #

# Standard Library Imports
import json
from collections.abc import Sequence
from pathlib import Path

# Third Party Imports

# Local Imports

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def executables(messages: Path, names: Sequence[str]) -> dict[str, Path]:
    """Select The Non-Test Binaries One Cargo Build Reported."""
    selected: dict[str, set[Path]] = {name: set() for name in names}
    for line in messages.read_text(encoding="utf-8").splitlines():
        item = json.loads(line) if line.startswith("{") else {}
        target = item.get("target", {})
        name = target.get("name")
        if (
            item.get("reason") == "compiler-artifact"
            and name in selected
            and "bin" in target.get("kind", [])
            and not item.get("profile", {}).get("test", False)
            and item.get("executable")
        ):
            selected[name].add(Path(item["executable"]))
    result: dict[str, Path] = {}
    for name, paths in selected.items():
        if len(paths) != 1:
            raise ValueError(f"Cargo must report exactly one {name} executable")
        result[name] = next(iter(paths))
    return result


def libraries(messages: Path, packages: Sequence[str]) -> list[Path]:
    """Select Shipping Workspace Libraries, Excluding Metadata And Dependency Crates."""
    names = {name.replace("-", "_") for name in packages}
    kinds = {"lib", "rlib", "dylib", "cdylib", "staticlib", "proc-macro"}
    suffixes = {".rlib", ".a", ".lib", ".so", ".dylib", ".dll"}
    selected = set()
    for line in messages.read_text(encoding="utf-8").splitlines():
        item = json.loads(line) if line.startswith("{") else {}
        target = item.get("target", {})
        if item.get("reason") != "compiler-artifact" or item.get("profile", {}).get("test", False):
            continue
        if target.get("name") not in names or not kinds.intersection(target.get("kind", [])):
            continue
        selected.update(Path(name) for name in item.get("filenames", []) if Path(name).suffix in suffixes)
    return sorted(selected)


# === Private Functions === #

# === Entry Point === #
