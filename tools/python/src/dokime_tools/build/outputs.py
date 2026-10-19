# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Select Native Executables From The Original Compiler Output"""

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


# === Private Functions === #

# === Entry Point === #
