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
import os
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


def read(path: str | Path | None = None) -> dict[str, Any]:
    path = path if path is not None else os.environ.get("DOKIME_REMOTE")
    if not path:
        raise ValueError("Name a .pair file or set DOKIME_REMOTE")
    return json.loads(_native.read_pairing(Path(path)))


# === Private Functions === #

# === Entry Point === #
