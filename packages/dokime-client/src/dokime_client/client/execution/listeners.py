# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Lossless Record Replay And Saved Telemetry Following."""

# === Imports === #

# Standard Library Imports
import asyncio
import re
from collections.abc import AsyncGenerator, Awaitable, Sequence
from contextlib import suppress
from fnmatch import fnmatchcase

# Third Party Imports
from google.protobuf.message import Message

# Local Imports
from .. import _wire
from .._transport import Transport
from ..errors import ProtocolError, RpcError
from ..models import Backend

# Type Checking Imports

# === Public Exports === #

# === Constants === #

FILTER_BYTES = 4096
FILTER_ITEMS = 64
MAXIMUM_SAMPLE_PAGE_ITEMS = 5000

# === Types === #

# === Classes === #

# === Public Functions === #


async def read_records(
    transport: Transport,
    backend: Backend,
    scope: Message,
    record_types: Sequence[str],
    completion: Awaitable[object] | None,
) -> AsyncGenerator[Message]:
    validate_patterns(record_types)
    selection = _wire.nested("Evidence", "ListRecords", "selection", scope=scope, record_types=record_types)
    after = None
    if completion is not None:
        done = asyncio.ensure_future(completion)
        reader = None
        pending = None
        try:
            reader = await transport.stream(
                "Evidence",
                "WatchRecords",
                _wire.request("Evidence", "WatchRecords", selection=selection),
            )
            while True:
                pending = asyncio.create_task(anext(reader))
                finished, _ = await asyncio.wait((done, pending), return_when=asyncio.FIRST_COMPLETED)
                if done in finished:
                    await done
                    break
                try:
                    page = pending.result()
                except StopAsyncIteration as error:
                    raise ProtocolError("record reader ended before completion") from error
                _records(scope, backend.epoch, page, after)
                after = _wire.nested(
                    "Evidence",
                    "ListRecords",
                    "after",
                )
                _wire.copy(after, page.scanned)
                yield page
        finally:
            for task in (pending, done):
                if task is not None:
                    task.cancel()
                    with suppress(asyncio.CancelledError, StopAsyncIteration):
                        await task
            if reader is not None:
                await reader.close()
    await transport.verify(backend, live=True)
    request = _wire.request("Evidence", "ListRecords", selection=selection)
    page = await transport.unary("Evidence", "ListRecords", request)
    _records(scope, backend.epoch, page, None)
    frontier = _wire.nested(
        "Evidence",
        "ListRecords",
        "after",
    )
    _wire.copy(frontier, page.through)
    if after is not None:
        _wire.copy(request.after, after)
        _wire.copy(request.through, frontier)
        page = await transport.unary("Evidence", "ListRecords", request)
    while True:
        _records(
            scope,
            backend.epoch,
            page,
            request.after if request.HasField("after") else None,
        )
        if not _wire.equal(page.through, frontier):
            raise ProtocolError("record replay changed its fixed frontier")
        yield page
        if page.replay_complete:
            return
        _wire.copy(request.after, page.scanned)
        _wire.copy(request.through, frontier)
        page = await transport.unary("Evidence", "ListRecords", request)


async def read_samples(
    transport: Transport,
    backend: Backend,
    scope: Message,
    streams: Sequence[str],
    page_size: int,
    follow: bool,
) -> AsyncGenerator[Message]:
    validate_patterns(streams)
    if type(page_size) is not int or not 1 <= page_size <= MAXIMUM_SAMPLE_PAGE_ITEMS:
        raise ValueError("page size exceeds the configured maximum")
    positions: dict[str, int] = {}
    reconnect = True
    while True:
        await transport.verify(backend, live=True)
        selection = _wire.request(
            "Telemetry",
            "ListSamples",
            scope=scope,
            streams=streams,
            limit=page_size,
            after_stream_cursors=[{"key": key, "value": value} for key, value in sorted(positions.items())],
            latest=False,
        )
        reader = None
        try:
            reader = await transport.stream(
                "Telemetry",
                "WatchSamples",
                _wire.request("Telemetry", "WatchSamples", selection=selection, follow=follow),
            )
            async for page in reader:
                await transport.verify(backend, live=True)
                if len(page.samples) > page_size:
                    raise ProtocolError("telemetry page exceeds requested limit")
                through = {entry.key: entry.value for entry in page.through_stream_cursors}
                if len(through) != len(page.through_stream_cursors):
                    raise ProtocolError("telemetry page repeats a stream cursor")
                if any(stream in through and through[stream] < position for stream, position in positions.items()) or (
                    not page.complete and not any(position > positions.get(stream, -1) for stream, position in through.items())
                ):
                    raise ProtocolError("telemetry continuation did not advance")
                for item in page.samples:
                    selected = not streams or any(fnmatchcase(item.stream, pattern) for pattern in streams)
                    if not item.HasField("scope") or not _scope(scope, item.scope) or not selected:
                        raise ProtocolError("sample identity or stream differs")
                yield page
                positions.update(through)
                reconnect = True
            return
        except RpcError as error:
            if error.code != "UNAVAILABLE" or not reconnect:
                raise
            reconnect = False
        finally:
            if reader is not None:
                await reader.close()


def validate_patterns(patterns: Sequence[str]) -> None:
    if isinstance(patterns, str) or len(patterns) > FILTER_ITEMS:
        raise ValueError("listener filters exceed the configured item limit")
    for pattern in patterns:
        if (
            not isinstance(pattern, str)
            or len(pattern.encode("utf-8")) > FILTER_BYTES
            or re.fullmatch(r"[A-Za-z0-9_*?][A-Za-z0-9_.*?-]*", pattern) is None
        ):
            raise ValueError("listener filters accept identifier globs using only '*' and '?' wildcards")


# === Private Functions === #


def _scope(expected: Message, actual: Message) -> bool:
    return all(
        not expected.HasField(field) or actual.HasField(field) and getattr(actual, field) == getattr(expected, field)
        for field in ("execution_id", "cycle_id")
    )


def _records(
    scope: Message,
    epoch: str,
    page: Message,
    after: Message | None,
) -> None:
    if not page.HasField("scanned") or not page.HasField("through") or page.scanned.epoch != epoch or page.through.epoch != epoch:
        raise ProtocolError("record page epoch differs")
    scanned, through = page.scanned, page.through
    if (
        not scanned.stream
        or scanned.stream != through.stream
        or scanned.position > through.position
        or page.retained_after != 0
        or page.replay_complete
        and scanned.position != through.position
    ):
        raise ProtocolError("record page frontier is invalid or evidence was lost")
    if after is not None and (
        after.stream != scanned.stream
        or scanned.position < after.position
        or not page.replay_complete
        and scanned.position == after.position
    ):
        raise ProtocolError("record continuation did not advance")
    previous = after.position if after is not None else None
    for item in page.records:
        if (
            not item.HasField("scope")
            or not _scope(scope, item.scope)
            or item.sequence > scanned.position
            or previous is not None
            and item.sequence <= previous
        ):
            raise ProtocolError("record identity or journal position differs")
        previous = item.sequence


# === Entry Point === #
