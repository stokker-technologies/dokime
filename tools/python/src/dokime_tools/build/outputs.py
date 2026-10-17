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
from dokime_tools.test import archive

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def executables(root: Path, producer: dict, names: Sequence[str]) -> dict[str, Path]:
    """Select Non-Test Binaries And Verify Their Original Producer Hashes."""
    selected: dict[str, set[Path]] = {name: set() for name in names}
    messages = root / producer["messages"]["path"]
    for line in messages.read_text(encoding="utf-8").splitlines():
        item = json.loads(line)
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
    archive.verify_outputs(producer, list(result.values()))
    return result


def exports(producer: dict, source: Path, destination: Path) -> dict[Path, Path]:
    """Project Only Files Declared By The Original Native Producer."""
    source = source.absolute()
    projection: dict[Path, Path] = {}
    for name in producer.get("exports", {}):
        path = Path(name)
        if path.is_relative_to(source):
            projection[path] = destination / path.relative_to(source)
    if not projection:
        raise ValueError(f"the producer exported no files under {source}")
    return projection


# === Private Functions === #

# === Entry Point === #
