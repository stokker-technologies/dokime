# ***
#
# dokime-rust-workspace
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Lossless Scalar Conversion At The Protobuf Boundary."""

# === Imports === #

# Standard Library Imports
from collections.abc import Mapping
from dataclasses import dataclass
from datetime import UTC, datetime
from enum import IntEnum
from fractions import Fraction
from typing import TYPE_CHECKING

# Third Party Imports
from google.protobuf.timestamp_pb2 import Timestamp

# Local Imports
from ._generated.shared import ExactNumber, Null, Value, ValueList

# Type Checking Imports
if TYPE_CHECKING:
    from ._payloads import Payload

# === Public Exports === #

__all__ = ["decode", "encode"]

# === Constants === #

# === Types === #

type Scalar = None | bool | int | float | str | bytes | Fraction | datetime | list[Scalar] | tuple[Scalar, ...]

# === Classes === #


class ResultStatus(IntEnum):
    UNSPECIFIED = 0
    PENDING = 1
    PASSED = 2
    FAILED = 3
    BLOCKED = 4
    CANCELLED = 5


class Termination(IntEnum):
    UNSPECIFIED = 0
    COMPLETED = 1
    FAILED = 2
    CANCELLED = 3
    INTERRUPTED = 4


@dataclass(frozen=True)
class Outcome:
    status: ResultStatus
    attribution: str
    reason: str
    termination: Termination | None = None
    importer: str | None = None


@dataclass(frozen=True)
class Progress:
    phase: str
    completed: int
    total: int
    label: str
    sealed: bool


@dataclass(frozen=True)
class Note:
    identifier: str
    text: str
    author: str | None
    created_at: datetime
    cycle: str
    execution: str | None


@dataclass(frozen=True)
class Artifact:
    identifier: str
    name: str
    mime: str
    size_bytes: int
    sha256: bytes
    cycle: str | None
    execution: str | None


@dataclass(frozen=True)
class PromptQuestion:
    identifier: str
    text: str
    choices: tuple[str, ...]
    execution: str


@dataclass(frozen=True)
class PromptAnswer:
    choice: str


@dataclass(frozen=True)
class RecordRow[T: "Payload"]:
    identifier: str
    sequence: int
    timestamp_ns: int
    received_at_ns: int
    record_type: str
    schema_version: int
    source: str
    value: T


@dataclass(frozen=True)
class SampleRow[T: "Payload"]:
    identifier: str
    sequence: int
    timestamp_ns: int
    received_at_ns: int
    stream: str
    schema_version: int
    source: str
    value: T
    values: Mapping[str, Scalar]


# === Public Functions === #


def encode(value: Scalar) -> Value:
    if value is None:
        return Value(null=Null())
    if isinstance(value, bool):
        return Value(boolean=value)
    if isinstance(value, Fraction):
        return Value(exact=ExactNumber(negative=value < 0, numerator=str(abs(value.numerator)), denominator=str(value.denominator)))
    if isinstance(value, int):
        if value < -(2**63) or value >= 2**64:
            return encode(Fraction(value))
        return Value(unsigned_integer=value) if value >= 2**63 else Value(integer=value)
    if isinstance(value, float):
        return Value(number=value)
    if isinstance(value, str):
        return Value(text=value)
    if isinstance(value, bytes):
        return Value(data=value)
    if isinstance(value, (list, tuple)):
        return Value(list=ValueList(values=[encode(item) for item in value]))
    if isinstance(value, datetime):
        if value.tzinfo is None:
            raise ValueError("Datetime values require a timezone")
        timestamp = Timestamp()
        timestamp.FromDatetime(value)
        return Value(datetime=timestamp)
    raise TypeError(f"Unsupported Dokime value: {type(value).__name__}")


def decode(value: Value) -> Scalar:
    kind = value.WhichOneof("kind")
    if kind is None:
        raise ValueError("Dokime Value has no kind")
    if kind == "null":
        return None
    if kind == "list":
        return [decode(item) for item in value.list.values]
    if kind == "exact":
        return Fraction((-1 if value.exact.negative else 1) * int(value.exact.numerator), int(value.exact.denominator))
    if kind == "datetime":
        return value.datetime.ToDatetime(tzinfo=UTC)
    if kind == "boolean":
        return value.boolean
    if kind == "integer":
        return value.integer
    if kind == "unsigned_integer":
        return value.unsigned_integer
    if kind == "number":
        return value.number
    if kind == "text":
        return value.text
    if kind == "data":
        return value.data
    raise ValueError(f"Decode Value: Unsupported Kind {kind}")


# === Private Functions === #

# === Entry Point === #
