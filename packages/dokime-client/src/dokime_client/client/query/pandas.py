# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Explicit Captured Dtypes For Optional Pandas Conversion"""

# === Imports === #

# Standard Library Imports
from typing import TYPE_CHECKING, Any

# Third Party Imports
# Local Imports
from .errors import QueryConversionError
from .frames import columns, metadata
from .reader import QueryReader
from .tabular import materialize

# Type Checking Imports
if TYPE_CHECKING:
    import pandas as pd


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


async def convert(reader: QueryReader, *, max_rows: int | None = None, max_bytes: int | None = None) -> "pd.DataFrame":
    library, rows = await materialize(reader, "pandas", max_rows=max_rows, max_bytes=max_bytes)
    module: Any = library
    dtypes = {
        "Boolean": "boolean",
        "Integer": "object",
        "Int64": "Int64",
        "Uint64": "UInt64",
        "Float64": "Float64",
        "Text": "string",
        "Severity": "string",
        "Datetime": "datetime64[ns, UTC]",
    }
    data = {}
    for column in columns(reader, rows):
        try:
            values = column.values
            if column.kind == "Datetime":
                values = [module.Timestamp(value, unit="ns", tz="UTC") if value is not None else module.NaT for value in values]
            # Object Construction Prevents A Float Intermediate For Nullable Large Integers.
            objects = module.Series(values, dtype=object)
            if column.kind == "Float64":
                # Preserve Non-Finite Values Separately From Explicit Missing Values.
                numbers = module.Series(values, dtype="float64").to_numpy()
                missing = module.Series([value is None for value in values], dtype=bool).to_numpy()
                data[column.name] = module.Series(module.arrays.FloatingArray(numbers, missing))
            else:
                data[column.name] = module.Series(module.array(objects, dtype=dtypes[column.kind]))
        except (TypeError, ValueError, OverflowError) as error:
            raise QueryConversionError(column.name, f"unsupported {column.kind} conversion: {error}") from error
    frame = module.DataFrame(data)
    frame.attrs["dokime"] = metadata(reader)
    return frame


# === Private Functions === #

# === Entry Point === #
