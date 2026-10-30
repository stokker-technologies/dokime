# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Public Python values returned by the remote client."""

# === Imports === #

# Standard Library Imports
from __future__ import annotations

import json
import re
from dataclasses import dataclass, fields
from datetime import datetime
from enum import StrEnum
from typing import Any, Self, TypedDict

# Third Party Imports

# Local Imports

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

type Value = None | bool | int | float | str | datetime | list[Value] | dict[str, Value]

# === Classes === #


class DatetimeValue(datetime):
    """A datetime retaining sub-microsecond precision for an unchanged wire round trip."""

    _wire_text: str | None = None
    _nanosecond_remainder: int = 0

    @classmethod
    def from_rfc3339(cls, text: str) -> DatetimeValue:
        date = datetime.fromisoformat(text)
        if date.tzinfo is None or date.utcoffset() is None:
            raise ValueError("datetimes must include a timezone")
        result = cls(
            date.year,
            date.month,
            date.day,
            date.hour,
            date.minute,
            date.second,
            date.microsecond,
            tzinfo=date.tzinfo,
            fold=date.fold,
        )
        result._wire_text = text
        fraction = re.search(r"\.(\d+)", text)
        if fraction:
            result._nanosecond_remainder = int(fraction.group(1).ljust(9, "0")[:9]) % 1000
        return result

    def __eq__(self, other: object) -> bool:
        return (
            isinstance(other, datetime)
            and datetime.__eq__(self, other)
            and self._nanosecond_remainder == getattr(other, "_nanosecond_remainder", 0)
        )

    def __hash__(self) -> int:
        projected = datetime.__hash__(self)
        return hash((projected, self._nanosecond_remainder)) if self._nanosecond_remainder else projected

    def __lt__(self, other: datetime) -> bool:
        if datetime.__eq__(self, other):
            return self._nanosecond_remainder < getattr(other, "_nanosecond_remainder", 0)
        return datetime.__lt__(self, other)

    def __le__(self, other: datetime) -> bool:
        return self == other or self < other

    def __gt__(self, other: datetime) -> bool:
        if datetime.__eq__(self, other):
            return self._nanosecond_remainder > getattr(other, "_nanosecond_remainder", 0)
        return datetime.__gt__(self, other)

    def __ge__(self, other: datetime) -> bool:
        return self == other or self > other

    def __ne__(self, other: object) -> bool:
        return not self == other


class UnsignedValue(int):
    """An unsigned 64-bit value, retaining its wire kind even below 2**63."""

    def __new__(cls, value: int) -> Self:
        if not 0 <= value < 2**64:
            raise ValueError("unsigned values must fit the 64-bit range")
        return super().__new__(cls, value)


class CommandAction(StrEnum):
    NEXT = "next"
    RUN_STEP = "run_step"
    FINISH = "finish"


class CommandPhase(StrEnum):
    ACCEPTED = "accepted"
    DISPATCHING = "dispatching"
    RUNNING = "running"
    COMPLETED = "completed"
    REFUSED = "refused"
    FAILED = "failed"
    INTERRUPTED = "interrupted"

    @property
    def terminal(self) -> bool:
        return self in {self.COMPLETED, self.REFUSED, self.FAILED, self.INTERRUPTED}


@dataclass(frozen=True)
class ExecutionCommandReceipt:
    """One command's progress; terminal does not certify execution sealing."""

    request_id: str
    command_id: str
    execution_id: str
    action: CommandAction
    step_id: str | None
    phase: CommandPhase
    attempt_id: str | None
    accepted_at: datetime | None
    updated_at: datetime | None
    detail: str | None

    @property
    def terminal(self) -> bool:
        return self.phase.terminal


@dataclass(frozen=True)
class Backend:
    id: str
    epoch: str
    api_version: str
    display_name: str
    methods: frozenset[str]
    product_version: str = ""
    source_revision: str | None = None
    sdk_revision: str | None = None
    descriptor_sha256: bytes = b""
    max_evidence_bytes: int | None = None


@dataclass(frozen=True)
class Template:
    # The readable `<suite id>.<segment>` reference, never the registered UUID.
    id: str
    label: str
    parameters: tuple[str, ...]


@dataclass(frozen=True)
class Suite:
    id: str
    identifier: str
    label: str
    templates: tuple[Template, ...]


@dataclass(frozen=True)
class Prompt:
    id: str
    type: str
    title: str
    message: str | None
    actions: tuple[str, ...]
    fields: tuple[str, ...]
    values: dict[str, Value]
    level: str
    """The author's declared level, named like `Record.severity`: "warn" for Warning."""


@dataclass(frozen=True)
class ResultSummary:
    """Backend-provided result counters."""

    status: str
    passed: int
    failed: int
    blocked: int
    skipped: int


@dataclass(frozen=True)
class Record:
    """One captured occurrence. Type identifies the event kind; id is unique."""

    id: str
    type: str
    sequence: int
    schema_version: int
    source: str
    severity: str
    timestamp: datetime | None
    message: str | None
    details: dict[str, Value]
    scope: dict[str, Any]
    error: dict[str, object] | None
    related_sample_id: str | None
    related_stream_id: str | None
    related_artifact_id: str | None

    def to_json(self) -> str:
        """Serialize This Occurrence With RFC 3339 Datetime Values."""
        return json.dumps({field.name: getattr(self, field.name) for field in fields(self)}, default=_json_value)


@dataclass(frozen=True)
class SavedRecord:
    """One native saved occurrence with its original producer epoch."""

    epoch: str
    record: Record


@dataclass(frozen=True)
class Sample:
    id: str
    stream: str
    sequence: int
    schema_version: int
    source: str
    timestamp: datetime | None
    received_at: datetime | None
    values: dict[str, Value]
    summary: dict[str, Value]
    metadata: dict[str, Value]
    scope: dict[str, Any]

    def to_json(self) -> str:
        """Serialize This Sample With RFC 3339 Datetime Values."""
        return json.dumps({field.name: getattr(self, field.name) for field in fields(self)}, default=_json_value)


@dataclass(frozen=True)
class SavedSample:
    """One saved sample with its original producer epoch and ordinal."""

    epoch: str
    ordinal: int
    sample: Sample


class SavedFile(TypedDict):
    path: str
    size: int
    sha256: str


class SaveManifest(TypedDict):
    cycle_id: str
    files: list[SavedFile]


# === Public Functions === #

# === Private Functions === #


def _json_value(value: object) -> str:
    if isinstance(value, DatetimeValue) and value._wire_text is not None:
        return value._wire_text
    if isinstance(value, datetime):
        return value.isoformat()
    raise TypeError(f"unsupported JSON value: {type(value).__name__}")


# === Entry Point === #
