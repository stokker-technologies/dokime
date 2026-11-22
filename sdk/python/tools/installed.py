# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Install The One Built Wheel Into The Shared Environment."""

# === Imports === #

# Standard Library Imports
import os
from pathlib import Path

# Third Party Imports
# Local Imports
from dokime_tools.test.archive import install_wheels

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def main() -> None:
    root = Path(__file__).resolve().parents[3]
    install_wheels(root, os.environ.get("DOKIME_BUILD_PROFILE", "dev"))


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    main()
