# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Server Statistics Query Contracts"""

# === Imports === #

# Standard Library Imports
import asyncio
from typing import Any, cast
from unittest.mock import AsyncMock

# Third Party Imports
import pytest

# Local Imports
from dokime_client import Aggregate, DokimeClient, QueryLimitError
from dokime_client.client import _wire
from dokime_client.client._contracts import operation
from dokime_client.client.errors import ProtocolError, RpcError
from dokime_client.client.execution import Execution

from .test_query_evidence import BACKEND, EXECUTION_ID

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def statistics_reply() -> Any:
    reply = operation("Query", "AggregateSamples").response()
    _wire.copy(
        reply,
        {
            "schema": {
                "identity": "example.stream",
                "versions": [2],
                "fields": [
                    {"id": "mean", "value_type": {"Float64": {}}, "nullable": True, "unit": "dBm"},
                    {"id": "p95", "value_type": {"Float64": {}}, "nullable": True, "unit": "dBm"},
                ],
            },
            "snapshot": {"token": "cut", "identity": "snapshot-1"},
            "selected_rows": 5,
            "quality_counts": [{"field": "rssi", "finite": 4, "null": 1}],
            "complete": True,
            "values": [
                {"alias": "mean", "field": "rssi", "aggregate": Aggregate.mean().to_wire(), "value": {"Number": {"field_0": 15.0}}},
                {"alias": "p95", "field": "rssi", "aggregate": Aggregate.percentile(95).to_wire(), "value": {"Number": {"field_0": 28.5}}},
            ],
        },
    )
    return reply


def test_query_statistics_one_request_and_captured_schema(monkeypatch: pytest.MonkeyPatch) -> None:
    async def scenario() -> None:
        client = DokimeClient()
        execution = Execution._bind(EXECUTION_ID, client._transport, BACKEND)
        calls = AsyncMock(return_value=statistics_reply())
        monkeypatch.setattr(client._transport, "call", calls)
        reader = execution.telemetry(
            "example.stream", fields=["unrelated"], where={"rssi": 10}, versions=[2], between=("2026-10-01", "2026-10-02")
        )
        result = await reader.aggregate("rssi", [Aggregate.mean(), Aggregate.percentile(95)])
        assert [value.name for value in result.values] == ["mean", "p95"]
        assert [value.value for value in result.values] == [15.0, 28.5]
        assert result.selected_rows == 5 and result.quality_counts[0].null == 1 and result.quality_counts[0].finite == 4
        assert result.schema.versions == [2] and result.schema.fields[0].unit == "dBm"
        assert result.complete and reader.snapshot == result.snapshot and reader.statistics is result and reader.metadata is result
        calls.assert_awaited_once()
        op, request = calls.call_args.args
        assert op.id == "dokime.query.aggregate_samples" and request.execution_id == EXECUTION_ID
        assert request.versions == [2] and request.HasField("between")
        assert request.DESCRIPTOR.fields_by_name.get("fields") is None
        assert [value.alias for value in request.outputs] == ["mean", "p95"]
        assert request.outputs[1].aggregate.Percentile.rank == 95
        assert len(request.where) == 1
        with pytest.raises(RuntimeError):
            reader.pages()
        await client.close()

    asyncio.run(scenario())


@pytest.mark.parametrize(
    "aggregates",
    [
        ["mean"],
        [],
        [Aggregate.mean(), Aggregate.mean()],
        [Aggregate.percentile(cast(Any, 95.5))],
        [Aggregate.percentile(cast(Any, True))],
        [Aggregate(cast(Any, "Unknown"))],
    ],
)
def test_query_statistics_invalid_aggregates(aggregates: Any, monkeypatch: pytest.MonkeyPatch) -> None:
    async def scenario() -> None:
        client = DokimeClient()
        execution = Execution._bind(EXECUTION_ID, client._transport, BACKEND)
        calls = AsyncMock()
        monkeypatch.setattr(client._transport, "call", calls)
        reader = execution.telemetry("example.stream")
        with pytest.raises((TypeError, ValueError)):
            await reader.aggregate("rssi", aggregates)
        calls.assert_not_called()
        await client.close()

    asyncio.run(scenario())


def test_query_statistics_mixed_consumption_and_buckets(monkeypatch: pytest.MonkeyPatch) -> None:
    async def scenario() -> None:
        client = DokimeClient()
        execution = Execution._bind(EXECUTION_ID, client._transport, BACKEND)
        call = AsyncMock()
        monkeypatch.setattr(client._transport, "call", call)
        for mode in ("items", "pages"):
            reader = execution.telemetry("example.stream")
            iterator = reader.__aiter__() if mode == "items" else reader.pages()
            with pytest.raises(RuntimeError):
                await reader.aggregate("rssi", [Aggregate.mean()])
            await reader.aclose()
            del iterator
        bucketed = execution.telemetry("example.stream", every="1s", agg=Aggregate.mean())
        with pytest.raises(ValueError):
            await bucketed.aggregate("rssi", [Aggregate.mean()])
        call.assert_not_called()
        await client.close()

    asyncio.run(scenario())


@pytest.mark.parametrize("failure", ["limit", "incomplete", "wrong_order"])
def test_query_statistics_failures(failure: str, monkeypatch: pytest.MonkeyPatch) -> None:
    async def scenario() -> None:
        client = DokimeClient()
        execution = Execution._bind(EXECUTION_ID, client._transport, BACKEND)
        reply = statistics_reply()
        if failure == "incomplete":
            reply.complete = False
        if failure == "wrong_order":
            reply.values[0].alias = "unexpected"
        call = (
            AsyncMock(side_effect=RpcError("RESOURCE_EXHAUSTED", "work exhausted")) if failure == "limit" else AsyncMock(return_value=reply)
        )
        monkeypatch.setattr(client._transport, "call", call)
        reader = execution.telemetry("example.stream")
        with pytest.raises(QueryLimitError if failure == "limit" else ProtocolError):
            await reader.aggregate("rssi", [Aggregate.mean(), Aggregate.percentile(95)])
        assert reader.statistics is None and not reader.complete
        call.assert_awaited_once()
        await client.close()

    asyncio.run(scenario())


def test_query_statistics_close_inflight(monkeypatch: pytest.MonkeyPatch) -> None:
    async def scenario() -> None:
        client = DokimeClient()
        execution = Execution._bind(EXECUTION_ID, client._transport, BACKEND)
        entered, cancelled = asyncio.Event(), asyncio.Event()

        async def call(op: Any, request: Any) -> Any:
            entered.set()
            try:
                await asyncio.Event().wait()
            finally:
                cancelled.set()

        mock = AsyncMock(side_effect=call)
        monkeypatch.setattr(client._transport, "call", mock)
        reader = execution.telemetry("example.stream")
        task = asyncio.create_task(reader.aggregate("rssi", [Aggregate.mean()]))
        await entered.wait()
        await reader.aclose()
        with pytest.raises(asyncio.CancelledError):
            await task
        assert cancelled.is_set() and reader.closed and reader.metadata is None and not reader.complete
        with pytest.raises(RuntimeError):
            await reader.aggregate("rssi", [Aggregate.mean()])
        mock.assert_awaited_once()
        await client.close()

    asyncio.run(scenario())


# === Private Functions === #

# === Entry Point === #
