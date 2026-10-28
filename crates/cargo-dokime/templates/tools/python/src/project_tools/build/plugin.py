# ***
#
# {{package}}
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Compile The Plugin Once And Freeze Its Catalog And Assets"""


import argparse
import subprocess
import sys
from pathlib import Path

from .. import temporary
from . import paths
from .rust.__main__ import build as build_rust


PACKAGE = "{{package}}"


def build(root: Path, profile: str) -> None:
    # Compile Through The Shared Product Owner
    build_rust(root, profile)
    output = paths.output(root, "build", profile)
    messages = output / "product/compiler.json"
    metadata = messages.with_name("metadata.json")
    with metadata.open("w", encoding="utf-8") as written:
        subprocess.run(
            ["cargo", "metadata", "--locked", "--format-version", "1"],
            cwd=root,
            stdout=written,
            check=True,
            creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
            stdin=sys.stdin,
            stderr=sys.stderr,
        )

    # Freeze The Catalog And Assets From Those Exact Compiler Outputs
    with temporary.directory() as scratch:
        collected = scratch / "collected.json"
        command = [
            "cargo",
            "dokime",
            "sidecar",
            "build",
            "--package",
            PACKAGE,
            "--profile",
            profile,
            "--json",
        ]
        command += [
            "--from-cargo-json",
            str(messages),
            "--metadata-json",
            str(metadata),
        ]
        with collected.open("w", encoding="utf-8") as written:
            subprocess.run(
                command,
                cwd=root,
                stdout=written,
                check=True,
                creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
                stdin=sys.stdin,
                stderr=sys.stderr,
            )
        bundles = scratch / "bundles"
        subprocess.run(
            [
                "cargo",
                "dokime",
                "sidecar",
                "package",
                "--build-json",
                str(collected),
                "--output",
                str(bundles),
            ],
            cwd=root,
            check=True,
            creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
            stdin=sys.stdin,
            stdout=sys.stdout,
            stderr=sys.stderr,
        )
        paths.replace(root, bundles, output / "plugins", origin=scratch)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", choices=("dev", "release"), default="dev")
    options = parser.parse_args()
    build(Path.cwd(), options.profile)


if __name__ == "__main__":
    main()
