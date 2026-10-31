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
from collections.abc import AsyncGenerator, AsyncIterator, Callable
from dataclasses import dataclass
from typing import Any

# Third Party Imports
# Local Imports
from ..._wire.dokime.v1.QueryMessages.ListExecutions import messages_pb2 as contracts
from ...operation import Operation
from .. import _wire
from .._contracts import operation
from .._transport import Transport
from ..errors import ProtocolError
from .errors import query_call
from .selection import positive

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
        positive(self.page_rows, "page_rows", 2**32 - 1)
        for name in ("page_bytes", "max_rows", "max_bytes"):
            positive(getattr(self, name), name)


@dataclass(frozen=True, slots=True)
class QueryPage[T]:
    rows: tuple[T, ...]
    schema: contracts.Row7
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
        self.metadata: QueryPage[T] | None = None
        self.complete = False

    def _claim(self, mode: str) -> None:
        if self._closed or self._mode is not None:
            raise RuntimeError("Query readers allow one item/page/collect/aggregate consumer; construct a fresh reader")
        self._mode = mode

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
                self.complete = page.complete
                yield page
                if page.complete or self._closed:
                    return
                self._request.page.after = page.next
                _wire.copy(self._request.at, {"Token": {"field_0": page.snapshot.token}})
        finally:
            await self.aclose()

    def _page(self, response: Any) -> QueryPage[T]:
        if any(not response.HasField(name) for name in ("snapshot", "schema", "limits")):
            raise ProtocolError("Query page is missing snapshot/schema/effective limits")
        snapshot = contracts.Row8()
        schema = contracts.Row7()
        limits = contracts.Row9()
        _wire.copy(snapshot, response.snapshot)
        _wire.copy(schema, response.schema)
        _wire.copy(limits, response.limits)
        after = response.next if response.HasField("next") else None
        if not snapshot.token or not snapshot.identity or (self.snapshot is not None and snapshot != self.snapshot):
            raise ProtocolError("Query page changed or omitted its snapshot")
        if not limits.rows or not limits.bytes or len(response.rows) > limits.rows:
            raise ProtocolError("Query page violates its effective limits")
        if response.complete == (after is not None) or (after is not None and (not after or after == self._request.page.after)):
            raise ProtocolError("Query continuation/completeness is invalid or does not advance")
        return QueryPage(tuple(self._decode(row) for row in response.rows), schema, snapshot, limits, after, response.complete)

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

# === Entry Point === #
