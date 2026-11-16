# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***
"Collect Packages Under Their Matching Profile and Native Target"

# === Imports === #

# Standard Library Imports
import argparse
import json
import subprocess
import sys
from pathlib import Path

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from .. import policy, temporary
from ..build import paths, wheel
from ..process import spawn_flags
from . import assembly, changelog, definition, verification

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def main() -> None:
    parser = argparse.ArgumentParser(description="Assemble Existing Build Outputs")
    parser.add_argument("language", choices=("rust", "python", "typescript", "native"))
    parser.add_argument("--profile", choices=("dev", "release"), default="dev")
    parser.add_argument("--target")
    options = parser.parse_args()
    if options.target == "native":
        options.target = None
    directory = Path.cwd()
    if options.language == "python":
        options.target = "any"
    destination = paths.output(directory, "package", options.profile, options.target)
    target = options.target or paths.target()
    language = "rust" if options.language == "native" else options.language
    products = definition.definitions(directory, options.profile, target)
    if language == "python":
        source = paths.output(directory, "build", options.profile, "any") / "python"
        wheel.verify(directory, source, policy.config(directory)["version"]["value"])
        paths.replace(directory, source, destination / "python")
        return
    if target == "any" and not any(item["language"] == language for item in products):
        staging_directory = temporary.Directory()
        staging = staging_directory.path
        try:
            paths.replace(directory, staging, destination / language, origin=staging)
        finally:
            staging_directory.close()
        return
    if any(item["language"] == language for item in products):
        staging_directory = temporary.Directory()
        staging = staging_directory.path
        try:
            assembly.assemble(directory, language, options.profile, target, staging)
            existing = paths.output(directory, "package", options.profile, target)
            for item in products:
                if item["language"] != language:
                    continue
                foreign = []
                for foreign_path in existing.glob("*/" + item["output"]):
                    if foreign_path.parent.name == language:
                        continue
                    foreign.append(foreign_path)

                if foreign:
                    raise policy.ToolError("Package Product: package output collides with another language")
            paths.replace(directory, staging, destination / language, origin=staging)
        finally:
            staging_directory.close()
        if language == "rust":
            verification.verify(directory, options.profile, target)
        return
    if options.language == "rust":
        metadata = json.loads(
            policy.execute(directory, ["cargo", "metadata", "--no-deps", "--format-version", "1", "--locked"], capture=True)
        )
        members = set(metadata["workspace_members"])
        crates = {}
        for crates_item in metadata["packages"]:
            if crates_item["id"] not in members:
                continue
            if crates_item.get("publish") == []:
                continue
            crates[crates_item["name"]] = crates_item

        scratch_directory = temporary.Directory()
        scratch = scratch_directory.path
        try:
            with changelog.staged(directory, crates):
                subprocess.run(
                    [
                        "cargo",
                        "package",
                        "--workspace",
                        "--locked",
                        "--no-verify",
                        "--allow-dirty",
                        "--target-dir",
                        str(scratch),
                    ],
                    stdin=sys.stdin,
                    stdout=sys.stdout,
                    stderr=sys.stderr,
                    creationflags=spawn_flags(),
                    cwd=directory,
                    check=True,
                )
                paths.replace(
                    directory,
                    scratch / "package",
                    destination / options.language,
                    origin=scratch,
                )
        finally:
            scratch_directory.close()
        return
    else:
        kind = {"typescript": "web", "native": "bin"}[options.language]
        source = paths.output(directory, "build", options.profile, options.target) / kind
    paths.replace(directory, source, destination / options.language)


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    main()
