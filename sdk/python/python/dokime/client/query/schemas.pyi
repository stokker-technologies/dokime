# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

# === Imports === #

# Standard Library Imports
from collections.abc import AsyncGenerator
from dataclasses import dataclass
from enum import StrEnum
from typing import Any

# Third Party Imports
# Local Imports
from .._transport import Transport

# Type Checking Imports

# === Public Exports === #

# === Constants === #

definitions: Any
catalog: Any

# === Types === #

# === Classes === #

class RecordValueType(StrEnum):
    UNSPECIFIED = "unspecified"
    BOOLEAN = "boolean"
    INTEGER = "integer"
    NUMBER = "number"
    TEXT = "text"
    FILE = "file"
    OBJECT = "object"
    ARRAY = "array"
    DATETIME = "datetime"

@dataclass(frozen=True, slots=True)
class RecordField:
    identifier: str
    value_type: RecordValueType
    label: str | None
    unit: str | None
    required: bool
    nullable: bool

@dataclass(frozen=True, slots=True)
class RecordSchema:
    record_type: str
    schema_version: int
    label: str | None
    fields: tuple[RecordField, ...]

# === Public Functions === #

def record_schemas(transport: Transport, *, page_rows: int) -> AsyncGenerator[RecordSchema]: ...
def _schemas(transport: Transport, page_rows: int) -> AsyncGenerator[RecordSchema]: ...
def _label(item: Any) -> str | None: ...
def _field(item: Any) -> RecordField: ...
