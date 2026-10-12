# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Select exactly one matching wheel artifact."""

# === Imports === #

# Standard Library Imports
from pathlib import Path

# Third Party Imports

# Local Imports

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class WheelSelectionError(Exception):
    """The artifact directory did not contain exactly one matching wheel."""

    def __init__(self, wheels: list[Path]) -> None:
        self.wheels = wheels
        super().__init__(", ".join(path.name for path in wheels))


# === Public Functions === #


def select_wheel(directory: Path, pattern: str) -> Path:
    wheels = sorted(directory.glob(pattern))
    if len(wheels) != 1:
        raise WheelSelectionError(wheels)
    return wheels[0]


# === Private Functions === #

# === Entry Point === #
