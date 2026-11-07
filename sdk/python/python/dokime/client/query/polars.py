# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Explicit Captured Dtypes For Optional Polars Conversion"""

# === Imports === #

# Standard Library Imports
from typing import TYPE_CHECKING, Any

# Third Party Imports
# Local Imports
from .errors import QueryConversionError
from .frames import columns
from .reader import QueryReader
from .tabular import materialize

# Type Checking Imports
if TYPE_CHECKING:
    import polars as pl


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


async def convert(reader: QueryReader, *, max_rows: int | None = None, max_bytes: int | None = None) -> "pl.DataFrame":
    library, rows = await materialize(reader, "polars", max_rows=max_rows, max_bytes=max_bytes)
    module: Any = library
    dtypes = {
        "Boolean": module.Boolean,
        "Integer": module.Int128,
        "Int64": module.Int64,
        "Uint64": module.UInt64,
        "Float64": module.Float64,
        "Text": module.String,
        "Severity": module.String,
        "Datetime": module.Datetime("ns", "UTC"),
    }
    series = []
    for column in columns(reader, rows):
        try:
            # Integer Epoch Nanoseconds Avoid Python Datetime's Microsecond Projection.
            dtype = module.Int64 if column.kind == "Datetime" else dtypes[column.kind]
            values = module.Series(column.name, column.values, dtype=dtype, strict=True)
            if column.kind == "Datetime":
                values = values.cast(dtypes["Datetime"], strict=True)
            series.append(values)
        except (TypeError, ValueError, OverflowError, module.exceptions.PolarsError) as error:
            raise QueryConversionError(column.name, f"unsupported {column.kind} conversion: {error}") from error
    return module.DataFrame(series)


# === Private Functions === #

# === Entry Point === #
