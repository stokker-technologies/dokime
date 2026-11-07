# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Bounded Version 1 Pairing Input And Certificate Trust Validation"""

# === Imports === #

# Standard Library Imports
import json
from pathlib import Path
from typing import Any

# Third Party Imports
# Local Imports
from .. import _native

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def read(path: str | Path) -> dict[str, Any]:
    return json.loads(_native.read_pairing(Path(path)))


# === Private Functions === #

# === Entry Point === #
