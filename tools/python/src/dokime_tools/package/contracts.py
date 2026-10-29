# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Locate The Contract Export Of The dokime-server Build Script In One Cargo Build"""

# === Imports === #

# Standard Library Imports
import json
from pathlib import Path

# Third Party Imports

# Local Imports

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def locate(messages: Path) -> Path:
    artifacts: set[str] = set()
    scripts: dict[str, set[Path]] = {}
    finished: list[bool] = []
    for line in messages.read_text(encoding="utf-8").splitlines():
        event = json.loads(line)
        reason = event.get("reason")
        if reason == "compiler-artifact":
            target = event["target"]
            # The server build script exports the operation contracts it serves.
            if target["name"] == "dokime_server" and "lib" in target["kind"]:
                artifacts.add(event["package_id"])
        elif reason == "build-script-executed":
            package = event["package_id"]
            output = Path(event["out_dir"])
            scripts.setdefault(package, set()).add(output)
        elif reason == "build-finished":
            finished.append(event["success"])
    if finished != [True] or len(artifacts) != 1:
        raise ValueError("expected one dokime-server contract producer from a successful Cargo build")
    outputs = scripts.get(next(iter(artifacts)), set())
    if not outputs:
        raise ValueError("selected Dokime build has no build-script output")
    if len(outputs) != 1:
        raise ValueError("selected Dokime build has multiple build variants")
    output = next(iter(outputs))
    root = output / "grpc/operations"
    descriptor = root / "descriptors.bin"
    manifest = root / "manifest.txt"
    if not descriptor.is_file() or not manifest.is_file():
        raise ValueError("selected Dokime contract build did not export contracts")
    names = manifest.read_text(encoding="utf-8").splitlines()
    if not {"descriptors.bin", "contracts.json"}.issubset(names) or len(names) != len(set(names)):
        raise ValueError("invalid contract export manifest")
    for name in names:
        relative = Path(name)
        if (
            not name
            or "\\" in name
            or ":" in name
            or relative.is_absolute()
            or ".." in relative.parts
            or relative.as_posix() != name
            or (name not in {"descriptors.bin", "contracts.json"} and relative.suffix != ".proto")
        ):
            raise ValueError(f"invalid contract export path: {name}")
        if not (root / relative).is_file():
            raise ValueError(f"missing selected contract export: {name}")
    return root


# === Private Functions === #

# === Entry Point === #
