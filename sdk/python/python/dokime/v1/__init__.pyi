# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

# === Imports === #

# Standard Library Imports
from typing import Any

# Third Party Imports
# Local Imports
# Type Checking Imports

# === Public Exports === #

__all__ = ["enum_value", "message"]

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #

def message(name: str, **fields: Any) -> Any: ...
def enum_value(enum: str, name: str) -> int: ...

# === Private Functions === #

# === Entry Point === #
