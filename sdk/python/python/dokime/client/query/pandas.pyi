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
import pandas as pd

# Local Imports
from .reader import QueryReader

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #

async def convert(reader: QueryReader, *, max_rows: int | None = None, max_bytes: int | None = None) -> pd.DataFrame: ...
