# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Execution context typing surface for authored templates."""

from __future__ import annotations

# === Standard Library Imports === #

from typing import Literal, ClassVar, Self, TypeVar, overload
from collections.abc import AsyncIterator, Sequence
from contextlib import AbstractAsyncContextManager, AsyncExitStack

# === Local Imports === #

from dokime.core.authoring.artifacts import Artifacts
from dokime.core.authoring.prompts import Confirmation

# === Types === #

_T = TypeVar("_T")

# === Classes === #

class TestContext:
    parameters: dict[str, object]
    plugin: object
    resources: object
    reason: str | None

    def __init__(
        self,
        native: object,
        *,
        plugin: object = None,
        resources: object = None,
        reason: str | None = None,
        acquisitions: AsyncExitStack | None = None,
    ) -> None: ...
    async def acquire(self, resource: AbstractAsyncContextManager[_T]) -> _T: ...
    @property
    def artifacts(self) -> Artifacts: ...
    async def trace(self, message: str) -> None: ...
    async def debug(self, message: str) -> None: ...
    async def info(self, message: str) -> None: ...
    async def warning(self, message: str) -> None: ...
    async def error(self, message: str) -> None: ...
    @overload
    async def ask(self, prompt: Confirmation) -> bool: ...
    @overload
    async def ask(self, prompt: _T) -> _T: ...
    async def publish(self, value: object) -> None: ...
    def progress(self, phase: str, items: Sequence[_T]) -> AsyncIterator[_T]: ...
    async def lease(self, row_type: type[_T]) -> Lease[_T]: ...
    async def listen(self, item_type: type[_T]) -> Listening[_T]: ...
    async def samples(
        self,
        row_type: type[_T],
        *,
        origin: Literal["default", "initialization", "cleanup"] | None = None,
    ) -> list[_T]: ...
    async def records(
        self,
        record_type: type[_T],
        *,
        origin: Literal["default", "initialization", "cleanup"] | None = None,
    ) -> list[_T]: ...
    async def steps_reached(self) -> list[str]: ...

class Listening[T]:
    __type_params__: ClassVar[tuple[object, ...]]
    def __init__(self, native: object, identifier: int, item_type: type[T]) -> None: ...
    def __aiter__(self) -> Self: ...
    async def __anext__(self) -> T: ...
    async def close(self) -> None: ...

class Lease[T]:
    __type_params__: ClassVar[tuple[object, ...]]
    def __init__(self, native: object, identifier: int, row_type: type[T]) -> None: ...
    async def listen(self) -> Listening[T]: ...
    async def close(self) -> None: ...
