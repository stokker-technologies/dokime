# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Lint Rust Selections That Embed Desktop Frontend Assets"""

# === Imports === #

# Standard Library Imports
import sys
from collections.abc import Sequence

# Third Party Imports
# Local Imports
from dokime_tools.build import cargo, frontend

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def desktop(arguments: Sequence[str]) -> None:
    cargo.clippy(frontend.ROOT, arguments)


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    arguments = sys.argv[1:]
    desktop(arguments[1:] if arguments[:1] == ["--"] else arguments)
