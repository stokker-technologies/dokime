# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Finite Query Discovery Contracts"""

# === Imports === #

# Standard Library Imports
import asyncio
import json
from datetime import UTC, datetime
from typing import Any, cast
from unittest.mock import AsyncMock

# Third Party Imports
import pytest

# Local Imports
from dokime_client import DokimeClient, Predicate, QueryError, QueryLimitError, QuerySettings, QuerySnapshotError
from dokime_client.client import _wire
from dokime_client.client._contracts import operation
from dokime_client.client.errors import RpcError

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def query_page(method: str, rows: list[dict[str, Any]], *, after: str | None = None, token: str = "cut") -> Any:
    response = operation("Query", method).response()
    _wire.copy(
        response,
        {
            "rows": rows,
            "schema": {"identity": "example.contract", "versions": [1]},
            "snapshot": {"token": token, "identity": "snapshot-1"},
            "limits": {"rows": 10, "bytes": 65536},
            "next": after,
            "complete": after is None,
        },
    )
    return response


def execution_row(identifier: str = "execution-1") -> dict[str, Any]:
    return {
        "execution_id": identifier,
        "case_id": "case-1",
        "template_id": "example.measurement",
        "status": {"Failed": {}},
        "availability": {"Live": {}},
        "checkpoint_available": False,
    }


def test_query_discovery_lazy_filters_paging(monkeypatch: pytest.MonkeyPatch) -> None:
    async def scenario() -> None:
        client = DokimeClient(query_settings=QuerySettings(page_rows=5))
        requests: list[Any] = []

        async def call(op: Any, request: Any) -> Any:
            requests.append(type(request).FromString(request.SerializeToString()))
            return query_page("ListExecutions", [execution_row(f"execution-{len(requests)}")], after="next" if len(requests) == 1 else None)

        mock = AsyncMock(side_effect=call)
        monkeypatch.setattr(client._transport, "call", mock)
        reader = client.executions(
            template="example.measurement",
            where=[Predicate("status", "in", ["failed", "passed"])],
            between=("2026-10-01", "2026-10-02T02:00:00+02:00"),
        )
        mock.assert_not_called()
        with pytest.raises(TypeError):
            await cast(Any, reader)
        rows = [row async for row in reader]
        assert [row.id for row in rows] == ["execution-1", "execution-2"]
        assert rows[0].status == "failed" and not rows[0].checkpoint_available
        assert requests[0].selection.template_ids == ["example.measurement"]
        assert [_wire.enum_name(item) for item in requests[0].selection.statuses] == ["failed", "passed"]
        assert requests[0].selection.started_at.end.seconds - requests[0].selection.started_at.start.seconds == 86400
        assert requests[0].page.rows == 5 and not requests[0].page.HasField("after")
        assert requests[1].page.after == "next" and requests[1].at.Token.field_0 == "cut"
        assert reader.snapshot is not None and reader.snapshot.token == "cut" and reader.complete
        with pytest.raises(RuntimeError):
            reader.pages()
        await client.close()

    asyncio.run(scenario())


def test_query_discovery_cycles_snapshots_and_consumer(monkeypatch: pytest.MonkeyPatch) -> None:
    async def scenario() -> None:
        client = DokimeClient()
        reply = query_page("ListCycles", [{"cycle_id": "cycle-1", "phase": {"Active": {}}, "execution_count": 3}])
        call = AsyncMock(return_value=reply)
        monkeypatch.setattr(client._transport, "call", call)
        reader = client.cycles(phase="active", archive="all")
        pages = reader.pages()
        with pytest.raises(RuntimeError):
            reader.__aiter__()
        page = await anext(pages)
        assert page.rows[0].phase == "active" and page.rows[0].execution_count == 3
        other = client.cycles(at=reader.snapshot)
        assert other._request.at.Token.field_0 == "cut"
        assert call.await_count == 1
        await pages.aclose()
        await other.aclose()
        await client.close()

    asyncio.run(scenario())


@pytest.mark.parametrize(
    "kwargs",
    [
        {"since": datetime(2026, 1, 1)},
        {"between": ("2026-10-02", "2026-10-01")},
        {"since": "2026-01-01T00:00:00.1234567890Z"},
        {"status": "completed"},
        {"where": [Predicate("status", "gt", "failed")]},
        {"page_rows": 0},
        {"at": "cut"},
    ],
)
def test_query_discovery_invalid_input(kwargs: dict[str, Any]) -> None:
    with pytest.raises((ValueError, TypeError)):
        DokimeClient().executions(**kwargs)


def test_query_discovery_nanoseconds_and_predicates() -> None:
    reader = DokimeClient().executions(between=("2026-10-01T00:00:00.000000001Z", "2026-10-01T00:00:00.000000002Z"))
    assert reader._request.selection.started_at.start.nanos == 1
    assert reader._request.selection.started_at.end.nanos == 2
    aware = DokimeClient().cycles(since=datetime(2026, 10, 1, tzinfo=UTC))
    assert aware._request.selection.created_at.start.nanos == 0
    wire = Predicate("counter", "eq", 2**63 + 1).to_wire()
    assert wire["values"] == [{"Unsigned": {"field_0": 2**63 + 1}}]
    for value in (None, float("inf"), {}, []):
        with pytest.raises((TypeError, ValueError)):
            Predicate("field", "eq", cast(Any, value))


@pytest.mark.parametrize(
    "reason,code,kind",
    [
        ("SnapshotExpired", "FAILED_PRECONDITION", QuerySnapshotError),
        ("ResourceLimit", "RESOURCE_EXHAUSTED", QueryLimitError),
        ("UnknownField", "INVALID_ARGUMENT", QueryError),
    ],
)
def test_query_discovery_error_details(reason: str, code: str, kind: type[QueryError], monkeypatch: pytest.MonkeyPatch) -> None:
    from google.protobuf.internal.encoder import _VarintBytes

    def field(number: int, payload: bytes) -> bytes:
        return _VarintBytes(number * 8 + 2) + _VarintBytes(len(payload)) + payload

    details = {"reason": reason, "operation": "dokime.query.list_executions", "field": "status", "limit": {"name": "rows", "value": 2}}
    entry = field(1, b"query") + field(2, json.dumps(details).encode())
    info = field(1, reason.encode()) + field(2, b"dokime.query") + field(3, entry)
    status = field(3, field(1, b"type.googleapis.com/google.rpc.ErrorInfo") + field(2, info))
    error = RpcError(
        code, "query failed", initial_metadata=(("example", "value"),), trailing_metadata=(("grpc-status-details-bin", status),)
    )

    async def scenario() -> None:
        client = DokimeClient()
        monkeypatch.setattr(client._transport, "call", AsyncMock(side_effect=error))
        reader = client.executions()
        with pytest.raises(kind) as caught:
            await anext(reader.__aiter__())
        assert caught.value.code == code and caught.value.reason == reason
        assert caught.value.details == details and caught.value.__cause__ is error
        assert caught.value.initial_metadata == error.initial_metadata and caught.value.trailing_metadata == error.trailing_metadata
        assert not reader.complete
        await client.close()

    asyncio.run(scenario())


# === Private Functions === #

# === Entry Point === #
