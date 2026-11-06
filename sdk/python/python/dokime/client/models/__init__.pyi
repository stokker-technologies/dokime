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
from datetime import date, datetime
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
    _wire_text: str | None = None
    _nanosecond_remainder: int = 0

    @classmethod
    def from_rfc3339(cls, text: str) -> DatetimeValue: ...
    def __eq__(self, other: object) -> bool: ...
    def __hash__(self) -> int: ...
    def __lt__(self, other: date) -> bool: ...
    def __le__(self, other: date) -> bool: ...
    def __gt__(self, other: date) -> bool: ...
    def __ge__(self, other: date) -> bool: ...
    def __ne__(self, other: object) -> bool: ...

class UnsignedValue(int):
    def __new__(cls, value: int) -> Self: ...

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
    def terminal(self) -> bool: ...

@dataclass(frozen=True)
class ExecutionCommandReceipt:
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
    def terminal(self) -> bool: ...

@dataclass(frozen=True)
class Backend:
    id: str
    epoch: str
    api_version: str
    display_name: str
    methods: frozenset[str]
    product_version: str = ""
    source_revision: str | None = None
    channel: str | None = None
    sdk_revision: str | None = None
    descriptor_sha256: bytes = b""
    max_evidence_bytes: int | None = None

@dataclass(frozen=True)
class Template:
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

@dataclass(frozen=True)
class ResultSummary:
    status: str
    passed: int
    failed: int
    blocked: int
    skipped: int

@dataclass(frozen=True)
class Record:
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
    received_at: datetime | None = None

    def to_json(self) -> str: ...

@dataclass(frozen=True)
class SavedRecord:
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

    def to_json(self) -> str: ...

@dataclass(frozen=True)
class SavedSample:
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

def _json_value(value: object) -> str: ...
