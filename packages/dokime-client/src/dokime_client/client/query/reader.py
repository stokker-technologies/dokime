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
import asyncio
import sys
from collections.abc import AsyncGenerator, AsyncIterator, Callable
from dataclasses import dataclass, fields, is_dataclass
from types import TracebackType
from typing import Any, Self

# Third Party Imports
from google.protobuf.message import Message

# Local Imports
from ..._wire.dokime.v1.QueryMessages.ListExecutions import messages_pb2 as contracts
from ...operation import Operation
from .. import _wire
from .._contracts import operation
from .._transport import Transport
from ..errors import ProtocolError, RpcError
from .errors import QueryLimitError, query_call
from .selection import positive
from .statistics import Statistics

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


@dataclass(frozen=True, slots=True)
class QuerySettings:
    """Finite Client Defaults; The Server Advertises Its Effective Page Limits."""

    page_rows: int = 1000
    page_bytes: int = 4 * 1024 * 1024
    max_rows: int = 100_000
    max_bytes: int = 64 * 1024 * 1024

    def __post_init__(self) -> None:
        for name in ("page_rows", "page_bytes", "max_rows", "max_bytes"):
            value = getattr(self, name)
            if value is None:
                raise ValueError(f"{name} requires a finite positive integer")
            positive(value, name, 2**32 - 1 if name == "page_rows" else 2**64 - 1)


@dataclass(frozen=True, slots=True)
class QueryPage[T]:
    rows: tuple[T, ...]
    schema: contracts.Row7 | None
    snapshot: contracts.Row8
    limits: contracts.Row9
    next: str | None
    complete: bool


class QueryReader[T]:
    """A Lazy, Single-Consumer Finite Query With No Prefetch."""

    def __init__(self, transport: Transport, method: str, fields: dict[str, Any], decode: Callable[[Any], T], settings: QuerySettings):
        self._transport = transport
        self._operation: Operation = operation("Query", method)
        self._request: Any = _wire.request("Query", method, **fields)
        self._decode = decode
        self._settings = settings
        self._mode: str | None = None
        self._closed = False
        self._pending: asyncio.Task | None = None
        self.snapshot: contracts.Row8 | None = None
        self.metadata: QueryPage[T] | Statistics | None = None
        self.complete = False

    def _claim(self, mode: str) -> None:
        if self._closed or self._mode is not None:
            raise RuntimeError("Query readers allow one item/page/collect/aggregate consumer; construct a fresh reader")
        self._mode = mode

    @property
    def closed(self) -> bool:
        return self._closed

    async def __aenter__(self) -> Self:
        if self._closed:
            raise RuntimeError("Query reader is closed; construct a fresh reader")
        return self

    async def __aexit__(self, exc_type: type[BaseException] | None, exc: BaseException | None, traceback: TracebackType | None) -> None:
        await self.aclose()

    def __aiter__(self) -> AsyncIterator[T]:
        self._claim("items")
        return self._items()

    async def _items(self) -> AsyncIterator[T]:
        try:
            async for page in self._pages():
                for row in page.rows:
                    if self._closed:
                        return
                    yield row
        finally:
            await self.aclose()

    def pages(self) -> AsyncGenerator[QueryPage[T]]:
        self._claim("pages")
        return self._pages()

    async def _pages(self) -> AsyncGenerator[QueryPage[T]]:
        try:
            while not self._closed:
                self._pending = asyncio.create_task(query_call(self._transport, self._operation, self._request))
                try:
                    response: Any = await self._pending
                finally:
                    self._pending = None
                if self._closed:
                    return
                page = self._page(response)
                self.snapshot = page.snapshot
                self.metadata = page
                self.complete = page.complete and self._mode == "pages"
                yield page
                if self._closed:
                    return
                if page.complete:
                    self.complete = True
                    return
                self._request.page.after = page.next
                _wire.copy(self._request.at, {"Token": {"field_0": page.snapshot.token}})
        finally:
            await self.aclose()

    def _page(self, response: Any) -> QueryPage[T]:
        if any(not response.HasField(name) for name in ("snapshot", "limits")):
            raise ProtocolError("Query page is missing snapshot/effective limits")
        if not response.HasField("schema") and self._operation.id not in ("dokime.query.list_executions", "dokime.query.list_cycles"):
            raise ProtocolError("Evidence query page is missing its captured schema")
        snapshot = contracts.Row8()
        schema = contracts.Row7() if response.HasField("schema") else None
        limits = contracts.Row9()
        _wire.copy(snapshot, response.snapshot)
        if schema is not None:
            _wire.copy(schema, response.schema)
        _wire.copy(limits, response.limits)
        after = response.next if response.HasField("next") else None
        if not snapshot.token or not snapshot.identity or (self.snapshot is not None and snapshot != self.snapshot):
            raise ProtocolError("Query page changed or omitted its snapshot")
        if self._request.HasField("at") and self._request.at.HasField("Token") and snapshot.token != self._request.at.Token.field_0:
            raise ProtocolError("Query page differs from the supplied snapshot")
        if self.metadata is not None and schema != self.metadata.schema:
            raise ProtocolError("Query page changed its captured schema")
        if (
            not limits.rows
            or not limits.bytes
            or len(response.rows) > limits.rows
            or response.ByteSize() > limits.bytes
            or limits.rows > self._request.page.rows
            or limits.bytes > self._request.page.bytes
        ):
            raise ProtocolError("Query page violates its effective limits")
        if response.complete == (after is not None) or (after is not None and (not after or after == self._request.page.after)):
            raise ProtocolError("Query continuation/completeness is invalid or does not advance")
        return QueryPage(tuple(self._decode(row) for row in response.rows), schema, snapshot, limits, after, response.complete)

    async def collect(self, *, max_rows: int | None = None, max_bytes: int | None = None) -> list[T]:
        """Materialize Atomically Within Finite Row And Decoded Python Byte Budgets."""
        row_limit = self._settings.max_rows if max_rows is None else positive(max_rows, "max_rows")
        byte_limit = self._settings.max_bytes if max_bytes is None else positive(max_bytes, "max_bytes")
        self._claim("collect")
        result: list[T] = []
        used = sys.getsizeof(result)
        try:
            async for page in self._pages():
                if len(result) + len(page.rows) > row_limit:
                    raise self._limit("max_rows", row_limit)
                # Charge Each Page's Decoded Rows And Metadata Before Publishing Any Collection.
                used += _decoded_size(page, set())
                if used > byte_limit:
                    raise self._limit("max_bytes", byte_limit)
                result.extend(page.rows)
            if not self.complete:
                raise ProtocolError("Query collection closed before selection exhaustion")
            return result
        except BaseException:
            self.complete = False
            raise
        finally:
            await self.aclose()

    def _limit(self, name: str, value: int) -> QueryLimitError:
        return QueryLimitError(
            RpcError("RESOURCE_EXHAUSTED", f"Query collection exceeds {name}={value}"),
            {
                "reason": "ResourceLimit",
                "operation": self._operation.id,
                "snapshot": self.snapshot.identity if self.snapshot is not None else None,
                "limit": {"name": name, "value": value},
            },
        )

    async def aclose(self) -> None:
        self._closed = True
        pending = self._pending
        if pending is not None and not pending.done():
            pending.cancel()
            try:
                await pending
            except asyncio.CancelledError:
                pass


# === Public Functions === #


def page_settings(settings: QuerySettings, page_rows: int | None, page_bytes: int | None) -> dict[str, int]:
    return {
        "rows": settings.page_rows if page_rows is None else positive(page_rows, "page_rows", 2**32 - 1),
        "bytes": settings.page_bytes if page_bytes is None else positive(page_bytes, "page_bytes"),
    }


# === Private Functions === #


def _decoded_size(value: Any, seen: set[int]) -> int:
    identity = id(value)
    if identity in seen:
        return 0
    seen.add(identity)
    size = sys.getsizeof(value)
    if isinstance(value, Message):
        for descriptor, child in value.ListFields():
            size += sys.getsizeof(descriptor.name)
            if descriptor.is_repeated:
                size += sys.getsizeof(child)
                size += sum(_decoded_size(item, seen) for item in child)
            else:
                size += _decoded_size(child, seen)
    elif is_dataclass(value) and not isinstance(value, type):
        size += sys.getsizeof(getattr(value, "__dict__", {}))
        size += sum(_decoded_size(getattr(value, field.name), seen) for field in fields(value) if not field.name.startswith("_"))
    elif isinstance(value, dict):
        size += sum(_decoded_size(key, seen) + _decoded_size(child, seen) for key, child in value.items())
    elif isinstance(value, (tuple, list)):
        size += sum(_decoded_size(child, seen) for child in value)
    return size


# === Entry Point === #
