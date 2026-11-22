# SPDX-License-Identifier: Apache-2.0

"""Standard Logging Buffered in the Current Invocation's Host Call Order."""

# === Imports === #

# Standard Library Imports
import asyncio
import inspect
import logging
from collections.abc import AsyncGenerator, Awaitable, Callable
from contextvars import ContextVar
from typing import TYPE_CHECKING, cast, overload

# Third Party Imports
from google.protobuf.message import Message
from google.protobuf.timestamp_pb2 import Timestamp
from harmos import OperationError, OperationPolicy, plugin

# Local Imports
from ._generated import shared
from ._generated.host import HostRoot

# Type Checking Imports
if TYPE_CHECKING:
    from ._context import Context

# === Public Exports === #

# === Constants === #

_CURRENT: ContextVar["_Buffer | None"] = ContextVar("dokime_execution_logs", default=None)

# === Types === #

# === Classes === #


class _Handler(logging.StreamHandler):
    def emit(self, record: logging.LogRecord) -> None:
        buffer = _CURRENT.get()
        if buffer is None or not buffer.execution:
            super().emit(record)
            return
        buffer.records.append(record)


class _Buffer:
    def __init__(self, transport: plugin.Context, execution: str) -> None:
        self.host = HostRoot(transport)
        self.execution = execution
        self.records: list[logging.LogRecord] = []
        self.lock = asyncio.Lock()

    async def flush(self) -> None:
        records, self.records = self.records, []
        if not records:
            return
        token = _CURRENT.set(None)
        try:
            for record in records:
                severity = shared.SEVERITY_DEBUG
                if record.levelno >= logging.ERROR:
                    severity = shared.SEVERITY_FAILURE
                elif record.levelno >= logging.WARNING:
                    severity = shared.SEVERITY_WARNING
                elif record.levelno >= logging.INFO:
                    severity = shared.SEVERITY_INFO
                timestamp = await self.host.time.get()
                source_at = Timestamp()
                source_at.CopyFrom(timestamp)
                text = record.getMessage()
                if record.exc_info:
                    text += "\n" + logging.Formatter().formatException(record.exc_info)
                await self.host.evidence.logs.append(self.execution, severity, text, source_at)
        finally:
            _CURRENT.reset(token)


class HostCalls:
    """The Single Buffer Boundary for Generated Host Calls and Peer Invocations."""

    def __init__(self, transport: plugin.Context) -> None:
        self.transport = transport

    async def _call[Q: Message, R: Message](
        self, operation: plugin.Operation[Q, R], request: Q, *, policy: OperationPolicy | None = None
    ) -> R:
        buffer = _CURRENT.get()
        if buffer is None:
            return await self.transport._call(operation, request, policy=policy)
        async with buffer.lock:
            await buffer.flush()
            return await self.transport._call(operation, request, policy=policy)

    async def watch[Q: Message, R: Message](
        self, operation: plugin.Operation[Q, R], request: Q, *, max_item_bytes: int
    ) -> AsyncGenerator[R]:
        call = self.transport.watch(operation, request, max_item_bytes=max_item_bytes)
        try:
            while True:
                await flush()
                try:
                    item = await anext(call)
                except StopAsyncIteration:
                    return
                yield item
        finally:
            await call.aclose()


# === Public Functions === #


def install() -> None:
    root = logging.getLogger()
    if any(isinstance(handler, _Handler) for handler in root.handlers):
        return
    root.addHandler(_Handler())
    root.setLevel(logging.DEBUG)


@overload
async def run[R](context: "Context", handler: Callable[..., Awaitable[R]], *arguments: object, **keywords: object) -> R: ...


@overload
async def run[R](context: "Context", handler: Callable[..., R], *arguments: object, **keywords: object) -> R: ...


async def run[R](context: "Context", handler: Callable[..., R | Awaitable[R]], *arguments: object, **keywords: object) -> R:
    try:
        execution = context._transport.scope()
    except OperationError as error:
        if error.value.code != "scope_required":
            raise
        execution = ""
    buffer = _Buffer(context._transport, execution)
    token = _CURRENT.set(buffer)
    try:
        result = handler(*arguments, **keywords)
        if inspect.isawaitable(result):
            return await result
        return cast(R, result)
    finally:
        try:
            async with buffer.lock:
                await buffer.flush()
        finally:
            _CURRENT.reset(token)


async def flush() -> None:
    buffer = _CURRENT.get()
    if buffer is not None:
        async with buffer.lock:
            await buffer.flush()


# === Private Functions === #

# === Entry Point === #
