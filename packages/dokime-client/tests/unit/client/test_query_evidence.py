# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Finite Query Evidence Contracts"""

# === Imports === #

# Standard Library Imports
import asyncio
from typing import Any
from unittest.mock import AsyncMock

# Third Party Imports
import pytest

# Local Imports
from dokime_client import Aggregate, Bucket, DokimeClient, Predicate
from dokime_client.client import _wire
from dokime_client.client.execution import Cycle, Execution
from dokime_client.client.models import Backend, Sample, UnsignedValue

from .test_query_discovery import execution_row, query_page

# Type Checking Imports


# === Public Exports === #

# === Constants === #

EXECUTION_ID = "00000000-0000-0000-0000-000000000001"
CYCLE_ID = "00000000-0000-0000-0000-000000000002"
BACKEND = Backend("example", "epoch", "1", "Example", frozenset())

# === Types === #

# === Classes === #

# === Public Functions === #


def record_row(identifier: str = "record-1", details: dict[str, Any] | None = None) -> dict[str, Any]:
    return {
        "record_id": identifier,
        "record_type": "example.event",
        "schema_version": 2,
        "sequence": 7,
        "source": "example.source",
        "severity": {"Info": {}},
        "received_at": {"seconds": 1, "nanos": 2},
        "details": _wire.encode({"counter": 2**63 + 1, "nullable": None} if details is None else details, 64),
        "scope": {"execution_id": "producer"},
    }


def test_query_evidence_sparse_schema_and_cut(monkeypatch: pytest.MonkeyPatch) -> None:
    async def scenario() -> None:
        client = DokimeClient()
        execution = Execution._bind(EXECUTION_ID, client._transport, BACKEND)
        empty = query_page("ListRecords", [], after="scan-1")
        reply = query_page("ListRecords", [record_row()])
        _wire.copy(
            reply.schema,
            {
                "identity": "example.event",
                "versions": [1, 2],
                "fields": [
                    {"id": "counter", "value_type": {"Uint64": {}}, "nullable": False, "unit": "items"},
                    {"id": "nullable", "value_type": {"Float64": {}}, "nullable": True},
                ],
            },
        )
        _wire.copy(empty.schema, reply.schema)
        empty.limits.rows = 7
        reply.limits.rows = 7
        calls = AsyncMock(side_effect=[empty, reply])
        monkeypatch.setattr(client._transport, "call", calls)
        reader = execution.query_records("example.event", where=[Predicate("nullable", "exists", False)], versions=[1, 2], page_rows=7)
        assert calls.await_count == 0
        rows = [row async for row in reader]
        assert len(rows) == 1 and rows[0].scope["execution_id"] == "producer"
        assert rows[0].details["counter"] == 2**63 + 1 and isinstance(rows[0].details["counter"], UnsignedValue)
        assert rows[0].details["nullable"] is None and "absent" not in rows[0].details
        assert rows[0].received_at is not None and rows[0].received_at.isoformat().startswith("1970-01-01T00:00:01")
        assert reader.metadata is not None and reader.metadata.schema is not None and reader.metadata.schema.fields[0].unit == "items"
        telemetry = execution.telemetry("example.stream", fields=["counter"], at=reader.snapshot)
        assert telemetry._request.at.Token.field_0 == "cut" and telemetry._request.fields == ["counter"]
        assert reader._request.page.after == "scan-1" and calls.await_count == 2
        await client.close()

    asyncio.run(scenario())


def test_query_evidence_raw_bucket_and_summary(monkeypatch: pytest.MonkeyPatch) -> None:
    async def scenario() -> None:
        client = DokimeClient()
        samples = query_page(
            "ListSamples",
            [
                {
                    "Raw": {
                        "field_0": {
                            "sample_id": "sample-1",
                            "stream": "example.stream",
                            "schema_version": 1,
                            "values": _wire.encode({"counter": 2**63 + 1, "nullable": None}, 64),
                        }
                    }
                }
            ],
        )
        buckets = query_page(
            "ListSamples",
            [
                {
                    "Bucket": {
                        "field_0": {
                            "start": {"seconds": 1},
                            "end": {"seconds": 2},
                            "execution_id": EXECUTION_ID,
                            "stream": "example.stream",
                            "source": "example.source",
                            "versions": [1],
                            "sample_count": 2,
                            "fields": [
                                {
                                    "field": "rssi",
                                    "count": 1,
                                    "value": {"Number": {"field_0": -40.5}},
                                    "unit": "dBm",
                                    "reducer": Aggregate.mean().to_wire(),
                                }
                            ],
                        }
                    }
                }
            ],
        )
        calls = AsyncMock(side_effect=[query_page("ListExecutions", [execution_row(EXECUTION_ID)]), samples, buckets])
        monkeypatch.setattr(client._transport, "call", calls)
        summary = await anext(client.executions().__aiter__())
        raw = summary.telemetry("example.stream")
        sample = await anext(raw.__aiter__())
        assert isinstance(sample, Sample) and sample.values["counter"] == 2**63 + 1 and sample.values["nullable"] is None
        bucketed = summary.telemetry("example.stream", fields=["rssi"], every="1s", agg=Aggregate.mean())
        bucket = await anext(bucketed.__aiter__())
        assert isinstance(bucket, Bucket) and bucket.fields[0].value == -40.5 and bucket.fields[0].reducer == Aggregate.mean()
        assert bucketed._request.bucket.every_nanos == 10**9
        cycle = Cycle(CYCLE_ID, client._transport, BACKEND, ())
        records = cycle.query_records("example.event", execution_id=EXECUTION_ID)
        assert records._request.scope.Cycle.field_0.execution_id == EXECUTION_ID
        assert records._request.scope.Cycle.field_0.cycle_id == CYCLE_ID
        await client.close()

    asyncio.run(scenario())


def test_query_evidence_early_close_and_cancel(monkeypatch: pytest.MonkeyPatch) -> None:
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
        reader = execution.query_records("example.event")
        items = reader.__aiter__()
        task = asyncio.ensure_future(anext(items))
        await entered.wait()
        await reader.aclose()
        with pytest.raises(asyncio.CancelledError):
            await task
        assert cancelled.is_set() and mock.await_count == 1
        with pytest.raises(StopAsyncIteration):
            await anext(items)
        assert mock.await_count == 1
        await client.close()

    asyncio.run(scenario())


@pytest.mark.parametrize(
    "kwargs",
    [
        {"every": "1s"},
        {"agg": Aggregate.mean()},
        {"every": "0s", "agg": Aggregate.mean()},
        {"every": "1.00000000000000000000000000001ns", "agg": Aggregate.mean()},
        {"every": "1s", "agg": Aggregate.percentile(95)},
        {"fields": ["rssi", "rssi"]},
        {"versions": [0]},
        {"versions": [1, 1]},
    ],
)
def test_query_evidence_invalid_shapes(kwargs: dict[str, Any]) -> None:
    client = DokimeClient()
    execution = Execution._bind(EXECUTION_ID, client._transport, BACKEND)
    with pytest.raises((TypeError, ValueError)):
        execution.telemetry("example.stream", **kwargs)


# === Private Functions === #

# === Entry Point === #
