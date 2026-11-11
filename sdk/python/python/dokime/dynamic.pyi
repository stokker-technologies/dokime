# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

# === Imports === #

# Standard Library Imports
from collections.abc import Mapping
from typing import Any

# Third Party Imports
# Local Imports
from .operation import Operation

# Type Checking Imports

# === Public Exports === #

__all__ = ["from_contract"]

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #

def from_contract(contract: Mapping[str, Any], *, generation: int = 0) -> Operation: ...
