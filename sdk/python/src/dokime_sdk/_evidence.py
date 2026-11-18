# ***
#
# dokime-rust-workspace
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Finite Frozen Evidence Queries"""

# === Imports === #

# Standard Library Imports
from collections.abc import AsyncIterator, Sequence
from datetime import datetime
from types import MappingProxyType
from typing import TYPE_CHECKING, cast

# Third Party Imports
from google.protobuf.timestamp_pb2 import Timestamp
from harmos import PageRequest, ReadCut

# Local Imports
from ._authoring import Record, Stream
from ._errors import call, invalid
from ._generated import client_types
from ._payloads import Payload, message_type, unpack
from ._values import RecordRow, SampleRow, Scalar, decode
from ._wire.host.v1.evidence.records_pb2 import CapturedRecord, CapturedRecordSelection, RecordScope, RecordsListResponse
from ._wire.model.v1.evidence_pb2 import EvidenceScope, Window

# Type Checking Imports
if TYPE_CHECKING:
    import pandas
    import polars

    from ._client import Execution

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class RecordsQuery[T: Payload]:
    def __init__(self, execution: "Execution", declaration: Record[T] | str, *, step: str = "", row_type: type[T] | None = None) -> None:
        self._execution = execution
        self._name, self._row_type = _selector(execution, declaration, row_type)
        self._step = step
        self._at: ReadCut | None = None

    async def _page(self, cursor: PageRequest) -> RecordsListResponse:
        await _prepare(self._execution, self._name)
        reply = await call(
            "evidence.records.list",
            self._execution._client._transport.evidence.records.list(
                self._name,
                [],
                Window(),
                self._step,
                CapturedRecordSelection(),
                scope=RecordScope(execution=self._execution.identifier),
                at=self._at,
                page=cursor,
            ),
        )
        if self._at is None:
            self._at = reply.resolved_at
        elif self._at != reply.resolved_at:
            raise invalid("evidence.records.list", "The Evidence Read Cut Changed")
        return reply

    async def _decode(self, row: CapturedRecord) -> RecordRow[T]:
        if row.record_type != self._name or row.scope.execution_id != self._execution.identifier:
            raise invalid("evidence.records.list", "The Host Returned a Record Outside the Selected Execution or Type")
        value = await _decode(self._execution, self._name, row.payload, self._row_type)
        return RecordRow(
            row.record_id,
            row.sequence,
            _nanoseconds(row.timestamp),
            _nanoseconds(row.received_at),
            row.record_type,
            row.schema_version,
            row.source,
            value,
        )

    async def pages(self, *, page_size: int = 0) -> AsyncIterator[list[RecordRow[T]]]:
        _page_size(page_size)
        cursor = PageRequest(page_size=page_size)
        tokens: set[str] = set()
        while True:
            reply = await self._page(cursor)
            yield [await self._decode(row) for row in reply.records]
            if not reply.next_page_token:
                return
            _advance(cursor, reply.next_page_token, tokens)

    async def read(self, *, limit: int = 0) -> list[RecordRow[T]]:
        _page_size(limit)
        if limit == 0:
            reply = await self._page(PageRequest())
            return [await self._decode(row) for row in reply.records]
        rows: list[RecordRow[T]] = []
        async for page in self.pages(page_size=limit):
            rows.extend(page[: limit - len(rows)])
            if len(rows) == limit:
                break
        return rows

    async def count(self) -> int:
        return (await self._page(PageRequest(page_size=1))).count

    async def last(self) -> T:
        reply = await self._page(PageRequest(page_size=1))
        if not reply.HasField("last"):
            raise invalid("evidence.records.list", "The Selected Records Are Empty")
        return (await self._decode(reply.last)).value


class SamplesQuery[T: Payload]:
    def __init__(
        self,
        execution: "Execution",
        stream: Stream | str,
        *,
        fields: Sequence[str] = (),
        between: tuple[datetime | None, datetime | None] | None = None,
        row_type: type[T] | None = None,
    ) -> None:
        self._execution = execution
        self._name, self._row_type = _selector(execution, stream, row_type)
        self._fields = tuple(fields)
        self._between = Window()
        for field, value in zip(("start", "end"), between or (None, None), strict=True):
            if value is None:
                continue
            if value.tzinfo is None:
                raise ValueError("Read Samples: Supply Timezone-Aware Bounds")
            getattr(self._between, field).FromDatetime(value)
        self._at: ReadCut | None = None
        self._kinds: dict[str, set[str]] = {}

    async def pages(self, *, page_size: int = 0) -> AsyncIterator[list[SampleRow[T]]]:
        _page_size(page_size)
        await _prepare(self._execution, self._name)
        cursor = PageRequest(page_size=page_size)
        tokens: set[str] = set()
        while True:
            reply = await call(
                "evidence.samples.list",
                self._execution._client._transport.evidence.samples.list(
                    EvidenceScope(execution_id=self._execution.identifier),
                    [self._name],
                    [],
                    self._fields,
                    [],
                    [],
                    self._between,
                    client_types.BucketRequest(),
                    False,
                    at=self._at,
                    page=cursor,
                ),
            )
            if self._at is None:
                self._at = reply.resolved_at
            elif self._at != reply.resolved_at:
                raise invalid("evidence.samples.list", "The Evidence Read Cut Changed")
            rows: list[SampleRow[T]] = []
            for sample in reply.samples:
                if (
                    sample.WhichOneof("kind") != "raw"
                    or sample.raw.stream != self._name
                    or sample.raw.scope.execution_id != self._execution.identifier
                ):
                    raise invalid("evidence.samples.list", "The Host Returned a Sample Outside the Selected Execution or Stream")
                raw = sample.raw
                values: dict[str, Scalar] = {}
                for name, value in raw.values.values.items():
                    self._kinds.setdefault(name, set()).add(value.WhichOneof("kind") or "null")
                    values[name] = decode(value)
                decoded = await _decode(self._execution, self._name, raw.payload, self._row_type)
                rows.append(
                    SampleRow(
                        raw.sample_id,
                        raw.sequence,
                        _nanoseconds(raw.timestamp),
                        _nanoseconds(raw.received_at),
                        raw.stream,
                        raw.schema_version,
                        raw.source,
                        decoded,
                        MappingProxyType(values),
                    )
                )
            yield rows
            if not reply.next_page_token:
                return
            _advance(cursor, reply.next_page_token, tokens)

    async def collect(self, *, max_rows: int = 100_000) -> list[SampleRow[T]]:
        if max_rows <= 0:
            raise ValueError("Read Samples: Supply a Positive Row Bound")
        rows: list[SampleRow[T]] = []
        async for page in self.pages():
            rows.extend(page)
            if len(rows) > max_rows:
                raise ValueError("Read Samples: Narrow the Selection or Increase max_rows")
        return rows

    async def to_polars(self, *, max_rows: int = 100_000) -> "polars.DataFrame":
        import polars

        rows = await self.collect(max_rows=max_rows)
        columns = {
            "sequence": polars.Series("sequence", [row.sequence for row in rows], dtype=polars.UInt64),
            "timestamp": polars.Series("timestamp", [row.timestamp_ns for row in rows], dtype=polars.Int64).cast(
                polars.Datetime("ns", "UTC")
            ),
        }
        for name in sorted(self._kinds):
            kind = self._kinds[name] - {"null"}
            dtype = (
                {
                    "unsigned_integer": polars.UInt64,
                    "integer": polars.Int64,
                    "number": polars.Float64,
                    "boolean": polars.Boolean,
                    "text": polars.String,
                    "data": polars.Binary,
                }.get(next(iter(kind)), polars.Object)
                if len(kind) == 1
                else polars.Object
            )
            columns[name] = polars.Series(name, [row.values.get(name) for row in rows], dtype=dtype)
        return polars.DataFrame(list(columns.values()))

    async def to_pandas(self, *, max_rows: int = 100_000) -> "pandas.DataFrame":
        import pandas

        rows = await self.collect(max_rows=max_rows)
        columns = {
            "sequence": pandas.Series([row.sequence for row in rows], dtype="UInt64"),
            "timestamp": pandas.Series(pandas.to_datetime([row.timestamp_ns for row in rows], unit="ns", utc=True)),
        }
        for name in sorted(self._kinds):
            kind = self._kinds[name] - {"null"}
            dtype = (
                {"unsigned_integer": "UInt64", "integer": "Int64", "number": "Float64", "boolean": "boolean", "text": "string"}.get(
                    next(iter(kind)), "object"
                )
                if len(kind) == 1
                else "object"
            )
            columns[name] = pandas.Series([row.values.get(name) for row in rows], dtype=dtype)
        return pandas.DataFrame(columns)


# === Public Functions === #

# === Private Functions === #


def _selector[T: Payload](
    execution: "Execution", declaration: Record[T] | Stream | str, row_type: type[T] | None
) -> tuple[str, type[T] | None]:
    if isinstance(declaration, str):
        if not declaration:
            raise ValueError("Read Evidence: Supply a Nonempty Selector")
        return declaration, row_type
    template = execution._client.plugins._templates.get(execution.case.template)
    if template is not None:
        name = template.resolve(declaration)
        declared = cast(type[T], declaration.row)
        if row_type is not None and row_type is not declared:
            raise TypeError("Read Evidence: Use the Declaration's Row Type")
        return name, declared
    raise ValueError("Read Evidence: Select a Declaration Bound to This Execution's Registered Template")


async def _prepare(execution: "Execution", name: str) -> None:
    if not execution._detail.HasField("schemas"):
        await execution.refresh()
    await execution._client._schemas.get(execution._client._transport, execution._detail)
    bindings = {row.record_type for row in execution._detail.schemas.records}
    bindings.update(row.stream for row in execution._detail.schemas.streams)
    if name not in bindings:
        raise invalid("evidence.decode", "The Selector Has No Frozen Message Binding")


async def _decode[T: Payload](execution: "Execution", name: str, payload: bytes, row_type: type[T] | None) -> T:
    if not execution._detail.HasField("schemas"):
        await execution.refresh()
    try:
        value = await execution._client._schemas.decode(execution._client._transport, execution._detail, name, payload)
    except ValueError as error:
        raise invalid("evidence.decode", str(error)) from error
    if row_type is None:
        return cast(T, value)
    expected = message_type(row_type)
    if value.DESCRIPTOR.full_name != expected.DESCRIPTOR.full_name:
        raise invalid("evidence.decode", "The Frozen Message Binding Differs from the Declared Row Type")
    return unpack(row_type, expected.FromString(payload))


def _nanoseconds(timestamp: Timestamp) -> int:
    return timestamp.seconds * 1_000_000_000 + timestamp.nanos


def _page_size(value: int) -> None:
    if value < 0:
        raise ValueError("Read Evidence: Supply a Nonnegative Page Size or Limit")


def _advance(cursor: PageRequest, token: str, tokens: set[str]) -> None:
    if token in tokens:
        raise invalid("evidence.list", "The Host Repeated a Page Token")
    tokens.add(token)
    cursor.page_token = token


# === Entry Point === #
