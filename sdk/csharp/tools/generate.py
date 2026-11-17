# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Regenerate C# in Memory and Verify the Frozen Outputs Without Rewriting Them."""

# === Imports === #

# Standard Library Imports
from pathlib import Path

# isort: split

# Third Party Imports
from harmos_tools.build.resources.sdk import generate

# isort: split

# Local Imports
from dokime_tools.build.protocol import configuration

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def main() -> None:
    root = Path(__file__).resolve().parents[3]
    generate(
        check=True,
        root=root,
        descriptors=root / "crates/dokime-protocol/src/descriptors.bin",
        configuration=configuration(root),
    )


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    main()
