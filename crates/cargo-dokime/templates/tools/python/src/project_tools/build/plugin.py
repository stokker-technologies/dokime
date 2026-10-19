# ***
#
# {{package}}
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Compile The Plugin Once And Freeze Its Catalog And Assets"""

# === Imports === #

# Standard Library Imports
import argparse
import subprocess
from pathlib import Path

# Third Party Imports
# Local Imports
from .. import temporary
from . import paths, run

# Type Checking Imports

# === Public Exports === #

# === Constants === #

PACKAGE = "{{package}}"

# === Types === #

# === Classes === #

# === Public Functions === #


def build(root: Path, profile: str) -> None:
    # Compile Through The Shared Product Owner
    run.rust(root, profile)
    output = paths.output(root, "build", profile)
    messages = output / "product/compiler.json"
    metadata = messages.with_name("metadata.json")
    with metadata.open("w", encoding="utf-8") as written:
        subprocess.run(["cargo", "metadata", "--locked", "--format-version", "1"], cwd=root, stdout=written, check=True)

    # Freeze The Catalog And Assets From Those Exact Compiler Outputs
    with temporary.directory() as scratch:
        collected = scratch / "collected.json"
        command = ["cargo", "dokime", "build", "--package", PACKAGE, "--profile", profile, "--json"]
        command += ["--from-cargo-json", str(messages), "--metadata-json", str(metadata)]
        with collected.open("w", encoding="utf-8") as written:
            subprocess.run(command, cwd=root, stdout=written, check=True)
        bundles = scratch / "bundles"
        subprocess.run(["cargo", "dokime", "package", "--build-json", str(collected), "--output", str(bundles)], cwd=root, check=True)
        paths.replace(root, bundles, output / "plugins", origin=scratch)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", choices=("dev", "release"), default="dev")
    options = parser.parse_args()
    build(Path.cwd(), options.profile)


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    main()
