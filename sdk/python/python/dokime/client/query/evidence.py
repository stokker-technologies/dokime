# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Finite Captured Evidence Readers"""

# === Imports === #

# Standard Library Imports
import asyncio
import re
from collections.abc import Sequence
from dataclasses import dataclass
from datetime import datetime
from typing import TYPE_CHECKING, Any, Protocol

# Third Party Imports
# Local Imports
from .. import _wire
from .._contracts import operation
from .._transport import Transport
from ..aggregate import Aggregate
from ..errors import ProtocolError
from ..models import Record, Sample
from . import statistics
from .errors import query_call
from .reader import QueryReader, QuerySettings, page_settings
from .selection import At, Where, Window, positive, predicates, snapshot_ref, window
from .statistics import Statistics, statistic_value

# Type Checking Imports
if TYPE_CHECKING:
    import pandas as pd
    import polars as pl


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


@dataclass(frozen=True, slots=True)
class BucketValue:
    field: str
    count: int
    value: int | float | None
    unit: str | None
    reducer: Aggregate


@dataclass(frozen=True, slots=True)
class Bucket:
    start: datetime | None
    end: datetime | None
    execution_id: str
    stream: str
    source: str
    versions: tuple[int, ...]
    sample_count: int
    fields: tuple[BucketValue, ...]


class EvidenceScope(Protocol):
    @property
    def id(self) -> str: ...

    @property
    def _transport(self) -> Transport: ...


class EvidenceReader[T](QueryReader[T]):
    """Finite Evidence With Optional, Explicitly Bounded Dataframe Conversion."""

    async def to_polars(self, *, max_rows: int | None = None, max_bytes: int | None = None) -> "pl.DataFrame":
        from .polars import convert

        return await convert(self, max_rows=max_rows, max_bytes=max_bytes)

    async def to_pandas(self, *, max_rows: int | None = None, max_bytes: int | None = None) -> "pd.DataFrame":
        from .pandas import convert

        return await convert(self, max_rows=max_rows, max_bytes=max_bytes)


class TelemetryReader(EvidenceReader[Sample | Bucket]):
    """Raw Samples Or Distinct Complete Bucket Rows At A Fixed Cut."""

    statistics: Statistics | None = None

    async def aggregate(self, field: str, aggregates: Sequence[Aggregate]) -> Statistics:
        """Reduce Raw Selected Samples On The Server In One Canonical Request."""
        if self._request.HasField("bucket"):
            raise ValueError("Statistics require raw telemetry; omit every/agg and use Aggregate objects here")
        requested = statistics.outputs(field, aggregates)
        self._claim("aggregate")
        fields = {name: getattr(self._request, name) for name in ("execution_id", "stream", "versions", "where")}
        for name in ("at", "between"):
            if self._request.HasField(name):
                fields[name] = getattr(self._request, name)
        fields["outputs"] = requested
        request = _wire.request("Query", "AggregateSamples", **fields)
        try:
            self._pending = asyncio.create_task(query_call(self._transport, operation("Query", "AggregateSamples"), request))
            response = await self._pending
            if self._closed:
                raise RuntimeError("Query reader was closed during aggregation")
            result = statistics.decode(response, requested)
            if (
                self._request.HasField("at")
                and self._request.at.HasField("Token")
                and result.snapshot.token != self._request.at.Token.field_0
            ):
                raise ProtocolError("Statistics differ from the supplied snapshot")
            self.snapshot = result.snapshot
            self.statistics = result
            self.metadata = result
            self.complete = result.complete
            return result
        finally:
            self._pending = None
            await self.aclose()


# === Public Functions === #


def execution_records(
    self: EvidenceScope,
    record_type: str,
    *,
    where: Where | None = None,
    between: Window | None = None,
    versions: Sequence[int] | None = None,
    at: At = None,
    page_rows: int | None = None,
    page_bytes: int | None = None,
) -> EvidenceReader[Record]:
    return records(
        self._transport,
        {"Execution": {"field_0": self.id}},
        record_type,
        where=where,
        between=between,
        versions=versions,
        at=at,
        page_rows=page_rows,
        page_bytes=page_bytes,
    )


def telemetry(
    self: EvidenceScope,
    stream: str,
    *,
    fields: Sequence[str] | None = None,
    where: Where | None = None,
    between: Window | None = None,
    versions: Sequence[int] | None = None,
    every: str | None = None,
    agg: Aggregate | None = None,
    at: At = None,
    page_rows: int | None = None,
    page_bytes: int | None = None,
) -> TelemetryReader:
    settings = self._transport.query_settings or QuerySettings()
    selected = selection(stream, versions, where, between, at)
    selected["stream"] = selected.pop("identity")
    selected["execution_id"] = self.id
    projection = tuple(fields) if fields is not None and not isinstance(fields, str) else fields
    if projection is not None and (
        isinstance(projection, str) or any(not isinstance(f, str) or not f for f in projection) or len(set(projection)) != len(projection)
    ):
        raise ValueError("fields requires unique nonempty literal field IDs")
    selected["fields"] = projection or ()
    selected["bucket"] = bucket_request(every, agg)
    selected["page"] = page_settings(settings, page_rows, page_bytes)

    def decode(row: Any) -> Sample | Bucket:
        member = _wire.variant(row)
        if member == "Raw" and every is None:
            return _wire.sample(row.Raw.field_0, 64)
        if member == "Bucket" and every is not None:
            value = row.Bucket.field_0
            return Bucket(
                _wire.timestamp(value, "start"),
                _wire.timestamp(value, "end"),
                value.execution_id,
                value.stream,
                value.source,
                tuple(value.versions),
                value.sample_count,
                tuple(
                    BucketValue(
                        f.field,
                        f.count,
                        statistic_value(f.value) if f.HasField("value") else None,
                        f.unit if f.HasField("unit") else None,
                        Aggregate.from_wire(f.reducer),
                    )
                    for f in value.fields
                ),
            )
        raise ProtocolError("Telemetry response differs from the requested raw/bucket mode")

    return TelemetryReader(self._transport, "ListSamples", selected, decode, settings)


def cycle_records(
    self: EvidenceScope,
    record_type: str,
    *,
    execution_id: str | None = None,
    where: Where | None = None,
    between: Window | None = None,
    versions: Sequence[int] | None = None,
    at: At = None,
    page_rows: int | None = None,
    page_bytes: int | None = None,
) -> EvidenceReader[Record]:
    if execution_id is not None and (not isinstance(execution_id, str) or not execution_id):
        raise ValueError("execution_id must be nonempty when supplied")
    scope = {"Cycle": {"field_0": {"cycle_id": self.id, "execution_id": execution_id}}}
    return records(
        self._transport,
        scope,
        record_type,
        where=where,
        between=between,
        versions=versions,
        at=at,
        page_rows=page_rows,
        page_bytes=page_bytes,
    )


def records(
    transport: Transport,
    scope: dict[str, Any],
    record_type: str,
    *,
    where: Where | None,
    between: Window | None,
    versions: Sequence[int] | None,
    at: At,
    page_rows: int | None,
    page_bytes: int | None,
) -> EvidenceReader[Record]:
    settings = transport.query_settings or QuerySettings()
    selected = selection(record_type, versions, where, between, at)
    selected["record_type"] = selected.pop("identity")
    selected.update(scope=scope, page=page_settings(settings, page_rows, page_bytes))
    return EvidenceReader(transport, "ListRecords", selected, lambda row: _wire.record(row, 64), settings)


def selection(identity: str, versions: Sequence[int] | None, where: Where | None, between: Window | None, at: At) -> dict[str, Any]:
    if not isinstance(identity, str) or not identity or any(c in identity for c in "*?"):
        raise ValueError("Select one concrete record type or stream ID without globs")
    selected_versions = tuple(versions) if versions is not None else ()
    for version in selected_versions:
        positive(version, "version", 2**32 - 1)
    if len(set(selected_versions)) != len(selected_versions):
        raise ValueError("versions must be unique")
    return {
        "identity": identity,
        "versions": selected_versions,
        "where": [p.to_wire() for p in predicates(where)],
        "between": window(between),
        "at": snapshot_ref(at),
    }


def bucket_request(every: str | None, aggregate: Aggregate | None) -> dict[str, Any] | None:
    if every is None and aggregate is None:
        return None
    if every is None or not isinstance(aggregate, Aggregate) or aggregate.member not in ("Count", "Sum", "Mean", "Min", "Max"):
        raise ValueError("every and agg are required together; use Aggregate.count/sum/mean/min/max for buckets")
    match = re.fullmatch(r"(\d+(?:\.\d+)?)(ns|us|ms|s|m|h|d)", every) if isinstance(every, str) else None
    if match is None:
        raise ValueError("every requires a positive fixed duration, for example '1s' or '250ms'")
    multipliers = {"ns": 1, "us": 1000, "ms": 10**6, "s": 10**9, "m": 60 * 10**9, "h": 3600 * 10**9, "d": 86400 * 10**9}
    whole, _, fraction = match[1].partition(".")
    fraction = fraction.rstrip("0")
    denominator = 10 ** len(fraction)
    fractional_nanos, remainder = divmod(int(fraction or "0") * multipliers[match[2]], denominator)
    nanos = int(whole) * multipliers[match[2]] + fractional_nanos
    if remainder or not 0 < nanos < 2**64:
        raise ValueError("every must fit positive uint64 nanoseconds without sub-nanosecond precision")
    return {"every_nanos": nanos, "aggregate": aggregate.to_wire()}


# === Private Functions === #

# === Entry Point === #
