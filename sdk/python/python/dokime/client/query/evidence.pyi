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
from typing import Any, ClassVar, Protocol, TypeVar

# Third Party Imports
import pandas as pd
import polars as pl

# Local Imports
from .._transport import Transport
from ..aggregate import Aggregate
from ..models import Record, Sample
from .reader import QueryReader
from .selection import At, Where, Window
from .statistics import Statistics

# Type Checking Imports

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
    __type_params__: ClassVar[tuple[TypeVar, ...]]
    async def to_polars(self, *, max_rows: int | None = None, max_bytes: int | None = None) -> pl.DataFrame: ...
    async def to_pandas(self, *, max_rows: int | None = None, max_bytes: int | None = None) -> pd.DataFrame: ...

class TelemetryReader(EvidenceReader[Sample | Bucket]):
    statistics: Statistics | None = None

    async def aggregate(self, field: str, aggregates: Sequence[Aggregate]) -> Statistics: ...

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
) -> EvidenceReader[Record]: ...
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
) -> TelemetryReader: ...
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
) -> EvidenceReader[Record]: ...
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
) -> EvidenceReader[Record]: ...
def selection(identity: str, versions: Sequence[int] | None, where: Where | None, between: Window | None, at: At) -> dict[str, Any]: ...
def bucket_request(every: str | None, aggregate: Aggregate | None) -> dict[str, Any] | None: ...
