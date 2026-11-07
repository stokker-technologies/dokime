# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

# === Imports === #

# Standard Library Imports
from dataclasses import dataclass
from typing import Any

# Third Party Imports
# Local Imports
from .reader import QueryReader

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

@dataclass(slots=True)
class Column:
    name: str
    kind: str
    values: list[Any]

# === Public Functions === #

def columns(reader: QueryReader, rows: list[Any]) -> list[Column]: ...
def metadata(reader: QueryReader) -> dict[str, Any]: ...
def _cell(row: Any, name: str) -> Any: ...
def _value(name: str, kind: str, nullable: bool, value: Any) -> Any: ...
