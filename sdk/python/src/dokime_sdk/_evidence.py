# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Evidence Queries Read at a Frozen Cut or Followed Live"""

# === Imports === #

# Standard Library Imports
from collections.abc import AsyncGenerator, AsyncIterator, Awaitable, Callable, Sequence
from datetime import datetime
from types import MappingProxyType
from typing import TYPE_CHECKING, ClassVar, Protocol, TypeVar, cast

# Third Party Imports
from google.protobuf.descriptor_pb2 import DescriptorProto
from google.protobuf.message import DecodeError, Message
from google.protobuf.timestamp_pb2 import Timestamp
from grpc import StatusCode
from grpc.aio import AioRpcError
from harmos import PageRequest, ReadCut

# Local Imports
from ._authoring import Record, Stream
from ._generated.proto.host.v1.evidence.records_pb2 import (
    CapturedRecord,
    CapturedRecordSelection,
    RecordScope,
    RecordsListResponse,
    RecordsWatchResponse,
)
from ._generated.proto.host.v1.evidence.samples_pb2 import BucketRequest, SamplesListResponse, SamplesWatchResponse
from ._generated.proto.host.v1.evidence.samples_pb2 import SampleRow as SampleRowMessage
from ._generated.proto.host.v1.executions_pb2 import EXECUTION_STATE_OPEN
from ._generated.proto.model.v1.evidence_pb2 import EvidenceScope, Window
from ._payloads import Payload, message_type, unpack
from ._transport import call, invalid, transport_error
from ._values import RecordRow, SampleRow, Scalar, decode

# Type Checking Imports
if TYPE_CHECKING:
    import pandas
    import polars

    from ._client import Execution
    from .templates._context import Context

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class RecordsReader(Protocol):
    async def list(
        self,
        record_type: str,
        versions: list[int],
        between: Window,
        step: str,
        selection: CapturedRecordSelection,
        *,
        scope: RecordScope | None = None,
        at: ReadCut | None = None,
        page: PageRequest | None = None,
    ) -> RecordsListResponse: ...


class RecordsQuery[T: Payload]:
    __type_params__: ClassVar[tuple[TypeVar, ...]]

    def __init__(
        self,
        name: str,
        execution: str,
        reader: RecordsReader,
        decoder: Callable[[bytes], Awaitable[T]],
        *,
        step: str = "",
        live: "Execution | None" = None,
    ) -> None:
        self._name = name
        self._execution = execution
        self._reader = reader
        self._decoder = decoder
        self._step = step
        # A Client Execution Binds Frozen Schemas and Follows Live Rows; a Plugin's Context Query Reads Frozen Cuts Only.
        self._live = live
        self._at: ReadCut | None = None

    @classmethod
    def _for_execution(
        cls,
        execution: "Execution",
        declaration: Record[T] | str,
        *,
        step: str = "",
        row_type: type[T] | None = None,
    ) -> "RecordsQuery[T]":
        name, row_type = _selector(execution, declaration, row_type)

        async def decode(payload: bytes) -> T:
            return await _decode(execution, name, payload, row_type)

        return cls(name, execution.identifier, execution._client._transport.evidence.records, decode, step=step, live=execution)

    @classmethod
    def _for_context(cls, context: "Context", declaration: Record[T], *, step: str = "") -> "RecordsQuery[T]":
        name = context._resolve(declaration)

        async def decode(payload: bytes) -> T:
            return await _decode(None, name, payload, declaration.row)

        return cls(name, context.execution, context._host.evidence.records, decode, step=step)

    async def _page(self, cursor: PageRequest) -> RecordsListResponse:
        if self._live is not None:
            await _prepare(self._live, self._name)
        reply = await call(
            "evidence.records.list",
            self._reader.list(
                self._name,
                [],
                Window(),
                self._step,
                CapturedRecordSelection(),
                scope=RecordScope(execution=self._execution),
                at=self._at,
                page=cursor,
            ),
        )
        if self._at is None:
            self._at = reply.resolved_at
        elif self._at != reply.resolved_at:
            raise invalid("evidence.records.list", "The Evidence Read Cut Changed")
        return reply

    async def _decode(self, row: CapturedRecord, operation: str = "evidence.records.list") -> RecordRow[T]:
        if row.record_type != self._name or row.scope.execution_id != self._execution:
            raise invalid(operation, "The Host Returned a Record Outside the Selected Execution or Type")
        value = await self._decoder(row.payload)
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

    async def follow(self) -> AsyncGenerator[RecordRow[T]]:
        """Yield the Captured Rows, Then Each New Row as It Is Captured, Until the Execution Closes."""
        execution = self._live
        if execution is None:
            raise ValueError("Follow Records: Select Them Through a Client Execution")
        await _prepare(execution, self._name)
        records = execution._client._transport.evidence.records
        frames = _frames(
            execution,
            "evidence.records.watch",
            lambda: records.watch(RecordScope(execution=self._execution), self._name, [], Window(), self._step, CapturedRecordSelection()),
        )
        after = -1
        try:
            async for frame in frames:
                for row in frame.records:
                    if row.sequence > after:
                        after = row.sequence
                        yield await self._decode(row, "evidence.records.watch")
        finally:
            await frames.aclose()


class SamplesQuery[T: Payload]:
    __type_params__: ClassVar[tuple[TypeVar, ...]]

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

    async def _page(self, cursor: PageRequest, *, latest: bool = False) -> SamplesListResponse:
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
                BucketRequest(),
                latest,
                at=self._at,
                page=cursor,
            ),
        )
        if self._at is None:
            self._at = reply.resolved_at
        elif self._at != reply.resolved_at:
            raise invalid("evidence.samples.list", "The Evidence Read Cut Changed")
        return reply

    async def _decode(self, sample: SampleRowMessage, operation: str = "evidence.samples.list") -> SampleRow[T]:
        raw = sample.raw
        if sample.WhichOneof("kind") != "raw" or raw.stream != self._name or raw.scope.execution_id != self._execution.identifier:
            raise invalid(operation, "The Host Returned a Sample Outside the Selected Execution or Stream")
        values: dict[str, Scalar] = {}
        for name, value in raw.values.values.items():
            self._kinds.setdefault(name, set()).add(value.WhichOneof("kind") or "null")
            values[name] = decode(value)
        return SampleRow(
            raw.sample_id,
            raw.sequence,
            _nanoseconds(raw.timestamp),
            _nanoseconds(raw.received_at),
            raw.stream,
            raw.schema_version,
            raw.source,
            await _decode(self._execution, self._name, raw.payload, self._row_type),
            MappingProxyType(values),
        )

    async def pages(self, *, page_size: int = 0) -> AsyncIterator[list[SampleRow[T]]]:
        _page_size(page_size)
        await _prepare(self._execution, self._name)
        cursor = PageRequest(page_size=page_size)
        tokens: set[str] = set()
        while True:
            reply = await self._page(cursor)
            yield [await self._decode(sample) for sample in reply.samples]
            if not reply.next_page_token:
                return
            _advance(cursor, reply.next_page_token, tokens)

    async def last(self) -> T:
        """The Newest Selected Sample's Row at the Query's Read Cut; a New Query Reads the Newest Durable Sample."""
        await _prepare(self._execution, self._name)
        reply = await self._page(PageRequest(page_size=1), latest=True)
        if not reply.samples:
            raise invalid("evidence.samples.list", "The Selected Samples Are Empty")
        return (await self._decode(reply.samples[-1])).value

    async def follow(self) -> AsyncGenerator[SampleRow[T]]:
        """Yield the Captured Samples, Then Each New Sample as It Is Captured, Until the Execution Closes."""
        await _prepare(self._execution, self._name)
        samples = self._execution._client._transport.evidence.samples
        frames = _frames(
            self._execution,
            "evidence.samples.watch",
            lambda: samples.watch(
                EvidenceScope(execution_id=self._execution.identifier),
                [self._name],
                [],
                self._fields,
                [],
                [],
                self._between,
                BucketRequest(),
                False,
            ),
        )
        after = -1
        try:
            async for frame in frames:
                for sample in frame.samples:
                    if sample.raw.sequence > after:
                        after = sample.raw.sequence
                        yield await self._decode(sample, "evidence.samples.watch")
        finally:
            await frames.aclose()

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
    if row_type is not None:
        message_type(row_type)
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


async def _frames[F: (RecordsWatchResponse, SamplesWatchResponse)](
    execution: "Execution", operation: str, watch: Callable[[], AsyncGenerator[F]]
) -> AsyncGenerator[F]:
    # Every Pass Replays Its Selection; a Clean End Is Final Only for a Pass Opened After the Execution Closed.
    while True:
        await execution.refresh()
        closed = execution._detail.summary.state != EXECUTION_STATE_OPEN
        frames = watch()
        try:
            async for frame in frames:
                if frame.HasField("resync_required"):
                    break
                yield frame
            else:
                if closed:
                    return
        except AioRpcError as error:
            # The Host Bounds Each Watch; Reopen After Its Deadline.
            if error.code() != StatusCode.DEADLINE_EXCEEDED:
                raise transport_error(operation, error) from error
        finally:
            await frames.aclose()


async def _prepare(execution: "Execution", name: str) -> None:
    if not execution._detail.HasField("schemas"):
        await execution.refresh()
    await execution._client._schemas.get(execution._client._transport, execution._detail)
    bindings = {row.record_type for row in execution._detail.schemas.records}
    bindings.update(row.stream for row in execution._detail.schemas.streams)
    if name not in bindings:
        raise invalid("evidence.decode", "The Selector Has No Frozen Message Binding")


async def _decode[T: Payload](execution: "Execution | None", name: str, payload: bytes, row_type: type[T] | None) -> T:
    try:
        if execution is None:
            if row_type is None:
                raise invalid("evidence.decode", "Bind a Declared Row Type Before Reading Scoped Records")
            value = message_type(row_type).FromString(payload)
        else:
            if not execution._detail.HasField("schemas"):
                await execution.refresh()
            value = await execution._client._schemas.decode(execution._client._transport, execution._detail, name, payload)
    except (DecodeError, ValueError) as error:
        raise invalid("evidence.decode", str(error)) from error
    if row_type is None:
        return cast(T, value)
    expected = message_type(row_type)
    frozen = _message_definition(type(value))
    declared = _message_definition(expected)
    if value.DESCRIPTOR.full_name != expected.DESCRIPTOR.full_name or frozen != declared:
        raise invalid("evidence.decode", "The Frozen Message Binding Differs from the Declared Row Type")
    return unpack(row_type, expected.FromString(payload))


def _message_definition(message_type: type[Message]) -> DescriptorProto:
    descriptor = message_type.DESCRIPTOR
    definition = DescriptorProto()
    descriptor.CopyToProto(definition)
    pending = [(definition, descriptor)]
    while pending:
        message, source = pending.pop()
        for field in message.field:
            field.json_name = source.fields_by_name[field.name].json_name
        pending.extend((nested, source.nested_types_by_name[nested.name]) for nested in message.nested_type)
    return definition


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
