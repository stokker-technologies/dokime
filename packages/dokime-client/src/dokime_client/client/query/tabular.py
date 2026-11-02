# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Lazy Optional Dataframe Materialization"""

# === Imports === #

# Standard Library Imports
from importlib import import_module
from types import ModuleType
from typing import Literal

# Third Party Imports
# Local Imports
from .errors import QueryDependencyError
from .reader import QueryReader

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


async def materialize[T](
    reader: QueryReader[T],
    extra: Literal["polars", "pandas"],
    *,
    max_rows: int | None = None,
    max_bytes: int | None = None,
) -> tuple[ModuleType, list[T]]:
    """Load Only The Selected Extra Before Consuming Within The Reader's Budgets."""
    try:
        library = import_module(extra)
    except ModuleNotFoundError as error:
        if error.name != extra:
            raise
        raise QueryDependencyError(extra) from error
    rows = await reader.collect(max_rows=max_rows, max_bytes=max_bytes)
    return library, rows


# === Private Functions === #

# === Entry Point === #
