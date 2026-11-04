# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Finite Query Lifecycle And Materialization"""

# === Imports === #

# Standard Library Imports
import asyncio
from typing import Any, cast
from unittest.mock import AsyncMock

# Third Party Imports
import pytest

# Local Imports
from dokime_client import DokimeClient, QueryLimitError, QuerySettings
from dokime_client.client.errors import ProtocolError, RpcError
from dokime_client.client.execution import Execution
from dokime_client.client.history import Checkpoint, StorageCursor, StoreHistory

from .test_query_discovery import execution_row, query_page
from .test_query_evidence import BACKEND, EXECUTION_ID, record_row

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


@pytest.mark.parametrize("mode", ["items", "pages"])
def test_query_lifecycle_backpressure_and_context_close(mode: str, monkeypatch: pytest.MonkeyPatch) -> None:
    async def scenario() -> None:
        client = DokimeClient()
        reply = query_page("ListExecutions", [execution_row(), execution_row("execution-2")], after="next")
        call = AsyncMock(return_value=reply)
        monkeypatch.setattr(client._transport, "call", call)
        reader = client.executions()
        async with reader:
            call.assert_not_called()
            iterator = reader.__aiter__() if mode == "items" else reader.pages()
            await anext(iterator)
            resumed = asyncio.Event()
            asyncio.get_running_loop().call_soon(resumed.set)
            await resumed.wait()
            assert call.await_count == 1 and reader.snapshot is not None and not reader.complete
            with pytest.raises(RuntimeError):
                await reader.collect()
        assert reader.closed
        with pytest.raises(StopAsyncIteration):
            await anext(iterator)
        assert call.await_count == 1
        await reader.aclose()
        await reader.aclose()
        with pytest.raises(RuntimeError):
            async with reader:
                pass
        await client.close()

    asyncio.run(scenario())


def test_query_lifecycle_collect_sparse_and_schema_optional(monkeypatch: pytest.MonkeyPatch) -> None:
    async def scenario() -> None:
        client = DokimeClient(query_settings=QuerySettings(max_rows=2, max_bytes=100_000))
        replies = [
            query_page("ListExecutions", [], after="scanned"),
            query_page("ListExecutions", [execution_row(), execution_row("execution-2")]),
        ]
        for reply in replies:
            reply.ClearField("schema")
        call = AsyncMock(side_effect=replies)
        monkeypatch.setattr(client._transport, "call", call)
        reader = client.executions()
        rows = await reader.collect()
        assert [row.id for row in rows] == ["execution-1", "execution-2"]
        assert reader.complete and reader.closed and reader.metadata is not None and reader.metadata.schema is None
        assert call.await_count == 2
        with pytest.raises(RuntimeError):
            reader.__aiter__()
        await client.close()

    asyncio.run(scenario())


@pytest.mark.parametrize("failure", ["rows", "bytes", "late_error", "incomplete", "schema", "snapshot"])
def test_query_lifecycle_collect_atomic_failure(failure: str, monkeypatch: pytest.MonkeyPatch) -> None:
    async def scenario() -> None:
        client = DokimeClient(
            query_settings=QuerySettings(max_rows=1 if failure == "rows" else 10, max_bytes=6000 if failure == "bytes" else 100_000)
        )
        execution = Execution._bind(EXECUTION_ID, client._transport, BACKEND)
        first = query_page("ListRecords", [record_row()], after="page-2")
        details = {"large": "x" * 10_000} if failure == "bytes" else {"value": 2}
        second = query_page("ListRecords", [record_row("record-2", details)])
        if failure == "incomplete":
            second.complete = False
        if failure == "schema":
            second.schema.versions.append(2)
        if failure == "snapshot":
            second.snapshot.token = "another-cut"
        error = RpcError("UNAVAILABLE", "evidence lost")
        call = AsyncMock(side_effect=[first, error if failure == "late_error" else second])
        monkeypatch.setattr(client._transport, "call", call)
        reader = execution.query_records("example.event")
        result: Any = "unpublished"
        with pytest.raises(QueryLimitError if failure in ("rows", "bytes") else (RpcError, ProtocolError)) as caught:
            result = await reader.collect()
        assert result == "unpublished" and reader.closed and not reader.complete
        assert call.await_count == 2
        assert reader.snapshot is not None and reader.snapshot.token == "cut"
        if isinstance(caught.value, QueryLimitError):
            assert caught.value.limit == {"name": f"max_{failure}", "value": 1 if failure == "rows" else 6000}
        await client.close()

    asyncio.run(scenario())


def test_query_lifecycle_collect_explicit_limits_and_empty_metadata(monkeypatch: pytest.MonkeyPatch) -> None:
    async def scenario() -> None:
        client = DokimeClient(query_settings=QuerySettings(max_rows=1, max_bytes=100_000))
        reply = query_page("ListExecutions", [execution_row(), execution_row("execution-2")])
        call = AsyncMock(return_value=reply)
        monkeypatch.setattr(client._transport, "call", call)
        assert len(await client.executions().collect(max_rows=2)) == 2
        with pytest.raises(QueryLimitError):
            await client.executions().collect(max_bytes=1)
        empty = query_page("ListExecutions", [])
        call.return_value = empty
        reader = client.executions()
        assert await reader.collect() == [] and reader.complete
        # Empty Collections Still Charge Their Captured Metadata.
        with pytest.raises(QueryLimitError):
            await client.executions().collect(max_bytes=1)
        await client.close()

    asyncio.run(scenario())


def test_query_lifecycle_checkpoint_and_same_cut(monkeypatch: pytest.MonkeyPatch) -> None:
    async def scenario() -> None:
        client = DokimeClient()
        position = StorageCursor(StoreHistory("example", "history"), 1)
        checkpoint = Checkpoint(EXECUTION_ID, position, b"\x07example\x07history\x01", position)
        execution = Execution._bind(EXECUTION_ID, client._transport, BACKEND)
        reader = execution.query_records("example.event", at=checkpoint)
        assert reader._request.at.Checkpoint.field_0.execution_identifier == EXECUTION_ID
        assert reader._request.at.Checkpoint.field_0.evidence_through.sequence == 1
        call = AsyncMock(return_value=query_page("ListRecords", [record_row()], after="next"))
        monkeypatch.setattr(client._transport, "call", call)
        async with reader:
            page = await anext(reader.pages())
            other = execution.telemetry("example.stream", at=page.snapshot)
            assert other._request.at.Token.field_0 == "cut"
            call.return_value = query_page("ListSamples", [], token="another-cut")
            with pytest.raises(ProtocolError):
                await other.collect()
        assert not reader.complete and reader.closed and call.await_count == 2
        await client.close()

    asyncio.run(scenario())


def test_query_lifecycle_cancel_and_concurrent_consumer(monkeypatch: pytest.MonkeyPatch) -> None:
    async def scenario() -> None:
        client = DokimeClient()
        entered, cancelled = asyncio.Event(), asyncio.Event()

        async def call(op: Any, request: Any) -> Any:
            entered.set()
            try:
                await asyncio.Event().wait()
            finally:
                cancelled.set()

        mock = AsyncMock(side_effect=call)
        monkeypatch.setattr(client._transport, "call", mock)
        reader = client.executions()
        task = asyncio.create_task(reader.collect())
        await entered.wait()
        with pytest.raises(RuntimeError):
            reader.pages()
        task.cancel()
        with pytest.raises(asyncio.CancelledError):
            await task
        assert cancelled.is_set() and reader.closed and not reader.complete and mock.await_count == 1
        await client.close()

    asyncio.run(scenario())


def test_query_lifecycle_final_page_early_close_is_incomplete(monkeypatch: pytest.MonkeyPatch) -> None:
    async def scenario() -> None:
        client = DokimeClient()
        call = AsyncMock(return_value=query_page("ListExecutions", [execution_row(), execution_row("execution-2")]))
        monkeypatch.setattr(client._transport, "call", call)
        reader = client.executions()
        async with reader:
            items = reader.__aiter__()
            await anext(items)
            assert reader.metadata is not None and reader.metadata.complete
            assert not reader.complete
        with pytest.raises(StopAsyncIteration):
            await anext(items)
        assert not reader.complete and call.await_count == 1
        await client.close()

    asyncio.run(scenario())


@pytest.mark.parametrize("value", [0, -1, True, 1.5])
def test_query_lifecycle_invalid_limits(value: Any) -> None:
    async def scenario() -> None:
        client = DokimeClient()
        reader = client.executions()
        with pytest.raises(ValueError):
            await reader.collect(max_rows=value)
        with pytest.raises(ValueError):
            await reader.collect(max_bytes=value)
        assert reader._mode is None
        with pytest.raises(ValueError):
            QuerySettings(max_rows=value)
        with pytest.raises(ValueError):
            QuerySettings(max_bytes=cast(Any, None))
        await client.close()

    asyncio.run(scenario())


# === Private Functions === #

# === Entry Point === #
