# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

# === Imports === #

# Standard Library Imports
from collections.abc import Sequence
from dataclasses import dataclass
from datetime import datetime
from enum import StrEnum
from typing import Any

# Third Party Imports
# Local Imports
from .._transport import Transport
from ..models import ResultSummary
from .evidence import cycle_records, execution_records, telemetry
from .reader import QueryReader, QuerySettings
from .selection import At, Time, Where, Window

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

type Alternatives = str | Sequence[str] | None
type CycleStatuses = CycleStatus | str | Sequence[CycleStatus | str] | None
type ExecutionStatuses = ExecutionStatus | str | Sequence[ExecutionStatus | str] | None

# === Classes === #

class CycleStatus(StrEnum):
    UNSPECIFIED = "unspecified"
    READY = "ready"
    RUNNING = "running"
    PAUSED = "paused"
    STOPPING = "stopping"
    COMPLETED = "completed"
    FAILED = "failed"
    BLOCKED = "blocked"
    STOPPED = "stopped"

class ExecutionStatus(StrEnum):
    UNSPECIFIED = "unspecified"
    INVALIDATED = "invalidated"
    PENDING = "pending"
    RUNNING = "running"
    PASSED = "passed"
    FAILED = "failed"
    BLOCKED = "blocked"
    CANCELLED = "cancelled"
    INTERRUPTED = "interrupted"
    SKIPPED = "skipped"

@dataclass(frozen=True, slots=True)
class ExecutionSummary:
    id: str
    cycle_id: str | None
    case_id: str
    template_id: str | None
    label: str | None
    status: ExecutionStatus
    started_at: datetime | None
    finished_at: datetime | None
    result: ResultSummary | None
    availability: str
    checkpoint_available: bool
    archived: bool
    _transport: Transport

    query_records = execution_records
    telemetry = telemetry

@dataclass(frozen=True, slots=True)
class CycleSummary:
    id: str
    label: str | None
    phase: str
    created_at: datetime | None
    updated_at: datetime | None
    execution_count: int
    status: CycleStatus
    running_count: int
    archived: bool
    _transport: Transport

    query_records = cycle_records

# === Public Functions === #

def executions(
    transport: Transport,
    settings: QuerySettings,
    *,
    template: Alternatives = None,
    cycle: Alternatives = None,
    status: ExecutionStatuses = None,
    since: Time | None = None,
    until: Time | None = None,
    archive: str = "unarchived",
    where: Where | None = None,
    between: Window | None = None,
    at: At = None,
    page_rows: int | None = None,
    page_bytes: int | None = None,
) -> QueryReader[ExecutionSummary]: ...
def cycles(
    transport: Transport,
    settings: QuerySettings,
    *,
    cycle: Alternatives = None,
    phase: Alternatives = None,
    status: CycleStatuses = None,
    since: Time | None = None,
    until: Time | None = None,
    archive: str = "unarchived",
    where: Where | None = None,
    between: Window | None = None,
    at: At = None,
    page_rows: int | None = None,
    page_bytes: int | None = None,
) -> QueryReader[CycleSummary]: ...
def _status[T: StrEnum](value: Any, kind: type[T]) -> T: ...
def _optional(row: Any, name: str) -> Any: ...
def _bounds(since: Time | None, until: Time | None, between: Window | None) -> dict[str, Any] | None: ...
def _selection(method: str, named: dict[str, Alternatives], where: Where | None, archive: str) -> dict[str, Any]: ...
def _alternatives(value: Any) -> tuple[str, ...]: ...
def _enum(value: str, branches: Any) -> dict[str, Any]: ...
