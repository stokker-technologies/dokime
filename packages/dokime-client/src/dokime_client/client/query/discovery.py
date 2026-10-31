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
from collections.abc import Sequence
from dataclasses import dataclass, field
from datetime import datetime
from typing import Any

# Third Party Imports
# Local Imports
from .. import _wire
from .._transport import Transport
from ..models import ResultSummary
from .evidence import cycle_records, execution_records, telemetry
from .reader import QueryReader, QuerySettings, page_settings
from .selection import At, Time, Where, Window, predicates, snapshot_ref, window

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

type Alternatives = str | Sequence[str] | None

# === Classes === #


@dataclass(frozen=True, slots=True)
class ExecutionSummary:
    id: str
    cycle_id: str | None
    case_id: str
    template_id: str | None
    label: str | None
    status: str
    started_at: datetime | None
    finished_at: datetime | None
    result: ResultSummary | None
    availability: str
    checkpoint_available: bool
    archived: bool
    _transport: Transport = field(repr=False, compare=False)
    _query_settings: QuerySettings = field(repr=False, compare=False)

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
    archived: bool
    _transport: Transport = field(repr=False, compare=False)
    _query_settings: QuerySettings = field(repr=False, compare=False)

    query_records = cycle_records


def executions(
    transport: Transport,
    settings: QuerySettings,
    *,
    template: Alternatives = None,
    cycle: Alternatives = None,
    status: Alternatives = None,
    since: Time | None = None,
    until: Time | None = None,
    archive: str = "unarchived",
    where: Where | None = None,
    between: Window | None = None,
    at: At = None,
    page_rows: int | None = None,
    page_bytes: int | None = None,
) -> QueryReader[ExecutionSummary]:
    selection = _selection("ListExecutions", {"template_id": template, "cycle_id": cycle, "status": status}, where, archive)
    selection["started_at"] = _bounds(since, until, between)

    def decode(row: Any) -> ExecutionSummary:
        return ExecutionSummary(
            row.execution_id,
            _optional(row, "cycle_id"),
            row.case_id,
            _optional(row, "template_id"),
            _optional(row, "label"),
            _wire.enum_name(row.status),
            _wire.timestamp(row, "started_at"),
            _wire.timestamp(row, "finished_at"),
            _wire.result(row.result) if row.HasField("result") else None,
            _wire.enum_name(row.availability),
            row.checkpoint_available,
            row.archived,
            transport,
            settings,
        )

    fields = {"selection": selection, "page": page_settings(settings, page_rows, page_bytes), "at": snapshot_ref(at)}
    return QueryReader(transport, "ListExecutions", fields, decode, settings)


def cycles(
    transport: Transport,
    settings: QuerySettings,
    *,
    cycle: Alternatives = None,
    phase: Alternatives = None,
    since: Time | None = None,
    until: Time | None = None,
    archive: str = "unarchived",
    where: Where | None = None,
    between: Window | None = None,
    at: At = None,
    page_rows: int | None = None,
    page_bytes: int | None = None,
) -> QueryReader[CycleSummary]:
    selection = _selection("ListCycles", {"cycle_id": cycle, "phase": phase}, where, archive)
    selection["created_at"] = _bounds(since, until, between)

    def decode(row: Any) -> CycleSummary:
        return CycleSummary(
            row.cycle_id,
            _optional(row, "label"),
            _wire.enum_name(row.phase),
            _wire.timestamp(row, "created_at"),
            _wire.timestamp(row, "updated_at"),
            row.execution_count,
            row.archived,
            transport,
            settings,
        )

    fields = {"selection": selection, "page": page_settings(settings, page_rows, page_bytes), "at": snapshot_ref(at)}
    return QueryReader(transport, "ListCycles", fields, decode, settings)


# === Public Functions === #

# === Private Functions === #


def _optional(row: Any, name: str) -> Any:
    return getattr(row, name) if row.HasField(name) else None


def _bounds(since: Time | None, until: Time | None, between: Window | None) -> dict[str, Any] | None:
    if between is not None and (since is not None or until is not None):
        raise ValueError("Use between or since/until, not both")
    return window(between if between is not None else (since, until))


def _selection(method: str, named: dict[str, Alternatives], where: Where | None, archive: str) -> dict[str, Any]:
    fields = {"template_id": "template_ids", "cycle_id": "cycle_ids", "status": "statuses", "phase": "phases"}
    selected = {name: _alternatives(value) for name, value in named.items() if value is not None}
    for predicate in predicates(where):
        if predicate.field not in named or predicate.operator not in ("eq", "in"):
            raise ValueError(f"History supports eq/in on {', '.join(named)}; use between for its time window")
        values = _alternatives(predicate.value)
        existing = selected.get(predicate.field)
        selected[predicate.field] = values if existing is None else tuple(value for value in existing if value in values)
        if not selected[predicate.field]:
            raise ValueError("History predicates have no common alternatives")
    selection: dict[str, Any] = {}
    request: Any = _wire.request("Query", method)
    descriptor = request.DESCRIPTOR.fields_by_name["selection"].message_type
    for name, values in selected.items():
        if name in ("status", "phase"):
            enum = descriptor.fields_by_name[fields[name]].message_type
            branches = enum.fields_by_name
            values = tuple(_enum(value, branches) for value in values)
        selection[fields[name]] = values
    selection["archive"] = _enum(archive, ("Unarchived", "Archived", "All"))
    return selection


def _alternatives(value: Any) -> tuple[str, ...]:
    values = (value,) if isinstance(value, str) else value
    if not isinstance(values, (list, tuple)) or not values or any(not isinstance(item, str) or not item for item in values):
        raise ValueError("History filters require a nonempty string or sequence of strings")
    return tuple(dict.fromkeys(values))


def _enum(value: str, branches: Any) -> dict[str, Any]:
    member = "".join(part.title() for part in value.split("_"))
    if member not in branches or member == "Unspecified":
        raise ValueError(
            f"Invalid history value {value!r}; use one of {', '.join(name.lower() for name in branches if name != 'Unspecified')}"
        )
    return {member: {}}


# === Entry Point === #
