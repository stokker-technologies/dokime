# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Optional Query Materialization Contracts"""

# === Imports === #

# Standard Library Imports
import asyncio
import os
import subprocess
import sys
from typing import Any, Literal
from unittest.mock import AsyncMock

# Third Party Imports
import pytest

# Local Imports
from dokime_client import DokimeClient, QueryDependencyError, QueryLimitError, QuerySettings
from dokime_client.client.execution import Execution
from dokime_client.client.query import tabular

from .test_query_discovery import query_page
from .test_query_evidence import BACKEND, EXECUTION_ID, record_row

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def test_query_tabular_core_imports() -> None:
    script = """
import importlib.abc
import sys
class MissingExtras(importlib.abc.MetaPathFinder):
    def find_spec(self, fullname, path=None, target=None):
        if fullname.split('.')[0] in ('polars', 'pandas'):
            raise ModuleNotFoundError(fullname, name=fullname)
sys.meta_path.insert(0, MissingExtras())
from dokime_client import DokimeClient
from dokime_client.client.query import tabular
assert not {'polars', 'pandas'}.intersection(sys.modules)
reader = DokimeClient().executions(status='failed')
assert not reader.closed
import asyncio
from unittest.mock import AsyncMock
from dokime_client import QueryDependencyError
from dokime_client.client import _wire
from dokime_client.client.execution import Execution
from dokime_client.client._contracts import operation
from dokime_client.client.models import Backend
async def scenario():
    client = DokimeClient()
    execution = Execution._bind('00000000-0000-0000-0000-000000000001', client._transport,
        Backend('example', 'epoch', '1', 'Example', frozenset()))
    reply = operation('Query', 'ListRecords').response()
    _wire.copy(reply, {'rows': [{'record_id': 'row-1', 'severity': {'Info': {}}, 'details': _wire.encode({'count': 7}, 64)}],
        'schema': {'identity': 'example.event'}, 'snapshot': {'token': 'cut', 'identity': 'cut-1'},
        'limits': {'rows': 10, 'bytes': 65536}, 'complete': True})
    client._transport.call = AsyncMock(return_value=reply)
    for extra in ('polars', 'pandas'):
        query = execution.query_records('example.event')
        try:
            await getattr(query, 'to_' + extra)()
        except QueryDependencyError as error:
            assert error.extra == extra
        else:
            raise AssertionError('Missing extra was not reported')
        assert client._transport.call.await_count == 0
        rows = [row async for row in query]
        assert rows[0].details['count'] == 7 and query.complete
        client._transport.call.reset_mock()
    await client.close()
asyncio.run(scenario())
"""
    environment = dict(os.environ, PYTHONPATH=os.pathsep.join(sys.path))
    result = subprocess.run([sys.executable, "-c", script], check=False, env=environment, capture_output=True, text=True)
    assert result.returncode == 0, result.stderr


@pytest.mark.parametrize("extra", ["polars", "pandas"])
def test_query_tabular_missing_extra(extra: Literal["polars", "pandas"], monkeypatch: pytest.MonkeyPatch) -> None:
    async def scenario() -> None:
        client = DokimeClient()
        call = AsyncMock()
        monkeypatch.setattr(client._transport, "call", call)
        reader = Execution._bind(EXECUTION_ID, client._transport, BACKEND).query_records("example.event")

        def missing(name: str) -> Any:
            raise ModuleNotFoundError(name, name=name)

        monkeypatch.setattr(tabular, "import_module", missing)
        with pytest.raises(QueryDependencyError) as caught:
            await getattr(reader, f"to_{extra}")(max_rows=10)
        assert caught.value.extra == extra and f"dokime-client[{extra}]" in str(caught.value)
        assert isinstance(caught.value.__cause__, ModuleNotFoundError)
        assert call.await_count == 0 and reader._mode is None
        await reader.aclose()
        await client.close()

    asyncio.run(scenario())


@pytest.mark.parametrize("extra", ["polars", "pandas"])
@pytest.mark.parametrize("failure", ["rows", "bytes", "late_error", "none"])
def test_query_tabular_bounded(extra: Literal["polars", "pandas"], failure: str, monkeypatch: pytest.MonkeyPatch) -> None:
    async def scenario() -> None:
        client = DokimeClient(query_settings=QuerySettings(max_rows=1, max_bytes=100_000))
        execution = Execution._bind(EXECUTION_ID, client._transport, BACKEND)
        error = RuntimeError("lost evidence")
        first = query_page("ListRecords", [record_row()], after="page-2")
        last = query_page("ListRecords", [record_row("record-2")])
        call = AsyncMock(side_effect=[first, error if failure == "late_error" else last])
        monkeypatch.setattr(client._transport, "call", call)
        reader = execution.query_records("example.event")
        if failure == "none":
            library, rows = await tabular.materialize(reader, extra, max_rows=2)
            assert library.__name__ == extra and [row.id for row in rows] == ["record-1", "record-2"]
            assert reader.complete
        else:
            with pytest.raises(RuntimeError if failure == "late_error" else QueryLimitError):
                await tabular.materialize(
                    reader, extra, max_rows=None if failure == "rows" else 2, max_bytes=1 if failure == "bytes" else None
                )
            assert not reader.complete
        assert reader.closed and call.await_count == (1 if failure == "bytes" else 2)
        await client.close()

    asyncio.run(scenario())


def test_query_tabular_broken_extra(monkeypatch: pytest.MonkeyPatch) -> None:
    async def scenario() -> None:
        client = DokimeClient()

        def broken(name: str) -> Any:
            raise ModuleNotFoundError("internal dependency", name="internal_dependency")

        monkeypatch.setattr(tabular, "import_module", broken)
        reader = client.executions()
        with pytest.raises(ModuleNotFoundError) as caught:
            await tabular.materialize(reader, "polars")
        assert not isinstance(caught.value, QueryDependencyError) and reader._mode is None
        await reader.aclose()
        await client.close()

    asyncio.run(scenario())


# === Private Functions === #

# === Entry Point === #
