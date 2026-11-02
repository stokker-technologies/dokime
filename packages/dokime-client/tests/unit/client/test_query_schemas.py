# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Record Schema Discovery Contracts"""

# === Imports === #

# Standard Library Imports
import asyncio
from typing import Any
from unittest.mock import AsyncMock

# Third Party Imports
import pytest

# Local Imports
from dokime_client import DokimeClient, RecordField, RecordSchema, RecordValueType
from dokime_client.client import Backend, ProtocolError, _wire
from dokime_client.client._contracts import operation

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def catalog_page(identifier: str, *, next_id: str | None = None, revision: int = 1) -> Any:
    page = operation("Catalog", "ListRecordSchemas").response()
    _wire.copy(
        page,
        {
            "schemas": [{"record_type": identifier, "schema_version": 3}],
            "cursor": {"epoch": "epoch", "position": revision},
            "next_after_record_type": next_id,
        },
    )
    return page


def declaration(identifier: str) -> Any:
    schema = operation("Catalog", "GetRecordSchema").response()
    _wire.copy(
        schema,
        {
            "record_type": identifier,
            "schema_version": 3,
            "fields": [
                {
                    "metadata": {"identifier": "voltage.rms", "label": "Voltage"},
                    "value_type": {"Number": {}},
                    "unit": {"symbol": "V"},
                    "required": True,
                },
                {"metadata": {"identifier": "state"}, "value_type": {"Text": {}}, "nullable": True},
            ],
        },
    )
    return schema


def fake_catalog(client: DokimeClient, monkeypatch: pytest.MonkeyPatch, call: Any) -> None:
    backend = Backend("backend", "epoch", "v1", "Example", frozenset())
    monkeypatch.setattr(client._transport, "connect", AsyncMock(return_value=backend))
    monkeypatch.setattr(client._transport, "verify", AsyncMock())
    monkeypatch.setattr(client._transport, "unary", AsyncMock(side_effect=call))


def test_query_schemas_lazy_paging_and_declared_columns(monkeypatch: pytest.MonkeyPatch) -> None:
    async def scenario() -> None:
        client = DokimeClient()
        requests: list[tuple[str, Any]] = []

        async def call(service: str, method: str, request: Any) -> Any:
            assert service == "Catalog"
            requests.append((method, type(request).FromString(request.SerializeToString())))
            if method == "GetRecordSchema":
                schema = declaration(request.record_type)
                if request.record_type == "example.a":
                    _wire.copy(schema.metadata, {"identifier": "example.a", "label": "Measurement"})
                return schema
            if not request.after_record_type:
                page = catalog_page("example.a", next_id="example.a")
                _wire.copy(page.schemas[0].metadata, {"identifier": "example.a", "label": "Measurement"})
                return page
            return catalog_page("example.b")

        fake_catalog(client, monkeypatch, call)
        reader = client.record_schemas(page_rows=1)
        assert not requests
        schema = await anext(reader)
        assert isinstance(schema, RecordSchema)
        assert schema.record_type == "example.a" and schema.schema_version == 3 and schema.label == "Measurement"
        assert schema.fields == (
            RecordField("voltage.rms", RecordValueType.NUMBER, "Voltage", "V", True, False),
            RecordField("state", RecordValueType.TEXT, None, None, False, True),
        )
        assert [method for method, _ in requests] == ["ListRecordSchemas", "GetRecordSchema"]
        assert [item.record_type async for item in reader] == ["example.b"]
        second_page = requests[2][1]
        assert second_page.limit == 1 and second_page.after_record_type == "example.a"
        assert second_page.cursor.epoch == "epoch" and second_page.cursor.position == 1
        assert [method for method, _ in requests] == ["ListRecordSchemas", "GetRecordSchema", "ListRecordSchemas", "GetRecordSchema"]
        await client.close()

    asyncio.run(scenario())


@pytest.mark.parametrize("page_rows", [0, -1, True, 2**32])
def test_query_schemas_rejects_invalid_page_size(page_rows: int) -> None:
    with pytest.raises(ValueError):
        DokimeClient().record_schemas(page_rows=page_rows)


@pytest.mark.parametrize("fault", ["continuation", "revision", "order", "limit", "identity", "version", "metadata", "field", "type"])
def test_query_schemas_rejects_inconsistent_catalog(fault: str, monkeypatch: pytest.MonkeyPatch) -> None:
    async def scenario() -> None:
        client = DokimeClient()

        async def call(service: str, method: str, request: Any) -> Any:
            if method == "GetRecordSchema":
                schema = declaration(request.record_type)
                if fault == "identity":
                    schema.record_type = "example.other"
                if fault == "version":
                    schema.schema_version += 1
                if fault == "metadata":
                    schema.metadata.label = "Changed"
                if fault == "field":
                    schema.fields[0].ClearField("metadata")
                if fault == "type":
                    schema.fields[0].ClearField("value_type")
                return schema
            page = catalog_page("example.a", next_id="example.a")
            if fault == "continuation":
                page.next_after_record_type = "example.other"
            if fault == "limit":
                page.schemas.add().CopyFrom(page.schemas[0])
            if request.after_record_type and fault == "revision":
                page.cursor.position += 1
            return page

        fake_catalog(client, monkeypatch, call)
        with pytest.raises(ProtocolError):
            _ = [item async for item in client.record_schemas(page_rows=1)]
        await client.close()

    asyncio.run(scenario())


def test_query_schemas_early_close_and_empty_catalog(monkeypatch: pytest.MonkeyPatch) -> None:
    async def scenario() -> None:
        client = DokimeClient()
        fake_catalog(client, monkeypatch, lambda *args: None)
        call = client._transport.unary
        assert isinstance(call, AsyncMock)
        call.side_effect = [catalog_page("example.a", next_id="example.a"), declaration("example.a")]
        reader = client.record_schemas()
        await anext(reader)
        await reader.aclose()
        assert call.await_count == 2
        page = catalog_page("example.a")
        page.ClearField("schemas")
        call.side_effect = [page]
        assert [item async for item in client.record_schemas()] == []
        await client.close()

    asyncio.run(scenario())


# === Private Functions === #

# === Entry Point === #
