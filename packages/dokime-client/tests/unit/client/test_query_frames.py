# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Captured Query Dataframe Fidelity"""

# === Imports === #

# Standard Library Imports
import asyncio
import math
from datetime import UTC, datetime
from typing import Any
from unittest.mock import AsyncMock

# Third Party Imports
import pytest

# Local Imports
from dokime_client import Aggregate, DokimeClient, QueryConversionError, QueryLimitError, QuerySettings
from dokime_client.client import _wire
from dokime_client.client.execution import Cycle, Execution
from dokime_client.client.models import DatetimeValue

from .test_query_discovery import query_page
from .test_query_evidence import BACKEND, CYCLE_ID, EXECUTION_ID, record_row

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def sample_row(identifier: str, values: dict[str, Any]) -> dict[str, Any]:
    return {
        "Raw": {
            "field_0": {
                "sample_id": identifier,
                "schema_version": 2,
                "sequence": 7,
                "stream": "example.stream",
                "source": "example.source",
                "timestamp": {"seconds": 1, "nanos": 123456789},
                "values": _wire.encode(values, 64),
            }
        }
    }


@pytest.mark.parametrize("extra", ["polars", "pandas"])
@pytest.mark.parametrize("scope", ["execution", "cycle", "telemetry"])
def test_query_frames_scalars(extra: str, scope: str, monkeypatch: pytest.MonkeyPatch) -> None:
    async def scenario() -> None:
        client = DokimeClient()
        execution = Execution._bind(EXECUTION_ID, client._transport, BACKEND)
        cycle = Cycle(CYCLE_ID, client._transport, BACKEND, ())
        instant = DatetimeValue.from_rfc3339("2026-10-01T02:00:00.123456789+02:00")
        values = [
            {"counter.total": 2**64 - 1, "signed": 2**53 + 1, "integer": -(2**63), "time": instant, "flag": True, "text": "a"},
            {"counter.total": None, "signed": None, "integer": 2**64 - 1, "time": None, "flag": False, "text": "b"},
            {},
        ]
        method = "ListSamples" if scope == "telemetry" else "ListRecords"
        rows = [sample_row(f"row-{i}", value) if scope == "telemetry" else record_row(f"row-{i}", value) for i, value in enumerate(values)]
        first = query_page(method, rows[:1], after="page-2")
        empty = query_page(method, [], after="page-3")
        last = query_page(method, rows[1:])
        schema: dict[str, Any] = {
            "identity": "example.contract",
            "versions": [1, 2],
            "fields": [
                {"id": "counter.total", "value_type": {"Uint64": {}}, "nullable": True, "unit": "items"},
                {"id": "signed", "value_type": {"Int64": {}}, "nullable": True},
                {"id": "integer", "value_type": {"Integer": {}}, "nullable": True},
                {"id": "time", "value_type": {"Datetime": {}}, "nullable": True},
                {"id": "flag", "value_type": {"Boolean": {}}, "nullable": True},
                {"id": "text", "value_type": {"Text": {}}, "nullable": True},
                {"id": "all_null", "value_type": {"Int64": {}}, "nullable": True},
            ],
        }
        for page in (first, empty, last):
            _wire.copy(page.schema, schema)
        call = AsyncMock(side_effect=[first, empty, last])
        monkeypatch.setattr(client._transport, "call", call)
        if scope == "telemetry":
            reader = execution.telemetry(
                "example.stream", fields=["counter.total", "signed", "integer", "time", "flag", "text", "all_null"]
            )
        else:
            reader = (cycle if scope == "cycle" else execution).query_records("example.event")
        frame: Any = await getattr(reader, f"to_{extra}")(max_rows=3)
        envelope = ["@id", "@timestamp", "@received_at", "@source", "@schema_version"]
        if scope != "telemetry":
            envelope.append("@severity")
        assert list(frame.columns) == envelope + ["counter.total", "signed", "integer", "time", "flag", "text", "all_null"]
        assert list(frame["@id"]) == ["row-0", "row-1", "row-2"]
        assert int(frame["counter.total"][0]) == 2**64 - 1 and int(frame["signed"][0]) == 2**53 + 1
        assert int(frame["integer"][0]) == -(2**63) and int(frame["integer"][1]) == 2**64 - 1
        if extra == "polars":
            assert str(frame.schema["counter.total"]) == "UInt64" and str(frame.schema["signed"]) == "Int64"
            assert str(frame.schema["integer"]) == "Int128" and str(frame.schema["all_null"]) == "Int64"
            assert frame["all_null"].null_count() == 3 and frame["counter.total"].null_count() == 2
            assert frame["time"].cast(__import__("polars").Int64)[0] == 1790812800123456789
            assert str(frame.schema["time"]) == "Datetime(time_unit='ns', time_zone='UTC')"
            if scope == "telemetry":
                assert frame["@timestamp"].cast(__import__("polars").Int64)[0] == 1123456789
        else:
            assert str(frame.dtypes["counter.total"]) == "UInt64" and str(frame.dtypes["signed"]) == "Int64"
            assert str(frame.dtypes["integer"]) == "object" and str(frame.dtypes["all_null"]) == "Int64"
            assert frame["all_null"].isna().sum() == 3 and frame["counter.total"].isna().sum() == 2
            assert frame["time"][0].value == 1790812800123456789 and str(frame.dtypes["time"]) == "datetime64[ns, UTC]"
            info = frame.attrs["dokime"]
            assert info["schema"].SerializeToString() == last.schema.SerializeToString()
            assert info["snapshot"] == reader.snapshot and info["complete"]
            assert info["projection"] == tuple(field["id"] for field in schema["fields"])
            assert info["units"]["counter.total"] == "items" and info["versions"] == (1, 2)
        assert reader.complete and reader.closed and call.await_count == 3
        assert reader.metadata is not None and reader.metadata.schema is not None
        assert reader.metadata.schema.SerializeToString() == last.schema.SerializeToString()
        with pytest.raises(RuntimeError):
            await getattr(reader, f"to_{extra}")()
        await client.close()

    asyncio.run(scenario())


@pytest.mark.parametrize("extra", ["polars", "pandas"])
@pytest.mark.parametrize("scope", ["record", "sample", "bucket"])
def test_query_frames_empty_bucket(extra: str, scope: str, monkeypatch: pytest.MonkeyPatch) -> None:
    async def scenario() -> None:
        bucketed = scope == "bucket"
        client = DokimeClient()
        execution = Execution._bind(EXECUTION_ID, client._transport, BACKEND)
        rows = []
        if bucketed:
            rows = [
                {
                    "Bucket": {
                        "field_0": {
                            "start": {"seconds": 1, "nanos": 1},
                            "end": {"seconds": 2, "nanos": 1},
                            "execution_id": EXECUTION_ID,
                            "source": "example.source",
                            "sample_count": 2,
                            "fields": [
                                {"field": "load", "count": 1, "value": {"Number": {"field_0": 0.5}}, "reducer": Aggregate.mean().to_wire()}
                            ],
                        }
                    }
                }
            ]
        page = query_page("ListRecords" if scope == "record" else "ListSamples", rows)
        _wire.copy(
            page.schema,
            {
                "identity": "example.stream",
                "versions": [1],
                "fields": [
                    {"id": "load", "value_type": {"Float64": {}}, "nullable": True, "unit": "%"},
                    {"id": "absent", "value_type": {"Uint64": {}}, "nullable": True},
                ],
            },
        )
        call = AsyncMock(return_value=page)
        monkeypatch.setattr(client._transport, "call", call)
        reader = (
            execution.query_records("example.event")
            if scope == "record"
            else execution.telemetry("example.stream", every="1s" if bucketed else None, agg=Aggregate.mean() if bucketed else None)
        )
        frame: Any = await getattr(reader, f"to_{extra}")()
        assert len(frame) == (1 if bucketed else 0) and list(frame.columns)[-2:] == ["load", "absent"]
        if extra == "polars":
            assert str(frame.schema["load"]) == "Float64" and str(frame.schema["absent"]) == "UInt64"
        else:
            assert str(frame.dtypes["load"]) == "Float64" and str(frame.dtypes["absent"]) == "UInt64"
            assert frame.attrs["dokime"]["complete"]
        if bucketed:
            assert float(frame["load"][0]) == 0.5 and frame["@execution_id"][0] == EXECUTION_ID
        assert reader.complete and reader.metadata is not None and reader.metadata.schema is not None
        assert reader.metadata.schema.fields[0].unit == "%"
        await client.close()

    asyncio.run(scenario())


@pytest.mark.parametrize("extra", ["polars", "pandas"])
@pytest.mark.parametrize(
    "kind,value",
    [
        ("Object", {}),
        ("Array", []),
        ("Int64", 2**63),
        ("Uint64", -1),
        ("Int64", True),
        ("Int64", 1.5),
        ("Int64", None),
        ("Text", {"a": 1}),
        ("Datetime", datetime(2500, 1, 1, tzinfo=UTC)),
    ],
)
def test_query_frames_errors(extra: str, kind: str, value: Any, monkeypatch: pytest.MonkeyPatch) -> None:
    async def scenario() -> None:
        client = DokimeClient()
        execution = Execution._bind(EXECUTION_ID, client._transport, BACKEND)
        page = query_page("ListRecords", [record_row(details={"value": value})])
        _wire.copy(
            page.schema,
            {
                "identity": "example.event",
                "fields": [
                    {"id": "value", "value_type": {kind: {}}, "nullable": False},
                ],
            },
        )
        monkeypatch.setattr(client._transport, "call", AsyncMock(return_value=page))
        reader = execution.query_records("example.event")
        with pytest.raises(QueryConversionError) as caught:
            await getattr(reader, f"to_{extra}")()
        assert caught.value.field == "value" and reader.closed
        if kind in ("Object", "Array"):
            page.ClearField("rows")
            with pytest.raises(QueryConversionError):
                await getattr(execution.query_records("example.event"), f"to_{extra}")()
        await client.close()

    asyncio.run(scenario())


@pytest.mark.parametrize("extra", ["polars", "pandas"])
@pytest.mark.parametrize("failure", ["rows", "bytes", "late_error"])
def test_query_frames_atomic(extra: str, failure: str, monkeypatch: pytest.MonkeyPatch) -> None:
    async def scenario() -> None:
        client = DokimeClient(query_settings=QuerySettings(max_rows=1, max_bytes=100_000))
        execution = Execution._bind(EXECUTION_ID, client._transport, BACKEND)
        call = AsyncMock(
            side_effect=[
                query_page("ListRecords", [record_row()], after="page-2"),
                RuntimeError("lost evidence") if failure == "late_error" else query_page("ListRecords", [record_row()]),
            ]
        )
        monkeypatch.setattr(client._transport, "call", call)
        reader = execution.query_records("example.event")
        with pytest.raises(RuntimeError if failure == "late_error" else QueryLimitError):
            await getattr(reader, f"to_{extra}")(max_rows=None if failure == "rows" else 2, max_bytes=1 if failure == "bytes" else None)
        assert call.await_count == (1 if failure == "bytes" else 2) and reader.closed and not reader.complete
        await client.close()

    asyncio.run(scenario())


@pytest.mark.parametrize("extra", ["polars", "pandas"])
def test_query_frames_nonfinite(extra: str, monkeypatch: pytest.MonkeyPatch) -> None:
    async def scenario() -> None:
        client = DokimeClient()
        execution = Execution._bind(EXECUTION_ID, client._transport, BACKEND)
        page = query_page(
            "ListSamples", [sample_row(str(i), {"value": value}) for i, value in enumerate([float("nan"), None, float("inf")])]
        )
        _wire.copy(
            page.schema, {"identity": "example.stream", "fields": [{"id": "value", "value_type": {"Float64": {}}, "nullable": True}]}
        )
        monkeypatch.setattr(client._transport, "call", AsyncMock(return_value=page))
        frame: Any = await getattr(execution.telemetry("example.stream"), f"to_{extra}")()
        assert math.isnan(frame["value"][0]) and math.isinf(frame["value"][2])
        missing = frame["value"].is_null() if extra == "polars" else frame["value"].isna()
        assert list(missing) == [False, True, False]
        await client.close()

    asyncio.run(scenario())


# === Private Functions === #

# === Entry Point === #
