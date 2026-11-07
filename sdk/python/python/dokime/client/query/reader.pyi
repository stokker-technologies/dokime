# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

# === Imports === #

# Standard Library Imports
from collections.abc import AsyncGenerator, AsyncIterator, Callable
from dataclasses import dataclass
from types import TracebackType
from typing import Any, ClassVar, Self, TypeVar

# Third Party Imports
# Local Imports
from .._transport import Transport
from .errors import QueryLimitError
from .models import PageLimits, Schema, Snapshot
from .statistics import Statistics

# Type Checking Imports

# === Public Exports === #

# === Constants === #

contracts: Any

# === Types === #

# === Classes === #

@dataclass(frozen=True, slots=True)
class QuerySettings:
    page_rows: int = 1000
    page_bytes: int = ...
    max_rows: int = 100_000
    max_bytes: int = ...

    def __post_init__(self) -> None: ...

@dataclass(frozen=True, slots=True)
class QueryPage[T]:
    __type_params__: ClassVar[tuple[TypeVar, ...]]
    rows: tuple[T, ...]
    schema: Schema | None
    snapshot: Snapshot
    limits: PageLimits
    next: str | None
    complete: bool

class QueryReader[T]:
    __type_params__: ClassVar[tuple[TypeVar, ...]]
    snapshot: Snapshot | None
    metadata: QueryPage[T] | Statistics | None
    complete: bool
    def __init__(self, transport: Transport, method: str, fields: dict[str, Any], decode: Callable[[Any], T], settings: QuerySettings): ...
    def _claim(self, mode: str) -> None: ...
    @property
    def closed(self) -> bool: ...
    async def __aenter__(self) -> Self: ...
    async def __aexit__(self, exc_type: type[BaseException] | None, exc: BaseException | None, traceback: TracebackType | None) -> None: ...
    def __aiter__(self) -> AsyncIterator[T]: ...
    def _items(self) -> AsyncIterator[T]: ...
    def pages(self) -> AsyncGenerator[QueryPage[T]]: ...
    def _pages(self) -> AsyncGenerator[QueryPage[T]]: ...
    def _page(self, response: Any) -> QueryPage[T]: ...
    async def collect(self, *, max_rows: int | None = None, max_bytes: int | None = None) -> list[T]: ...
    def _limit(self, name: str, value: int) -> QueryLimitError: ...
    async def aclose(self) -> None: ...

def page_settings(settings: QuerySettings, page_rows: int | None, page_bytes: int | None) -> dict[str, int]: ...
def _decoded_size(value: Any, seen: set[int]) -> int: ...
