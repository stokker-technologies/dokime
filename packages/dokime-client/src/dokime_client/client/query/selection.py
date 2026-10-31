# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Finite Query Client"""

# === Imports === #

# Standard Library Imports
import math
import re
from collections.abc import Mapping, Sequence
from dataclasses import dataclass
from datetime import UTC, datetime
from typing import Any, Literal, overload

# Third Party Imports
from google.protobuf.message import Message

# Local Imports
from ..history import Checkpoint
from ..models import UnsignedValue

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

type Time = str | datetime
type Window = tuple[Time | None, Time | None]
type Scalar = bool | int | float | str | datetime
type Where = Mapping[str, Scalar] | Sequence["Predicate"]
type Operator = Literal["eq", "lt", "lte", "gt", "gte", "in", "exists"]
type At = Message | Checkpoint | None

# === Classes === #


@dataclass(frozen=True, slots=True)
class Predicate:
    """One Conjunctive Predicate On A Literal Captured Field ID."""

    field: str
    operator: Operator
    value: Scalar | Sequence[Scalar]

    def __post_init__(self) -> None:
        if not isinstance(self.field, str) or not self.field:
            raise ValueError("Predicate requires a nonempty field ID")
        if self.operator not in ("eq", "lt", "lte", "gt", "gte", "in", "exists"):
            raise ValueError("Use eq/lt/lte/gt/gte/in/exists predicates")
        if self.operator == "in":
            if not isinstance(self.value, (list, tuple)) or not self.value:
                raise ValueError("IN requires a nonempty scalar sequence")
            object.__setattr__(self, "value", tuple(self.value))
        elif self.operator == "exists" and not isinstance(self.value, bool):
            raise ValueError("exists requires a boolean; use exists(false) for missing/null")
        self.to_wire()

    def to_wire(self) -> dict[str, Any]:
        values = self.value if self.operator == "in" and isinstance(self.value, (list, tuple)) else (self.value,)
        return {
            "field": self.field,
            "operator": {self.operator.title(): {}},
            "values": [_scalar(value, self.field) for value in values],
        }


# === Public Functions === #


def predicates(where: Where | None) -> tuple[Predicate, ...]:
    if where is None:
        return ()
    if isinstance(where, Mapping):
        return tuple(Predicate(field, "eq", value) for field, value in where.items())
    if isinstance(where, (str, bytes)) or not isinstance(where, Sequence) or any(not isinstance(p, Predicate) for p in where):
        raise TypeError("where requires an equality map or a sequence of Predicate objects")
    return tuple(where)


def timestamp(value: Time) -> dict[str, int]:
    nanos = 0
    if isinstance(value, str):
        if re.fullmatch(r"\d{4}-\d{2}-\d{2}", value):
            value += "T00:00:00Z"
        match = re.fullmatch(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.(\d{1,9}))?(?:Z|[+-]\d{2}:\d{2})", value)
        if match is None:
            raise ValueError("Use an aware RFC3339 time (at most nine fractional digits) or YYYY-MM-DD")
        nanos = int((match[1] or "").ljust(9, "0"))
        value = datetime.fromisoformat(value)
    elif isinstance(value, datetime):
        nanos = value.microsecond * 1000 + getattr(value, "_nanosecond_remainder", 0)
    else:
        raise TypeError("Time bounds require an aware datetime or RFC3339/date string")
    if value.tzinfo is None or value.utcoffset() is None:
        raise ValueError("Naive datetimes are invalid; supply a UTC offset")
    delta = value.astimezone(UTC).replace(microsecond=0) - datetime(1970, 1, 1, tzinfo=UTC)
    return {"seconds": delta.days * 86400 + delta.seconds, "nanos": nanos}


def window(between: Window | None) -> dict[str, Any] | None:
    if between is None:
        return None
    if not isinstance(between, tuple) or len(between) != 2:
        raise ValueError("between requires a (start, end) pair; either bound may be None")
    start, end = (timestamp(value) if value is not None else None for value in between)
    if start is not None and end is not None and (start["seconds"], start["nanos"]) >= (end["seconds"], end["nanos"]):
        raise ValueError("The start must be strictly before the exclusive end")
    return {"start": start, "end": end}


def snapshot_ref(at: At = None) -> dict[str, Any] | None:
    if at is None:
        return None
    if isinstance(at, Checkpoint):
        return {"Checkpoint": {"field_0": at._wire()}}
    if not isinstance(at, Message) or at.DESCRIPTOR is None or set(at.DESCRIPTOR.fields_by_name) != {"token", "identity", "expires_at"}:
        raise TypeError("at requires a generated query snapshot or a saved Checkpoint")
    snapshot: Any = at
    token = snapshot.token
    if not token:
        raise ValueError("The snapshot token is empty")
    return {"Token": {"field_0": token}}


@overload
def positive(value: int, name: str, maximum: int = 2**64 - 1) -> int: ...


@overload
def positive(value: None, name: str, maximum: int = 2**64 - 1) -> None: ...


def positive(value: int | None, name: str, maximum: int = 2**64 - 1) -> int | None:
    if value is not None and (type(value) is not int or not 0 < value <= maximum):
        raise ValueError(f"{name} must be a positive integer at most {maximum}")
    return value


# === Private Functions === #


def _scalar(value: object, field: str) -> dict[str, Any]:
    if value is None:
        raise ValueError("Null operands are invalid; use Predicate(field, 'exists', False)")
    if isinstance(value, bool):
        return {"Boolean": {"field_0": value}}
    if isinstance(value, int):
        if not -(2**63) <= value < 2**64:
            raise ValueError("Integer operands must fit int64/uint64")
        member = "Unsigned" if isinstance(value, UnsignedValue) or value >= 2**63 else "Integer"
        return {member: {"field_0": int(value)}}
    if isinstance(value, float) and math.isfinite(value):
        return {"Number": {"field_0": value}}
    if isinstance(value, datetime):
        return {"Datetime": {"field_0": timestamp(value)}}
    if isinstance(value, str):
        if field in ("@timestamp", "@received_at"):
            return {"Datetime": {"field_0": timestamp(value)}}
        if field == "@severity":
            if value not in ("trace", "debug", "info", "warn", "error"):
                raise ValueError("Use a declared severity: trace/debug/info/warn/error")
            return {"Severity": {"field_0": {value.title(): {}}}}
        return {"Text": {"field_0": value}}
    raise TypeError("Predicate operands must be finite typed scalars; use exists for arrays/objects/null")


# === Entry Point === #
