# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

# === Imports === #

# Standard Library Imports
from types import ModuleType
from typing import Literal

# Third Party Imports
# Local Imports
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
) -> tuple[ModuleType, list[T]]: ...
