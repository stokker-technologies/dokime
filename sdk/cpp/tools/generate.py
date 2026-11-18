# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Check C++ Descriptor Projections Without Changing Frozen Outputs."""

# === Imports === #

# Standard Library Imports
from pathlib import Path

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from dokime_tools.build.protocol import configuration, generate

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
