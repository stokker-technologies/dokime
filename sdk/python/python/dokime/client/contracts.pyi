# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

# === Imports === #

# Standard Library Imports

# Third Party Imports
# Local Imports
from ..operation import Operation

# Type Checking Imports

# === Public Exports === #

__all__ = ["catalog", "operation"]

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #

def catalog() -> tuple[Operation, ...]: ...
def operation(service: str, method: str) -> Operation: ...
