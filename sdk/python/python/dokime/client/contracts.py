# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Operation Views Of The Native Client's Contract Catalog."""

# === Imports === #

# Standard Library Imports
import json
from functools import cache

# Third Party Imports
# Local Imports
from .. import _native
from ..dynamic import from_contract
from ..operation import Operation

# Type Checking Imports

# === Public Exports === #

__all__ = ["catalog", "operation"]

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


@cache
def catalog() -> tuple[Operation, ...]:
    return tuple(from_contract(contract) for contract in json.loads(_native.contracts()))


def operation(service: str, method: str) -> Operation:
    package = "harmos.host.v1" if service == "Host" else "dokime.v1"
    path = f"/{package}.{service}/{method}"
    for entry in catalog():
        if entry.grpc_path == path:
            return entry
    raise ValueError(f"the native catalog does not contain {path}")


# === Private Functions === #

# === Entry Point === #
