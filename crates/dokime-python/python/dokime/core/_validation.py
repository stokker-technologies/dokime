# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Native validation helpers for Python wrappers."""

# === Imports === #

# Standard Library Imports
import uuid
from collections.abc import Callable
from typing import cast

# Third Party Imports

# Local Imports
from dokime import _native

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def validate(method_name: str, payload: object) -> object:
    method = cast(Callable[[object], object], getattr(_native, method_name))
    return method(payload)


def new_id(method_name: str) -> str:
    method = cast(Callable[[], str], getattr(_native, method_name))
    return str(method())


def stable_id(namespace: str, value: str) -> str:
    return str(uuid.uuid5(uuid.NAMESPACE_URL, f"dokime:{namespace}:{value}"))


# === Private Functions === #

# === Entry Point === #
