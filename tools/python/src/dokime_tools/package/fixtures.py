# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Package Plugin Fixtures From The Existing Workspace Build"""

# === Imports === #

# Standard Library Imports
import json
from pathlib import Path

# Third Party Imports
# Local Imports
from dokime_tools.package.staging import clear_output
from dokime_tools.test import archive

# Type Checking Imports

# === Public Exports === #

# === Constants === #

PACKAGES = ("obc-plugin",)

# === Types === #

# === Classes === #

# === Public Functions === #


def prepare(
    root: Path,
    producer: dict,
    *,
    tool: Path,
    output: Path,
) -> tuple[list[dict], dict[str, str]]:
    """Collect Typed Build Receipts And Package Without Another Compiler."""
    root = root.resolve()
    tool = tool.resolve()
    output = output.resolve()
    archive.verify_outputs(producer, [tool])
    messages = root / producer["messages"]["path"]
    metadata = root / producer["metadata"]["path"]
    receipt = archive.location(root, producer["profile"]).with_name(archive.RECEIPT)
    output.mkdir(parents=True, exist_ok=True)
    files: dict[str, str] = {}
    producers: list[dict] = []
    for package in PACKAGES:
        directory = output / package
        clear_output(directory)
        build_json = directory / "build.json"
        package_json = directory / "package.json"
        collector = [
            str(tool),
            "dokime",
            "build",
            "--from-cargo-json",
            str(messages),
            "--metadata-json",
            str(metadata),
            "--archive-json",
            str(receipt),
            "--profile",
            producer["profile"],
            "--package",
            package,
            "--json",
        ]
        packager = [
            str(tool),
            "dokime",
            "package",
            "--build-json",
            str(build_json),
            "--output",
            str(directory),
            "--json",
        ]
        produced = archive.native(
            root,
            producer["profile"],
            "package",
            [collector, packager],
            [directory],
            locks=[root / "Cargo.lock"],
            inputs=[messages, metadata, receipt],
            parents=[producer],
            stdout={0: build_json, 1: package_json},
        )
        producers.append(produced)
        for item in json.loads(package_json.read_text(encoding="utf-8")):
            path = Path(item["path"])
            path = path.resolve() if path.is_absolute() else (root / path).resolve()
            relative = path.relative_to(root).as_posix()
            if relative in files:
                raise ValueError(f"plugin fixture output collision: {relative}")
            files[relative] = item["sha256"]
    return producers, files


# === Private Functions === #

# === Entry Point === #
