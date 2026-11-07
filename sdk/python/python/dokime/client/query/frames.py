# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Captured Schema Columns Without Optional Library Imports"""

# === Imports === #

# Standard Library Imports
from dataclasses import dataclass
from datetime import datetime
from typing import Any

# Third Party Imports
# Local Imports
from .. import _wire
from ..models import Record, Sample
from .errors import QueryConversionError
from .evidence import Bucket
from .reader import QueryReader
from .selection import timestamp

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


def columns(reader: QueryReader, rows: list[Any]) -> list[Column]:
    page = reader.metadata
    if page is None or page.schema is None:
        raise QueryConversionError("schema", "captured evidence schema is required")
    bucketed = reader._request.HasField("bucket") if reader._operation.id == "dokime.query.list_samples" else False
    envelopes = (
        [("@start", "Datetime"), ("@end", "Datetime"), ("@execution_id", "Text"), ("@source", "Text"), ("@sample_count", "Uint64")]
        if bucketed
        else [("@id", "Text"), ("@timestamp", "Datetime"), ("@received_at", "Datetime"), ("@source", "Text"), ("@schema_version", "Uint64")]
    )
    if reader._operation.id == "dokime.query.list_records":
        envelopes.append(("@severity", "Severity"))
    definitions = [(name, kind, True) for name, kind in envelopes]
    definitions.extend((field.id, _wire.variant(field.value_type), field.nullable) for field in page.schema.fields)
    result: list[Column] = []
    names: set[str] = set()
    for name, kind, nullable in definitions:
        if not name or name in names:
            raise QueryConversionError(name, "empty or duplicate column ID")
        names.add(name)
        if kind not in ("Boolean", "Integer", "Int64", "Uint64", "Float64", "Text", "Datetime", "Severity"):
            raise QueryConversionError(name, f"unsupported captured dtype {kind!r}")
        values = [_value(name, kind, nullable, _cell(row, name)) for row in rows]
        result.append(Column(name, kind, values))
    return result


def metadata(reader: QueryReader) -> dict[str, Any]:
    page = reader.metadata
    if page is None or page.schema is None:
        raise QueryConversionError("schema", "captured evidence schema is required")
    return {
        "schema": page.schema,
        "units": {field.id: field.unit if field.HasField("unit") else None for field in page.schema.fields},
        "versions": tuple(page.schema.versions),
        "projection": tuple(field.id for field in page.schema.fields),
        "snapshot": page.snapshot,
        "complete": reader.complete,
    }


# === Private Functions === #


def _cell(row: Any, name: str) -> Any:
    if name.startswith("@"):
        return getattr(row, name[1:], None)
    if isinstance(row, Record):
        return row.details.get(name)
    if isinstance(row, Sample):
        return row.values.get(name)
    if isinstance(row, Bucket):
        return next((field.value for field in row.fields if field.field == name), None)
    raise QueryConversionError(name, "expected a record, sample or bucket")


def _value(name: str, kind: str, nullable: bool, value: Any) -> Any:
    if value is None:
        if not nullable:
            raise QueryConversionError(name, "null or missing value in a non-nullable field")
        return None
    valid = False
    if kind in ("Integer", "Int64", "Uint64"):
        lower = 0 if kind == "Uint64" else -(2**63)
        upper = 2**63 if kind == "Int64" else 2**64
        valid = isinstance(value, int) and not isinstance(value, bool) and lower <= value < upper
    elif kind == "Boolean":
        valid = isinstance(value, bool)
    elif kind == "Float64":
        valid = isinstance(value, float)
    elif kind in ("Text", "Severity"):
        valid = isinstance(value, str)
    elif kind == "Datetime" and isinstance(value, datetime):
        try:
            instant = timestamp(value)
            nanos = instant["seconds"] * 10**9 + instant["nanos"]
            if not -(2**63) < nanos < 2**63:
                raise ValueError("UTC nanoseconds exceed the supported range")
            return nanos
        except (ValueError, OverflowError) as error:
            raise QueryConversionError(name, str(error)) from error
    if not valid:
        raise QueryConversionError(name, f"value of type {type(value).__name__} does not fit captured {kind}")
    return int(value) if kind in ("Integer", "Int64", "Uint64") else value


# === Entry Point === #
